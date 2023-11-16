#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CString;
use std::sync::atomic::AtomicI8;
use std::sync::{Arc, Mutex, RwLock};

pub struct DotLottiePlayer {
    // Playback related
    autoplay: bool,
    loop_animation: bool,
    speed: i32,
    direction: Arc<RwLock<AtomicI8>>,

    // Animation information related
    duration: f32,
    current_frame: Arc<Mutex<f32>>,
    total_frames: Arc<Mutex<f32>>,

    // Data
    animation: Arc<RwLock<*mut Tvg_Animation>>,
    canvas: Arc<RwLock<*mut Tvg_Canvas>>,
}

impl DotLottiePlayer {
    pub fn new() -> Self {
        DotLottiePlayer {
            autoplay: false,
            loop_animation: false,
            speed: 1,
            direction: Arc::new(RwLock::new(AtomicI8::new(1))),
            duration: 0.0,
            current_frame: Arc::new(Mutex::new(0.0)),

            total_frames: Arc::new(Mutex::new(0.0)),
            animation: Arc::new(RwLock::new(std::ptr::null_mut())),
            canvas: Arc::new(RwLock::new(std::ptr::null_mut())),
            // For some reason initializing here doesn't work
            // animation: tvg_animation_new(),
            // canvas: tvg_swcanvas_create(),
        }
    }

    pub fn tick(&self) {
        unsafe {
            let mut current_frame_lock = self.current_frame.lock().unwrap();
            let total_frames_lock = self.total_frames.lock().unwrap();
            let direction = self.direction.read().unwrap().as_ptr();
            let canvas = self.canvas.read().unwrap().as_mut().unwrap();
            let animation = self.animation.read().unwrap().as_mut().unwrap();

            tvg_animation_get_frame(animation, &mut *current_frame_lock);

            if *direction == 1 {
                // Thorvg doesnt allow you ot go to total_frames
                println!("Current frame : {}", *current_frame_lock);

                if *current_frame_lock >= *total_frames_lock - 1.0 {
                    *current_frame_lock = 0.0;
                } else {
                    *current_frame_lock += 1.0;
                }
            } else if *direction == -1 {
                if *current_frame_lock == 0.0 {
                    // If we set to total_frames, thorvg goes to frame 0
                    *current_frame_lock = *total_frames_lock;

                    // self.current_frame
                    //     .write()
                    //     .unwrap()
                    //     .store(*total_frames - 1, std::sync::atomic::Ordering::Relaxed);
                } else {
                    *current_frame_lock -= 1.0;
                }
            }

            tvg_animation_set_frame(animation, *current_frame_lock);

            tvg_canvas_update_paint(canvas, tvg_animation_get_picture(animation));

            //Draw the canvas
            tvg_canvas_draw(canvas);
            tvg_canvas_sync(canvas);
            print!(
                "current frame {} - {}",
                *current_frame_lock, total_frames_lock
            );
        };
    }

    pub fn load_animation(&self, buffer: &Vec<u32>, animation_data: &str, width: u32, height: u32) {
        let mut frame_image = std::ptr::null_mut();

        let mimetype = CString::new("lottie").expect("Failed to create CString");

        unsafe {
            tvg_engine_init(Tvg_Engine_TVG_ENGINE_SW, 0);

            *self.canvas.write().unwrap() = tvg_swcanvas_create();

            let canvas = self.canvas.read().unwrap().as_mut().unwrap();

            tvg_swcanvas_set_target(
                canvas,
                buffer.as_ptr() as *mut u32,
                width,
                width,
                height,
                Tvg_Colorspace_TVG_COLORSPACE_ARGB8888,
            );

            *self.animation.write().unwrap() = tvg_animation_new();

            let animation = self.animation.read().unwrap().as_mut().unwrap();

            frame_image = tvg_animation_get_picture(animation);

            let load_result = tvg_picture_load_data(
                frame_image,
                animation_data.as_ptr() as *const std::os::raw::c_char,
                animation_data.len() as u32,
                mimetype.as_ptr(),
                false,
            );

            if load_result != Tvg_Result_TVG_RESULT_SUCCESS {
                tvg_animation_del(animation);

                // DotLottieError::LoadContentError;
            } else {
                println!("Animation loaded successfully");

                tvg_paint_scale(frame_image, 1.0);

                let current_frame_lock = self.current_frame.lock().unwrap();
                let mut total_frames_lock = self.total_frames.lock().unwrap();

                tvg_animation_get_total_frame(animation, &mut *total_frames_lock);

                // tvg_animation_get_duration(animation, &mut self.duration);
                tvg_animation_set_frame(animation, 0.0);
                tvg_canvas_push(canvas, frame_image);
                tvg_canvas_draw(canvas);
                tvg_canvas_sync(canvas);

                println!("Total frames: {}", *total_frames_lock);
                println!("Duration: {}", self.duration);
            }
        }
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}

// #[no_mangle]
// pub extern "C" fn create_dotlottie_player(
//     autoplay: bool,
//     loop_animation: bool,
//     direction: i8,
//     speed: i32,
// ) -> *mut DotLottiePlayer {
//     Box::into_raw(Box::new(DotLottiePlayer {
//         autoplay,
//         loop_animation,
//         direction,
//         speed,
//         duration: 0.0,
//         current_frame: 0,
//         total_frames: 0,
//         animation: std::ptr::null_mut(),
//         canvas: std::ptr::null_mut(),
//     }))
// }

// #[no_mangle]
// pub extern "C" fn tick(ptr: *mut DotLottiePlayer) {
//     unsafe {
//         let rust_struct = &mut *ptr;

//         rust_struct.tick();
//     }
// }

// #[no_mangle]
// pub extern "C" fn load_animation(
//     ptr: *mut DotLottiePlayer,
//     buffer: *mut u32,
//     animation_data: *const ::std::os::raw::c_char,
//     width: u32,
//     height: u32,
// ) {
//     unsafe {
//         let rust_struct = &mut *ptr;

//         let animation_data_str = CStr::from_ptr(animation_data).to_str().unwrap();

//         rust_struct.load_animation(buffer, animation_data_str, width, height);
//     }
// }

// #[no_mangle]
// pub extern "C" fn destroy_dotlottie_player(ptr: *mut DotLottiePlayer) {
//     if ptr.is_null() {
//         return;
//     }
//     unsafe {
//         drop(Box::from_raw(ptr));
//     }
// }
