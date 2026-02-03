use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
use dotlottie_rs::events::Event;
use dotlottie_rs::{Config, DotLottiePlayer, StateMachineEngine, StateMachineEvent};
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::fs::{self, File};
use std::io::Read;

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

// pub const ANIMATION_NAME: &str = "smiley_slider";
// pub const STATE_MACHINE_NAME: &str = "smiley-slider-state";

pub const ANIMATION_NAME: &str = "traffic_lights";
pub const STATE_MACHINE_NAME: &str = "traffic_lights";

// pub const STATE_MACHINE_NAME: &str = "pigeon_with_listeners";
// pub const ANIMATION_NAME: &str = "pigeon";
// pub const STATE_MACHINE_NAME: &str = "theming";
// pub const ANIMATION_NAME: &str = "themed";
// pub const STATE_MACHINE_NAME: &str = "rating";
// pub const STATE_MACHINE_NAME: &str = "StateMachine1";
// pub const ANIMATION_NAME: &str = "pig";

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
                println!(
                    "[state machine event] on_transition: {} -> {}",
                    previous_state, new_state
                );
            }
            StateMachineEvent::StateEntered { state } => {
                println!("[state machine event] on_state_entered: {}", state);
            }
            StateMachineEvent::StateExit { state } => {
                println!("[state machine event] on_state_exit: {}", state);
            }
            StateMachineEvent::CustomEvent { message } => {
                println!("[state machine event] custom_event: {}", message);
            }
            StateMachineEvent::Error { message } => {
                println!("[state machine event] error: {}", message);
            }
            StateMachineEvent::StringInputChange {
                name,
                old_value,
                new_value,
            } => {
                println!(
                    "[state machine event] string_input_value_change ==> {} : {} -> {}",
                    name, old_value, new_value
                );
            }
            StateMachineEvent::NumericInputChange {
                name,
                old_value,
                new_value,
            } => {
                println!(
                    "[state machine event] numeric_input_value_change ==> {} : {} -> {}",
                    name, old_value, new_value
                );
            }
            StateMachineEvent::BooleanInputChange {
                name,
                old_value,
                new_value,
            } => {
                println!(
                    "[state machine event] boolean_input_value_change ==> {} : {} -> {}",
                    name, old_value, new_value
                );
            }
            StateMachineEvent::InputFired { name } => {
                println!("[state machine event] input_fired ==> {}", name);
            }
        }
    }
}

fn main() {
    let mut window = Window::new(
        "dotLottie rust demo - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Create player
    let mut player = DotLottiePlayer::new(
        Config {
            background_color: 0xffffffff,
            ..Config::default()
        },
        0, // threads (0 = auto)
    );

    // Load animation
    let mut file = File::open(format!(
        "./src/bin/shared/animations/{}.lottie",
        ANIMATION_NAME
    ))
    .expect("no file found");
    let metadata = fs::metadata(format!(
        "./src/bin/shared/animations/{}.lottie",
        ANIMATION_NAME
    ))
    .expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    file.read(&mut buffer).expect("buffer overflow");

    player.load_dotlottie_data(&buffer, WIDTH as u32, HEIGHT as u32);

    // Load state machine definition
    let state_machine_def: String = fs::read_to_string(format!(
        "./src/bin/shared/statemachines/{}.json",
        STATE_MACHINE_NAME
    ))
    .unwrap();

    // Create state machine engine (borrows player)
    let mut engine = player.state_machine_load_data(&state_machine_def).expect("Failed to load state machine");

    // Configure and start state machine
    let open_url = OpenUrlPolicy {
        whitelist: ["example.com/path/*/another_path/*".to_string()].to_vec(),
        require_user_interaction: true,
    };

    let started = engine.start(&open_url);
    println!("Start state machine -> {}", started.is_ok());

    let mut mx = 0.0;
    let mut my = 0.0;
    let mut left_down = false;
    let mut entered = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Process state machine events (SDL-style polling)
        process_state_machine_events(&mut engine);

        let tmp = window.get_mouse_down(MouseButton::Left);
        let mouse_pos = window.get_mouse_pos(minifb::MouseMode::Discard);
        mouse_pos.map(|mouse| {
            if mouse.0 != mx || mouse.1 != my {
                mx = mouse.0;
                my = mouse.1;
            }

            if mx >= 0.0 && mx <= WIDTH as f32 && my >= 0.0 && my <= HEIGHT as f32 {
                if !entered {
                    let event = Event::PointerEnter {
                        x: mx * 2.0,
                        y: my * 2.0,
                    };

                    engine.post_event(&event);
                }
                entered = true;
            } else {
                if entered {
                    let event = Event::PointerExit { x: mx, y: my };

                    engine.post_event(&event);
                }

                entered = false;
            }
        });

        if !tmp && left_down {
            println!("Sending click");
            let event = Event::Click { x: mx, y: my };

            engine.post_event(&event);
        }

        left_down = tmp;

        if left_down {
            let event = Event::PointerDown { x: mx, y: my };

            engine.post_event(&event);
        } else {
            if mx != 0.0 && my != 0.0 {
                let event = Event::PointerMove { x: mx, y: my };
                engine.post_event(&event);
            }
        }

        // Tick the state machine (handles player.tick() internally)
        if engine.tick().is_ok() {
            window.update_with_buffer(engine.player.buffer(), WIDTH, HEIGHT).unwrap();
        }
    }

    // Release the state machine (releases borrow of player)
    engine.release();
}
