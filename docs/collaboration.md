# Real-time Collaboration Design Document

This extends design.md with real-time collaboration: several people and AI agents editing
the same board at the same time. The SPA, the server, and the shared core described here
are to be rebuilt together as a monorepo; this document, with tech-stack.md and
design-guidelines.md, is the complete record of the design they follow and the reasoning
behind it. Technology choices for the server are recorded in tech-stack.md; this document
states the design decisions they follow from.

## Context

design.md makes the URL the complete save form of a board and deliberately holds no state
on a server. Sharing today is asynchronous: someone edits, copies the URL, and hands it
over. An LLM takes part the same way — it is given DSL text and returns DSL text, which a
human pastes back in.

Two kinds of participants now want to work on a board at the same time. People running an
event storming session together want to see each other's notes appear as they are placed,
and to point at things with the cursor while explaining. AI agents want to read the
current board and write to it directly, without a human relaying text in both directions.

## Motivation

The value of event storming comes from discovery happening in the room: someone places an
event, someone else questions it with a hotspot, a third person adds the missing command.
Passing a URL back and forth serializes that into turns and loses the conversation.

For AI agents, the relay through a human is the bottleneck. An agent that can read and
write the live board over plain HTTP can join the session as a participant: propose a
flow, fill in read models, or flag inconsistencies while the humans keep working.

What must not be lost is the property design.md is built on: the URL is the save form,
and the tool needs no accounts and no database. Collaboration is added as a live session
laid over that model, not as a replacement for it.

## Goals

- Several people can edit the same board simultaneously, and each change reaches the
  others within well under a second.
- Presence: remote cursors smooth enough to explain something by pointing, remote
  selections, an indicator for a note someone is editing, and a participant list.
- AI agents participate as first-class participants over plain HTTP, with no persistent
  connection required. Wrapping the API as an MCP server is a thin translation.
- A single HTTP API, fully described by OpenAPI, is shared by the browser, AI agents, and
  any other application. There is no second protocol (no WebSocket, no RPC framework).
- AI agents join by answering a challenge in which they explicitly declare themselves as
  agents, and are shown as such to everyone in the session.
- The URL remains the complete save form. The server holds only the live session, in
  memory, and forgets it once nobody is connected.
- Stateless endpoints render a board given by URL as raw DSL, SVG, or PNG.
- Without the server, the SPA keeps working exactly as it does today. Collaboration is a
  progressive enhancement.

## Non-Goals

- **Server-side persistence, accounts, and access control beyond the link**: the server
  never stores a board beyond the life of a session. Anyone holding a session link can
  join it; there are no users, owners, or permissions. Restricting who can reach the
  server at all (e.g. to one company) is left to the hosting platform in front of it (see
  Deployment and access control).
- **Offline editing and merging long-diverged copies**: a session always has a live
  server. Editing while disconnected and reconciling hours of divergent work is not
  supported. A disconnected client falls back to being a single-user board.
- **Character-level concurrent editing inside one note**: note text is short. Two people
  typing into the same note at the same time is avoided by an editing indicator, not
  merged.
- **Proving that a participant is an AI**: the challenge is a declaration, not a
  verification (see below).
- **Voice, video, and chat**: people are expected to talk over whatever call tool they
  already use.
- **Horizontal scaling across instances**: a single server instance holds all live
  sessions at first. How to go beyond that is left open (see Concerns).

## Solution

### Three layers of shared state

"Concurrent editing" is three problems with different characteristics, and each gets its
own mechanism.

| Layer | Examples | Mechanism |
|---|---|---|
| Board state | notes, committed positions, edges, title | Ordered operations applied by the server |
| Presence | cursors, selections, a note being dragged or edited | Broadcast only; never stored or replayed |
| Text inside a note | two people typing into the same note | Avoided by an editing indicator; last write wins |

The logical state of design.md (notes, edges, title) is exactly the first layer. Presence
never enters it, just as rendering values such as arrow control points never do.

### Board state: server-authoritative operations

