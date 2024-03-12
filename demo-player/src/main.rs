use dotlottie_player_core::{Config, DotLottiePlayer, Mode, Observer};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::fs::{self, File};
use std::io::Read;
use std::sync::Arc;
use std::thread;
use std::{env, path, time::Instant};
use sysinfo::System;

pub const WIDTH: usize = 500;
pub const HEIGHT: usize = 500;

struct DummyObserver2;

impl Observer for DummyObserver2 {
    fn on_play(&self) {
        println!("on_play2");
    }
    fn on_pause(&self) {
        println!("on_pause2");
    }
    fn on_stop(&self) {
        println!("on_stop2");
    }
    fn on_frame(&self, frame_no: f32) {
        println!("on_frame2: {}", frame_no);
    }
    fn on_render(&self, frame_no: f32) {
        println!("on_render2: {}", frame_no);
    }
    fn on_load(&self) {
        println!("on_load2");
    }
    fn on_load_error(&self) {
        println!("on_load_error2");
    }
    fn on_loop(&self, loop_count: u32) {
        println!("on_loop2: {}", loop_count);
    }
    fn on_complete(&self) {
        println!("on_complete2");
    }
}

struct DummyObserver {
    id: u32,
}

impl Observer for DummyObserver {
    fn on_play(&self) {
        println!("on_play {} ", self.id);
    }
    fn on_pause(&self) {
        println!("on_pause {} ", self.id);
    }
    fn on_stop(&self) {
        println!("on_stop {} ", self.id);
    }
    fn on_frame(&self, frame_no: f32) {
        println!("on_frame {}: {}", self.id, frame_no);
    }
    fn on_render(&self, frame_no: f32) {
        println!("on_render {}: {}", self.id, frame_no);
    }
    fn on_load(&self) {
        println!("on_load {} ", self.id);
    }
    fn on_load_error(&self) {
        println!("on_load_error {} ", self.id);
    }
    fn on_loop(&self, loop_count: u32) {
        println!("on_loop {}: {}", self.id, loop_count);
    }
    fn on_complete(&self) {
        println!("on_complete {} ", self.id);
    }
}

struct Timer {
    last_update: Instant,
}

impl Timer {
    fn new() -> Self {
        Self {
            last_update: Instant::now(),
        }
    }

    fn tick(&mut self, animation: &mut DotLottiePlayer) {
        let next_frame = animation.request_frame();

        // println!("next_frame: {}", next_frame);
        let updated = animation.set_frame(next_frame);

        if updated {
            animation.render();
        }

        self.last_update = Instant::now(); // Reset the timer
    }
}

