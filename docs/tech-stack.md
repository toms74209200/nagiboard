# Event Storming Board Technology Stack

This describes the technology choices for realizing the design decided in design.md.
This document is not meant to enumerate adopted technologies (listing framework names
means nothing without showing how they follow from the requirements). It focuses on, and
records the reasoning for, the places where a real decision is needed: rendering
approach, hosting, and which dependencies to take on or leave out.

The premise follows design.md: the core is a static SPA that holds state in the URL,
aimed at a single local user. `f=raw` is an extension point not included in the core,
handled, if needed, by a co-located stateless edge layer. The core and the edge layer are
orthogonal and don't constrain each other's runtime choice.

Real-time collaboration (collaboration.md) adds a server and a shared core library. The
SPA, the server, and the core are to be rebuilt together as a monorepo. Their technology
choices are recorded under "Collaboration server" below; the SPA's own choices are
unchanged apart from the domain logic moving into the shared core.

## Framework

The core uses Preact + Signals (decided). The fact that state lives in the URL makes
initial-load weight matter; fine-grained updates that touch only what changed, across many
notes and arrows, matter; and the reactive flow we want — "operate -> update a signal ->
an effect calls `replaceState` on the URL" — can be written directly. No further
justification is needed.

## Rendering approach: DOM notes + SVG arrows

Notes are absolutely-positioned DOM elements; arrows are SVG `<path>` elements (drawn as
beziers between anchors); and the layer containing both is panned/zoomed via
`transform: translate + scale`. This follows the prototype's structure.

Making each note a DOM element lets text editing (contenteditable), caret and IME,
focus, click hit-testing, and accessibility all be delegated to the browser's existing
features. The straightforward mapping — note = component, arrow = SVG path component —
also dovetails directly with Preact/Signals' fine-grained updates. An arrow's path
(which edge it connects to and its control points) is simply derived from the positions
of its two endpoints, requiring no drawing library.

### Alternative: Canvas / WebGL

**Pros**: rendering doesn't break down even at thousands to tens of thousands of
elements; avoids any DOM-node-count bottleneck.
**Cons**: an event storming board fits within tens to a few hundred elements per session,
nowhere near this threshold. Adopting Canvas means reimplementing, from scratch, things
the browser provides for free: text editing, hit-testing, accessibility, text wrapping,
IME. Recreating the prototype's experience of double-clicking a note to edit its body
directly would not be worth it on Canvas.
**Decision**: go with DOM + SVG. For the expected scale, Canvas/WebGL is overkill — the
implementation cost it adds outweighs the rendering performance it buys. Revisit if signs
emerge that the scale will grow by orders of magnitude.

## Hosting and the edge layer

The core only needs to be deliverable as static assets and doesn't pin a hosting target
(written neutrally, on the premise that an edge platform may be adopted). Only if `f=raw`
is enabled is an edge middleware layer needed — one that runs before the request reaches
the asset. Candidates include Cloudflare Pages Functions and Vercel/Netlify edge
middleware, all of which allow co-locating a stateless function with a static SPA.

raw is implemented by middleware performing content negotiation on `&f=raw` appended to
the same URL as the core (raw DSL as `text/plain` for clients without JS, the normal
static SPA for browsers). We accept the resulting line: pure static hosting without a
middleware layer (e.g. GitHub Pages) cannot serve raw. This corresponds to the Non-Goals
and extension-point description in design.md.

With the collaboration server, that server is this layer: it serves the SPA's files and
answers `&f=raw` itself, so no separate edge function is needed in a deployment that
includes it. The edge-function route remains relevant only for a static deployment that
wants raw without collaboration.

## Dependency policy: zero dependencies in principle

Since state lives in the URL, the weight of the initial load is directly a functional
requirement. Dependencies are therefore actively avoided. Only what the requirements
genuinely demand is added, kept to a minimum.

### Compression: standard Compression Streams by default

Since state is carried in the URL, the raw DSL is too long as-is and must be compressed.
The compression algorithm is DEFLATE/zlib (RFC 1950/1951) — a fixed, settled spec — and
both browsers and edge runtimes ship a standard implementation as `CompressionStream` /
`DecompressionStream`. Given that the same standard spec is callable directly, there's no
reason to carry a library (e.g. pako) that's just one implementation of it, and no
external requirement calls for one either. So standard Compression Streams are the
default, keeping the dependency count at zero. pako is positioned only as a "fallback for
a runtime where the standard API isn't available."

