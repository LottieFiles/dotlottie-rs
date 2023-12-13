use std::sync::atomic::AtomicI8;
use std::sync::Mutex;

use crate::thorvg;

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
        let canvas = thorvg::Canvas::new(thorvg::TvgEngine::TvgEngineSw, 3);
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

    pub fn frame(&mut self, no: f32) {
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

    pub fn clear(&mut self) {
        self.canvas.clear(false, true);
    }

    pub fn load_animation_from_path(&mut self, path: &str, width: u32, height: u32) -> bool {
        let mut buffer_lock = self.buffer.lock().unwrap();

        *buffer_lock = vec![0; (width * height * 4) as usize];

        self.canvas.set_target(
            buffer_lock.as_ptr() as *mut u32,
            width,
            width,
            height,
            thorvg::TvgColorspace::ABGR8888,
        );

        if let Some(mut frame_image) = self.animation.get_picture() {
            if frame_image.load(path).is_err() {
                return false;
            }

            let (pw, ph) = frame_image.get_size().unwrap();

            let (scale, shift_x, shift_y) = calculate_scale_and_shift(pw, ph, width, height);

            frame_image.scale(scale);
            frame_image.translate(shift_x, shift_y);

            self.canvas.push(&frame_image);

            self.animation.set_frame(0.0);

            self.canvas.draw();
            self.canvas.sync();
        } else {
            return false;
        }

        true
    }

    pub fn load_animation(&mut self, animation_data: &str, width: u32, height: u32) -> bool {
        let mut buffer_lock = self.buffer.lock().unwrap();

        *buffer_lock = vec![0; (width * height * 4) as usize];

        self.canvas.set_target(
            buffer_lock.as_ptr() as *mut u32,
            width,
            width,
            height,
            thorvg::TvgColorspace::ABGR8888,
        );

        if let Some(mut frame_image) = self.animation.get_picture() {
            if frame_image
                .load_data(animation_data.as_bytes(), "lottie")
                .is_err()
            {
                return false;
            }

            let (pw, ph) = frame_image.get_size().unwrap();

            let (scale, shift_x, shift_y) = calculate_scale_and_shift(pw, ph, width, height);

            frame_image.scale(scale);
            frame_image.translate(shift_x, shift_y);

            self.canvas.push(&frame_image);

            self.animation.set_frame(0.0);

            self.canvas.draw();
            self.canvas.sync();
        } else {
            return false;
        }

        true
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}

fn calculate_scale_and_shift(pw: f32, ph: f32, width: u32, height: u32) -> (f32, f32, f32) {
    let scale = if pw > ph {
        width as f32 / pw
    } else {
        height as f32 / ph
    };

    let shift_x = (width as f32 - pw * scale) / 2.0;
    let shift_y = (height as f32 - ph * scale) / 2.0;

    (scale, shift_x, shift_y)
}
