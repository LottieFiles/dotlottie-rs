use dotlottie_rs::actions::open_url::OpenUrl;
use dotlottie_rs::events::Event;
use dotlottie_rs::{Config, DotLottiePlayer, StateMachineObserver};
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::fs::{self, File};
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

pub const STATE_MACHINE_NAME: &str = "smiley-slider-state";
pub const ANIMATION_NAME: &str = "smiley-slider";
// pub const STATE_MACHINE_NAME: &str = "rating";
// pub const ANIMATION_NAME: &str = "star_marked";
// pub const STATE_MACHINE_NAME: &str = "StateMachine1";
// pub const ANIMATION_NAME: &str = "pig";

struct Timer {
    last_update: Instant,
    prev_frame: f32,
    first: bool,
}

struct DummyObserver;

impl StateMachineObserver for DummyObserver {
    fn on_transition(&self, previous_state: String, new_state: String) {
        println!("on_transition: {} -> {}", previous_state, new_state);
    }

    fn on_state_entered(&self, entering_state: String) {
        println!("on_state_entered: {}", entering_state);
    }

    fn on_state_exit(&self, leaving_state: String) {
        println!("on_state_exit: {}", leaving_state);
    }

    fn on_custom_event(&self, message: String) {
        println!("custom_event: {}", message);
    }

    fn on_error(&self, error: String) {
        println!("error: {}", error);
    }

    fn on_start(&self) {
        // println!(">>>> start");
    }

    fn on_stop(&self) {
        // println!(">>>> stop");
    }

    fn on_string_input_value_change(
        &self,
        input_name: String,
        old_value: String,
        new_value: String,
    ) {
        println!(
            "string_input_value_change ==> {} : {} -> {}",
            input_name, old_value, new_value
        );
    }

    fn on_numeric_input_value_change(&self, input_name: String, old_value: f32, new_value: f32) {
        println!(
            "numeric_input_value_change ==> {} : {} -> {}",
            input_name, old_value, new_value
        );
    }

    fn on_boolean_input_value_change(&self, input_name: String, old_value: bool, new_value: bool) {
        // println!(
        //     "boolean_input_value_change ==> {} : {} -> {}",
        //     input_name, old_value, new_value
        // );
    }

