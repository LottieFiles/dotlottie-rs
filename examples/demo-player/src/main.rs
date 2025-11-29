use dotlottie_rs::{Config, DotLottiePlayer};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::Instant;

const WIDTH: usize = 600;
const HEIGHT: usize = 600;
const EASE_LINEAR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

struct Player {
    player: DotLottiePlayer,
    current_marker: usize,
    last_update: Instant,
}

impl Player {
    fn new(animation_path: &str) -> Self {
        let threads = std::thread::available_parallelism().unwrap().get() as u32;

        println!("Using {} threads", threads);

        let mut player = DotLottiePlayer::new(
            Config {
                autoplay: true,
                loop_animation: true,
                ..Default::default()
            },
            threads,
        );

        let is_dotlottie = animation_path.ends_with(".lottie");

        if is_dotlottie {
            let data = std::fs::read(animation_path).unwrap();
            player.load_dotlottie_data(&data, WIDTH as u32, HEIGHT as u32);
        } else {
            player.load_animation_path(animation_path, WIDTH as u32, HEIGHT as u32);
        }

        for marker in player.markers() {
            println!("Marker '{}' at frame {}", marker.name, marker.time);
        }

        if let Some(marker) = player.markers().first() {
            let mut config = player.config();
            config.marker = marker.name.clone();
            player.set_config(config);
        }

        Self {
            player,
            current_marker: 0,
            last_update: Instant::now(),
        }
    }

    fn update(&mut self) -> bool {
        let updated = self.player.tick();
        self.last_update = Instant::now();
        updated
    }

    fn play_marker(&mut self, index: usize) {
        let markers = self.player.markers();
        if index >= markers.len() || index == self.current_marker {
            return;
        }

        let marker = &markers[index];
        // self.player.tween_to(marker.time, 1.0, EASE_LINEAR);
        self.player
            .tween_to_marker(&marker.name, Some(1.0), Some(EASE_LINEAR));
        println!("Playing marker: '{}'", marker.name);
        let mut config = self.player.config();
        config.marker = marker.name.clone();
        self.player.set_config(config);

        self.current_marker = index;
    }

    fn next_marker(&mut self) {
        if self.player.is_tweening() {
            return;
        }
        let next = (self.current_marker + 1) % self.player.markers().len();
        self.play_marker(next);
    }

    fn frame_buffer(&self) -> &[u32] {
        let (ptr, len) = (self.player.buffer().as_ptr(), self.player.buffer().len());
        unsafe { std::slice::from_raw_parts(ptr as *const u32, len as usize) }
    }
}

fn main() {
    println!("\nDemo Player Controls:");
    println!("  T - Apply text override slot (test set_slots fix)");
    println!("  U - Clear slots (reset to original)");
    println!("  P - Play");
    println!("  S - Stop");
    println!("  → - Next marker");
    println!("  ESC - Exit\n");

    let mut window = Window::new(
        "Lottie Player Demo (ESC to exit, ←/→ to change markers, P to play, S to stop)",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    let mut player = Player::new("src/text.json");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::U, KeyRepeat::No) {
            player.player.set_slots("");
        }
        if window.is_key_pressed(Key::T, KeyRepeat::No) {
            player.player.set_slots(r#"{"my_text": { "p": { "k": [{ "s": { "f": "cartoon", "fc": [0, 1, 0, 1], "s": 50, "t": "overridden", "j": 0 }, "t": 0 }] } } }"#);
        }
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            player.player.play();
        }
        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            player.player.stop();
        }
        if window.is_key_pressed(Key::Right, KeyRepeat::No) {
            player.next_marker();
        }

        if player.update() {
            window
                .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
                .expect("Failed to update window");
        }
    }
}
