use dotlottie_player_core::events::Event;
use dotlottie_player_core::{Config, DotLottiePlayer, Observer, StateMachineObserver};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::fs::{self, File};
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::thread;
use std::{env, time::Instant};
use sysinfo::System;

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

struct DummyObserver2;

impl Observer for DummyObserver2 {
    fn on_play(&self) {
        // println!("on_play2");
    }
    fn on_pause(&self) {
        // println!("on_pause2");
    }
    fn on_stop(&self) {
        // println!("on_stop2");
    }
    fn on_frame(&self, frame_no: f32) {
        // println!("on_frame2: {}", frame_no);
    }
    fn on_render(&self, frame_no: f32) {
        // println!("on_render2: {}", frame_no);
    }
    fn on_load(&self) {
        println!("on_load2");
    }
    fn on_load_error(&self) {
        println!("on_load_error2");
    }
    fn on_loop(&self, loop_count: u32) {
        // println!("on_loop2: {}", loop_count);
    }
    fn on_complete(&self) {
        // println!("on_complete2");
    }
}

struct DummyObserver {
    id: u32,
}

impl Observer for DummyObserver {
    fn on_play(&self) {
        // println!("on_play {} ", self.id);
    }
    fn on_pause(&self) {
        // println!("on_pause {} ", self.id);
    }
    fn on_stop(&self) {
        // println!("on_stop {} ", self.id);
    }
    fn on_frame(&self, frame_no: f32) {
        // println!("on_frame {}: {}", self.id, frame_no);
    }
    fn on_render(&self, frame_no: f32) {
        // println!("on_render {}: {}", self.id, frame_no);
    }
    fn on_load(&self) {
        println!("on_load {} ", self.id);
    }
    fn on_load_error(&self) {
        println!("on_load_error {} ", self.id);
    }
    fn on_loop(&self, loop_count: u32) {
        // println!("on_loop {}: {}", self.id, loop_count);
    }
    fn on_complete(&self) {
        // println!("on_complete {} ", self.id);
    }
}

struct SMObserver {}

impl StateMachineObserver for SMObserver {
    fn on_transition(&self, previous_state: String, new_state: String) {
        println!(
            "transition_occured: {:?} -> \n {:?}",
            previous_state, new_state
        );
    }

    fn on_state_entered(&self, entering_state: String) {
        // println!("entering state: {:?}", entering_state);
    }

    fn on_state_exit(&self, leaving_state: String) {
        // println!("exiting state: {:?}", leaving_state);
    }
}

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

        // println!("next_frame: {}", next_frame);
        let updated = animation.set_frame(next_frame);

        if updated || next_frame != self.prev_frame {
            animation.render();
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

    let base_path = env::var("CARGO_MANIFEST_DIR").unwrap();

    // let mut path = path::PathBuf::from(base_path);
    // path.push("src/markers.json");

    let lottie_player: DotLottiePlayer = DotLottiePlayer::new(Config {
        loop_animation: true,
        background_color: 0xffffffff,
        ..Config::default()
    });

    let mut markers = File::open("src/pigeon_with_listeners.lottie").expect("no file found");
    let metadatamarkers =
        fs::metadata("src/pigeon_with_listeners.lottie").expect("unable to read metadata");
    let mut markers_buffer = vec![0; metadatamarkers.len() as usize];
    markers.read(&mut markers_buffer).expect("buffer overflow");

    lottie_player.load_dotlottie_data(&markers_buffer, WIDTH as u32, HEIGHT as u32);

    let observer1: Arc<dyn Observer + 'static> = Arc::new(DummyObserver { id: 1 });
    let observer2: Arc<dyn Observer + 'static> = Arc::new(DummyObserver { id: 2 });

    let observer3: Arc<dyn StateMachineObserver + 'static> = Arc::new(SMObserver {});

    lottie_player.subscribe(observer1.clone());
    lottie_player.subscribe(observer2.clone());

    let mut timer = Timer::new();

    let mut sys = System::new_all();

    let cpu_memory_monitor_thread = thread::spawn(move || {
        loop {
            sys.refresh_all();

            for (pid, process) in sys.processes() {
                if pid.as_u32() == std::process::id() {
                    // println!(
                    //     "CPU: {} % | Memory: {} MB",
                    //     process.cpu_usage(),
                    //     process.memory() / 1024 / 1024,
                    // );
                }
            }

            thread::sleep(std::time::Duration::from_secs(1)); // Adjust sleep duration as needed
        }
    });

    let mut cpu_memory_monitor_timer = Instant::now();

    let message: String = fs::read_to_string("src/pigeon_fsm.json").unwrap();

    // lottie_player.load_state_machine("pigeon_fsm");
    let r = lottie_player.load_state_machine_data(&message);

    println!("Load state machine data -> {}", r);

    lottie_player.start_state_machine();

    println!("is_playing: {}", lottie_player.is_playing());

    lottie_player.state_machine_subscribe(observer3.clone());

    let locked_player = Arc::new(RwLock::new(lottie_player));

    let mut pushed = 10.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        timer.tick(&*locked_player.read().unwrap());

        if window.is_key_down(Key::S) {
            let p = &mut *locked_player.write().unwrap();
            p.start_state_machine();
        }

        if window.is_key_pressed(Key::O, KeyRepeat::No) {
            let pointer_event = Event::OnPointerDown { x: 1.0, y: 1.0 };

            let p = &mut *locked_player.write().unwrap();
            println!("POST EVENT {}", p.post_event(&pointer_event));
            println!("is_playing: {}", p.is_playing());
        }

        if window.is_key_pressed(Key::P, KeyRepeat::Yes) {
            // let string_event = Event::String {
            //     value: "explosion".to_string(),
            // };

            // p.post_event(&string_event);

            if pushed <= 9.0 {
                pushed = 109.0;
            } else {
                pushed -= 2.0;
            }

            let numeric_context_event = Event::SetNumericContext {
                key: "sync_key".to_string(),
                value: pushed,
            };

            let p = &mut *locked_player.write().unwrap();
            let mut r = 0;

            r = p.post_event(&numeric_context_event);

            println!("POST EVENT {}", r);
            println!("is_playing: {}", p.is_playing());
        }

        if cpu_memory_monitor_timer.elapsed().as_secs() >= 1 {
            cpu_memory_monitor_timer = Instant::now();
        }

        let p = &mut *locked_player.write().unwrap();

        let (buffer_ptr, buffer_len) = (p.buffer_ptr(), p.buffer_len());

        let buffer =
            unsafe { std::slice::from_raw_parts(buffer_ptr as *const u32, buffer_len as usize) };

        window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
    }

    cpu_memory_monitor_thread.join().unwrap();
}
