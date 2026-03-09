use std::ffi::CString;

use js_sys::{Array, Float32Array, Object, Uint8Array};
use wasm_bindgen::prelude::*;

use crate::{ColorSpace, DotLottiePlayer, Fit, Layout, Mode as PlayerMode};

// ─── Renderer mode ───────────────────────────────────────────────────────────

const RENDERER_SW: u8 = 0;
#[cfg(feature = "webgl")]
const RENDERER_GL: u8 = 1;
#[cfg(feature = "webgpu")]
const RENDERER_WG: u8 = 2;

// ─── StoredGlContext — wraps the pointer in webgl_stubs::CONTEXT_PTR ────────
//
// ThorVG's GL renderer checks `context != nullptr` before proceeding, and
// later compares `mContext` against `emscripten_webgl_get_current_context()`.
// Both must match the pointer stored by `webgl_stubs::set_webgl_context`, so
// we cannot pass null_mut here.

#[cfg(feature = "webgl")]
struct StoredGlContext;

#[cfg(feature = "webgl")]
impl crate::GlContext for StoredGlContext {
    fn as_ptr(&self) -> *mut std::ffi::c_void { crate::webgl_stubs::context_ptr() }
    unsafe fn from_ptr(_ptr: *mut std::ffi::c_void) -> Self { StoredGlContext }
}

// ─── WgpuPtr helpers ────────────────────────────────────────────────────────

#[cfg(feature = "webgpu")]
struct WgpuDevicePtr(usize);
#[cfg(feature = "webgpu")]
impl crate::WgpuDevice for WgpuDevicePtr {
    fn as_ptr(&self) -> *mut std::ffi::c_void { self.0 as *mut std::ffi::c_void }
    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self { WgpuDevicePtr(ptr as usize) }
}

// In browser WebGPU there is no JS GPUInstance object.  ThorVG only stores
// the instance pointer for equality comparison (to detect device changes), so
// any stable non-null sentinel works.
#[cfg(feature = "webgpu")]
static WGPU_INSTANCE_SENTINEL: u8 = 0;

#[cfg(feature = "webgpu")]
struct WgpuSentinelInstance;
#[cfg(feature = "webgpu")]
impl crate::WgpuInstance for WgpuSentinelInstance {
    fn as_ptr(&self) -> *mut std::ffi::c_void {
        &raw const WGPU_INSTANCE_SENTINEL as *mut std::ffi::c_void
    }
    unsafe fn from_ptr(_ptr: *mut std::ffi::c_void) -> Self { WgpuSentinelInstance }
}

#[cfg(feature = "webgpu")]
struct WgpuSurfacePtr(usize);
#[cfg(feature = "webgpu")]
impl crate::WgpuTarget for WgpuSurfacePtr {
    fn as_ptr(&self) -> *mut std::ffi::c_void { self.0 as *mut std::ffi::c_void }
    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self { WgpuSurfacePtr(ptr as usize) }
}

// ─── Exported enums ───────────────────────────────────────────────────────────

/// Playback direction / bounce mode.
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Forward      = 0,
    Reverse      = 1,
    Bounce       = 2,
    ReverseBounce = 3,
}

impl From<Mode> for PlayerMode {
    fn from(m: Mode) -> Self {
        match m {
            Mode::Forward       => PlayerMode::Forward,
            Mode::Reverse       => PlayerMode::Reverse,
            Mode::Bounce        => PlayerMode::Bounce,
            Mode::ReverseBounce => PlayerMode::ReverseBounce,
        }
    }
}

impl From<PlayerMode> for Mode {
    fn from(m: PlayerMode) -> Self {
        match m {
            PlayerMode::Forward       => Mode::Forward,
            PlayerMode::Reverse       => Mode::Reverse,
            PlayerMode::Bounce        => Mode::Bounce,
            PlayerMode::ReverseBounce => Mode::ReverseBounce,
        }
    }
}

// ─── JS object helpers ────────────────────────────────────────────────────────

fn js_obj_with_type(type_name: &str) -> Object {
    let obj = Object::new();
    let _ = js_sys::Reflect::set(&obj, &"type".into(), &type_name.into());
    obj
}

fn set_str(obj: &Object, key: &str, v: &str) {
    let _ = js_sys::Reflect::set(obj, &key.into(), &v.into());
}

fn set_f64(obj: &Object, key: &str, v: f64) {
    let _ = js_sys::Reflect::set(obj, &key.into(), &JsValue::from_f64(v));
}

fn set_bool(obj: &Object, key: &str, v: bool) {
    let _ = js_sys::Reflect::set(obj, &key.into(), &JsValue::from_bool(v));
}

fn vec_to_f32array(v: Vec<f32>) -> Float32Array {
    let arr = Float32Array::new_with_length(v.len() as u32);
    for (i, &x) in v.iter().enumerate() {
        arr.set_index(i as u32, x);
    }
    arr
}

