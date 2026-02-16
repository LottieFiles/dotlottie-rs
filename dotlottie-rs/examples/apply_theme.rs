#![allow(clippy::print_stdout)]

use std::ffi::CString;

/// Theme Example
///
/// This example demonstrates how to use the `set_theme` API to switch between
/// predefined themes in a .lottie file. The multi_themes.lottie file contains
/// several themes: dark, sky, light, animated_light, animated_dark, and animated_sky.
///
/// Themes are a convenient way to bundle multiple slot changes together and switch
/// between different visual styles of the same animation.
use dotlottie_rs::{ColorSpace, DotLottiePlayer};
use minifb::{Key, Window, WindowOptions};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

fn main() {
    let mut window = Window::new(
        "Theme Example - Press SPACE to cycle themes",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    // Create player and load .lottie file
    let mut player = DotLottiePlayer::new();
    player.set_loop(false);
    player.set_autoplay(true);

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();

    let dotlottie_data = include_bytes!("../assets/animations/dotlottie/v2/multi_themes.lottie");

    if player
        .load_dotlottie_data(dotlottie_data, WIDTH, HEIGHT)
        .is_err()
    {
        eprintln!("Failed to load .lottie file");
        return;
    }

    println!("Animation loaded successfully!");
    println!("Animation ID: '{:?}'", player.animation_id());

    println!("Press SPACE to cycle through different themes");
    println!("Press ESC to quit");
    println!();

    let themes = [
        "light",
        "dark",
        "sky",
        "animated_light",
        "animated_dark",
        "animated_sky",
    ];

    let mut current_theme_index = 0;
    let mut last_space_press = std::time::Instant::now();

    println!("Attempting to set theme: '{}'", themes[current_theme_index]);
    let current_theme =
        CString::new(themes[current_theme_index]).expect("Failed to create CString");
    if player.set_theme(&current_theme).is_ok() {
        println!("✓ Theme set: {}", themes[current_theme_index]);
    } else {
        println!("✗ Failed to set theme: {}", themes[current_theme_index]);
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_down(Key::Space) {
            let now = std::time::Instant::now();
            if now.duration_since(last_space_press).as_millis() > 300 {
                current_theme_index = (current_theme_index + 1) % themes.len();

                // Switch to the new theme
                let new_theme =
                    CString::new(themes[current_theme_index]).expect("Failed to create CString");
                if player.set_theme(&new_theme).is_ok() {
                    println!("✓ Theme set: {}", themes[current_theme_index]);
                } else {
                    println!("✗ Failed to set theme: {}", themes[current_theme_index]);
                }

                last_space_press = now;
            }
        }

        // Update animation frame and render
        if player.tick().is_ok() {
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!();
    println!("Example finished!");
}
