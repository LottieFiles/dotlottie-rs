//! Dedicated DragAndDrop interaction demo.
//!
//! Compare with examples/star_drop.rs (the same behavior authored as a
//! 3-state machine + live-bound slots). Here the whole gesture lives in ONE
//! interaction (assets/statemachines/star_drop_dnd.json):
//!
//!   { "type": "DragAndDrop",
//!     "layerName": "Star 2", "slotId": "star_pos",
//!     "tween": { "duration": 0.25, "easing": [0.25, 0.1, 0.25, 1] },
//!     "dropZones": [{ "layerName": "drop_zone", "lock": true,
//!                     "actions": [Increment docked_count] }] }
//!
//! Differences from the state-machine version you can feel:
//! - grab offset: the star doesn't jump to center itself on the pointer
//! - the snap tween is non-blocking (engine keeps Running)
//! - snap target comes from the drop_zone layer's authored position —
//!   zero coordinates in the state machine JSON
//! - once docked (lock: true), the star can't be grabbed again
//!
//! The host forwards pointer events. No cursor inputs at all.
//!
//! Run with:
//!   cargo run --example star_drop_dnd --features dev

#![allow(clippy::print_stdout)]

use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
use dotlottie_rs::events::Event;
use dotlottie_rs::{ColorSpace, Player, StateMachineEvent};
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
        "DragAndDrop interaction - ESC to exit",
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

    // Optional args: <state machine> <animation>. Defaults to the static
    // pairing; run the MOVING-zone tracking demo with:
    //   cargo run --example star_drop_dnd --features dev -- star_drop_track star_drop
    let sm_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "star_drop_dnd".to_string());
    let anim_name = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "star_drop_static".to_string());

    let animation_data =
        fs::read_to_string(format!("{ASSETS_DIR}/animations/lottie/{anim_name}.json"))
            .unwrap_or_else(|e| panic!("failed to read animation '{anim_name}': {e}"));
    let c_data = CString::new(animation_data).expect("animation data should be valid");
    player
        .load_animation_data(&c_data)
        .expect("animation should load");

    let definition = fs::read_to_string(format!("{ASSETS_DIR}/statemachines/{sm_name}.json"))
        .unwrap_or_else(|e| panic!("failed to read state machine '{sm_name}': {e}"));

    let mut engine = player
        .state_machine_load_data(&definition)
        .expect("state machine should load");
    engine
        .start(&OpenUrlPolicy::default())
        .expect("state machine should start");

    println!("Drag the star onto the drop zone (grab it anywhere — offset is kept).");

    let mut left_down = false;
    let mut last_mouse = (0.0_f32, 0.0_f32);
    let mut clock = common::Clock::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = clock.dt();

        while let Some(event) = engine.poll_event() {
            if let StateMachineEvent::NumericInputChange {
                name, new_value, ..
            } = event
            {
                println!("{name} -> {new_value}");
            }
        }

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