fn fit_from_str(s: &str) -> Fit {
    match s {
        "contain"    => Fit::Contain,
        "fill"       => Fit::Fill,
        "cover"      => Fit::Cover,
        "fit-width"  => Fit::FitWidth,
        "fit-height" => Fit::FitHeight,
        _            => Fit::None,
    }
}

fn fit_to_str(f: Fit) -> &'static str {
    match f {
        Fit::Contain   => "contain",
        Fit::Fill      => "fill",
        Fit::Cover     => "cover",
        Fit::FitWidth  => "fit-width",
        Fit::FitHeight => "fit-height",
        Fit::None      => "none",
    }
}

// ─── Main wrapper struct ──────────────────────────────────────────────────────
//
// Field order matters for Drop: `state_machine` MUST come before `player` so
// the engine (which holds a raw pointer into player) is dropped first.

#[wasm_bindgen]
pub struct DotLottiePlayerWasm {
    /// Active state machine engine.  Declared before `player` to ensure it is
    /// dropped first (it holds a raw mutable pointer into `player`).
    #[cfg(feature = "state-machines")]
    state_machine: Option<crate::StateMachineEngine<'static>>,
    player: DotLottiePlayer,
    /// Owned pixel buffer for the SW renderer (ARGB8888 u32 values).
    sw_buffer: Vec<u32>,
    width: u32,
    height: u32,
    renderer_mode: u8,
    #[cfg(feature = "webgpu")]
    wg_device_ptr: usize,
    #[cfg(feature = "webgpu")]
    wg_surface_ptr: usize,
}