That said, because the codec must behave identically as pure functions across the browser
and the edge, the two sides must agree on the compression format string (fix which of
`deflate` / `deflate-raw` / `gzip` is used). This isn't something that goes away by not
adopting pako — it's an implementation convention: precisely because we use the standard
API, we fix which format to align on.

### base64url conversion

Turns the compressed byte sequence into a URL-safe string. `btoa` plus a few character
substitutions fit in a handful of lines and need no library.

### What we don't bring in

- **State management library**: Signals suffice. Redux etc. are unnecessary.
- **Routing library**: a single page, where state is just read/written to/from URL
  parameters by hand. No router needed.
- **UI component library**: notes, arrows, and the palette are all custom-drawn, leaving
  no role for a general UI kit. Bringing one in would bloat the bundle, working against
  the goal of a lightweight, URL-centric SPA.
- **Arrow/bezier drawing library**: hand-computed SVG paths (pick the nearest pair of
  edges between two notes and place control points) suffice.

### DSL parser: keep it hand-written

The grammar is just two kinds of lines — declaration lines (type, ID, body, coordinates)
and connection lines (`->` / `..>`) — simple enough that a parser generator (e.g. peg.js)
isn't worth bringing in. Continue with the prototype's hand-written parser. If more
careful error-position reporting is wanted later, there's room to refactor into a small
tokenizer. When the parser moves into the shared core (see "Collaboration server"), it
stays hand-written, now also to keep the WebAssembly build small.

## Collaboration server

The design is in collaboration.md. What follows are the choices it leaves to technology,
and their reasoning. The server's workload is small and specific: hold a few hundred
small boards in memory, keep one long-lived event stream open per browser, apply
operations one at a time per room, and render images on request. It is not a
general-purpose backend.

### Language: Rust

The server shares its domain logic with the SPA through a core library compiled to
WebAssembly (below), and Rust is the language with the most mature path to WebAssembly
without a runtime. Rendering PNG is the other deciding factor: resvg is the most complete
SVG rasterizer available as a library, including text layout with bundled fonts, and it is
native Rust. Memory per open event stream and per room is small and predictable, which
keeps the cost of idle sessions low.

The web layer is axum on tokio: HTTP/2, streaming responses for SSE, and nothing more is
required of it.

#### Alternative: Go

**Pros**: simple concurrency, fast builds, a very mature HTTP/2 and SSE story.
**Cons**: WebAssembly output carries the Go runtime and is far too heavy for the SPA's
initial load, so the domain logic would have to be maintained twice. SVG rasterization
with proper text rendering is weak in its ecosystem.
**Decision**: Rust.

#### Alternative: Elixir (Phoenix Channels/Presence)

**Pros**: a process per room and presence tracking are what the platform is built for.
**Cons**: brings a third language for the domain logic, and needs a native extension for
rendering anyway. Its strongest feature, clustering across nodes, solves a problem a
single instance doesn't have yet. Phoenix Channels also assume WebSocket, which
collaboration.md decides against.
**Decision**: Rust, with a task per room giving the same structure (below).

#### Alternative: TypeScript on Cloudflare Workers with Durable Objects

**Pros**: the existing TypeScript domain code runs as-is; a Durable Object is a per-room
singleton, which solves room routing across instances by construction; idle connections
cost almost nothing.
**Cons**: ties the server to one platform's programming model, where the preference is
container hosting (Cloud Run or ECS) that can be moved freely.
**Decision**: Rust in a container. Durable Objects remain the fallback if routing rooms
across many instances becomes necessary and building it ourselves looks too costly.

### Rooms: one task per room, in memory

Each room is a tokio task that owns its board and receives every request for that room
through a channel. Operations for a room are therefore applied strictly one at a time
without locks, the same structure as a process per room in Erlang/Elixir. The only shared
structure is the map from room ID to channel.

State is plain in-memory data. No database, including an in-memory SQLite: a room's state
is one board, a sequence number, a bounded operation log, and a participant list, and
nothing queries across rooms. A database would add a schema and a serialization step for
no capability that is used.

### Hosting: one container instance

