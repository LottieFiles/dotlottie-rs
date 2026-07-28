use std::{
    cell::RefCell,
    ffi::{c_char, c_int, CStr, CString},
    ptr,
    result::Result,
};

use rustc_hash::FxHashMap;

#[cfg(feature = "tvg-ttf")]
use crate::renderer::fallback_font;

use super::{
    Animation, ClipRegion, ColorSpace, Drawable, GlContext, GlDisplay, GlSurface, LayerProps,
    CloneProps, Marker, OverlayFill, OverlayProps, Point, Renderer, Rgba, Segment, Shape,
    SpotMask, WgpuDevice,
    WgpuInstance, WgpuTarget, WgpuTargetType,
};
#[cfg(feature = "audio")]
use super::{AudioEvent, AudioResolver, AudioSource};

#[expect(non_upper_case_globals)]
#[allow(non_snake_case)]
#[expect(non_camel_case_types)]
#[expect(dead_code)]
mod tvg {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Debug, thiserror::Error)]
pub enum TvgError {
    #[error("invalid argument")]
    InvalidArgument,
    #[error("insufficient condition")]
    InsufficientCondition,
    #[error("allocation failed")]
    FailedAllocation,
    #[error("memory corruption")]
    MemoryCorruption,
    #[error("not supported")]
    NotSupported,
    #[error("unknown error")]
    Unknown,
}

pub trait IntoResult {
    fn into_result(self) -> Result<(), TvgError>;
}

impl IntoResult for tvg::Tvg_Result {
    fn into_result(self) -> Result<(), TvgError> {
        match self {
            tvg::Tvg_Result_TVG_RESULT_SUCCESS => Ok(()),
            tvg::Tvg_Result_TVG_RESULT_INVALID_ARGUMENT => Err(TvgError::InvalidArgument),
            tvg::Tvg_Result_TVG_RESULT_INSUFFICIENT_CONDITION => {
                Err(TvgError::InsufficientCondition)
            }
            tvg::Tvg_Result_TVG_RESULT_FAILED_ALLOCATION => Err(TvgError::FailedAllocation),
            tvg::Tvg_Result_TVG_RESULT_MEMORY_CORRUPTION => Err(TvgError::MemoryCorruption),
            tvg::Tvg_Result_TVG_RESULT_NOT_SUPPORTED => Err(TvgError::NotSupported),
            tvg::Tvg_Result_TVG_RESULT_UNKNOWN => Err(TvgError::Unknown),
            _ => unreachable!(),
        }
    }
}

impl From<ColorSpace> for tvg::Tvg_Colorspace {
    fn from(color_space: ColorSpace) -> Self {
        match color_space {
            ColorSpace::ABGR8888 => tvg::Tvg_Colorspace_TVG_COLORSPACE_ABGR8888,
            ColorSpace::ABGR8888S => tvg::Tvg_Colorspace_TVG_COLORSPACE_ABGR8888S,
            ColorSpace::ARGB8888 => tvg::Tvg_Colorspace_TVG_COLORSPACE_ARGB8888,
            ColorSpace::ARGB8888S => tvg::Tvg_Colorspace_TVG_COLORSPACE_ARGB8888S,
        }
    }
}

impl From<WgpuTargetType> for std::ffi::c_int {
    fn from(target_type: WgpuTargetType) -> Self {
        target_type as std::ffi::c_int
    }
}

#[non_exhaustive]
enum TvgEngineOption {
    Default,
}
impl From<TvgEngineOption> for tvg::Tvg_Engine_Option {
    fn from(option: TvgEngineOption) -> Self {
        match option {
            TvgEngineOption::Default => tvg::Tvg_Engine_Option_TVG_ENGINE_OPTION_DEFAULT,
        }
    }
}

static RENDERERS_COUNT: std::sync::Mutex<usize> = std::sync::Mutex::new(0);

pub struct TvgRenderer {
    raw_canvas: Option<tvg::Tvg_Canvas>,
}

impl TvgRenderer {
    pub fn new(threads: u32) -> Self {
        let mut count = RENDERERS_COUNT.lock().unwrap();

        if *count == 0 {
            unsafe { tvg::tvg_engine_init(threads).into_result() }.unwrap();

            #[cfg(feature = "tvg-ttf")]
            {
                let (font_name, font_data) = fallback_font::font();
                Self::load_font(font_name, &font_data).unwrap();
            }
        }

        *count += 1;

        TvgRenderer { raw_canvas: None }
    }

    pub fn create_sw_canvas(&mut self) -> Result<(), TvgError> {
        let canvas = unsafe { tvg::tvg_swcanvas_create(TvgEngineOption::Default.into()) };

        if canvas.is_null() {
            return Err(TvgError::FailedAllocation);
        }

        self.raw_canvas = Some(canvas);

        Ok(())
    }

    pub fn create_gl_canvas(&mut self) -> Result<(), TvgError> {
        {
            let canvas = unsafe { tvg::tvg_glcanvas_create(TvgEngineOption::Default.into()) };

            if canvas.is_null() {
                return Err(TvgError::FailedAllocation);
            }

            self.raw_canvas = Some(canvas);

            Ok(())
        }
    }

    pub fn create_wg_canvas(&mut self) -> Result<(), TvgError> {
        unsafe {
            let canvas = tvg::tvg_wgcanvas_create(TvgEngineOption::Default.into());

            if canvas.is_null() {
                return Err(TvgError::FailedAllocation);
            }

            self.raw_canvas = Some(canvas);
            Ok(())
        }
    }
}

impl Renderer for TvgRenderer {
    type Animation = TvgAnimation;
    type Shape = TvgShape;
    type Error = TvgError;

    fn load_font(font_name: &str, font_data: &[u8]) -> Result<(), Self::Error> {
        let font_name_cstr = CString::new(font_name).map_err(|_| TvgError::InvalidArgument)?;
        let font_data_ptr = font_data.as_ptr() as *const ::std::os::raw::c_char;
        let font_size: usize = font_data.len();
        let mimetype_cstr = CString::new("ttf").map_err(|_| TvgError::InvalidArgument)?;
        let copy: bool = true;

        unsafe {
            tvg::tvg_font_load_data(
                font_name_cstr.as_ptr(),
                font_data_ptr,
                font_size as u32,
                mimetype_cstr.as_ptr(),
                copy,
            )
        }
        .into_result()
    }

    fn unload_font(font_name: &str) -> Result<(), Self::Error> {
        let font_name_cstr = CString::new(font_name).map_err(|_| TvgError::InvalidArgument)?;
        unsafe { tvg::tvg_font_unload(font_name_cstr.as_ptr()) }.into_result()
    }

    fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> Result<(), TvgError> {
        if let Some(raw_canvas) = self.raw_canvas {
            self.raw_canvas = Some(raw_canvas);
            unsafe { tvg::tvg_canvas_set_viewport(raw_canvas, x, y, w, h).into_result() }
        } else {
            Err(TvgError::InvalidArgument)
        }
    }

