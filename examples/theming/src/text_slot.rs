/// Text Slot Example
///
/// This example demonstrates how to use the `set_text_slot` API to dynamically
/// change text content, font, size, and color in a Lottie animation.
/// The text.json animation has a slot with ID "my_text" that we can modify.
///
/// Demonstrates both static and animated slot values.

use dotlottie_rs::{Config, DotLottiePlayer, TextDocument, TextKeyframe, TextSlot};
use minifb::{Key, Window, WindowOptions};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

fn main() {
    // Create window
    let mut window = Window::new(
        "Text Slot Example - Press T to toggle, SPACE to cycle",
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

    let animation_data = include_str!("../../demo-player/src/text.json");

    if !player.load_animation_data(animation_data, WIDTH, HEIGHT) {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Animation loaded successfully!");
    println!("Press T to toggle between static and animated modes");
    println!("Press SPACE to cycle through different messages (static mode)");
    println!("Press ESC to quit");

    // Define some text variations with colors [R, G, B, A]
    let texts = vec![
        ("Hello!", vec![1.0, 0.0, 0.0, 1.0]),      // Red
        ("World", vec![0.0, 1.0, 0.0, 1.0]),       // Green
        ("Slots!", vec![0.0, 0.0, 1.0, 1.0]),      // Blue
        ("Amazing", vec![1.0, 0.5, 0.0, 1.0]),     // Orange
        ("Dynamic", vec![1.0, 0.0, 1.0, 1.0]),     // Magenta
        ("Text", vec![0.0, 1.0, 1.0, 1.0]),        // Cyan
    ];

    let mut current_text_index = 0;
    let mut last_space_press = std::time::Instant::now();
    let mut last_toggle_press = std::time::Instant::now();
    let mut is_animated = false;

    // Set initial text (static)
    let text_doc = TextDocument::new(texts[current_text_index].0)
        .with_font("Arial")
        .with_size(200.0)
        .with_fill_color(texts[current_text_index].1.clone());

    let text_slot = TextSlot::with_document(text_doc);
    player.set_text_slot("my_text", text_slot);
    println!("Mode: STATIC | Current text: {}", texts[current_text_index].0);

    // Main render loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();

        // Handle toggle between static and animated with T key
        if window.is_key_down(Key::T) {
            if now.duration_since(last_toggle_press).as_millis() > 200 {
                is_animated = !is_animated;

                if is_animated {
                    // Create animated text slot: "Hello" -> "World"
                    let text_slot = TextSlot::with_keyframes(vec![
                        TextKeyframe {
                            frame: 0,
                            text_document: TextDocument::new("Hello")
                                .with_font("Arial")
                                .with_size(200.0)
                                .with_fill_color(vec![1.0, 0.0, 0.0, 1.0]), // Red
                        },
                        TextKeyframe {
                            frame: 30,
                            text_document: TextDocument::new("World")
                                .with_font("Arial")
                                .with_size(200.0)
                                .with_fill_color(vec![0.0, 0.0, 1.0, 1.0]), // Blue
                        },
                    ]);
                    player.set_text_slot("my_text", text_slot);
                    println!("Mode: ANIMATED (\"Hello\" -> \"World\")");
                } else {
                    // Switch back to static mode
                    let text_doc = TextDocument::new(texts[current_text_index].0)
                        .with_font("Arial")
                        .with_size(200.0)
                        .with_fill_color(texts[current_text_index].1.clone());

                    let text_slot = TextSlot::with_document(text_doc);
                    player.set_text_slot("my_text", text_slot);
                    println!("Mode: STATIC | Current text: {}", texts[current_text_index].0);
                }

                last_toggle_press = now;
            }
        }

        // Handle text cycling with SPACE key (only in static mode)
        if !is_animated && window.is_key_down(Key::Space) {
            if now.duration_since(last_space_press).as_millis() > 200 {
                current_text_index = (current_text_index + 1) % texts.len();

                // Create and set the new text slot with custom styling
                let text_doc = TextDocument::new(texts[current_text_index].0)
                    .with_font("Arial")
                    .with_size(200.0)
                    .with_fill_color(texts[current_text_index].1.clone());

                let text_slot = TextSlot::with_document(text_doc);
                player.set_text_slot("my_text", text_slot);

                println!("Mode: STATIC | Current text: {}", texts[current_text_index].0);
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
