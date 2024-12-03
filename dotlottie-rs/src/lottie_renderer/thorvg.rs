use std::{
    ffi::CString,
    ptr::{self},
};
use thiserror::Error;

use super::{Animation, ColorSpace, Drawable, Renderer, Shape};

#[expect(non_upper_case_globals)]
#[allow(non_snake_case)]
#[expect(non_camel_case_types)]
#[expect(dead_code)]
mod tvg {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Error, Debug)]
pub enum TvgError {
    #[error("Invalid argument provided in {function_name}")]
    InvalidArgument { function_name: String },

    #[error("Insufficient condition in {function_name}")]
    InsufficientCondition { function_name: String },

    #[error("Failed memory allocation in {function_name}")]
    FailedAllocation { function_name: String },

    #[error("Memory corruption detected in {function_name}")]
    MemoryCorruption { function_name: String },

    #[error("Operation not supported in {function_name}")]
    NotSupported { function_name: String },

    #[error("Unknown error occurred in {function_name}")]
    Unknown { function_name: String },
}

pub enum TvgBlendMethod {
    Normal,
    Add,
    Difference,
    Exclusion,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
}

impl From<TvgBlendMethod> for tvg::Tvg_Blend_Method {
    fn from(blend_method: TvgBlendMethod) -> Self {
        match blend_method {
            TvgBlendMethod::Normal => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_NORMAL,
            TvgBlendMethod::Add => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_ADD,
            TvgBlendMethod::Difference => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_DIFFERENCE,
            TvgBlendMethod::Exclusion => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_EXCLUSION,
            TvgBlendMethod::Multiply => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_MULTIPLY,
            TvgBlendMethod::Screen => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_SCREEN,
            TvgBlendMethod::Overlay => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_OVERLAY,
            TvgBlendMethod::Darken => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_DARKEN,
            TvgBlendMethod::Lighten => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_LIGHTEN,
            TvgBlendMethod::ColorDodge => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_COLORDODGE,
            TvgBlendMethod::ColorBurn => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_COLORBURN,
            TvgBlendMethod::HardLight => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_HARDLIGHT,
            TvgBlendMethod::SoftLight => tvg::Tvg_Blend_Method_TVG_BLEND_METHOD_SOFTLIGHT,
        }
    }
}

pub struct TvgMatrix {
    inner: tvg::Tvg_Matrix,
}

impl TvgMatrix {
    pub fn new(transform: [f32; 9]) -> Self {
        Self {
            inner: tvg::Tvg_Matrix {
                e11: transform[0],
                e12: transform[1],
                e13: transform[2],
                e21: transform[3],
                e22: transform[4],
                e23: transform[5],
                e31: transform[6],
                e32: transform[7],
                e33: transform[8],
            },
        }
    }
}

impl From<TvgMatrix> for tvg::Tvg_Matrix {
    fn from(matrix: TvgMatrix) -> Self {
        matrix.inner
    }
}

pub enum TvgEngine {
    TvgEngineSw,
    TvgEngineGl,
}

fn convert_tvg_result(result: tvg::Tvg_Result, function_name: &str) -> Result<(), TvgError> {
    let func_name = function_name.to_string();

    match result {
        tvg::Tvg_Result_TVG_RESULT_SUCCESS => Ok(()),
        tvg::Tvg_Result_TVG_RESULT_INVALID_ARGUMENT => Err(TvgError::InvalidArgument {
            function_name: func_name,
        }),
        tvg::Tvg_Result_TVG_RESULT_INSUFFICIENT_CONDITION => Err(TvgError::InsufficientCondition {
            function_name: func_name,
        }),
        tvg::Tvg_Result_TVG_RESULT_FAILED_ALLOCATION => Err(TvgError::FailedAllocation {
            function_name: func_name,
        }),
        tvg::Tvg_Result_TVG_RESULT_MEMORY_CORRUPTION => Err(TvgError::MemoryCorruption {
            function_name: func_name,
        }),
        tvg::Tvg_Result_TVG_RESULT_NOT_SUPPORTED => Err(TvgError::NotSupported {
            function_name: func_name,
        }),
        tvg::Tvg_Result_TVG_RESULT_UNKNOWN => Err(TvgError::Unknown {
            function_name: func_name,
        }),
        _ => Err(TvgError::Unknown {
            function_name: func_name,
        }),
    }
}

