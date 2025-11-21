/// Theme Example
///
/// This example demonstrates how to use the `set_theme` API to switch between
/// predefined themes in a .lottie file. The multi_themes.lottie file contains
/// several themes: dark, sky, light, animated_light, animated_dark, and animated_sky.
///
/// Themes are a convenient way to bundle multiple slot changes together and switch
/// between different visual styles of the same animation.

use dotlottie_rs::{Config, DotLottiePlayer};
use minifb::{Key, Window, WindowOptions};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

fn main() {
    // Create window
    let mut window = Window::new(
        "Theme Example - Press SPACE to cycle themes",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    // Create player and load .lottie file
    let mut player = DotLottiePlayer::new(Config {
        loop_animation: true,
        autoplay: true,
        ..Config::default()
    });

    let dotlottie_data = include_bytes!("../../demo-player/src/v2/multi_themes.lottie");

    if !player.load_dotlottie_data(dotlottie_data, WIDTH, HEIGHT) {
        eprintln!("Failed to load .lottie file");
        return;
    }

    println!("Animation loaded successfully!");

    // Debug: Check active animation ID
    println!("Active animation ID: '{}'", player.active_animation_id());

    println!("Press SPACE to cycle through different themes");
    println!("Press ESC to quit");
    println!();

    // Available themes from the multi_themes.lottie file
    let themes = vec![
        "light",
        "dark",
        "sky",
        "animated_light",
        "animated_dark",
        "animated_sky",
    ];

    let mut current_theme_index = 0;
    let mut last_space_press = std::time::Instant::now();

    // Set initial theme
    println!("Attempting to set theme: '{}'", themes[current_theme_index]);
    if player.set_theme(themes[current_theme_index]) {
        println!("✓ Theme set: {}", themes[current_theme_index]);
    } else {
        println!("✗ Failed to set theme: {}", themes[current_theme_index]);
    }

    // Main render loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Handle theme cycling with SPACE key
        if window.is_key_down(Key::Space) {
            let now = std::time::Instant::now();
            if now.duration_since(last_space_press).as_millis() > 300 {
                current_theme_index = (current_theme_index + 1) % themes.len();

                // Switch to the new theme
                if player.set_theme(themes[current_theme_index]) {
                    println!("✓ Theme set: {}", themes[current_theme_index]);
                } else {
                    println!("✗ Failed to set theme: {}", themes[current_theme_index]);
                }

                last_space_press = now;
            }
        }

        // Update animation frame and render
        if player.tick() {
            // Get buffer as a slice
            let buffer_ptr = player.buffer();
            let buffer_len = player.buffer_len();
            let buffer = unsafe { std::slice::from_raw_parts(buffer_ptr, buffer_len as usize) };

            window
                .update_with_buffer(buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!();
    println!("Example finished!");
}
