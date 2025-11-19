/// Expression Slot Example
///
/// This example demonstrates how to use expressions with slots to create
/// dynamic, time-based animations without keyframes.
///
/// Expressions use JavaScript syntax and MUST use `$bm_rt` to return the value:
///
/// ```javascript
/// var $bm_rt = time * 360;  // Rotate 360° per second
/// ```
///
/// Available variables:
/// - `time` - current time in seconds
/// - `value` - the property's base value
/// - `width`, `height` - composition dimensions
/// - Math functions: `Math.sin()`, `Math.cos()`, `Math.random()`, etc.
///
/// Reference: https://lottiefiles.github.io/lottie-docs/expressions/

use dotlottie_rs::{Config, DotLottiePlayer, ScalarSlot};
use minifb::{Key, Window, WindowOptions};

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

fn main() {
    // Create window
    let mut window = Window::new(
        "Expression Slot Example - Press 1-5 to change expressions",
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
    println!();
    println!("Expression Examples:");
    println!("  1 - Pulsing (sine wave)");
    println!("  2 - Fast pulse");
    println!("  3 - Bounce effect");
    println!("  4 - Random flicker");
    println!("  5 - Smooth oscillation");
    println!("  6 - Rotation based on time");
    println!("  0 - Static value (no expression)");
    println!();
    println!("Press ESC to quit");

    // Define expressions with descriptions
    // Note: Lottie expressions must use $bm_rt to return the calculated value
    let expressions = vec![
        (
            None,
            "Static (100%)",
            100.0,
        ),
        (
            Some("var $bm_rt = 50 + 50 * Math.sin(time * 2 * Math.PI);".to_string()),
            "Pulsing (sine wave, 1Hz)",
            100.0,
        ),
        (
            Some("var $bm_rt = 50 + 50 * Math.sin(time * 4 * Math.PI);".to_string()),
            "Fast pulse (2Hz)",
            100.0,
        ),
        (
            Some("var $bm_rt = Math.abs(Math.sin(time * Math.PI)) * 100;".to_string()),
            "Bounce effect",
            100.0,
        ),
        (
            Some("var $bm_rt = 50 + Math.random() * 50;".to_string()),
            "Random flicker",
            100.0,
        ),
        (
            Some("var $bm_rt = 50 + 45 * Math.cos(time * Math.PI);".to_string()),
            "Smooth oscillation (0.5Hz)",
            100.0,
        ),
        (
            Some("var $bm_rt = time * 360;".to_string()),
            "Rotation based on time (360°/sec)",
            0.0,
        ),
    ];

    let mut current_expression_index = 1; // Start with first expression
    let mut last_key_press = std::time::Instant::now();

    // Set initial expression
    let opacity_slot = if let Some(expr) = &expressions[current_expression_index].0 {
        ScalarSlot::new(expressions[current_expression_index].2)
            .with_expression(expr.clone())
    } else {
        ScalarSlot::new(expressions[current_expression_index].2)
    };
    player.set_scalar_slot("ball_opacity", opacity_slot);
    println!("Current: {} - {}",
        current_expression_index,
        expressions[current_expression_index].1
    );

    // Main render loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        let mut expression_changed = false;

        // Handle expression switching with number keys
        if now.duration_since(last_key_press).as_millis() > 200 {
            let new_index = if window.is_key_down(Key::Key0) {
                Some(0)
            } else if window.is_key_down(Key::Key1) {
                Some(1)
            } else if window.is_key_down(Key::Key2) {
                Some(2)
            } else if window.is_key_down(Key::Key3) {
                Some(3)
            } else if window.is_key_down(Key::Key4) {
                Some(4)
            } else if window.is_key_down(Key::Key5) {
                Some(5)
            } else if window.is_key_down(Key::Key6) {
                Some(6)
            } else {
                None
            };

            if let Some(idx) = new_index {
                if idx < expressions.len() && idx != current_expression_index {
                    current_expression_index = idx;
                    expression_changed = true;
                    last_key_press = now;
                }
            }
        }

        if expression_changed {
            // Create and set the new scalar slot with or without expression
            let opacity_slot = if let Some(expr) = &expressions[current_expression_index].0 {
                ScalarSlot::new(expressions[current_expression_index].2)
                    .with_expression(expr.clone())
            } else {
                ScalarSlot::new(expressions[current_expression_index].2)
            };

            player.set_scalar_slot("ball_opacity", opacity_slot);
            println!();
            println!("Current: {} - {}",
                current_expression_index,
                expressions[current_expression_index].1
            );
            if let Some(expr) = &expressions[current_expression_index].0 {
                println!("Expression: {}", expr);
            }
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
