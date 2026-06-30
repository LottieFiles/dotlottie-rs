#![allow(clippy::print_stdout)]

/// Image Slot Example
///
/// Demonstrates `set_image_slot` together with `ImageSlot::from_src`, the single
/// source-field resolution introduced by the theming redesign. The `image.json`
/// animation exposes an image slot with ID "sun_img"; pressing SPACE swaps the sun
/// for a replacement image and cycles back to the original.
///
/// `src` is resolved by its prefix:
///   - `data:` URI       -> embedded image
///   - `http(s)://` URL  -> remote image (fetched by the host/renderer)
///   - otherwise         -> a file in the package `i/` folder, referenced by name
///
/// This demo uses `data:` URIs so it stays self-contained; the package-file and
/// URL forms resolve through the same `from_src` call.
use dotlottie_rs::{ColorSpace, ImageSlot, Player};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;

mod common;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

/// Slot ID declared on the image asset (`sid`) in image.json.
const SLOT_ID: &str = "sun_img";
/// The original sun asset is 1362x1362; match it so replacements fill the same box.
const SLOT_W: u32 = 1362;
const SLOT_H: u32 = 1362;

// Tiny 24x24 solid-color PNGs, encoded as data URIs for this demo.
const MAGENTA: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAIAAABvFaqvAAAAH0lEQVR42mO4pjGNKohh1KBRg0YNGjVo1KBRgwbeIADJIY0uOOudVwAAAABJRU5ErkJggg==";
const TEAL: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAIAAABvFaqvAAAAHklEQVR42mNgWLGCOmjUoFGDRg0aNWjUoFGDBt4gABRe9B/QJEtQAAAAAElFTkSuQmCC";
const AMBER: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAIAAABvFaqvAAAAH0lEQVR42mP4ukqOKohh1KBRg0YNGjVo1KBRgwbeIADadOluhHvNaQAAAABJRU5ErkJggg==";

fn main() {
    let mut window = Window::new(
        "Image Slot Example - Press SPACE to swap, ESC to quit",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut player = Player::new();
    player.set_autoplay(true);
    player.set_loop(true);

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ARGB8888)
        .unwrap();

    let animation_data = include_str!("../assets/animations/lottie/image.json");
    let c_data = CString::new(animation_data).expect("CString conversion failed");

    if player.load_animation_data(&c_data).is_err() {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Animation loaded successfully!");
    println!("Image slot ID: '{SLOT_ID}'");
    println!("Tip: the rising sun is the slotted image — swaps show as it crosses the frame.");
    println!("Press SPACE to cycle the replacement image");
    println!("Press ESC to quit");
    println!();

    // (label, source). `None` resets the slot back to the original asset.
    let states: [(&str, Option<&str>); 4] = [
        ("Original (bundled sun)", None),
        ("Magenta (data: URI)", Some(MAGENTA)),
        ("Teal (data: URI)", Some(TEAL)),
        ("Amber (data: URI)", Some(AMBER)),
    ];

    let mut index = 0usize;
    let mut last_space_press = std::time::Instant::now();
    let mut clock = common::Clock::new();

    println!("State: {}", states[index].0);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = clock.dt();
        let now = std::time::Instant::now();

        if window.is_key_down(Key::Space) && now.duration_since(last_space_press).as_millis() > 250
        {
            index = (index + 1) % states.len();
            let (label, src) = states[index];

            match src {
                Some(src) => {
                    let slot = ImageSlot::from_src(src.to_string()).with_dimensions(SLOT_W, SLOT_H);
                    let _ = player.set_image_slot(SLOT_ID, slot);
                }
                None => {
                    let _ = player.reset_slot(SLOT_ID);
                }
            }

            println!("State: {label}");
            last_space_press = now;
        }

        if player.tick(dt).unwrap_or(false) {
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!();
    println!("Example finished!");
}
