# Paint-fx POC — findings & recommendations

Branch `feat/paint-fx-poc` · 2026-07-28
Scope: paint-level manipulation of the rendered Lottie (whole-scene + per-layer), use-case
examples, and an assessment of landing these as state-machine actions.

## 1. What exists on the branch

| Surface | Ops |
|---|---|
| Whole animation | `set_transform` (pre-existing) · `set_opacity` · `set_blend_mode` · effects: `add_gaussian_blur` / `add_drop_shadow` / `add_fill_effect` / `add_tint` / `add_tritone` / `clear_effects` · clip: `set_clip_rect` / `set_clip_circle` / `set_clip_path` (arbitrary bezier) / `clear_clip` · mask: `set_spot_mask` (feathered, alpha/inverse-alpha) / `clear_mask` |
| Per layer (by name) | `set_layer_transform` · `set_layer_opacity` · `set_layer_visible` · `set_layer_blur` · `set_layer_clip_rect` / `set_layer_clip_path` · `clear_layer_props` |
| Overlay shapes | `add_overlay(below)` → id · `set_overlay_path` (flat cmds/pts buffers) · `set_overlay_fill` / `set_overlay_fill_linear` / `set_overlay_fill_radial` (gradient stops) · `set_overlay_stroke` / `set_overlay_stroke_dash` / `set_overlay_transform` · `remove_overlay` / `clear_overlays` — capi-owned procedural geometry in the wrapping scene, z-ordered above or below the Lottie |
| Layer clones | `add_layer_clone(layer, below)` → id · `set_clone_transform` / `set_clone_opacity` · `remove_clone` / `clear_clones` — frozen `tvg_paint_duplicate` snapshots retained in the wrapping scene (canvas space, don't animate); replayed across reloads via a render-time heal (drag ghosts, spawned copies) |
| Queries | `hit_test(layer, x, y)` (OBB) · `hit_test_precise` / `intersects_layer` (RLE coverage via `tvg_paint_intersects_region`; hidden layers included unless `visible_only`) · `get_layer_aabb(layer)` (canvas-space box) · `get_layer_transform(layer)` / `get_layer_opacity(layer)` (animated values at the current frame, excluding user overrides) |

Architecture: the Lottie Picture is wrapped in a `Tvg_Scene` (self-ref'd; validated safe
against every canvas/drop interleaving). Whole-scene ops target the wrapper. Scene-level
state (opacity/blend/effect stack) is stored renderer-side and **replayed across animation
reloads**. Per-layer overrides live in a `LayerProps` map flushed every render: re-fetch
paint by name→id (pointers die on every frame rebuild), compose user values onto a cached
pristine base (invalidated on frame/tween/slot changes), then poke the picture's transform
once per flush so ThorVG's update traversal descends into the mutated subtree.

Demos: `examples/web/` — 7 use-case pages + index (see §5). Tests: `tests/paint_fx.rs`
(pixel-diff assertions incl. frame-change survival, blur detach, reload persistence).

## 2. Validated ThorVG mechanics (the load-bearing facts)

- **Update traversal skips clean Pictures and child dirt does not bubble up**
  (`PictureImpl::skip`, `tvgPaint.cpp:201`). Any mutation below the picture renders only if
  the picture itself is re-marked — hence the once-per-flush transform poke. Opacity-0 and
  visibility misleadingly work without it (draw-phase checks); nothing else does.
- **Layer scenes are destroyed and rebuilt on every effective frame change** (and tween, and
  slot apply). Cached `Tvg_Paint` pointers are use-after-free; effects attached to a layer
  die with it. Everything per-layer is therefore re-applied on flush. Cost: an accessor
  full-tree walk per layer per frame (see perf).
- **Effects are re-prepared on every canvas update** and force-damage their region — so
  animating effect parameters via clear+re-add costs nothing beyond the effect itself.
  capi has no parameter setters (C++ API is add-only too); replay is the only model.
- **Effect sigma scales with the scene's accumulated transform** (`sigma × scale`,
  `tvgSwPostEffect.cpp:162`). Wrapper-scene effects sit above the layout transform → canvas
  pixels. Layer effects sit below it → **composition units**. Asymmetry is real; per-layer
  behavior is arguably the better one (zoom-consistent).
