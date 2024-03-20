#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::ffi::CString;
use thiserror::Error;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

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

pub enum TvgEngine {
    TvgEngineSw,
    TvgEngineGl,
}

pub enum TvgColorspace {
    ABGR8888,
    ABGR8888S,
    ARGB8888,
    ARGB8888S,
}

fn convert_tvg_result(result: Tvg_Result, function_name: &str) -> Result<(), TvgError> {
    let func_name = function_name.to_string();

    match result {
        Tvg_Result_TVG_RESULT_SUCCESS => Ok(()),
        Tvg_Result_TVG_RESULT_INVALID_ARGUMENT => Err(TvgError::InvalidArgument {
            function_name: func_name,
        }),
        Tvg_Result_TVG_RESULT_INSUFFICIENT_CONDITION => Err(TvgError::InsufficientCondition {
            function_name: func_name,
        }),
        Tvg_Result_TVG_RESULT_FAILED_ALLOCATION => Err(TvgError::FailedAllocation {
            function_name: func_name,
        }),
        Tvg_Result_TVG_RESULT_MEMORY_CORRUPTION => Err(TvgError::MemoryCorruption {
            function_name: func_name,
        }),
        Tvg_Result_TVG_RESULT_NOT_SUPPORTED => Err(TvgError::NotSupported {
            function_name: func_name,
        }),
        Tvg_Result_TVG_RESULT_UNKNOWN | _ => Err(TvgError::Unknown {
            function_name: func_name,
        }),
    }
}

pub trait Drawable {
    fn as_raw_paint(&self) -> *mut Tvg_Paint;
}

pub struct Canvas {
    raw_canvas: *mut Tvg_Canvas,
    engine_method: Tvg_Engine,
}

impl Canvas {
    pub fn new(engine_method: TvgEngine, threads: u32) -> Self {
        let engine = match engine_method {
            TvgEngine::TvgEngineSw => Tvg_Engine_TVG_ENGINE_SW,
            TvgEngine::TvgEngineGl => Tvg_Engine_TVG_ENGINE_GL,
        };

        let init_result = unsafe { tvg_engine_init(engine, threads) };

        if init_result != Tvg_Result_TVG_RESULT_SUCCESS {
            panic!("Failed to initialize ThorVG engine");
        }

        Canvas {
            raw_canvas: unsafe { tvg_swcanvas_create() },
            engine_method: engine,
        }
    }

    pub fn set_target(
        &mut self,
        buffer: &mut Vec<u32>,
        stride: u32,
        width: u32,
        height: u32,
        color_space: TvgColorspace,
    ) -> Result<(), TvgError> {
        let color_space = match color_space {
            TvgColorspace::ABGR8888 => Tvg_Colorspace_TVG_COLORSPACE_ABGR8888,
            TvgColorspace::ABGR8888S => Tvg_Colorspace_TVG_COLORSPACE_ABGR8888S,
            TvgColorspace::ARGB8888 => Tvg_Colorspace_TVG_COLORSPACE_ARGB8888,
            TvgColorspace::ARGB8888S => Tvg_Colorspace_TVG_COLORSPACE_ARGB8888S,
        };

        let result = unsafe {
            tvg_swcanvas_set_target(
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

    pub fn clear(&self, free: bool) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_clear(self.raw_canvas, free) };

        convert_tvg_result(result, "tvg_canvas_clear")
    }

    pub fn push<T: Drawable>(&mut self, drawable: &T) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_push(self.raw_canvas, drawable.as_raw_paint()) };

        convert_tvg_result(result, "tvg_canvas_push")
    }

    pub fn draw(&mut self) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_draw(self.raw_canvas) };

        convert_tvg_result(result, "tvg_canvas_draw")
    }

    pub fn sync(&mut self) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_sync(self.raw_canvas) };

        convert_tvg_result(result, "tvg_canvas_sync")
    }

    pub fn update(&mut self) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_update(self.raw_canvas) };

        convert_tvg_result(result, "tvg_canvas_update")
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            tvg_canvas_destroy(self.raw_canvas);
            tvg_engine_term(self.engine_method);
        };
    }
}

pub struct Animation {
    raw_animation: *mut Tvg_Animation,
    raw_paint: *mut Tvg_Paint,
}

impl Animation {
    pub fn new() -> Self {
        let raw_animation = unsafe { tvg_animation_new() };
        let raw_paint = unsafe { tvg_animation_get_picture(raw_animation) };

        Animation {
            raw_animation,
            raw_paint,
        }
    }

