#[cfg(feature = "tvg-v1")]
use crate::time::Instant;

use std::{
    error::Error,
    ffi::{c_char, CString},
    fmt, ptr,
    result::Result,
};

use super::{Animation, ColorSpace, Drawable, Renderer, Shape};

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

pub enum TvgEngine {
    TvgEngineSw,
    TvgEngineGl,
}

#[cfg(feature = "tvg-v0")]
impl From<TvgEngine> for tvg::Tvg_Engine {
    fn from(engine_method: TvgEngine) -> Self {
        match engine_method {
            TvgEngine::TvgEngineSw => tvg::Tvg_Engine_TVG_ENGINE_SW,
            TvgEngine::TvgEngineGl => tvg::Tvg_Engine_TVG_ENGINE_GL,
        }
    }
}

static RENDERERS_COUNT: std::sync::Mutex<usize> = std::sync::Mutex::new(0);

pub struct TvgRenderer {
    raw_canvas: *mut tvg::Tvg_Canvas,
    #[cfg(feature = "tvg-v0")]
    engine_method: tvg::Tvg_Engine,
}

impl TvgRenderer {
    pub fn new(
        #[cfg_attr(not(feature = "tvg-v0"), allow(unused_variables))] engine_method: TvgEngine,
        threads: u32,
    ) -> Self {
        let mut count = RENDERERS_COUNT.lock().unwrap();

        #[cfg(feature = "tvg-v0")]
        let engine = engine_method.into();

        if *count == 0 {
            #[cfg(feature = "tvg-v0")]
            {
                unsafe { tvg::tvg_engine_init(engine, threads).into_result() }.unwrap();
            }

            #[cfg(feature = "tvg-v1")]
            unsafe { tvg::tvg_engine_init(threads).into_result() }.unwrap();
        }

        *count += 1;

        TvgRenderer {
            raw_canvas: unsafe { tvg::tvg_swcanvas_create() },
            #[cfg(feature = "tvg-v0")]
            engine_method: engine,
        }
    }
}

impl Renderer for TvgRenderer {
    type Animation = TvgAnimation;
    type Shape = TvgShape;
    type Error = TvgError;

    fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_canvas_set_viewport(self.raw_canvas, x, y, w, h).into_result() }
    }

    fn set_target(
        &mut self,
        buffer: &mut [u32],
        stride: u32,
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Result<(), TvgError> {
        unsafe {
            tvg::tvg_swcanvas_set_target(
                self.raw_canvas,
                buffer.as_mut_ptr(),
                stride,
                width,
                height,
                color_space.into(),
            )
            .into_result()
        }
    }

    fn clear(&self, _free: bool) -> Result<(), TvgError> {
        #[cfg(feature = "tvg-v1")]
        unsafe {
            tvg::tvg_canvas_remove(self.raw_canvas, ptr::null_mut::<tvg::Tvg_Paint>()).into_result()
        }

        #[cfg(feature = "tvg-v0")]
        unsafe {
            tvg::tvg_canvas_clear(self.raw_canvas, _free).into_result()
        }
    }

    fn push(&mut self, drawable: Drawable<Self>) -> Result<(), TvgError> {
        let raw_paint = match drawable {
            Drawable::Animation(animation) => animation.raw_paint,
            Drawable::Shape(shape) => shape.raw_shape,
        };

        unsafe { tvg::tvg_canvas_push(self.raw_canvas, raw_paint).into_result() }
    }

    fn draw(&mut self, _clear_buffer: bool) -> Result<(), TvgError> {
        #[cfg(feature = "tvg-v1")]
        unsafe {
            tvg::tvg_canvas_draw(self.raw_canvas, _clear_buffer).into_result()
        }

        #[cfg(feature = "tvg-v0")]
        unsafe {
            tvg::tvg_canvas_draw(self.raw_canvas).into_result()
        }
    }

    fn sync(&mut self) -> Result<(), TvgError> {
        unsafe { tvg::tvg_canvas_sync(self.raw_canvas).into_result() }
    }

    fn update(&mut self) -> Result<(), TvgError> {
        unsafe { tvg::tvg_canvas_update(self.raw_canvas).into_result() }
    }
}

