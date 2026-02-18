#![allow(clippy::print_stdout)]

/// Position Slot Example
///
/// This example demonstrates how to use the `set_position_slot` API to dynamically
/// change 2D position values in a Lottie animation.
/// The bouncy_ball.json animation has a slot with ID "ball_position" that we can modify.
///
/// Position slots support spatial tangents for curved motion paths.
/// Demonstrates both static and animated slot values.
use dotlottie_rs::{ColorSpace, DotLottiePlayer, LottieKeyframe, LottieProperty};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

fn main() {
    let mut window = Window::new(
        "Position Slot Example - Press T to toggle, arrows to move",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    // Create player and load animation
    let mut player = DotLottiePlayer::new();
    player.set_loop(true);
    player.set_autoplay(true);

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();

    let animation_data = include_str!("../assets/animations/lottie/bouncy_ball.json");

    let c_data = CString::new(animation_data).expect("CString conversion failed");

    if player.load_animation_data(&c_data, WIDTH, HEIGHT).is_err() {
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

    let position_slot = LottieProperty::static_value([pos_x, pos_y]);
    let _ = player.set_position_slot("ball_position", position_slot);
    println!("Mode: STATIC | Current position: X={pos_x:.0}, Y={pos_y:.0}");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();

        if window.is_key_down(Key::T) && now.duration_since(last_toggle_press).as_millis() > 200 {
            is_animated = !is_animated;

            if is_animated {
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
                let _ = player.set_position_slot("ball_position", position_slot);
                println!("Mode: ANIMATED ([100, 100] -> [400, 400])");
            } else {
                // Switch back to static mode
                let position_slot = LottieProperty::static_value([pos_x, pos_y]);
                let _ = player.set_position_slot("ball_position", position_slot);
                println!("Mode: STATIC | Current position: X={pos_x:.0}, Y={pos_y:.0}");
            }

            last_toggle_press = now;
        }

        let mut position_changed = false;

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
            let position_slot = LottieProperty::static_value([pos_x, pos_y]);
            let _ = player.set_position_slot("ball_position", position_slot);
            println!("Mode: STATIC | Current position: X={pos_x:.0}, Y={pos_y:.0}");
        }

        // Update animation frame and render
        if player.tick().is_ok() {
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!("Example finished!");
}
