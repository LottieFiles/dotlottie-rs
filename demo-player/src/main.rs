use std::fs::{self, File};
use std::io::Read;
use std::{env, path, time::Instant};

use dotlottie_player_core::{Config, DotLottiePlayer, Mode};
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

        // println!("next_frame: {}", next_frame);
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

    let mut lottie_player: DotLottiePlayer = DotLottiePlayer::new(Config {
        mode: Mode::ReverseBounce,
        loop_animation: true,
        speed: 1.0,
        use_frame_interpolation: true,
        autoplay: true,
    });

    // read dotlottie in to vec<u8>
    let mut f = File::open("src/cartoon.json").expect("no file found");
    let metadata = fs::metadata("src/cartoon.json").expect("unable to read metadata");

    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    let string = String::from_utf8(buffer.clone()).unwrap();
    lottie_player.load_animation_data(string.as_str(), WIDTH as u32, HEIGHT as u32);
    println!("{:?}", Some(lottie_player.manifest()));

    // lottie_player.load_dotlottie_data(&buffer, WIDTH as u32, HEIGHT as u32);
    // lottie_player.load_animation("confused", WIDTH as u32, HEIGHT as u32);

    let mut timer = Timer::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        timer.tick(&mut lottie_player);

        // if window.is_key_down(Key::Space) {
        //     lottie_player.next_animation(WIDTH as u32, HEIGHT as u32);
        // }

        let (buffer_ptr, buffer_len) = (lottie_player.buffer_ptr(), lottie_player.buffer_len());

        let buffer =
            unsafe { std::slice::from_raw_parts(buffer_ptr as *const u32, buffer_len as usize) };

        window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
    }
}
