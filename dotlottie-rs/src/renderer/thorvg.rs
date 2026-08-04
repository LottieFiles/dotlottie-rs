use std::{
    cell::RefCell,
    ffi::{c_char, CStr, CString},
    ptr,
    result::Result,
};

use rustc_hash::FxHashMap;

#[cfg(feature = "tvg-ttf")]
use crate::renderer::fallback_font;

use super::{
    Animation, ClipRegion, ColorSpace, Drawable, GlContext, GlDisplay, GlSurface, Marker,
    NodeProps, Point, Renderer, Rgba, Segment, Shape, SpotMask, WgpuDevice, WgpuInstance,
    WgpuTarget, WgpuTargetType,
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

/// Pristine animated values of a layer paint, read before user props are composed on
/// top. Recaptured when the paint pointer changes (scene rebuild).
struct NodeBase {
    paint: tvg::Tvg_Paint,
    matrix: tvg::Tvg_Matrix,
    opacity: u8,
    /// Comp-space center of the animated bounds (auto anchor).
    anchor: (f32, f32),
    /// Whether user effects / mask / clip are attached to this (still-live) paint,
    /// so a props change back to `None` clears exactly once.
    has_fx: bool,
    has_spot: bool,
    has_clip: bool,
}

pub struct TvgAnimation {
    raw_animation: tvg::Tvg_Animation,
    raw_paint: tvg::Tvg_Paint,
    // Wrapper scene holding the picture — the `@stage` node. Stage-level effects,
    // masks, clips, and duplicates attach here. Owns one ref so the pointer
    // survives canvas clears.
    raw_scene: tvg::Tvg_Paint,
    data: Option<CString>,
    segment: Option<Segment>,
    markers: Vec<Marker>,
    total_frames: f32,
    duration: f32,
    layer_id_map: LayerIdMap,
    node_base: FxHashMap<String, NodeBase>,
    // Scene-owned duplicates; stable pointers, canvas-space transforms.
    user_nodes: FxHashMap<String, tvg::Tvg_Paint>,
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
            data: None,
            segment: None,
            markers: Vec::new(),
            total_frames: 0.0,
            duration: 0.0,
            layer_id_map: LayerIdMap::new(),
            node_base: FxHashMap::default(),
            user_nodes: FxHashMap::default(),
            #[cfg(feature = "audio")]
            audio_resolver: None,
        }
    }
}

/// Fresh clipper shape for `tvg_paint_set_clip`; ThorVG refs it on attach and frees it
/// on replace/clear, so the caller hands it over untouched.
fn make_clip_shape(region: &ClipRegion) -> tvg::Tvg_Paint {
    unsafe {
        let shape = tvg::tvg_shape_new();
        match *region {
            ClipRegion::Rect { x, y, w, h, r } => {
                let _ = tvg::tvg_shape_append_rect(shape, x, y, w, h, r, r, true);
            }
            ClipRegion::Circle { cx, cy, r } => {
                let _ = tvg::tvg_shape_append_circle(shape, cx, cy, r, r, true);
            }
        }
        shape
    }
}

