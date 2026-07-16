# RFC 0002 — DragAndDrop Interaction

- **Status**: Draft
- **Date**: 2026-07-15
- **Prototype**: implemented and tested in dotlottie-rs (see [Prototype status](#prototype-status))
- **Related**: [RFC 0001 — State-Declared Slots](./0001-state-declared-slots.md) (the substrate this builds on and the approach it's measured against), [state-slots-alternatives.md B7](../spec-updates/state-slots-alternatives.md) (exploration log)

## Summary

Add a stateful `DragAndDrop` interaction type that owns the entire drag
gesture: pickup (engine hit-tests the draggable layer), drag (writes a
Position slot, preserving the grab offset), and drop (hit-tests drop zones,
snaps into the matched zone or glides back, runs zone actions). The state
machine keeps owning the *game logic*; the interaction owns the *gesture*.

## Motivation

RFC 0001 proved drag & drop is fully expressible with state-declared slots —
and measured the cost. Per draggable object the pure state-machine version
needs two gesture states, two interactions, and five inputs of plumbing, and
it structurally cannot express three things users expect:

1. **Grab offset** — bindings have no arithmetic, so objects jump to center
   themselves on the pointer.
2. **Simultaneous drags** — a single current state can't hold two objects
   mid-gesture.
3. **Single-source coordinates** — actions can't read layer positions, so
   drop-zone coordinates are hardcoded into the state machine, duplicating
   the animation.

A gesture is interaction-shaped (pointer event → effect, like `Click` and
`PointerEnter`), just longer-lived. Making it one interaction removes the
per-object state explosion and fixes all three limitations natively.

## Proposed spec change

One new interaction type:

```json
{
  "type": "DragAndDrop",
  "layerName": "Star 2",
  "slotId": "star_pos",
  "grabOffset": true,
  "tween": { "duration": 0.25, "easing": [0.25, 0.1, 0.25, 1] },
  "dropZones": [
    {
      "layerName": "drop_zone",
      "lock": true,
      "actions": [{ "type": "Increment", "inputName": "docked_count" }]
    }
  ]
}
```

Note the absence of coordinates: the snap target is derived from the
`drop_zone` layer's authored position. The animation is the single source
of truth; moving the zone in the editor moves the dock with it.

- `layerName` (required): the draggable layer; the engine hit-tests
  `PointerDown` against it for pickup.
- `slotId` (required): the Position slot written while dragging and
  snapping.
- `grabOffset` (optional, default `true`): preserve the pointer-to-object
  offset captured at grab, so the object doesn't jump to center on the
  pointer.
- `tween` (optional): duration (seconds) + cubic-bézier easing for the
  snap-in and return glides; omitted = instant.
- `dropZones` (ordered): each zone has
  - `layerName` (required): hit-tested against the `PointerUp` position;
    first hit wins.
  - `snap` (optional): explicit coordinate override for the rare case where
    the dock point differs from the zone layer's position (e.g. snapping to
    the mouth of a container). **Omit it and the zone layer's authored
    position is the snap target** — the normal case.
  - `lock` (optional, default `false`): once docked here, the object can no
    longer be grabbed.
  - `actions` (optional): executed when the object docks — the bridge into
    normal state machine logic (fire events, increment counters, trigger
    guarded transitions).

Behavioral rules:

1. **Gesture lifecycle**: Idle → Held (PointerDown hits the layer) →
   Snapping (PointerUp; into the matched zone, or back to the *rest
   position* on a miss) → Idle. The rest position is where the object was
   first grabbed, updated to the snap target on every successful dock.
2. **Non-blocking**: the snap glide is a property tween independent of
   `Tweened` transitions — the machine stays Running, inputs stay writable,
   and grabbing mid-snap cancels the glide and continues from where it was.
3. **Failures are silent no-ops**: unknown layers or a non-Position slot
   make the interaction inert; a zone with no derivable snap target behaves
   as a miss.

No changes to states, transitions, guards, inputs, or RFC 0001 semantics.

## Measured impact

Same single-star demo, both approaches (prototype-validated):

| | Pure state machine (RFC 0001 mechanisms) | `DragAndDrop` interaction |
|---|---|---|
| States | 3 (idle / dragging / docking) | 1 (game logic only) |
| Interactions | 3 | 1 |
| Inputs | 7 | 1 (`docked_count`, game logic) |
| Coordinates in SM JSON | dock repeated twice | **zero** |
| Grab offset | not expressible | native |
| Input freeze during glide | yes (transition tween) | no |
| Re-grab mid-glide | rejected | cancels glide, continues |
| Docked-object locking | needs extra guard plumbing | `lock: true` |
| Four-object version | 9 states / 9 interactions / 19 inputs | 1 state / 4 interactions / few game inputs |
| Simultaneous multi-drag | structurally impossible | per-interaction phases (multi-pointer TBD) |

A note on how the "zero coordinates" row is achieved, since it shapes what
can be promised: snap targets are the drop-zone layer's **authored transform
position** (`layers[].ks.p`), extracted from the animation JSON once at load
— a map lookup at drop time, no renderer query. Three constraints follow:
it is the transform origin, not a computed visual center (they coincide when
shapes are drawn around the layer origin, the normal editor output);
keyframe-animated zone positions are not tracked (a moving drop zone needs
the `snap` override); and only top-level, unparented layers resolve
correctly. A general "layer geometry query" (bounds center, transform-chain
resolution, live positions) would lift all three but is a real renderer
feature with its own cost — the spec should either state these constraints
or scope that feature explicitly.

## Relationship to RFC 0001

Complementary, not competing. State-declared slots are the general
*styling* substrate — state looks, live-bound gauges, cursor effects,
tweened themes-of-one. `DragAndDrop` is a *gesture* primitive for the one
interaction pattern that state machines express only with heavy plumbing.
The two compose: a zone's `actions` can drive guarded transitions into
states whose declared slots restyle the scene (`docked_count == 4` →
celebration state).

The honest cost, carried over from the exploration log: this is the spec's
first **stateful** interaction — a hidden per-object gesture machine running
beside the main one — and every conforming player must implement identical
gesture semantics (capture, touch vs. mouse, cancellation). That parity
burden is the price of the authoring economy above.

## Open questions

- **Interplay with the main machine**: what happens to a Held object when a
  transition changes the animation, a theme resets slots, or the machine
  stops mid-gesture? Prototype behavior is "gesture writes win until the
  slot is clobbered"; the spec needs a defined answer.
- **Observability**: the gesture currently fires nothing on grab / return /
  miss. `onGrab` / `onReturn` action lists (mirroring `dropZones.actions`)
  would let files react to gesture edges without host help.
- **Multi-pointer**: phases are per-interaction, so simultaneous drags of
  *different* objects are structurally supported, but pointer identity is
  not yet modeled (two touches = one pointer stream today).
- **Scope discipline**: v1 deliberately excludes drag bounds, axis locking,
  hover-over-zone feedback, z-order lifting, and inertia. Each is a
  plausible future field; the knob-creep boundary should be set explicitly.

## Prototype status

Implemented in dotlottie-rs: new interaction variant, per-interaction
gesture runtime, non-blocking snap tween, and load-time extraction of
authored layer positions for snap targets. Validated headlessly
(`tests/dnd_interaction_check.rs`: off-center grab with offset, drag, miss
→ glide home, dock → snap to the zone layer's derived position, zone
actions firing, lock) with a runnable demo (`examples/star_drop_dnd.rs`,
compare `examples/star_drop.rs` for the RFC 0001 version). Full test suite
green.

One engine finding worth spec-level awareness: gesture tween writes must be
applied *before* the frame renders each tick, or hit-testing operates on a
one-frame-stale scene — exactly the class of subtlety that makes defining
the gesture's timing model part of the spec work, not an implementation
detail.
