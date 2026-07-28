# Paint-fx POC — findings & recommendations

Branch `feat/paint-fx-poc` · 2026-07-28
Scope: paint-level manipulation of the rendered Lottie (whole-scene + per-layer), use-case
examples, and an assessment of landing these as state-machine actions.

## 1. What exists on the branch

| Surface | Ops |
|---|---|
| Whole animation | `set_transform` (pre-existing) · `set_opacity` · `set_blend_mode` · effects: `add_gaussian_blur` / `add_drop_shadow` / `add_fill_effect` / `add_tint` / `add_tritone` / `clear_effects` |
| Per layer (by name) | `set_layer_transform` · `set_layer_opacity` · `set_layer_visible` · `set_layer_blur` · `clear_layer_props` |
| Queries | `hit_test(layer, x, y)` · `get_layer_transform(layer)` / `get_layer_opacity(layer)` (animated values at the current frame, excluding user overrides) |

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
SetOpacity        { opacity: u8 }
SetBlendMode      { mode: BlendMode }                          // named enum, not raw u8
```

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

**Interpolation** (rack focus, fade-ins, springy hovers): two options, both grounded in
existing machinery:
1. Transition-level envelopes: reuse the tweened-transition driver to interpolate action
   params (duration + easing per transition) — covers 80% of the examples.
2. Continuous input binding (`$pointer.x` → layer transform factors) for parallax-class
   interactions — same requirement pointer-driven frame tweening already has.

**Event envelope**: the badge-shake pattern (override for N ms, then auto-restore) suggests
an optional `duration` on layer actions, after which `ClearLayerProps` fires implicitly.

**Blockers to close before landing:**
1. C API parity (all ops are wasm/Rust-only today; SM runs on native).
2. Named enums (`BlendMode`, `Effect`) for the declarative format + error variants per the
   #593 conventions (typed, no unread payloads).
3. Layer-name diagnostics (unknown name, duplicate name, matte source) — log once.
4. Document the mask/clip/matte transform limitation; decide whether to upstream a fix.

## 7. Recommendations, in priority order

1. Land the examples + report as the POC deliverable; gather feedback on which use cases
   matter (per-layer blur and layer transforms carried every compelling demo).
2. If SM landing proceeds: implement `SetEffects` replace-semantics + named enums first —
   they change the public shape; everything else is additive.
3. Upstream conversation with ThorVG: (a) mask/clip should follow the layer scene
   transform, or expose a supported "layer override" hook; (b) an id→paint map in the
   Lottie loader (or a real `access()` impl) to kill the per-frame accessor walk.
4. Keep effects budgeted: one effect-bearing canvas per view on SW; GL/WGPU support all
   five effects if more is needed.