    fn set_sw_target(
        &mut self,
        frame_ptr: &mut [u32],
        stride: u32,
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Result<(), TvgError> {
        if self.raw_canvas.is_none() {
            self.create_sw_canvas()?;
        }

        if let Some(raw_canvas) = self.raw_canvas {
            unsafe {
                tvg::tvg_swcanvas_set_target(
                    raw_canvas,
                    frame_ptr.as_mut_ptr(),
                    stride,
                    width,
                    height,
                    color_space.into(),
                )
                .into_result()
            }
        } else {
            Err(TvgError::InvalidArgument)
        }
    }

    fn set_gl_target(
        &mut self,
        display: &dyn GlDisplay,
        surface: &dyn GlSurface,
        context: &dyn GlContext,
        id: i32,
        width: u32,
        height: u32,
    ) -> Result<(), Self::Error> {
        if self.raw_canvas.is_none() {
            self.create_gl_canvas()?;
        }

        if let Some(raw_canvas) = self.raw_canvas {
            unsafe {
                tvg::tvg_glcanvas_set_target(
                    raw_canvas,
                    display.as_ptr(),
                    surface.as_ptr(),
                    context.as_ptr(),
                    id,
                    width,
                    height,
                    tvg::Tvg_Colorspace_TVG_COLORSPACE_ABGR8888S,
                )
                .into_result()
            }
        } else {
            Err(TvgError::InvalidArgument)
        }
    }

    fn set_wg_target(
        &mut self,
        device: &dyn WgpuDevice,
        instance: &dyn WgpuInstance,
        target: &dyn WgpuTarget,
        width: u32,
        height: u32,
        target_type: WgpuTargetType,
    ) -> Result<(), Self::Error> {
        if self.raw_canvas.is_none() {
            self.create_wg_canvas()?;
        }

        if let Some(raw_canvas) = self.raw_canvas {
            let device_ptr = device.as_ptr();
            let actual_device = if device_ptr.is_null() {
                std::ptr::null_mut()
            } else {
                device_ptr
            };

            unsafe {
                tvg::tvg_wgcanvas_set_target(
                    raw_canvas,
                    actual_device,
                    instance.as_ptr(),
                    target.as_ptr(),
                    width,
                    height,
                    tvg::Tvg_Colorspace_TVG_COLORSPACE_ABGR8888S,
                    target_type.into(),
                )
            }
            .into_result()?;

            unsafe { tvg::tvg_canvas_sync(raw_canvas).into_result() }?;

            Ok(())
        } else {
            Err(TvgError::InvalidArgument)
        }
    }

    fn clear(&self) -> Result<(), TvgError> {
        if let Some(raw_canvas) = self.raw_canvas {
            unsafe { tvg::tvg_canvas_remove(raw_canvas, ptr::null_mut()).into_result() }
        } else {
            Err(TvgError::InvalidArgument)
        }
    }

    fn push(&mut self, drawable: Drawable<Self>) -> Result<(), TvgError> {
        if let Some(raw_canvas) = self.raw_canvas {
            let raw_paint = match drawable {
                Drawable::Animation(animation) => animation.raw_scene,
                Drawable::Shape(shape) => shape.raw_shape,
            };

            unsafe { tvg::tvg_canvas_add(raw_canvas, raw_paint).into_result() }
        } else {
            Err(TvgError::InvalidArgument)
        }
    }

    fn insert(&mut self, drawable: Drawable<Self>, at: Drawable<Self>) -> Result<(), TvgError> {
        if let Some(raw_canvas) = self.raw_canvas {
            let target = match drawable {
                Drawable::Animation(animation) => animation.raw_scene,
                Drawable::Shape(shape) => shape.raw_shape,
            };
            let at_paint = match at {
                Drawable::Animation(animation) => animation.raw_scene,
                Drawable::Shape(shape) => shape.raw_shape,
            };

            unsafe { tvg::tvg_canvas_insert(raw_canvas, target, at_paint).into_result() }
        } else {
            Err(TvgError::InvalidArgument)
        }
    }

    fn draw(&mut self, clear_buffer: bool) -> Result<(), TvgError> {
        if let Some(raw_canvas) = self.raw_canvas {
            unsafe { tvg::tvg_canvas_draw(raw_canvas, clear_buffer).into_result() }
        } else {
            Err(TvgError::InvalidArgument)
        }
    }

    fn sync(&mut self) -> Result<(), TvgError> {
        if let Some(raw_canvas) = self.raw_canvas {
            unsafe { tvg::tvg_canvas_sync(raw_canvas).into_result() }
        } else {
            Err(TvgError::InvalidArgument)
        }
    }

    fn update(&mut self) -> Result<(), TvgError> {
        if let Some(raw_canvas) = self.raw_canvas {
            unsafe {
                let res = tvg::tvg_canvas_update(raw_canvas);
                res.into_result()
            }
        } else {
            Err(TvgError::InvalidArgument)
        }
    }
}

impl Drop for TvgRenderer {
    fn drop(&mut self) {
        let mut count = RENDERERS_COUNT.lock().unwrap();

        if let Some(raw_canvas) = self.raw_canvas {
            unsafe {
                tvg::tvg_canvas_destroy(raw_canvas);
            }
        }

        *count = count.checked_sub(1).unwrap();

        if *count == 0 {
            unsafe { tvg::tvg_engine_term() };
        }
    }
}

struct LayerIdMap {
    cache: RefCell<FxHashMap<String, u32>>,
}

impl LayerIdMap {
    fn new() -> Self {
        Self {
            cache: RefCell::new(FxHashMap::default()),
        }
    }

    fn get_or_insert(&self, layer_name: &str) -> Result<u32, TvgError> {
        if let Some(&id) = self.cache.borrow().get(layer_name) {
            return Ok(id);
        }
        let cstr = CString::new(layer_name).map_err(|_| TvgError::InvalidArgument)?;
        let id = unsafe { tvg::tvg_accessor_generate_id(cstr.as_ptr()) };
        self.cache.borrow_mut().insert(layer_name.to_owned(), id);
        Ok(id)
    }

