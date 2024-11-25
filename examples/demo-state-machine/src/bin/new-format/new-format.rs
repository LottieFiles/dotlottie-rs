use dotlottie_rs::events::Event;
use dotlottie_rs::{Config, DotLottiePlayer};
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::fs::{self, File};
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

pub const STATE_MACHINE_NAME: &str = "new-actions";
pub const ANIMATION_NAME: &str = "smileys";

struct Timer {
    last_update: Instant,
    prev_frame: f32,
    first: bool,
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

        self.last_update = Instant::now(); // Reset the timer
        self.prev_frame = next_frame;
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

    let lottie_player: DotLottiePlayer = DotLottiePlayer::new(Config {
        background_color: 0xffffffff,
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

    println!("Load state machine data -> {}", r);

    let s = lottie_player.state_machine_start();

    println!("Start state machine -> {}", s);

    println!("is_playing: {}", lottie_player.is_playing());

    lottie_player.render();

    let locked_player = Arc::new(RwLock::new(lottie_player));

    let mut rating = 0.0;
    // return;

    let mut mx = 0.0;
    let mut my = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let left_down = window.get_mouse_down(MouseButton::Left);
        if left_down {
            // if left_down {
            window.get_mouse_pos(minifb::MouseMode::Pass).map(|mouse| {
                if mouse.0 != mx || mouse.1 != my {
                    mx = mouse.0;
                    my = mouse.1;

                    let event = Event::PointerDown { x: mx, y: my };

                    let p = &mut *locked_player.write().unwrap();
                    let _m = p.state_machine_post_event(&event);
                }
            });

            // Get the coordinates
            // let (x, y) = window.get_mouse_pos(minifb::MouseMode::Pass).unwrap();

            // let pointer_event = Event::PointerDown { x, y };

            // let p = &mut *locked_player.write().unwrap();

            // println!("PointerDown -> x: {}, y: {}", x, y);

            // p.post_event(&pointer_event);
        }

        timer.tick(&*locked_player.read().unwrap());

        // Send event on key press
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::Yes) {
            let p = &mut *locked_player.write().unwrap();

            p.state_machine_set_numeric_trigger("Rating", 1.0);
        }

        if window.is_key_pressed(Key::Enter, minifb::KeyRepeat::No) {
            let p = &mut *locked_player.write().unwrap();

            let r = p.state_machine_set_numeric_trigger("Progress", rating);
            let m = p.state_machine_set_propagate_events(false);
            p.state_machine_fire_event("Step");
            // println!("{}", r);
            println!("{}", m);
            rating += 1.0;
            // p.state_machine_fire_event("Step");
        }
        let p = &mut *locked_player.write().unwrap();

        let (buffer_ptr, buffer_len) = (p.buffer_ptr(), p.buffer_len());

        let buffer =
            unsafe { std::slice::from_raw_parts(buffer_ptr as *const u32, buffer_len as usize) };

        window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
    }
}
