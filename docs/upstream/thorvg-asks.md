# ThorVG upstream asks

Running list of ThorVG capabilities that would simplify or unlock parts of
the state-machine interactivity work (state slots, DragAndDrop, PathDrag,
precomp scrubbing). Each entry: what we need, why, what we do today instead,
and what lands if upstream ships it. Ordered by impact.

*Origin: prototyping sessions 2026-06/07 (RFCs 0001/0002, PathDrag, tm-scrub
bridge). ThorVG references are against the vendored copy in
`dotlottie-rs/deps/thorvg`.*

---

## 1. Native slot support on time remap (`tm`)

**What**: allow `"sid"` on a precomp layer's `tm` property, so a scalar slot
can drive a precomp's internal clock directly.

**Why we need it**: "gesture scrubs a nested timeline" is our core pattern —
sliders, spirals, PathDrag all publish one progress number that replays
authored keyframes inside a precomp. It moves all visible motion back into
designer-owned keyframes instead of engine-computed positions.

**Today's workaround**: a `sid` on `tm` crashes ThorVG (null-object slot
pair), so every animation needs a 3-piece bridge: hidden `time_driver` layer
with a slotted rotation, a top-level `slots` entry, and a `tm` expression
(`var $bm_rt = thisComp.layer('time_driver').transform.rotation * duration;`).
See `docs/explainers/path-drag-pipeline.md`, authoring checklist item 4.

**If it lands**: the bridge collapses to a single `sid` on `tm`; the hidden
layer, the expression, and the hard dependency on the expressions build flag
(jerryscript) all disappear for scrub-style interactions.

## 2. `pointOnPath` / `tangentOnPath` reachable from `content()` chains

**What**: expression path objects obtained via `content(...)` chains should
support `pointOnPath`/`tangentOnPath`. The builtins exist in
`tvgLottieExpressions.cpp` but paths coming out of `_content` lack the
`_buildPath` attachment, so they are unreachable (fix: attach it in the
`_content` Path branch).

**Why we need it**: it is the clean way to make an object follow an authored
path from a single progress value — `position = path.pointOnPath(t)` —
without generating arc-length keyframes or baking a full bezier evaluator
into an expression string.

**Today's workaround**: either 25 generated arc-length-proportional
keyframes per path (spiral_scrub) or a baked-JS bezier expression
(spiral_expr). Both work; both are generated artifacts a designer can't
hand-edit.

**If it lands**: path-following becomes one short authored expression;
PathDrag's animation-side authoring checklist shrinks to "name the path
layer, add the expression".

## 3. Path/shape slot type

**What**: support a path (`PathSet`) property type in the slot parser.
`LottieParser::parse(LottieSlot*)` currently handles Float, Scalar, Vector,
Opacity, Color, ColorStop, TextDoc, Image — a path-typed `sid` hits
`default: break` and is silently ignored.

**Why we need it**: PathDrag's constraint curve should be a declared,
replaceable value rather than "the first static shape found in a named
layer". A path slot would give one source of truth: overriding the slot
retargets both the *drawn* track and the drag constraint, and enables
runtime-mutable constraint paths (engine rebuilds its sample table at next
grab).

**Today's workaround**: `pathLayerName` + load-time JSON extraction of the
layer's first static `sh` shape (`layer_paths_from_value`), with the drawn
track and the extracted path being the same layer by construction.

**If it lands**: `pathSlotId` replaces `pathLayerName`; extraction machinery
is deleted (the slots block is already parsed for defaults); dynamic paths
come free.

## 4. Slot override fallback: copy the static value alongside an expression

**What**: when a slot override carries both a static `k` and an expression
`x`, `LottieProperty::copy` transfers only the expression; the static value
is not copied, so on players/builds without expression support the property
falls back to the *authored* value, not the written one.

**Why we need it**: DragAndDrop tracking docks write
`{k: releasePoint, x: "…layer('zone').transform.position;"}` intending `k`
as the declared fallback for expression-less players. Today that fallback
silently lands on the authored rest position instead — the docked object
appears to never have docked.

