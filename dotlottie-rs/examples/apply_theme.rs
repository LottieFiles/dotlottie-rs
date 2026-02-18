#![allow(clippy::print_stdout)]

/// Theme Example
///
/// This example demonstrates how to use the `set_theme` API to switch between
/// predefined themes in a .lottie file. The multi_themes.lottie file contains
/// several themes: dark, sky, light, animated_light, animated_dark, and animated_sky.
///
/// Themes are a convenient way to bundle multiple slot changes together and switch
/// between different visual styles of the same animation.
use dotlottie_rs::{ColorSpace, Config, DotLottiePlayer};
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

    let mut player = DotLottiePlayer::new(
        Config {
            loop_animation: true,
            autoplay: true,
            ..Config::default()
        },
        0, // threads (0 = auto)
    );

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    player.set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888);

    let dotlottie_data = include_bytes!("../assets/animations/dotlottie/v2/multi_themes.lottie");

    if !player.load_dotlottie_data(dotlottie_data, WIDTH, HEIGHT) {
        eprintln!("Failed to load .lottie file");
        return;
    }

    println!("Animation loaded successfully!");
    println!("Active animation ID: '{}'", player.active_animation_id());

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
    if player.set_theme(themes[current_theme_index]) {
        println!("✓ Theme set: {}", themes[current_theme_index]);
    } else {
        println!("✗ Failed to set theme: {}", themes[current_theme_index]);
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_down(Key::Space) {
            let now = std::time::Instant::now();
            if now.duration_since(last_space_press).as_millis() > 300 {
                current_theme_index = (current_theme_index + 1) % themes.len();

                if player.set_theme(themes[current_theme_index]) {
                    println!("✓ Theme set: {}", themes[current_theme_index]);
                } else {
                    println!("✗ Failed to set theme: {}", themes[current_theme_index]);
                }

                last_space_press = now;
            }
        }

        if player.tick() {
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!();
    println!("Example finished!");
}
