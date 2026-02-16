#![allow(clippy::print_stdout)]

/// Color Slot Example
///
/// This example demonstrates how to use the `set_color_slot` API to dynamically
/// change colors in a Lottie animation. The bouncy_ball.json animation has a
/// slot with ID "ball_color" that we can modify.
///
/// Demonstrates both static and animated slot values.
use dotlottie_rs::{ColorSlot, ColorSpace, DotLottiePlayer, LottieKeyframe};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

fn main() {
    let mut window = Window::new(
        "Color Slot Example - Press T to toggle, SPACE to cycle",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    // Create player and load animation
    let mut player = DotLottiePlayer::new(0);
    player.set_autoplay(true);
    player.set_loop(true);

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888S)
        .unwrap();

    let animation_data = include_str!("../assets/animations/lottie/bouncy_ball.json");

    let c_data = CString::new(animation_data).expect("CString conversion failed");

    if !player.load_animation_data(&c_data, WIDTH, HEIGHT).is_ok() {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Animation loaded successfully!");
    println!("Press T to toggle between static and animated modes");
    println!("Press SPACE to cycle through different colors (static mode)");
    println!("Press ESC to quit");

    let colors = [
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

    let color_slot = ColorSlot::new(colors[current_color_index].0);
    let _ = player.set_color_slot("ball_color", color_slot);
    println!(
        "Mode: STATIC | Current color: {}",
        colors[current_color_index].1
    );

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();

        if window.is_key_down(Key::T) && now.duration_since(last_toggle_press).as_millis() > 200 {
            is_animated = !is_animated;

            if is_animated {
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
                let _ = player.set_color_slot("ball_color", color_slot);
                println!("Mode: ANIMATED (Red -> Blue)");
            } else {
                let color_slot = ColorSlot::new(colors[current_color_index].0);
                let _ = player.set_color_slot("ball_color", color_slot);
                println!(
                    "Mode: STATIC | Current color: {}",
                    colors[current_color_index].1
                );
            }

            last_toggle_press = now;
        }

        if !is_animated
            && window.is_key_down(Key::Space)
            && now.duration_since(last_space_press).as_millis() > 200
        {
            current_color_index = (current_color_index + 1) % colors.len();

            let color_slot = ColorSlot::new(colors[current_color_index].0);
            let _ = player.set_color_slot("ball_color", color_slot);

            println!(
                "Mode: STATIC | Current color: {}",
                colors[current_color_index].1
            );
            last_space_press = now;
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