impl Drop for TvgRenderer {
    fn drop(&mut self) {
        let mut count = RENDERERS_COUNT.lock().unwrap();

        unsafe {
            tvg::tvg_canvas_destroy(self.raw_canvas);
        }

        *count = count.checked_sub(1).unwrap();

        if *count == 0 {
            #[cfg(feature = "tvg-v0")]
            unsafe {
                tvg::tvg_engine_term(self.engine_method)
            };

            #[cfg(feature = "tvg-v1")]
            unsafe {
                tvg::tvg_engine_term()
            };
        }
    }
}

#[cfg(feature = "tvg-v1")]
struct TweenState {
    from: f32,
    to: f32,
    start_time: Option<Instant>,
    duration: Option<f32>,
    easing: Option<[f32; 4]>,
}

pub struct TvgAnimation {
    raw_animation: *mut tvg::Tvg_Animation,
    raw_paint: *mut tvg::Tvg_Paint,
    #[cfg(feature = "tvg-v1")]
    tween_state: Option<TweenState>,
    data: Option<CString>,
}

impl Default for TvgAnimation {
    fn default() -> Self {
        let raw_animation = unsafe { tvg::tvg_animation_new() };
        let raw_paint = unsafe { tvg::tvg_animation_get_picture(raw_animation) };

        Self {
            raw_animation,
            raw_paint,
            #[cfg(feature = "tvg-v1")]
            tween_state: None,
            data: None,
        }
    }
}

impl TvgAnimation {
    #[cfg(feature = "tvg-v1")]
    fn get_layer_obb(&self, layer_name: &str) -> Result<Option<[tvg::Tvg_Point; 4]>, TvgError> {
        unsafe {
            let mut obb: [tvg::Tvg_Point; 4] = [tvg::Tvg_Point { x: 0.0, y: 0.0 }; 4];
            let paint = self.raw_paint;
            let layer_name_cstr = CString::new(layer_name).expect("Failed to create CString");
            let layer_id = tvg::tvg_accessor_generate_id(layer_name_cstr.as_ptr());
            let layer_paint = tvg::tvg_picture_get_paint(paint, layer_id);

            if !layer_paint.is_null() {
                tvg::tvg_paint_get_obb(layer_paint as *mut tvg::Tvg_Paint, obb.as_mut_ptr());
                Ok(Some(obb))
            } else {
                Ok(None)
            }
        }
    }

    unsafe fn tvg_load_data_dispatch(
        raw_paint: *mut tvg::Tvg_Paint,
        data_ptr: *const c_char,
        data_len: u32,
        mimetype_ptr: *const c_char,
    ) -> Result<(), TvgError> {
        #[cfg(feature = "tvg-v1")]
        {
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

        #[cfg(feature = "tvg-v0")]
        {
            tvg::tvg_picture_load_data(raw_paint, data_ptr, data_len, mimetype_ptr, false)
                .into_result()
        }
    }
}

impl Animation for TvgAnimation {
    type Error = TvgError;

    fn load_data(&mut self, data: &str, mimetype: &str) -> Result<(), TvgError> {
        let mimetype_cstr = CString::new(mimetype).map_err(|_| TvgError::InvalidArgument)?;
        let data_cstr = CString::new(data).map_err(|_| TvgError::InvalidArgument)?;
        let data_len_u32 =
            u32::try_from(data_cstr.as_bytes().len()).map_err(|_| TvgError::InvalidArgument)?;

        let result = unsafe {
            TvgAnimation::tvg_load_data_dispatch(
                self.raw_paint,
                data_cstr.as_ptr(),
                data_len_u32,
                mimetype_cstr.as_ptr(),
            )
        };

        match result {
            Ok(()) => {
                // Keep the payload alive for ThorVG
                self.data = Some(data_cstr);
                Ok(())
            }
            Err(e) => {
                self.data = None;
                Err(e)
            }
        }
    }