- **Refcounts**: wrapper scene owns picture ref via `scene_add`; TvgAnimation self-refs the
  scene; drop order verified safe in all orderings (adversarial review §"verified-sound").
- **Clippers and mask targets update with the *parent* matrix** (`tvgPaint.cpp:242/256` pass
  `pm`, the paint itself gets `pm × tr.m`) — so clip/mask coordinates on the wrapper scene
  are canvas pixels, on a layer scene composition units. Same asymmetry as effect sigma.
  The clip stack reaches children **during the update traversal**, so scene-level clip/mask
  changes need the same picture poke as layer props (a clean picture short-circuits and its
  render data never picks up the new clip — verified by a failing pixel test, fixed).
- **Clip/mask ownership is hand-over**: `Paint::clip`/`mask` ref a fresh shape (0→1) and
  free the previous one on replace or clear; a shape that already has a parent is rejected.
  Fresh-shape-per-apply is therefore the only correct lifecycle — never retain the pointer.
  A new shape carries a full renderFlag, so replacing a clipper per frame self-invalidates
  (animating clip geometry needs no extra dirtying beyond the poke).
- **Scenes never skip their update traversal** (`SceneImpl::skip` returns false,
  `tvgScene.h:81`; children updated unconditionally) — the opposite of the Picture. So
  overlay shapes held as children of the wrapping scene mutate in place
  (`tvg_shape_reset` + `append_path` + setters) with **no poke**, and their ownership is
  the ordinary retained kind: `tvg_scene_add`/`insert` refs the shape, the raw pointer
  stays valid until `tvg_scene_remove`, and `insert(scene, shape, picture)` places it
  before the picture in paint order — genuine below-the-animation z-ordering (verified by
  a pixel test: a below-overlay is fully occluded by an opaque frame).
- **Hidden layers stay hit-testable** — `set_layer_visible(false)` acts at draw
  phase only; render data stays prepared, and `tvg_paint_intersects_region`
  still reports coverage unless `visibleOnly` is set. Validated by pixel test:
  invisible drop zones work. Intersects/AABB require prepared render data
  (post-update); reads between renders are fine.
- **A duplicated layer leaves the picture's subtree**, so `tvg_paint_duplicate`
  clones must carry the picture's comp→canvas transform themselves (baked
  `picture × layer` base; user transform composes on top). Clones are frozen
  snapshots — the loader never rebuilds them — and reload replay runs before
  the new layer tree exists, so the renderer re-creates missing clones lazily
  at render time (`has_clone` heal).
- **Upstream bug: clearing a stroke dash segfaults.** `strokeDash(nullptr, 0)`
  frees the pattern but leaves the stale `dash.length`, and the SW engine's
  dash guard checks `length > threshold` — the next stroke render dashes with
  a null pattern (`tvgSwShape.cpp:229`). Workaround: clear via a one-element
  zero pattern (length 0 → solid path). Worth an upstream fix.
- **Gradient fills are hand-over like clips**: `tvg_shape_set_gradient` makes
  the shape own the gradient and free the previous fill on replace; setting a
  color fill deletes an existing gradient. Fresh gradient per sync, never
  retain.
- **The capi path surface maps 1:1 onto flat buffers**: `tvg_shape_append_path(cmds, pts)`
  takes a `Tvg_Path_Command` array (Close/MoveTo/LineTo/CubicTo) and an interleaved point
  array — exactly a `Uint8Array` + `Float32Array` across the wasm boundary. A ~50-line JS
  `parseSvgPath` converts design-tool `d` strings; no path parsing exists in Rust (a
  4-command validation pass only). Lottie-internal paths remain out of reach (accessor ids
  only on layer scenes; paths rebuilt from keyframes every frame; slots have no path type).

### Perf (release, 512² SW canvas, scene.json, 150-frame playback)

| Scenario | ms/frame |
|---|---|
| Baseline playback | 0.50 |
| +1 layer prop | 0.48 |
| +3 layer props | 0.50 |
| Whole-scene blur σ6 (set once) | 1.96 |
| Whole-scene blur σ6 (clear+re-add every frame) | 1.92 |
| Whole-scene drop shadow | 2.93 |

