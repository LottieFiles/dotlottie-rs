/// Color Slot Example
///
/// This example demonstrates how to use the `set_color_slot` API to dynamically
/// change colors in a Lottie animation. The bouncy_ball.json animation has a
/// slot with ID "ball_color" that we can modify.
///
/// Demonstrates both static and animated slot values.

use dotlottie_rs::{ColorSlot, Config, DotLottiePlayer, LottieKeyframe};
use minifb::{Key, Window, WindowOptions};

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

fn main() {
    // Create window
    let mut window = Window::new(
        "Color Slot Example - Press T to toggle, SPACE to cycle",
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
    println!("Press SPACE to cycle through different colors (static mode)");
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
    let mut last_toggle_press = std::time::Instant::now();
    let mut is_animated = false;

    // Set initial color (static)
    let color_slot = ColorSlot::new(colors[current_color_index].0);
    player.set_color_slot("ball_color", color_slot);
    println!("Mode: STATIC | Current color: {}", colors[current_color_index].1);

    // Main render loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();

        // Handle toggle between static and animated with T key
        if window.is_key_down(Key::T) {
            if now.duration_since(last_toggle_press).as_millis() > 200 {
                is_animated = !is_animated;

                if is_animated {
                    // Create animated color slot: Red -> Blue (linear interpolation)
                    let color_slot = ColorSlot::with_keyframes(vec![
                        LottieKeyframe {
                            frame: 0,
                            start_value: [1.0, 0.0, 0.0], // Red
                            in_tangent: None,
                            out_tangent: None,
                            value_in_tangent: None,
                            value_out_tangent: None,
                            hold: None,
                        },
                        LottieKeyframe {
                            frame: 60,
                            start_value: [0.0, 0.0, 1.0], // Blue
                            in_tangent: None,
                            out_tangent: None,
                            value_in_tangent: None,
                            value_out_tangent: None,
                            hold: None,
                        },
                    ]);
                    player.set_color_slot("ball_color", color_slot);
                    println!("Mode: ANIMATED (Red -> Blue)");
                } else {
                    // Switch back to static mode
                    let color_slot = ColorSlot::new(colors[current_color_index].0);
                    player.set_color_slot("ball_color", color_slot);
                    println!("Mode: STATIC | Current color: {}", colors[current_color_index].1);
                }

                last_toggle_press = now;
            }
        }

        // Handle color cycling with SPACE key (only in static mode)
        if !is_animated && window.is_key_down(Key::Space) {
            if now.duration_since(last_space_press).as_millis() > 200 {
                current_color_index = (current_color_index + 1) % colors.len();

                // Create and set the new color slot
                let color_slot = ColorSlot::new(colors[current_color_index].0);
                player.set_color_slot("ball_color", color_slot);

                println!("Mode: STATIC | Current color: {}", colors[current_color_index].1);
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