#[wasm_bindgen]
impl DotLottiePlayerWasm {
    // ── Constructor ───────────────────────────────────────────────────────────

    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        DotLottiePlayerWasm {
            #[cfg(feature = "state-machines")]
            state_machine: None,
            player: DotLottiePlayer::new(),
            sw_buffer: Vec::new(),
            width: 0,
            height: 0,
            renderer_mode: RENDERER_SW,
            #[cfg(feature = "webgpu")]
            wg_device_ptr: 0,
            #[cfg(feature = "webgpu")]
            wg_surface_ptr: 0,
        }
    }

    // ── Renderer context setup ────────────────────────────────────────────────

    /// Store the WebGL2 context.  Call before `load_animation`.
    #[cfg(feature = "webgl")]
    pub fn set_webgl_context(&mut self, ctx: web_sys::WebGl2RenderingContext) {
        crate::webgl_stubs::set_webgl_context(ctx);
        self.renderer_mode = RENDERER_GL;
    }

    /// Store the WebGPU device.  Call before `set_webgpu_surface` and `load_animation`.
    #[cfg(feature = "webgpu")]
    pub fn set_webgpu_device(&mut self, device: web_sys::GpuDevice) {
        if self.wg_device_ptr != 0 {
            unsafe { drop(Box::from_raw(self.wg_device_ptr as *mut web_sys::GpuDevice)); }
        }
        self.wg_device_ptr = Box::into_raw(Box::new(device)) as usize;
        self.renderer_mode = RENDERER_WG;
    }

    /// Store the WebGPU canvas context (surface).  Call before `load_animation`.
    #[cfg(feature = "webgpu")]
    pub fn set_webgpu_surface(&mut self, surface: web_sys::GpuCanvasContext) {
        if self.wg_surface_ptr != 0 {
            unsafe { drop(Box::from_raw(self.wg_surface_ptr as *mut web_sys::GpuCanvasContext)); }
        }
        self.wg_surface_ptr = Box::into_raw(Box::new(surface)) as usize;
    }

    // ── Internal render-target setup ──────────────────────────────────────────

    fn setup_target(&mut self, width: u32, height: u32) -> bool {
        self.width  = width;
        self.height = height;
        match self.renderer_mode {
            #[cfg(feature = "webgl")]
            RENDERER_GL => self.player.set_gl_target(&StoredGlContext, 0, width, height).is_ok(),

            #[cfg(feature = "webgpu")]
            RENDERER_WG => {
                if self.wg_device_ptr == 0 || self.wg_surface_ptr == 0 {
                    return false;
                }
                self.player
                    .set_wg_target(
                        &WgpuDevicePtr(self.wg_device_ptr),
                        &WgpuSentinelInstance,
                        &WgpuSurfacePtr(self.wg_surface_ptr),
                        width,
                        height,
                        crate::WgpuTargetType::Surface,
                    )
                    .is_ok()
            }

            _ => {
                let required = (width * height) as usize;
                if self.sw_buffer.len() != required {
                    self.sw_buffer.resize(required, 0);
                }
                self.player
                    .set_sw_target(&mut self.sw_buffer, width, height, ColorSpace::ABGR8888)
                    .is_ok()
            }
        }
    }

    // ── Loading ───────────────────────────────────────────────────────────────

    /// Load a Lottie JSON animation.  Sets up the rendering target automatically.
    pub fn load_animation(&mut self, data: &str, width: u32, height: u32) -> bool {
        if !self.setup_target(width, height) { return false; }
        let Ok(c_data) = CString::new(data) else { return false; };
        self.player.load_animation_data(&c_data, width, height).is_ok()
    }

    /// Load a .lottie archive from raw bytes.
    #[cfg(feature = "dotlottie")]
    pub fn load_dotlottie_data(&mut self, data: &[u8], width: u32, height: u32) -> bool {
        if !self.setup_target(width, height) { return false; }
        self.player.load_dotlottie_data(data, width, height).is_ok()
    }

    /// Load an animation from an already-loaded .lottie archive by its ID.
    #[cfg(feature = "dotlottie")]
    pub fn load_animation_from_id(&mut self, id: &str, width: u32, height: u32) -> bool {
        if !self.setup_target(width, height) { return false; }
        let Ok(c_id) = CString::new(id) else { return false; };
        self.player.load_animation(&c_id, width, height).is_ok()
    }

    // ── Render loop ───────────────────────────────────────────────────────────

    /// Advance time and render.  Call once per `requestAnimationFrame`.
    pub fn tick(&mut self) -> bool { self.player.tick().is_ok() }

    /// Render the current frame without advancing time.
    pub fn render(&mut self) -> bool { self.player.render().is_ok() }

    /// Clear the canvas to the background colour.
    pub fn clear(&mut self) { self.player.clear(); }

    /// Resize the canvas.  For the SW renderer this also resizes the pixel buffer.
    pub fn resize(&mut self, width: u32, height: u32) -> bool {
        if self.renderer_mode == RENDERER_SW {
            let required = (width * height) as usize;
            self.sw_buffer.resize(required, 0);
            if self.player
                .set_sw_target(&mut self.sw_buffer, width, height, ColorSpace::ABGR8888)
                .is_err()
            {
                return false;
            }
        }
        self.width  = width;
        self.height = height;
        self.player.resize(width, height).is_ok()
    }

    // ── SW pixel buffer ───────────────────────────────────────────────────────

    /// Zero-copy `Uint8Array` view into WASM linear memory.
    ///
    /// **Use the returned array immediately.**  Do not store the reference across
    /// any call that may reallocate the buffer (e.g. `resize` / `load_animation`
    /// with different dimensions).
    pub fn get_pixel_buffer(&self) -> Uint8Array {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                self.sw_buffer.as_ptr() as *const u8,
                self.sw_buffer.len() * 4,
            )
        };
        unsafe { Uint8Array::view(bytes) }
    }

    // ── Playback control ──────────────────────────────────────────────────────

    pub fn play(&mut self)  -> bool { self.player.play().is_ok() }
    pub fn pause(&mut self) -> bool { self.player.pause().is_ok() }
    pub fn stop(&mut self)  -> bool { self.player.stop().is_ok() }

    // ── State queries ─────────────────────────────────────────────────────────

    pub fn is_playing(&self) -> bool { self.player.is_playing() }
    pub fn is_paused(&self)  -> bool { self.player.is_paused() }
    pub fn is_stopped(&self) -> bool { self.player.is_stopped() }
    pub fn is_loaded(&self)  -> bool { self.player.is_loaded() }
    pub fn is_complete(&self) -> bool { self.player.is_complete() }
    pub fn is_tweening(&self) -> bool { self.player.is_tweening() }

    // ── Frame queries ─────────────────────────────────────────────────────────

    pub fn current_frame(&self)  -> f32 { self.player.current_frame() }
    pub fn total_frames(&self)   -> f32 { self.player.total_frames() }
    pub fn request_frame(&mut self) -> f32 { self.player.request_frame() }

    pub fn set_frame(&mut self, no: f32) -> bool { self.player.set_frame(no).is_ok() }
    pub fn seek(&mut self, no: f32) -> bool       { self.player.seek(no).is_ok() }

    // ── Duration / loop queries ───────────────────────────────────────────────

    pub fn duration(&self)             -> f32 { self.player.duration() }
    pub fn segment_duration(&self)     -> f32 { self.player.segment_duration() }
    pub fn current_loop_count(&self)   -> u32 { self.player.current_loop_count() }
    pub fn reset_current_loop_count(&mut self) { self.player.reset_current_loop_count(); }

    // ── Size ──────────────────────────────────────────────────────────────────

    pub fn width(&self)  -> u32 { self.player.size().0 }
    pub fn height(&self) -> u32 { self.player.size().1 }

    /// `[width, height]` of the animation in its native coordinate space.
    pub fn animation_size(&self) -> Float32Array {
        vec_to_f32array(self.player.animation_size())
    }

    // ── Playback settings ─────────────────────────────────────────────────────

    pub fn mode(&self) -> Mode { self.player.mode().into() }
    pub fn set_mode(&mut self, mode: Mode) { self.player.set_mode(mode.into()); }

    pub fn speed(&self) -> f32 { self.player.speed() }
    pub fn set_speed(&mut self, speed: f32) { self.player.set_speed(speed); }

    pub fn loop_animation(&self) -> bool { self.player.loop_animation() }
    pub fn set_loop(&mut self, v: bool)  { self.player.set_loop(v); }

    pub fn loop_count(&self) -> u32      { self.player.loop_count() }
    pub fn set_loop_count(&mut self, n: u32) { self.player.set_loop_count(n); }

    pub fn autoplay(&self) -> bool       { self.player.autoplay() }
    pub fn set_autoplay(&mut self, v: bool) { self.player.set_autoplay(v); }

    pub fn use_frame_interpolation(&self) -> bool { self.player.use_frame_interpolation() }
    pub fn set_use_frame_interpolation(&mut self, v: bool) {
        self.player.set_use_frame_interpolation(v);
    }

    pub fn background_color(&self) -> u32 { self.player.background_color() }

    /// Set background colour (`0xAARRGGBB`).
    pub fn set_background_color(&mut self, color: u32) -> bool {
        self.player.set_background_color(Some(color)).is_ok()
    }

    /// Clear the background colour (transparent).
    pub fn clear_background_color(&mut self) -> bool {
        self.player.set_background_color(None).is_ok()
    }

    pub fn set_quality(&mut self, quality: u8) -> bool {
        self.player.set_quality(quality).is_ok()
    }

    // ── Segment ───────────────────────────────────────────────────────────────

    pub fn has_segment(&self)    -> bool { self.player.segment().is_some() }
    pub fn segment_start(&self)  -> f32  { self.player.segment().map(|s| s[0]).unwrap_or(0.0) }
    pub fn segment_end(&self)    -> f32  { self.player.segment().map(|s| s[1]).unwrap_or(0.0) }

    pub fn set_segment(&mut self, start: f32, end: f32) -> bool {
        self.player.set_segment(Some([start, end])).is_ok()
    }

    pub fn clear_segment(&mut self) -> bool {
        self.player.set_segment(None).is_ok()
    }

    // ── Layout ────────────────────────────────────────────────────────────────

    /// Set the layout.
    ///
    /// `fit` is one of `"contain"`, `"fill"`, `"cover"`, `"fit-width"`,
    /// `"fit-height"`, `"none"`.  `align_x` / `align_y` are in [0, 1].
    pub fn set_layout(&mut self, fit: &str, align_x: f32, align_y: f32) -> bool {
        self.player
            .set_layout(Layout { fit: fit_from_str(fit), align: [align_x, align_y] })
            .is_ok()
    }

    pub fn layout_fit(&self)     -> String { fit_to_str(self.player.layout().fit).to_string() }
    pub fn layout_align_x(&self) -> f32    { self.player.layout().align[0] }
    pub fn layout_align_y(&self) -> f32    { self.player.layout().align[1] }

    // ── Viewport ──────────────────────────────────────────────────────────────

    pub fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> bool {
        self.player.set_viewport(x, y, w, h).is_ok()
    }

    // ── Slots ─────────────────────────────────────────────────────────────────

    /// Set a color slot (`r`, `g`, `b` in [0, 1]).
    pub fn set_color_slot(&mut self, id: &str, r: f32, g: f32, b: f32) -> bool {
        self.player.set_color_slot(id, crate::ColorSlot::new([r, g, b])).is_ok()
    }

    pub fn set_scalar_slot(&mut self, id: &str, value: f32) -> bool {
        self.player.set_scalar_slot(id, crate::ScalarSlot::new(value)).is_ok()
    }

    pub fn set_text_slot(&mut self, id: &str, text: &str) -> bool {
        self.player.set_text_slot(id, crate::TextSlot::new(text.to_string())).is_ok()
    }

    pub fn set_vector_slot(&mut self, id: &str, x: f32, y: f32) -> bool {
        self.player.set_vector_slot(id, crate::VectorSlot::static_value([x, y])).is_ok()
    }

    pub fn set_position_slot(&mut self, id: &str, x: f32, y: f32) -> bool {
        self.player.set_position_slot(id, crate::PositionSlot::static_value([x, y])).is_ok()
    }

    pub fn clear_slots(&mut self)           -> bool { self.player.clear_slots().is_ok() }
    pub fn clear_slot(&mut self, id: &str)  -> bool { self.player.clear_slot(id).is_ok() }

    /// Set multiple slots at once from a JSON string.
    pub fn set_slots_str(&mut self, json: &str) -> bool {
        self.player.set_slots_str(json).is_ok()
    }

    /// Set a single slot by ID from a JSON value string.
    pub fn set_slot_str(&mut self, id: &str, json: &str) -> bool {
        self.player.set_slot_str(id, json).is_ok()
    }

    /// Get the JSON value of a single slot by ID, or `undefined` if not found.
    pub fn get_slot_str(&self, id: &str) -> Option<String> {
        let s = self.player.get_slot_str(id);
        if s.is_empty() { None } else { Some(s) }
    }

    /// Get all slots as a JSON object string.
    pub fn get_slots_str(&self) -> String {
        self.player.get_slots_str()
    }

    /// Get all slot IDs as a JS array.
    pub fn get_slot_ids(&self) -> JsValue {
        let arr = Array::new();
        for id in self.player.get_slot_ids() {
            arr.push(&id.as_str().into());
        }
        arr.into()
    }

    /// Get the type string of a slot, or `undefined` if not found.
    pub fn get_slot_type(&self, id: &str) -> Option<String> {
        let s = self.player.get_slot_type(id);
        if s.is_empty() { None } else { Some(s) }
    }

    /// Reset a slot to its default value from the animation.
    pub fn reset_slot(&mut self, id: &str) -> bool {
        self.player.reset_slot(id).is_ok()
    }

    /// Reset all slots to their default values from the animation.
    pub fn reset_slots(&mut self) -> bool {
        self.player.reset_slots()
    }

    // ── Layer inspection ──────────────────────────────────────────────────────

    pub fn intersect(&self, x: f32, y: f32, layer_name: &str) -> bool {
        self.player.intersect(x, y, layer_name)
    }

    /// Returns `[x, y, width, height]` of the layer's bounding box.
    pub fn get_layer_bounds(&self, layer_name: &str) -> Float32Array {
        vec_to_f32array(self.player.get_layer_bounds(layer_name))
    }

    // ── Transform ─────────────────────────────────────────────────────────────

    /// Returns the current affine transform as a flat `Float32Array`.
    pub fn get_transform(&self) -> Float32Array {
        vec_to_f32array(self.player.get_transform())
    }

    pub fn set_transform(&mut self, data: &[f32]) -> bool {
        self.player.set_transform(data.to_vec()).is_ok()
    }

    // ── Tween ─────────────────────────────────────────────────────────────────

    /// Tween to `to` frame.  `duration` in seconds; pass `undefined` for default.
    pub fn tween(&mut self, to: f32, duration: Option<f32>) -> bool {
        self.player.tween(to, duration, None).is_ok()
    }

    /// Tween with a cubic-bezier easing (`e0..e3`).
    pub fn tween_with_easing(
        &mut self, to: f32, duration: Option<f32>,
        e0: f32, e1: f32, e2: f32, e3: f32,
    ) -> bool {
        self.player.tween(to, duration, Some([e0, e1, e2, e3])).is_ok()
    }

    pub fn tween_stop(&mut self) -> bool { self.player.tween_stop().is_ok() }

    pub fn tween_update(&mut self, progress: Option<f32>) -> bool {
        self.player.tween_update(progress).is_ok()
    }

    pub fn tween_to_marker(&mut self, marker: &str, duration: Option<f32>) -> bool {
        let Ok(c) = CString::new(marker) else { return false; };
        self.player.tween_to_marker(&c, duration, None).is_ok()
    }

    // ── Markers ───────────────────────────────────────────────────────────────

    /// Returns an array of `{ name, time, duration }` objects.
    pub fn markers(&self) -> JsValue {
        let arr = Array::new();
        for m in self.player.markers() {
            let obj = Object::new();
            let _ = js_sys::Reflect::set(&obj, &"name".into(),     &m.name.into());
            let _ = js_sys::Reflect::set(&obj, &"time".into(),     &JsValue::from_f64(m.time as f64));
            let _ = js_sys::Reflect::set(&obj, &"duration".into(), &JsValue::from_f64(m.duration as f64));
            arr.push(&obj);
        }
        arr.into()
    }

    /// Returns an array of marker name strings.
    pub fn marker_names(&self) -> JsValue {
        let arr = Array::new();
        for name in self.player.marker_names() {
            arr.push(&name.to_string_lossy().as_ref().into());
        }
        arr.into()
    }

    /// Name of the currently active marker, or `undefined` if none.
    pub fn current_marker(&self) -> Option<String> {
        self.player.marker().map(|c| c.to_string_lossy().into_owned())
    }

    pub fn set_marker(&mut self, name: &str) {
        let Ok(c) = CString::new(name) else { return; };
        self.player.set_marker(Some(&c));
    }

    pub fn clear_marker(&mut self) {
        self.player.set_marker(None);
    }

    // ── Events ────────────────────────────────────────────────────────────────

    /// Poll the next player event.  Returns `null` if the queue is empty,
    /// otherwise a plain JS object with a `type` string field and optional
    /// payload fields (`frameNo`, `loopCount`).
    pub fn poll_event(&mut self) -> JsValue {
        let Some(evt) = self.player.poll_event() else { return JsValue::null() };
        match evt {
            crate::DotLottieEvent::Load      => js_obj_with_type("Load").into(),
            crate::DotLottieEvent::LoadError => js_obj_with_type("LoadError").into(),
            crate::DotLottieEvent::Play      => js_obj_with_type("Play").into(),
            crate::DotLottieEvent::Pause     => js_obj_with_type("Pause").into(),
            crate::DotLottieEvent::Stop      => js_obj_with_type("Stop").into(),
            crate::DotLottieEvent::Complete  => js_obj_with_type("Complete").into(),
            crate::DotLottieEvent::Frame  { frame_no } => {
                let obj = js_obj_with_type("Frame");
                set_f64(&obj, "frameNo", frame_no as f64);
                obj.into()
            }
            crate::DotLottieEvent::Render { frame_no } => {
                let obj = js_obj_with_type("Render");
                set_f64(&obj, "frameNo", frame_no as f64);
                obj.into()
            }
            crate::DotLottieEvent::Loop { loop_count } => {
                let obj = js_obj_with_type("Loop");
                set_f64(&obj, "loopCount", loop_count as f64);
                obj.into()
            }
        }
    }

    pub fn emit_on_loop(&mut self) { self.player.emit_on_loop(); }

    // ── Font ──────────────────────────────────────────────────────────────────

    #[cfg(feature = "tvg")]
    pub fn load_font(&mut self, name: &str, data: &[u8]) -> bool {
        DotLottiePlayer::load_font(name, data).is_ok()
    }

    #[cfg(feature = "tvg")]
    pub fn unload_font(name: &str) -> bool {
        DotLottiePlayer::unload_font(name).is_ok()
    }

    // ── Theming ───────────────────────────────────────────────────────────────

    #[cfg(feature = "theming")]
    pub fn set_theme(&mut self, id: &str) -> bool {
        let Ok(c) = CString::new(id) else { return false; };
        self.player.set_theme(&c).is_ok()
    }

    #[cfg(feature = "theming")]
    pub fn reset_theme(&mut self) -> bool { self.player.reset_theme().is_ok() }

    #[cfg(feature = "theming")]
    pub fn set_theme_data(&mut self, data: &str) -> bool {
        let Ok(c) = CString::new(data) else { return false; };
        self.player.set_theme_data(&c).is_ok()
    }

    #[cfg(feature = "theming")]
    pub fn theme_id(&self) -> Option<String> {
        self.player.theme_id().map(|c| c.to_string_lossy().into_owned())
    }

    // ── DotLottie manifest / animation info ───────────────────────────────────

    #[cfg(feature = "dotlottie")]
    pub fn animation_id(&self) -> Option<String> {
        self.player.animation_id().map(|c| c.to_string_lossy().into_owned())
    }

    /// Returns the animation manifest as a JSON string, or empty string if unavailable.
    #[cfg(feature = "dotlottie")]
    pub fn manifest_string(&self) -> String {
        match self.player.manifest() {
            Some(m) => serde_json::to_string(m).unwrap_or_default(),
            None => String::new(),
        }
    }

    // ── State machines ────────────────────────────────────────────────────────

    /// Returns the raw JSON definition of a state machine by ID, or `undefined`.
    #[cfg(feature = "state-machines")]
    pub fn get_state_machine(&self, id: &str) -> Option<String> {
        let Ok(c) = CString::new(id) else { return None; };
        self.player.get_state_machine(&c)
    }

    /// Returns the ID of the currently active state machine, or `undefined`.
    #[cfg(feature = "state-machines")]
    pub fn state_machine_id(&self) -> Option<String> {
        self.player.state_machine_id().map(|c| c.to_string_lossy().into_owned())
    }

    /// Load a state machine from a JSON definition string.  Returns `true` on
    /// success.  The engine is kept alive inside the player and interacted
    /// with via the `sm_*` methods.
    #[cfg(feature = "state-machines")]
    pub fn state_machine_load(&mut self, definition: &str) -> bool {
        // Drop any existing engine first to release its mutable pointer.
        self.state_machine = None;
        match self.player.state_machine_load_data(definition) {
            Ok(engine) => {
                // SAFETY: `DotLottiePlayerWasm` owns both `player` (field 2) and
                // `state_machine` (field 1).  The engine holds a raw `&mut player`
                // reference.  Because `state_machine` is declared before `player`,
                // it is dropped first, so the pointer is never dangling.  We must
                // not create additional `&mut player` references while the engine
                // lives; all player mutations must go through the `sm_*` delegate
                // methods (which call into the engine) or be done only after
                // calling `state_machine_unload`.
                let engine_static = unsafe {
                    std::mem::transmute::<
                        crate::StateMachineEngine<'_>,
                        crate::StateMachineEngine<'static>,
                    >(engine)
                };
                self.state_machine = Some(engine_static);
                true
            }
            Err(_) => false,
        }
    }

    /// Load a state machine from a .lottie archive by state-machine ID.
    #[cfg(all(feature = "state-machines", feature = "dotlottie"))]
    pub fn state_machine_load_from_id(&mut self, id: &str) -> bool {
        self.state_machine = None;
        let Ok(c) = CString::new(id) else { return false; };
        match self.player.state_machine_load(&c) {
            Ok(engine) => {
                let engine_static = unsafe {
                    std::mem::transmute::<
                        crate::StateMachineEngine<'_>,
                        crate::StateMachineEngine<'static>,
                    >(engine)
                };
                self.state_machine = Some(engine_static);
                true
            }
            Err(_) => false,
        }
    }

    /// Unload the active state machine.
    #[cfg(feature = "state-machines")]
    pub fn state_machine_unload(&mut self) {
        self.state_machine = None;
    }

    /// Fire a named event into the state machine.
    #[cfg(feature = "state-machines")]
    pub fn sm_fire(&mut self, event: &str) -> bool {
        let Some(ref mut sm) = self.state_machine else { return false };
        sm.fire(event, true).is_ok()
    }

    #[cfg(feature = "state-machines")]
    pub fn sm_set_numeric_input(&mut self, key: &str, value: f32) -> bool {
        let Some(ref mut sm) = self.state_machine else { return false };
        sm.set_numeric_input(key, value, true, false);
        true
    }

    #[cfg(feature = "state-machines")]
    pub fn sm_get_numeric_input(&self, key: &str) -> Option<f32> {
        self.state_machine.as_ref()?.get_numeric_input(key)
    }

    #[cfg(feature = "state-machines")]
    pub fn sm_set_string_input(&mut self, key: &str, value: &str) -> bool {
        let Some(ref mut sm) = self.state_machine else { return false };
        sm.set_string_input(key, value, true, false);
        true
    }

    #[cfg(feature = "state-machines")]
    pub fn sm_get_string_input(&self, key: &str) -> Option<String> {
        self.state_machine.as_ref()?.get_string_input(key)
    }

    #[cfg(feature = "state-machines")]
    pub fn sm_set_boolean_input(&mut self, key: &str, value: bool) -> bool {
        let Some(ref mut sm) = self.state_machine else { return false };
        sm.set_boolean_input(key, value, true, false);
        true
    }

    #[cfg(feature = "state-machines")]
    pub fn sm_get_boolean_input(&self, key: &str) -> Option<bool> {
        self.state_machine.as_ref()?.get_boolean_input(key)
    }

    #[cfg(feature = "state-machines")]
    pub fn sm_reset_input(&mut self, key: &str) {
        if let Some(ref mut sm) = self.state_machine {
            sm.reset_input(key, true, false);
        }
    }

    /// Poll the next state machine event.  Returns `null` if the queue is empty,
    /// otherwise a JS object with a `type` field and optional payload.
    #[cfg(feature = "state-machines")]
    pub fn sm_poll_event(&mut self) -> JsValue {
        let Some(ref mut sm) = self.state_machine else { return JsValue::null() };
        let Some(evt) = sm.poll_event() else { return JsValue::null() };

        use crate::StateMachineEvent;
        let cstr = |c: &std::ffi::CStr| c.to_string_lossy().into_owned();

        match &evt {
            StateMachineEvent::Start => js_obj_with_type("Start").into(),
            StateMachineEvent::Stop  => js_obj_with_type("Stop").into(),
            StateMachineEvent::Transition { previous_state, new_state } => {
                let obj = js_obj_with_type("Transition");
                set_str(&obj, "previousState", &cstr(previous_state));
                set_str(&obj, "newState",      &cstr(new_state));
                obj.into()
            }
            StateMachineEvent::StateEntered { state } => {
                let obj = js_obj_with_type("StateEntered");
                set_str(&obj, "state", &cstr(state));
                obj.into()
            }
            StateMachineEvent::StateExit { state } => {
                let obj = js_obj_with_type("StateExit");
                set_str(&obj, "state", &cstr(state));
                obj.into()
            }
            StateMachineEvent::CustomEvent { message } => {
                let obj = js_obj_with_type("CustomEvent");
                set_str(&obj, "message", &cstr(message));
                obj.into()
            }
            StateMachineEvent::Error { message } => {
                let obj = js_obj_with_type("Error");
                set_str(&obj, "message", &cstr(message));
                obj.into()
            }
            StateMachineEvent::StringInputChange { name, old_value, new_value } => {
                let obj = js_obj_with_type("StringInputChange");
                set_str(&obj, "name",     &cstr(name));
                set_str(&obj, "oldValue", &cstr(old_value));
                set_str(&obj, "newValue", &cstr(new_value));
                obj.into()
            }
            StateMachineEvent::NumericInputChange { name, old_value, new_value } => {
                let obj = js_obj_with_type("NumericInputChange");
                set_str(&obj, "name", &cstr(name));
                set_f64(&obj, "oldValue", *old_value as f64);
                set_f64(&obj, "newValue", *new_value as f64);
                obj.into()
            }
            StateMachineEvent::BooleanInputChange { name, old_value, new_value } => {
                let obj = js_obj_with_type("BooleanInputChange");
                set_str(&obj, "name", &cstr(name));
                set_bool(&obj, "oldValue", *old_value);
                set_bool(&obj, "newValue", *new_value);
                obj.into()
            }
            StateMachineEvent::InputFired { name } => {
                let obj = js_obj_with_type("InputFired");
                set_str(&obj, "name", &cstr(name));
                obj.into()
            }
        }
    }

    // ── SM lifecycle ──────────────────────────────────────────────────────

    /// Start the state machine with an open-URL policy.
    #[cfg(feature = "state-machines")]
    pub fn sm_start(&mut self, require_user_interaction: bool, whitelist: Vec<JsValue>) -> bool {
        let Some(ref mut sm) = self.state_machine else { return false };
        use crate::actions::open_url_policy::OpenUrlPolicy;
        let policy = OpenUrlPolicy::new(
            whitelist.iter().filter_map(|v| v.as_string()).collect(),
            require_user_interaction,
        );
        sm.start(&policy).is_ok()
    }

    /// Stop the state machine.
    #[cfg(feature = "state-machines")]
    pub fn sm_stop(&mut self) -> bool {
        let Some(ref mut sm) = self.state_machine else { return false };
        sm.stop();
        true
    }

    /// Get the current status of the state machine as a string.
    #[cfg(feature = "state-machines")]
    pub fn sm_status(&self) -> String {
        self.state_machine.as_ref().map(|sm| sm.status()).unwrap_or_default()
    }

    /// Get the name of the current state.
    #[cfg(feature = "state-machines")]
    pub fn sm_current_state(&self) -> String {
        self.state_machine.as_ref().map(|sm| sm.get_current_state_name()).unwrap_or_default()
    }

    /// Override the current state.
    #[cfg(feature = "state-machines")]
    pub fn sm_override_current_state(&mut self, state: &str, immediate: bool) -> bool {
        let Some(ref mut sm) = self.state_machine else { return false };
        sm.override_current_state(state, immediate).is_ok()
    }

    // ── SM introspection ──────────────────────────────────────────────────

    /// Returns the framework setup listeners as a JS array of strings.
    #[cfg(feature = "state-machines")]
    pub fn sm_framework_setup(&self) -> JsValue {
        let Some(ref sm) = self.state_machine else { return Array::new().into() };
        let listeners = sm.framework_setup();
        let arr = Array::new();
        for l in &listeners { arr.push(&l.as_str().into()); }
        arr.into()
    }

    /// Returns all state machine inputs as a JS array of strings.
    #[cfg(feature = "state-machines")]
    pub fn sm_get_inputs(&self) -> JsValue {
        let Some(ref sm) = self.state_machine else { return Array::new().into() };
        let inputs = sm.get_inputs();
        let arr = Array::new();
        for i in &inputs { arr.push(&i.as_str().into()); }
        arr.into()
    }

    // ── SM pointer events ─────────────────────────────────────────────────

    #[cfg(feature = "state-machines")]
    pub fn sm_post_click(&mut self, x: f32, y: f32) {
        if let Some(ref mut sm) = self.state_machine { sm.post_event(&crate::Event::Click { x, y }); }
    }

    #[cfg(feature = "state-machines")]
    pub fn sm_post_pointer_down(&mut self, x: f32, y: f32) {
        if let Some(ref mut sm) = self.state_machine { sm.post_event(&crate::Event::PointerDown { x, y }); }
    }

    #[cfg(feature = "state-machines")]
    pub fn sm_post_pointer_up(&mut self, x: f32, y: f32) {
        if let Some(ref mut sm) = self.state_machine { sm.post_event(&crate::Event::PointerUp { x, y }); }
    }

    #[cfg(feature = "state-machines")]
    pub fn sm_post_pointer_move(&mut self, x: f32, y: f32) {
        if let Some(ref mut sm) = self.state_machine { sm.post_event(&crate::Event::PointerMove { x, y }); }
    }

    #[cfg(feature = "state-machines")]
    pub fn sm_post_pointer_enter(&mut self, x: f32, y: f32) {
        if let Some(ref mut sm) = self.state_machine { sm.post_event(&crate::Event::PointerEnter { x, y }); }
    }

    #[cfg(feature = "state-machines")]
    pub fn sm_post_pointer_exit(&mut self, x: f32, y: f32) {
        if let Some(ref mut sm) = self.state_machine { sm.post_event(&crate::Event::PointerExit { x, y }); }
    }

    // ── SM internal events ────────────────────────────────────────────────

    /// Poll the next state machine internal event.  Returns `null` if the
    /// queue is empty, otherwise a JS object `{ type: "Message", message }`.
    #[cfg(feature = "state-machines")]
    pub fn sm_poll_internal_event(&mut self) -> JsValue {
        let Some(ref mut sm) = self.state_machine else { return JsValue::null() };
        let Some(evt) = sm.poll_internal_event() else { return JsValue::null() };
        match &evt {
            crate::StateMachineInternalEvent::Message { message } => {
                let obj = js_obj_with_type("Message");
                set_str(&obj, "message", &message.to_string_lossy());
                obj.into()
            }
        }
    }

    /// Advance the state machine by one tick.  Returns `false` if no state machine
    /// is loaded, otherwise `true` (even if the machine is stopped or errored).
    #[cfg(feature = "state-machines")]
    pub fn sm_tick(&mut self) -> bool {
        let Some(ref mut sm) = self.state_machine else { return false };
        sm.tick().is_ok()
    }
}

impl Default for DotLottiePlayerWasm {
    fn default() -> Self { Self::new() }
}

// ─── Free functions ──────────────────────────────────────────────────────────

/// Register a font globally (static, not tied to a player instance).
#[wasm_bindgen]
#[cfg(feature = "tvg")]
pub fn register_font(name: &str, data: &[u8]) -> bool {
    DotLottiePlayer::load_font(name, data).is_ok()
}