Layer props are noise-level during playback (playback already rebuilds everything; the poke
only matters for paused re-renders). Effects have a real fixed cost — budget ~1.5 ms per
blur, ~2.4 ms per shadow at 512² SW; fine for one hero canvas, wrong for a grid of them.

## 3. Adversarial review — confirmed issues and their status

| Finding | Status |
|---|---|
| `clear_layer_props` left a permanent restore entry taxing every future frame | **Fixed**: restore applies once, entry dropped |
| Poke executed once per layer instead of once per flush | **Fixed**: hoisted after the flush loop (reuses `apply_user_transform`) |
| Slot/theme apply rebuilds layer scenes without invalidating the base cache → user props clobber fresh slot values | **Fixed**: `apply_slot`/`del_slot` clear the cache |
| Scene-level ops silently lost on animation reload; effects were fire-and-forget | **Fixed**: declarative renderer-side state, replayed on load |
| Frame-survival property untested (evidence captured, never asserted) | **Fixed**: real assertions vs clean-frame baselines |
| Audio unit test pushed the now-parented picture to a canvas (dead render path) | **Fixed**: pushes wrapper scene |
| **Masks/clips/mattes don't follow layer transforms** — mask geometry lives in sibling paints carrying their own matrix copy | **Open (upstream)**: per-layer transform is correct only on unmasked layers; document, or upstream a fix |
| Matte-source layers unreachable by name; duplicate layer names resolve to first match; unknown names silently no-op forever | **Open**: needs diagnostics (log-once) before SM landing |
| `tvg_picture_set_accessible(true)` would break Lottie lookups (LottieLoader lacks `access()`) | Documented trap — never enable |
| Threaded builds get joined on the caller thread inside the flush (shrinks pipelining when layer props active) | Accepted; wasm/default players are single-threaded |

## 4. Key semantics decisions (deliberate, worth keeping)

- **Compose, don't overwrite**: user layer values multiply onto animated values. Authored
  motion keeps playing under user overrides, and identity-compose is a bit-exact restore.
- **Set-semantics for layer ops** (`set_layer_blur(σ=0)` removes) — idempotent, safe to
  re-enter, natural for declarative states.
- **Per-layer blur replaces the layer's authored effect list while active** (clear+re-add is
  the only rebuild-proof policy). Authored layer effects are rare; documented trade-off.
- Layer transform pivots at the composition origin — callers conjugate around their own
  pivot (`T(c)·M·T(-c)`); composition-space coordinates are what animation authors know.

## 5. Use-case examples (examples/web/)

| # | Page | Story | Ops |
|---|---|---|---|
| 01 | depth-of-field | Rack focus by blurring individual layers | `set_layer_blur`, `set_layer_opacity` |
| 02 | parallax | Five layers translate at different rates under the cursor | `set_layer_transform` |
| 03 | layer-spotlight | Renderer-side hit test isolates the hovered layer | `hit_test`, layer opacity/blur |
| 04 | notification-badge | Events shake the bell, pop/clear the badge | layer transform/opacity, `clear_layer_props` |
| 05 | button-states | 5 UI states as a declarative table of paint values | scene opacity/tritone/shadow/blur |
| 06 | entrance-choreography | Staggered per-layer assembly the file never authored | layer opacity/transform |
| 07 | like-button | Playback control and paint ops composed | tritone, layer transform, `set_frame`/`play` |
| 08 | reference-morph | Designer-authored null layer as a queryable transform target; runtime morphs the bell onto it | `get_layer_transform`, `set_layer_transform` |
| 09 | circular-reveal | Material-style enter/exit: clip circle expands from the click point | `set_clip_circle`, `clear_clip` |
| 10 | progress-reveal | Rect clip on one layer turns the heart into a determinate progress meter | `set_layer_clip_rect` |
| 11 | spotlight-mask | Feathered radial alpha mask follows the cursor; inverse = cutout | `set_spot_mask`, `clear_mask` |
| 12 | path-clip-reveal | Designer-authored heart `d` string clips the scene; slider lerps its points into a circle (same cubic layout) | `parseSvgPath`, `set_clip_path` |
| 13 | ink-annotation | Smoothed Catmull-Rom ink strokes drawn above the running art; soft blob parked behind it (`below`) | `add_overlay`, `set_overlay_path`/`stroke`/`fill`, `remove_overlay` |
| 14 | look-at-hero | Spline-homepage choreography: multi-depth blobs follow the cursor at depth-scaled rates and turn to face it (damped look-at), byte-exact return on leave | `get_layer_transform`, `set_layer_transform` |
| 15 | drag-and-drop | Geometry-accurate grabs, a frozen clone ghost dragged above everything, gradient/dashed drop zones that light up on approach, spring snap or return-to-original | `hit_test_precise`, `add_layer_clone`, `get_layer_aabb`, overlay gradients/dash |
| 16 | motion-driver | The real motion.dev engine (CDN import) drives the blobs through a ~20-line `propEffect`-style sink: `springValue` pointer follows, an `animate()` sequence with relative `at` offsets, `press()` squash with `info.success` bounce, `scroll()` parallax — zero renderer changes | `layerEffect` (JS) over `get_layer_transform`, `set_layer_transform`, `hit_test_precise` |

