use dotlottie_rs::{Config, DotLottiePlayer};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::{path::Path, time::Instant};

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
        let mut player = DotLottiePlayer::new(Config {
            autoplay: true,
            loop_animation: true,
            ..Default::default()
        });

        player.load_animation_path(animation_path, WIDTH as u32, HEIGHT as u32);

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
            .tween_to_marker(&marker.name, 1.0, EASE_LINEAR.to_vec());
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
        let (ptr, len) = (self.player.buffer_ptr(), self.player.buffer_len());
        unsafe { std::slice::from_raw_parts(ptr as *const u32, len as usize) }
    }
}

fn main() {
    let mut window = Window::new(
        "Lottie Player Demo (ESC to exit, ←/→ to change markers, P to play, S to stop)",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    let mut player = Player::new("src/emoji.json");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            player.player.stop();
        }
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            player.player.play();
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
