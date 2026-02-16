#![allow(clippy::print_stdout)]

/// Vector Slot Example
///
/// This example demonstrates how to use the `set_vector_slot` API to dynamically
/// change 2D vector values (like scale, size) in a Lottie animation.
/// The bouncy_ball.json animation has a slot with ID "ball_scale" that we can modify.
///
/// Vector slots are for 2D properties like scale [x, y] without spatial tangents.
/// Demonstrates both static and animated slot values.
use dotlottie_rs::{ColorSpace, DotLottiePlayer, LottieKeyframe, LottieProperty};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

fn main() {
    let mut window = Window::new(
        "Vector Slot Example - Press T to toggle, arrows to adjust",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut player = DotLottiePlayer::new();

    player.set_autoplay(true);
    player.set_loop(true);

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
    println!("Press UP/DOWN arrows to scale uniformly (static mode)");
    println!("Press LEFT/RIGHT arrows to adjust X scale only (static mode)");
    println!("Press ESC to quit");

    let mut scale_x = 100.0; // Start at 100% scale
    let mut scale_y = 100.0;
    let mut last_key_press = std::time::Instant::now();
    let mut last_toggle_press = std::time::Instant::now();
    let mut is_animated = false;

    let scale_slot = LottieProperty::static_value([scale_x, scale_y]);
    let _ = player.set_vector_slot("ball_scale", scale_slot);
    println!("Mode: STATIC | Current scale: X={scale_x:.0}%, Y={scale_y:.0}%");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();

        if window.is_key_down(Key::T) && now.duration_since(last_toggle_press).as_millis() > 200 {
            is_animated = !is_animated;
            if is_animated {
                let scale_slot = LottieProperty::animated(vec![
                    LottieKeyframe {
                        frame: 0,
                        start_value: [50.0, 50.0],
                        in_tangent: None,
                        out_tangent: None,
                        value_in_tangent: None,
                        value_out_tangent: None,
                        hold: None,
                    },
                    LottieKeyframe {
                        frame: 60,
                        start_value: [150.0, 150.0],
                        in_tangent: None,
                        out_tangent: None,
                        value_in_tangent: None,
                        value_out_tangent: None,
                        hold: None,
                    },
                ]);
                let _ = player.set_vector_slot("ball_scale", scale_slot);
                println!("Mode: ANIMATED ([50%, 50%] -> [150%, 150%])");
            } else {
                // Switch back to static mode
                let scale_slot = LottieProperty::static_value([scale_x, scale_y]);
                let _ = player.set_vector_slot("ball_scale", scale_slot);
                println!("Mode: STATIC | Current scale: X={scale_x:.0}%, Y={scale_y:.0}%");
            }

            last_toggle_press = now;
        }

        let mut scale_changed = false;

        if !is_animated && now.duration_since(last_key_press).as_millis() > 50 {
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
            let scale_slot = LottieProperty::static_value([scale_x, scale_y]);
            let _ = player.set_vector_slot("ball_scale", scale_slot);
            println!("Mode: STATIC | Current scale: X={scale_x:.0}%, Y={scale_y:.0}%");
        }

        if player.tick().is_ok() {
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!("Example finished!");
}
