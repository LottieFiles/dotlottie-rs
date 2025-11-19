/// Color Slot Example
///
/// This example demonstrates how to use the `set_color_slot` API to dynamically
/// change colors in a Lottie animation. The bouncy_ball.json animation has a
/// slot with ID "ball_color" that we can modify.

use dotlottie_rs::{ColorSlot, Config, DotLottiePlayer};
use minifb::{Key, Window, WindowOptions};

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

fn main() {
    // Create window
    let mut window = Window::new(
        "Color Slot Example - Press SPACE to cycle colors",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    // Create player and load animation
    let mut player = DotLottiePlayer::new(Config {
        loop_animation: true,
        autoplay: true,
        ..Config::default()
    });

    let animation_data = include_str!("../../demo-player/src/bouncy_ball.json");

    if !player.load_animation_data(animation_data, WIDTH, HEIGHT) {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Animation loaded successfully!");
    println!("Press SPACE to cycle through different colors");
    println!("Press ESC to quit");

    // Define some colors to cycle through [R, G, B] (normalized 0.0-1.0)
    let colors = vec![
        ([1.0, 0.0, 0.0], "Red"),
        ([0.0, 1.0, 0.0], "Green"),
        ([0.0, 0.0, 1.0], "Blue"),
        ([1.0, 1.0, 0.0], "Yellow"),
        ([1.0, 0.0, 1.0], "Magenta"),
        ([0.0, 1.0, 1.0], "Cyan"),
        ([1.0, 0.5, 0.0], "Orange"),
        ([0.5, 0.0, 1.0], "Purple"),
    ];

    let mut current_color_index = 0;
    let mut last_space_press = std::time::Instant::now();

    // Set initial color
    let color_slot = ColorSlot::new(colors[current_color_index].0);
    player.set_color_slot("ball_color", color_slot);
    println!("Current color: {}", colors[current_color_index].1);

    // Main render loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Handle color cycling with SPACE key
        if window.is_key_down(Key::Space) {
            let now = std::time::Instant::now();
            if now.duration_since(last_space_press).as_millis() > 200 {
                current_color_index = (current_color_index + 1) % colors.len();

                // Create and set the new color slot
                let color_slot = ColorSlot::new(colors[current_color_index].0);
                player.set_color_slot("ball_color", color_slot);

                println!("Current color: {}", colors[current_color_index].1);
                last_space_press = now;
            }
        }

        // Update animation frame and render
        if player.tick() {
            player.render();

            // Get buffer as a slice
            let buffer_ptr = player.buffer();
            let buffer_len = player.buffer_len();
            let buffer = unsafe { std::slice::from_raw_parts(buffer_ptr, buffer_len as usize) };

            window
                .update_with_buffer(buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!("Example finished!");
}
