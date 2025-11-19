/// Position Slot Example
///
/// This example demonstrates how to use the `set_position_slot` API to dynamically
/// change 2D position values in a Lottie animation.
/// The bouncy_ball.json animation has a slot with ID "ball_position" that we can modify.
///
/// Position slots support spatial tangents for curved motion paths.
/// Demonstrates both static and animated slot values.

use dotlottie_rs::{Config, DotLottiePlayer, LottieKeyframe, LottieProperty};
use minifb::{Key, Window, WindowOptions};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

fn main() {
    // Create window
    let mut window = Window::new(
        "Position Slot Example - Press T to toggle, arrows to move",
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
    println!("Press T to toggle between static and animated modes");
    println!("Use arrow keys to move the ball position (static mode)");
    println!("Press ESC to quit");

    // Start at center position
    let mut pos_x = 250.0;
    let mut pos_y = 250.0;
    let mut last_key_press = std::time::Instant::now();
    let mut last_toggle_press = std::time::Instant::now();
    let mut is_animated = false;

    // Set initial position (static)
    let position_slot = LottieProperty::static_value([pos_x, pos_y]);
    player.set_position_slot("ball_position", position_slot);
    println!("Mode: STATIC | Current position: X={:.0}, Y={:.0}", pos_x, pos_y);

    // Main render loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();

        // Handle toggle between static and animated with T key
        if window.is_key_down(Key::T) {
            if now.duration_since(last_toggle_press).as_millis() > 200 {
                is_animated = !is_animated;

                if is_animated {
                    // Create animated position slot: top-left -> bottom-right (linear interpolation)
                    let position_slot = LottieProperty::animated(vec![
                        LottieKeyframe {
                            frame: 0,
                            start_value: [100.0, 100.0],
                            in_tangent: None,
                            out_tangent: None,
                            value_in_tangent: None,
                            value_out_tangent: None,
                            hold: None,
                        },
                        LottieKeyframe {
                            frame: 60,
                            start_value: [400.0, 400.0],
                            in_tangent: None,
                            out_tangent: None,
                            value_in_tangent: None,
                            value_out_tangent: None,
                            hold: None,
                        },
                    ]);
                    player.set_position_slot("ball_position", position_slot);
                    println!("Mode: ANIMATED ([100, 100] -> [400, 400])");
                } else {
                    // Switch back to static mode
                    let position_slot = LottieProperty::static_value([pos_x, pos_y]);
                    player.set_position_slot("ball_position", position_slot);
                    println!("Mode: STATIC | Current position: X={:.0}, Y={:.0}", pos_x, pos_y);
                }

                last_toggle_press = now;
            }
        }

        let mut position_changed = false;

        // Handle position adjustment with arrow keys (only in static mode)
        if !is_animated && now.duration_since(last_key_press).as_millis() > 16 {
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
            println!("Mode: STATIC | Current position: X={:.0}, Y={:.0}", pos_x, pos_y);
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
