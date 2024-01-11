use std::fs::read_to_string;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::{
    env,
    fs::File,
    io::Read,
    path,
    time::{Duration, Instant},
};

use dotlottie_player_core::DotLottiePlayer;
use minifb::{Key, Window, WindowOptions};

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

// Tick will update the Lottie once a second has passed since last being called
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
        animation.clear();
        animation.set_frame(next_frame);
        animation.render();

        self.last_update = Instant::now(); // Reset the timer
    }
}

fn main() {
    let mut buffer: *const u32;

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

    let animation_data = read_to_string(path.to_str().unwrap()).unwrap();

    let mut lottie_player: DotLottiePlayer = DotLottiePlayer::new(true, false);
    lottie_player.load_animation(animation_data.as_str(), WIDTH as u32, HEIGHT as u32);
    lottie_player.play();

    let buffer_slice = unsafe {
        std::slice::from_raw_parts(lottie_player.get_buffer() as *const u32, WIDTH * HEIGHT * 4)
    };

    let mut timer = Timer::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        timer.tick(&mut lottie_player);

        window
            .update_with_buffer(buffer_slice, WIDTH, HEIGHT)
            .unwrap();
    }
}
