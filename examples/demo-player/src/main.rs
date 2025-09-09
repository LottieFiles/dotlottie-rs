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
        let threads = std::thread::available_parallelism().unwrap().get() as u32;

        println!("Using {} threads", threads);

        let player = DotLottiePlayer::with_threads(
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
            .tween_to_marker(&marker.name, Some(1.0), Some(EASE_LINEAR.to_vec()));
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

    let mut player = Player::new("src/sm_theme_action.lottie");
    // let mut player = Player::new("src/emoji.lottie");

    let mut ran = false;

    let mut i = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if !ran {
            player.player.eval(
                "
                // State machine configuration
                var themes = ['Water', 'air', 'earth'];
                var currentThemeIndex = 0;
                var frameCount = 0;
                var autoMode = true;
                var framesPerTheme = 60; // Change theme every 60 frames in auto mode
                var animationSpeed = 1;

                // State machine functions (without 'var' to make them global)
                switchToTheme = function(newTheme) {
                    var themeIndex = themes.indexOf(newTheme);
                    if (themeIndex != -1 && themeIndex != currentThemeIndex) {
                        currentThemeIndex = themeIndex;
                        setTheme(themes[currentThemeIndex]);
                        frameCount = 0; // Reset frame counter when manually switching
                    }
                };

                nextTheme = function() {
                    currentThemeIndex = (currentThemeIndex + 1) % themes.length;
                    setTheme(themes[currentThemeIndex]);
                    frameCount = 0;
                };

                toggleAutoMode = function() {
                    autoMode = !autoMode;
                    frameCount = 0; // Reset counter when toggling mode
                };

                updateStateMachine = function() {
                    frameCount = frameCount + animationSpeed;

                    // Auto-cycle themes if in auto mode
                    if (autoMode && frameCount >= framesPerTheme) {
                        nextTheme();
                    }

                    // Always update animation frame
                    setFrame(frameCount);
                };

                setSpeed = function(speed) {
                    animationSpeed = speed;
                };

                // Initialize
                switchToTheme('Water');
                ", 
                true,
            );
            ran = true;
        }

        // Manual theme switching
        if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            player.player.eval("switchToTheme('Water');", true);
        }
        if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            player.player.eval("switchToTheme('Air');", true);
        }
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            player.player.eval("switchToTheme('Earth');", true);
        }
        if window.is_key_pressed(Key::Key4, KeyRepeat::No) {
            player.player.eval(
                "setConfig({
                        loop_animation: true,
                        speed: 3,
                        segment: [10, 50],
                        autoplay: true,
                        animation_id: 'blush',
                        });",
                true,
            );
        }

        // State machine controls
        if window.is_key_pressed(Key::A, KeyRepeat::No) {
            // Toggle auto-cycling mode
            player.player.eval("toggleAutoMode();", true);
        }

        if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            player.player.eval("switchToTheme('Water');", true);
        }
        if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            player.player.eval("switchToTheme('air');", true);
        }
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            player.player.eval("switchToTheme('earth');", true);
        }

        // Update the state machine every frame
        player.player.eval("updateStateMachine();", true);

        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            player.player.eval(&"stop();".to_string(), false);
        }
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            player.player.eval(&"play();".to_string(), false);
        }
        if window.is_key_pressed(Key::T, KeyRepeat::No) {
            if !ran {
                player.player.eval("this.theme = 'Water'; i = 0;", true);
            }

            player
                .player
                .eval("i += 1; setTheme(this.theme); setFrame(i);", true);

            ran = true;
        }
        if window.is_key_pressed(Key::Right, KeyRepeat::No) {
            player.next_marker();
        }

        // if player.update() {
        player.update();
        window
            .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window");
        // }
    }
}
