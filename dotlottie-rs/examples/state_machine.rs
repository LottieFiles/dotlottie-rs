#![allow(clippy::print_stdout)]

use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
use dotlottie_rs::events::Event;
use dotlottie_rs::{ColorSpace, Config, DotLottiePlayer, StateMachineEngine, StateMachineEvent};
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::ffi::CString;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

pub const ANIMATION_NAME: &str = "smiley-slider.lottie";
pub const STATE_MACHINE_NAME: &str = "smileys";

fn process_state_machine_events(engine: &mut StateMachineEngine) {
    while let Some(event) = engine.poll_event() {
        match event {
            StateMachineEvent::Start => {
                println!("[state machine event] on_start");
            }
            StateMachineEvent::Stop => {
                println!("[state machine event] on_stop");
            }
            StateMachineEvent::Transition {
                previous_state,
                new_state,
            } => {
                println!("[state machine event] on_transition: {previous_state} -> {new_state}");
            }
            StateMachineEvent::StateEntered { state } => {
                println!("[state machine event] on_state_entered: {state}");
            }
            StateMachineEvent::StateExit { state } => {
                println!("[state machine event] on_state_exit: {state}");
            }
            StateMachineEvent::CustomEvent { message } => {
                println!("[state machine event] custom_event: {message}");
            }
            StateMachineEvent::Error { message } => {
                println!("[state machine event] error: {message}");
            }
            StateMachineEvent::StringInputChange {
                name,
                old_value,
                new_value,
            } => {
                println!(
                    "[state machine event] string_input_value_change ==> {name} : {old_value} -> {new_value}"
                );
            }
            StateMachineEvent::NumericInputChange {
                name,
                old_value,
                new_value,
            } => {
                println!(
                    "[state machine event] numeric_input_value_change ==> {name} : {old_value} -> {new_value}"
                );
            }
            StateMachineEvent::BooleanInputChange {
                name,
                old_value,
                new_value,
            } => {
                println!(
                    "[state machine event] boolean_input_value_change ==> {name} : {old_value} -> {new_value}"
                );
            }
            StateMachineEvent::InputFired { name } => {
                println!("[state machine event] input_fired ==> {name}");
            }
        }
    }
}

fn load_animation(player: &mut DotLottiePlayer, path: &PathBuf) -> bool {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let success = match extension {
        "lottie" => {
            let mut file = File::open(path).expect("Could not open file");
            let metadata = fs::metadata(path).expect("Could not read metadata");
            let mut buffer = vec![0; metadata.len() as usize];
            file.read_exact(&mut buffer).expect("Buffer overflow");
            player.load_dotlottie_data(&buffer, WIDTH as u32, HEIGHT as u32)
        }
        "json" => {
            let data = fs::read_to_string(path).expect("Could not read JSON file");
            let c_data = CString::new(data).expect("CString conversion failed");
            player.load_animation_data(&c_data, WIDTH as u32, HEIGHT as u32)
        }
        _ => false,
    };

    if success {
        println!(
            "Loaded: {}",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
    } else {
        eprintln!(
            "Failed to load: {}",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
    }

    success
}

fn main() {
    let mut window = Window::new(
        "dotLottie State Machine Demo - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut player = DotLottiePlayer::new(
        Config {
            background_color: 0xffffffff,
            ..Config::default()
        },
        0,
    );

    // Allocate buffer for software rendering
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    // Set software rendering target
        player.set_sw_target_buffer(
            &mut buffer,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ABGR8888,
        );
    let animation_path =
        PathBuf::from(format!("./assets/animations/dotlottie/v1/{ANIMATION_NAME}"));

    if !load_animation(&mut player, &animation_path) {
        eprintln!("Failed to load animation, exiting");
        return;
    }

    let state_machine_path = format!("./assets/statemachines/{STATE_MACHINE_NAME}.json");
    let state_machine_def = fs::read_to_string(&state_machine_path)
        .unwrap_or_else(|e| panic!("Failed to read state machine file {state_machine_path}: {e}"));

    let mut engine = player
        .state_machine_load_data(&state_machine_def)
        .expect("Failed to load state machine");

    let open_url = OpenUrlPolicy::default();

    let started = engine.start(&open_url);
    println!("State machine started: {started}");

    if !started {
        eprintln!("Warning: State machine failed to start");
    }

    engine.player.set_frame(0.0);
    engine.player.render();

    let mut mx = 0.0_f32;
    let mut my = 0.0_f32;
    let mut left_down = false;
    let mut entered = false;
    let mut last_buffer_update = std::time::Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        process_state_machine_events(&mut engine);

        let mouse_pressed = window.get_mouse_down(MouseButton::Left);

        if let Some((new_mx, new_my)) = window.get_mouse_pos(minifb::MouseMode::Discard) {
            let mouse_moved = new_mx != mx || new_my != my;
            mx = new_mx;
            my = new_my;

            let inside = mx >= 0.0 && mx <= WIDTH as f32 && my >= 0.0 && my <= HEIGHT as f32;

            if inside && !entered {
                engine.post_event(&Event::PointerEnter { x: mx, y: my });
                entered = true;
            } else if !inside && entered {
                engine.post_event(&Event::PointerExit { x: mx, y: my });
                entered = false;
            }

            if inside && mouse_moved && !mouse_pressed {
                engine.post_event(&Event::PointerMove { x: mx, y: my });
            }
        }

        if !mouse_pressed && left_down {
            engine.post_event(&Event::Click { x: mx, y: my });
        }

        if mouse_pressed && !left_down {
            engine.post_event(&Event::PointerDown { x: mx, y: my });
        } else if !mouse_pressed && left_down {
            engine.post_event(&Event::PointerUp { x: mx, y: my });
        }

        left_down = mouse_pressed;

        let frame_changed = engine.tick();

        if frame_changed || last_buffer_update.elapsed().as_millis() > 100 {
            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
            last_buffer_update = std::time::Instant::now();
        } else {
            // Still need to call update to process window events
            window.update();
        }
    }

    engine.release();
    println!("Example finished!");
}
