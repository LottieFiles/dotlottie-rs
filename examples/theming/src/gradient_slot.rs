/// Gradient Slot Example
///
/// This example demonstrates how to use the `set_gradient_slot` API to dynamically
/// change gradient fills in a Lottie animation. The gradient.json animation has
/// a slot with ID "gradient_fill" that we can modify.

use dotlottie_rs::{Config, DotLottiePlayer, GradientSlot, GradientStop};
use minifb::{Key, Window, WindowOptions};

const WIDTH: u32 = 550;
const HEIGHT: u32 = 550;

fn main() {
    // Create window
    let mut window = Window::new(
        "Gradient Slot Example - Press SPACE to cycle gradients",
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

    let animation_data = include_str!("../../demo-player/src/v2/gradient.json");

    if !player.load_animation_data(animation_data, WIDTH, HEIGHT) {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Animation loaded successfully!");
    println!("Press SPACE to cycle through different gradients");
    println!("Press ESC to quit");

    // Define some gradient presets
    // Each gradient is a vector of GradientStop { offset, color: [R, G, B, A] }
    let gradients = vec![
        (
            "Sunset",
            vec![
                GradientStop { offset: 0.0, color: [1.0, 0.4, 0.0, 1.0] },  // Orange
                GradientStop { offset: 0.5, color: [1.0, 0.6, 0.2, 1.0] },  // Light Orange
                GradientStop { offset: 1.0, color: [1.0, 0.8, 0.0, 1.0] },  // Yellow
            ],
        ),
        (
            "Ocean",
            vec![
                GradientStop { offset: 0.0, color: [0.0, 0.3, 0.6, 1.0] },  // Deep Blue
                GradientStop { offset: 0.5, color: [0.0, 0.5, 0.8, 1.0] },  // Ocean Blue
                GradientStop { offset: 1.0, color: [0.0, 0.8, 1.0, 1.0] },  // Light Blue
            ],
        ),
        (
            "Forest",
            vec![
                GradientStop { offset: 0.0, color: [0.1, 0.3, 0.1, 1.0] },  // Dark Green
                GradientStop { offset: 0.5, color: [0.2, 0.6, 0.2, 1.0] },  // Green
                GradientStop { offset: 1.0, color: [0.5, 0.8, 0.3, 1.0] },  // Light Green
            ],
        ),
        (
            "Fire",
            vec![
                GradientStop { offset: 0.0, color: [1.0, 0.0, 0.0, 1.0] },  // Red
                GradientStop { offset: 0.33, color: [1.0, 0.5, 0.0, 1.0] }, // Orange
                GradientStop { offset: 0.66, color: [1.0, 1.0, 0.0, 1.0] }, // Yellow
                GradientStop { offset: 1.0, color: [1.0, 1.0, 1.0, 1.0] },  // White
            ],
        ),
        (
            "Purple Haze",
            vec![
                GradientStop { offset: 0.0, color: [0.3, 0.0, 0.5, 1.0] },  // Deep Purple
                GradientStop { offset: 0.5, color: [0.6, 0.2, 0.8, 1.0] },  // Purple
                GradientStop { offset: 1.0, color: [1.0, 0.4, 1.0, 1.0] },  // Pink
            ],
        ),
        (
            "Grayscale",
            vec![
                GradientStop { offset: 0.0, color: [0.0, 0.0, 0.0, 1.0] },  // Black
                GradientStop { offset: 0.5, color: [0.5, 0.5, 0.5, 1.0] },  // Gray
                GradientStop { offset: 1.0, color: [1.0, 1.0, 1.0, 1.0] },  // White
            ],
        ),
    ];

    let mut current_gradient_index = 0;
    let mut last_space_press = std::time::Instant::now();

    // Set initial gradient
    let gradient_slot = GradientSlot::new(gradients[current_gradient_index].1.clone());
    player.set_gradient_slot("gradient_fill", gradient_slot);
    println!("Current gradient: {}", gradients[current_gradient_index].0);

    // Main render loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Handle gradient cycling with SPACE key
        if window.is_key_down(Key::Space) {
            let now = std::time::Instant::now();
            if now.duration_since(last_space_press).as_millis() > 200 {
                current_gradient_index = (current_gradient_index + 1) % gradients.len();

                // Create and set the new gradient slot
                let gradient_slot = GradientSlot::new(gradients[current_gradient_index].1.clone());
                player.set_gradient_slot("gradient_fill", gradient_slot);

                println!("Current gradient: {}", gradients[current_gradient_index].0);
                last_space_press = now;
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
