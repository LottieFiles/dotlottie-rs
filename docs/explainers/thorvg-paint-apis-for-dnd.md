# ThorVG paint APIs for DragAndDrop

**Audience**: an implementing agent modifying the DragAndDrop runtime in this
repo. Read this before touching `src/state_machine_engine/mod.rs` or
`src/lottie_renderer/`.

**Purpose**: DragAndDrop currently moves objects by writing a Lottie
**Position slot**. ThorVG exposes paint-level APIs that move, query, hide,
clip, and duplicate a *rendered layer* directly. This doc lists those APIs,
their exact semantics, the traps that silently break them, and how each maps
onto the DnD gesture.

**Provenance**: every claim was verified against the vendored ThorVG in
`dotlottie-rs/deps/thorvg` (`v1.0.0-314-ge8591f5e`) and against a working
prototype — the `feat/paint-fx-poc` branch of dotlottie-rs, which wraps all of
these APIs and covers them with pixel-diff tests in
`dotlottie-rs/tests/paint_fx.rs`. Where this doc says "the POC", that is the
reference implementation to copy from. C API declarations cited below live in
`deps/thorvg/src/bindings/capi/thorvg_capi.h`.

---

## 1. The mental model

A loaded Lottie is a ThorVG `Picture`. Inside it, the Lottie builder produces
one `Scene` per layer, each tagged with `id = djb2Encode(layer_name)`. You
reach a layer's scene with `tvg_picture_get_paint(picture, id)` and then treat
it as an ordinary paint: transform it, hide it, blur it, clip it, hit-test it,
duplicate it.

Two facts govern everything else:

1. **The builder regenerates every layer scene on each effective frame change.**
   `LottieBuilder::updateLayer` does `layer->scene = Scene::gen()` every build
   (`deps/thorvg/src/loaders/lottie/tvgLottieBuilder.cpp:1560`). A `Tvg_Paint`
   pointer to a layer is therefore **valid only until the next frame change**.
   Never cache one. Re-fetch by name→id every time you touch a layer.
2. **Anything you set on a layer scene is wiped by that rebuild.** Your
   transform, opacity, blur and clip die with the old scene. So per-layer
   overrides must be *re-applied every render*, composed onto whatever the
   animation produced for the new frame.

The POC implements exactly this: a `BTreeMap<String, LayerProps>` of user
overrides, flushed on every render, each composed onto a cached *pristine
base* read from the freshly built scene.

---

## 2. Coordinate spaces — read this twice

This is the single largest source of wrong code, and the current DnD
implementation has a live bug here (§7).

| Space | What it is |
|---|---|
| **Composition units** | The Lottie's authored coordinate system, sized `w`/`h` in the animation JSON (e.g. 512×512, or 1500×1500). Slot position values are in this space. |
| **Canvas pixels** | The output surface. Pointer events, `tvg_paint_get_aabb`, `tvg_paint_get_obb`, and `tvg_paint_intersects_region` are all in this space. |

The two are related by the **layout matrix**, computed in
`src/layout.rs:36` (`Layout::to_transform_matrix`) and applied to the Picture
in `src/lottie_renderer/mod.rs:332` (`apply_layout_transform`). The picture is
kept at its authored comp size and the layout matrix scales/offsets it. They
coincide **only** when canvas size == comp size and fit is 1:1.

Now the rule that decides which space each API speaks:

- A **layer** scene's own matrix maps layer-local → its parent (the comp root,
  or an enclosing precomp). Composing a user matrix onto it (`composed = user ×
  base`, ThorVG is column-vector: `p' = M·p`, so `base` applies first) means
  **your matrix acts in composition units**.
- A **clone** (§5.2) leaves the picture's subtree, so the POC bakes
  `picture_matrix × layer_matrix` into its base. Composing onto *that* means
  **your matrix acts in canvas pixels**.
- The **wrapping scene** (§5.1) sits above the layout transform, so scene-level
  clip, mask, effects and overlays are all **canvas pixels**.