**Today's workaround**: none; documented as a spec finding (RFC 0002).
*Status update*: tracking docks moved to engine-driven follow (per-tick
bounds read + slot write), so this is **off the DnD critical path** — it
still matters for any expression-carrying slot override.

**If it lands**: degradation behavior for expression-less players becomes
definable in the spec instead of "whatever copy() does".

## 5. Evaluated path readback in canvas space

**What**: a way to read a layer's *current* (frame-evaluated, transform-
composed) path geometry at runtime. `tvg_shape_get_path` exists in the capi,
but reaching a layer's child Shape needs Accessor-callback traversal, and
the points come back in shape-local space with no composed-transform query
to lift them into canvas space.

**Why we need it**: it removes PathDrag's static-only constraint — a
morphing or keyframe-animated constraint path could be re-sampled at grab
time, the same way DragAndDrop snap targets moved from load-time JSON to
drop-time `layer_bounds` reads (which worked because `tvg_paint_get_obb`
composes transforms internally; the path API has no equivalent).

**Today's workaround**: load-time JSON extraction, static paths only.

**If it lands**: PathDrag drops its last load-time dependency and its
static-path limitation together.

## 6. `LottieLoader::access(id)` — O(1) layer lookup

**What**: `Picture::paint(id)` currently falls back to allocating an
`Accessor` and walking the entire paint tree, because `LottieLoader` does
not implement `access()` (the O(1) id → paint map used by other loaders).

**Why we need it**: every runtime geometry query — hit tests, bounds for
snap targets and boundary clamps, per-tick tracking reads — pays O(tree)
per lookup. Fine at 1–4 draggables; linear growth with scene size and
override count. Caching paint handles engine-side is not an alternative:
`updateLayer` regenerates each layer's scene every frame, so held
pointers go stale immediately. (Credit: surfaced by the paint-APIs
exploration doc, `docs/explainers/thorvg-paint-apis-for-dnd.md` §6.5.)

**Today's workaround**: accept the walk; keep per-tick queries few.

**If it lands**: hit-testing, bounds, and tracking all become map hits —
likely the cheapest-to-implement, widest-benefit ask on this list.

## 7. Typed / per-slot slot application — skip the JSON round-trip

**What**: applying slot overrides means serializing them into a JSON
string (`gen_slot`) that the loader re-parses (`apply_slot`), and
overrides apply as one batch — there is no API to set a single slot's
value directly, nor to update one slot without regenerating the whole
batch.

**Why we need it**: DragAndDrop writes a Position slot on every held
pointer move, and a `track` dock writes every tick while its zone moves.
Each write dirties the batch, so every dragged frame re-serializes and
re-parses *all* active overrides — theme colors, text, gradients — to
move one position. Cost scales with total override count, not with what
changed.

**Today's workaround**: writes coalesce per frame behind a dirty flag,
and the gesture slot can be split into its own `gen_slot` code so the
per-frame JSON shrinks to a few bytes. The serialize/parse round-trip
itself cannot be avoided from outside.

**If it lands**: drag frames become parse-free (`set_slot(id, value)` or
incremental per-slot codes) — the difference shows on heavily themed
animations, where today a drag pays for every slot in the file.

## 8. Bounds/picking freshness (minor)

**What**: `tvg_paint_get_obb` / hit-testing reflect the scene as of the
*previous* canvas update — one frame stale relative to the latest tick.

**Why it matters**: forces "settle ticks" in tests and one-frame-stale grab
offsets; we mitigated by capturing DnD anchor offsets at first grab only.
Self-consistent (picking and bounds lag identically), so this is a paper
cut, not a blocker — but a "query against latest scene" option or an
explicit sync primitive would remove a class of subtle timing bugs.

---

Items 1–2 are pure additions with existing machinery nearby (slots already
work on transform properties; the path builtins already exist). Item 4 is
arguably a bug fix. Items 6 and 7 are self-contained perf wins. Items 3
and 5 are new surface area and worth raising as design questions rather
than patches.
