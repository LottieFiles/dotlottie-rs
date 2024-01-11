use std::fs::read_to_string;
use std::{env, path, time::Instant};

use dotlottie_player_core::{DotLottiePlayer, Mode};
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
        animation.clear();
        animation.set_frame(next_frame);
        animation.render();

        self.last_update = Instant::now(); // Reset the timer
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

    let animation_data = read_to_string(path.to_str().unwrap()).unwrap();

    let mut lottie_player: DotLottiePlayer = DotLottiePlayer::new(Mode::Forward, true, true, 1.0);
    lottie_player.load_animation_data(animation_data.as_str(), WIDTH as u32, HEIGHT as u32);

    lottie_player.play();

    let mut timer = Timer::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        timer.tick(&mut lottie_player);

        window
            .update_with_buffer(lottie_player.buffer(), WIDTH, HEIGHT)
            .unwrap();
    }
}
