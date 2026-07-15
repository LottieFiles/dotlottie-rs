# Spec Update: Slot Actions for State Machines

Status: **draft — decisions in progress** (grilling session, 2026-07-09)

Extends the dotLottie 2.0 state machine spec with actions that write slot values,
making slot overrides authorable inside `.lottie` files instead of requiring host
player API calls.

## Motivation

Two use cases, in priority order:

1. **File-authored interactivity** (primary): a `.lottie` file whose slots react to
   interactions and state changes with zero host code, portable across conforming
   players (hover → color change, click → image swap, drag → position follow).
2. **Host data binding via inputs**: the host sets state machine inputs; guards and
   slot actions map them onto rendered slots. This is why every value field accepts
   an input reference (`$name`), not just literals.

The host can already set slots directly through player APIs; what this update adds
is the *file-authored* path driven by state machine logic.

## Decisions

### D1 — Per-type actions, one per slot type

Six new action types, following the existing per-type action precedent
(`SetBoolean` / `SetNumeric` / `SetString`):

- `SetColorSlot`
- `SetScalarSlot`
- `SetVectorSlot`
- `SetPositionSlot`
- `SetTextSlot`
- `SetImageSlot`

Each targets a single slot via a required `slotId` field and carries value fields
typed for its slot type. Rationale: flat tagged-enum dispatch (matches every other
action), precise per-type validation, no type-dependent value schema inside one
action. See alternatives doc for the rejected generic `SetSlot` and
parameterized-themes approaches.

### D2 — Gradient deferred

`SetGradientSlot` is out of scope for this update. Gradient values are arrays of
stops (offset + color each); input references inside nested stop arrays have no
clean shape, and static gradient swaps are already served by `SetTheme`. Revisit
on demand.

### D3 — Value fields accept literal or input reference

Every value-carrying field accepts either a literal of its natural JSON type or a
`$`-prefixed string referencing an input of the matching type (numeric fields →
Numeric input, string fields → String input), consistent with existing actions
(`Increment`, `SetFrame`, `OpenUrl`).

### D4 — Multi-component values are arrays with per-element references

`SetColorSlot`, `SetVectorSlot`, and `SetPositionSlot` take a single `value`
array; each element is a number literal or a `$`-prefixed Numeric input
reference:

```json
{ "type": "SetColorSlot", "slotId": "ball_color", "value": ["$r", 0, 0] }
{ "type": "SetPositionSlot", "slotId": "ball_pos", "value": ["$x", 120.5] }
```

- **Color**: 3 or 4 elements (RGB or RGBA), components in the 0–1 range,
  matching theme color rules and Lottie conventions.
- **Vector / Position**: exactly 2 elements.

Rationale: one representation for a color/vector across the whole dotLottie
spec — theme rules already write these as arrays — and per-component clamping
is still available by clamping the input (`Clamp` action) before referencing
it. Named component fields (`r`/`g`/`b`, `x`/`y`) were explored and rejected;
see alternatives doc.

### D5 — `SetImageSlot`: theme-style `src`, no data: URLs, ungated remote

```json
{
  "type": "SetImageSlot",
  "slotId": "avatar",
  "src": "https://cdn.example.com/cat.png",
  "width": 256,
  "height": 256
}
```

- `slotId` is **required** (the early draft omitted it — oversight).
- `src` uses the same prefix resolution as theme image rules: `http(s)://` →
  remote, anything else → package `i/` path. Accepts a `$` String input
  reference.
- Optional `width` / `height` mirror theme `ImageValue`.
- **data: URIs are rejected at runtime on the resolved value** — if the
  resolved `src` starts with `data:`, the action is a no-op. Enforcing on the
  resolved value (not just literals) is what makes the ban real, since a
  `$`-referenced String input could otherwise smuggle one in.
- **Remote URLs are not whitelist-gated.** SetImageSlot is asset loading, and
  the format already loads remote images ungated (theme image rules); a
  SM-triggered fetch adds no new capability. Players MAY apply host policy
  (e.g. CSP on web). This intentionally differs from `OpenUrl`, which is
  navigation and stays default-deny.

### D6 — `SetTextSlot` carries text only

