#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CString;
use std::sync::atomic::AtomicI8;
use std::sync::Mutex;

pub mod thorvg {
    use super::*;

    pub struct Canvas {
        raw_canvas: *mut Tvg_Canvas,
    }

    impl Canvas {
        pub fn new(engine_method: Tvg_Engine, threads: u32) -> Self {
            Canvas {
                raw_canvas: unsafe {
                    tvg_engine_init(engine_method, threads);

                    tvg_swcanvas_create()
                },
            }
        }

        pub fn set_target(
            &self,
            buffer: *mut u32,
            stride: u32,
            width: u32,
            height: u32,
            color_space: Tvg_Colorspace,
        ) -> Tvg_Result {
            unsafe {
                tvg_swcanvas_set_target(self.raw_canvas, buffer, stride, width, height, color_space)
            }
        }

        pub fn clear(&self, paints: bool, buffer: bool) -> Tvg_Result {
            unsafe { tvg_canvas_clear(self.raw_canvas, paints, buffer) }
        }

        pub fn push(&self, picture: *mut Tvg_Paint) -> Tvg_Result {
            unsafe { tvg_canvas_push(self.raw_canvas, picture) }
        }

        pub fn draw(&self) -> Tvg_Result {
            unsafe { tvg_canvas_draw(self.raw_canvas) }
        }

        pub fn sync(&self) -> Tvg_Result {
            unsafe { tvg_canvas_sync(self.raw_canvas) }
        }

        pub fn update(&self) -> Tvg_Result {
            unsafe { tvg_canvas_update(self.raw_canvas) }
        }

        pub fn destroy(&self) -> Tvg_Result {
            unsafe { tvg_canvas_destroy(self.raw_canvas) }
        }

        pub fn set_mempool(&self, policy: Tvg_Mempool_Policy) -> Tvg_Result {
            unsafe { tvg_swcanvas_set_mempool(self.raw_canvas, policy) }
        }
    }

    impl Drop for Canvas {
        fn drop(&mut self) {
            self.destroy();

            self.raw_canvas = std::ptr::null_mut();
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

        pub fn get_picture(&self) -> *mut Tvg_Paint {
            unsafe { tvg_animation_get_picture(self.raw_animation) }
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

                let result =
                    tvg_animation_get_duration(self.raw_animation, &mut duration as *mut f32);

                if result != Tvg_Result_TVG_RESULT_SUCCESS {
                    return Err(result);
                }

                return Ok(duration);
            }
        }

        pub fn set_frame(&self, frame_no: f32) -> Tvg_Result {
            unsafe { tvg_animation_set_frame(self.raw_animation, frame_no) }
        }

        pub fn get_frame(&self) -> Result<f32, Tvg_Result> {
            unsafe {
                let mut curr_frame: f32 = 0.0;

                let result =
                    tvg_animation_get_frame(self.raw_animation, &mut curr_frame as *mut f32);

                if result != Tvg_Result_TVG_RESULT_SUCCESS {
                    return Err(result);
                }

                Ok(curr_frame)
            }
        }

        pub fn del(&self) -> Tvg_Result {
            unsafe { tvg_animation_del(self.raw_animation) }
        }
    }

    impl Drop for Animation {
        fn drop(&mut self) {
            self.del();

            self.raw_animation = std::ptr::null_mut();
        }
    }
}

#[allow(dead_code)]
pub struct DotLottiePlayer {
    // Playback related
    autoplay: bool,
    loop_animation: bool,
    speed: i32,
    direction: AtomicI8,

    // Data
    animation: thorvg::Animation,
    canvas: thorvg::Canvas,
    buffer: Mutex<Vec<u32>>,
}

impl DotLottiePlayer {
    pub fn new() -> Self {
        let canvas = thorvg::Canvas::new(Tvg_Engine_TVG_ENGINE_SW, 0);
        let animation = thorvg::Animation::new();
        let buffer = Mutex::new(vec![]);

        DotLottiePlayer {
            autoplay: false,
            loop_animation: false,
            speed: 1,
            direction: AtomicI8::new(1),
            animation,
            canvas,
            buffer,
        }
    }

    pub fn frame(&self, no: f32) {
        self.canvas.clear(false, true);

        self.animation.set_frame(no);

        self.canvas.update();
        self.canvas.draw();
        self.canvas.sync();
    }

