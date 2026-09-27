# Event Storming Board Design Document

A tool for storing an event storming whiteboard as a highly portable plain text DSL. The
primary motivation is collaboration with LLMs, and users operate a whiteboard, not text.
This document covers the design decisions for rebuilding the prototype (a single HTML
file) as a maintainable frontend. Technology stack selection is split into a separate
document. Real-time collaboration, added on top of this design, is covered in
collaboration.md.

## Context

Event storming is a method for discovering business flows by placing sticky notes for
roles such as domain events, commands, actors, aggregates, policies, read models, and
external systems, and connecting them with arrows. The color of a note (its type) carries
meaning, and arrows follow rules about which type can connect to which. Unlike a general
whiteboard, all that's needed is typed notes and rule-conformant connections; free-form
shapes or arbitrary decoration are not required.

There is strong demand to move this output back and forth with LLMs. A diagram is hard to
hand to an LLM, and it's just as hard to turn a structure an LLM produces back into a
diagram. Bridging the two requires a plain text representation that both humans and
machines can read.

A working prototype (a single HTML file) confirms that this approach is viable, with a
basic configuration of 8 note types, connection rules, a plain text DSL, pan/zoom, and
file save. This document states, declaratively, the design decisions needed to realize
the same goal as a maintainable frontend.

## Motivation

The output of event storming takes the form of a diagram of notes and arrows. We want to
move this diagram back and forth with whoever is advancing the design or implementation —
especially an LLM. But a diagram is hard to hand to an LLM, and it's just as hard to turn
a structure an LLM returns back into a diagram.

Existing general-purpose whiteboards (e.g. Miro) have great interaction but hold state in
a proprietary, opaque format. It isn't portable, can't be diffed, and can't be handed to
an LLM as text. Conversely, writing the DSL directly in a text editor is portable but
loses the experience that's central to event storming: discovering things by directly
manipulating a diagram.

What this tool solves is making both of these true at once: the surface the user touches
is direct manipulation of a whiteboard, while the canonical representation of state is
plain text that both humans and machines can read. If that holds, the same board can be
manipulated as a diagram, handed to an LLM as text, and shared/versioned as a URL or file.
Operation is the whiteboard; storage and sharing are portable plain text — making this one
thing true is the reason this tool exists.

## Goals

- On a Miro-like direct-manipulation canvas, place notes, move them freely, and pan/zoom.
  Users draw by directly manipulating the diagram and are never forced into text editing.
- Arrows are drawn by dragging from a note's anchor (a connection point on its edge) to
  another note. A line is not something drawn by hand; it's a relationship between two
  notes.
- The board state can be exported as a plain text DSL usable by both humans and LLMs.
- A single URL fully reproduces the board, and the URL reflects the current state after
  every operation.
- Arrows that conform to connection rules are shown as solid lines; arrows that don't are
  automatically distinguished as dashed (exception) lines.
- All notes except hotspots share a common size, aiding alignment and pattern
  recognition.
- Multi-select and copy/paste let users work with recurring structures as a unit.
- Recurring chains of patterns can be assembled quickly via automatic connection on
  adjacent placement.
- A policy can be left empty (representing "always fires").
- The board can be exported as an image (SVG / PNG).

## Non-Goals

- **General-purpose whiteboard features**: free-form lines, images, arbitrary shapes, and
  font decoration are out of scope. The expressible content is limited to typed notes and
  their relationships. This is not a limitation but a deliberate choice to focus on the
  vocabulary of event storming.
- **Server-resident persistence and accounts**: state lives in the URL and in files, and
  the core is complete for a single local user. Real-time multi-user editing was
  originally a Non-Goal as well; it is now designed in collaboration.md as a volatile live
  session laid over this core, which leaves the URL as the only save form and still
  introduces no accounts or database.
- **A machine-oriented raw DSL endpoint (`f=raw`) in the core**: serving the DSL as
  `text/plain` to HTTP clients that don't execute JS is not part of the core. As described
  below, the design leaves a hook for this, and the collaboration server of
  collaboration.md provides it as a stateless endpoint.
- **Groups/templates as persistent entities**: since recurring chunks of patterns can be
  handled with copy/paste, no new concept such as a group object or a template library is
  introduced.

## Solution

### Core abstraction: a single state representation with two projections

The board's logical state is held as a single canonical piece of data (a set of nodes, a
set of edges, and a title), from which two **projections** are derived. A projection is a
mapping of state into another form; it holds no state of its own.

```
            ┌─────────────┐
            │ Logical state │  <- canonical (nodes/edges/title)
            │  (in-memory) │
            └──────┬──────┘
   projection ↓     │     ↓ projection
   ┌──────────┐    │   ┌──────────────┐
   │ Whiteboard │ <-operate-┴-edit-> │ DSL text          │
   │ (canvas)   │           │ (plain text panel) │
   └──────────┘           └──────────────┘
                                   │ encode
                                   ↓
                          URL query (portable save form)
```