pub struct TvgRenderer {
    raw_canvas: *mut tvg::Tvg_Canvas,
    engine_method: tvg::Tvg_Engine,
}

impl Default for TvgRenderer {
    fn default() -> Self {
        Self::new(TvgEngine::TvgEngineSw, 0)
    }
}

impl TvgRenderer {
    pub fn new(engine_method: TvgEngine, threads: u32) -> Self {
        let engine = match engine_method {
            TvgEngine::TvgEngineSw => tvg::Tvg_Engine_TVG_ENGINE_SW,
            TvgEngine::TvgEngineGl => tvg::Tvg_Engine_TVG_ENGINE_GL,
        };

        let init_result = unsafe { tvg::tvg_engine_init(engine, threads) };

        if init_result != tvg::Tvg_Result_TVG_RESULT_SUCCESS {
            panic!("Failed to initialize ThorVG engine");
        }

        TvgRenderer {
            raw_canvas: unsafe { tvg::tvg_swcanvas_create() },
            engine_method: engine,
        }
    }

    pub fn push_animation(&mut self, animation: &TvgAnimation) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_canvas_push(self.raw_canvas, animation.raw_paint) };

        convert_tvg_result(result, "tvg_canvas_push")
    }

    pub fn render(&mut self) -> Result<(), TvgError> {
        self.update()?;
        self.draw()?;
        self.sync()
    }
}

impl Renderer for TvgRenderer {
    type Animation = TvgAnimation;
    type Shape = TvgShape;
    type Error = TvgError;

    fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_canvas_set_viewport(self.raw_canvas, x, y, w, h) };

        convert_tvg_result(result, "tvg_canvas_set_viewport")
    }

    fn set_target(
        &mut self,
        buffer: &mut Vec<u32>,
        stride: u32,
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Result<(), TvgError> {
        let color_space = match color_space {
            ColorSpace::ABGR8888 => tvg::Tvg_Colorspace_TVG_COLORSPACE_ABGR8888,
            ColorSpace::ABGR8888S => tvg::Tvg_Colorspace_TVG_COLORSPACE_ABGR8888S,
            ColorSpace::ARGB8888 => tvg::Tvg_Colorspace_TVG_COLORSPACE_ARGB8888,
            ColorSpace::ARGB8888S => tvg::Tvg_Colorspace_TVG_COLORSPACE_ARGB8888S,
        };

        let result = unsafe {
            tvg::tvg_swcanvas_set_target(
                self.raw_canvas,
                buffer.as_mut_ptr(),
                stride,
                width,
                height,
                color_space,
            )
        };

        convert_tvg_result(result, "tvg_swcanvas_set_target")
    }

    fn clear(&self, free: bool) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_canvas_clear(self.raw_canvas, free) };

        convert_tvg_result(result, "tvg_canvas_clear")
    }

    fn push(&mut self, drawable: Drawable<Self>) -> Result<(), TvgError> {
        let raw_paint = match drawable {
            Drawable::Animation(animation) => animation.raw_paint,
            Drawable::Shape(shape) => shape.raw_shape,
        };

        let result = unsafe { tvg::tvg_canvas_push(self.raw_canvas, raw_paint) };

        convert_tvg_result(result, "tvg_canvas_push")
    }

    fn draw(&mut self) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_canvas_draw(self.raw_canvas) };

        convert_tvg_result(result, "tvg_canvas_draw")
    }

    fn sync(&mut self) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_canvas_sync(self.raw_canvas) };

        convert_tvg_result(result, "tvg_canvas_sync")
    }

    fn update(&mut self) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_canvas_update(self.raw_canvas) };

        convert_tvg_result(result, "tvg_canvas_update")
    }
}

impl Drop for TvgRenderer {
    fn drop(&mut self) {
        unsafe {
            tvg::tvg_canvas_destroy(self.raw_canvas);
            tvg::tvg_engine_term(self.engine_method);
        };
    }
}

pub struct TvgAnimation {
    raw_animation: *mut tvg::Tvg_Animation,
    raw_paint: *mut tvg::Tvg_Paint,
}

impl Default for TvgAnimation {
    fn default() -> Self {
        let raw_animation = unsafe { tvg::tvg_animation_new() };
        let raw_paint = unsafe { tvg::tvg_animation_get_picture(raw_animation) };

        Self {
            raw_animation,
            raw_paint,
        }
    }
}