    fn clear(&self) {
        self.cache.borrow_mut().clear();
    }
}

/// Pristine animated values of a layer paint, read before user props are
/// composed on top. Invalidated whenever ThorVG rebuilds the layer scenes.
struct LayerBase {
    matrix: tvg::Tvg_Matrix,
    opacity: u8,
    /// Whether a user blur is currently attached to this (still-live) scene.
    has_blur: bool,
    /// Whether a user clip is currently attached to this (still-live) scene.
    has_clip: bool,
}

/// Fresh clipper shape for `tvg_paint_set_clip`; ThorVG refs it on attach and
/// frees it on replace/clear, so the caller must hand it over untouched.
fn make_clip_shape(region: &ClipRegion) -> tvg::Tvg_Paint {
    unsafe {
        let shape = tvg::tvg_shape_new();
        match *region {
            ClipRegion::Rect { x, y, w, h, rx, ry } => {
                let _ = tvg::tvg_shape_append_rect(shape, x, y, w, h, rx, ry, true);
            }
            ClipRegion::Circle { cx, cy, rx, ry } => {
                let _ = tvg::tvg_shape_append_circle(shape, cx, cy, rx, ry, true);
            }
            ClipRegion::Path { ref cmds, ref pts } => {
                let _ = tvg::tvg_shape_append_path(
                    shape,
                    cmds.as_ptr(),
                    cmds.len() as u32,
                    pts.as_ptr() as *const tvg::Tvg_Point,
                    (pts.len() / 2) as u32,
                );
            }
        }
        shape
    }
}

/// Circle whose alpha ramps from opaque to transparent over the feathered rim,
/// used as the alpha-mask target.
fn make_spot_mask_shape(mask: &SpotMask) -> tvg::Tvg_Paint {
    unsafe {
        let shape = tvg::tvg_shape_new();
        let _ =
            tvg::tvg_shape_append_circle(shape, mask.cx, mask.cy, mask.radius, mask.radius, true);
        let feather = mask.feather.clamp(0.0, 1.0);
        if feather > 0.0 {
            let stop = |offset: f32, a: u8| tvg::Tvg_Color_Stop {
                offset,
                r: 255,
                g: 255,
                b: 255,
                a,
            };
            let stops = [stop(0.0, 255), stop(1.0 - feather, 255), stop(1.0, 0)];
            let grad = tvg::tvg_radial_gradient_new();
            let _ = tvg::tvg_radial_gradient_set(
                grad,
                mask.cx,
                mask.cy,
                mask.radius,
                mask.cx,
                mask.cy,
                0.0,
            );
            let _ = tvg::tvg_gradient_set_color_stops(grad, stops.as_ptr(), stops.len() as u32);
            let _ = tvg::tvg_shape_set_gradient(shape, grad);
        } else {
            let _ = tvg::tvg_shape_set_fill_color(shape, 255, 255, 255, 255);
        }
        shape
    }
}

pub struct TvgAnimation {
    raw_animation: tvg::Tvg_Animation,
    raw_paint: tvg::Tvg_Paint,
    // Wrapper scene holding the picture; effects/opacity/blend target this.
    // TvgAnimation owns one ref so the pointer survives canvas clear.
    raw_scene: tvg::Tvg_Paint,
    // Scene-owned procedural shapes; pointers stay valid until removed.
    overlay_paints: FxHashMap<u32, tvg::Tvg_Paint>,
    // Scene-owned layer duplicates + their baked comp→canvas base matrix.
    clone_paints: FxHashMap<u32, (tvg::Tvg_Paint, tvg::Tvg_Matrix)>,
    layer_base: FxHashMap<String, LayerBase>,
    data: Option<CString>,
    segment: Option<Segment>,
    markers: Vec<Marker>,
    total_frames: f32,
    duration: f32,
    layer_id_map: LayerIdMap,
    // Boxed for a stable address to pass as the callback's user data.
    #[cfg(feature = "audio")]
    audio_resolver: Option<Box<AudioResolver>>,
}

/// Bridges ThorVG's C audio callback to the Rust resolver stored in `data`.
#[cfg(feature = "audio")]
unsafe extern "C" fn audio_resolver_trampoline(
    info: *const tvg::Tvg_Audio_Info,
    data: *mut std::ffi::c_void,
) {
    if info.is_null() || data.is_null() {
        return;
    }
    let resolver = unsafe { &mut *(data as *mut AudioResolver) };
    let info = unsafe { &*info };

    let source = if info.embedded {
        let bytes = if info.src.is_null() || info.size == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(info.src as *const u8, info.size as usize) }
        };
        let mime = if info.mimeType.is_null() {
            None
        } else {
            unsafe { CStr::from_ptr(info.mimeType) }.to_str().ok()
        };
        AudioSource::Embedded { bytes, mime }
    } else if info.src.is_null() {
        return;
    } else {
        match unsafe { CStr::from_ptr(info.src) }.to_str() {
            Ok(s) => AudioSource::External(s),
            Err(_) => return,
        }
    };

    resolver(AudioEvent {
        source,
        offset: info.offset,
        volume: info.volume,
        active: info.active,
    });
}

impl Default for TvgAnimation {
    fn default() -> Self {
        let raw_animation = unsafe { tvg::tvg_animation_new() };
        let raw_paint = unsafe { tvg::tvg_animation_get_picture(raw_animation) };
        let raw_scene = unsafe {
            let scene = tvg::tvg_scene_new();
            tvg::tvg_scene_add(scene, raw_paint);
            tvg::tvg_paint_ref(scene);
            scene
        };

        Self {
            raw_animation,
            raw_paint,
            raw_scene,
            overlay_paints: FxHashMap::default(),
            clone_paints: FxHashMap::default(),
            layer_base: FxHashMap::default(),
            data: None,
            segment: None,
            markers: Vec::new(),
            total_frames: 0.0,
            duration: 0.0,
            layer_id_map: LayerIdMap::new(),
            #[cfg(feature = "audio")]
            audio_resolver: None,
        }
    }
}

impl TvgAnimation {
    fn load_markers(&mut self) {
        let mut cnt: u32 = 0;
        unsafe {
            tvg::tvg_lottie_animation_get_markers_cnt(self.raw_animation, &mut cnt);
        }

        self.markers.clear();
        self.markers.reserve(cnt as usize);

        for i in 0..cnt {
            let mut name_ptr: *const c_char = ptr::null();
            let mut begin: f32 = 0.0;
            let mut end: f32 = 0.0;

            let ok = unsafe {
                tvg::tvg_lottie_animation_get_marker_info(
                    self.raw_animation,
                    i,
                    &mut name_ptr,
                    &mut begin,
                    &mut end,
                )
            };

            if ok == tvg::Tvg_Result_TVG_RESULT_SUCCESS && !name_ptr.is_null() {
                let name = unsafe { CStr::from_ptr(name_ptr) }.to_owned();
                self.markers.push(Marker {
                    name,
                    segment: Segment { start: begin, end },
                });
            }
        }
    }

    fn get_total_frame(&self) -> Result<f32, TvgError> {
        let mut total_frame: f32 = 0.0;
        unsafe {
            tvg::tvg_animation_get_total_frame(self.raw_animation, &mut total_frame as *mut f32)
                .into_result()
        }?;
        Ok(total_frame)
    }

    fn get_duration(&self) -> Result<f32, TvgError> {
        let mut duration: f32 = 0.0;
        unsafe {
            tvg::tvg_animation_get_duration(self.raw_animation, &mut duration as *mut f32)
                .into_result()
        }?;
        Ok(duration * 1000.0)
    }