/// Circle whose alpha ramps from opaque to transparent over the feathered rim; used as
/// the alpha-mask target.
fn make_spot_shape(mask: &SpotMask) -> tvg::Tvg_Paint {
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
                mask.radius.max(1e-3),
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

/// Row-major 3×3 with translation in [2] and [5], matching `Tvg_Matrix` e13/e23.
fn mat_mul(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    let mut out = [0.0f32; 9];
    for row in 0..3 {
        for col in 0..3 {
            out[row * 3 + col] =
                a[row * 3] * b[col] + a[row * 3 + 1] * b[3 + col] + a[row * 3 + 2] * b[6 + col];
        }
    }
    out
}

fn to_tvg_matrix(m: &[f32; 9]) -> tvg::Tvg_Matrix {
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

fn from_tvg_matrix(m: &tvg::Tvg_Matrix) -> [f32; 9] {
    [
        m.e11, m.e12, m.e13, m.e21, m.e22, m.e23, m.e31, m.e32, m.e33,
    ]
}

/// User transform in comp space: translate, then rotate/scale about the anchor
/// (the pivot follows the translation).
fn compose_user_matrix(props: &NodeProps, anchor: (f32, f32)) -> [f32; 9] {
    let (ax, ay) = props.anchor.unwrap_or(anchor);
    let x = props.x.unwrap_or(0.0);
    let y = props.y.unwrap_or(0.0);
    let sx = props.scale_x.unwrap_or(1.0);
    let sy = props.scale_y.unwrap_or(1.0);
    let rad = props.rotate.unwrap_or(0.0).to_radians();
    let (sin, cos) = rad.sin_cos();

    // T(x + ax, y + ay) · R · S · T(-ax, -ay)
    let rs = [
        cos * sx,
        -sin * sy,
        0.0,
        sin * sx,
        cos * sy,
        0.0,
        0.0,
        0.0,
        1.0,
    ];
    let mut out = rs;
    out[2] = rs[0] * -ax + rs[1] * -ay + x + ax;
    out[5] = rs[3] * -ax + rs[4] * -ay + y + ay;
    out
}

impl TvgAnimation {
    /// Capture-and-compose core shared by layers, duplicates, and `@stage`.
    /// `space` maps the paint's owner coordinate space back from canvas space
    /// (identity for stage/duplicates, canvas→comp for authored layers).
    fn apply_to_paint(
        &mut self,
        name: &str,
        paint: tvg::Tvg_Paint,
        props: &NodeProps,
        space: &[f32; 9],
    ) -> Result<(), TvgError> {
        let needs_capture = match self.node_base.get(name) {
            Some(base) => base.paint != paint,
            None => true,
        };

        if needs_capture {
            let mut matrix = to_tvg_matrix(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
            let mut opacity: u8 = 255;
            let (mut bx, mut by, mut bw, mut bh) = (0f32, 0f32, 0f32, 0f32);
            unsafe {
                tvg::tvg_paint_get_transform(paint, &mut matrix).into_result()?;
                tvg::tvg_paint_get_opacity(paint, &mut opacity).into_result()?;
                let _ = tvg::tvg_paint_get_aabb(paint, &mut bx, &mut by, &mut bw, &mut bh);
            }
            let cx = bx + bw * 0.5;
            let cy = by + bh * 0.5;
            let anchor = (
                space[0] * cx + space[1] * cy + space[2],
                space[3] * cx + space[4] * cy + space[5],
            );
            self.node_base.insert(
                name.to_owned(),
                NodeBase {
                    paint,
                    matrix,
                    opacity,
                    anchor,
                    has_fx: false,
                    has_spot: false,
                    has_clip: false,
                },
            );
        }

        let base = self.node_base.get_mut(name).unwrap();

        unsafe {
            if props.restore {
                tvg::tvg_paint_set_transform(paint, &base.matrix).into_result()?;
                tvg::tvg_paint_set_opacity(paint, base.opacity).into_result()?;
                tvg::tvg_paint_set_visible(paint, true).into_result()?;
                if base.has_fx {
                    let _ = tvg::tvg_scene_clear_effects(paint);
                    let _ = tvg::tvg_paint_set_blend_method(
                        paint,
                        tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_NORMAL,
                    );
                }
                if base.has_spot {
                    let _ = tvg::tvg_paint_set_mask_method(
                        paint,
                        std::ptr::null_mut(),
                        tvg::Tvg_Mask_Method_TVG_MASK_METHOD_NONE,
                    );
                }
                if base.has_clip {
                    let _ = tvg::tvg_paint_set_clip(paint, std::ptr::null_mut());
                }
                base.has_fx = false;
                base.has_spot = false;
                base.has_clip = false;
                return Ok(());
            }

            let user = compose_user_matrix(props, base.anchor);
            let composed = mat_mul(&user, &from_tvg_matrix(&base.matrix));
            tvg::tvg_paint_set_transform(paint, &to_tvg_matrix(&composed)).into_result()?;

            let opacity =
                (base.opacity as f32 * props.opacity.unwrap_or(1.0).clamp(0.0, 1.0)).round() as u8;
            tvg::tvg_paint_set_opacity(paint, opacity).into_result()?;

            if let Some(visible) = props.visible {
                tvg::tvg_paint_set_visible(paint, visible).into_result()?;
            }

            // Effects: clear-then-add keeps exactly one user stack per apply; while an
            // override is active, authored effects on this paint are suppressed.
            let wants_fx = props.blur.is_some_and(|s| s > 0.05)
                || props.tint.is_some_and(|t| t.intensity > 0.001);
            if wants_fx || base.has_fx {
                tvg::tvg_scene_clear_effects(paint).into_result()?;
            }
            if let Some(sigma) = props.blur {
                if sigma > 0.05 {
                    tvg::tvg_scene_add_effect_gaussian_blur(paint, sigma as f64, 0, 0, 60)
                        .into_result()?;
                }
            }
            if let Some(t) = props.tint {
                if t.intensity > 0.001 {
                    tvg::tvg_scene_add_effect_tint(
                        paint,
                        t.black[0] as std::ffi::c_int,
                        t.black[1] as std::ffi::c_int,
                        t.black[2] as std::ffi::c_int,
                        t.white[0] as std::ffi::c_int,
                        t.white[1] as std::ffi::c_int,
                        t.white[2] as std::ffi::c_int,
                        (t.intensity.clamp(0.0, 1.0) * 100.0) as f64,
                    )
                    .into_result()?;
                }
            }
            base.has_fx = wants_fx;

            if let Some(mode) = props.blend_mode {
                tvg::tvg_paint_set_blend_method(paint, mode as tvg::Tvg_Blend_Method)
                    .into_result()?;
            }

            match props.spot {
                Some(spot) if spot.radius > 0.0 => {
                    let shape = make_spot_shape(&spot);
                    tvg::tvg_paint_set_mask_method(
                        paint,
                        shape,
                        tvg::Tvg_Mask_Method_TVG_MASK_METHOD_ALPHA,
                    )
                    .into_result()?;
                    base.has_spot = true;
                }
                _ if base.has_spot => {
                    let _ = tvg::tvg_paint_set_mask_method(
                        paint,
                        std::ptr::null_mut(),
                        tvg::Tvg_Mask_Method_TVG_MASK_METHOD_NONE,
                    );
                    base.has_spot = false;
                }
                _ => {}
            }

            match props.clip {
                Some(region) => {
                    let shape = make_clip_shape(&region);
                    tvg::tvg_paint_set_clip(paint, shape).into_result()?;
                    base.has_clip = true;
                }
                None if base.has_clip => {
                    let _ = tvg::tvg_paint_set_clip(paint, std::ptr::null_mut());
                    base.has_clip = false;
                }
                None => {}
            }
        }
        Ok(())
    }

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

    fn apply_node_props(
        &mut self,
        name: &str,
        props: &NodeProps,
        canvas_to_comp: &[f32; 9],
    ) -> Result<(), TvgError> {
        const IDENT: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

        // Stable-owner nodes: the wrapper scene and duplicates live in canvas space.
        let (paint, space) = if name == "@stage" {
            (self.raw_scene, &IDENT)
        } else if let Some(&paint) = self.user_nodes.get(name) {
            (paint, &IDENT)
        } else {
            let id = self.layer_id_map.get_or_insert(name)?;
            let paint = unsafe { tvg::tvg_picture_get_paint(self.raw_paint, id) };
            if paint.is_null() {
                // Outside its in/out range — retained, re-attaches when it returns.
                self.node_base.remove(name);
                return Ok(());
            }
            (paint, canvas_to_comp)
        };

        self.apply_to_paint(name, paint, props, space)
    }

    fn duplicate_node(
        &mut self,
        source: &str,
        as_name: &str,
        comp_to_canvas: &[f32; 9],
    ) -> Result<(), TvgError> {
        if as_name == "@stage" || self.user_nodes.contains_key(as_name) {
            return Err(TvgError::InvalidArgument);
        }
        let id = self.layer_id_map.get_or_insert(source)?;
        let src = unsafe { tvg::tvg_picture_get_paint(self.raw_paint, id) };
        if src.is_null() {
            return Err(TvgError::InsufficientCondition);
        }
        unsafe {
            let dup = tvg::tvg_paint_duplicate(src);
            if dup.is_null() {
                return Err(TvgError::FailedAllocation);
            }
            // Bake the picture chain so the copy renders identically from the stage.
            let mut local = to_tvg_matrix(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
            tvg::tvg_paint_get_transform(dup, &mut local).into_result()?;
            let baked = mat_mul(comp_to_canvas, &from_tvg_matrix(&local));
            tvg::tvg_paint_set_transform(dup, &to_tvg_matrix(&baked)).into_result()?;
            tvg::tvg_scene_add(self.raw_scene, dup).into_result()?;
            self.user_nodes.insert(as_name.to_owned(), dup);
        }
        Ok(())
    }

    fn remove_node(&mut self, name: &str) -> Result<(), TvgError> {
        if let Some(paint) = self.user_nodes.remove(name) {
            self.node_base.remove(name);
            unsafe { tvg::tvg_scene_remove(self.raw_scene, paint).into_result()? };
        }
        Ok(())
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
        unsafe { tvg::tvg_animation_set_frame(self.raw_animation, frame_no).into_result() }
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
        unsafe { tvg::tvg_lottie_animation_apply_slot(self.raw_animation, slot_code) }.into_result()
    }

    fn del_slot(&mut self, slot_code: u32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_lottie_animation_del_slot(self.raw_animation, slot_code) }.into_result()
    }

    fn set_quality(&mut self, quality: u8) -> Result<(), TvgError> {
        unsafe { tvg::tvg_lottie_animation_set_quality(self.raw_animation, quality).into_result() }
    }

    fn tween_to(&mut self, to: f32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_lottie_animation_tween_to(self.raw_animation, to).into_result() }
    }

    fn tween_go(&mut self, progress: f32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_lottie_animation_tween_go(self.raw_animation, progress).into_result() }
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
            tvg::tvg_canvas_add(canvas, anim.raw_paint);
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

    const IDENT: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

    fn read_paint_state(animation: &TvgAnimation, name: &str) -> ([f32; 9], u8) {
        let id = animation.layer_id_map.get_or_insert(name).unwrap();
        let paint = unsafe { tvg::tvg_picture_get_paint(animation.raw_paint, id) };
        assert!(!paint.is_null());
        let mut m = to_tvg_matrix(&IDENT);
        let mut opacity: u8 = 0;
        unsafe {
            tvg::tvg_paint_get_transform(paint, &mut m)
                .into_result()
                .unwrap();
            tvg::tvg_paint_get_opacity(paint, &mut opacity)
                .into_result()
                .unwrap();
        }
        (from_tvg_matrix(&m), opacity)
    }

    #[test]
    fn node_props_compose_idempotently_and_restore() {
        let (_r, mut animation) = load_test_animation();
        let (base_m, base_o) = read_paint_state(&animation, "B");

        let props = NodeProps {
            rotate: Some(90.0),
            x: Some(10.0),
            ..Default::default()
        };
        animation.apply_node_props("B", &props, &IDENT).unwrap();
        let (m1, _) = read_paint_state(&animation, "B");
        assert_ne!(m1, base_m, "override must change the transform");

        // Re-applying within the same generation must not accumulate.
        animation.apply_node_props("B", &props, &IDENT).unwrap();
        let (m2, _) = read_paint_state(&animation, "B");
        assert_eq!(m1, m2);

        let restore = NodeProps {
            restore: true,
            ..Default::default()
        };
        animation.apply_node_props("B", &restore, &IDENT).unwrap();
        let (m3, o3) = read_paint_state(&animation, "B");
        assert_eq!(m3, base_m);
        assert_eq!(o3, base_o);
    }

    #[test]
    fn node_props_opacity_multiplies_base() {
        let (_r, mut animation) = load_test_animation();
        let (_, base_o) = read_paint_state(&animation, "B");

        let props = NodeProps {
            opacity: Some(0.5),
            ..Default::default()
        };
        animation.apply_node_props("B", &props, &IDENT).unwrap();
        let (_, o) = read_paint_state(&animation, "B");
        let expected = (base_o as f32 * 0.5).round() as u8;
        assert_eq!(o, expected);
    }

    #[test]
    fn node_props_unknown_layer_is_soft() {
        let (_r, mut animation) = load_test_animation();
        let props = NodeProps {
            rotate: Some(45.0),
            ..Default::default()
        };
        animation
            .apply_node_props("does-not-exist", &props, &IDENT)
            .unwrap();
    }
}