The server holds the canonical board for a session and applies operations to it one at a
time. Each applied operation gets a sequence number and is broadcast to every participant.
This is the same structure as an authoritative game server, and the one most whiteboard
tools use; it is chosen over a CRDT for reasons given in Alternative Solutions.

Operations are domain operations, not generic patches:

| Operation | Effect | Conflict rule |
|---|---|---|
| `addNote` | add a note of a type at a position with text | ID collision resolved by the server (below) |
| `moveNotes` | set absolute positions of one or more notes | last write wins per note |
| `setText` | set a note's text | last write wins per note |
| `connect` | add an edge; dashed is derived from the rules | rejected if an endpoint no longer exists |
| `disconnect` | remove an edge | no-op if already gone |
| `deleteNotes` | remove notes and every edge touching them | no-op for notes already gone |
| `setTitle` | set the board title | last write wins |

A paste or a DSL replacement (see HTTP API) becomes a batch of these operations applied
atomically under a single sequence number.

**Note IDs are allocated by the server.** IDs are human-readable and sequential per type
(`E1`, `E2`, ...) because they appear in the DSL. Two clients adding a domain event at the
same moment would both pick `E3`. The client proposes an ID from its own view; the server
keeps it if it's free and otherwise assigns the next free one and broadcasts the operation
with the final ID. The originating client rewrites that ID in its pending operations.

**Edge integrity is checked by the server.** A `connect` that arrives after one of its
endpoints was deleted is rejected, and `deleteNotes` removes the edges touching the
deleted notes in the same step. The board can therefore never hold an edge that points at
a missing note, which the DSL could not express.

**Clients apply optimistically and reconcile.** A client applies its own operation to its
local board immediately and keeps it in a pending queue, tagged with a client operation
ID. The local board is always "the last confirmed board plus the pending operations."
When an operation arrives from the server, the client applies it to the confirmed board,
drops its own pending operation if the incoming one is its echo, and re-applies the rest
of the queue on top. A rejected operation is simply dropped from the queue. Because the
same apply function runs on both sides (see Shared core), the client's confirmed board
always equals the server's.

**Drags are presence until the drop.** While a note is being dragged, its position is
broadcast as presence (below), not as operations. Releasing it produces one `moveNotes`.
The operation log stays small, and a reconnecting client only needs the final position.

**Undo is per participant.** Each client undoes only its own operations, by sending their
inverses as new operations. An inverse whose target has since been deleted by someone
else is skipped. Undo never rewinds the shared history.

**Text is guarded by an editing indicator.** Entering text edit on a note publishes an
"editing" presence for it. Other clients show who is editing and don't open that note for
editing until the indicator clears. The server itself stays last-write-wins; the indicator
is advisory, which is enough for text as short as a sticky note's.

### Presence

Presence is ephemeral. It is broadcast to the other participants, never written to the
operation log, and not replayed to a client that reconnects later (a reconnecting client
gets only the current snapshot of who is present).

**Coordinates are board coordinates.** Each viewer pans and zooms independently, so a
cursor position in screen pixels means nothing to anyone else. Cursors, drag positions,
and selection rectangles are sent in the canvas (world) coordinate system and each viewer
projects them through its own pan/zoom.

**Cursors are recorded, batched, and replayed.** The sender samples its cursor at display
rate (about 60 Hz) and sends the samples, each with a relative timestamp, in one request
every 100–200 ms, and only while the cursor is moving. Receivers play the samples back on
their own timeline with a fixed delay of about 200 ms. The effect is the actual path the
sender's cursor took, rather than a line interpolated between sparse points, at 5–10
requests per second from an active sender. A delay of this size is the same order as a
voice call's and goes unnoticed when someone is explaining by pointing.

A cursor that has not moved for a while is faded out. An AI agent has no cursor; instead,
its presence can name the notes it is working on, which viewers highlight.

### Transport: SSE down, HTTP POST up

Everything from the server to a participant goes over one Server-Sent Events stream;
everything from a participant to the server is an ordinary HTTP request. There is no
WebSocket.

- **One API for everyone.** A browser is "a participant that also holds an event stream";
  an AI agent is "a participant that only sends requests" (and may open the stream if it
  wants to watch). The OpenAPI document describes the whole surface, including the event
  stream.