> **Implementation requirement**: `set_layer_transform` takes comp units.
> Pointer deltas are canvas pixels. You must divide by the layout scale.
> Expose the scale from the renderer (it already computes the layout matrix)
> rather than recomputing it in the state machine engine. The same conversion
> is needed for the *existing* slot-writing code and is currently missing.

The same asymmetry applies to blur sigma and clip geometry: scene-level =
canvas px, layer-level = comp units.

---

## 3. Query APIs — picking and bounds

### 3.1 `tvg_paint_intersects_region` — geometry-accurate hit test

```c
bool tvg_paint_intersects_region(Tvg_Paint paint, int32_t x, int32_t y,
                                 int32_t w, int32_t h, bool visibleOnly);
```
*(`thorvg_capi.h:1143`, marked Experimental. A non-`visibleOnly` variant,
`tvg_paint_intersects`, exists at `:1114`.)*

- **Space**: canvas pixels. `w`/`h` must be > 0; a point test is `w=1, h=1`.
- **Semantics**: tests the region against the paint's *actual filled area*
  (RLE coverage), not its bounding box. Internally does an AABB reject first,
  so the fast path is cheap.
- **Precondition**: "The paint must be updated in a Canvas beforehand" — i.e.
  after a draw+sync. Results are as of the last canvas update.
- **`visibleOnly = false`** includes paints hidden via
  `tvg_paint_set_visible(false)`. This is how you get **invisible drop zones**.
- **Does not** account for blending or masking.

POC wrappers: `Player::hit_test_precise(layer, x, y, visible_only)` and
`Player::intersects_layer(layer, x, y, w, h, visible_only)`.

**Why DnD wants it**: today `hit_test` (`src/lottie_renderer/thorvg.rs:643`)
projects the point onto the layer's oriented bounding box. A 5-point star
grabs its empty corners; two overlapping draggables steal each other's grabs.
Region form additionally lets a drop be "the dragged object overlaps the zone"
instead of "the pointer is inside the zone" — closer to what users expect from
a large dragged object.

**Caveat on invisible zones**: `visibleOnly=false` covers layers hidden at
*runtime*. It does **not** resurrect layers the Lottie builder pruned — a
layer authored at opacity 0 is skipped entirely by `updateLayer`
(`tvgLottieBuilder.cpp`, "full transparent scene. no need to perform") and has
no paint at all. RFC 0002's "keep dock markers at ≥1% opacity" guidance still
holds for authored transparency; the new capability is that *you* can hide a
layer at runtime and still hit-test it.

### 3.2 `tvg_paint_get_aabb` / `tvg_paint_get_obb` — rendered bounds

```c
Tvg_Result tvg_paint_get_aabb(Tvg_Paint paint, float* x, float* y, float* w, float* h);
Tvg_Result tvg_paint_get_obb(Tvg_Paint paint, Tvg_Point* pt4);  // 4 corners
```
*(`thorvg_capi.h:1165`, `:1187`)*

- **Space**: canvas pixels, all transforms applied.
- `INSUFFICIENT_CONDITION` on failure (usually invalid path data) — always
  check the return, the out-params are garbage otherwise.
