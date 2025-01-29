use dotlottie_rs::events::Event;
use dotlottie_rs::{Config, DotLottiePlayer};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use std::fs::{self, File};
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use sysinfo::System;

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;
struct Timer {
    last_update: Instant,
    prev_frame: f32,
}

impl Timer {
    fn new() -> Self {
        Self {
            last_update: Instant::now(),
            prev_frame: 0.0,
        }
    }

    fn tick(&mut self, animation: &DotLottiePlayer) {
        let next_frame = animation.request_frame();
        let updated = animation.set_frame(next_frame);

        if updated || next_frame != self.prev_frame {
            animation.render();
        }

        self.last_update = Instant::now();
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
        loop_animation: true,
        background_color: 0x00000000,
        ..Config::default()
    });

    let mut markers =
        File::open("src/bin/play-on-hover/star-rating.lottie").expect("no file found");
    let metadatamarkers =
        fs::metadata("src/bin/play-on-hover/star-rating.lottie").expect("unable to read metadata");
    let mut markers_buffer = vec![0; metadatamarkers.len() as usize];
    markers.read(&mut markers_buffer).expect("buffer overflow");

    lottie_player.load_dotlottie_data(&markers_buffer, WIDTH as u32, HEIGHT as u32);

    let mut timer = Timer::new();

    System::new_all();

    let mut cpu_memory_monitor_timer = Instant::now();

    let message: String = fs::read_to_string("./src/bin/play-on-hover/star_hover.json").unwrap();

    let _r = lottie_player.load_state_machine_data(&message);

    lottie_player.start_state_machine();

    lottie_player.render();

    let locked_player = Arc::new(RwLock::new(lottie_player));

    let mut mx = 0.0;
    let mut my = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        timer.tick(&*locked_player.read().unwrap());

        window.get_mouse_pos(MouseMode::Pass).map(|mouse| {
            if mouse.0 != mx || mouse.1 != my {
                mx = mouse.0;
                my = mouse.1;

                let event = Event::OnPointerMove { x: mx, y: my };

                let p = &mut *locked_player.write().unwrap();

                // println!("pointer move {:?}", event);

                let _m = p.post_event(&event);
            }
        });

        if cpu_memory_monitor_timer.elapsed().as_secs() >= 1 {
            cpu_memory_monitor_timer = Instant::now();
        }

        let p = &mut *locked_player.write().unwrap();

        let (buffer_ptr, buffer_len) = (p.buffer_ptr(), p.buffer_len());

        let buffer =
            unsafe { std::slice::from_raw_parts(buffer_ptr as *const u32, buffer_len as usize) };

        window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
    }
}