- **Resumption comes for free.** Each operation event carries its sequence number as the
  SSE `id`. When the stream drops, the browser reconnects on its own and sends
  `Last-Event-ID`; the server replays operations after that number from its in-memory log,
  or sends a fresh snapshot if the log no longer reaches back that far. Presence events
  carry no `id` and are not replayed.
- **HTTP/2 is required between the browser and the front end.** It multiplexes the event
  stream and the stream of POSTs over a single connection and compresses their headers.
  Under HTTP/1.1 the browser's per-origin connection limit would be hit. The platform's
  front end (Cloud Run, a load balancer) provides this to browsers; the container itself
  may speak HTTP/1.1 behind it.
- **Ordering of POSTs.** Separate requests are not guaranteed to arrive in order. Cursor
  batches carry a sender-side sequence and stale ones are dropped; operations carry client
  operation IDs and a per-client sequence so the server applies them in the order sent.

### Sessions (rooms) and their lifecycle

A session ("room") is the live, in-memory state of one board being edited together: the
board, its sequence number, a bounded log of recent operations, and the participants.

- **Room IDs are capabilities.** A room ID is 128 random bits, base64url-encoded. Knowing
  it is what lets you join. The share URL carries both the board and the room:
  `?data=<encoded>&room=<id>`.
- **Starting a session.** "Share live" on a board sends its current DSL to the server,
  which creates a room and returns its ID. The SPA adds `room` to its URL.
- **The URL keeps working as a save form.** Every participant's SPA keeps updating
  `data` in its own URL after every change, exactly as today. At any moment, any
  participant's URL is a complete copy of the board.
- **Liveness.** A room stays alive as long as any browser has the board open, which the
  server sees as an open event stream. When no stream is open, a single fixed grace
  period starts (initially 5 minutes); any request to the room restarts it. A room with no
  open stream and no request for the whole grace period is discarded. The grace period
  covers a reload, a network drop, and the server closing long-lived requests, and it
  keeps a room alive while an AI agent, which holds no stream, is still working on it.
  An agent whose room has expired gets `404` and revives it from the share URL it was
  last given (below).
- **Opening an expired link revives it.** Opening `?data=...&room=<id>` for a room that
  no longer exists recreates it under the same ID from the link's `data`. Several people
  opening the same old link therefore land in the same new session. If two people open
  links with different `data` for the same expired room, the first to arrive wins and the
  second sees the first one's board.
- **Nothing written by an agent is lost silently.** Every write response to an agent
  includes the current share URL with `data`, so an agent that is the last participant
  still ends up holding the board.

### Participants and the AI agent challenge

A browser joins with a display name and receives a participant token of kind `human`.

An AI agent joins through a challenge:

1. The agent requests to join. The server answers `401` with a challenge: a nonce, the
   fields the agent must declare, and a usage guide — the DSL grammar, the connection
   rules, and the endpoints it will need.
2. The agent sends back its declaration (agent name, model, operator) with the nonce.
3. The server issues a short-lived participant token of kind `agent`.

Agents are marked as agents in the participant list and next to anything they are
editing, and have their own rate limits.

**How the token travels.** `EventSource` cannot set request headers, so a browser's
participant token is set as an HttpOnly cookie scoped to the room's path when it joins,
and the browser sends it with the event stream and every request alike. This is simple
because the SPA and the API share one origin (see Deployment and access control). Agents
send their token in the `Authorization` header as a bearer token.

This is a declaration, not proof. Nothing stops a person from using the agent flow, or an
agent from driving a browser. Its value is elsewhere: it makes self-identification an
explicit, required step rather than an optional header, it gives every session a clear
record of which participants said they are agents, and it guarantees that an agent has
been handed the grammar and rules before it writes anything. If cryptographic
verification of agents is wanted later, a signed-request scheme (e.g. HTTP Message
Signatures, as in Web Bot Auth) can be added to the same flow.

### HTTP API surface

