# Event Storming Board Design Guidelines

This sets the direction for visuals and interaction. As with design.md / tech-stack.md,
what's recorded is the "why" behind a choice, not just the choice itself. The biggest
difference from a typical web app's design guide is that color here is not decoration —
it's the vocabulary of the event storming method. That constraint runs through everything
below.

## Principles

- **Only notes carry color.** The canvas, panels, toolbar, buttons, selection outlines,
  anchors, and arrows are all monochrome (white to dark gray). No accent color is used.
  Color is used solely to indicate an event-storming type, never for UI decoration. This
  way, the only thing that carries meaning on the board is a note's color, and the
  mapping "color = type" reads without noise. The single exception is the presence of
  other participants in a live session, whose colors are kept dark, saturated, and
  confined to lines and small marks (see "Presence of other participants").
- **White-based, low saturation, easy on the eyes.** True to the whiteboard metaphor, the
  background is white. Notes are pale, low-saturation pastels, kept at a lightness/
  saturation that stays comfortable to look at over long periods.
- **Color is vocabulary, not a palette to choose from.** The mapping between type and
  color is fixed by the method itself (domain event = orange, command = blue, ...). The
  design task isn't picking colors; it's faithfully reproducing a fixed vocabulary with
  attention to legibility, color-vision diversity, and distinguishability between
  adjacent notes.
- **Don't rely on color alone to carry meaning.** Every note always shows its type label
  and ID, so the type is identifiable even where color is hard to tell apart (color-vision
  diversity, adjacent pale colors, monochrome printing). Color is the primary cue; the
  label is the reliable identifier — a two-tier approach.

## Color system

### Notes: align lightness, secure saturation per hue (one lightness exception for yellow)

Type colors are defined in the OKLCH color space. As a rule, **lightness is aligned at
L=0.84**, and **saturation is taken per hue, as much as the sRGB gamut allows, capped at
0.15**. Hue H varies by type. The sole exception is the actor (yellow), whose lightness
alone is raised to L=0.91 (reasoning below).

Aligning lightness lets the types harmonize as a group sharing the same "weight."
Saturation is not pinned to a single low value across the board, because bright hues like
yellow-to-orange all drift toward "beigeish" and become indistinguishable once their
saturation is lowered. By taking as much saturation as each hue allows, the warm types
that can afford it (event, aggregate, actor) come out clearly colored and well separated,
while the types that are inherently pale (command, policy, system) stay calm. The cap of
0.15 keeps the warm colors from becoming jarringly vivid (i.e., keeps them easy on the
eyes).

| Type | Lightness L | Hue H | OKLCH | sRGB |
|------|------|------|-------|------|
| Domain event | 0.84 | 45 | `oklch(0.84 0.094 45)` | `#ffb798` |
| Aggregate | 0.84 | 88 | `oklch(0.84 0.15 88)` | `#f3c443` |
| Actor (lightness exception) | 0.91 | 100 | `oklch(0.91 0.15 100)` | `#f9e361` |
| Read model | 0.84 | 150 | `oklch(0.84 0.13 150)` | `#89e29d` |
| Command | 0.84 | 250 | `oklch(0.84 0.082 250)` | `#a2cfff` |
| Policy | 0.84 | 305 | `oklch(0.84 0.098 305)` | `#d9bbfe` |
| External system | 0.84 | 352 | `oklch(0.84 0.102 352)` | `#ffafd1` |
| Hotspot (exception) | 0.55 | 20 | `oklch(0.55 0.17 20)` | `#c13a46` |

By convention, event, aggregate, and actor are orange, gold, and yellow — but at low
saturation these all turn muddy together, so saturation is secured first and the hues are
separated as event = orange (45) and aggregate = gold (88). The actor keeps the
conventional yellow, but yellow turns yellow-green/olive when its lightness is brought
down to the same 0.84 as the others, so **the actor alone has its lightness raised to
0.91, giving a clean, bright yellow**. This is a deliberate, justifiable exception to the
lightness alignment, grounded in the fact that yellow is, perceptually, an inherently
bright color. It also compensates — via both the lightness and hue difference from the
aggregate (gold, L0.84) — for the size-based distinction that's no longer available now
that note sizes are uniform. The principles of "only notes carry color," "low saturation,
easy on the eyes," and "harmony through aligned lightness" are otherwise preserved.

Text contrast against dark-gray text (`oklch(0.30 0 0)` ≒ `#2e2e2e`) stays at roughly
8.0–10.5:1 across all colors, exceeding WCAG AAA (7:1) for every one of them.

### The hotspot is a deliberate exception

The hotspot (an issue or open question) exists to draw attention and must not blend into
the same paleness as the others. So it uses a darker, more saturated red,
`oklch(0.55 0.17 20)` (`#c13a46`), with white text (contrast against white is roughly
5.3:1, meeting WCAG AA). On the size axis too, this tool unifies note sizes for everything
except the hotspot, which alone is allowed a different size (see design.md). It's treated
as the single exception, signaling "this is not part of the normal flow" through both
color and size.

### Monochrome chrome

The rest of the UI is built from the following neutrals only. No accent color is used.