    pub fn get_total_frame(&self) -> f32 {
        let total_frames = self.animation.get_total_frame();

        match total_frames {
            Ok(total_frames) => total_frames,
            Err(_) => 0.0,
        }
    }

    pub fn get_duration(&self) -> f32 {
        let duration = self.animation.get_duration();

        match duration {
            Ok(duration) => duration,
            Err(_) => 0.0,
        }
    }

    pub fn get_current_frame(&self) -> f32 {
        let result = self.animation.get_frame();

        match result {
            Ok(frame) => frame,
            Err(_) => 0.0,
        }
    }

    pub fn get_buffer(&self) -> i64 {
        let buffer_lock = self.buffer.lock().unwrap();
        return buffer_lock.as_ptr().cast::<u32>() as i64;
    }

    pub fn get_buffer_size(&self) -> i64 {
        let buffer_lock = self.buffer.lock().unwrap();

        buffer_lock.len() as i64
    }

    pub fn clear(&self) {
        self.canvas.clear(false, true);
    }

    pub fn load_animation_from_path(&self, path: &str, width: u32, height: u32) -> bool {
        unsafe {
            let mut buffer_lock = self.buffer.lock().unwrap();

            *buffer_lock = vec![0; (width * height * 4) as usize];

            self.canvas.set_target(
                buffer_lock.as_ptr() as *mut u32,
                width,
                width,
                height,
                Tvg_Colorspace_TVG_COLORSPACE_ABGR8888,
            );

            let frame_image = self.animation.get_picture();

            let load_result =
                tvg_picture_load(frame_image, path.as_ptr() as *const std::os::raw::c_char);

            if load_result != Tvg_Result_TVG_RESULT_SUCCESS {
                self.animation.del();

                return false;
            } else {
                let mut pw: f32 = 0.0;
                let mut ph: f32 = 0.0;
                let scale: f32;
                let mut shiftY: f32 = 0.0;
                let mut shiftX: f32 = 0.0;

                tvg_picture_get_size(frame_image, &mut pw as *mut f32, &mut ph as *mut f32);

                if pw > ph {
                    scale = width as f32 / pw;
                    shiftY = (height as f32 / ph * scale) * 0.5;
                } else {
                    scale = height as f32 / ph;
                    shiftX = (width as f32 - pw * scale) * 0.5;
                }

                tvg_paint_scale(frame_image, scale);
                tvg_paint_translate(frame_image, shiftX, shiftY);

                self.animation.set_frame(0.0);

                self.canvas.push(frame_image);
                self.canvas.draw();
                self.canvas.sync();
            }
        }

        true
    }

    pub fn load_animation(&self, animation_data: &str, width: u32, height: u32) -> bool {
        let mimetype = CString::new("lottie").expect("Failed to create CString");

        unsafe {
            let mut buffer_lock = self.buffer.lock().unwrap();

            *buffer_lock = vec![0; (width * height * 4) as usize];

            self.canvas.set_target(
                buffer_lock.as_ptr() as *mut u32,
                width,
                width,
                height,
                Tvg_Colorspace_TVG_COLORSPACE_ABGR8888,
            );

            let frame_image = self.animation.get_picture();

            // resource path (null if not needed)
            let rpath = std::ptr::null();

            let load_result = tvg_picture_load_data(
                frame_image,
                animation_data.as_ptr() as *const std::os::raw::c_char,
                animation_data.len() as u32,
                mimetype.as_ptr(),
                rpath,
                false,
            );

            if load_result != Tvg_Result_TVG_RESULT_SUCCESS {
                self.animation.del();

                return false;
            } else {
                let mut pw: f32 = 0.0;
                let mut ph: f32 = 0.0;
                let scale: f32;
                let mut shiftY: f32 = 0.0;
                let mut shiftX: f32 = 0.0;

                tvg_picture_get_size(frame_image, &mut pw as *mut f32, &mut ph as *mut f32);

                if pw > ph {
                    scale = width as f32 / pw;
                    shiftY = (height as f32 / ph * scale) * 0.5;
                } else {
                    scale = height as f32 / ph;
                    shiftX = (width as f32 - pw * scale) * 0.5;
                }

                tvg_paint_scale(frame_image, scale);
                tvg_paint_translate(frame_image, shiftX, shiftY);

                self.animation.set_frame(0.0);

                self.canvas.push(frame_image);
                self.canvas.draw();
                self.canvas.sync();
            }
        }

        true
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}
