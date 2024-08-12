use dotlottie_player_core::events::Event;
use dotlottie_player_core::{Config, DotLottiePlayer, Observer, StateMachineObserver};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use std::fs::{self, File};
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::thread;
use std::{env, time::Instant};
use sysinfo::System;

pub const WIDTH: usize = 1000;
pub const HEIGHT: usize = 1000;

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

        // println!("next_frame: {}", next_frame);
        let updated = animation.set_frame(next_frame);

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

    let mut markers = File::open("src/star-rating.lottie").expect("no file found");
    let metadatamarkers = fs::metadata("src/star-rating.lottie").expect("unable to read metadata");
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
    // let r = lottie_player.load_state_machine_data(&message);

    // println!("Load state machine data -> {}", r);

    lottie_player.start_state_machine();

    println!("is_playing: {}", lottie_player.is_playing());

    lottie_player.render();

    // lottie_player.state_machine_subscribe(observer3.clone());

    let locked_player = Arc::new(RwLock::new(lottie_player));

    let mut pushed = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let left_down = window.get_mouse_down(MouseButton::Left);
        // println!("is left down? {}", left_down);

        if left_down {
            let mut mx = 0.0;
            let mut my = 0.0;
            window.get_mouse_pos(MouseMode::Clamp).map(|mouse| {
                // println!("x {} y {}", mouse.0, mouse.1);
                mx = mouse.0;
                my = mouse.1;
            });

            let p = &mut *locked_player.write().unwrap();
            if p.hit_check("pad1", mx, my) {
                // println!("hit");
                let mut c = p.config();

                c.segment = vec![0.0, 20.0];
                c.loop_animation = false;
                p.set_config(c);

                p.play();
            } else {
                println!("not hit");
            }
            if p.hit_check("bar", mx, my) {
                // println!("hit");
                let mut c = p.config();

                c.segment = vec![56.0, 82.0];
                c.loop_animation = false;
                p.set_config(c);

                p.play();
            } else {
                println!("not hit");
            }
        }

        timer.tick(&*locked_player.read().unwrap());

        if window.is_key_down(Key::S) {
            let p = &mut *locked_player.write().unwrap();
            p.start_state_machine();
        }

        if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            pushed = 1.0;

            let pointer_event = Event::Numeric { value: pushed };

            let p = &mut *locked_player.write().unwrap();

            let m = p.post_event(&pointer_event);

            println!("POST EVENT {:?}", m);
        }

        if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            pushed = 2.0;

            let pointer_event = Event::Numeric { value: pushed };

            let p = &mut *locked_player.write().unwrap();

            let m = p.post_event(&pointer_event);

            println!("POST EVENT {:?}", m);
        }
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            pushed = 3.0;

            let pointer_event = Event::Numeric { value: pushed };

            let p = &mut *locked_player.write().unwrap();

            let m = p.post_event(&pointer_event);

            println!("POST EVENT {:?}", m);
        }
        if window.is_key_pressed(Key::Key4, KeyRepeat::No) {
            pushed = 4.0;

            let pointer_event = Event::Numeric { value: pushed };

            let p = &mut *locked_player.write().unwrap();

            let m = p.post_event(&pointer_event);

            println!("POST EVENT {:?}", m);
        }
        if window.is_key_pressed(Key::Key5, KeyRepeat::No) {
            pushed = 5.0;

            let pointer_event = Event::Numeric { value: pushed };

            let p = &mut *locked_player.write().unwrap();

            let m = p.post_event(&pointer_event);

            println!("POST EVENT {:?}", m);
        }
        if window.is_key_pressed(Key::Key6, KeyRepeat::No) {
            pushed = 6.0;

            let pointer_event = Event::Numeric { value: pushed };

            let p = &mut *locked_player.write().unwrap();

            let m = p.post_event(&pointer_event);

            println!("POST EVENT {:?}", m);
        }
        if window.is_key_pressed(Key::Key7, KeyRepeat::No) {
            let key = "jump";

            let string_event = Event::String {
                value: key.to_string(),
            };

            let p = &mut *locked_player.write().unwrap();

            let m = p.post_event(&string_event);
        }

        if window.is_key_pressed(Key::O, KeyRepeat::No) {
            let pointer_event = Event::OnPointerDown { x: 1.0, y: 1.0 };

            let p = &mut *locked_player.write().unwrap();

            let m = p.post_event(&pointer_event);

            println!("POST EVENT {:?}", m);
        }

        if window.is_key_pressed(Key::I, KeyRepeat::No) {
            let pointer_event = Event::OnPointerUp { x: 1.0, y: 1.0 };

            let p = &mut *locked_player.write().unwrap();

            let m = p.post_event(&pointer_event);

            println!("POST EVENT {:?}", m);
        }
        if window.is_key_pressed(Key::U, KeyRepeat::No) {
            let pointer_event = Event::OnPointerMove { x: 1.0, y: 1.0 };

            let p = &mut *locked_player.write().unwrap();

            let m = p.post_event(&pointer_event);

            println!("POST EVENT {:?}", m);
        }
        if window.is_key_pressed(Key::Y, KeyRepeat::No) {
            let pointer_event = Event::String {
                value: "play_all".to_string(),
            };

            let p = &mut *locked_player.write().unwrap();

            let m = p.post_event(&pointer_event);

            println!("POST EVENT {:?}", m);
        }

        if window.is_key_pressed(Key::P, KeyRepeat::Yes) {
            let p = &mut *locked_player.write().unwrap();
            let mut r = 0;

            if pushed >= p.total_frames() as f32 {
                pushed = 0.0;
            } else {
                pushed += 1.0;
            }

            let format = format!("SetNumericContext: sync_key {}", pushed);
            // r = p.post_serialized_event(format.to_string());

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