All endpoints are described in a single OpenAPI document (see "Where the specification
lives" below). The sketch here fixes the shape; the OpenAPI document pins down payloads.

| Method and path | Purpose |
|---|---|
| `POST /rooms` | Create a room from DSL; returns the room ID |
| `POST /rooms/{id}/participants` | Join as a human |
| `POST /rooms/{id}/agents` | Join as an AI agent (challenge, then declaration) |
| `GET /rooms/{id}` | Current board as DSL; `ETag` is the sequence number |
| `PUT /rooms/{id}` | Replace the board with DSL, conditional on `If-Match` |
| `POST /rooms/{id}/ops` | Submit a batch of operations |
| `POST /rooms/{id}/presence` | Submit cursor samples, selection, editing, focus |
| `GET /rooms/{id}/events` | SSE stream of operations, presence, and participants |
| `GET /raw?data=` | The DSL for a board URL, as `text/plain` |
| `GET /render.svg?data=`, `GET /render.png?data=` | The board rendered as an image |
| `GET /rooms/{id}/render.svg`, `.png` | The live board rendered as an image |

**Whole-DSL replacement is the main path for agents.** LLMs are much better at rewriting
a text document than at composing a sequence of fine-grained operations. `PUT` takes the
full DSL together with the version it was based on. The server parses it, diffs it
against the current board by note ID, and applies the difference as one batch of
operations, which the browsers see as ordinary changes. If the board has moved on since
the given version, the server answers `412` with the current DSL and version, and the
agent retries on top of it. `POST /ops` remains for agents that prefer precise edits, and
is what the browser uses.

**MCP** is left for later. An agent can use the HTTP API directly, and an MCP server,
when wanted, is a thin wrapper over these endpoints (read board, replace board, apply
operations, render), with the challenge performed once when it connects. Nothing in the
API is shaped for MCP in advance.

### Where the specification lives

The API is schema-first: the OpenAPI document is written by hand before the code and is
the source of truth for every message. The server's HTTP layer is generated from it (see
tech-stack.md), which is what keeps implementation and specification from drifting. What OpenAPI
cannot express has a defined home of its own, so nothing needs a separate prose protocol
specification:

| What | Where |
|---|---|
| Endpoints, request and response bodies, headers (`If-Match`, `ETag`, `Last-Event-ID`) | OpenAPI |
| Operations, as one type discriminated by a `type` field | OpenAPI (JSON Schema) |
| Event stream events (operation, snapshot, presence, participant joined/left) | OpenAPI, as schemas under `components/schemas` referenced from the stream's description |
| Errors | OpenAPI, as RFC 9457 `application/problem+json` |
| Call sequences (challenge, `412` then re-read and retry, resume with `Last-Event-ID`) | This document, summarized in the OpenAPI operation descriptions |
| Meaning of operations (conflict rules, ID reassignment, cascade on delete, batches) | The shared core's apply function and its tests |
| Client-side reconciliation (pending queue, rebase, undo) | This document; implemented once in the SPA |
| DSL grammar | A grammar document (below) |

The meaning of operations is deliberately not written as prose. Because the server and
the SPA run the same core, its apply function and its test cases are the specification;
a prose copy could only fall out of date.

The DSL grammar, on the other hand, exists today only in the prototype and in the parser.
It needs a short grammar document (EBNF, the escaping rules, coordinates, and the
connection rules table) before the server is built, because it is what the agent
challenge hands to an agent, and what any other application writing DSL has to follow.

The operation submission response reports, per operation, whether it was applied, applied
with a reassigned note ID, or rejected. Applied operations additionally arrive through the
event stream like everyone else's, which is what the client reconciles against.

### Stateless endpoints

`/raw` and `/render.*` take nothing but the `data` value from a board URL, so they need no
room and cache perfectly: the same `data` always yields the same bytes. They are served
with long-lived immutable caching. They also make a per-board image available for the OGP
`og:image`. A `POST` variant accepting DSL in the body covers boards whose URL is too long
for a `GET`.

Rendering PNG on the server means fonts must be bundled with it; note text is usually
Japanese, and without a Japanese font the text would not render.

### Shared core

