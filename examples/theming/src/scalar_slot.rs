/// Scalar Slot Example
///
/// This example demonstrates how to use the `set_scalar_slot` API to dynamically
/// change scalar values (like opacity, rotation, scale) in a Lottie animation.
/// The bouncy_ball.json animation has a slot with ID "ball_opacity" that we can modify.
///
/// Note: Opacity values in Lottie are typically in the range 0-100 (percentage).
/// Demonstrates both static and animated slot values.

use dotlottie_rs::{Config, DotLottiePlayer, LottieKeyframe, ScalarSlot};
use minifb::{Key, Window, WindowOptions};

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

fn main() {
    // Create window
    let mut window = Window::new(
        "Scalar Slot Example - Press T to toggle, UP/DOWN to adjust",
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
    println!("Press UP/DOWN arrows to adjust opacity (static mode)");
    println!("Press ESC to quit");

    let mut opacity = 100.0; // Start at 100% opacity
    let mut last_key_press = std::time::Instant::now();
    let mut last_toggle_press = std::time::Instant::now();
    let mut is_animated = false;

    // Set initial opacity (static)
    let opacity_slot = ScalarSlot::new(opacity);
    player.set_scalar_slot("ball_opacity", opacity_slot);
    println!("Mode: STATIC | Current opacity: {:.0}%", opacity);

    // Main render loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();

        // Handle toggle between static and animated with T key
        if window.is_key_down(Key::T) {
            if now.duration_since(last_toggle_press).as_millis() > 200 {
                is_animated = !is_animated;

                if is_animated {
                    // Create animated opacity slot: 100% -> 20% (linear interpolation)
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
                    player.set_scalar_slot("ball_opacity", opacity_slot);
                    println!("Mode: ANIMATED (100% -> 20%)");
                } else {
                    // Switch back to static mode
                    let opacity_slot = ScalarSlot::new(opacity);
                    player.set_scalar_slot("ball_opacity", opacity_slot);
                    println!("Mode: STATIC | Current opacity: {:.0}%", opacity);
                }

                last_toggle_press = now;
            }
        }

        let mut opacity_changed = false;

        // Handle opacity adjustment with UP/DOWN keys (only in static mode)
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
            // Create and set the new scalar slot
            let opacity_slot = ScalarSlot::new(opacity);
            player.set_scalar_slot("ball_opacity", opacity_slot);
            println!("Mode: STATIC | Current opacity: {:.0}%", opacity);
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
