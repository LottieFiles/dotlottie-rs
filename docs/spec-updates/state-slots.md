# Spec Exploration: State-Declared Slots (with Transition Tweening)

Status: **draft — prototyped in dotlottie-rs, 2026-07-13** (decisions from
grilling session, 2026-07-09; see "Prototype results" below)

A standalone spec direction, independent of the slot-actions proposal
([slot-actions.md](./slot-actions.md)). Here, slot values are **part of a
state's configuration** — declared alongside `loop`, `segment`, `autoplay` —
and the *transition* determines how the visual change arrives: instantly for a
plain `Transition`, interpolated over time for a `Tweened` transition.

## Motivation

States already declare their playback look (loop, speed, segment, mode). This
extends the same declarative idea to styling: a state declares what its slots
look like, and moving between states smoothly morphs colors, positions, sizes
— the way the frame pose already morphs during a `Tweened` transition. The
author thinks "state = look", never "when do I fire a write".

## Decisions

### D1 — Slots are state configuration; tweening rides the transition

`PlaybackState` gains an optional `slots` array declaring the state's slot
values:

```json
{
  "type": "PlaybackState",
  "name": "alert",
  "animation": "a1",
  "loop": true,
  "slots": [
    { "slotId": "accent", "type": "Color", "value": [1, 0, 0] }
  ]
}
```

- Entering via a plain `Transition` → declared values apply instantly, with
  the rest of state configuration.
- Entering via a `Tweened` transition → interpolable values interpolate from
  their current effective values to the declared values, reusing the
  transition's `duration` and cubic-bézier `easing` — the same clock that
  drives the frame-pose tween. Per-transition control falls out for free:
  entering the same state can be fast from one state and slow from another.

This deliberately piggybacks the engine's existing tween lifecycle (one clock,
transition-owned timing, existing `Tweening` status semantics) rather than
introducing an independent property-tween runner.

### D2 — Scoped lifetime: declared slots release to base on exit

A state's declared slot values live exactly as long as the state is current.
Transitioning to a state that does not declare a given slot releases it to its
**base value**: the active theme's value if a theme rule covers the slot,
otherwise the authored animation value. On a `Tweened` exit, released slots
tween *back* to base with the same transition clock — enter and exit are
symmetric.

This is what "state configuration" already means in the engine: undeclared
`loop`/`speed`/`segment` reset to defaults on every entry rather than
inheriting the previous state's values. Same contract, applied to styling.
It also kills the sticky-model bug class ("passed through `alert` once, the
red followed me forever"). Rejected alternatives (sticky last-write-wins,
sticky with explicit per-slot release) in the alternatives doc.

Implementation note: provenance tracking is cheap — the engine knows exactly
which slots the current state declared; base values come from the renderer's
tracked theme writes and `extract_slots_from_animation` for authored values.

### D3 — Interpolable types only: Color, Scalar, Vector, Position

State slots accept the four numerically interpolable slot types. Text and
Image are **excluded**: everything a state declares can tween, so the
mechanism never needs a discrete-snap rule. The cost is accepted: per-state
text/image looks are not expressible in this spec direction (see alternatives
doc for the snap-timing analysis — CSS's 50% rule — if this is revisited).
Gradient is likewise out (consistent with its deferral everywhere else).

### D4 — Cross-animation transitions apply instantly (forced)

The engine only tweens between states sharing the same animation; a `Tweened`
transition to a different animation already degrades to an instant switch,
and the outgoing animation's slot state does not survive the load. State
slots inherit this: different target animation → declared values apply
instantly with the rest of state configuration. No new rule, just inherited
behavior made explicit.

### D5 — GlobalState carries no slots (forced)

`GlobalState` is never the current state — it holds transition rules that
redirect to playback states. There is no "while in this state" for a scoped
overlay to live in, so `slots` is a `PlaybackState`-only property, exactly
like the playback settings it sits beside.

### D6 — Input references are live-bound while the state is current

A declared value may reference Numeric inputs (`"$heat"`) per element. The
declaration is a **standing rule, not a write**: whenever a referenced input
changes while the declaring state is current, the slot re-applies immediately
(no tween — tweening belongs to transitions). The binding is established at
state entry and dies at exit, consistent with the scoped lifetime (D2).

- Refs resolve at tween start when entering via a `Tweened` transition; input
  writes are already rejected during tweening, so a binding cannot fire
  mid-tween.
- `@` globals (e.g. `@elapsedTime`) are **not** allowed in state slot values:
  bindings re-fire on input writes, and globals tick continuously — allowing
  them would silently promise per-frame re-evaluation (an animation driver,
  not a binding).

Implementation note: build an input→slot-entry map at state entry; on input
write, re-resolve affected entries. Slot flushing is already batched and
dirty-flagged per render.

### D7 — Dedicated entry shape

```json
"slots": [
  { "slotId": "accent", "type": "Color",  "value": ["$heat", 0, 0] },
  { "slotId": "scale",  "type": "Vector", "value": [1.2, 1.2] }
]
```

- `slotId` (required): target slot ID.
- `type` (required): `Color` | `Scalar` | `Vector` | `Position`. Explicit
  because 2-element values are ambiguous (Vector vs Position).
- `value` (required): Color = 3 or 4 elements in 0–1; Scalar = single
  number; Vector/Position = 2 elements. Every element is a number literal or
  a `$` Numeric input reference.

Deliberately **not** the theme-rule schema: theme rules carry
keyframes/expressions/animations that state slots must reject, and state
slots carry `$refs` that theme rules must reject — honestly separate beats
compatible-except-where-it-isn't.

### D8 — Failures are silent no-ops, per entry

Unknown `slotId`, type mismatch with the animation's slot, or unresolvable
reference → that entry does nothing; the rest of the state's slots (and the
transition) proceed. While live-bound, an entry whose ref becomes
unresolvable keeps its last applied value rather than flapping. Players MAY
surface diagnostics (non-normative) — same posture as the slot-actions
proposal.