Assets are purpose-built (`examples/web/assets/`): semantic layer names, ambient loops,
composition == canvas size so coordinates read 1:1. All pages browser-verified; no console
errors; ~30 fps with a single blurred 512² SW canvas.

## 6. State-machine landing sketch

The SM already has states, transitions, entry-action lists, tweened transitions, and
layer-name hit-testing — paint ops slot in as new action types. Proposed shapes:

**Clean fits (persist-until-changed, idempotent):**

```
SetLayerTransform { layer: String, transform: [f32; 9] }
SetLayerOpacity   { layer: String, opacity: u8 }
SetLayerVisible   { layer: String, visible: bool }
SetLayerBlur      { layer: String, sigma: f32, quality: u8 }   // sigma 0 removes
ClearLayerProps   { layer: String }
SetLayerClip      { layer: String, shape: ClipShape }          // comp units; empty removes
SetOpacity        { opacity: u8 }
SetBlendMode      { mode: BlendMode }                          // named enum, not raw u8
SetClip           { shape: ClipShape }                         // canvas px; None removes
SetMask           { spot: SpotMask }                           // feather + inverse; None removes
Overlay           { id: String, path: SvgPath, style: Style, below: bool }  // declare/replace
RemoveOverlay     { id: String }
```

`ClipShape` covers rect / circle / path, the path carried as SVG `d` data in the config
(designer-authored, parsed at the host boundary); compatible path layouts tween point-wise
(demo 12). Overlays use SM-scoped string ids mapped to renderer u32 handles internally.

