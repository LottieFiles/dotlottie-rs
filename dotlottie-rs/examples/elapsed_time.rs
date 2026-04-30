#![allow(clippy::print_stdout)]

//! Demonstrates the built-in `elapsedTime` numeric input with live rendering.
//!
//! A two-state state machine (`wink` and `laugh`) auto-toggles between the
//! `wink` and `laughing` segments of `smiley-slider.lottie` every 1 second,
//! driven entirely by an `elapsedTime > 1.0` guard. Each state's entry action
//! calls `Reset { inputName: "elapsedTime" }` so the next cycle starts at zero.
//!
//! What to look for:
//!   - The animation visibly switches between segments every second.
//!   - The console transition log fires at ~1s intervals.
//!
//! ESC to quit.

use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
use dotlottie_rs::{ColorSpace, Player, Rgba, StateMachineEvent};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::time::Instant;

mod common;

const WIDTH: usize = 500;
const HEIGHT: usize = 500;
const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

// Edit these to swap in your own animation + state machine.
// Paths are relative to `dotlottie-rs/assets/`.
// const ANIMATION_PATH: &str = "animations/dotlottie/v1/smiley-slider.lottie";
// const STATE_MACHINE_PATH: &str = "statemachines/elapsed_time_demo.json";

const ANIMATION_PATH: &str = "animations/lottie/counter.json";
const STATE_MACHINE_PATH: &str = "statemachines/countdown_timer.json";

fn load_animation(player: &mut Player, path: &str) {
    let extension = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let ok = match extension {
        "lottie" => {
            let bytes = fs::read(path).expect("failed to read .lottie");
            player.load_dotlottie_data(&bytes).is_ok()
        }
        "json" => {
            let data = fs::read_to_string(path).expect("failed to read .json");
            let cdata = CString::new(data).expect("CString conversion failed");
            player.load_animation_data(&cdata).is_ok()
        }
        other => panic!("unsupported animation extension: {other:?}"),
    };
    assert!(ok, "failed to load animation: {path}");
}

fn main() {
    let mut window = Window::new(
        "elapsedTime demo - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{}", e));
    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut player = Player::new();
    let _ = player.set_background(Rgba::from(0xffffffff));
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    player
        .set_sw_target(
            &mut buffer,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ABGR8888,
        )
        .expect("failed to set sw target");

    let animation_path = format!("{ASSETS_DIR}/{ANIMATION_PATH}");
    load_animation(&mut player, &animation_path);

    let state_machine_path = format!("{ASSETS_DIR}/{STATE_MACHINE_PATH}");
    let state_machine_def = fs::read_to_string(&state_machine_path)
        .unwrap_or_else(|e| panic!("failed to read state machine {state_machine_path}: {e}"));

    let mut engine = player
        .state_machine_load_data(&state_machine_def)
        .expect("failed to load state machine");
    engine
        .start(&OpenUrlPolicy::default())
        .expect("failed to start state machine");

    println!("elapsedTime demo");
    println!("----------------");
    println!(
        "initial state: {} | elapsedTime = {:?}",
        engine.get_current_state_name(),
        engine.get_numeric_input("elapsedTime"),
    );

    let start = Instant::now();
    let mut clock = common::Clock::new();
    let mut last_print = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = clock.dt();
        let _ = engine.tick(dt);

        while let Some(evt) = engine.poll_event() {
            if let StateMachineEvent::Transition {
                previous_state,
                new_state,
            } = evt
            {
                println!(
                    "[t={:>5.2}s] transition: {} -> {}",
                    start.elapsed().as_secs_f32(),
                    previous_state,
                    new_state,
                );
            }
        }

        if last_print.elapsed() >= std::time::Duration::from_millis(500) {
            let et = engine.get_numeric_input("elapsedTime").unwrap_or(0.0);
            println!(
                "  elapsedTime = {:.3}s | state = {}",
                et,
                engine.get_current_state_name(),
            );
            last_print = Instant::now();
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }

    engine.release();
    println!("done.");
}