## Implementation feasibility (verified against dotlottie-rs)

The model rides existing machinery; no architectural change required:

- **Clock/easing**: eased progress is already computed engine-side per tick
  (`TweenState::update`, cubic-bézier solver in `tween.rs`) and currently
  feeds only the ThorVG frame-pose blend. Slot lerps consume the same
  progress value — synchronization with the pose blend is by construction.
  Only gap: `player.tick` discards progress; expose a `tween_progress()`
  getter.
- **Per-tick application**: `StateMachineEngine::tick` already runs per frame
  and owns the tween-resume handshake. Slot setters only mutate the
  renderer's tracked `slot_values` + dirty flag; the ThorVG flush is already
  batched once per render (`flush_slots`). Cost: one JSON serialize + 3 FFI
  calls per tween tick.
- **Setup/teardown**: the `Tweened` branch of `set_current_state` (where the
  target frame is computed) builds the lerp list — declared slots plus
  release-to-base entries, giving D2's tween-back for free;
  `resume_from_tweening` snaps finals and installs live bindings.
  Cross-animation needs nothing (the `same_animation` check already degrades
  to instant).
- **From/base values**: renderer's `slot_values` map for anything previously
  written; `extract_slots_from_animation` for authored values. Live-binding
  hooks the three input setters, which already reject writes during
  tweening (D6's no-fire-mid-tween rule is enforced by existing code).

Known gaps: (1) authored slots with *keyframed* values have no engine-side
evaluator to sample at the current frame — first implementation snaps that
edge case; (2) slot values changing during an in-flight ThorVG pose blend is
an unexercised path — spike early to confirm ThorVG re-evaluates slotted
properties per blended pose.

## Prototype results (2026-07-13)

The full model was implemented as a test-purpose prototype in dotlottie-rs
and validated with integration tests against the real ThorVG software
renderer (`tests/state_machine_state_slots.rs`, `bouncy_ball.json` slots).
All six pass, full suite green:

- instant apply on entry, with `$ref` resolution (D1 plain-transition path)
- redeclare chains preserving the original base + partial release (D2)
- live binding re-applying on input writes, dying on exit (D6)
- tweened entry interpolating authored → declared on the transition clock (D1)
- tweened exit interpolating declared → authored base symmetrically (D2)
- all failure modes as silent no-ops, including `@`-global rejection (D8, D6)

Implementation shape matched the feasibility analysis: ~1 getter each on
`TweenState`/`Player`/renderer trait, a `slots` field on `PlaybackState`, one
new `state_slots` module, and four hook points in the engine
(`set_current_state`, `resume_from_tweening`, `tick`, input setters). No
architectural changes, no new blocking rules, no separate tween runner.

Spike findings on the two known gaps: ThorVG accepted per-tick slot batch
regeneration during an in-flight pose blend without errors (rendered every
tween tick through the software target); visual confirmation that the
blended pose picks up mid-tween slot values still deserves an eyeball check
in a real player. Keyframed authored values (ball_position, ball_scale)
correctly fall back to instant apply / snap release rather than lerping.

## Summary of the model

A `PlaybackState` optionally declares a scoped, live-bound styling overlay of
interpolable slot values. Plain transitions snap the overlay in; `Tweened`
transitions morph it in and out on the transition's own clock, alongside the
frame-pose blend the engine already performs. Leaving the state releases the
overlay back to the theme/authored base. Nothing here requires host code, new
blocking rules, or a second tween runner.
