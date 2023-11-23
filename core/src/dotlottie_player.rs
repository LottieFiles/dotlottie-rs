#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::{CStr, CString};

#[repr(C)]
pub struct DotLottiePlayer {
    // Playback related
    autoplay: bool,
    loop_animation: bool,
    speed: i32,
    direction: i8,

    // Animation information related
    duration: f32,
    current_frame: f32,
    total_frames: f32,

    // Data
    animation: *mut Tvg_Animation,
    canvas: *mut Tvg_Canvas,
}

impl DotLottiePlayer {
    pub fn new(autoplay: bool, loop_animation: bool, direction: i8, speed: i32) -> Self {
        DotLottiePlayer {
            autoplay,
            loop_animation,
            speed,
            direction,
            duration: 0.0,
            current_frame: 0.0,
            total_frames: 0.0,
            animation: std::ptr::null_mut(),
            canvas: std::ptr::null_mut(),
            // For some reason initializing here doesn't work
            // animation: tvg_animation_new(),
            // canvas: tvg_swcanvas_create(),
        }
    }

    pub fn tick(&mut self) {
        unsafe {
            tvg_canvas_clear(self.canvas, false, true);
            tvg_animation_get_frame(self.animation, &mut self.current_frame as *mut f32);
        }

        if self.direction == 1 {
            // Thorvg doesnt allow you ot go to total_frames
            if self.current_frame >= self.total_frames - 1.0 {
                self.current_frame = 0.0;
            } else {
                self.current_frame += 1.0;
            }
        } else if self.direction == -1 {
            if self.current_frame == 0.0 {
                // If we set to total_frames, thorvg goes to frame 0
                self.current_frame = self.total_frames - 1.0;
            } else {
                self.current_frame -= 1.0;
            }
        }

        unsafe {
            if tvg_animation_set_frame(self.animation, self.current_frame) == Tvg_Result_TVG_RESULT_SUCCESS {
                tvg_canvas_update_paint(self.canvas, tvg_animation_get_picture(self.animation));
            }
            //Draw the canvas
            tvg_canvas_draw(self.canvas);
            tvg_canvas_sync(self.canvas);
        }
    }

    pub fn load_animation(
        &mut self,
        buffer: *mut u32,
        animation_data: &str,
        width: u32,
        height: u32,
    ) {
        let mut frame_image = std::ptr::null_mut();

        // let mut duration: f32 = 0.0;
        let mimetype = CString::new("lottie").expect("Failed to create CString");

        unsafe {
            tvg_engine_init(Tvg_Engine_TVG_ENGINE_SW, 0);

            self.canvas = tvg_swcanvas_create();

            tvg_swcanvas_set_target(
                self.canvas,
                buffer,
                width,
                width,
                height,
                Tvg_Colorspace_TVG_COLORSPACE_ARGB8888,
            );
        }

        unsafe {
            self.animation = tvg_animation_new();
            frame_image = tvg_animation_get_picture(self.animation);

            let load_result = tvg_picture_load_data(
                frame_image,
                animation_data.as_ptr() as *const i8,
                animation_data.len() as u32,
                mimetype.as_ptr(),
                std::ptr::null_mut(),
                false
            );

            if load_result != Tvg_Result_TVG_RESULT_SUCCESS {
                tvg_animation_del(self.animation);

                // DotLottieError::LoadContentError;
            } else {
                println!("Animation loaded successfully");

                //resize the animation with the given aspect ratio.
                let mut w: f32 = 0.0;
                let mut h: f32 = 0.0;
                tvg_picture_get_size(frame_image, &mut w as *mut f32, &mut h as *mut f32);
                let scale =  (width as f32) / w;
                tvg_picture_set_size(frame_image, w * scale, h * scale);

                tvg_animation_get_total_frame(self.animation, &mut self.total_frames as *mut f32);
                tvg_animation_get_duration(self.animation, &mut self.duration);
                tvg_animation_set_frame(self.animation, 0.0);
                tvg_canvas_push(self.canvas, frame_image);
                tvg_canvas_draw(self.canvas);
                tvg_canvas_sync(self.canvas);

                println!("Total frames: {}", self.total_frames);
                println!("Duration: {}", self.duration);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn create_dotlottie_player(
    autoplay: bool,
    loop_animation: bool,
    direction: i8,
    speed: i32,
) -> *mut DotLottiePlayer {
    Box::into_raw(Box::new(DotLottiePlayer {
        autoplay,
        loop_animation,
        direction,
        speed,
        duration: 0.0,
        current_frame: 0.0,
        total_frames: 0.0,
        animation: std::ptr::null_mut(),
        canvas: std::ptr::null_mut(),
    }))
}

#[no_mangle]
pub extern "C" fn tick(ptr: *mut DotLottiePlayer) {
    unsafe {
        let rust_struct = &mut *ptr;

        rust_struct.tick();
    }
}

#[no_mangle]
pub extern "C" fn load_animation(
    ptr: *mut DotLottiePlayer,
    buffer: *mut u32,
    animation_data: *const ::std::os::raw::c_char,
    width: u32,
    height: u32,
) {
    unsafe {
        let rust_struct = &mut *ptr;

        let animation_data_str = CStr::from_ptr(animation_data).to_str().unwrap();

        rust_struct.load_animation(buffer, animation_data_str, width, height);
    }
}

#[no_mangle]
pub extern "C" fn destroy_dotlottie_player(ptr: *mut DotLottiePlayer) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(ptr));
    }
}