```json
{ "type": "SetTextSlot", "slotId": "score_label", "value": "$score_text" }
```

`value` is a string literal or `$` String input reference. No styling fields:
the state machine use case is dynamic *content*; styling belongs to theme text
rules. Safe by construction — text-only slot writes leave unset text-document
fields falling back to the animation's authored styling, and styling fields
remain addable later without breakage (all optional). See alternatives doc for
the rejected full-`TextValue` shape.

### D7 — Lifecycle: last write wins

A slot holds the value of whoever wrote it last — slot action or theme rule.
No layering, no write provenance:

- Applying a theme overwrites only the slots its rules target; other
  action-written slots keep their values.
- Action-written values are cleared (restored to authored animation values) by
  theme reset, `SetTheme` with empty value, or switching animations.
- Actions execute in listed order, which gives authors control:
  `[SetTheme("dark"), SetColorSlot("accent", [1,0,0])]` → accent is red;
  reversed → accent is the theme's color.

This codifies current player behavior. Rejected alternatives (sticky action
layer above themes; themes atomically clearing action writes) in the
alternatives doc.

### D8 — `ResetSlot` action restores the authored value

```json
{ "type": "ResetSlot", "slotId": "glow" }
```

Restores a single slot to its authored animation value (maps 1:1 to the
existing per-slot clear in the player). Completes enter/exit symmetry —
`PointerExit: [ResetSlot("glow")]` undoes `PointerEnter: [SetColorSlot(...)]`
without hardcoding authored values that go stale on re-export. Naming follows
the input `Reset` action ("restore to initial value").

### D9 — Failures are silent no-ops

Unknown `slotId`, slot of a different type than the action, or unresolvable
`$` reference → the action does nothing and execution continues with the next
action. Matches the existing action convention (missing input → no-op) and
keeps files degradable: a renamed slot doesn't break the rest of the state
machine. Non-normative: players MAY surface diagnostics through their
error/observer channels.

### Out of scope for this doc

Spec version targeting (2.1 vs amendment) is deliberately left to the spec
maintainers; this doc is framed as version-agnostic proposed additions.

## Action definitions

Seven new action types. Common rules:

- `slotId` (required, string): the target slot's ID in the active animation.
- Numeric value elements accept a number literal or a `$`-prefixed Numeric
  input reference; string values accept a literal or a `$`-prefixed String
  input reference — same resolution rules as existing actions.
- All failures (unknown slot, type mismatch, unresolvable ref) are silent
  no-ops (D9).
- Slot writes are one-shot at execution time — no live binding (re-execution
  comes from interactions/state changes firing again).

### SetColorSlot

```json
{ "type": "SetColorSlot", "slotId": "ball_color", "value": ["$r", 0, 0] }
```

`value`: array of 3 (RGB) or 4 (RGBA) elements, each number-or-ref,
components in 0–1.

### SetScalarSlot

```json
{ "type": "SetScalarSlot", "slotId": "opacity", "value": "$level" }
```

`value`: number-or-ref.

### SetVectorSlot

```json
{ "type": "SetVectorSlot", "slotId": "scale", "value": ["$sx", 100] }
```

`value`: array of exactly 2 elements, each number-or-ref.

### SetPositionSlot

```json
{ "type": "SetPositionSlot", "slotId": "ball_pos", "value": ["$x", "$y"] }
```

`value`: array of exactly 2 elements, each number-or-ref.

### SetTextSlot

```json
{ "type": "SetTextSlot", "slotId": "score_label", "value": "$score_text" }
```

`value`: string literal or String input ref. Text content only; unset text
styling falls back to the animation's authored values (D6).

### SetImageSlot

```json
{
  "type": "SetImageSlot",
  "slotId": "avatar",
  "src": "https://cdn.example.com/cat.png",
  "width": 256,
  "height": 256
}
```

`src` (required): string literal or String input ref; resolved by prefix —
`http(s)://` → remote, otherwise package `i/` path. Resolved values starting
with `data:` are rejected (no-op, D5). `width`/`height` optional.

### ResetSlot

```json
{ "type": "ResetSlot", "slotId": "glow" }
```

Restores the slot to its authored animation value (D8).
