use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
use dotlottie_rs::events::Event;
use dotlottie_rs::{Config, DotLottiePlayer, StateMachineEngine, StateMachineEvent};
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::fs::{self, File};
use std::io::Read;

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

struct AnimationManager {
    current_index: usize,
    animations: Vec<String>,
}

impl AnimationManager {
    fn new() -> Self {
        let animations = Self::load_animations();
        Self {
            current_index: 0,
            animations,
        }
    }

    fn load_animations() -> Vec<String> {
        let paths = ["./src/bin/shared/packaged_state_machines"];

        let mut animations = Vec::new();

        for path in paths.iter() {
            match fs::read_dir(path) {
                Ok(entries) => {
                    println!("Successfully found animations directory at: {}", path);
                    for entry in entries.flatten() {
                        if let Some(ext) = entry.path().extension() {
                            if ext == "lottie" {
                                if let Some(filename) = entry.path().file_stem() {
                                    if let Some(name) = filename.to_str() {
                                        animations.push(name.to_string());
                                        println!("Found animation: {}", name);
                                    }
                                }
                            }
                        }
                    }
                    break;
                }
                Err(e) => {
                    eprintln!("Could not read directory {}: {}", path, e);
                }
            }
        }

        if animations.is_empty() {
            eprintln!("Warning: No .lottie files found!");
        } else {
            println!("Found {} animation(s)", animations.len());
        }

        animations.sort();
        animations
    }

    fn next(&mut self) -> Option<String> {
        if self.animations.is_empty() {
            return None;
        }
        self.current_index = (self.current_index + 1) % self.animations.len();
        Some(self.animations[self.current_index].clone())
    }

    fn previous(&mut self) -> Option<String> {
        if self.animations.is_empty() {
            return None;
        }
        if self.current_index == 0 {
            self.current_index = self.animations.len() - 1;
        } else {
            self.current_index -= 1;
        }
        Some(self.animations[self.current_index].clone())
    }

    fn current(&self) -> Option<String> {
        if self.animations.is_empty() {
            None
        } else {
            Some(self.animations[self.current_index].clone())
        }
    }
}

/// Process state machine events using SDL-style poll_event pattern
fn process_state_machine_events(engine: &mut StateMachineEngine) {
    while let Some(event) = engine.poll_event() {
        match event {
            StateMachineEvent::Transition {
                previous_state,
                new_state,
            } => {
                println!(
                    "\x1b[33m on_transition: {} -> {} \x1b[0m",
                    previous_state, new_state
                );
            }
            StateMachineEvent::StateEntered { state } => {
                println!("\x1b[33m on_state_entered: {} \x1b[0m", state);
            }
            StateMachineEvent::StateExit { state } => {
                println!("\x1b[33m on_state_exit: {} \x1b[0m", state);
            }
            StateMachineEvent::CustomEvent { message } => {
                println!("\x1b[33m custom_event: {} \x1b[0m", message);
            }
            _ => {}
        }
    }
}

fn load_animation(player: &mut DotLottiePlayer, animation_name: &str) -> bool {
    let path = format!(
        "./src/bin/shared/packaged_state_machines/{}.lottie",
        animation_name
    );

    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open file {}: {}", path, e);
            return false;
        }
    };

    let metadata = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to read metadata: {}", e);
            return false;
        }
    };

    let mut buffer = vec![0; metadata.len() as usize];
    if let Err(e) = file.read(&mut buffer) {
        eprintln!("Failed to read file: {}", e);
        return false;
    }

    player.load_dotlottie_data(&buffer, WIDTH as u32, HEIGHT as u32);
    println!(
        "Successfully loaded animation: {} from {}",
        animation_name, path
    );
    true
}

fn get_first_state_machine_id(player: &DotLottiePlayer) -> Option<String> {
    player
        .manifest()
        .and_then(|m| m.state_machines.as_ref())
        .and_then(|sms| sms.first().map(|sm| sm.id.clone()))
}

