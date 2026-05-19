#![allow(clippy::print_stdout)]

//! Demonstrates the built-in `@elapsedTime` numeric input.
//!
//! Default SM is a two-state splash → main: a single `@elapsedTime > 2.0`
//! guard transitions away from the splash state ~2 seconds after start.
//! `@elapsedTime` is read-only — there is no Reset action and no way for the
//! author or host to mutate it.
//!
//! Edit `STATE_MACHINE_PATH` to point at any other @elapsedTime SM.
//!
//! ESC to quit.

use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
use dotlottie_rs::events::Event;
use dotlottie_rs::{ColorSpace, Player, Rgba, StateMachineEvent};
use minifb::{Key, MouseButton, Window, WindowOptions};
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
// const STATE_MACHINE_PATH: &str = "statemachines/splash_demo.json";

const ANIMATION_PATH: &str = "animations/lottie/counter.json";
const STATE_MACHINE_PATH: &str = "statemachines/long_press.json";

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
        "@elapsedTime demo - ESC to exit",
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
            ColorSpace::ARGB8888,
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

    println!("@elapsedTime demo");
    println!("----------------");
    println!(
        "initial state: {} | @elapsedTime = {:?}",
        engine.get_current_state_name(),
        engine.get_numeric_input("@elapsedTime"),
    );

    let start = Instant::now();
    let mut clock = common::Clock::new();
    let mut last_print = Instant::now();
    let mut mx = 0.0_f32;
    let mut my = 0.0_f32;
    let mut left_down = false;
    let mut entered = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = clock.dt();
        let _ = engine.tick(dt);

        // Forward mouse input as pointer events so SMs that mix @elapsedTime
        // with pointer interactions react. Time-only SMs ignore these.
        let mouse_pressed = window.get_mouse_down(MouseButton::Left);
        if let Some((nmx, nmy)) = window.get_mouse_pos(minifb::MouseMode::Discard) {
            let moved = nmx != mx || nmy != my;
            mx = nmx;
            my = nmy;
            let inside = mx >= 0.0 && mx <= WIDTH as f32 && my >= 0.0 && my <= HEIGHT as f32;

            if inside && !entered {
                engine.post_event(&Event::PointerEnter { x: mx, y: my });
                entered = true;
            } else if !inside && entered {
                engine.post_event(&Event::PointerExit { x: mx, y: my });
                entered = false;
            }
            if inside && moved && !mouse_pressed {
                engine.post_event(&Event::PointerMove { x: mx, y: my });
            }
        }
        if mouse_pressed && !left_down {
            engine.post_event(&Event::PointerDown { x: mx, y: my });
        } else if !mouse_pressed && left_down {
            engine.post_event(&Event::PointerUp { x: mx, y: my });
            engine.post_event(&Event::Click { x: mx, y: my });
            println!(
                "[t={:>5.2}s] click posted at ({mx:.1}, {my:.1})",
                start.elapsed().as_secs_f32(),
            );
        }
        left_down = mouse_pressed;

        while let Some(evt) = engine.poll_event() {
            let now = start.elapsed().as_secs_f32();
            match evt {
                StateMachineEvent::Transition {
                    previous_state,
                    new_state,
                } => {
                    println!("[t={now:>5.2}s] transition: {previous_state} -> {new_state}");
                }
                // The events below are the proximate evidence that a click was
                // *captured* by a layer interaction — the click action fired,
                // mutating an input or emitting a custom event. Their absence
                // after a `click posted` line means the click missed every
                // hit-tested layer.
                StateMachineEvent::StringInputChange {
                    name,
                    old_value,
                    new_value,
                } => {
                    println!(
                        "[t={now:>5.2}s] click captured -> string input '{name}': {old_value} -> {new_value}"
                    );
                }
                StateMachineEvent::NumericInputChange {
                    name,
                    old_value,
                    new_value,
                } => {
                    println!(
                        "[t={now:>5.2}s] click captured -> numeric input '{name}': {old_value} -> {new_value}"
                    );
                }
                StateMachineEvent::BooleanInputChange {
                    name,
                    old_value,
                    new_value,
                } => {
                    println!(
                        "[t={now:>5.2}s] click captured -> boolean input '{name}': {old_value} -> {new_value}"
                    );
                }
                StateMachineEvent::InputFired { name } => {
                    println!("[t={now:>5.2}s] click captured -> event fired: '{name}'");
                }
                StateMachineEvent::CustomEvent { message } => {
                    println!("[t={now:>5.2}s] custom event: {message}");
                }
                _ => {}
            }
        }

        if last_print.elapsed() >= std::time::Duration::from_millis(500) {
            let et = engine.get_numeric_input("@elapsedTime").unwrap_or(0.0);
            println!(
                "  @elapsedTime = {:.3}s | state = {}",
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