Cloud Run with the maximum instance count set to 1, or a single ECS task. One instance
guarantees that all participants of a room reach the process holding it; see
collaboration.md for what happens beyond that. Browsers get HTTP/2 from the platform's
front end; the container can speak HTTP/1.1 behind it, since each open event stream is
its own request to the container either way (end-to-end h2c is optional). The instance's
concurrency limit bounds the number of simultaneously open event streams.

The same container serves the SPA's built static files (see collaboration.md), so a
deployment is one service. For internal use, access is restricted by the platform — on
Cloud Run, its Identity-Aware Proxy integration — with no change to the server. The
server has no authentication code and no configuration for it.

Keeping a stream open keeps the instance billed, whether it is SSE or WebSocket, so the
transport choice doesn't change the cost of holding sessions. What SSE plus POST adds is
the request count from cursor batches (5–10 per second per moving cursor), which at
request-based pricing is a few cents per participant-hour at most.

### API specification: OpenAPI 3.1

One OpenAPI document describes every endpoint, including the event stream. The document
is schema-first — written by hand before the code, and the source of truth — rather than
generated from the server's code, because the API is reviewed as a design before it is
implemented. What the document does not cover, and where that lives instead, is set out
in collaboration.md.

The server's HTTP layer is generated from the document with OpenAPI Generator's
`rust-axum` generator: routing, request parsing and validation, models, and one trait per
tag, which `mast/` implements. The generated crate is a build artifact (`mast/openapi/`,
not committed), produced from the committed bundle (see below). Implementing the
generated traits is what keeps the server from drifting from the document: an endpoint or
payload that changes in the document no longer compiles until the server follows.

The event stream is the exception. SSE needs no special schema — it is a `GET` whose
response is `text/event-stream` — so the document declares it as a string response, and
the payload of each event (operation, snapshot, presence, participant) is an ordinary
schema under `components/schemas`, referenced from the operation's description. The
generator turns those schemas into models, but has no way to return a stream from a
generated handler, so the event stream's route is written by hand and merged into the
generated router.

#### Alternative: OpenAPI 3.2

**Pros**: 3.2 can bind a schema to each item of a `text/event-stream` response
(`itemSchema`), so which events a stream carries is stated formally instead of in a
description.
**Cons**: OpenAPI Generator (7.25) can't read a 3.2 document at all, and its 3.1 support
rejects `itemSchema`; the generated handler couldn't return a stream even if it could.
Using 3.2 would mean down-converting the document before every generation for a gain
that is documentation only.
**Decision**: 3.1. The event payloads remain machine-readable schemas; only the formal
link from the stream to them is lost.

### API tooling: Redocly CLI

The document is split into files and linted and bundled with Redocly CLI, one tool for
both. The bundle (`spec/dist/openapi.yaml`) is committed: it is the single artifact every
consumer reads — the server's code generation now, API tests and any client-side
generation later — so that each of them needs neither Redocly nor a bundling step of its
own. CI checks that the committed bundle matches the sources. Generated code is not
committed; the server's generated crate is cached in CI keyed on the bundle and the
generator version, so it is regenerated only when either changes. Project-specific rules are written as Redocly configurable rules, or as plugin rules
where an assertion can't express them. Spectral was the alternative linter; it was
checked while 3.2 was still the plan and rejected for validating 3.2 against the 3.0
schema, and with 3.1 there is no reason left to add a second tool next to the bundler.

The SPA's client code is written against it by hand (it is a handful of calls, and
generated clients would add a dependency). Connect/gRPC was considered and rejected in
collaboration.md. An MCP wrapper is out of scope for now; the HTTP API is usable by
agents directly.

### Repository layout: a monorepo

The SPA, the server, the core, and the OpenAPI document live in one repository, so that a
change to the core, the API, and both of their consumers is a single change:

```
keel/          the core. Rust crate: board model, operations, DSL, rules, SVG (native + WASM)
mast/          the server. Rust: axum server, rooms, rendering
deck/          the SPA (Preact), consuming keel's WASM build
spec/          the OpenAPI document, schema-first
mast-tests/    API tests against the server
docs/          these documents, plus the DSL grammar
```

