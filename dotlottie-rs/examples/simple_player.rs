#![allow(clippy::print_stdout)]

use dotlottie_rs::{ColorSpace, DotLottiePlayer};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

fn get_animation_files() -> Vec<PathBuf> {
    let animations_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/animations");
    let dir =
        fs::read_dir(animations_dir.join("dotlottie/v2")).expect("Could not read animations dir");

    let mut files: Vec<PathBuf> = dir
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .map(|ext| ext == "lottie" || ext == "json")
                .unwrap_or(false)
        })
        .collect();

    files.sort();
    files
}

fn load_animation(player: &mut DotLottiePlayer, path: &PathBuf) {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match extension {
        "lottie" => {
            let mut file = File::open(path).expect("Could not open file");
            let metadata = fs::metadata(path).expect("Could not read metadata");
            let mut buffer = vec![0; metadata.len() as usize];
            file.read_exact(&mut buffer).expect("Buffer overflow");
            let _ = player.load_dotlottie_data(&buffer, WIDTH as u32, HEIGHT as u32);
        }
        "json" => {
            let data = fs::read_to_string(path).expect("Could not read JSON file");
            let c_data = CString::new(data).expect("CString conversion failed");
            let _ = player.load_animation_data(&c_data, WIDTH as u32, HEIGHT as u32);
        }
        _ => {}
    }

    let _ = player.play();

    println!(
        "Loaded: {}",
        path.file_name().unwrap_or_default().to_string_lossy()
    );
}

fn main() {
    let mut window = Window::new(
        "dotLottie Player - Left/Right to change, ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    let mut player = DotLottiePlayer::new();
    player.set_autoplay(true);
    player.set_loop(true);
    let _ = player.set_background_color(Some(0xffffffff));

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    if player
        .set_sw_target(
            &mut buffer,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ABGR8888,
        )
        .is_err()
    {
        panic!("Failed to set software rendering target");
    }

    let animations = get_animation_files();
    if animations.is_empty() {
        panic!("No animations found in assets/animations/dotlottie/v2/");
    }

    let mut current_index: usize = 0;
    load_animation(&mut player, &animations[current_index]);

    let mut left_was_down = false;
    let mut right_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let left_is_down = window.is_key_down(Key::Left);
        let right_is_down = window.is_key_down(Key::Right);

        if left_was_down && !left_is_down {
            current_index = if current_index == 0 {
                animations.len() - 1
            } else {
                current_index - 1
            };
            load_animation(&mut player, &animations[current_index]);
        }

        if right_was_down && !right_is_down {
            current_index = (current_index + 1) % animations.len();
            load_animation(&mut player, &animations[current_index]);
        }

        left_was_down = left_is_down;
        right_was_down = right_is_down;

        if player.tick().is_ok() {
            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        }
    }
}