| Role | OKLCH | sRGB |
|------|-------|------|
| Canvas background | `oklch(0.995 0 0)` | `#fdfdfd` |
| Panel/toolbar background | `oklch(0.985 0 0)` | `#fafafa` |
| Dot grid | `oklch(0.90 0 0)` | `#dedede` |
| Border | `oklch(0.88 0 0)` | `#d7d7d7` |
| Secondary text | `oklch(0.55 0 0)` | `#717171` |
| Body text | `oklch(0.30 0 0)` | `#2e2e2e` |
| Selection outline | `oklch(0.40 0 0)` | `#484848` |
| Arrow/edge | `oklch(0.55 0 0)` | `#717171` |

The prototype used blue (`#4263eb`) for selection outlines and anchors, but under the
"only notes carry color" principle, selection, anchors, the temporary in-progress arrow,
and arrows themselves are all replaced with dark gray. Selection is shown not by color but
by a thick (2px) dark-gray outline, and an anchor is a small dark-gray circle with a white
border. The distinction between rule-conforming arrows (solid) and out-of-rule
(exception) arrows (dashed) is made entirely in gray, using only stroke width and dash
pattern.

## Typography

- Note body text and the UI use the same sans-serif (with a fallback to the system's
  Japanese sans-serif, since Japanese text is involved).
- Only the DSL panel's text uses a monospace font, signaling visually too that this is a
  surface for plain text.
- Font sizes are kept to a small number of steps (roughly three: note body, UI labels,
  secondary text). No decorative large headings — this is a tool, not reading material, so
  visual hierarchy is kept minimal.

## Notes and canvas

- All notes except hotspots are unified to a common size. The distinction between types is
  carried by color and the type label, not by size (a requirement from design.md). This
  keeps the visual rhythm consistent when notes are arranged, supporting alignment and
  pattern recognition.
- Notes don't have strongly rounded corners (small `rx`). This leans toward the metaphor
  of a physical sticky note rather than looking too much like a generic UI widget.
- The canvas is a white background with a faint dot grid. The grid is an alignment aid and
  shouldn't assert itself (the palest gray available).
- Every note always carries its type label and ID — the concrete realization of the
  "don't rely on color alone" principle, so the type is identifiable without depending on
  color.

## Presence of other participants

Each participant gets a color for their cursor, selection, and name label. With several
people pointing at once, telling cursors apart at a glance matters more than anything
else, and a color does that faster than reading a name. This is the one deliberate
exception to "only notes carry color," and it is kept from being confused with the note
vocabulary by contrast along every axis the notes use:

- **Dark and saturated, where notes are pale.** Participant colors sit at a low lightness
  and high chroma (around `oklch(0.55 0.18 H)`), clearly apart from the notes' L=0.84
  pastels. A participant color is never used as a fill of anything note-sized.
- **Lines and small marks only.** Participant colors appear only on the cursor arrow, the
  name label's background (with white text), and outlines. The notes remain the only
  large areas of color on the board.
- **Clear of the hotspot.** The hotspot is the one dark saturated note (red, H=20), so the
  red range around it is not used for participants.
- **A fixed, ordered palette.** A small palette (around eight hues spread evenly outside
  the red range) is assigned in join order, so each participant keeps their color for the
  session. Beyond the palette size, colors repeat and the name label disambiguates.

- **Remote cursors** are an arrow in the participant's color with a label showing their
  name. The name remains the reliable identifier, in the same way that the type label is
  for a note: color is the fast cue, the label is the definitive one.
- **Remote selections** are a 2px dashed outline in the participant's color with the name
  label at its corner. Your own selection keeps its 2px solid dark-gray outline, so you
  never mistake someone else's selection for yours.
- **A note someone is editing** carries the same dashed outline and a label
  "<name> is editing." It is not dimmed or recolored, so the note's type color stays
  readable.
- **AI agents** get a participant color like anyone else, plus a text badge "AI" in their
  name label and in the participant list. An agent has no cursor; the notes it is working
  on get the dashed outline with its label instead.
- **Idle cursors** fade out after a few seconds without movement, so a board with many
  participants doesn't fill up with stationary labels.

## Concerns

- **Proximity in the warm range (event, aggregate, actor).** Bright hues drift toward
  "beigeish" and become indistinguishable when their saturation is lowered. To address
  this, saturation is secured per hue, event and aggregate are separated by hue (orange vs
  gold), and the actor (yellow) has its lightness raised for a clean yellow, separating it
  from the aggregate via both lightness and hue. The fact that yellow alone doesn't share
  the common lightness is a deliberate exception; in situations where discrimination
  degrades (e.g. under color-vision diversity), the type label serves as the final
  identifier.
- **Verification under color-vision diversity.** Pale colors with aligned lightness/
  saturation may converge for some types of color vision. How well pastels with little
  lightness difference can be told apart needs to be checked in real use. If
  discrimination turns out to be insufficient, the plan is to compensate via labels or
  pattern overlays rather than reshuffling hues (since the vocabulary is fixed).
- **Browser rendering differences for OKLCH.** Fallback to sRGB and on-screen rendering of
  `oklch()` may vary across environments. OKLCH values are the source of truth; sRGB hex
  values are included as reference only.
- **Participant colors next to note colors.** Dark saturated outlines on pale notes should
  read as "someone is here," not as a type. This needs to be checked on a busy board,
  especially a participant's dashed outline around a note of a similar hue, and under
  color-vision diversity, where the name label is the fallback.
- **Dark mode.** This tool adopts a white-based theme as part of the whiteboard metaphor.
  If a dark theme is added, inverting the background while keeping "only notes carry
  color" makes the notes' pale colors hard to sustain, so this is left as a separate item
  to consider (a single white-based theme for now).