    fn get_layer_obb(&self, layer_name: &str) -> Result<Option<[tvg::Tvg_Point; 4]>, TvgError> {
        let layer_id = self.layer_id_map.get_or_insert(layer_name)?;
        unsafe {
            let mut obb: [tvg::Tvg_Point; 4] = [tvg::Tvg_Point { x: 0.0, y: 0.0 }; 4];
            let layer_paint = tvg::tvg_picture_get_paint(self.raw_paint, layer_id);

            if !layer_paint.is_null() {
                tvg::tvg_paint_get_obb(layer_paint as tvg::Tvg_Paint, obb.as_mut_ptr());
                Ok(Some(obb))
            } else {
                Ok(None)
            }
        }
    }

    unsafe fn tvg_load_data_dispatch(
        raw_paint: tvg::Tvg_Paint,
        data_ptr: *const c_char,
        data_len: u32,
        mimetype_ptr: *const c_char,
    ) -> Result<(), TvgError> {
        tvg::tvg_picture_load_data(
            raw_paint,
            data_ptr,
            data_len,
            mimetype_ptr,
            ptr::null(),
            false,
        )
        .into_result()
    }
}

const TVG_IDENTITY: tvg::Tvg_Matrix = tvg::Tvg_Matrix {
    e11: 1.0,
    e12: 0.0,
    e13: 0.0,
    e21: 0.0,
    e22: 1.0,
    e23: 0.0,
    e31: 0.0,
    e32: 0.0,
    e33: 1.0,
};

fn tvg_mat_mul(a: &tvg::Tvg_Matrix, b: &tvg::Tvg_Matrix) -> tvg::Tvg_Matrix {
    tvg::Tvg_Matrix {
        e11: a.e11 * b.e11 + a.e12 * b.e21 + a.e13 * b.e31,
        e12: a.e11 * b.e12 + a.e12 * b.e22 + a.e13 * b.e32,
        e13: a.e11 * b.e13 + a.e12 * b.e23 + a.e13 * b.e33,
        e21: a.e21 * b.e11 + a.e22 * b.e21 + a.e23 * b.e31,
        e22: a.e21 * b.e12 + a.e22 * b.e22 + a.e23 * b.e32,
        e23: a.e21 * b.e13 + a.e22 * b.e23 + a.e23 * b.e33,
        e31: a.e31 * b.e11 + a.e32 * b.e21 + a.e33 * b.e31,
        e32: a.e31 * b.e12 + a.e32 * b.e22 + a.e33 * b.e32,
        e33: a.e31 * b.e13 + a.e32 * b.e23 + a.e33 * b.e33,
    }
}

fn tvg_color_stops(stops: &[(f32, [u8; 4])]) -> Vec<tvg::Tvg_Color_Stop> {
    stops
        .iter()
        .map(|&(offset, [r, g, b, a])| tvg::Tvg_Color_Stop { offset, r, g, b, a })
        .collect()
}

fn tvg_mat_from(m: &[f32; 9]) -> tvg::Tvg_Matrix {
    tvg::Tvg_Matrix {
        e11: m[0],
        e12: m[1],
        e13: m[2],
        e21: m[3],
        e22: m[4],
        e23: m[5],
        e31: m[6],
        e32: m[7],
        e33: m[8],
    }
}

impl Animation for TvgAnimation {
    type Error = TvgError;

    fn load_data(&mut self, data: &CStr, mimetype: &CStr) -> Result<(), TvgError> {
        let data_owned = data.to_owned();
        let data_len_u32 =
            u32::try_from(data.to_bytes().len()).map_err(|_| TvgError::InvalidArgument)?;

        let result = unsafe {
            TvgAnimation::tvg_load_data_dispatch(
                self.raw_paint,
                data_owned.as_ptr(),
                data_len_u32,
                mimetype.as_ptr(),
            )
        };

        match result {
            Ok(()) => {
                // Keep the payload alive for ThorVG
                self.data = Some(data_owned);
                self.total_frames = self.get_total_frame()?;
                self.duration = self.get_duration()?;
                self.load_markers();
                self.layer_id_map.clear();
                Ok(())
            }
            Err(e) => {
                self.data = None;
                self.markers.clear();
                self.total_frames = 0.0;
                self.duration = 0.0;
                self.layer_id_map.clear();
                Err(e)
            }
        }
    }

    #[cfg(feature = "audio")]
    fn set_audio_resolver(&mut self, resolver: Option<AudioResolver>) -> Result<(), TvgError> {
        match resolver {
            Some(cb) => {
                let mut boxed: Box<AudioResolver> = Box::new(cb);
                let data = (&mut *boxed) as *mut AudioResolver as *mut std::ffi::c_void;
                let result = unsafe {
                    tvg::tvg_lottie_animation_set_audio_resolver(
                        self.raw_animation,
                        Some(audio_resolver_trampoline),
                        data,
                    )
                };
                self.audio_resolver = Some(boxed);
                result.into_result()
            }
            None => {
                let result = unsafe {
                    tvg::tvg_lottie_animation_set_audio_resolver(
                        self.raw_animation,
                        None,
                        std::ptr::null_mut(),
                    )
                };
                self.audio_resolver = None;
                result.into_result()
            }
        }
    }

    fn hit_test(&self, point: Point, layer_name: &str) -> Result<bool, TvgError> {
        if let Some(obb) = self.get_layer_obb(layer_name)? {
            // OBB edge vectors from the origin corner
            let (e1x, e1y) = (obb[1].x - obb[0].x, obb[1].y - obb[0].y);
            let (e2x, e2y) = (obb[3].x - obb[0].x, obb[3].y - obb[0].y);

            let e1_len_sq = e1x * e1x + e1y * e1y;
            let e2_len_sq = e2x * e2x + e2y * e2y;

            // Degenerate OBB (zero-area layer) — cannot contain any point
            if e1_len_sq == 0.0 || e2_len_sq == 0.0 {
                return Ok(false);
            }

            // Vector from OBB origin to the test point
            let (ox, oy) = (point.x - obb[0].x, point.y - obb[0].y);

            // Project onto first edge — early exit if outside [0, 1]
            let u = (ox * e1x + oy * e1y) / e1_len_sq;
            if !(0.0..=1.0).contains(&u) {
                return Ok(false);
            }

            // Project onto second edge
            let v = (ox * e2x + oy * e2y) / e2_len_sq;
            Ok((0.0..=1.0).contains(&v))
        } else {
            Ok(false)
        }
    }

    fn get_size(&self) -> Result<(f32, f32), TvgError> {
        let mut width = 0.0;
        let mut height = 0.0;

        unsafe {
            tvg::tvg_picture_get_size(
                self.raw_paint,
                &mut width as *mut f32,
                &mut height as *mut f32,
            )
            .into_result()
        }?;

        Ok((width, height))
    }

