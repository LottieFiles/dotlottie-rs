# Slot Actions — Explored Alternatives

Companion to [slot-actions.md](./slot-actions.md). Approaches we considered and
did not choose, kept for the record so the reasoning survives.

## A1 — One generic `SetSlot` action (rejected)

```json
{
  "type": "SetSlot",
  "slotType": "Color",
  "slotId": "ball_color",
  "value": [1, 0, 0]
}
```

One spec entry instead of six. Rejected because the `value` schema depends on a
sibling field (`slotType`), which makes validation and deserialization a
discriminated-union-inside-an-action — unlike every other action in the spec,
which dispatches purely on `type`. Per-type actions keep errors local and the
schema flat. Worth revisiting only if the action list grows unmanageably.

## A2 — Parameterized themes instead of slot actions (rejected)

```json
{
  "type": "SetTheme",
  "value": "red_theme",
  "bindings": { "intensity": "$level" }
}
```

Keep all visual data in `t/` theme files (where designers author it) and let
`SetTheme` pass input values into theme rules as template variables. One styling
mechanism instead of two. Rejected because it is heavyweight for "set one slot on
click", requires reworking the theme spec (themes become templates), and couples
state machine execution to theme loading. Slot actions are the surgical tool;
themes remain the bulk tool.

## A3 — Named component fields for multi-component values (rejected)

```json
{ "type": "SetColorSlot", "slotId": "ball_color", "r": "$r", "g": 0, "b": 0 }
{ "type": "SetPositionSlot", "slotId": "ball_pos", "x": "$x", "y": 120.5 }
```

Self-documenting, no positional ambiguity, alpha cleanly optional, and each
component clamps naturally. Rejected because it creates a *second* way to write
a color/vector in the same ecosystem — theme rules already use arrays
(`"value": [1, 0, 0]`) — so every editor and converter would need both
representations. The clamping benefit is preserved in the chosen shape by
clamping inputs (`Clamp` action) before referencing them in the array.

## A4 — Whole-value string reference (rejected for v1)

`"value": "$color_str"` where one String input holds an entire serialized
color/vector. Rejected: requires the spec to define a string encoding for
colors/vectors (hex? CSS? array-string?), a whole new parsing concern for
marginal benefit over per-element refs.

## A5 — data: URLs in `SetImageSlot.src` (rejected)

Full parity with theme image `src` semantics would include
`data:image/png;base64,…` embedded images (`ImageSlot::from_data_url` exists
and themes use it). Rejected to keep giant base64 blobs out of state machine
definitions. The ban is enforced at runtime on the *resolved* value (post-`$ref`
resolution → no-op), otherwise a String input could smuggle one in and the ban
would be advisory. Trade-off accepted: state machine `src` semantics are now a
strict subset of theme `src` semantics.

## A6 — Whitelist-gating remote image URLs (rejected)

Reusing the OpenUrl whitelist (default-deny) for `SetImageSlot` remote fetches
was considered for a consistent "SM-initiated external access is opt-in"
posture. Rejected: theme image rules already load remote images ungated, so
gating only the action is bypassable via `SetTheme` (security theater), and
default-deny breaks remote-image files on every unconfigured host — directly
against the file-authored-first goal. A separate default-allow image allowlist
was also considered and dropped (new spec + API surface for a control hosts
already have via CSP / network layer).

## A7 — Full `TextValue` styling in `SetTextSlot` (rejected)

Mirroring the theme text rule's 13 styling fields (fontName, fontSize,
fillColor, stroke, tracking, justify, …) each as literal-or-ref. Rejected as
spec bloat duplicating theme text rules field-for-field; the "style change on
state" use case is served by `SetTheme` with a text rule. A middle-ground
subset (text + fillColor + fontSize) was also rejected — subset boundaries age
badly ("why tracking but not stroke?"). Styling fields can be added to the
action later without breakage since a text-only write leaves unset fields on
authored values.

## A8 — Slot-write precedence models (rejected)

Two alternatives to last-write-wins:

- **Sticky action layer**: SM-written slots outrank later theme applications
  until explicitly reset. Rejected: requires per-slot write-provenance
  tracking and re-application of overrides after every theme apply — a
  CSS-specificity-style model for a rare conflict that action ordering already
  resolves.
- **Atomic themes**: any `SetTheme` clears all action-written slots first.
  Rejected: destructive order-dependence (theme + per-slot tweak in one action
  list becomes impossible) and diverges from current player behavior, which
  only overwrites overlapping slots.

## A9 — `ResetAllSlots` bulk action (deferred)

"Restore every slot to authored values without touching the active theme" is a
real semantic gap (`SetTheme("")` clears slots *and* the theme), but no
concrete use case has asked for it. Trivially addable later as one more action
type.

## A10 — `SetGradientSlot` (deferred, not rejected)

Gradient slot values are arrays of stops (offset + color each). Supporting input
references inside nested stop arrays has no clean JSON shape, and the dominant use
case (swap the whole gradient) is already served by `SetTheme`. Deferred until
there is concrete demand; the player API (`set_gradient_slot`) already exists, so
only the spec/action layer is missing.