The user primarily operates the whiteboard. At the same time, an equally valid path
exists for updating the board by pasting in DSL text generated or edited by, say, a chat
AI. State is therefore **bidirectional**: a change from either entry point is folded into
the logical state and reflected in the other projection. Neither the DSL text nor the
canvas DOM is treated as canonical — both are views of the logical state. This
structurally cuts off the kind of problem the prototype had, where rebuilding the note
DOM killed `dblclick` — i.e., where display concerns leak into state management.

Real-time collaboration (collaboration.md) fits this abstraction without changing it:
operations from other participants are one more entry point folded into the same logical
state, and both projections follow as they do for local edits.

### Interaction model: direct-manipulation canvas and anchor connections

The surface the user touches is a Miro-like, directly manipulated whiteboard. Notes are
placed from a palette, dragged freely, and the canvas itself can be panned/zoomed. A note
has a type (color and label) and its body text can be edited directly.

An arrow is not a line-drawing tool; it is a relationship between two notes. Each note has
anchors (connection points) on its four sides, and dragging from one to another note
establishes the relationship. The path of the line and which edge each endpoint attaches
to are determined automatically from the positions of the two notes — the user never
manipulates the path itself. This reflects a policy where "the user specifies only whether
a relationship exists, and its appearance is derived from state," which also keeps extra
rendering information (such as line control points) out of the logical state.

On every connection, the combination of types is checked against the rules; ones that
conform are shown as solid lines, and ones that don't are automatically distinguished as
dashed (exception) lines (the content of the connection rules follows a separate rule
definition). There is no facility for drawing arbitrary shapes or free-form lines as a
general whiteboard would. What can be expressed is limited to typed notes and their
relationships — a deliberate constraint that aligns the operating surface with the
vocabulary of event storming.

### Portable save form: the URL as the single representation of state

The logical state is serialized to the DSL, which is then encoded into the URL's query.
`?data=<encoded>` is the complete save form for the board, and is also the flip side of
not holding state on a server. The URL is updated (replacing, so as not to pollute
history) after every operation, and sharing that URL lets another person open the same
board. This satisfies the need to "save and reuse a template" without introducing a
separate concept: build a recurring board and bookmark its URL, and that bookmark
effectively becomes a template.

The encoding doesn't carry the raw DSL as-is; it's compressed for URL length and
portability (an environment-independent, lightly-dependent scheme such as deflate +
base64url). The result is that the query value is unreadable to humans. It's precisely
this operation — turning an unreadable value back into a human-readable DSL — where the
codec and the raw extension point described below have value.

Why a query string rather than a hash fragment (`#data=`)? A fragment is never sent to
the server, so it can be entirely client-side, with the benefit that state never appears
in logs or the Referer. But because a fragment can never reach the server in principle,
it forecloses, from the start, any future path to implementing `f=raw` at the edge. Since
this tool's primary purpose is LLM collaboration, retaining a machine-readable path is
valuable, so we adopt the query string. State appearing in logs/Referer, and the fact
that some CDNs enforce strict URL length limits, are accepted as trade-offs (compression
mitigates the length issue).

### codec: param ⇄ DSL split out as pure functions

Conversion between URL parameters and DSL text is split out as **pure functions** that
depend on neither rendering nor any framework. Both the browser-side core and a future
edge function call the same codec. The codec doesn't need the logical-state model layer
(building nodes/edges via the parser, validating connection rules); it only handles the
round trip between text and parameters. This separation is the key that lets raw be added
later while keeping the core a static SPA.

### Extension point: machine-oriented raw (same URL, addable later)

For clients that don't execute JS (curl, server-side fetch, many LLM fetch tools), static
hosting just returns the same JS bundle for any query, and the query only exists once JS
has started. So satisfying `f=raw` requires a layer that runs before the request reaches
the asset.

This isn't a real backend (a DB, holding state) — a **stateless edge function** suffices.
Since all state is in the URL, the function's job is just: take the parameters, use the
decode half of the codec to turn them back into the DSL, and return it as `text/plain`.
The path is not split; only when `&f=raw` is appended to the same URL as the app does an
edge middleware perform content negotiation: raw DSL for clients without JS, the normal
static SPA for browsers.

The core is a static SPA that's fully functional without raw; raw remains an extension
that can be enabled if the hosting has a middleware layer. The design explicitly states
the line: on pure static hosting without a middleware layer (e.g. GitHub Pages), this
feature is given up.

The collaboration server of collaboration.md changes what is available, not this
principle: it serves raw DSL (and SVG/PNG renderings) for a given `data` value at its own
paths, as stateless endpoints. Because that server also serves the SPA's files, it is the
layer that sees the request for the app's URL, so same-URL negotiation on `&f=raw` is
available wherever the server is deployed. A purely static deployment still gives it up.

### Handling notes, selection, and connections

These are properties the system must satisfy as requirements; this section describes how
they sit on top of the core abstraction of logical state and its projections.

- **Uniform size**: all notes except hotspots share a common size. The distinction
  between types is carried by color and the type label, not by size. Hotspots (issues)
  are allowed a different size as a deliberate exception meant to draw attention.
- **Multi-select and copy/paste**: selection is a set, not a single item. Copying
  duplicates the selected nodes plus any internal edges whose both endpoints are in the
  selection. Pasting assigns new IDs and offsets the placement. Reuse of recurring chunks
  of patterns rides on this general mechanism; no separate template concept is built.
