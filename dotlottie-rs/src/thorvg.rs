#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::{ffi::CString, marker::PhantomData, ptr::null};
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
    ARGB8888,
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

pub struct Canvas {
    raw_canvas: *mut Tvg_Canvas,
}

impl Canvas {
    pub fn new(engine_method: TvgEngine, threads: u32) -> Self {
        let engine_method = match engine_method {
            TvgEngine::TvgEngineSw => Tvg_Engine_TVG_ENGINE_SW,
            TvgEngine::TvgEngineGl => Tvg_Engine_TVG_ENGINE_GL,
        };

        let init_result = unsafe { tvg_engine_init(engine_method, threads) };

        if init_result != Tvg_Result_TVG_RESULT_SUCCESS {
            panic!("Failed to initialize ThorVG engine");
        }

        Canvas {
            raw_canvas: unsafe { tvg_swcanvas_create() },
        }
    }

    pub fn set_target(
        &self,
        buffer: &mut Vec<u32>,
        stride: u32,
        width: u32,
        height: u32,
        color_space: TvgColorspace,
    ) -> Result<(), TvgError> {
        let color_space = match color_space {
            TvgColorspace::ABGR8888 => Tvg_Colorspace_TVG_COLORSPACE_ABGR8888,
            TvgColorspace::ARGB8888 => Tvg_Colorspace_TVG_COLORSPACE_ARGB8888,
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

    pub fn push(&self, picture: &Picture) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_push(self.raw_canvas, picture.raw_paint()) };

        convert_tvg_result(result, "tvg_canvas_push")
    }

    pub fn draw(&self) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_draw(self.raw_canvas) };

        convert_tvg_result(result, "tvg_canvas_draw")
    }

    pub fn sync(&self) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_sync(self.raw_canvas) };

        convert_tvg_result(result, "tvg_canvas_sync")
    }

    pub fn update(&self) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_update(self.raw_canvas) };

        convert_tvg_result(result, "tvg_canvas_update")
    }

    pub fn set_mempool(&self, policy: Tvg_Mempool_Policy) -> Result<(), TvgError> {
        let result = unsafe { tvg_swcanvas_set_mempool(self.raw_canvas, policy) };

        convert_tvg_result(result, "tvg_swcanvas_set_mempool")
    }
}

pub struct Picture<'a> {
    raw_paint: *mut Tvg_Paint,
    // Ensure `Picture` does not outlive the `Animation` it is associated with.
    _lifetime: PhantomData<&'a Animation>,
}

impl<'a> Picture<'a> {
    pub fn from_raw(raw: *mut Tvg_Paint) -> Self {
        Picture {
            raw_paint: raw,
            _lifetime: PhantomData,
        }
    }

    fn raw_paint(&self) -> *mut Tvg_Paint {
        self.raw_paint
    }

    pub fn load(&mut self, path: &str) -> Result<(), TvgError> {
        let result =
            unsafe { tvg_picture_load(self.raw_paint, path.as_ptr() as *const std::ffi::c_char) };

        convert_tvg_result(result, "tvg_picture_load")
    }

    pub fn load_data(&mut self, data: &[u8], mimetype: &str, copy: bool) -> Result<(), TvgError> {
        let mimetype = CString::new(mimetype).expect("Failed to create CString");

        let result = unsafe {
            tvg_picture_load_data(
                self.raw_paint,
                data.as_ptr() as *const std::ffi::c_char,
                data.len() as u32,
                mimetype.as_ptr(),
                false,
            )
        };

        convert_tvg_result(result, "tvg_picture_load_data")?;

        Ok(())
    }

    pub fn get_size(&self) -> Result<(f32, f32), TvgError> {
        let mut width = 0.0;
        let mut height = 0.0;

        let result = unsafe {
            tvg_picture_get_size(
                self.raw_paint,
                &mut width as *mut f32,
                &mut height as *mut f32,
            )
        };

        convert_tvg_result(result, "tvg_picture_get_size")?;

        Ok((width, height))
    }

    pub fn set_size(&self, width: f32, height: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg_picture_set_size(self.raw_paint, width, height) };

        convert_tvg_result(result, "tvg_picture_set_size")
    }

    pub fn scale(&mut self, factor: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg_paint_scale(self.raw_paint(), factor) };

        convert_tvg_result(result, "tvg_paint_scale")
    }

    pub fn translate(&mut self, tx: f32, ty: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg_paint_translate(self.raw_paint(), tx, ty) };

        convert_tvg_result(result, "tvg_paint_translate")
    }
}

pub struct Animation {
    raw_animation: *mut Tvg_Animation,
}

impl Animation {
    pub fn new() -> Self {
        Animation {
            raw_animation: unsafe { tvg_animation_new() },
        }
    }

    pub fn get_picture<'a>(&'a self) -> Option<Picture<'a>> {
        let raw_picture = unsafe { tvg_animation_get_picture(self.raw_animation) };

        if raw_picture.is_null() {
            return None;
        }

        Some(Picture::from_raw(raw_picture))
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

    pub fn set_frame(&self, frame_no: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg_animation_set_frame(self.raw_animation, frame_no) };

        convert_tvg_result(result, "tvg_animation_set_frame")
    }

    pub fn get_frame(&self) -> Result<f32, TvgError> {
        let mut curr_frame: f32 = 0.0;
        let result =
            unsafe { tvg_animation_get_frame(self.raw_animation, &mut curr_frame as *mut f32) };

        convert_tvg_result(result, "tvg_animation_get_frame")?;

        Ok(curr_frame)
    }
}
