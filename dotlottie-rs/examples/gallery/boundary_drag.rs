//! Boundary-constrained free drag.
//!
//! Uses assets/statemachines/boundary_drag.json + boundary_drag.json: a
//! DragAndDrop with no drop zones (the circle stays wherever it's
//! released) and `boundaryLayerName: "Rectangle 1"` — while held, the
//! circle's center is clamped into the rectangle layer's RENDERED bounds,
//! inset by the circle's own size so the whole circle stays inside.
//!
//! The bounds are read from the scene every move (same query as
//! hit-testing), so the rectangle's 134%/132% layer scale — invisible to
//! any static JSON extraction — is honored automatically, and an animated
//! or re-scaled boundary would be too.
//!
//! Run with:
//!   cargo run --example boundary_drag --features dev

#![allow(clippy::print_stdout)]

use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
use dotlottie_rs::events::Event;
use dotlottie_rs::{ColorSpace, Player};
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::ffi::CString;
use std::fs;

#[path = "../common/mod.rs"]
mod common;

pub const WIDTH: usize = 512;
pub const HEIGHT: usize = 512;
const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

fn main() {
    let mut window = Window::new(
        "Boundary drag: circle stays inside the rectangle - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut player = Player::new();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    player
        .set_sw_target(
            &mut buffer,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ABGR8888,
        )
        .unwrap();

    let animation_data =
        fs::read_to_string(format!("{ASSETS_DIR}/animations/lottie/boundary_drag.json"))
            .expect("boundary_drag.json animation should exist");
    let c_data = CString::new(animation_data).expect("animation data should be valid");
    player
        .load_animation_data(&c_data)
        .expect("animation should load");

    let definition = fs::read_to_string(format!("{ASSETS_DIR}/statemachines/boundary_drag.json"))
        .expect("boundary_drag.json state machine should exist");

    let mut engine = player
        .state_machine_load_data(&definition)
        .expect("state machine should load");
    engine
        .start(&OpenUrlPolicy::default())
        .expect("state machine should start");

    println!("Drag the circle — it can go anywhere inside the rectangle, nowhere outside.");

    let mut left_down = false;
    let mut last_mouse = (0.0_f32, 0.0_f32);
    let mut clock = common::Clock::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = clock.dt();

        if let Some((mx, my)) = window.get_mouse_pos(minifb::MouseMode::Discard) {
            if (mx, my) != last_mouse {
                engine.post_event(&Event::PointerMove { x: mx, y: my });
                last_mouse = (mx, my);
            }

            let mouse_pressed = window.get_mouse_down(MouseButton::Left);
            if mouse_pressed && !left_down {
                engine.post_event(&Event::PointerDown { x: mx, y: my });
            } else if !mouse_pressed && left_down {
                engine.post_event(&Event::PointerUp { x: mx, y: my });
            }
            left_down = mouse_pressed;
        }

        let _ = engine.tick(dt);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }

    engine.release();
    println!("Example finished!");
}
