//! Spiral slide-to-unlock: drag scrubs the MAIN timeline.
//!
//! Uses assets/statemachines/spiral_unlock.json + spiral_unlock.json, whose
//! knob position slot (`knob_pos`) carries authored keyframes riding the
//! spiral over the "locked" segment (frames 0..119). No precomp, no tm
//! bridge: the drag scrubs the main timeline directly and every keyframed
//! property follows.
//!
//! Wiring:
//! - PathDrag (global) projects the pointer onto the Spiral Path layer and
//!   emits `path_t`; its hooks fire `grab`/`release` events.
//! - `idle` (paused at locked start) -> `dragging` on grab.
//! - A PointerMove interaction SCOPED to `dragging` (stateName) converts
//!   progress to a frame: scrub = path_t * 119 -> SetFrame. Scoping is what
//!   keeps the scrub from corrupting the other states.
//! - Release fires guarded Tweened transitions: path_t >= 0.95 -> `unlocked`
//!   (plays the celebration segment once); otherwise back to `idle`, the
//!   tween gliding the knob pose home.
//! - In `unlocked` nothing consumes path_t and the scrub is out of scope,
//!   so dragging the knob does nothing.
//!
//! Run with:
//!   cargo run --example spiral_unlock --features dev

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
        "Spiral unlock: drag the knob along the spiral - ESC to exit",
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
        fs::read_to_string(format!("{ASSETS_DIR}/animations/lottie/spiral_unlock.json"))
            .expect("spiral_unlock.json animation should exist");
    let c_data = CString::new(animation_data).expect("animation data should be valid");
    player
        .load_animation_data(&c_data)
        .expect("animation should load");

    // Optional arg: state machine name. `spiral_unlock_zones` adds dock
    // points on the spiral:
    //   cargo run --example spiral_unlock --features dev -- spiral_unlock_zones
    let sm_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "spiral_unlock".to_string());
    let definition = fs::read_to_string(format!("{ASSETS_DIR}/statemachines/{sm_name}.json"))
        .unwrap_or_else(|e| panic!("failed to read state machine '{sm_name}': {e}"));

    let mut engine = player
        .state_machine_load_data(&definition)
        .expect("state machine should load");
    engine
        .start(&OpenUrlPolicy::default())
        .expect("state machine should start");

    println!("Drag the knob all the way around the spiral to unlock.");

    let mut left_down = false;
    let mut last_mouse = (0.0_f32, 0.0_f32);
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
