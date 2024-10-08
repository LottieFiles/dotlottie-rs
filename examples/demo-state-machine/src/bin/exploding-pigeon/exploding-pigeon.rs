use dotlottie_rs::events::Event;
use dotlottie_rs::{Config, DotLottiePlayer};
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::fs::{self, File};
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

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
        loop_animation: true,
        background_color: 0xffffffff,
        ..Config::default()
    });

    let mut markers =
        File::open("./src/bin/exploding-pigeon/pigeon.lottie").expect("no file found");
    let metadatamarkers =
        fs::metadata("./src/bin/exploding-pigeon/pigeon.lottie").expect("unable to read metadata");
    let mut markers_buffer = vec![0; metadatamarkers.len() as usize];
    markers.read(&mut markers_buffer).expect("buffer overflow");

    lottie_player.load_dotlottie_data(&markers_buffer, WIDTH as u32, HEIGHT as u32);

    let mut timer = Timer::new();

    let r = lottie_player.load_state_machine("pigeon_fsm");

    println!("Load state machine data -> {}", r);

    lottie_player.start_state_machine();

    println!("is_playing: {}", lottie_player.is_playing());

    lottie_player.render();

    let locked_player = Arc::new(RwLock::new(lottie_player));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let left_down = window.get_mouse_down(MouseButton::Left);
        if left_down {
            let pointer_event = Event::OnPointerDown { x: 0.0, y: 0.0 };

            let p = &mut *locked_player.write().unwrap();

            p.post_event(&pointer_event);
        }

        timer.tick(&*locked_player.read().unwrap());

        let p = &mut *locked_player.write().unwrap();

        let (buffer_ptr, buffer_len) = (p.buffer_ptr(), p.buffer_len());

        let buffer =
            unsafe { std::slice::from_raw_parts(buffer_ptr as *const u32, buffer_len as usize) };

        window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
    }
}
