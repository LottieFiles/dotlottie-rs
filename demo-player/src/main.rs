use std::{env, path, sync::Arc, time::Instant};

use dotlottie_player_core::{Config, DotLottiePlayer, Mode, Observer};
use minifb::{Key, Window, WindowOptions};

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

struct Timer {
    last_update: Instant,
}

impl Timer {
    fn new() -> Self {
        Self {
            last_update: Instant::now(),
        }
    }

    fn tick(&mut self, animation: &mut DotLottiePlayer) {
        let next_frame = animation.request_frame();

        println!("next_frame: {}", next_frame);
        animation.set_frame(next_frame);
        animation.render();

        self.last_update = Instant::now(); // Reset the timer
    }
}

struct DummyObserver;

impl Observer for DummyObserver {
    fn on_play(&self) {
        println!("on_play");
    }
    fn on_pause(&self) {
        println!("on_pause");
    }
    fn on_stop(&self) {
        println!("on_stop");
    }
    fn on_frame(&self, frame_no: f32) {
        println!("on_frame: {}", frame_no);
    }
    fn on_render(&self, frame_no: f32) {
        println!("on_render: {}", frame_no);
    }
    fn on_load(&self) {
        println!("on_load");
    }
    fn on_loop(&self, loop_count: u32) {
        println!("on_loop: {}", loop_count);
    }
}

fn main() {
    let mut window = Window::new(
        "Thorvg inside Rust - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let base_path = env::var("CARGO_MANIFEST_DIR").unwrap();

    let mut path = path::PathBuf::from(base_path);
    path.push("src/cartoon.json");

    let mut lottie_player: DotLottiePlayer = DotLottiePlayer::new(Config {
        mode: Mode::ReverseBounce,
        loop_animation: true,
        speed: 1.0,
        use_frame_interpolation: true,
        autoplay: true,
        segments: vec![10.0, 45.0],
        background_color: 0xffffffff,
    });
    lottie_player.subscribe(Arc::new(DummyObserver));
    lottie_player.load_animation_path(path.to_str().unwrap(), WIDTH as u32, HEIGHT as u32);
    // lottie_player.set_background_color(0x00ffffff);

    let mut timer = Timer::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        timer.tick(&mut lottie_player);

        let (buffer_ptr, buffer_len) = (lottie_player.buffer_ptr(), lottie_player.buffer_len());

        let buffer =
            unsafe { std::slice::from_raw_parts(buffer_ptr as *const u32, buffer_len as usize) };

        window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
    }
}