impl TvgAnimation {
    pub fn rotate(&mut self, angle: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_paint_rotate(self.raw_paint, angle) };

        convert_tvg_result(result, "tvg_paint_rotate")
    }

    pub fn set_transform(&mut self, matrix: TvgMatrix) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_paint_set_transform(self.raw_paint, &matrix.into()) };

        convert_tvg_result(result, "tvg_paint_transform")
    }

    pub fn get_transform(&self) -> Result<TvgMatrix, TvgError> {
        let mut matrix = TvgMatrix::new([0.0; 9]);

        let result = unsafe { tvg::tvg_paint_get_transform(self.raw_paint, &mut matrix.inner) };

        convert_tvg_result(result, "tvg_paint_get_transform")?;

        Ok(matrix)
    }

    pub fn set_translate(&mut self, tx: f32, ty: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_paint_translate(self.raw_paint, tx, ty) };

        convert_tvg_result(result, "tvg_paint_translate")
    }

    pub fn set_blend_method(&mut self, blend_method: TvgBlendMethod) -> Result<(), TvgError> {
        let result =
            unsafe { tvg::tvg_paint_set_blend_method(self.raw_paint, blend_method.into()) };

        convert_tvg_result(result, "tvg_paint_set_blend_method")
    }

    pub fn get_translate(&self) -> Result<(f32, f32), TvgError> {
        let mut matrix = TvgMatrix::new([0.0; 9]);

        let result = unsafe { tvg::tvg_paint_get_transform(self.raw_paint, &mut matrix.inner) };

        convert_tvg_result(result, "tvg_paint_get_transform")?;

        Ok((matrix.inner.e13, matrix.inner.e23))
    }

    pub fn set_marker(&mut self, marker: &str) -> Result<(), TvgError> {
        let marker_cstr = CString::new(marker).expect("Failed to create CString");

        let result = unsafe {
            tvg::tvg_lottie_animation_set_marker(self.raw_animation, marker_cstr.as_ptr())
        };

        convert_tvg_result(result, "tvg_lottie_animation_set_marker")
    }

    pub fn get_bounds(&self) -> Result<(f32, f32, f32, f32), TvgError> {
        let mut px: f32 = 0.0;
        let mut py: f32 = 0.0;
        let mut pw: f32 = 0.0;
        let mut ph: f32 = 0.0;

        let result = unsafe {
            tvg::tvg_paint_get_bounds(self.raw_paint, &mut px, &mut py, &mut pw, &mut ph, true)
        };

        convert_tvg_result(result, "tvg_paint_get_bounds")?;

        Ok((px, py, pw, ph))
    }
}

impl Animation for TvgAnimation {
    type Error = TvgError;

    fn load_data(&mut self, data: &str, mimetype: &str, copy: bool) -> Result<(), TvgError> {
        let mimetype = CString::new(mimetype).expect("Failed to create CString");
        let data = CString::new(data).expect("Failed to create CString");

        let result = unsafe {
            tvg::tvg_picture_load_data(
                self.raw_paint,
                data.as_ptr(),
                data.as_bytes().len() as u32,
                mimetype.as_ptr(),
                copy,
            )
        };

        convert_tvg_result(result, "tvg_picture_load_data")?;

        Ok(())
    }

    fn get_layer_bounds(&self, layer_name: &str) -> Result<(f32, f32, f32, f32), TvgError> {
        let paint = self.raw_paint;
        let layer_name_cstr = CString::new(layer_name).expect("Failed to create CString");
        let layer_id = unsafe { tvg::tvg_accessor_generate_id(layer_name_cstr.as_ptr()) };
        let layer = unsafe { tvg::tvg_picture_get_paint(paint, layer_id) };

        if !layer.is_null() {
            let mut px: f32 = -1.0;
            let mut py: f32 = -1.0;
            let mut pw: f32 = -1.0;
            let mut ph: f32 = -1.0;

            let bounds = unsafe {
                tvg::tvg_paint_get_bounds(
                    layer,
                    &mut px as *mut f32,
                    &mut py as *mut f32,
                    &mut pw as *mut f32,
                    &mut ph as *mut f32,
                    true,
                )
            };

            convert_tvg_result(bounds, "tvg_paint_get_bounds")?;

            Ok((px, py, pw, ph))
        } else {
            Err(TvgError::Unknown {
                function_name: "tvg_picture_get_paint".to_string(),
            })
        }
    }