fn main() {
    let mut window =
        Window::new(
            "dotLottie rust demo - ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    let base_path = env::var("CARGO_MANIFEST_DIR").unwrap();

    let mut path = path::PathBuf::from(base_path);
    path.push("src/markers.json");

    let mut lottie_player: DotLottiePlayer = DotLottiePlayer::new(Config {
        mode: Mode::Forward,
        loop_animation: true,
        speed: 1.0,
        use_frame_interpolation: true,
        autoplay: true,
        segments: vec![],
        background_color: 0xffffffff,
        marker: "feather".to_string(),
    });

    // read dotlottie in to vec<u8>
    let mut f =
        File::open(
            // "src/emoji.lottie"
            "src/theming_example.lottie",
        )
        .expect("no file found");
    let metadata =
        fs::metadata(
            // "src/emoji.lottie"
            "src/theming_example.lottie",
        )
        .expect("unable to read metadata");

    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    let mut markers = File::open("src/markers.json").expect("no file found");
    let metadatamarkers = fs::metadata("src/markers.json").expect("unable to read metadata");
    let mut markersBuffer = vec![0; metadatamarkers.len() as usize];
    markers.read(&mut markersBuffer).expect("buffer overflow");
    let string = String::from_utf8(markersBuffer.clone()).unwrap();
    // lottie_player.load_animation_data(string.as_str(), WIDTH as u32, HEIGHT as u32);
    // println!("{:?}", Some(lottie_player.manifest()));

    lottie_player.load_animation_path(
        path.as_path().to_str().unwrap(),
        WIDTH as u32,
        HEIGHT as u32,
    );

    // lottie_player.load_dotlottie_data(&buffer, WIDTH as u32, HEIGHT as u32);
    // lottie_player.load_animation("confused", WIDTH as u32, HEIGHT as u32);

    let observer1: Arc<dyn Observer + 'static> = Arc::new(DummyObserver { id: 1 });
    let observer2: Arc<dyn Observer + 'static> = Arc::new(DummyObserver { id: 2 });

    lottie_player.subscribe(observer1.clone());
    lottie_player.subscribe(observer2.clone());

    let mut timer = Timer::new();

    let mut i = 0;

    let mut sys = System::new_all();

    let cpu_memory_monitor_thread = thread::spawn(move || {
        loop {
            sys.refresh_all();

            for (pid, process) in sys.processes() {
                if pid.as_u32() == std::process::id() {
                    println!(
                        "CPU: {} % | Memory: {} MB",
                        process.cpu_usage(),
                        process.memory() / 1024 / 1024,
                    );
                }
            }

            thread::sleep(std::time::Duration::from_secs(1)); // Adjust sleep duration as needed
        }
    });

    let mut cpu_memory_monitor_timer = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        timer.tick(&mut lottie_player);

        if window.is_key_down(Key::S) {
            lottie_player.stop();
        }
        if window.is_key_down(Key::P) {
            lottie_player.play();
        }

        if window.is_key_down(Key::J) {
            let updated = lottie_player.set_frame(20.0);
            if updated {
                lottie_player.render();
            }
        }

        if window.is_key_down(Key::Left) {
            let mut config = lottie_player.config();

            config.mode = Mode::Bounce;
            lottie_player.set_config(config)
        }

        if window.is_key_pressed(Key::T, KeyRepeat::No) {
            if let Some(manifest) = lottie_player.manifest() {
                if let Some(themes) = manifest.themes {
                    let theme = &themes[0];

                    lottie_player.load_theme(&theme.id.as_str());
                }
            }
        }

        if window.is_key_pressed(Key::Right, KeyRepeat::No) {
            if let Some(manifest) = lottie_player.manifest() {
                println!("{:?}", i);

                if i >= manifest.animations.len() - 1 {
                    i = 0;
                } else {
                    i += 1;
                }

                let animation_id = manifest.animations[i].id.clone();

                lottie_player.load_animation(animation_id.as_str(), WIDTH as u32, HEIGHT as u32);
            }
        }

        if window.is_key_pressed(Key::L, KeyRepeat::No) {
            lottie_player = DotLottiePlayer::new(Config {
                mode: Mode::ReverseBounce,
                loop_animation: true,
                speed: 1.0,
                use_frame_interpolation: true,
                autoplay: true,
                segments: vec![10.0, 45.0],
                background_color: 0xffffffff,
                marker: "".to_string(),
            });

            lottie_player.load_animation_data(&string, WIDTH as u32, HEIGHT as u32);
        }

        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            lottie_player.load_dotlottie_data(&buffer, WIDTH as u32, HEIGHT as u32);
        }

        if window.is_key_down(Key::Up) {
            lottie_player.unsubscribe(&observer1);
        }

        if window.is_key_down(Key::Down) {
            lottie_player.unsubscribe(&observer2);
        }

        if window.is_key_pressed(Key::Q, KeyRepeat::No) {
            let mut config = lottie_player.config();

            config.marker = "bird".to_string();

            lottie_player.set_config(config);
        }

        if window.is_key_pressed(Key::W, KeyRepeat::No) {
            let mut config = lottie_player.config();

            config.marker = "explosion".to_string();

            lottie_player.set_config(config);
        }

        if window.is_key_pressed(Key::E, KeyRepeat::No) {
            let mut config = lottie_player.config();

            config.marker = "feather".to_string();

            lottie_player.set_config(config);
        }

        if cpu_memory_monitor_timer.elapsed().as_secs() >= 1 {
            cpu_memory_monitor_timer = Instant::now();
        }

        let (buffer_ptr, buffer_len) = (lottie_player.buffer_ptr(), lottie_player.buffer_len());

        let buffer =
            unsafe { std::slice::from_raw_parts(buffer_ptr as *const u32, buffer_len as usize) };

        window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
    }

    cpu_memory_monitor_thread.join().unwrap();
}