The server needs the domain logic that today lives only in this SPA: the DSL parser and
serializer (for `GET`/`PUT` of DSL and for `/raw`), the connection rules (to derive
dashed edges and validate operations), SVG generation (for `/render.*`), and the codec.
Operation application must additionally behave identically on the client and the
server, or optimistic application would drift from the confirmed board.

This logic moves into a single core library, written once and used in two builds: natively
by the server, and compiled to WebAssembly for the SPA. The core contains the board model
and operation application, the DSL, the connection rules, and SVG generation. UI-only
state — selection, viewport, drag in progress, the pending queue — stays in the SPA's
TypeScript, so the SPA's structure (signals feeding components) is unchanged.

The codec is the exception: it is only deflate-raw plus base64url, a fixed specification.
The browser keeps using the standard Compression Streams API, and the server uses a native
implementation, rather than shipping a deflate implementation inside the WebAssembly
build. Both sides are checked against the same test vectors.

### Deployment and access control

**The server also serves the SPA.** A collaborative deployment is a single container: the
server answers the SPA's static files, the API, the event stream, and the stateless
endpoints, all from one origin. The SPA remains an SPA — rendering still happens entirely
in the browser — only where its files come from changes. This removes CORS altogether,
which matters most behind an authenticating proxy, where cross-origin preflight requests
are typically blocked. It also makes the same-URL `f=raw` negotiation of design.md
possible, since the server sees every request for the app's URL.

The SPA can still be deployed on its own to static hosting (e.g. GitHub Pages). Such a
deployment has no collaboration: it is the single-user tool it is today. A statically
hosted SPA does not talk to a server on another origin.

**Access control is entirely the platform's job.** An internal deployment — for example,
Cloud Run with Identity-Aware Proxy (IAP) — restricts who can reach the service before a
request ever arrives at the server. The server contains no authentication code and behaves
identically with or without it. Within a deployment, the room ID remains the capability
to join a particular session.

