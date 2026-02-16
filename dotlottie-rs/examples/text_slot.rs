#![allow(clippy::print_stdout)]

/// Text Slot Example
///
/// This example demonstrates how to use the `set_text_slot` API to dynamically
/// change text content, font, size, and color in a Lottie animation.
/// The text.json animation has a slot with ID "my_text" that we can modify.
///
/// Demonstrates both static and animated slot values.
use dotlottie_rs::{ColorSpace, DotLottiePlayer, TextDocument, TextKeyframe, TextSlot};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

fn main() {
    let mut window = Window::new(
        "Text Slot Example - Press T to toggle, SPACE to cycle",
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

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();

    let animation_data = include_str!("../assets/animations/lottie/text.json");
    let c_data = CString::new(animation_data).expect("CString conversion failed");

    if player.load_animation_data(&c_data, WIDTH, HEIGHT).is_err() {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Animation loaded successfully!");
    println!("Press T to toggle between static and animated modes");
    println!("Press SPACE to cycle through different messages (static mode)");
    println!("Press ESC to quit");

    // Define some text variations with colors [R, G, B, A]
    let texts = [
        ("Hello!", vec![1.0, 0.0, 0.0, 1.0]),  // Red
        ("World", vec![0.0, 1.0, 0.0, 1.0]),   // Green
        ("Slots!", vec![0.0, 0.0, 1.0, 1.0]),  // Blue
        ("Amazing", vec![1.0, 0.5, 0.0, 1.0]), // Orange
        ("Dynamic", vec![1.0, 0.0, 1.0, 1.0]), // Magenta
        ("Text", vec![0.0, 1.0, 1.0, 1.0]),    // Cyan
    ];

    let mut current_text_index = 0;
    let mut last_space_press = std::time::Instant::now();
    let mut last_toggle_press = std::time::Instant::now();
    let mut is_animated = false;

    let text_doc = TextDocument::new(texts[current_text_index].0)
        .with_font("Arial")
        .with_size(200.0)
        .with_fill_color(texts[current_text_index].1.clone());

    let text_slot = TextSlot::with_document(text_doc);
    let _ = player.set_text_slot("my_text", text_slot);
    println!(
        "Mode: STATIC | Current text: {}",
        texts[current_text_index].0
    );

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();

        if window.is_key_down(Key::T) && now.duration_since(last_toggle_press).as_millis() > 200 {
            is_animated = !is_animated;

            if is_animated {
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
                let _ = player.set_text_slot("my_text", text_slot);
                println!("Mode: ANIMATED (\"Hello\" -> \"World\")");
            } else {
                let text_doc = TextDocument::new(texts[current_text_index].0)
                    .with_font("Arial")
                    .with_size(200.0)
                    .with_fill_color(texts[current_text_index].1.clone());

                let text_slot = TextSlot::with_document(text_doc);
                let _ = player.set_text_slot("my_text", text_slot);
                println!(
                    "Mode: STATIC | Current text: {}",
                    texts[current_text_index].0
                );
            }

            last_toggle_press = now;
        }

        // Handle text cycling with SPACE key (only in static mode)
        if !is_animated
            && window.is_key_down(Key::Space)
            && now.duration_since(last_space_press).as_millis() > 200
        {
            current_text_index = (current_text_index + 1) % texts.len();

            let text_doc = TextDocument::new(texts[current_text_index].0)
                .with_font("Arial")
                .with_size(200.0)
                .with_fill_color(texts[current_text_index].1.clone());

            let text_slot = TextSlot::with_document(text_doc);
            let _ = player.set_text_slot("my_text", text_slot);

            println!(
                "Mode: STATIC | Current text: {}",
                texts[current_text_index].0
            );
            last_space_press = now;
        }

        if player.tick().is_ok() {
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!("Example finished!");
}
