---
default: minor
---

# feat: motion API — animate named layers at runtime

A motion.dev-style imperative animation and compositing API over named Lottie layers.
The authored animation is the document; the motion API styles and animates on top of it
at runtime, driven by the player clock in the Rust core.

New player surface:

- `animate(name, keyframes, options)` — spring (closed-form, velocity handoff on
  interruption) and cubic-bezier tween transitions, keyframe waypoints, delays; returns
  an animation id with `animation_pause/resume/stop/cancel/set_speed` controls and a
  `MotionComplete` event when every property settles.
- `set_node_props` / `get_node_props` / `reset_node` / `reset_nodes` — instant retained
  overrides, composed against the pristine animated base every frame (idempotent,
  absolute values).
- Animatable properties: `x`, `y`, `rotate`, `scaleX/Y`, `opacity`, `visible`,
  `blend`, per-layer gaussian `blur`, duotone `tint`, feathered `spot` alpha mask, and
  rect/circle `clip` — grouped props animate via dotted keys (`spot.cx`, `clip.r`,
  `tint.intensity`).
- `@stage` — a reserved node targeting the whole animation (wrapper scene): whole-canvas
  grading, iris clips, and cursor spotlights use the same props as any layer, in canvas
  coordinates.
- `duplicate_node` / `remove_node` — deep-copy a layer at its current pose into the node
  namespace; duplicates animate like any node (ghost trails, stamps).
- `animate_value` / `animation_value` — raw driver-clocked values for syncing DOM or
  app state with canvas motion.
- `layers()` — authored layer names in document order (precomp children depth-first),
  enumerated from ThorVG's Lottie model; every name is a valid node target.

The wasm bindings expose the full surface with JS-object keyframes/options
(`player.animate("arm", { rotate: 30 }, { type: "spring", bounce: 0.3 })`). A 12-demo
showcase lives in `motion-showcase.html` with purpose-built assets in `motion-assets/`.

Overrides persist across frames, loops, and segments; a new animation load clears them.
While a transform/effect override is active on a layer, that layer's authored effects
are suppressed (upstream ThorVG limitation).