The server does not use the identity such a proxy can forward (for IAP, the authenticated
user's email in a signed header). Display names stay self-declared, as they are in a
public deployment. If verified names are wanted later, the server would have to verify the
proxy's signed assertion, not just read a header, and this would be the first piece of
the server that knows about a specific platform.

Behind an authenticating proxy, some things change outside the server:

- **Session expiry breaks the event stream.** When the proxy's login session expires, the
  stream's automatic reconnect is answered with a redirect to a login page, which
  `EventSource` cannot follow. The SPA treats repeated stream errors as a cue to check with
  a plain request and, if it is no longer authenticated, reloads the page to go through
  login again.
- **AI agents need platform credentials.** An agent must present whatever the proxy
  accepts (for IAP, an OIDC ID token, typically for a service account) on every request.
  The challenge still applies on top: the platform says who is calling, the challenge says
  that the caller is an agent.
- **Link previews stop working.** Crawlers that unfurl links (chat tools, social sites)
  can't pass the proxy, so the `og:image` rendering is unavailable in an internal
  deployment. The same applies to `/raw` and `/render.*` for anything outside the
  organization, which for an internal deployment is the intent.

### Changes to the SPA

- A sync module: joining a room, the `EventSource` stream, the pending queue and
  reconciliation, and sending operations and presence with `fetch`. No new library.
- Handling of a lost login session behind an authenticating proxy (see above).
- Rendering of presence: remote cursors, selections, editing indicators, and a
  participant list (visual rules in design-guidelines.md).
- A "Share live" action that creates a room and a `room` parameter in the URL.
- The domain logic now in `board`, `dsl`, and `imageExport` models is replaced by calls
  into the WebAssembly core.
- When served statically, or with no `room` in the URL, the SPA behaves exactly as it
  does today.

## Alternative Solutions

### CRDT library (Yjs, Automerge, Loro)

**Pros**: a mature sync protocol, reconnect handling, awareness (presence), and undo come
ready-made. Concurrent typing inside a note would merge character by character.
**Cons**: the defining strength of a CRDT — converging without any central authority,
after arbitrarily long divergence — goes unused, because a session always has a live
server and offline editing is a Non-Goal. The problems this board actually has aren't
solved by one: sequential human-readable IDs collide under concurrent adds (in a CRDT map,
one note silently overwrites the other), and an edge can be left pointing at a concurrently
deleted note. Both need a central decision anyway. It would also add a client dependency of
tens of kilobytes against the dependency policy, and constrain the server language to one
with a compatible implementation.
**Decision**: server-authoritative operations. If character-level merging inside a note
is ever required, a CRDT text type can be adopted for note text alone.

### WebSocket

**Pros**: bidirectional and ordered on one connection; the most efficient transport for
high-frequency cursor updates.
**Cons**: a second protocol next to the HTTP API. It cannot be described by OpenAPI, needs
its own handling in the infrastructure, and gives AI agents a different path from humans.
The cost of holding a connection open is the same as an SSE stream's.
**Decision**: SSE plus POST. The only thing WebSocket does better is cursor upload
efficiency, and batching cursor samples brings that down to a few requests per second.

### RPC streaming (Connect, gRPC-Web)

**Pros**: a typed schema and generated clients.
**Cons**: browsers can only do server-to-client streaming with it, which is exactly what
SSE already provides. It adds a client dependency and a protobuf toolchain, and AI agents
and MCP are better served by plain JSON and text.
**Decision**: plain HTTP with OpenAPI.

### Send each cursor position as it happens

**Pros**: the lowest latency, and nothing to schedule on the receiver.
**Cons**: 30–60 requests per second per moving cursor, and the receiver still has to
interpolate between points that arrive with jitter.
**Decision**: batch samples every 100–200 ms and replay them with a fixed delay. Fewer
requests, and a more faithful path.

### Agents write only through fine-grained operations

**Pros**: no DSL parser on the server, no diffing, no `412` retries.
**Cons**: gives up the form LLMs handle best. Composing correct operation sequences with
the right IDs is error-prone for an LLM, while rewriting a short text document is not.
**Decision**: offer both, with whole-DSL replacement as the primary path for agents.

### Store sessions persistently

**Pros**: a session survives restarts and can be reopened after everyone leaves.
**Cons**: contradicts the premise of design.md that the URL is the save form, and brings
back everything the tool has avoided: a database, retention, and ownership of data.
Every participant already holds the board in their URL.
**Decision**: memory only. Revival from the link's `data` covers reopening an expired
session.

## Concerns

- **Scaling beyond one instance.** All participants of a room must reach the process that
  holds it. A single instance avoids the question at first. Going beyond it requires
  routing requests by room ID (consistent hashing in front of the instances) or a
  platform with per-key singletons; which to use is left until it's needed.
- **Server-side request duration limits.** Hosting platforms close long-lived requests
  (e.g. after 60 minutes). SSE resumption covers this, but the reconnect must be verified
  to be seamless, including presence recovering promptly.
- **Size of the WebAssembly core.** The first render of a URL waits for the core to load
  and parse. The core must avoid heavy dependencies (a regex engine, a general
  serialization framework) and have a size budget that is measured in CI.
- **Cost of the JavaScript–WebAssembly boundary.** Each operation crosses the boundary
  with its board data. At the expected scale of tens to a few hundred notes this should be
  negligible, but it should be measured on a large board.
- **IAP on Cloud Run with long-lived streams.** Whether IAP in front of Cloud Run passes an
  SSE stream through untouched for its full duration, and exactly how an expired login
  session shows up to `EventSource`, needs to be verified on the real platform.
- **Abuse of open rooms.** Anyone with a link can write. Rate limits per participant,
  limits on board size and participant count, and the unguessability of room IDs are the
  only defenses. Room IDs appear in URLs and therefore in logs and Referer headers, as the
  board itself already does.
- **Participant colors on a colored board.** Cursors are colored per participant, as an
  explicit exception to "only notes carry color" (see design-guidelines.md). Whether they
  stay distinguishable from note types on a busy board needs to be checked with a
  realistic number of participants.
- **Reconciliation of the auto-connect on adjacency.** Automatic connection is inferred
  when a note is placed. If two people place notes next to the same note concurrently,
  both inferred edges are valid operations and both are kept. Whether this matches user
  expectations needs to be checked in use.