- **Automatic connection on adjacency**: when a note is placed next to an existing note in
  a direction that the connection rules allow, a solid arrow conforming to the rules is
  drawn automatically. This is implemented not as a new entity but as behavior where "the
  act of placement also infers a connection" — the logical state still contains only nodes
  and edges.
- **Empty policy**: a policy's text may be left empty, representing a relay point with no
  body, like "always fires." In the DSL this appears as a policy line with empty text; no
  special syntax is added.

### Shape of the DSL

The DSL is plain text consisting of declaration lines (type, ID, body text, coordinates)
and connection lines (`->` / `..>`), following the prototype's `# eventstorming v1`
format. Most of the requirements need little or no syntax extension: uniform size is a
rendering-side property, and multi-select/copy-paste are operation-side properties —
neither appears in the DSL. An empty policy is expressed as a policy line with empty body
text, with no new token added. Lines produced by automatic connection on adjacency are
written out as ordinary edge lines. This policy of not introducing new concepts into the
DSL keeps the surface an LLM has to deal with small, serving the goal of ease of
collaboration.

## Alternative Solutions

### Make the DSL text the source of truth

**Pros**: the save form and the edit target coincide, so synchronization issues are less
likely at the implementation level. Round-tripping with an LLM is straightforward.
**Cons**: contradicts the premise that the user primarily operates the whiteboard. Every
diagram operation would need to regenerate the text and reconcile it with caret position
and uncommitted edits — display concerns end up leaking into state, which is the opposite
of what we want.
**Decision**: treat logical state as canonical, with the DSL and the canvas as equal
projections. Updates via text are handled uniformly as flowing back from a projection;
neither entry point is privileged.

### Put state in the URL hash fragment

**Pros**: fully client-side. State never appears in server logs or the Referer, which is
good for privacy, and it works on pure static hosting alone.
**Cons**: a fragment never reaches the server, which forecloses, in principle, any path to
implementing `f=raw` at the edge.
**Decision**: since LLM collaboration is the primary purpose, retaining a
machine-readable path is valuable enough to adopt the query string. Log exposure and URL
length are addressed by compression and accepting the trade-off.

### A separate path/subdomain for raw

**Pros**: just one dedicated edge function is needed; no content-negotiation middleware
required — the most straightforward option.
**Cons**: the board URL and the raw URL become different things, hurting the portability
of shared links. It breaks the intuition that "appending `f=raw` to the same URL gives you
the raw DSL."
**Decision**: don't split the path. Edge middleware on the same URL determines whether
`&f=raw` is present. We accept the resulting constraints: middleware is required, and pure
static hosting can't support it.

### Introduce groups/templates as first-class entities

**Pros**: a recurring chunk of pattern can be held as a meaningful unit, making
move-as-a-group, collapsing, etc. natural to express.
**Cons**: adds new concepts to both logical state and the DSL, widening the surface an LLM
has to deal with. The requirement can be met with copy/paste.
**Decision**: don't introduce new concepts; substitute the combination of multi-select +
copy/paste + automatic connection on adjacency. Templates are substituted by the practice
of "bookmark the URL of a recurring board."

### Unify raw and the app via an integrated meta-framework (SSR)

**Pros**: rendering and server-side processing can live in one framework.
**Cons**: what raw requires is a single stateless edge function; an SSR framework is
overkill, and would needlessly constrain the core's framework choice (which leans toward a
lightweight SPA).
**Decision**: keep the core a static SPA, with raw as a co-located stateless edge
function — the two are orthogonal. Because the codec is split out as pure functions, both
can share the same conversion code.

## Concerns

- **URL length**: for large boards, the query can grow long even after compression and may
  hit URL length limits imposed by CDNs or browsers. The choice of compression scheme and
  the behavior when the limit is exceeded (e.g. prompting the user to save to a file) need
  to be worked out during implementation.
- **State exposed in logs**: adopting the query string means state can appear in server
  access logs and the Referer header. This is considered acceptable for a single-user
  local use case, but a note of caution about handling shared URLs may be needed. With
  live sessions, the room ID in the URL is also the capability to join and write, which
  raises the stakes of a leaked URL (see collaboration.md).
- **Forward/backward compatibility of the codec**: if the encoding scheme changes later,
  existing shared URLs may stop opening. Whether to include a version identifier in the
  encoded value, and how a version marker like `# eventstorming v1` should be reflected on
  the URL side, need further consideration.
- **Criteria for automatic connection on adjacency**: what distance/direction counts as
  "adjacent," and how to prevent connections the user didn't intend (including how easy
  they are to undo), directly affects the feel of the tool and needs to be tuned during
  implementation.
- **Hosting dependency of raw**: raw only works on the premise of an edge with a
  middleware layer. If the deployment target is decided to be purely static, this feature
  can't be provided. Since it's a Non-Goal, this doesn't affect the core, but it should be
  documented as an operational policy. Deployments that include the collaboration server
  serve the SPA through it and therefore provide raw.