Semantics: state-entry actions; values persist until another state changes them (the
renderer's per-frame re-apply is an implementation detail the SM never sees). `states own
paint values` — see the button-states example for the exact shape.

**Needs a design decision — effects:**

`add_*` accumulation is wrong for SM re-entry (re-entering a state must not stack blurs).
Recommend a single **`SetEffects { effects: Vec<Effect> }`** action with replace-semantics
over the stored effect stack; the renderer already holds the stack declaratively, so this
is a thin wrapper. Keep `add_*` as imperative conveniences only.

**Reference layers — keeping the SM an interaction tool, not a design tool:**

Interaction targets should be *authored in the file*, not hand-typed into SM configs.
Validated pattern: **null layers (ty:3)** are explicitly kept by ThorVG's builder
(exempt from the opacity-0 skip, `tvgLottieBuilder.cpp:1557`) as named, unrendered
scenes carrying their animated transform — a designer places `REF:dock` in AE, and the
runtime queries it per frame via `get_layer_transform` and morphs a target layer onto
it (`M = R·B⁻¹`, lerped; demo 08). Constraints discovered: zero-opacity normal layers
are *skipped entirely* (unqueryable) and AE-hidden (`hd:true`) layers are deleted at
parse — so references must be nulls (transform-only; null opacity is deliberately
ignored by the builder), or normal layers hidden at runtime via
`set_layer_visible(false)` when the reference must also carry opacity. Proposed action:

```
TweenToLayerReference { target: String, reference: String, duration: f32, easing: Easing }
```

The SM stores two layer names; all design data stays in the .lottie, so designers
iterate without touching the interaction graph.

### Envelope semantics — Spline as field evidence

The Spline docs crawl (`docs/spline-interactions-mapping.md`) is a usage survey of what
interaction designers are actually given, and therefore use. Mapped against what the SM
already has (`Transition::Tweened { duration, easing: [f32; 4] }`, typed guards, the
`Toggle`/`SetRandom`/`Increment` input actions, interruptible tweens from #592), the T3
gaps rank as follows.

**1. Auto-revert (held) semantics** — Spline's Mouse Press, Key Press and Mouse Hover all
share one behavior: actions apply while the condition holds and *reverse themselves* when
it ends. Designers get hover states without authoring the exit path. Today that's a
hand-built pair of states; the sugar is one flag on entry actions:

```
entry_actions: [ { ...SetLayerBlur..., revert_on_exit: true } ]
```

Exit runs the stored inverse (`ClearLayerProps` / previous value) — cheap because every
paint action already has identity-compose restore semantics (§4). Covers the
badge-shake/"override for N ms" pattern too when combined with delay (below).

**2. Toggle mode** — Spline supports alternating forward/reverse on Mouse Down/Up, Key
Down/Up, Collision and Trigger Area; it's their most-documented interaction pattern. The
SM already has `Action::Toggle` on a boolean input plus guards — a two-state toggle works
today but takes four config blocks. Proposed sugar: `mode: "toggle"` on a transition,
desugaring to exactly that pair internally. No engine change, parser-level only.

**3. Delay, loop, cycle on tweens** — Spline's *only* sequencing primitives are per-action
`delay` (present on nearly every action), transition `loop` (count | infinite) and `cycle`
(ping-pong). No timeline, no sequence node — evidence that these three cover real
choreography (our demo 06 entrance stagger is pure delay). Proposed:

```
Transition::Tweened { duration, easing, delay: f32, loop: LoopCount, cycle: bool }
Action::*            { ..., delay: Option<f32> }
```

Delayed actions need a tick-driven scheduler in the engine (the tween driver already owns
per-tick time); loop/cycle are envelope state on the existing tween.

**4. Spring easing** — Spline ships `Spring` beside the five bezier presets, and every
compelling pointer demo here (02, 14, 15) used the 6-line critically-damped spring, not a
bezier. The deeper argument is interruption: #592 made tweens interruptible, and a spring
carries velocity across interruption for free — retargeting mid-flight is its native
operation, where bezier retargeting needs the tween-cache machinery. Proposed:

```
easing: [x1, y1, x2, y2]                 // today, stays the default
easing: { spring: { stiffness, damping } }  // target-based, duration-free
```

Springs ignore `duration` (they settle); `cycle` still applies (retarget to origin).

**5. Dynamic inputs** — Spline's Timer / Stopwatch / Counter / Clock / Random variables
plus the Variable Control transport (play/pause/stop/restart/ping-pong). The SM's
`SetRandom` already covers static Random; the rest is one new input family that the
engine ticks:

```
Input::Timer   { name, from, to, interval, step, on_end: Restart | Stop }
Action::TimeControl { input_name, op: Play | Pause | Stop | Restart | PingPong }
```

Value changes flow through existing guard evaluation — Spline's idle-timeout,
countdown-UI and slideshow patterns fall out with no new transition machinery.

**6. Continuous input binding** — parallax/look-at/scroll-scrub need per-tick value flow
(`$pointer.x` → layer transform factors), not edge-triggered transitions. Spline models
these as dedicated events (Look At, Follow, Scroll) rather than generic binding — worth
copying, because it keeps configs declarative and bounded:

```
behaviors: [ { type: "FollowLayer", layer, rate_x, rate_y, damping },
             { type: "LookAtLayer", layer, strength, damping },
             { type: "ScrollScrub", from_frame, to_frame } ]
```

Behaviors are state-scoped (active while the state is), evaluated in the SM tick before
render — demos 02/14 are the reference implementations, ~15 lines each over the wrapped
surface. This subsumes the general binding design without an expression DSL.

**7. Key events** — trivially missing: `Event::KeyDown/KeyUp { key }` beside the pointer
events; hosts already own the event feed. Spline treats key and mouse as symmetric
(same held/toggle modifiers), which the envelope flags above give us for free.

**Deliberately not adopted**: Spline's expression language (arithmetic in conditionals /
Set Variable). The existing guard set plus `Increment`/`Multiply`/`Clamp` actions covers
Spline's documented examples (`A + B`, `A * 10 / 2`); a real DSL is a security and
complexity cliff the SM's typed actions were designed to avoid — revisit only with
concrete demand. Physics stays host-side (§7 of the mapping report).

**Priority order** (engine-change size × Spline-evidenced usage): toggle sugar (parser
only) → auto-revert flag → delay scheduler + loop/cycle → spring easing → key events →
dynamic inputs → behaviors. The first two are config-format decisions and should land
with the initial action set; behaviors can trail as a second round.

**Blockers to close before landing:**
1. C API parity (all ops are wasm/Rust-only today; SM runs on native).
2. Named enums (`BlendMode`, `Effect`) for the declarative format + error variants per the
   #593 conventions (typed, no unread payloads).
3. Layer-name diagnostics (unknown name, duplicate name, matte source) — log once.
4. Document the mask/clip/matte transform limitation; decide whether to upstream a fix.

## 7. Unwrapped capi surface — levers for richer interactions

Every `tvg_paint_*` function is already linked (bindgen has no allowlist); these are
unwrapped but one vertical slice away. Mapped to the interaction use cases they unlock:

| capi | Unlocks | Notes |
|---|---|---|
| `tvg_paint_intersects` / `intersects_region` | **Now wrapped** (`hit_test_precise` / `intersects_layer`, demo 15): geometry-accurate grabs and region overlap. Hidden-layer drop zones **validated** — hidden is draw-phase only, coverage still reported unless `visible_only` | Requires prepared render data (post-update) |
| `tvg_paint_get_aabb` | **Now wrapped** (`get_layer_aabb`, demo 15): zone overlap, snap targets, drag bounds | Canvas space, reflects composed user transforms |
| `tvg_paint_duplicate` | **Now wrapped** (`add_layer_clone`, demo 15): drag ghosts rendered above everything, spawned copies | Frozen snapshot; comp→canvas base baked in; heal-on-render replay |
| `tvg_paint_set_mask_method` / `set_clip` | **Now wrapped** (scene clip/spot-mask + per-layer clip; rect, circle and arbitrary bezier paths; demos 09–12). Still unwrapped: masking one *layer by another layer* (rejected today: a parented paint can't be a mask target) | Layer-as-mask would need duplication or upstream support |
| `tvg_shape_*` path/fill/stroke surface | **Now wrapped** via `ClipRegion::Path` + overlay shapes (demos 12–13, 15): append_path, fill color, linear/radial gradients, stroke width/color/dash, per-shape transform | Still unwrapped on overlays: trim, fill rule, picture overlays; `tvg_shape_get_path` read-back unused. Dash-clear upstream bug worked around (§2) |
| `tvg_paint_get_parent` / `get_type` / `get_id` | Tree introspection — enumerate/validate layer targets, diagnostics for unknown names | Pairs with the layer-name diagnostics blocker (§6) |
| `tvg_paint_ref/unref/get_ref/rel` | Lifecycle plumbing (already used for the wrapper scene) | Internal-only; not API surface |
| `tvg_paint_translate/scale/rotate` | Convenience transforms | Redundant — they overwrite rather than compose; our matrix path is strictly more capable |

Drag-and-drop sketch with this surface: designer authors draggable layer + (hidden)
drop-zone layers; runtime = `hit_test`/`intersects` to grab, `set_layer_transform` to
follow the pointer, `intersects_region(zone, dragged-AABB)` to detect hover-over-zone,
reference-layer morph (§6) to snap into the slot. Every piece except `intersects`/`aabb`
exists on the branch today.

## 8. Recommendations, in priority order

1. Land the examples + report as the POC deliverable; gather feedback on which use cases
   matter (per-layer blur and layer transforms carried every compelling demo).
2. If SM landing proceeds: implement `SetEffects` replace-semantics + named enums first —
   they change the public shape; everything else is additive.
3. Upstream conversation with ThorVG: (a) mask/clip should follow the layer scene
   transform, or expose a supported "layer override" hook; (b) an id→paint map in the
   Lottie loader (or a real `access()` impl) to kill the per-frame accessor walk.
4. Keep effects budgeted: one effect-bearing canvas per view on SW; GL/WGPU support all
   five effects if more is needed.
