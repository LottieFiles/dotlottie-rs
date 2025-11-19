/// Position Slot Example
///
/// This example demonstrates how to use the `set_position_slot` API to dynamically
/// change 2D position values in a Lottie animation.
/// The bouncy_ball.json animation has a slot with ID "ball_position" that we can modify.
///
/// Position slots support spatial tangents for curved motion paths, though this example
/// demonstrates static positioning. Use arrow keys to move the ball around.

use dotlottie_rs::{Config, DotLottiePlayer, LottieProperty};
use minifb::{Key, Window, WindowOptions};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

fn main() {
    // Create window
    let mut window = Window::new(
        "Position Slot Example - Use arrow keys to move the ball",
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
    println!("Use arrow keys to move the ball position");
    println!("Press ESC to quit");

    // Start at center position
    let mut pos_x = 250.0;
    let mut pos_y = 250.0;
    let mut last_key_press = std::time::Instant::now();

    // Set initial position
    let position_slot = LottieProperty::static_value([pos_x, pos_y]);
    player.set_position_slot("ball_position", position_slot);
    println!("Current position: X={:.0}, Y={:.0}", pos_x, pos_y);

    // Main render loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        let mut position_changed = false;

        // Handle position adjustment with arrow keys
        if now.duration_since(last_key_press).as_millis() > 16 {
            let move_speed = 5.0_f32;

            if window.is_key_down(Key::Up) {
                pos_y = (pos_y - move_speed).max(0.0_f32);
                position_changed = true;
                last_key_press = now;
            }
            if window.is_key_down(Key::Down) {
                pos_y = (pos_y + move_speed).min(HEIGHT as f32);
                position_changed = true;
                last_key_press = now;
            }
            if window.is_key_down(Key::Left) {
                pos_x = (pos_x - move_speed).max(0.0_f32);
                position_changed = true;
                last_key_press = now;
            }
            if window.is_key_down(Key::Right) {
                pos_x = (pos_x + move_speed).min(WIDTH as f32);
                position_changed = true;
                last_key_press = now;
            }
        }

        if position_changed {
            // Create and set the new position slot
            let position_slot = LottieProperty::static_value([pos_x, pos_y]);
            player.set_position_slot("ball_position", position_slot);
            println!("Current position: X={:.0}, Y={:.0}", pos_x, pos_y);
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