    fn hit_check(&self, layer_name: &str, x: f32, y: f32) -> Result<bool, TvgError> {
        let paint = self.raw_paint;
        let layer_name_cstr = CString::new(layer_name).expect("Failed to create CString");
        let layer_id = unsafe { tvg::tvg_accessor_generate_id(layer_name_cstr.as_ptr()) };
        let layer = unsafe { tvg::tvg_picture_get_paint(paint, layer_id) };

        if !layer.is_null() {
            let mut px: f32 = -1.0;
            let mut py: f32 = -1.0;
            let mut pw: f32 = -1.0;
            let mut ph: f32 = -1.0;

            let bounds = unsafe {
                tvg::tvg_paint_get_bounds(
                    layer,
                    &mut px as *mut f32,
                    &mut py as *mut f32,
                    &mut pw as *mut f32,
                    &mut ph as *mut f32,
                    true,
                )
            };

            convert_tvg_result(bounds, "tvg_paint_get_bounds")?;

            if x >= px && x <= px + pw && y >= py && y <= py + ph {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn get_size(&self) -> Result<(f32, f32), TvgError> {
        let mut width = 0.0;
        let mut height = 0.0;

        let result = unsafe {
            tvg::tvg_picture_get_size(
                self.raw_paint,
                &mut width as *mut f32,
                &mut height as *mut f32,
            )
        };

        convert_tvg_result(result, "tvg_picture_get_size")?;

        Ok((width, height))
    }

    fn set_size(&mut self, width: f32, height: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_picture_set_size(self.raw_paint, width, height) };

        convert_tvg_result(result, "tvg_picture_set_size")
    }

    fn scale(&mut self, factor: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_paint_scale(self.raw_paint, factor) };

        convert_tvg_result(result, "tvg_paint_scale")
    }

    fn translate(&mut self, tx: f32, ty: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_paint_translate(self.raw_paint, tx, ty) };

        convert_tvg_result(result, "tvg_paint_translate")
    }

    fn get_total_frame(&self) -> Result<f32, TvgError> {
        let mut total_frame: f32 = 0.0;

        let result = unsafe {
            tvg::tvg_animation_get_total_frame(self.raw_animation, &mut total_frame as *mut f32)
        };

        convert_tvg_result(result, "tvg_animation_get_total_frame")?;

        Ok(total_frame)
    }

    fn get_duration(&self) -> Result<f32, TvgError> {
        let mut duration: f32 = 0.0;

        let result = unsafe {
            tvg::tvg_animation_get_duration(self.raw_animation, &mut duration as *mut f32)
        };

        convert_tvg_result(result, "tvg_animation_get_duration")?;

        Ok(duration)
    }

    fn set_frame(&mut self, frame_no: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_animation_set_frame(self.raw_animation, frame_no) };

        convert_tvg_result(result, "tvg_animation_set_frame")
    }

    fn get_frame(&self) -> Result<f32, TvgError> {
        let mut curr_frame: f32 = 0.0;
        let result = unsafe {
            tvg::tvg_animation_get_frame(self.raw_animation, &mut curr_frame as *mut f32)
        };

        convert_tvg_result(result, "tvg_animation_get_frame")?;

        Ok(curr_frame)
    }

    fn set_slots(&mut self, slots: &str) -> Result<(), TvgError> {
        let result = if slots.is_empty() {
            unsafe { tvg::tvg_lottie_animation_override(self.raw_animation, ptr::null()) }
        } else {
            let slots_cstr = CString::new(slots).expect("Failed to create CString");
            unsafe { tvg::tvg_lottie_animation_override(self.raw_animation, slots_cstr.as_ptr()) }
        };

        convert_tvg_result(result, "tvg_lottie_animation_override")
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
        let result = unsafe {
            tvg::tvg_shape_set_fill_color(self.raw_shape, color.0, color.1, color.2, color.3)
        };

        convert_tvg_result(result, "tvg_shape_set_fill_color")
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
        let result = unsafe { tvg::tvg_shape_append_rect(self.raw_shape, x, y, w, h, rx, ry) };

        convert_tvg_result(result, "tvg_shape_append_rect")
    }

    fn reset(&mut self) -> Result<(), TvgError> {
        let result = unsafe { tvg::tvg_shape_reset(self.raw_shape) };

        convert_tvg_result(result, "tvg_shape_reset")
    }
}
