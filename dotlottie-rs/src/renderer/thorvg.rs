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
    Animation, AssetResolver, ColorSpace, Drawable, GlContext, GlDisplay, GlSurface, Marker, Point,
    Renderer, Rgba, Segment, Shape, WgpuDevice, WgpuInstance, WgpuTarget, WgpuTargetType,
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
                Drawable::Animation(animation) => animation.raw_paint,
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
                Drawable::Animation(animation) => animation.raw_paint,
                Drawable::Shape(shape) => shape.raw_shape,
            };
            let at_paint = match at {
                Drawable::Animation(animation) => animation.raw_paint,
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

pub struct TvgAnimation {
    raw_animation: tvg::Tvg_Animation,
    raw_paint: tvg::Tvg_Paint,
    data: Option<CString>,
    segment: Option<Segment>,
    markers: Vec<Marker>,
    total_frames: f32,
    duration: f32,
    layer_id_map: LayerIdMap,
    // Boxed for a stable address to pass as the callback's user data.
    #[cfg(feature = "audio")]
    audio_resolver: Option<Box<AudioResolver>>,
    // Boxed for a stable address; installed pre-load and frozen by ThorVG.
    asset_resolver: Option<Box<AssetResolverState>>,
}

struct AssetResolverState {
    resolver: AssetResolver,
    /// (fName, fPath) pairs from the animation's fonts.list.
    fonts: Vec<(String, String)>,
    failed_fonts: Vec<String>,
}

unsafe extern "C" fn asset_resolver_trampoline(
    paint: tvg::Tvg_Paint,
    src: *const c_char,
    data: *mut std::ffi::c_void,
) -> bool {
    if paint.is_null() || src.is_null() || data.is_null() {
        return false;
    }
    let state = unsafe { &mut *(data as *mut AssetResolverState) };
    let Ok(src) = unsafe { CStr::from_ptr(src) }.to_str() else {
        return false;
    };
    let mut ty: tvg::Tvg_Type = 0;
    unsafe { tvg::tvg_paint_get_type(paint, &mut ty) };

    if ty == tvg::Tvg_Type_TVG_TYPE_PICTURE {
        let Some(bytes) = (state.resolver)(src).filter(|b| !b.is_empty()) else {
            return false;
        };
        unsafe {
            tvg::tvg_picture_load_data(
                paint,
                bytes.as_ptr() as *const c_char,
                bytes.len() as u32,
                c"".as_ptr(),
                ptr::null(),
                true,
            )
        }
        .into_result()
        .is_ok()
    } else if ty == tvg::Tvg_Type_TVG_TYPE_TEXT {
        if state.failed_fonts.iter().any(|f| f == src) {
            return false;
        }
        let name = src
            .strip_prefix("name:")
            .map(str::to_owned)
            .or_else(|| {
                state
                    .fonts
                    .iter()
                    .find(|(_, path)| path == src)
                    .map(|(name, _)| name.clone())
            })
            .unwrap_or_else(|| src.to_owned());
        let ok = (|| {
            let bytes = (state.resolver)(src).filter(|b| !b.is_empty())?;
            let cname = CString::new(name).ok()?;
            unsafe {
                tvg::tvg_font_load_data(
                    cname.as_ptr(),
                    bytes.as_ptr() as *const c_char,
                    bytes.len() as u32,
                    c"ttf".as_ptr(),
                    true,
                )
                .into_result()
                .ok()?;
                tvg::tvg_text_set_font(paint, cname.as_ptr())
                    .into_result()
                    .ok()?;
            }
            Some(())
        })()
        .is_some();
        if !ok {
            state.failed_fonts.push(src.to_owned());
        }
        ok
    } else {
        false
    }
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

        Self {
            raw_animation,
            raw_paint,
            data: None,
            segment: None,
            markers: Vec::new(),
            total_frames: 0.0,
            duration: 0.0,
            layer_id_map: LayerIdMap::new(),
            #[cfg(feature = "audio")]
            audio_resolver: None,
            asset_resolver: None,
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

    fn install_asset_resolver(
        &mut self,
        resolver: AssetResolver,
        fonts: Vec<(String, String)>,
    ) -> Result<(), TvgError> {
        let mut boxed = Box::new(AssetResolverState {
            resolver,
            fonts,
            failed_fonts: Vec::new(),
        });
        let data = (&mut *boxed) as *mut AssetResolverState as *mut std::ffi::c_void;
        unsafe {
            tvg::tvg_picture_set_asset_resolver(
                self.raw_paint,
                Some(asset_resolver_trampoline),
                data,
            )
        }
        .into_result()?;
        // Store only after success; a loaded picture keeps the old pointer.
        self.asset_resolver = Some(boxed);
        Ok(())
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

    #[test]
    fn asset_resolver_supplies_image_bytes() {
        use std::sync::Mutex;

        static RED_PNG: &[u8] = include_bytes!("../../assets/images/red.png");
        static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

        const JSON: &str = r#"{"v":"5.7.4","fr":30,"ip":0,"op":10,"w":100,"h":100,
        "assets":[{"id":"img0","w":16,"h":16,"u":"images/","p":"img_0.png","e":0}],
        "layers":[{"ddd":0,"ind":1,"ty":2,"nm":"i0","refId":"img0","ks":{"o":{"a":0,"k":100},"r":{"a":0,"k":0},"p":{"a":0,"k":[50,50,0]},"a":{"a":0,"k":[0,0,0]},"s":{"a":0,"k":[100,100,100]}},"ip":0,"op":10,"st":0}]}"#;

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
        let resolver: AssetResolver = Box::new(|src: &str| {
            SEEN.lock().unwrap().push(src.to_string());
            src.ends_with(".png").then(|| RED_PNG.to_vec())
        });
        anim.install_asset_resolver(resolver, Vec::new()).unwrap();

        let cjson = CString::new(JSON).unwrap();
        anim.load_data(&cjson, c"lottie+json").unwrap();

        unsafe {
            tvg::tvg_canvas_add(canvas, anim.raw_paint);
            tvg::tvg_animation_set_frame(anim.raw_animation, 0.0);
            tvg::tvg_canvas_update(canvas);
            tvg::tvg_canvas_draw(canvas, true);
            tvg::tvg_canvas_sync(canvas);
        }

        assert_eq!(SEEN.lock().unwrap().as_slice(), ["/images/img_0.png"]);
        assert!(
            buf.iter().any(|&px| px != 0),
            "resolver-supplied image should produce pixels"
        );
    }

    #[cfg(feature = "tvg-ttf")]
    #[test]
    fn asset_resolver_registers_font_under_fname_and_memoizes_failures() {
        use std::sync::Mutex;
        static CALLS: Mutex<Vec<String>> = Mutex::new(Vec::new());

        const FONT_JSON: &str = r#"{"v":"5.7.4","fr":30,"ip":0,"op":10,"w":100,"h":100,"assets":[],
        "fonts":{"list":[{"fName":"ResolverFontA","fFamily":"My","fStyle":"Regular","fPath":"/f/ResolverFontA.ttf","origin":3,"ascent":75}]},
        "layers":[{"ddd":0,"ind":1,"ty":5,"nm":"t","ks":{"o":{"a":0,"k":100},"r":{"a":0,"k":0},"p":{"a":0,"k":[10,50,0]},"a":{"a":0,"k":[0,0,0]},"s":{"a":0,"k":[100,100,100]}},"t":{"d":{"k":[{"s":{"s":24,"f":"ResolverFontA","t":"Hi","j":0,"tr":0,"lh":29,"ls":0,"fc":[0,0,0]},"t":0}]}},"ip":0,"op":10,"st":0}]}"#;

        let run = |supply: bool, fonts: Vec<(String, String)>| -> usize {
            CALLS.lock().unwrap().clear();
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
            let resolver: AssetResolver = Box::new(move |src: &str| {
                CALLS.lock().unwrap().push(src.to_string());
                supply.then(|| fallback_font::font().1)
            });
            anim.install_asset_resolver(resolver, fonts).unwrap();
            let cjson = CString::new(FONT_JSON).unwrap();
            anim.load_data(&cjson, c"lottie+json").unwrap();
            unsafe {
                tvg::tvg_canvas_add(canvas, anim.raw_paint);
            }
            for f in 0..4 {
                unsafe {
                    tvg::tvg_animation_set_frame(anim.raw_animation, f as f32);
                    tvg::tvg_canvas_update(canvas);
                    tvg::tvg_canvas_draw(canvas, true);
                    tvg::tvg_canvas_sync(canvas);
                }
            }
            CALLS.lock().unwrap().len()
        };

        let fonts = vec![(
            "ResolverFontA".to_string(),
            "/f/ResolverFontA.ttf".to_string(),
        )];
        assert_eq!(run(true, fonts), 1, "font resolved once under fName");
        assert_eq!(run(false, Vec::new()), 1, "failed font not retried");
    }
}