    fn set_size(&mut self, width: f32, height: f32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_picture_set_size(self.raw_paint, width, height).into_result() }
    }

    fn get_total_frame(&self) -> Result<f32, TvgError> {
        Ok(self.total_frames)
    }

    fn get_duration(&self) -> Result<f32, TvgError> {
        Ok(self.duration)
    }

    fn set_frame(&mut self, frame_no: f32) -> Result<(), TvgError> {
        if let Some(Segment { start, end }) = self.segment {
            if frame_no < start || frame_no > end {
                return Err(TvgError::InvalidArgument);
            }
        }
        unsafe { tvg::tvg_animation_set_frame(self.raw_animation, frame_no).into_result() }?;
        // Frame changes rebuild the layer scenes; cached base values are stale.
        self.layer_base.clear();
        Ok(())
    }

    fn gen_slot(&mut self, slot_json: &CStr) -> Result<u32, TvgError> {
        let slot_code =
            unsafe { tvg::tvg_lottie_animation_gen_slot(self.raw_animation, slot_json.as_ptr()) };
        if slot_code == 0 {
            return Err(TvgError::InvalidArgument);
        }
        Ok(slot_code)
    }

    fn apply_slot(&mut self, slot_code: u32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_lottie_animation_apply_slot(self.raw_animation, slot_code) }
            .into_result()?;
        // Slot application rebuilds the layer scenes on next sync.
        self.layer_base.clear();
        Ok(())
    }

    fn del_slot(&mut self, slot_code: u32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_lottie_animation_del_slot(self.raw_animation, slot_code) }
            .into_result()?;
        self.layer_base.clear();
        Ok(())
    }

    fn set_quality(&mut self, quality: u8) -> Result<(), TvgError> {
        unsafe { tvg::tvg_lottie_animation_set_quality(self.raw_animation, quality).into_result() }
    }

    fn tween_to(&mut self, to: f32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_lottie_animation_tween_to(self.raw_animation, to).into_result() }?;
        self.layer_base.clear();
        Ok(())
    }

    fn tween_go(&mut self, progress: f32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_lottie_animation_tween_go(self.raw_animation, progress).into_result() }?;
        self.layer_base.clear();
        Ok(())
    }

    fn set_transform(&mut self, matrix: &[f32; 9]) -> Result<(), TvgError> {
        let tvg_matrix = tvg::Tvg_Matrix {
            e11: matrix[0],
            e12: matrix[1],
            e13: matrix[2],
            e21: matrix[3],
            e22: matrix[4],
            e23: matrix[5],
            e31: matrix[6],
            e32: matrix[7],
            e33: matrix[8],
        };

        unsafe { tvg::tvg_paint_set_transform(self.raw_paint, &tvg_matrix).into_result() }
    }

    // ── Paint-level ops (POC) ────────────────────────────────────────────

    fn set_opacity(&mut self, opacity: u8) -> Result<(), TvgError> {
        unsafe { tvg::tvg_paint_set_opacity(self.raw_scene, opacity).into_result() }
    }

    fn set_blend_method(&mut self, method: u8) -> Result<(), TvgError> {
        unsafe {
            tvg::tvg_paint_set_blend_method(self.raw_scene, method as tvg::Tvg_Blend_Method)
                .into_result()
        }
    }

    fn clear_effects(&mut self) -> Result<(), TvgError> {
        unsafe { tvg::tvg_scene_clear_effects(self.raw_scene).into_result() }
    }

    fn add_gaussian_blur(
        &mut self,
        sigma: f32,
        direction: u8,
        border: u8,
        quality: u8,
    ) -> Result<(), TvgError> {
        unsafe {
            tvg::tvg_scene_add_effect_gaussian_blur(
                self.raw_scene,
                sigma as f64,
                direction as c_int,
                border as c_int,
                quality as c_int,
            )
            .into_result()
        }
    }

    fn add_drop_shadow(
        &mut self,
        color: [u8; 4],
        angle: f32,
        distance: f32,
        sigma: f32,
        quality: u8,
    ) -> Result<(), TvgError> {
        unsafe {
            tvg::tvg_scene_add_effect_drop_shadow(
                self.raw_scene,
                color[0] as c_int,
                color[1] as c_int,
                color[2] as c_int,
                color[3] as c_int,
                angle as f64,
                distance as f64,
                sigma as f64,
                quality as c_int,
            )
            .into_result()
        }
    }

    fn add_fill_effect(&mut self, color: [u8; 4]) -> Result<(), TvgError> {
        unsafe {
            tvg::tvg_scene_add_effect_fill(
                self.raw_scene,
                color[0] as c_int,
                color[1] as c_int,
                color[2] as c_int,
                color[3] as c_int,
            )
            .into_result()
        }
    }

    fn add_tint(&mut self, black: [u8; 3], white: [u8; 3], intensity: f32) -> Result<(), TvgError> {
        unsafe {
            tvg::tvg_scene_add_effect_tint(
                self.raw_scene,
                black[0] as c_int,
                black[1] as c_int,
                black[2] as c_int,
                white[0] as c_int,
                white[1] as c_int,
                white[2] as c_int,
                intensity as f64,
            )
            .into_result()
        }
    }

    fn add_tritone(
        &mut self,
        shadow: [u8; 3],
        midtone: [u8; 3],
        highlight: [u8; 3],
        blend: u8,
    ) -> Result<(), TvgError> {
        unsafe {
            tvg::tvg_scene_add_effect_tritone(
                self.raw_scene,
                shadow[0] as c_int,
                shadow[1] as c_int,
                shadow[2] as c_int,
                midtone[0] as c_int,
                midtone[1] as c_int,
                midtone[2] as c_int,
                highlight[0] as c_int,
                highlight[1] as c_int,
                highlight[2] as c_int,
                blend as c_int,
            )
            .into_result()
        }
    }

    fn set_clip(&mut self, region: Option<&ClipRegion>) -> Result<(), TvgError> {
        let clipper = match region {
            Some(region) => make_clip_shape(region),
            None => ptr::null_mut(),
        };
        unsafe { tvg::tvg_paint_set_clip(self.raw_scene, clipper).into_result() }
    }

    fn set_mask(&mut self, mask: Option<&SpotMask>) -> Result<(), TvgError> {
        let (target, method) = match mask {
            Some(mask) => (
                make_spot_mask_shape(mask),
                if mask.inverse {
                    tvg::Tvg_Mask_Method_TVG_MASK_METHOD_INVERSE_ALPHA
                } else {
                    tvg::Tvg_Mask_Method_TVG_MASK_METHOD_ALPHA
                },
            ),
            None => (ptr::null_mut(), tvg::Tvg_Mask_Method_TVG_MASK_METHOD_NONE),
        };
        unsafe { tvg::tvg_paint_set_mask_method(self.raw_scene, target, method).into_result() }
    }

    fn intersects_layer(
        &self,
        layer_name: &str,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        visible_only: bool,
    ) -> Result<bool, TvgError> {
        let layer_id = self.layer_id_map.get_or_insert(layer_name)?;
        let layer_paint = unsafe { tvg::tvg_picture_get_paint(self.raw_paint, layer_id) };
        if layer_paint.is_null() {
            return Ok(false);
        }
        Ok(unsafe {
            tvg::tvg_paint_intersects_region(
                layer_paint as tvg::Tvg_Paint,
                x,
                y,
                w,
                h,
                visible_only,
            )
        })
    }

    fn get_layer_aabb(&self, layer_name: &str) -> Result<Option<[f32; 4]>, TvgError> {
        let layer_id = self.layer_id_map.get_or_insert(layer_name)?;
        unsafe {
            let layer_paint = tvg::tvg_picture_get_paint(self.raw_paint, layer_id);
            if layer_paint.is_null() {
                return Ok(None);
            }
            let (mut x, mut y, mut w, mut h) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
            tvg::tvg_paint_get_aabb(
                layer_paint as tvg::Tvg_Paint,
                &mut x,
                &mut y,
                &mut w,
                &mut h,
            )
            .into_result()?;
            Ok(Some([x, y, w, h]))
        }
    }

    fn sync_clone(&mut self, id: u32, props: &CloneProps) -> Result<(), TvgError> {
        let (paint, base) = match self.clone_paints.get(&id) {
            Some(&entry) => entry,
            None => {
                let layer_id = self.layer_id_map.get_or_insert(&props.layer)?;
                let layer_paint = unsafe { tvg::tvg_picture_get_paint(self.raw_paint, layer_id) };
                if layer_paint.is_null() {
                    return Err(TvgError::InvalidArgument);
                }
                let dup = unsafe { tvg::tvg_paint_duplicate(layer_paint as tvg::Tvg_Paint) };
                if dup.is_null() {
                    return Err(TvgError::Unknown);
                }
                // The duplicate leaves the picture's subtree, so it must carry
                // the picture's comp→canvas transform itself.
                let mut pm = TVG_IDENTITY;
                let mut lm = TVG_IDENTITY;
                unsafe {
                    tvg::tvg_paint_get_transform(self.raw_paint, &mut pm).into_result()?;
                    tvg::tvg_paint_get_transform(layer_paint as tvg::Tvg_Paint, &mut lm)
                        .into_result()?;
                    if props.below {
                        tvg::tvg_scene_insert(self.raw_scene, dup, self.raw_paint).into_result()?;
                    } else {
                        tvg::tvg_scene_add(self.raw_scene, dup).into_result()?;
                    }
                }
                let base = tvg_mat_mul(&pm, &lm);
                self.clone_paints.insert(id, (dup, base));
                (dup, base)
            }
        };
        let composed = match props.transform.as_ref() {
            Some(user) => tvg_mat_mul(&tvg_mat_from(user), &base),
            None => base,
        };
        unsafe {
            tvg::tvg_paint_set_transform(paint, &composed).into_result()?;
            if let Some(opacity) = props.opacity {
                tvg::tvg_paint_set_opacity(paint, opacity).into_result()?;
            }
        }
        Ok(())
    }

    fn remove_clone(&mut self, id: u32) -> Result<(), TvgError> {
        if let Some((paint, _)) = self.clone_paints.remove(&id) {
            unsafe { tvg::tvg_scene_remove(self.raw_scene, paint).into_result()? };
        }
        Ok(())
    }

    fn has_clone(&self, id: u32) -> bool {
        self.clone_paints.contains_key(&id)
    }

    fn sync_overlay(&mut self, id: u32, props: &OverlayProps) -> Result<(), TvgError> {
        let shape = match self.overlay_paints.get(&id) {
            Some(&shape) => {
                unsafe { tvg::tvg_shape_reset(shape).into_result()? };
                shape
            }
            None => unsafe {
                let shape = tvg::tvg_shape_new();
                let _ = tvg::tvg_shape_set_stroke_cap(shape, tvg::Tvg_Stroke_Cap_TVG_STROKE_CAP_ROUND);
                let _ =
                    tvg::tvg_shape_set_stroke_join(shape, tvg::Tvg_Stroke_Join_TVG_STROKE_JOIN_ROUND);
                if props.below {
                    tvg::tvg_scene_insert(self.raw_scene, shape, self.raw_paint).into_result()?;
                } else {
                    tvg::tvg_scene_add(self.raw_scene, shape).into_result()?;
                }
                self.overlay_paints.insert(id, shape);
                shape
            },
        };
        unsafe {
            if !props.cmds.is_empty() {
                tvg::tvg_shape_append_path(
                    shape,
                    props.cmds.as_ptr(),
                    props.cmds.len() as u32,
                    props.pts.as_ptr() as *const tvg::Tvg_Point,
                    (props.pts.len() / 2) as u32,
                )
                .into_result()?;
            }
            // Gradients are hand-over: the shape owns them and frees the
            // previous fill on replace, same lifecycle as clip shapes.
            match &props.fill {
                Some(OverlayFill::Linear { x1, y1, x2, y2, stops }) => {
                    let grad = tvg::tvg_linear_gradient_new();
                    tvg::tvg_linear_gradient_set(grad, *x1, *y1, *x2, *y2).into_result()?;
                    let stops = tvg_color_stops(stops);
                    tvg::tvg_gradient_set_color_stops(grad, stops.as_ptr(), stops.len() as u32)
                        .into_result()?;
                    tvg::tvg_shape_set_gradient(shape, grad).into_result()?;
                }
                Some(OverlayFill::Radial { cx, cy, r, stops }) => {
                    let grad = tvg::tvg_radial_gradient_new();
                    tvg::tvg_radial_gradient_set(grad, *cx, *cy, *r, *cx, *cy, 0.0)
                        .into_result()?;
                    let stops = tvg_color_stops(stops);
                    tvg::tvg_gradient_set_color_stops(grad, stops.as_ptr(), stops.len() as u32)
                        .into_result()?;
                    tvg::tvg_shape_set_gradient(shape, grad).into_result()?;
                }
                Some(OverlayFill::Solid([r, g, b, a])) => {
                    tvg::tvg_shape_set_fill_color(shape, *r, *g, *b, *a).into_result()?;
                }
                None => {
                    tvg::tvg_shape_set_fill_color(shape, 0, 0, 0, 0).into_result()?;
                }
            }
            let (width, [sr, sg, sb, sa]) = props.stroke.unwrap_or((0.0, [0, 0, 0, 0]));
            tvg::tvg_shape_set_stroke_width(shape, width).into_result()?;
            tvg::tvg_shape_set_stroke_color(shape, sr, sg, sb, sa).into_result()?;
            if props.dash.is_empty() {
                // Upstream: clearing with cnt=0 leaves a stale dash.length and
                // the SW engine then dashes with a null pattern (segfault) —
                // clear via a zero-length pattern instead.
                let zero = [0.0f32];
                tvg::tvg_shape_set_stroke_dash(shape, zero.as_ptr(), 1, 0.0).into_result()?;
            } else {
                tvg::tvg_shape_set_stroke_dash(
                    shape,
                    props.dash.as_ptr(),
                    props.dash.len() as u32,
                    0.0,
                )
                .into_result()?;
            }
            let m = props
                .transform
                .unwrap_or([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
            let matrix = tvg::Tvg_Matrix {
                e11: m[0],
                e12: m[1],
                e13: m[2],
                e21: m[3],
                e22: m[4],
                e23: m[5],
                e31: m[6],
                e32: m[7],
                e33: m[8],
            };
            tvg::tvg_paint_set_transform(shape, &matrix).into_result()
        }
    }

    fn remove_overlay(&mut self, id: u32) -> Result<(), TvgError> {
        if let Some(shape) = self.overlay_paints.remove(&id) {
            unsafe { tvg::tvg_scene_remove(self.raw_scene, shape).into_result()? };
        }
        Ok(())
    }

    // NOTE: callers must poke the picture (re-set its transform) once per
    // flush batch — the update traversal short-circuits at a clean picture
    // (PictureImpl::skip) and never sees dirty children. The renderer does
    // this via apply_user_transform() after the flush loop.
    fn apply_layer_prop(&mut self, layer_name: &str, props: &LayerProps) -> Result<(), TvgError> {
        let layer_id = self.layer_id_map.get_or_insert(layer_name)?;
        let layer_paint = unsafe { tvg::tvg_picture_get_paint(self.raw_paint, layer_id) };
        if layer_paint.is_null() {
            // Layer outside its in/out range or animated to opacity 0.
            self.layer_base.remove(layer_name);
            return Ok(());
        }

        // Read the pristine animated values once per rebuilt scene; user props
        // always compose against this base so re-applies stay idempotent.
        let base = match self.layer_base.get_mut(layer_name) {
            Some(base) => base,
            None => {
                let mut matrix = tvg::Tvg_Matrix {
                    e11: 1.0,
                    e12: 0.0,
                    e13: 0.0,
                    e21: 0.0,
                    e22: 1.0,
                    e23: 0.0,
                    e31: 0.0,
                    e32: 0.0,
                    e33: 1.0,
                };
                let mut base_opacity: u8 = 255;
                unsafe {
                    tvg::tvg_paint_get_transform(layer_paint, &mut matrix).into_result()?;
                    tvg::tvg_paint_get_opacity(layer_paint, &mut base_opacity).into_result()?;
                }
                self.layer_base.entry(layer_name.to_owned()).or_insert(LayerBase {
                    matrix,
                    opacity: base_opacity,
                    has_blur: false,
                    has_clip: false,
                })
            }
        };

        if let Some(user) = props.transform.as_ref() {
            let m = &base.matrix;
            let composed = tvg::Tvg_Matrix {
                e11: user[0] * m.e11 + user[1] * m.e21 + user[2] * m.e31,
                e12: user[0] * m.e12 + user[1] * m.e22 + user[2] * m.e32,
                e13: user[0] * m.e13 + user[1] * m.e23 + user[2] * m.e33,
                e21: user[3] * m.e11 + user[4] * m.e21 + user[5] * m.e31,
                e22: user[3] * m.e12 + user[4] * m.e22 + user[5] * m.e32,
                e23: user[3] * m.e13 + user[4] * m.e23 + user[5] * m.e33,
                e31: user[6] * m.e11 + user[7] * m.e21 + user[8] * m.e31,
                e32: user[6] * m.e12 + user[7] * m.e22 + user[8] * m.e32,
                e33: user[6] * m.e13 + user[7] * m.e23 + user[8] * m.e33,
            };
            unsafe { tvg::tvg_paint_set_transform(layer_paint, &composed).into_result()? };
        }

        if let Some(user_opacity) = props.opacity {
            let composed = ((base.opacity as u16 * user_opacity as u16) / 255) as u8;
            unsafe { tvg::tvg_paint_set_opacity(layer_paint, composed).into_result()? };
        }

        if let Some(visible) = props.visible {
            unsafe { tvg::tvg_paint_set_visible(layer_paint, visible).into_result()? };
        }

        // Layer paints are id'd Scenes, so scene effects attach directly. A
        // fresh (rebuilt) scene starts with only its authored effects; ours
        // died with the old scene. Clear-then-add keeps exactly one user blur
        // regardless of rebuilds, at the cost of suppressing authored layer
        // effects while an override is active.
        match props.blur {
            Some((sigma, quality)) => unsafe {
                tvg::tvg_scene_clear_effects(layer_paint).into_result()?;
                tvg::tvg_scene_add_effect_gaussian_blur(
                    layer_paint,
                    sigma as f64,
                    0,
                    0,
                    quality as c_int,
                )
                .into_result()?;
                base.has_blur = true;
            },
            None => {
                // Remove a previously applied blur when the scene wasn't rebuilt.
                if base.has_blur {
                    unsafe { tvg::tvg_scene_clear_effects(layer_paint).into_result()? };
                    base.has_blur = false;
                }
            }
        }

        match props.clip.as_ref() {
            Some(region) => unsafe {
                tvg::tvg_paint_set_clip(layer_paint, make_clip_shape(region)).into_result()?;
                base.has_clip = true;
            },
            None => {
                // Remove a previously applied clip when the scene wasn't rebuilt.
                if base.has_clip {
                    unsafe { tvg::tvg_paint_set_clip(layer_paint, ptr::null_mut()).into_result()? };
                    base.has_clip = false;
                }
            }
        }

        Ok(())
    }

    fn get_layer_prop(&self, layer_name: &str) -> Result<Option<([f32; 9], u8)>, TvgError> {
        // Pristine animated values were cached before any user compose.
        if let Some(base) = self.layer_base.get(layer_name) {
            let m = &base.matrix;
            return Ok(Some((
                [m.e11, m.e12, m.e13, m.e21, m.e22, m.e23, m.e31, m.e32, m.e33],
                base.opacity,
            )));
        }

        let layer_id = self.layer_id_map.get_or_insert(layer_name)?;
        let layer_paint = unsafe { tvg::tvg_picture_get_paint(self.raw_paint, layer_id) };
        if layer_paint.is_null() {
            return Ok(None);
        }

        let mut m = tvg::Tvg_Matrix {
            e11: 1.0,
            e12: 0.0,
            e13: 0.0,
            e21: 0.0,
            e22: 1.0,
            e23: 0.0,
            e31: 0.0,
            e32: 0.0,
            e33: 1.0,
        };
        let mut opacity: u8 = 255;
        unsafe {
            tvg::tvg_paint_get_transform(layer_paint, &mut m).into_result()?;
            tvg::tvg_paint_get_opacity(layer_paint, &mut opacity).into_result()?;
        }
        Ok(Some((
            [m.e11, m.e12, m.e13, m.e21, m.e22, m.e23, m.e31, m.e32, m.e33],
            opacity,
        )))
    }

    // ── Markers & Segments ───────────────────────────────────────────────

    fn markers(&self) -> &[Marker] {
        &self.markers
    }

    fn set_segment(&mut self, segment: Option<Segment>) {
        self.segment = segment;
    }

    fn segment(&self) -> Result<Segment, TvgError> {
        match self.segment {
            Some(seg) => Ok(seg),
            None => Ok(Segment {
                start: 0.0,
                end: self.total_frames - 1.0,
            }),
        }
    }
}

impl Drop for TvgAnimation {
    fn drop(&mut self) {
        unsafe {
            tvg::tvg_paint_unref(self.raw_scene, true);
            tvg::tvg_animation_del(self.raw_animation);
        };
    }
}

pub struct TvgShape {
    raw_shape: tvg::Tvg_Paint,
}

impl Default for TvgShape {
    fn default() -> Self {
        Self {
            raw_shape: unsafe { tvg::tvg_shape_new() },
        }
    }
}

impl Shape for TvgShape {
    type Error = TvgError;

    fn fill(&mut self, color: Rgba) -> Result<(), TvgError> {
        unsafe {
            tvg::tvg_shape_set_fill_color(self.raw_shape, color.r, color.g, color.b, color.a)
                .into_result()
        }
    }

    fn append_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rx: f32,
        ry: f32,
    ) -> Result<(), TvgError> {
        unsafe {
            tvg::tvg_shape_append_rect(self.raw_shape, x, y, w, h, rx, ry, true).into_result()
        }
    }

    fn reset(&mut self) -> Result<(), TvgError> {
        unsafe { tvg::tvg_shape_reset(self.raw_shape).into_result() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[cfg(all(feature = "dotlottie", feature = "audio"))]
    #[test]
    fn audio_resolver_fires_for_external_audio() {
        use std::ffi::c_void;
        use std::sync::Mutex;

        static EVENTS: Mutex<Vec<(bool, bool, String)>> = Mutex::new(Vec::new());

        unsafe extern "C" fn cb(info: *const tvg::Tvg_Audio_Info, _data: *mut c_void) {
            if info.is_null() {
                return;
            }
            let i = unsafe { &*info };
            let src = if i.src.is_null() {
                String::new()
            } else {
                unsafe { CStr::from_ptr(i.src) }
                    .to_string_lossy()
                    .into_owned()
            };
            EVENTS.lock().unwrap().push((i.active, i.embedded, src));
        }

        let data =
            include_bytes!("../../assets/animations/dotlottie/v2/happy_birthday_audio.lottie");
        let mgr = crate::DotLottieManager::new(data).unwrap();
        let json = mgr.get_active_animation().unwrap();
        let cjson = CString::new(json).unwrap();

        let mut renderer = TvgRenderer::new(0);
        renderer.create_sw_canvas().unwrap();
        let canvas = renderer.raw_canvas.unwrap();
        let mut buf = vec![0u32; 128 * 128];
        unsafe {
            tvg::tvg_swcanvas_set_target(
                canvas,
                buf.as_mut_ptr(),
                128,
                128,
                128,
                tvg::Tvg_Colorspace_TVG_COLORSPACE_ABGR8888,
            );
        }

        let mut anim = TvgAnimation::default();
        anim.load_data(&cjson, c"lottie+json").unwrap();
        let total = anim.total_frames;

        unsafe {
            tvg::tvg_lottie_animation_set_audio_resolver(
                anim.raw_animation,
                Some(cb),
                std::ptr::null_mut(),
            );
            // The picture is parented to the wrapper scene now; add the scene.
            tvg::tvg_canvas_add(canvas, anim.raw_scene);
        }

        let mut f = 0.0;
        while f < total {
            unsafe {
                tvg::tvg_animation_set_frame(anim.raw_animation, f);
                tvg::tvg_canvas_update(canvas);
                tvg::tvg_canvas_draw(canvas, true);
                tvg::tvg_canvas_sync(canvas);
            }
            f += 2.0;
        }

        let events = EVENTS.lock().unwrap();
        assert!(
            events.iter().any(|(active, _, _)| *active),
            "expected at least one active audio event"
        );
        assert!(
            events
                .iter()
                .any(|(_, embedded, src)| !*embedded && src.ends_with(".mp3")),
            "expected an external .mp3 audio src, got {events:?}"
        );
    }

    #[test]
    fn test_tvg_renderer_no_deadlock() {
        const THREAD_COUNT: usize = 10;
        let barrier = Arc::new(Barrier::new(THREAD_COUNT));
        let mut handles = vec![];

        for _ in 0..THREAD_COUNT {
            let barrier_clone = Arc::clone(&barrier);
            let handle = thread::spawn(move || {
                barrier_clone.wait();

                let renderer = TvgRenderer::new(0);
                drop(renderer);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    /// Load test.json (1500×1500, layers: "B" solid, "R"/"E" shapes) into a fresh TvgAnimation.
    fn load_test_animation() -> (TvgRenderer, TvgAnimation) {
        let renderer = TvgRenderer::new(0);
        let mut animation = TvgAnimation::default();
        let data = CString::new(include_str!("../../assets/animations/lottie/test.json")).unwrap();
        animation.load_data(&data, c"lottie+json").unwrap();
        (renderer, animation)
    }

    #[test]
    fn test_hit_test_nonexistent_layer_returns_false() {
        let (_r, animation) = load_test_animation();
        assert!(!animation
            .hit_test(Point { x: 750.0, y: 750.0 }, "nonexistent")
            .unwrap());
    }

    #[test]
    fn test_hit_test_solid_layer_center_hit() {
        let (_r, animation) = load_test_animation();
        // "B" spans (0,0)–(1500,1500). Center point is clearly inside.
        assert!(animation
            .hit_test(Point { x: 750.0, y: 750.0 }, "B")
            .unwrap());
    }

    #[test]
    fn test_hit_test_solid_layer_origin_hit() {
        let (_r, animation) = load_test_animation();
        // OBB projection uses inclusive [0,1], so the origin corner is inside.
        assert!(animation.hit_test(Point { x: 0.0, y: 0.0 }, "B").unwrap());
    }

    #[test]
    fn test_hit_test_outside_bounds_miss() {
        let (_r, animation) = load_test_animation();
        assert!(!animation
            .hit_test(
                Point {
                    x: 2000.0,
                    y: 2000.0
                },
                "B"
            )
            .unwrap());
    }

    #[test]
    fn test_hit_test_negative_coords_miss() {
        let (_r, animation) = load_test_animation();
        assert!(!animation
            .hit_test(Point { x: -10.0, y: -10.0 }, "B")
            .unwrap());
    }

    #[test]
    fn test_hit_test_shape_layer_inside_obb() {
        let (_r, animation) = load_test_animation();
        // "E" has OBB (560,404)–(940,784). A centred point should hit.
        assert!(animation
            .hit_test(Point { x: 750.0, y: 600.0 }, "E")
            .unwrap());
    }

    #[test]
    fn test_hit_test_shape_layer_outside_obb() {
        let (_r, animation) = load_test_animation();
        // "R" has OBB (560,784)–(940,1122). (750,750) is above its top edge.
        assert!(!animation
            .hit_test(Point { x: 750.0, y: 750.0 }, "R")
            .unwrap());
    }
}
