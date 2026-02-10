#![allow(clippy::print_stdout)]

use base64::{engine::general_purpose::STANDARD, Engine};
use dotlottie_rs::{
    ColorSlot, Config, DotLottiePlayer, GradientSlot, GradientStop, ImageSlot, PositionSlot,
    ScalarSlot, TextDocument, TextSlot, VectorSlot,
};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

fn png_to_data_url(png_bytes: &[u8]) -> String {
    format!("data:image/png;base64,{}", STANDARD.encode(png_bytes))
}

/// Each combination applies a different mix of slot overrides to showcase the slots API.
///
/// Slot IDs present in slots.json:
///   Color    : "star_color"
///   Gradient : "star_gradient_color"
///   Text     : "title_text"
///   Vector   : "star_scale"
///   Position : "star_position"
///   Scalar   : "star_rotation", "star_opacity"
///   Image    : "img"
fn apply_combination(player: &mut DotLottiePlayer, index: usize) {
    player.clear_slots();

    match index {
        0 => {
            player.set_color_slot("star_color", ColorSlot::new([1.0, 0.0, 0.0]));
            player.set_text_slot(
                "title_text",
                TextSlot::with_document(
                    TextDocument::new("Red Star!")
                        .with_font("Cal Sans Regular")
                        .with_size(10.0),
                ),
            );
            player.set_color_slot("title_text_color", ColorSlot::new([1.0, 0.0, 0.0]));
        }

        1 => {
            player.set_gradient_slot(
                "star_gradient_color",
                GradientSlot::new(vec![
                    GradientStop {
                        offset: 0.0,
                        color: [1.0, 0.0, 0.0, 1.0],
                    },
                    GradientStop {
                        offset: 0.5,
                        color: [0.0, 1.0, 0.0, 1.0],
                    },
                    GradientStop {
                        offset: 1.0,
                        color: [0.0, 0.0, 1.0, 1.0],
                    },
                ]),
            );
            player.set_vector_slot("star_scale", VectorSlot::static_value([150.0, 150.0]));
        }

        2 => {
            player.set_position_slot("star_position", PositionSlot::static_value([350.0, 200.0]));
            player.set_scalar_slot("star_rotation", ScalarSlot::new(45.0));
            player.set_scalar_slot("star_opacity", ScalarSlot::new(50.0));
        }

        3 => {
            player.set_text_slot(
                "title_text",
                TextSlot::with_document(
                    TextDocument::new("Image Swap")
                        .with_font("Cal Sans Regular")
                        .with_size(40.0),
                ),
            );
            player.set_color_slot("title_text_color", ColorSlot::new([0.2, 0.4, 0.9]));
            let png_bytes = include_bytes!("../assets/images/image.png");
            player.set_image_slot(
                "img",
                ImageSlot::from_data_url(png_to_data_url(png_bytes)).with_dimensions(300, 300),
            );
            player.set_color_slot("star_color", ColorSlot::new([0.2, 0.4, 0.9]));
        }

        4 => {
            player.set_color_slot("star_color", ColorSlot::new([0.0, 0.8, 0.3]));
            player.set_gradient_slot(
                "star_gradient_color",
                GradientSlot::new(vec![
                    GradientStop {
                        offset: 0.0,
                        color: [1.0, 0.84, 0.0, 1.0],
                    },
                    GradientStop {
                        offset: 1.0,
                        color: [1.0, 0.27, 0.0, 1.0],
                    },
                ]),
            );
            player.set_text_slot(
                "title_text",
                TextSlot::with_document(
                    TextDocument::new("All Slots!")
                        .with_font("Cal Sans Regular")
                        .with_size(45.0)
                ),
            );
            player.set_color_slot("title_text_color", ColorSlot::new([0.0, 0.8, 0.3]));
            player.set_vector_slot("star_scale", VectorSlot::static_value([120.0, 120.0]));
            player.set_position_slot("star_position", PositionSlot::static_value([300.0, 220.0]));
            player.set_scalar_slot("star_rotation", ScalarSlot::new(30.0));
            player.set_scalar_slot("star_opacity", ScalarSlot::new(10.0));
            player.set_image_slot(
                "img",
                ImageSlot::from_data_url(png_to_data_url(include_bytes!(
                    "../assets/images/image.png"
                )))
                .with_dimensions(100, 100),
            );
        }

        _ => unreachable!(),
    }
}

const NUM_COMBOS: usize = 5;

fn main() {
    let mut window = Window::new(
        "Slots Example - SPACE: cycle | C: clear | R: reset | ESC: quit",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut player = DotLottiePlayer::new(
        Config {
            loop_animation: true,
            autoplay: true,
            background_color: 0xffffffff,
            ..Config::default()
        },
        0,
    );

    let animation_data = include_str!("../assets/animations/lottie/slots.json");
    let c_data = CString::new(animation_data).expect("CString conversion failed");

    if !player.load_animation_data(&c_data, WIDTH, HEIGHT) {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Animation loaded successfully!");
    println!("──────────────────────────────────────────");
    println!("  SPACE  → Cycle through slot combinations");
    println!("  C      → Clear all slot overrides");
    println!("  R      → Reset (reload animation)");
    println!("  ESC    → Quit");
    println!("──────────────────────────────────────────");

    let mut current_combo: usize = 0;
    let mut space_was_down = false;
    let mut c_was_down = false;
    let mut r_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let space_is_down = window.is_key_down(Key::Space);
        let c_is_down = window.is_key_down(Key::C);
        let r_is_down = window.is_key_down(Key::R);

        // Space released → cycle to next combination
        if space_was_down && !space_is_down {
            apply_combination(&mut player, current_combo);
            current_combo = (current_combo + 1) % NUM_COMBOS;
        }

        // C released → clear all slot overrides (revert to original animation values)
        if c_was_down && !c_is_down {
            player.clear_slots();
        }

        // R released → full reset by reloading the animation
        if r_was_down && !r_is_down {
            player.load_animation_data(&c_data, WIDTH, HEIGHT);
            current_combo = 0;
        }

        space_was_down = space_is_down;
        c_was_down = c_is_down;
        r_was_down = r_is_down;

        if player.tick() {
            window
                .update_with_buffer(player.buffer(), WIDTH as usize, HEIGHT as usize)
                .unwrap();
        }
    }
}
