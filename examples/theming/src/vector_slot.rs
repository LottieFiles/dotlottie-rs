/// Vector Slot Example
///
/// This example demonstrates how to use the `set_vector_slot` API to dynamically
/// change 2D vector values (like scale, size) in a Lottie animation.
/// The bouncy_ball.json animation has a slot with ID "ball_scale" that we can modify.
///
/// Vector slots are for 2D properties like scale [x, y] without spatial tangents.

use dotlottie_rs::{Config, DotLottiePlayer, LottieProperty};
use minifb::{Key, Window, WindowOptions};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

fn main() {
    // Create window
    let mut window = Window::new(
        "Vector Slot Example - Press UP/DOWN to scale uniformly, LEFT/RIGHT for X/Y",
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
    println!("Press UP/DOWN arrows to scale uniformly (both X and Y)");
    println!("Press LEFT/RIGHT arrows to adjust X scale only");
    println!("Press ESC to quit");

    let mut scale_x = 100.0; // Start at 100% scale
    let mut scale_y = 100.0;
    let mut last_key_press = std::time::Instant::now();

    // Set initial scale
    let scale_slot = LottieProperty::static_value([scale_x, scale_y]);
    player.set_vector_slot("ball_scale", scale_slot);
    println!("Current scale: X={:.0}%, Y={:.0}%", scale_x, scale_y);

    // Main render loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        let mut scale_changed = false;

        // Handle scale adjustment with arrow keys
        if now.duration_since(last_key_press).as_millis() > 50 {
            if window.is_key_down(Key::Up) {
                // Scale up uniformly
                scale_x = (scale_x + 5.0_f32).min(200.0);
                scale_y = (scale_y + 5.0_f32).min(200.0);
                scale_changed = true;
                last_key_press = now;
            } else if window.is_key_down(Key::Down) {
                // Scale down uniformly
                scale_x = (scale_x - 5.0_f32).max(10.0);
                scale_y = (scale_y - 5.0_f32).max(10.0);
                scale_changed = true;
                last_key_press = now;
            } else if window.is_key_down(Key::Right) {
                // Increase X scale
                scale_x = (scale_x + 5.0_f32).min(200.0);
                scale_changed = true;
                last_key_press = now;
            } else if window.is_key_down(Key::Left) {
                // Decrease X scale
                scale_x = (scale_x - 5.0_f32).max(10.0);
                scale_changed = true;
                last_key_press = now;
            }
        }

        if scale_changed {
            // Create and set the new vector slot
            let scale_slot = LottieProperty::static_value([scale_x, scale_y]);
            player.set_vector_slot("ball_scale", scale_slot);
            println!("Current scale: X={:.0}%, Y={:.0}%", scale_x, scale_y);
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

    println!("Example finished!");
}