    pub fn load_data(&mut self, data: &str, mimetype: &str, copy: bool) -> Result<(), TvgError> {
        let mimetype = CString::new(mimetype).expect("Failed to create CString");
        let data = CString::new(data).expect("Failed to create CString");

        let result =
            unsafe {
                tvg_picture_load_data(
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

    pub fn get_size(&self) -> Result<(f32, f32), TvgError> {
        let mut width = 0.0;
        let mut height = 0.0;

        let result =
            unsafe {
                tvg_picture_get_size(
                    self.raw_paint,
                    &mut width as *mut f32,
                    &mut height as *mut f32,
                )
            };

        convert_tvg_result(result, "tvg_picture_get_size")?;

        Ok((width, height))
    }

    pub fn set_size(&mut self, width: f32, height: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg_picture_set_size(self.raw_paint, width, height) };

        convert_tvg_result(result, "tvg_picture_set_size")
    }

    pub fn scale(&mut self, factor: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg_paint_scale(self.raw_paint, factor) };

        convert_tvg_result(result, "tvg_paint_scale")
    }

    pub fn translate(&mut self, tx: f32, ty: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg_paint_translate(self.raw_paint, tx, ty) };

        convert_tvg_result(result, "tvg_paint_translate")
    }

    pub fn get_total_frame(&self) -> Result<f32, TvgError> {
        let mut total_frame: f32 = 0.0;

        let result = unsafe {
            tvg_animation_get_total_frame(self.raw_animation, &mut total_frame as *mut f32)
        };

        convert_tvg_result(result, "tvg_animation_get_total_frame")?;

        return Ok(total_frame);
    }

    pub fn get_duration(&self) -> Result<f32, TvgError> {
        let mut duration: f32 = 0.0;

        let result =
            unsafe { tvg_animation_get_duration(self.raw_animation, &mut duration as *mut f32) };

        convert_tvg_result(result, "tvg_animation_get_duration")?;

        return Ok(duration);
    }

    pub fn set_frame(&mut self, frame_no: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg_animation_set_frame(self.raw_animation, frame_no) };

        convert_tvg_result(result, "tvg_animation_set_frame")
    }

    pub fn get_frame(&self) -> Result<f32, TvgError> {
        let mut curr_frame: f32 = 0.0;
        let result =
            unsafe { tvg_animation_get_frame(self.raw_animation, &mut curr_frame as *mut f32) };

        convert_tvg_result(result, "tvg_animation_get_frame")?;

        return Ok(curr_frame);
    }

    pub fn set_slots(&mut self, slots: &str) -> Result<(), TvgError> {
        let slots = CString::new(slots).expect("Failed to create CString");

        let result = unsafe { tvg_lottie_animation_override(self.raw_animation, slots.as_ptr()) };

        convert_tvg_result(result, "tvg_animation_override")
    }
}

impl Drawable for Animation {
    fn as_raw_paint(&self) -> *mut Tvg_Paint {
        self.raw_paint
    }
}

impl Drop for Animation {
    fn drop(&mut self) {
        unsafe {
            tvg_animation_del(self.raw_animation);
        };
    }
}

pub struct Shape {
    raw_shape: *mut Tvg_Paint,
}

impl Shape {
    pub fn new() -> Self {
        Shape {
            raw_shape: unsafe { tvg_shape_new() },
        }
    }

    pub fn fill(&mut self, color: (u8, u8, u8, u8)) -> Result<(), TvgError> {
        let result =
            unsafe { tvg_shape_set_fill_color(self.raw_shape, color.0, color.1, color.2, color.3) };

        convert_tvg_result(result, "tvg_shape_set_fill_color")
    }

    pub fn append_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rx: f32,
        ry: f32,
    ) -> Result<(), TvgError> {
        let result = unsafe { tvg_shape_append_rect(self.raw_shape, x, y, w, h, rx, ry) };

        convert_tvg_result(result, "tvg_shape_append_rect")
    }

    pub fn reset(&mut self) -> Result<(), TvgError> {
        let result = unsafe { tvg_shape_reset(self.raw_shape) };

        convert_tvg_result(result, "tvg_shape_reset")
    }
}

impl Drawable for Shape {
    fn as_raw_paint(&self) -> *mut Tvg_Paint {
        self.raw_shape
    }
}