    fn on_input_fired(&self, input_name: String) {
        todo!()
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

    fn tick(&mut self, animation: &DotLottiePlayer) -> bool {
        let updated = animation.tick();

        // let next_frame = animation.request_frame();
        // animation.set_frame(next_frame);

        if updated || !self.first {
            animation.render();
            self.first = true;

            return updated;
        }

        self.last_update = Instant::now(); // Reset the timer
                                           // self.prev_frame = next_frame;

        // updated
        true
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

    let observer: Arc<dyn StateMachineObserver + 'static> = Arc::new(DummyObserver {});

    let lottie_player: DotLottiePlayer = DotLottiePlayer::new(Config {
        background_color: 0xffffffff,
        state_machine_id: STATE_MACHINE_NAME.to_string(),
        ..Config::default()
    });
    let mut markers = File::open(format!(
        "./src/bin/shared/animations/{}.lottie",
        ANIMATION_NAME
    ))
    .expect("no file found");
    let metadatamarkers = fs::metadata(format!(
        "./src/bin/shared/animations/{}.lottie",
        ANIMATION_NAME
    ))
    .expect("unable to read metadata");
    let mut markers_buffer = vec![0; metadatamarkers.len() as usize];
    markers.read(&mut markers_buffer).expect("buffer overflow");

    lottie_player.load_dotlottie_data(&markers_buffer, WIDTH as u32, HEIGHT as u32);

    let mut timer = Timer::new();

    let state_machine: String = fs::read_to_string(format!(
        "./src/bin/shared/statemachines/{}.json",
        STATE_MACHINE_NAME
    ))
    .unwrap();

    let r = lottie_player.state_machine_load_data(&state_machine);

    println!(
        "🚨 Activate state machine -> {}",
        lottie_player.active_state_machine_id()
    );

    println!("Load state machine data -> {}", r);
    lottie_player.state_machine_subscribe(observer.clone());
    let s = lottie_player.state_machine_start(OpenUrl::default());

    println!("Start state machine -> {}", s);

    println!("is_playing: {}", lottie_player.is_playing());

    lottie_player.render();

    let locked_player = Arc::new(RwLock::new(lottie_player));

    let mut progress = 0.0;
    let mut rating = 1.0;
    // return;

    let mut mx = 0.0;
    let mut my = 0.0;

    let mut oo = false;

    let mut left_down = false;

    let mut entered = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let tmp = window.get_mouse_down(MouseButton::Left);
        let mouse_pos = window.get_mouse_pos(minifb::MouseMode::Pass);
        mouse_pos.map(|mouse| {
            if mouse.0 != mx || mouse.1 != my {
                mx = mouse.0;
                my = mouse.1;
            }

            if mx >= 0.0 && mx <= WIDTH as f32 && my >= 0.0 && my <= HEIGHT as f32 {
                // println!("Sending pointer enter");
                if !entered {
                    let event = Event::PointerEnter { x: mx, y: my };

                    let p = &mut *locked_player.write().unwrap();
                    let _m = p.state_machine_post_event(&event);
                }
                entered = true;
            } else {
                // println!("Sending pointer Exit");
                if entered {
                    let event = Event::PointerExit { x: mx, y: my };

                    let p = &mut *locked_player.write().unwrap();
                    let _m = p.state_machine_post_event(&event);
                }

                entered = false;
            }
        });

        if !tmp && left_down {
            let event = Event::Click { x: mx, y: my };

            // println!("Sending pointer up");
            let p = &mut *locked_player.write().unwrap();
            let _m = p.state_machine_post_event(&event);
        }

        left_down = tmp;

        // left_down = window.get_mouse_down(MouseButton::Left);
        if left_down {
            let event = Event::PointerDown { x: mx, y: my };

            // println!("Sending pointer down");
            let p = &mut *locked_player.write().unwrap();
            let _m = p.state_machine_post_event(&event);
        } else {
            // println!("Sending pointer move {} {}", mx, my);
            let event = Event::PointerMove { x: mx, y: my };
            let p = &mut *locked_player.write().unwrap();
            let _m = p.state_machine_post_event(&event);
        }

        // Send event on key press
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::Yes) {
            let p = &mut *locked_player.write().unwrap();

            progress += 0.01;

            println!("SETTING PROGRESS {}", progress);

            p.state_machine_set_numeric_input("Progress", progress);
        }
        if window.is_key_pressed(Key::Up, minifb::KeyRepeat::Yes) {
            let p = &mut *locked_player.write().unwrap();

            progress -= 0.01;

            println!("SETTING PROGRESS {}", progress);

            p.state_machine_set_numeric_input("Progress", progress);
        }

        if window.is_key_pressed(Key::Enter, minifb::KeyRepeat::No) {
            let p = &mut *locked_player.write().unwrap();

            // oo = !oo;
            // p.state_machine_set_boolean_input("OnOffSwitch", oo);

            rating += 1.0;
            // println!("current state: {}", p.state_machine_current_state());
            p.state_machine_set_numeric_input("rating", rating);
        }
        if window.is_key_pressed(Key::Delete, minifb::KeyRepeat::No) {
            let p = &mut *locked_player.write().unwrap();

            // oo = !oo;
            // p.state_machine_set_boolean_input("OnOffSwitch", oo);

            rating -= 1.0;
            // println!("current state: {}", p.state_machine_current_state());
            p.state_machine_set_numeric_input("rating", rating);
        }

        let updated = timer.tick(&*locked_player.read().unwrap());

        if updated {
            let p = &mut *locked_player.write().unwrap();

            let (buffer_ptr, buffer_len) = (p.buffer_ptr(), p.buffer_len());

            let buffer = unsafe {
                std::slice::from_raw_parts(buffer_ptr as *const u32, buffer_len as usize)
            };

            window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
        }
    }
}