The application directories are named after the parts of a ship, ordered by distance from
the user: people stand on the deck, the mast stands in its middle for everyone, and the
keel runs unseen beneath both — the mast is stepped on the keel, as the server is built on
the core. The names are deliberately kept apart from the domain's own vocabulary (board,
room, note), so that a directory is never mistaken for a concept in the code. A future
native client is another place people stand on the ship (e.g. `bridge/`).

Each directory is an independent project with its own lockfile and CI, triggered only by
changes to that directory; there is no Cargo workspace at the root. `mast/` depends on
`keel/` by path, and the WASM build of `keel/` is consumed by `deck/` as a local build
artifact, with no package publishing in between. A change upstream reaches a downstream
project's CI when that project is next changed: an incompatible change to `keel/`'s API
comes with the fix to its callers in the same pull request, and a change to the API
specification is merged first and implemented in `mast/` afterwards.

### Rendering: resvg with bundled fonts

`/render.png` turns the SVG produced by the shared core into PNG with resvg. A Japanese
font (a subset of Noto Sans JP) is bundled into the server image, because note text is
usually Japanese and the container has no system fonts to fall back on. The same font is
declared in the generated SVG so that `/render.svg` matches as closely as the viewer's
fonts allow.

### Shared core: Rust compiled to native and WebAssembly

The core library (board model and operation application, DSL, connection rules, SVG
generation) is a Rust crate. The server links it natively; the SPA loads it as
WebAssembly through a thin generated binding (wasm-bindgen). The TypeScript for these
parts is removed from the SPA once the WebAssembly build replaces it.

This is the SPA's own code, not a third-party dependency, so it doesn't conflict with the
dependency policy above. It does bear on the policy's reason — initial load weight — so
the crate avoids heavy dependencies: the parser stays hand-written instead of using a
regex engine, and data crosses the boundary in a simple hand-defined shape instead of
through a general serialization framework. The size of the WebAssembly build has a budget
checked in CI.

The codec is excluded from the WebAssembly build. The browser keeps using Compression
Streams (see above), and the server uses a native deflate implementation. Both are tested
against the same vectors, which also resolves the concern below about bit-for-bit
agreement.

#### Alternative: keep TypeScript in the SPA, port to Rust, share test fixtures

**Pros**: the SPA stays pure TypeScript with no WebAssembly toolchain in its build, and
no load-time cost.
**Cons**: the parser, rules, SVG generation, and operation application exist twice. Shared
fixtures (DSL in, expected board or SVG out) catch divergence, but every change is still
made twice, and operation application in particular must match exactly for optimistic
updates to reconcile.
**Decision**: a single Rust core. If the WebAssembly build can't be brought within its
size budget, this is the fallback.

## Implementation Conventions

- **URL updates**: after every operation, encode state and reflect it in the URL via
  `history.replaceState` (replace, not push, so as not to pollute history). On load,
  restore from the URL, or start with an empty board if there is none.
- **Codec identity**: the param ⇄ DSL conversion is split out as pure functions that
  depend on neither rendering nor any framework, called identically by the browser and
  (in the future) the edge layer. The compression format string is fixed in one place
  inside this codec.
- **Separation of rendering and state**: DOM/SVG render the logical state (nodes / edges /
  title) as a projection. Rendering-only values such as a line's control points are not
  put into the logical state; they are derived from note positions.

## Concerns

- **Format differences in Compression Streams**: even if the browser and the edge runtime
  are told to use the same format string, whether the actual output matches bit-for-bit
  needs to be verified during implementation. If it doesn't match, either pin the format
  more tightly in the codec or, as a last resort, fall back to pako. The shared test
  vectors between the browser and the collaboration server address this directly. Note
  that only decoding must agree: different compressed bytes for the same DSL still decode
  identically, so a mismatch in encoder output affects only URL stability and caching of
  the stateless endpoints, not correctness.
- **WebAssembly toolchain in the SPA build**: building the SPA now requires the Rust
  toolchain and a WASM build step before Vite, in CI and in the dev container. Keeping
  `npm run dev` fast with a watch on `keel/` needs to be worked out when the monorepo is
  set up.
- **URL length**: even after compression, the query can hit CDN/browser limits for large
  boards. The behavior when the limit is exceeded (e.g. prompting the user to save to a
  file) needs to be worked out during implementation (corresponds to the Concerns in
  design.md).
