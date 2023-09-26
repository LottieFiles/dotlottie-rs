#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CString;

pub struct LottiePlayer {
    // Playback related
    autoplay: bool,
    loop_animation: bool,
    speed: i32,
    direction: i8,

    // Animation information related
    duration: f32,
    current_frame: u32,
    total_frames: u32,

    // Data
    animation: *mut Tvg_Animation,
    canvas: *mut Tvg_Canvas,
}

impl LottiePlayer {
    pub fn new(autoplay: bool, loop_animation: bool, direction: i8, speed: i32) -> Self {
        LottiePlayer {
            autoplay,
            loop_animation,
            speed,
            direction,
            duration: 0.0,
            current_frame: 0,
            total_frames: 0,
            animation: std::ptr::null_mut(),
            canvas: std::ptr::null_mut(),
            // For some reason initializing here doesn't work
            // animation: tvg_animation_new(),
            // canvas: tvg_swcanvas_create(),
        }
    }

    pub fn tick(&mut self) {
        unsafe { tvg_animation_get_frame(self.animation, &mut self.current_frame as *mut u32) };

        self.current_frame += 1;

        if self.current_frame >= self.total_frames {
            self.current_frame = 0;
        }

        unsafe { tvg_animation_set_frame(self.animation, self.current_frame) };

        unsafe { tvg_canvas_update_paint(self.canvas, tvg_animation_get_picture(self.animation)) };

        //Draw the canvas
        unsafe { tvg_canvas_draw(self.canvas) };
        unsafe { tvg_canvas_sync(self.canvas) };
    }

    pub fn load_animation(
        &mut self,
        buffer: &mut Vec<u32>,
        animation_data: &str,
        width: u32,
        height: u32,
    ) {
        let mut frame_image = std::ptr::null_mut();

        // let mut duration: f32 = 0.0;
        let mimetype = CString::new("lottie").expect("Failed to create CString");

        println!("Loading up : {}", animation_data);

        unsafe {
            tvg_engine_init(Tvg_Engine_TVG_ENGINE_SW, 0);

            self.canvas = tvg_swcanvas_create();

            tvg_swcanvas_set_target(
                self.canvas,
                buffer.as_mut_ptr(),
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
                false,
            );

            if load_result != Tvg_Result_TVG_RESULT_SUCCESS {
                tvg_animation_del(self.animation);

                // DotLottieError::LoadContentError;
            } else {
                println!("Animation loaded successfully");

                tvg_paint_scale(frame_image, 1.0);

                tvg_animation_get_total_frame(self.animation, &mut self.total_frames as *mut u32);
                tvg_animation_get_duration(self.animation, &mut self.duration);
                tvg_animation_set_frame(self.animation, 0);
                tvg_canvas_push(self.canvas, frame_image);
                tvg_canvas_draw(self.canvas);
                tvg_canvas_sync(self.canvas);

                println!("Total frames: {}", self.total_frames);
                println!("Duration: {}", self.duration);
            }
        }
    }
}

/*
    Fill the buffer with animation data.
    Returns the buffer filled with the first frame.
    Todo: Test on wasm - this is because im not sure if classes will be usable
*/
pub fn load_animation(buffer: &mut Vec<u32>, animation_data: &str, width: u32, height: u32) {
    let mut animation = std::ptr::null_mut();
    let mut canvas = std::ptr::null_mut();
    let mut frame_image = std::ptr::null_mut();
    let mut duration: f32 = 0.0;
    let mimetype = CString::new("lottie").expect("Failed to create CString");

    unsafe {
        tvg_engine_init(Tvg_Engine_TVG_ENGINE_SW, 0);

        canvas = tvg_swcanvas_create();

        tvg_swcanvas_set_target(
            canvas,
            buffer.as_mut_ptr(),
            width,
            width,
            height,
            Tvg_Colorspace_TVG_COLORSPACE_ARGB8888,
        );
    }

    unsafe {
        animation = tvg_animation_new();
        frame_image = tvg_animation_get_picture(animation);

        let load_result = tvg_picture_load_data(
            frame_image,
            animation_data.as_ptr() as *const i8,
            animation_data.len() as u32,
            mimetype.as_ptr(),
            false,
        );

        if load_result != Tvg_Result_TVG_RESULT_SUCCESS {
            tvg_animation_del(animation);

            // DotLottieError::LoadContentError;
        } else {
            tvg_paint_scale(frame_image, 1.0);

            let mut total_frame: u32 = 0;
            tvg_animation_get_total_frame(animation, &mut total_frame as *mut u32);
            tvg_animation_get_duration(animation, &mut duration);
            tvg_animation_set_frame(animation, 0);
            tvg_canvas_push(canvas, frame_image);
            tvg_canvas_draw(canvas);
            tvg_canvas_sync(canvas);
        }
    }
}

/*
todo: Put this inside a class so that it has access to the canvas and animation.
*/
fn tick() {}
