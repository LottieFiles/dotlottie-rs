use std::{
    error::Error,
    ffi::{c_char, CStr, CString},
    fmt, ptr,
    result::Result,
};

#[cfg(feature = "tvg-ttf")]
use crate::lottie_renderer::fallback_font;

use super::{
    Animation, ColorSpace, Drawable, GlContext, GlDisplay, GlSurface, Marker, Renderer, Rgba,
    Segment, Shape, WgpuDevice, WgpuInstance, WgpuTarget, WgpuTargetType,
};

#[expect(non_upper_case_globals)]
#[allow(non_snake_case)]
#[expect(non_camel_case_types)]
#[expect(dead_code)]
mod tvg {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Debug)]
pub enum TvgError {
    InvalidArgument,
    InsufficientCondition,
    FailedAllocation,
    MemoryCorruption,
    NotSupported,
    Unknown,
}

impl fmt::Display for TvgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for TvgError {}

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

pub struct TvgAnimation {
    raw_animation: tvg::Tvg_Animation,
    raw_paint: tvg::Tvg_Paint,
    data: Option<CString>,
    segment: Option<Segment>,
    markers: Vec<Marker>,
    total_frames: f32,
    duration: f32,
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
        unsafe {
            let mut obb: [tvg::Tvg_Point; 4] = [tvg::Tvg_Point { x: 0.0, y: 0.0 }; 4];
            let paint = self.raw_paint;
            let layer_name_cstr = CString::new(layer_name).expect("Failed to create CString");
            let layer_id = tvg::tvg_accessor_generate_id(layer_name_cstr.as_ptr());
            let layer_paint = tvg::tvg_picture_get_paint(paint, layer_id);

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
                Ok(())
            }
            Err(e) => {
                self.data = None;
                self.markers.clear();
                self.total_frames = 0.0;
                self.duration = 0.0;
                Err(e)
            }
        }
    }

    fn intersect(&self, _x: f32, _y: f32, _layer_name: &str) -> Result<bool, TvgError> {
        if let Some(obb) = self.get_layer_obb(_layer_name)? {
            let e1 = tvg::Tvg_Point {
                x: obb[1].x - obb[0].x,
                y: obb[1].y - obb[0].y,
            };
            let e2 = tvg::Tvg_Point {
                x: obb[3].x - obb[0].x,
                y: obb[3].y - obb[0].y,
            };
            let o = tvg::Tvg_Point {
                x: _x - obb[0].x,
                y: _y - obb[0].y,
            };
            let u = (o.x * e1.x + o.y * e1.y) / (e1.x * e1.x + e1.y * e1.y);
            let v = (o.x * e2.x + o.y * e2.y) / (e2.x * e2.x + e2.y * e2.y);

            // Check if point is inside the OBB
            Ok((0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v))
        } else {
            Ok(false)
        }
    }

    fn get_layer_bounds(&self, _layer_name: &str) -> Result<[f32; 8], TvgError> {
        if let Some(obb) = self.get_layer_obb(_layer_name)? {
            // Return the 8 points out of obb
            let mut point_vec: Vec<f32> = Vec::with_capacity(8);

            for item in &obb {
                point_vec.push(item.x);
                point_vec.push(item.y);
            }

            Ok([
                point_vec[0],
                point_vec[1],
                point_vec[2],
                point_vec[3],
                point_vec[4],
                point_vec[5],
                point_vec[6],
                point_vec[7],
            ])
        } else {
            Err(TvgError::Unknown)
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

    fn tween(&mut self, from: f32, to: f32, progress: f32) -> Result<(), TvgError> {
        unsafe {
            tvg::tvg_lottie_animation_tween(self.raw_animation, from, to, progress);
        }
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
}
