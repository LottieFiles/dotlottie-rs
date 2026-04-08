#![allow(clippy::print_stdout)]

use dotlottie_rs::{ColorSpace, DotLottiePlayer, Rgba};
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
            let _ = player.load_dotlottie_data(&buffer);
        }
        "json" => {
            let data = fs::read_to_string(path).expect("Could not read JSON file");
            let c_data = CString::new(data).expect("CString conversion failed");
            let _ = player.load_animation_data(&c_data);
        }
        _ => {}
    }

    let _ = player.play();

    println!(
        "Loaded: {}",
        path.file_name().unwrap_or_default().to_string_lossy()
    );
}

const BACKGROUNDS: &[(Key, Rgba, &str)] = &[
    (Key::Key0, Rgba::TRANSPARENT, "transparent"),
    (Key::Key1, Rgba::new(255, 255, 255, 255), "white"),
    (Key::Key2, Rgba::new(0, 0, 0, 255), "black"),
    (Key::Key3, Rgba::new(255, 59, 48, 255), "red"),
    (Key::Key4, Rgba::new(52, 199, 89, 255), "green"),
    (Key::Key5, Rgba::new(0, 122, 255, 255), "blue"),
    (Key::Key6, Rgba::new(255, 204, 0, 255), "yellow"),
    (Key::Key7, Rgba::new(175, 82, 222, 255), "purple"),
];

fn main() {
    let mut window = Window::new(
        "dotLottie Player - Left/Right: animation, 0-7: background, ESC: exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .expect("Failed to create window");

    let mut player = DotLottiePlayer::new();
    player.set_autoplay(true);
    player.set_loop(true);

    let mut current_width = WIDTH;
    let mut current_height = HEIGHT;
    let mut buffer: Vec<u32> = vec![0; current_width * current_height];

    if player
        .set_sw_target(
            &mut buffer,
            current_width as u32,
            current_height as u32,
            ColorSpace::ARGB8888,
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

        for &(key, color, name) in BACKGROUNDS {
            if window.is_key_pressed(key, minifb::KeyRepeat::No) {
                let _ = player.set_background(color);
                println!("Background: {name}");
            }
        }

        let (new_width, new_height) = window.get_size();
        if new_width != current_width || new_height != current_height {
            current_width = new_width;
            current_height = new_height;
            buffer = vec![0; current_width * current_height];
            if player
                .set_sw_target(
                    &mut buffer,
                    current_width as u32,
                    current_height as u32,
                    ColorSpace::ARGB8888,
                )
                .is_err()
            {
                eprintln!("Failed to resize software rendering target");
            }
        }

        if player.tick().is_ok() {
            window
                .update_with_buffer(&buffer, current_width, current_height)
                .unwrap();
        }
    }
}
