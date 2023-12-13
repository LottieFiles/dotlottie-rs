#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CString;
use std::sync::atomic::AtomicI8;
use std::sync::{Arc, Mutex, RwLock};

#[allow(dead_code)]
pub struct DotLottiePlayer {
    // Playback related
    autoplay: bool,
    loop_animation: bool,
    speed: i32,
    direction: AtomicI8,

    // Animation information related
    current_frame: Arc<RwLock<f32>>,
    total_frames: Arc<RwLock<f32>>,
    duration: Arc<RwLock<f32>>,

    // Data
    animation: Arc<RwLock<*mut Tvg_Animation>>,
    canvas: Arc<RwLock<*mut Tvg_Canvas>>,
    buffer: Mutex<Vec<u32>>,
}

impl DotLottiePlayer {
    pub fn new() -> Self {
        DotLottiePlayer {
            autoplay: false,
            loop_animation: false,
            speed: 1,
            direction: AtomicI8::new(1),
            current_frame: Arc::new(RwLock::new(0.0)),

            total_frames: Arc::new(RwLock::new(0.0)),
            duration: Arc::new(RwLock::new(0.0)),
            animation: Arc::new(RwLock::new(std::ptr::null_mut())),
            canvas: Arc::new(RwLock::new(std::ptr::null_mut())),
            buffer: Mutex::new(vec![]),
            // For some reason initializing here doesn't work
            // animation: tvg_animation_new(),
            // canvas: tvg_swcanvas_create(),
        }
    }

    pub fn frame(&self, no: f32) {
        unsafe {
            let current_frame = &mut *self.current_frame.write().unwrap();
            let animation = self.animation.read().unwrap().as_mut().unwrap();
            let canvas = self.canvas.read().unwrap().as_mut().unwrap();

            *current_frame = no;

            tvg_canvas_clear(canvas, false, true);

            tvg_animation_set_frame(animation, *current_frame);

            tvg_canvas_update(canvas);

            tvg_canvas_draw(canvas);

            tvg_canvas_sync(canvas);
        }
    }

    pub fn get_total_frame(&self) -> f32 {
        return self.total_frames.read().unwrap().clone();
    }

    pub fn get_duration(&self) -> f32 {
        return self.duration.read().unwrap().clone();
    }

    pub fn get_current_frame(&self) -> f32 {
        return self.current_frame.read().unwrap().clone();
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
        unsafe {
            let canvas = self.canvas.read().unwrap().as_mut().unwrap();

            tvg_canvas_clear(canvas, false, true);
        }
    }

    pub fn load_animation_from_path(&self, path: &str, width: u32, height: u32) -> bool {
        unsafe {
            tvg_engine_init(Tvg_Engine_TVG_ENGINE_SW, 0);

            *self.canvas.write().unwrap() = tvg_swcanvas_create();

            let canvas = self.canvas.read().unwrap().as_mut().unwrap();

            let mut buffer_lock = self.buffer.lock().unwrap();

            *buffer_lock = vec![0; (width * height * 4) as usize];

            // self.buffer.as = vec![width * height];

            tvg_swcanvas_set_target(
                canvas,
                buffer_lock.as_ptr() as *mut u32,
                width,
                width,
                height,
                Tvg_Colorspace_TVG_COLORSPACE_ABGR8888,
            );

            *self.animation.write().unwrap() = tvg_animation_new();

            let animation = self.animation.read().unwrap().as_mut().unwrap();

            let frame_image = tvg_animation_get_picture(animation);

            let load_result =
                tvg_picture_load(frame_image, path.as_ptr() as *const std::os::raw::c_char);

            if load_result != Tvg_Result_TVG_RESULT_SUCCESS {
                tvg_animation_del(animation);

                return false;
            } else {
                let total_frames = &mut *self.total_frames.write().unwrap();
                let duration = &mut *self.duration.write().unwrap();
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

                tvg_animation_get_total_frame(animation, total_frames as *mut f32);
                tvg_animation_get_duration(animation, duration as *mut f32);
                tvg_animation_set_frame(animation, 0.0);
                tvg_canvas_push(canvas, frame_image);
                tvg_canvas_draw(canvas);
                tvg_canvas_sync(canvas);
            }
        }

        true
    }

    pub fn load_animation(&self, animation_data: &str, width: u32, height: u32) -> bool {
        let mimetype = CString::new("lottie").expect("Failed to create CString");

        unsafe {
            tvg_engine_init(Tvg_Engine_TVG_ENGINE_SW, 0);

            *self.canvas.write().unwrap() = tvg_swcanvas_create();

            let canvas = self.canvas.read().unwrap().as_mut().unwrap();

            let mut buffer_lock = self.buffer.lock().unwrap();

            *buffer_lock = vec![0; (width * height * 4) as usize];

            tvg_swcanvas_set_target(
                canvas,
                buffer_lock.as_ptr() as *mut u32,
                width,
                width,
                height,
                Tvg_Colorspace_TVG_COLORSPACE_ABGR8888,
            );

            *self.animation.write().unwrap() = tvg_animation_new();

            let animation = self.animation.read().unwrap().as_mut().unwrap();

            let frame_image = tvg_animation_get_picture(animation);

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
                tvg_animation_del(animation);

                return false;
            } else {
                let total_frames = &mut *self.total_frames.write().unwrap();
                let duration = &mut *self.duration.write().unwrap();
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

                tvg_animation_get_total_frame(animation, total_frames as *mut f32);
                tvg_animation_get_duration(animation, duration as *mut f32);
                tvg_animation_set_frame(animation, 0.0);
                tvg_canvas_push(canvas, frame_image);
                tvg_canvas_draw(canvas);
                tvg_canvas_sync(canvas);
            }
        }

        true
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}