    fn intersect(&self, _x: f32, _y: f32, _layer_name: &str) -> Result<bool, TvgError> {
        #[cfg(feature = "tvg-v1")]
        {
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

        #[cfg(not(feature = "tvg-v1"))]
        Err(TvgError::NotSupported)
    }

    fn get_layer_bounds(&self, _layer_name: &str) -> Result<[f32; 8], TvgError> {
        #[cfg(feature = "tvg-v1")]
        {
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

        #[cfg(not(feature = "tvg-v1"))]
        Err(TvgError::NotSupported)
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

    fn scale(&mut self, factor: f32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_paint_scale(self.raw_paint, factor).into_result() }
    }

    fn translate(&mut self, tx: f32, ty: f32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_paint_translate(self.raw_paint, tx, ty).into_result() }
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

        Ok(duration)
    }

    fn set_frame(&mut self, frame_no: f32) -> Result<(), TvgError> {
        unsafe { tvg::tvg_animation_set_frame(self.raw_animation, frame_no).into_result() }
    }

    fn get_frame(&self) -> Result<f32, TvgError> {
        let mut curr_frame: f32 = 0.0;

        unsafe {
            tvg::tvg_animation_get_frame(self.raw_animation, &mut curr_frame as *mut f32)
                .into_result()
        }?;

        Ok(curr_frame)
    }

    fn set_slots(
        &mut self,
        #[cfg_attr(not(feature = "tvg-v1"), allow(unused_variables))]
        slots: &str,
    ) -> Result<(), TvgError> {
        #[cfg(feature = "tvg-v1")]
        {
            let result = if slots.is_empty() {
                unsafe { tvg::tvg_lottie_animation_apply_slot(self.raw_animation, 0) }
            } else {
                let slots_cstr = CString::new(slots).expect("Failed to create CString");
                let slot_id = unsafe { tvg::tvg_lottie_animation_gen_slot(self.raw_animation, slots_cstr.as_ptr()) };
                if slot_id == 0 {
                    return Err(TvgError::InvalidArgument);
                }
                unsafe { tvg::tvg_lottie_animation_apply_slot(self.raw_animation, slot_id) }
            };

            result.into_result()
        }

        #[cfg(not(feature = "tvg-v1"))]
        Err(TvgError::NotSupported)
    }

    fn tween(
        &mut self,
        _to: f32,
        _duration: Option<f32>,
        _easing: Option<[f32; 4]>,
    ) -> Result<(), TvgError> {
        #[cfg(feature = "tvg-v1")]
        {
            if self.is_tweening() {
                return Err(TvgError::InvalidArgument);
            }
            if _duration.is_some() && _duration.unwrap() <= 0.0 {
                return Err(TvgError::InvalidArgument);
            }
            if _easing.is_some() && _easing.unwrap().iter().any(|&x| !(0.0..=1.0).contains(&x)) {
                return Err(TvgError::InvalidArgument);
            }

            let from = self.get_frame()?;

            self.tween_state = Some(TweenState {
                start_time: {
                    if _duration.is_some() {
                        Some(Instant::now())
                    } else {
                        None
                    }
                },
                from,
                to: _to,
                duration: _duration,
                easing: _easing,
            });

            Ok(())
        }

        #[cfg(not(feature = "tvg-v1"))]
        Err(TvgError::NotSupported)
    }

    fn is_tweening(&self) -> bool {
        #[cfg(feature = "tvg-v1")]
        return self.tween_state.is_some();

        #[cfg(not(feature = "tvg-v1"))]
        false
    }

    fn tween_stop(&mut self) -> Result<(), TvgError> {
        #[cfg(feature = "tvg-v1")]
        {
            self.tween_state = None;
            Ok(())
        }

        #[cfg(not(feature = "tvg-v1"))]
        Err(TvgError::NotSupported)
    }

    fn tween_update(&mut self, _given_progress: Option<f32>) -> Result<bool, TvgError> {
        #[cfg(feature = "tvg-v1")]
        {
            if self.tween_state.is_some() && self.tween_state.as_ref().unwrap().duration.is_none() {
                if _given_progress.is_none() {
                    return Err(TvgError::InvalidArgument);
                }

                unsafe {
                    tvg::tvg_lottie_animation_tween(
                        self.raw_animation,
                        self.tween_state.as_ref().unwrap().from,
                        self.tween_state.as_ref().unwrap().to,
                        _given_progress.unwrap(),
                    );
                };

                return Ok(true);
            }

            if let Some(tween_state) = self.tween_state.as_mut() {
                let elapsed = Instant::now().duration_since(tween_state.start_time.unwrap());
                let t = elapsed.as_secs_f32() / tween_state.duration.unwrap();
                let progress = if t >= 1.0 {
                    1.0
                } else {
                    let [x1, y1, x2, y2] = tween_state.easing.unwrap_or([0.0, 0.0, 1.0, 1.0]);
                    bezier::cubic_bezier(t, x1, y1, x2, y2)
                };

                unsafe {
                    tvg::tvg_lottie_animation_tween(
                        self.raw_animation,
                        tween_state.from,
                        tween_state.to,
                        progress,
                    );
                };

                if progress >= 1.0 {
                    let target_frame = tween_state.to;
                    self.tween_state = None;
                    self.set_frame(target_frame)?;
                    Ok(false)
                } else {
                    Ok(true)
                }
            } else {
                Ok(false)
            }
        }

        #[cfg(not(feature = "tvg-v1"))]
        Err(TvgError::NotSupported)
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
    raw_shape: *mut tvg::Tvg_Paint,
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

    fn fill(&mut self, color: (u8, u8, u8, u8)) -> Result<(), TvgError> {
        unsafe {
            tvg::tvg_shape_set_fill_color(self.raw_shape, color.0, color.1, color.2, color.3)
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
        #[cfg(feature = "tvg-v1")]
        unsafe {
            tvg::tvg_shape_append_rect(self.raw_shape, x, y, w, h, rx, ry, true).into_result()
        }

        #[cfg(feature = "tvg-v0")]
        unsafe {
            tvg::tvg_shape_append_rect(self.raw_shape, x, y, w, h, rx, ry).into_result()
        }
    }

    fn reset(&mut self) -> Result<(), TvgError> {
        unsafe { tvg::tvg_shape_reset(self.raw_shape).into_result() }
    }
}

#[cfg(feature = "tvg-v1")]
mod bezier {
    /// Computes the x-coordinate of the cubic Bézier for parameter `u`.
    /// P0 = 0, P1 = (x1, _), P2 = (x2, _), P3 = 1.
    pub(super) fn sample_curve_x(u: f32, x1: f32, x2: f32) -> f32 {
        let inv_u = 1.0 - u;
        3.0 * inv_u * inv_u * u * x1 + 3.0 * inv_u * u * u * x2 + u * u * u
    }

    /// Computes the y-coordinate of the cubic Bézier for parameter `u`.
    /// P0 = 0, P1 = (_, y1), P2 = (_, y2), P3 = 1.
    pub(super) fn sample_curve_y(u: f32, y1: f32, y2: f32) -> f32 {
        let inv_u = 1.0 - u;
        3.0 * inv_u * inv_u * u * y1 + 3.0 * inv_u * u * u * y2 + u * u * u
    }

    /// Computes the derivative dx/du for a given u.
    fn sample_curve_derivative_x(u: f32, x1: f32, x2: f32) -> f32 {
        let inv_u = 1.0 - u;
        3.0 * inv_u * inv_u * x1 + 6.0 * inv_u * u * (x2 - x1) + 3.0 * u * u * (1.0 - x2)
    }

    /// Uses binary subdivision to find a parameter u such that sample_curve_x(u) ≈ t.
    fn binary_subdivide(t: f32, x1: f32, x2: f32) -> f32 {
        let mut a = 0.0;
        let mut b = 1.0;
        let mut u = t;
        for _ in 0..10 {
            let x = sample_curve_x(u, x1, x2);
            if (x - t).abs() < 1e-6 {
                return u;
            }
            if x > t {
                b = u;
            } else {
                a = u;
            }
            u = (a + b) * 0.5;
        }
        u
    }

    /// Given a linear progress t in [0,1], uses a cubic Bézier easing function to compute
    /// an eased progress value in [0,1].
    ///
    /// The cubic Bézier is defined by:
    ///   P0 = (0, 0)
    ///   P1 = (x1, y1)
    ///   P2 = (x2, y2)
    ///   P3 = (1, 1)
    pub(super) fn cubic_bezier(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }

        // First try Newton–Raphson iteration.
        let mut u = t;
        for _ in 0..8 {
            let x = sample_curve_x(u, x1, x2);
            let dx = sample_curve_derivative_x(u, x1, x2);
            if dx.abs() < 1e-6 {
                break;
            }
            let delta = (x - t) / dx;
            u -= delta;
            if delta.abs() < 1e-6 {
                break;
            }
        }

        // Fallback to binary subdivision if necessary.
        if !(0.0..=1.0).contains(&u) {
            u = binary_subdivide(t, x1, x2);
        }
        u = u.clamp(0.0, 1.0);
        sample_curve_y(u, y1, y2)
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

                let renderer = TvgRenderer::new(TvgEngine::TvgEngineSw, 0);
                drop(renderer);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }
}