- AABB is cheaper and simpler; OBB survives rotation. This repo already uses
  OBB via `Player::layer_bounds`. **Keep OBB for the boundary clamp** (it
  projects onto the box's own edge vectors, so rotated boundaries work) and use
  AABB where you only need half-extents or a center.

POC wrapper: `Player::get_layer_aabb(layer) -> Option<[x,y,w,h]>`.

### 3.3 Reading a layer's animated transform / opacity

```c
Tvg_Result tvg_paint_get_transform(Tvg_Paint paint, Tvg_Matrix* m);
Tvg_Result tvg_paint_get_opacity(const Tvg_Paint paint, uint8_t* opacity);
```
*(`thorvg_capi.h:1055`, `:1074`)*

POC wrappers: `Player::get_layer_transform(layer) -> Option<Vec<f32>>` (row-major
3×3) and `Player::get_layer_opacity(layer) -> Option<u8>`. Both return the
**pristine animated value excluding user overrides**, because the POC caches
the base before composing.

**Why DnD wants it**: it replaces `dnd_slot_position`
(`src/state_machine_engine/mod.rs:987`), which requires the slot to exist and
to hold a static 2D value. Reading the layer works for any layer — including
`ty: 3` null "reference" layers, which the builder keeps in the tree
(`if (layer->type != LottieLayer::Null && layer->cache.opacity == 0) return;`)
even though they render nothing. A designer can therefore author an invisible
null as a moving anchor and the engine can read its position every frame.

---

## 4. Layer mutation APIs

All of these operate on the paint returned by
`tvg_picture_get_paint(picture, djb2_id(layer_name))`, and all must be
re-applied every render (§1).

| C API | POC wrapper | Space | Note |
|---|---|---|---|
| `tvg_paint_set_transform` (`:1044`) | `set_layer_transform` | comp units | Compose onto the animated matrix, don't replace |
| `tvg_paint_set_opacity` (`:1065`) | `set_layer_opacity` | — | POC multiplies: `composed = base * user / 255` |
| `tvg_paint_set_visible` (`:945`) | `set_layer_visible` | — | Draw-phase only; the paint still updates and stays hit-testable |
| `tvg_scene_add_effect_gaussian_blur` (`:2292`) | `set_layer_blur` | sigma in **comp units** | Layer scenes are Scenes, so scene effects attach directly |
| `tvg_paint_set_clip` (`:1226`) | `set_layer_clip_rect` / `set_layer_clip_path` | comp units | See ownership below |

**Composition, not replacement.** The POC reads the freshly-built layer matrix
once per rebuild as `base`, then writes `user × base`. This is what lets an
object be dragged *while its authored animation still plays* — something a
Position slot write cannot do, because it clobbers the authored value outright.

**Clip/mask ownership is hand-over.** `tvg_paint_set_clip` and
`tvg_paint_set_mask_method` take ownership of the shape you pass: ThorVG refs
it and frees the previous one on replace or clear. Both return
`INSUFFICIENT_CONDITION` if the shape already has a parent. So: **always build
a fresh shape per apply, never retain the pointer.** A fresh shape also carries
a full render flag, so animating clip geometry self-invalidates.

**Blur caveat**: attaching a user blur clears the layer's authored effect list
(clear-then-add is the only way to guarantee exactly one user blur across
rebuilds). A layer with authored effects loses them while a blur override is
active.

**Mask/matte caveat**: layers with masks, mattes or track mattes keep their
mask geometry in *sibling* paints. Transforming the layer moves the content but
not the mask. **Per-layer transform overrides are only fully correct on
unmasked layers.** If DnD adopts layer transforms, either document this
constraint for draggables or use the clone approach (§5.2), which does not have
it.

---

## 5. New paints: the wrapping scene, overlays, clones

The POC wraps the Lottie Picture in a `Tvg_Scene`:

```
canvas
└── wrapper Scene            <- scene opacity / blend / effects / clip / mask
    ├── overlay shapes (below)
    ├── Picture (the Lottie) <- layout transform lives here
    ├── clones
    └── overlay shapes (above)
```

Everything in the wrapper is **canvas pixels** and is never rebuilt by the
Lottie builder (`SceneImpl::skip` returns false — scenes always traverse), so
state you put there survives frame changes without re-application.

### 5.1 Overlay shapes — procedural geometry

`tvg_shape_new` + `tvg_shape_append_path` (`:1440`) + fills
(`tvg_shape_set_fill_color`, `tvg_linear_gradient_*` `:1782`,
`tvg_radial_gradient_*` `:1828`) + strokes with dashes
(`tvg_shape_set_stroke_dash` `:1558`), inserted with `tvg_scene_add` (`:2219`)
or `tvg_scene_insert` (`:2248`, to place below the Picture).

POC wrappers: `add_overlay(below) -> id`, `set_overlay_path`,
`set_overlay_fill{,_linear,_radial}`, `set_overlay_stroke`,
`set_overlay_stroke_dash`, `set_overlay_transform`, `remove_overlay`,
`clear_overlays`.

Path encoding is flat buffers: `cmds` are `Tvg_Path_Command` values
(0 Close, 1 MoveTo, 2 LineTo, 3 CubicTo), `pts` is interleaved x,y. Point
counts must match (MoveTo/LineTo = 1 point, CubicTo = 3, Close = 0) and the
first command must be MoveTo.

**Why DnD wants it**: drop-zone highlight rings, snap guides, drag trails, path
previews — none of which need to exist in the animation. Dashed stroke +
gradient fill + a hover restyle is a few calls, as in the POC's
`examples/web/drag-and-drop.html`.

> One upstream bug is worked around in the POC: clearing a dash with `cnt=0`
> leaves a stale `dash.length` and the SW engine then dashes against a null
> pattern (segfault). Clear with a single zero-length entry instead.

### 5.2 Layer clones — the drag ghost

```c
Tvg_Paint tvg_paint_duplicate(Tvg_Paint paint);   // :1085
```

POC wrappers: `add_layer_clone(layer, below) -> id`, `set_clone_transform`,
`set_clone_opacity`, `remove_clone`, `clear_clones`.

The POC duplicates the layer's paint, bakes `picture_matrix × layer_matrix` as
its base so it renders exactly on top of the original, and parks it in the
wrapper scene.

Properties:
- **Canvas space.** `set_clone_transform` composes onto the baked base, so you
  pass pointer deltas directly — no comp-unit conversion during the drag.
- **Above everything.** This is runtime z-order lifting, which RFC 0002
  excluded from v1 because Lottie layer order is fixed. A clone added with
  `below = false` renders above the whole animation.
- **No mask caveat** — the duplicate carries its own composed transform.
- **Frozen.** The snapshot is taken at clone time and does not animate. Correct
  for a drag ghost of a settled object; wrong for dragging an object whose own
  animation must keep playing. Treat it as a mode, not the default.
- Clones must be re-created after an animation reload (the POC re-syncs lazily
  each render via a `heal_clones` pass).

### 5.3 Scene-level effects, clip and mask

`tvg_paint_set_opacity` / `tvg_paint_set_blend_method` on the wrapper, plus
`tvg_scene_add_effect_{gaussian_blur,drop_shadow,fill,tint,tritone}`
(`:2292`–`:2369`), `tvg_paint_set_clip`, and `tvg_paint_set_mask_method`
(`:1199`, methods at `:179`; `ALPHA` = 1, `INVERSE_ALPHA` = 2).

Blend method values are the `TVG_BLEND_METHOD_*` enum at `:201`
(0 = Normal, 1 = Multiply, 2 = Screen, 3 = Overlay, … 16 = Add).

The C API has **no parameter setters for effects** — changing a parameter means
clearing the whole stack and re-adding it. The POC therefore stores effects
declaratively and replays them. Effects are re-prepared on every canvas update
anyway, so replay costs nothing beyond the effect itself.

**Why DnD wants it**: depth-of-field while dragging (blur the scene, keep the
ghost sharp), spotlighting the valid drop zone with a feathered alpha mask,
tinting on invalid drop. All drivable from `onGrab`/`onDrag`/`onDrop` hooks.

---

## 6. Sequencing rules that silently break things

These are the failure modes that produce "the API does nothing" rather than an
error. All were found the hard way in the POC.

### 6.1 A clean Picture short-circuits the update traversal

`PictureImpl::skip` causes `tvg_canvas_update` to stop at a clean Picture, and
**child dirt does not bubble up**. If you mutate a layer paint without dirtying
the Picture itself, the mutation is never prepared and never drawn.

Slot writes happen to dirty the Picture. **Direct layer-paint writes do not.**
This is the trap you will hit the moment you switch DnD off slots.

The POC's fix: after the per-layer flush loop, re-set the Picture's transform
once ("poke" it) to force the subtree to re-prepare. One poke per batch, not
per layer. The same poke is required after changing the wrapper scene's clip or
mask, because the clip stack reaches children during the update traversal.

Misleading detail: **opacity and visibility work without the poke** (they are
checked at draw time). Transform, clip and blur do not. Testing only opacity
will convince you the poke is unnecessary.

### 6.2 Layer paints are invalidated by more than frame changes

The POC drops its cached base on `set_frame`, `apply_slot`, `del_slot`,
`tween_to` and `tween_go`. Any of these rebuilds the layer scenes. If you cache
anything keyed to a layer paint, invalidate it in all five places.

### 6.3 Queries join the background build

`LottieLoader::frame()` dispatches the rebuild to a worker thread when the
player is constructed with `threads > 0` (the C API's
`dotlottie_new_player(threads)` allows this; `Player::new()` uses 0).
`tvg_picture_get_paint` → `PictureImpl::iterator()` → `load()` →
`loader->sync()` → `done()` **joins** that task. So layer access is safe and
correctly ordered — but a per-frame flush that touches layers will serialize
against the build and eliminate the pipelining benefit of threads. Known,
accepted, worth measuring if you add many overridden layers.

### 6.4 Bounds and picking are one canvas update stale

`get_aabb`/`get_obb`/`intersects_region` reflect the scene as of the last
update. Picking and bounds lag identically, so they are self-consistent, but
a query issued between a write and the next render sees the old scene. This
repo already mitigates it by capturing DnD anchor offsets at first grab only
(`src/state_machine_engine/mod.rs:1192`). Keep that discipline. It is logged as
upstream ask #6 in `docs/upstream/thorvg-asks.md`.

### 6.5 Layer lookup is a full tree walk

`Picture::paint(id)` takes a fallback path — allocate an `Accessor`, walk the
entire paint tree, free it — because `accessible` is false and enabling it
(`tvg_picture_set_accessible`) does not help: `LottieLoader` has no `access()`
implementation, so the map lookup returns nothing. Cost is O(tree) per lookup,
paid per overridden layer per frame and per hit test.

Fine for 1–4 draggables. If it becomes hot, the right upstream ask is
*implement `LottieLoader::access(id)`* so `Picture::paint(id)` becomes an O(1)
map hit — cheaper for upstream than existing ask #5 and it accelerates
hit-testing, bounds and the flush at once.

---

## 7. Known bug in the current DnD, fix before or with this work

`src/state_machine_engine/mod.rs` mixes the two coordinate spaces of §2:

- `:1212` — `offset = slot_pos − pointer` (comp units − canvas px)
- `:1194` — `anchor_offset = slot_pos − layer_center` (comp units − canvas px)
- `:1008` — `dnd_clamp_to_boundary` insets a canvas-space OBB by half-extents
  and then applies the result to a comp-unit slot value

Every DnD test uses a 512×512 canvas with a 512×512 comp, so the scale factor
is 1 and the bug is invisible. Probe at 256×256 (layout scale 0.5), on
`star_drop_static.json` with `star_drop_dnd.json`:

```
slot pos (comp units)    = [138.5, 392.5]
star rendered center px  = [69.25, 193.86]
pointer dragged +40 px   -> object moved +20 px on screen
```

The object drags at half pointer speed and detaches from the cursor. Snap
targets and boundary insets are wrong by the same factor, and any non-centered
fit adds a translation error on top.

**Add a regression test at a non-1:1 canvas size.** It is the proof that the
conversion is right, whichever position-writing mechanism you keep.

---

## 8. Migration map

Split deliberately: the first table is mechanical — same model, better
mechanism. The second is a set of **spec decisions** that must go through an
RFC, not a refactor. Do not conflate them.

### 8.1 Mechanical — adopt freely

| Today | Replace with | Gain |
|---|---|---|
| `hit_test` OBB (`thorvg.rs:643`) | `hit_test_precise` / `intersects_layer` | Shape-accurate pickup and drop matching; runtime-hidden drop zones |
| z-order lifting (excluded from RFC v1) | `add_layer_clone(layer, below=false)` + `set_layer_opacity(original, …)` | Ghost renders above everything, dragged in canvas space |
| Boundary half-extents from OBB corners (`mod.rs:1198`) | `get_layer_aabb` where rotation is irrelevant | Cheaper, simpler; keep OBB for the rotated-boundary clamp |
| Comp/canvas conflation (§7) | Layout-scale conversion at the engine boundary | Correctness bug, independent of everything else here |

These change no authored artifact and no cross-player contract. Scene-level
effects, overlays and masks (§5.1, §5.3) are also in this bucket as long as
they are driven from `onGrab`/`onDrag`/`onDrop` and produce no persisted state.

### 8.2 Spec decisions — RFC material, not table rows

**(a) Position slot writes → layer transform deltas.** This is a fork in the
model, not a mechanism swap:

| | Position slot (today) | Layer transform delta |
|---|---|---|
| Channel | Serializable, defined by the dotLottie spec | Renderer-private, per-session |
| Portability | Any conforming player implements slots | Every player must implement "compose a delta onto a named layer's animated transform" as new required surface |
| Model | Absolute — the write *is* the position | Delta — composes onto whatever the animation produced |
| Fallback | Defined (authored value) | Undefined; nothing to fall back to |
| Inspectable/themeable | Yes — the gesture's output is a slot a theme can also target | No |
| Requires authoring | A `sid` on every draggable layer's position | Nothing |
| Enables | — | Dragging an object while its authored animation plays |

The real gain is the last row, and the real cost is the second: `slotId`
is not merely ergonomic overhead — it is what makes the gesture's output a
first-class, inspectable, themeable value. Removing the requirement removes
that property too. Whether the gesture's *declared output* is "a Position slot
value" or "a layer transform delta" is the RFC's headline question. A middle
path exists (declare slots as the model, use paint writes as a ThorVG fast path
when no slot is declared) but it doubles the semantics rather than choosing.

**(b) Expression-free tracking docks.** `get_layer_transform(zone)` +
`set_layer_transform(object)` per frame removes the jerryscript dependency and
the security surface, and lets `track` stop implying `lock` (the engine still
knows where the object is). **Cost**: it reverses the deliberate
gestures-emit-inputs / renderer-owns-visuals split — per-frame visual
computation moves back into the engine, which is exactly what the expression
design pushed out. Treat this as the **degradation path for expression-less
builds**, not the preferred one.

It also does *not* retire upstream ask #4. Ask #4 is about what a *conforming
player* does when a slot override carries both `k` and `x`; a runtime-side
workaround here says nothing about that spec question.

**(c) Load-time extraction.** See §8.3 — the "remove it" framing was wrong.

### 8.3 What `default_slots` actually feeds

Correcting an earlier overclaim. `extract_authored_from_animation`
(`lottie_renderer/mod.rs:563`) produces two things, and only one of them is
DnD-specific:

- `layer_paths` — used **only** by PathDrag. Safe to defer to first grab.
- `default_slots` — general slot machinery, not DnD:
  - `store_default_slots` (`:855`) seeds `slot_values`, so `slot_value()` /
    `get_slot_str()` return authored values before any write
  - `reset_slot(id)` (`:860`) restores **one** slot to its authored value
  - `reset_slots()` (`:871`) restores all

`apply_slot(0)` does **not** substitute for `reset_slot(id)` — it is
all-or-nothing. Per-slot reset genuinely requires knowing the authored value in
order to re-write it into the batch. So the map is load-bearing for
`ResetSlot`, theming, and every slot getter, regardless of what DnD does.

Deferring it also is not free. `TvgAnimation::load_data`
(`lottie_renderer/thorvg.rs:578`) hands ThorVG an owned copy and keeps it alive
because ThorVG parses in place and mutates the buffer (nulling string
terminators). That retained buffer is therefore **not** re-parseable later.
Lazy extraction requires holding a *second, pristine* copy of the animation
JSON for the session — memory proportional to the file, which for a large
animation can cost more than the parse it avoids.

Honest position: extraction can be deferred to the first slot API call, and
that is worth doing for animations that never touch slots — but it needs either
a retained pristine copy or re-reading from the original source (path /
`.lottie` package), and the choice between those is a real trade, not a
cleanup. **DnD alone does not justify it.**

### 8.4 Upstream asks affected

- **#4** (slot expression fallback) — **unchanged**. A runtime workaround does
  not answer the spec question of cross-player degradation.
- **#5** (evaluated path readback) — partly answered:
  `tvg_paint_intersects_region` gives shape-accurate *containment* today. Still
  needed for path *sampling* in PathDrag.
- **New candidate**: `LottieLoader::access(id)` (§6.5).

---

## 9. Gesture recipes

Pseudo-code, canvas-space pointer coords, `S` = layout scale (canvas px per
comp unit).

> These show the paint-write model end to end. The grab, hover and ghost steps
> are mechanical (§8.1) and stand on their own. The **commit** step assumes the
> layer-transform model, which is an open spec decision (§8.2a) — with slots
> retained, only that one line changes: write the target position to the
> Position slot instead of composing a delta.

**Grab**
```
if hit_test_precise(layer, px, py, visible_only = false):
    aabb   = get_layer_aabb(layer)              # canvas px
    ghost  = add_layer_clone(layer, below = false)
    set_clone_opacity(ghost, 150)
    set_layer_opacity(layer, 70)                # original dims in place
    grab   = (px, py); center = aabb center
    run onGrab actions
```

**Drag** (clone moves in canvas space — no conversion)
```
d = (px - grab.x, py - grab.y)
set_clone_transform(ghost, translate(d))
for zone in zones:                              # object-overlap, not pointer
    hovered = intersects_layer(zone, center.x + d.x - w/2, center.y + d.y - h/2,
                               w, h, visible_only = false)
    restyle the zone's overlay ring on change
run onDrag actions
```

**Drop**
```
zone = first zone where intersects_layer(...)   # or pointer test, per spec
target = zone ? aabb center of zone : rest      # canvas px
commit  = (target - center) / S                 # -> comp units
set_layer_transform(layer, translate(commit))   # composed onto animated value
remove_clone(ghost); set_layer_opacity(layer, 255)
run onDrop actions, then the zone's actions
```

**Tracking dock, no expressions**
```
each frame while docked:
    zpos = get_layer_transform(zone)            # comp units, animated value
    set_layer_transform(object, translate(zpos - object_authored_pos))
```

---

## 10. Constraints and non-goals

- Per-layer transform overrides are correct only on **unmasked** layers (§4).
  Use a clone if a draggable has a mask or matte.
- Clones are **frozen snapshots** — they do not animate (§5.2).
- A per-layer blur **suppresses the layer's authored effects** while active
  (§4).
- `tvg_paint_intersects_region` ignores blending and masking, and is one
  canvas update stale (§6.4).
- Layers authored at opacity 0 are pruned by the builder and have no paint at
  all — they cannot be hit-tested, hidden, or transformed (§3.1).
- `tvg_paint_intersects_region` is marked **Experimental API** upstream; the
  signature may change.

---

## 11. Verifying against the reference

```bash
# The POC's pixel-diff coverage of every API above
cd <dotlottie-rs paint-fx-poc checkout>/dotlottie-rs
cargo test --test paint_fx

# Runnable demos, including the clone-ghost DnD recipe of §9
open examples/web/index.html          # examples/web/drag-and-drop.html
```

Reference implementation files in the POC:
- `src/renderer/backend.rs` — `LayerProps`, `ClipRegion`, `OverlayProps`,
  `CloneProps`, `SpotMask` and the `Animation` trait additions
- `src/renderer/thorvg.rs` — `apply_layer_prop`, `sync_clone`, `sync_overlay`,
  the wrapper-scene construction, the C API calls
- `src/renderer/mod.rs` — override storage, the per-render flush, the Picture
  poke, reload replay
- `docs/paint-fx-poc-report.md` — the validated ThorVG mechanics in full
