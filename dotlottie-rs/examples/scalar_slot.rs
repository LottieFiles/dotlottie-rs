#![allow(clippy::print_stdout)]

/// Scalar Slot Example
///
/// This example demonstrates how to use the `set_scalar_slot` API to dynamically
/// change scalar values (like opacity, rotation, scale) in a Lottie animation.
/// The bouncy_ball.json animation has a slot with ID "ball_opacity" that we can modify.
///
/// Note: Opacity values in Lottie are typically in the range 0-100 (percentage).
/// Demonstrates both static and animated slot values.
use dotlottie_rs::{ColorSpace, Config, DotLottiePlayer, LottieKeyframe, ScalarSlot};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

fn main() {
    let mut window = Window::new(
        "Scalar Slot Example - Press T to toggle, UP/DOWN to adjust",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    // Create player and load animation
    let mut player = DotLottiePlayer::new(0);
    player.set_loop(true);
    player.set_autoplay(true);

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    player.set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888).unwrap();

    let animation_data = include_str!("../assets/animations/lottie/bouncy_ball.json");

    let c_data = CString::new(animation_data).expect("CString conversion failed");

    if !player.load_animation_data(&c_data, WIDTH, HEIGHT).is_ok() {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Animation loaded successfully!");
    println!("Press T to toggle between static and animated modes");
    println!("Press UP/DOWN arrows to adjust opacity (static mode)");
    println!("Press ESC to quit");

    let mut opacity = 100.0; // Start at 100% opacity
    let mut last_key_press = std::time::Instant::now();
    let mut last_toggle_press = std::time::Instant::now();
    let mut is_animated = false;

    let opacity_slot = ScalarSlot::new(opacity);
    let _ = player.set_scalar_slot("ball_opacity", opacity_slot);
    println!("Mode: STATIC | Current opacity: {opacity:.0}%");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();

        if window.is_key_down(Key::T) && now.duration_since(last_toggle_press).as_millis() > 200 {
            is_animated = !is_animated;

            if is_animated {
                let opacity_slot = ScalarSlot::with_keyframes(vec![
                    LottieKeyframe {
                        frame: 0,
                        start_value: 100.0,
                        in_tangent: None,
                        out_tangent: None,
                        value_in_tangent: None,
                        value_out_tangent: None,
                        hold: None,
                    },
                    LottieKeyframe {
                        frame: 60,
                        start_value: 20.0,
                        in_tangent: None,
                        out_tangent: None,
                        value_in_tangent: None,
                        value_out_tangent: None,
                        hold: None,
                    },
                ]);
                let _ = player.set_scalar_slot("ball_opacity", opacity_slot);
                println!("Mode: ANIMATED (100% -> 20%)");
            } else {
                // Switch back to static mode
                let opacity_slot = ScalarSlot::new(opacity);
                let _ = player.set_scalar_slot("ball_opacity", opacity_slot);
                println!("Mode: STATIC | Current opacity: {opacity:.0}%");
            }

            last_toggle_press = now;
        }

        let mut opacity_changed = false;

        if !is_animated && now.duration_since(last_key_press).as_millis() > 100 {
            if window.is_key_down(Key::Up) && opacity < 100.0 {
                opacity = (opacity + 5.0).min(100.0);
                opacity_changed = true;
                last_key_press = now;
            } else if window.is_key_down(Key::Down) && opacity > 0.0 {
                opacity = (opacity - 5.0).max(0.0);
                opacity_changed = true;
                last_key_press = now;
            }
        }

        if opacity_changed {
            let opacity_slot = ScalarSlot::new(opacity);
            let _ = player.set_scalar_slot("ball_opacity", opacity_slot);
            println!("Mode: STATIC | Current opacity: {opacity:.0}%");
        }

        if player.tick().is_ok() {
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!("Example finished!");
}
