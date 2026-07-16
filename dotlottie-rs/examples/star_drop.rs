//! Minimal PURE state-machine drag & drop — single object.
//!
//! The colleague-explainable version of examples/drag_drop_pure.rs:
//! one star (slot "star_pos"), one drop zone (layer "drop_zone"), and a
//! 3-state machine (assets/statemachines/star_drop.json):
//!
//!   idle      star_pos ← $star_x/$star_y      (rest, live-bound)
//!   dragging  star_pos ← $cursor_x/$cursor_y  (follows pointer)
//!   docking   transient: bakes the dock coords into star_x/star_y,
//!             falls through to idle
//!
//!   idle -> dragging       on grab (PointerDown hit-tests "Star 2")
//!   dragging -> docking    on released & over_zone   (0.25s tween)
//!   dragging -> idle       on released               (0.25s tween home)
//!
//! The host below only forwards pointer events, feeds cursor inputs, and
//! ticks. Zero drag logic.
//!
//! Run with:
//!   cargo run --example star_drop --features dev

#![allow(clippy::print_stdout)]

use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
use dotlottie_rs::events::Event;
use dotlottie_rs::{ColorSpace, Player, StateMachineEvent};
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::ffi::CString;
use std::fs;

mod common;

pub const WIDTH: usize = 512;
pub const HEIGHT: usize = 512;
const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

fn main() {
    let mut window = Window::new(
        "Star drop (pure SM) - ESC to exit",
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
        fs::read_to_string(format!("{ASSETS_DIR}/animations/lottie/star_drop.json"))
            .expect("star_drop.json animation should exist");
    let c_data = CString::new(animation_data).expect("animation data should be valid");
    player
        .load_animation_data(&c_data)
        .expect("animation should load");

    let definition = fs::read_to_string(format!("{ASSETS_DIR}/statemachines/star_drop.json"))
        .expect("star_drop.json state machine should exist");

    let mut engine = player
        .state_machine_load_data(&definition)
        .expect("state machine should load");
    engine
        .start(&OpenUrlPolicy::default())
        .expect("state machine should start");

    println!("Drag the star onto the drop zone.");

    let mut left_down = false;
    let mut clock = common::Clock::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = clock.dt();

        while let Some(event) = engine.poll_event() {
            if let StateMachineEvent::Transition {
                previous_state,
                new_state,
            } = event
            {
                println!("transition: {previous_state} -> {new_state}");
            }
        }

        // The host's ONLY inputs: cursor position + pointer events.
        if let Some((mx, my)) = window.get_mouse_pos(minifb::MouseMode::Discard) {
            // run_pipeline=true keeps the machine ticking (docking's
            // guardless fall-through after the glide).
            let _ = engine.set_numeric_input("cursor_x", mx, true, false);
            let _ = engine.set_numeric_input("cursor_y", my, false, false);

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