fn main() {
    let mut window = Window::new(
        "dotLottie State Machine Showcase - ESC to exit, LEFT/RIGHT to switch",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{}", e));

    let mut animation_manager = AnimationManager::new();

    // Create player
    let mut player = DotLottiePlayer::new(
        Config {
            background_color: 0xffffffff,
            ..Config::default()
        },
        0, // threads (0 = auto)
    );

    // Load initial animation
    let mut current_animation = match animation_manager.current() {
        Some(name) => name,
        None => {
            eprintln!("No animations found!");
            return;
        }
    };

    let mut mx = 0.0;
    let mut my = 0.0;
    let mut left_down = false;
    let mut entered = false;
    let mut just_switched = false;

    // Outer loop for animation switching
    'outer: loop {
        // Load animation
        if !load_animation(&mut player, &current_animation) {
            eprintln!("Failed to load animation: {}", current_animation);
            return;
        }

        // Get state machine ID from manifest
        let state_machine_id = match get_first_state_machine_id(&player) {
            Some(id) => id,
            None => {
                eprintln!("No state machine found in manifest");
                return;
            }
        };

        println!(
            "\x1b[32m Loading state machine: {} \x1b[0m",
            state_machine_id
        );

        // Create state machine engine (borrows player)
        let mut engine = match player.state_machine_load(&state_machine_id) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Failed to load state machine: {:?}", e);
                return;
            }
        };

        // Start the state machine
        if !engine.start(&OpenUrlPolicy::default()) {
            eprintln!("Failed to start state machine");
            engine.release();
            return;
        }

        println!(
            "\x1b[32m Successfully started state machine: {} \x1b[0m",
            state_machine_id
        );

        engine.player.render();

        // Inner loop for main event handling
        while window.is_open() && !window.is_key_down(Key::Escape) {
            // Check for animation switch request (skip if we just switched to avoid rapid cycling)
            if !just_switched {
                if window.is_key_pressed(Key::Right, minifb::KeyRepeat::No) {
                    if let Some(next) = animation_manager.next() {
                        current_animation = next;
                        engine.release();
                        just_switched = true;
                        continue 'outer;
                    }
                }
                if window.is_key_pressed(Key::Left, minifb::KeyRepeat::No) {
                    if let Some(prev) = animation_manager.previous() {
                        current_animation = prev;
                        engine.release();
                        just_switched = true;
                        continue 'outer;
                    }
                }
            } else {
                // Reset flag after one frame
                just_switched = false;
            }

            // Process state machine events
            process_state_machine_events(&mut engine);

            // Handle mouse input
            let tmp = window.get_mouse_down(MouseButton::Left);
            let mouse_pos = window.get_mouse_pos(minifb::MouseMode::Pass);
            if let Some(mouse) = mouse_pos {
                if mouse.0 != mx || mouse.1 != my {
                    mx = mouse.0;
                    my = mouse.1;
                }

                if mx >= 0.0 && mx <= WIDTH as f32 && my >= 0.0 && my <= HEIGHT as f32 {
                    if !entered {
                        let event = Event::PointerEnter { x: mx, y: my };
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
            }

            if !tmp && left_down {
                println!("Sending pointer up");
                let event = Event::PointerUp { x: mx, y: my };
                engine.post_event(&event);
            }

            left_down = tmp;

            if left_down {
                println!("Sending pointer down");
                let event = Event::PointerDown { x: mx, y: my };
                engine.post_event(&event);
            } else {
                let event = Event::PointerMove { x: mx, y: my };
                engine.post_event(&event);
            }

            // Handle keyboard inputs for state machine
            if window.is_key_pressed(Key::Space, minifb::KeyRepeat::Yes) {
                engine.set_numeric_input("Rating", 1.0, true, false);
            }

            if window.is_key_pressed(Key::Enter, minifb::KeyRepeat::No) {
                if let Some(current) = engine.get_boolean_input("OnOffSwitch") {
                    engine.set_boolean_input("OnOffSwitch", !current, true, false);
                }
            }

            // Tick and render
            if engine.tick() {
                let buffer = engine.player.buffer();
                window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
            }
        }

        // Clean up and exit
        engine.release();
        break;
    }
}
