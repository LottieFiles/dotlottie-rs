# State-Declared Slots — Explored Alternatives

Companion to [state-slots.md](./state-slots.md). This exploration is
independent of the slot-actions proposal
([slot-actions.md](./slot-actions.md)); reconciling the two directions, if
both proceed, is a separate product decision.

## B1 — Tween via per-action durations (the other fork)

The competing authoring model for tweened slot changes: keep slot changes
imperative (the slot-actions proposal) and add optional `duration` + `easing`
to each interpolable setter:

```json
{ "type": "SetColorSlot", "slotId": "accent", "value": [1,0,0],
  "duration": 0.4, "easing": [0.4, 0, 0.2, 1] }
```

Strengths: one mechanism total; works in interactions (hover fade via
PointerEnter) not just state entries. Weaknesses: needs a *new* concurrent,
non-blocking property-tween runner with cancellation rules (the existing
tween lifecycle blocks inputs and the pipeline — acceptable for a 300ms pose
blend, not for a 2s color fade); tween duration is fixed by the action, not
per-transition; and a "state look" is smeared across entry-action lists
instead of being declared. Not chosen for this exploration — the declarative
model piggybacks the existing tween lifecycle and keeps per-transition
timing control.

## B2 — Sticky slot lifetime (rejected)

Declared values persist after leaving the state until another state
redeclares the slot (pure last-write-wins, no release machinery). Rejected:
contradicts what state configuration already means in the engine (undeclared
`loop`/`speed`/`segment` reset to defaults on entry), reduces `slots` to
sugar for entry-time writes, and creates the "passed through `alert` once,
the red followed me forever" bug class. A sticky-with-explicit-release
variant (`"reset": true` entries) was also rejected — it pushes release
bookkeeping onto every downstream state.

## B3 — Text/Image in state slots (excluded, revisitable)

Discrete types can't interpolate, so a `Tweened` transition needs a snap
rule. If this exclusion is ever lifted, the analysis was: snap at **50%
progress** (CSS's rule for discrete-property transitions — hides the pop
where the composition is furthest from both stable looks) beat snap-at-start
(new image on a still-old pose reads as a glitch) and snap-at-end (the
transition visually completes twice). Excluded for now so that everything a
state declares is tweenable and the mechanism needs no discrete-snap rule.
Cost accepted: per-state badge icons/text are not expressible in this
direction.

## B4 — Sampled-at-entry input references (rejected)

Resolve `$refs` once at state entry; later input changes do nothing until
re-entry. Simpler engine (no dependency tracking), but in a spec direction
with no actions, dynamic styling would require transitioning out and back in
to refresh a value — a gauge that only updates on state churn. Live binding
is the capability that makes the declarative direction more than sugar.
Literals-only (no refs at all) was also rejected: it severs the host-data
path entirely.

## B5 — Theme-rule-compatible entry shape (rejected)

Reusing `{ id, type, value }` from theme rules so state slots read as an
inline mini-theme and tooling reuses theme validators. Rejected because the
compatibility is partial in both directions — no keyframes/expression/
animations allowed here, and `$refs` are not valid in theme rules — so the
shared shape would invite exactly the mistakes a schema exists to prevent.

## B6 — State-level theme reference (future companion, not chosen)

`{ "type": "PlaybackState", "theme": "alert_look" }` — the state's look
lives in `t/` as a real theme; transitions interpolate between theme values.
Maximum designer-tooling reuse and whole-look granularity, but no `$refs`
(incompatible with live binding), and scoped release semantics would tangle
with host-driven `set_theme`. Worth revisiting as a *companion* to inline
state slots (coarse look via theme ref + fine dynamic values via slots), not
as a replacement.

> **Update 2026-07-15**: prototyped in dotlottie-rs and validated headlessly
> (`tests/dnd_interaction_check.rs`, `examples/star_drop_dnd.rs`). The
> single-star state machine shrank from 3 states / 3 interactions / 7 inputs
> to 1 state / 1 interaction / 1 input, with zero coordinates in the JSON
> (snap targets derived from drop-zone layers at load). Snap tween is
> non-blocking (engine stays Running), grab offset works natively, and
> `lock` + per-zone actions bridge cleanly into SM logic. One engine finding:
> gesture tween writes must advance *before* the player renders each tick,
> or hit-testing sees a one-frame-stale scene.

## B7 — Dedicated `DragAndDrop` interaction (explored, undecided)

Prototype experience: pure-SM drag & drop of 4 objects is expressible
(per-object dragging states + prioritized Event/Boolean guard conjunctions +
transient docking states baking rest inputs) but costs ~9 states / 9
interactions / 19 inputs, O(objects) in three dimensions, with coordinates
duplicated because actions can't read layer positions. A dedicated stateful
interaction (`layerName` + `slotId` + `dropZones[{layerName, snap, lock,
actions}]` + return tween) collapses this to gesture-free game-logic states
(~70 lines), derives dock coordinates from the zone layers (single source of
truth), and unlocks what the single-current-state model structurally cannot:
simultaneous multi-touch drags, native grab offset, mid-gesture cancellation.
Costs: a hidden per-object state machine inside a transparent model
(interplay with transitions/theme/animation changes needs speccing), heavy
cross-player gesture-parity burden, guaranteed knob creep. Lean alternative
if drag & drop is not strategic: built-in `$pointer.x/y` inputs,
machine-level default slot bindings, and transition-scoped actions — ~60%
of the boilerplate for none of the new primitive's costs.

## B8 — `@` globals in state slot values (rejected)

Allowing `@elapsedTime` in a live-bound declaration would promise per-frame
re-evaluation (bindings re-fire on input writes; globals tick continuously),
turning a binding mechanism into an animation driver with per-tick slot
flushes. Excluded from this spec direction.
