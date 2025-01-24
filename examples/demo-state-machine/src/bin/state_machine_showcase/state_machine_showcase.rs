use dotlottie_rs::events::Event;
use dotlottie_rs::{Config, DotLottiePlayer, StateMachineObserver};
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::fs::{self, File};
use std::io::Read;
use std::os::macos::raw::stat;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

struct Timer {
    last_update: Instant,
    prev_frame: f32,
    first: bool,
}

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
        let mut used_path = None;

        for path in paths.iter() {
            match fs::read_dir(path) {
                Ok(entries) => {
                    used_path = Some(path);
                    println!("Successfully found animations directory at: {}", path);
                    for entry in entries {
                        match entry {
                            Ok(entry) => {
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
                            Err(e) => {
                                eprintln!("Error reading directory entry: {}", e);
                            }
                        }
                    }
                    break; // Found working directory, stop checking others
                }
                Err(e) => {
                    eprintln!("Could not read directory {}: {}", path, e);
                }
            }
        }

        if animations.is_empty() {
            eprintln!("Warning: No .lottie files found in any of the searched directories!");
            if let Some(path) = used_path {
                eprintln!("Last searched path: {}", path);
            }
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

struct DummyObserver;

impl StateMachineObserver for DummyObserver {
    fn on_transition(&self, previous_state: String, new_state: String) {
        println!(
            "\x1b[33m on_transition2: {} -> {} \x1b[0m",
            previous_state, new_state
        );
    }

    fn on_state_entered(&self, entering_state: String) {
        println!("\x1b[33m on_state_entereds: {} \x1b[0m", entering_state);
    }

    fn on_state_exit(&self, leaving_state: String) {
        println!("\x1b[33m on_state_exit: {} \x1b[0m", leaving_state);
    }

    fn on_custom_event(&self, message: String) {
        println!("\x1b[33m custom_event: {} \x1b[0m", message);
    }
}

impl Timer {
    fn new() -> Self {
        Self {
            last_update: Instant::now(),
            prev_frame: 0.0,
            first: false,
        }
    }

    fn tick(&mut self, animation: &DotLottiePlayer) {
        let next_frame = animation.request_frame();
        animation.set_frame(next_frame);

        if next_frame != self.prev_frame || !self.first {
            animation.render();
            self.first = true;
        }

        self.last_update = Instant::now();
        self.prev_frame = next_frame;
    }
}

fn load_animation(
    player: &DotLottiePlayer,
    animation_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let paths = [format!(
        "./src/bin/shared/packaged_state_machines/{}.lottie",
        animation_name
    )];

    let mut last_error = None;

    for path in paths.iter() {
        match File::open(path) {
            Ok(mut file) => match fs::metadata(path) {
                Ok(metadata) => {
                    let mut buffer = vec![0; metadata.len() as usize];
                    match file.read(&mut buffer) {
                        Ok(_) => {
                            println!(
                                "Successfully loaded animation: {} from {}",
                                animation_name, path
                            );
                            player.load_dotlottie_data(&buffer, WIDTH as u32, HEIGHT as u32);

                            let state_machine_id = player
                                .manifest()
                                .unwrap()
                                .state_machines
                                .unwrap()
                                .get(0)
                                .unwrap()
                                .id
                                .clone();

                            println!(
                                "\x1b[32m Loading state machine: {} \x1b[0m",
                                state_machine_id
                            );

                            let success = player.state_machine_load(&state_machine_id);

                            if success {
                                println!(
                                    "\x1b[32m Successfully loaded state machine: {} \x1b[0m",
                                    state_machine_id
                                );

                                println!(
                                    "\x1b[32m Starting state machine: {} \x1b[0m",
                                    state_machine_id
                                );

                                let success = player.state_machine_start();

                                if success {
                                    println!(
                                        "\x1b[32m Successfully started state machine: {} \x1b[0m",
                                        state_machine_id
                                    );
                                } else {
                                    eprintln!(
                                        "\x1b[31m Failed to start state machine: {} \x1b[0m",
                                        state_machine_id
                                    );
                                }
                            } else {
                                // print in red
                                eprintln!(
                                    "\x1b[31m Failed to load state machine: {} \x1b[0m",
                                    state_machine_id
                                );
                            }

                            return Ok(());
                        }
                        Err(e) => {
                            last_error = Some(format!("Failed to read file contents: {}", e));
                            continue;
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(format!("Failed to read file metadata: {}", e));
                    continue;
                }
            },
            Err(e) => {
                last_error = Some(format!("Failed to open file {}: {}", path, e));
                continue;
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| "Unknown error loading animation".to_string())
        .into())
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

    let observer: Arc<dyn StateMachineObserver + 'static> = Arc::new(DummyObserver {});
    let mut animation_manager = AnimationManager::new();

    let lottie_player: DotLottiePlayer = DotLottiePlayer::new(Config {
        background_color: 0xffffffff,
        state_machine_id: "default".to_string(),
        ..Config::default()
    });

    // Load initial animation
    match animation_manager.current() {
        Some(initial_animation) => match load_animation(&lottie_player, &initial_animation) {
            Ok(_) => {
                println!(
                    "Successfully loaded initial animation: {}",
                    initial_animation
                );

                let state_machine_id = lottie_player
                    .manifest()
                    .unwrap()
                    .state_machines
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .id
                    .clone();

                println!(
                    "\x1b[32m Loading state machine: {} \x1b[0m",
                    state_machine_id
                );

                let success = lottie_player.state_machine_load(&state_machine_id);

                if success {
                    println!(
                        "\x1b[32m Successfully loaded state machine: {} \x1b[0m",
                        state_machine_id
                    );

                    println!(
                        "\x1b[32m Starting state machine: {} \x1b[0m",
                        state_machine_id
                    );

                    let success = lottie_player.state_machine_start();

                    if success {
                        println!(
                            "\x1b[32m Successfully started state machine: {} \x1b[0m",
                            state_machine_id
                        );
                    } else {
                        eprintln!(
                            "\x1b[31m Failed to start state machine: {} \x1b[0m",
                            state_machine_id
                        );
                    }
                } else {
                    // print in red
                    eprintln!(
                        "\x1b[31m Failed to load state machine: {} \x1b[0m",
                        state_machine_id
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to load initial animation {}: {}",
                    initial_animation, e
                );
                eprintln!("The program will continue but no animation will be shown initially.");
            }
        },
        None => {
            eprintln!("No animations found in the directories!");
            eprintln!("Please ensure .lottie files are present in either:");
            eprintln!("  - ./src/bin/shared/shared/packaged_state_machines/");
            eprintln!("  - ./shared/packaged_state_machines/");
        }
    }

    let mut timer = Timer::new();
    lottie_player.state_machine_subscribe(observer.clone());
    lottie_player.render();

    let locked_player = Arc::new(RwLock::new(lottie_player));

    let mut rating = 0.0;
    let mut mx = 0.0;
    let mut my = 0.0;
    let mut oo = false;
    let mut left_down = false;
    let mut entered = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Handle animation switching
        if window.is_key_pressed(Key::Right, minifb::KeyRepeat::No) {
            if let Some(next_animation) = animation_manager.next() {
                let p = &mut *locked_player.write().unwrap();

                p.state_machine_stop();

                if let Err(e) = load_animation(p, &next_animation) {
                    eprintln!("Failed to load next animation: {}", e);
                }

                timer.first = false;
            }
        }

        if window.is_key_pressed(Key::Left, minifb::KeyRepeat::No) {
            if let Some(prev_animation) = animation_manager.previous() {
                let p = &mut *locked_player.write().unwrap();

                p.state_machine_stop();

                if let Err(e) = load_animation(p, &prev_animation) {
                    eprintln!("Failed to load previous animation: {}", e);
                }

                timer.first = false;

            }
        }

        let tmp = window.get_mouse_down(MouseButton::Left);
        let mouse_pos = window.get_mouse_pos(minifb::MouseMode::Pass);
        mouse_pos.map(|mouse| {
            if mouse.0 != mx || mouse.1 != my {
                mx = mouse.0;
                my = mouse.1;
            }

            if mx >= 0.0 && mx <= WIDTH as f32 && my >= 0.0 && my <= HEIGHT as f32 {
                if !entered {
                    let event = Event::PointerEnter { x: mx, y: my };
                    let p = &mut *locked_player.write().unwrap();
                    let _m = p.state_machine_post_event(&event);
                }
                entered = true;
            } else {
                if entered {
                    let event = Event::PointerExit { x: mx, y: my };
                    let p = &mut *locked_player.write().unwrap();
                    let _m = p.state_machine_post_event(&event);
                }
                entered = false;
            }
        });

        if !tmp && left_down {
            let event = Event::PointerUp { x: mx, y: my };
            println!("Sending pointer up");
            let p = &mut *locked_player.write().unwrap();
            let _m = p.state_machine_post_event(&event);
        }

        left_down = tmp;

        if left_down {
            let event = Event::PointerDown { x: mx, y: my };
            println!("Sending pointer down");
            let p = &mut *locked_player.write().unwrap();
            let _m = p.state_machine_post_event(&event);
        } else {
            let event = Event::PointerMove { x: mx, y: my };
            let p = &mut *locked_player.write().unwrap();
            let _m = p.state_machine_post_event(&event);
        }

        timer.tick(&*locked_player.read().unwrap());

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::Yes) {
            let p = &mut *locked_player.write().unwrap();
            p.state_machine_set_numeric_trigger("Rating", 1.0);
        }

        if window.is_key_pressed(Key::Enter, minifb::KeyRepeat::No) {
            let p = &mut *locked_player.write().unwrap();
            oo = !oo;
            p.state_machine_set_boolean_trigger("OnOffSwitch", oo);
        }

        let p = &mut *locked_player.write().unwrap();
        let (buffer_ptr, buffer_len) = (p.buffer_ptr(), p.buffer_len());
        let buffer =
            unsafe { std::slice::from_raw_parts(buffer_ptr as *const u32, buffer_len as usize) };
        window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
    }
}
