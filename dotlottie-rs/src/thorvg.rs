#![allow(non_upper_case_globals)]

use std::ffi::CString;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Debug)]
pub enum TvgError {
    InvalidArgument,
    InsufficientCondition,
    FailedAllocation,
    MemoryCorruption,
    NotSupported,
    Unknown,
}

pub enum TvgEngine {
    TvgEngineSw,
    TvgEngineGl,
}

pub enum TvgColorspace {
    ABGR8888,
    ARGB8888,
}

fn convert_tvg_result(result: Tvg_Result) -> Result<(), TvgError> {
    match result {
        Tvg_Result_TVG_RESULT_SUCCESS => Ok(()),
        Tvg_Result_TVG_RESULT_INVALID_ARGUMENT => Err(TvgError::InvalidArgument),
        Tvg_Result_TVG_RESULT_INSUFFICIENT_CONDITION => Err(TvgError::InsufficientCondition),
        Tvg_Result_TVG_RESULT_FAILED_ALLOCATION => Err(TvgError::FailedAllocation),
        Tvg_Result_TVG_RESULT_MEMORY_CORRUPTION => Err(TvgError::MemoryCorruption),
        Tvg_Result_TVG_RESULT_NOT_SUPPORTED => Err(TvgError::NotSupported),
        Tvg_Result_TVG_RESULT_UNKNOWN | _ => Err(TvgError::Unknown),
    }
}

pub struct Canvas {
    raw_canvas: *mut Tvg_Canvas,
}

impl Canvas {
    pub fn new(engine_method: TvgEngine, threads: u32) -> Self {
        Canvas {
            raw_canvas: unsafe {
                let engine_method = match engine_method {
                    TvgEngine::TvgEngineSw => Tvg_Engine_TVG_ENGINE_SW,
                    TvgEngine::TvgEngineGl => Tvg_Engine_TVG_ENGINE_GL,
                };

                tvg_engine_init(engine_method, threads);

                tvg_swcanvas_create()
            },
        }
    }

    pub fn set_target(
        &mut self,
        buffer: *mut u32,
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
            tvg_swcanvas_set_target(self.raw_canvas, buffer, stride, width, height, color_space)
        };

        convert_tvg_result(result)
    }

    pub fn clear(&mut self, paints: bool, buffer: bool) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_clear(self.raw_canvas, paints, buffer) };

        convert_tvg_result(result)
    }

    pub fn push(&mut self, picture: &Picture) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_push(self.raw_canvas, picture.raw_paint()) };

        convert_tvg_result(result)
    }

    pub fn draw(&mut self) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_draw(self.raw_canvas) };

        convert_tvg_result(result)
    }

    pub fn sync(&mut self) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_sync(self.raw_canvas) };

        convert_tvg_result(result)
    }

    pub fn update(&mut self) -> Result<(), TvgError> {
        let result = unsafe { tvg_canvas_update(self.raw_canvas) };

        convert_tvg_result(result)
    }

    pub fn set_mempool(&mut self, policy: Tvg_Mempool_Policy) -> Result<(), TvgError> {
        let result = unsafe { tvg_swcanvas_set_mempool(self.raw_canvas, policy) };

        convert_tvg_result(result)
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            tvg_canvas_destroy(self.raw_canvas);
        }

        self.raw_canvas = std::ptr::null_mut();
    }
}

pub struct Picture {
    raw_paint: *mut Tvg_Paint,
}

impl Picture {
    pub fn from_raw(raw: *mut Tvg_Paint) -> Self {
        Picture { raw_paint: raw }
    }

    fn raw_paint(&self) -> *mut Tvg_Paint {
        self.raw_paint
    }

    pub fn load(&self, path: &str) -> Result<(), TvgError> {
        let path = CString::new(path).expect("Failed to create CString");

        let result = unsafe { tvg_picture_load(self.raw_paint, path.as_ptr()) };

        convert_tvg_result(result)
    }

    pub fn load_data(&self, data: &[u8], mimetype: &str) -> Result<(), TvgError> {
        let mimetype = CString::new(mimetype).expect("Failed to create CString");

        let result = unsafe {
            tvg_picture_load_data(
                self.raw_paint,
                data.as_ptr() as *const std::os::raw::c_char,
                data.len() as u32,
                mimetype.as_ptr(),
                std::ptr::null(),
                false,
            )
        };

        convert_tvg_result(result)
    }

    pub fn get_size(&self) -> Result<(f32, f32), TvgError> {
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;

        let result = unsafe {
            tvg_picture_get_size(
                self.raw_paint,
                &mut width as *mut f32,
                &mut height as *mut f32,
            )
        };

        convert_tvg_result(result)?;

        Ok((width, height))
    }

    pub fn set_size(&self, width: f32, height: f32) {
        unsafe {
            tvg_picture_set_size(self.raw_paint, width, height);
        }
    }

    pub fn scale(&mut self, factor: f32) -> Tvg_Result {
        unsafe { tvg_paint_scale(self.raw_paint(), factor) }
    }

    pub fn translate(&mut self, tx: f32, ty: f32) -> Tvg_Result {
        unsafe { tvg_paint_translate(self.raw_paint(), tx, ty) }
    }

    pub fn rotate(&mut self, degree: f32) -> Tvg_Result {
        unsafe { tvg_paint_rotate(self.raw_paint(), degree) }
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

    pub fn get_picture(&self) -> Option<Picture> {
        let raw_picture = unsafe { tvg_animation_get_picture(self.raw_animation) };
        if !raw_picture.is_null() {
            Some(Picture::from_raw(raw_picture))
        } else {
            None
        }
    }

    pub fn get_total_frame(&self) -> Result<f32, Tvg_Result> {
        unsafe {
            let mut total_frame: f32 = 0.0;

            let result =
                tvg_animation_get_total_frame(self.raw_animation, &mut total_frame as *mut f32);

            if result != Tvg_Result_TVG_RESULT_SUCCESS {
                return Err(result);
            }

            return Ok(total_frame);
        }
    }

    pub fn get_duration(&self) -> Result<f32, Tvg_Result> {
        unsafe {
            let mut duration: f32 = 0.0;

            let result = tvg_animation_get_duration(self.raw_animation, &mut duration as *mut f32);

            if result != Tvg_Result_TVG_RESULT_SUCCESS {
                return Err(result);
            }

            return Ok(duration);
        }
    }

    pub fn set_frame(&self, frame_no: f32) -> Result<(), TvgError> {
        let result = unsafe { tvg_animation_set_frame(self.raw_animation, frame_no) };

        convert_tvg_result(result)
    }

    pub fn get_frame(&self) -> Result<f32, TvgError> {
        let mut curr_frame: f32 = 0.0;
        let result =
            unsafe { tvg_animation_get_frame(self.raw_animation, &mut curr_frame as *mut f32) };

        convert_tvg_result(result)?;

        Ok(curr_frame)
    }
}

impl Drop for Animation {
    fn drop(&mut self) {
        unsafe {
            tvg_animation_del(self.raw_animation);
        }

        self.raw_animation = std::ptr::null_mut();
    }
}
