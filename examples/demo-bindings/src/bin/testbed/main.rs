use dotlottie_rs::{
    actions::open_url_policy::OpenUrlPolicy, parser::GradientStop, Config, DotLottiePlayer, Event,
};
use minifb::{Key, KeyRepeat, MouseButton, Window, WindowOptions};
use std::time::Instant;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;
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

        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            loop_animation: true,
            background_color: 0xffffffff,
            ..Default::default()
        });

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

        // if let Some(marker) = player.markers().first() {
        //     let mut config = player.config();
        //     config.marker = marker.name.clone();
        //     player.set_config(config);
        // }

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

// pub const ANIMATION_NAME: &str = "test_inputs_ball_scalar";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";

pub const ANIMATION_NAME: &str = "test_inputs_ball_gradient";
pub const BINDING_FILE_NAME: &str = "inputs";
pub const THEMING_FILE_NAME: &str = "theme";

// pub const ANIMATION_NAME: &str = "test_inputs_ball_color";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";

// pub const ANIMATION_NAME: &str = "test_inputs_ball_vector";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";

// pub const ANIMATION_NAME: &str = "test_ball_gradient";
// pub const ANIMATION_NAME: &str = "test_inputs_text";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const SM_FILE_NAME: &str = "toggleButton";
// pub const THEMING_FILE_NAME: &str = "theme";
// //
// pub const ANIMATION_NAME: &str = "test_ball_scalar";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";

// pub const ANIMATION_NAME: &str = "test_inputs_star_sm";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "";
// pub const SM_FILE_NAME: &str = "starRating";

fn main() {
    let mut window = Window::new(
        "Lottie Player Demo (ESC to exit, ←/→ to change markers, P to play, S to stop)",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    let mut player = Player::new(&format!("./src/bin/testbed/{}.lottie", ANIMATION_NAME));

    player.player.global_inputs_load(BINDING_FILE_NAME);

    // let sml = player.player.state_machine_load(SM_FILE_NAME);
    // let sms = player.player.state_machine_start(OpenUrlPolicy::default());

    // println!("State machine loaded: {}", sml);
    // println!("State machine started: {}", sms);

    // let binding_file_path = format!( "./src/bin/testbed/unzip/g/{}.json", BINDING_FILE_NAME);
    // let binding_file_data = std::fs::read_to_string(&binding_file_path).expect(&format!(
    //     "Failed to read binding file: {}",
    //     binding_file_path
    // ));
    // player.player.global_inputs_load_data(&binding_file_data);

    let st = player.player.set_theme(THEMING_FILE_NAME);
    println!("Set theme: {}", st);

    let mut mx = 0.0;
    let mut my = 0.0;

    let mut left_down = false;
    let mut toggle = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mouse_down = window.get_mouse_down(MouseButton::Left);

        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            let mut gradient_storage = vec![];
            gradient_storage.push(GradientStop {
                color: vec![0.0, 1.0, 1.0, 1.0],
                offset: 0.0,
            });
            gradient_storage.push(GradientStop {
                color: vec![1.0, 0.0, 1.0, 1.0],
                offset: 1.0,
            });
            gradient_storage.push(GradientStop {
                color: vec![1.0, 0.0, 1.0, 1.0],
                offset: 1.0,
            });
            player
                .player
                .global_inputs_set_gradient("ball", &gradient_storage);

            // Scalar
            player.player.global_inputs_set_scalar("ball", 90.0);

            toggle = !toggle;
            player
                .player
                .global_inputs_set_boolean("OnOffSwitch", toggle);

            // Vector
            // player.player.mutate_color_binding("face", &[0.5, 0.5, 0.5]);

            // Text
            // player
            //     .player
            //     .mutate_text_binding("interaction_title", "New title");
        }

        if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            player.player.global_inputs_set_scalar("rating", 1.0);
        }
        if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            player.player.global_inputs_set_scalar("rating", 2.0);
        }
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            player.player.global_inputs_set_scalar("rating", 3.0);
        }
        if window.is_key_pressed(Key::Key4, KeyRepeat::No) {
            player.player.global_inputs_set_scalar("rating", 4.0);
        }
        if window.is_key_pressed(Key::Key5, KeyRepeat::No) {
            player.player.global_inputs_set_scalar("rating", 5.0);
        }

        if !mouse_down && left_down {
            println!("sending click");
            let event = Event::Click { x: mx, y: my };

            player.player.state_machine_post_event(&event);
        }
        left_down = mouse_down;

        let mouse_pos = window.get_mouse_pos(minifb::MouseMode::Discard);
        mouse_pos.map(|mouse| {
            if mouse.0 != mx || mouse.1 != my {
                mx = mouse.0;
                my = mouse.1;
            }
        });

        if mx != 0.0 && my != 0.0 {
            let mut gradient_storage = vec![];
            gradient_storage.push(GradientStop {
                color: vec![0.0, 1.0, 1.0, 1.0],
                offset: 0.0,
            });
            gradient_storage.push(GradientStop {
                color: vec![1.0, 0.0, 1.0, 1.0],
                offset: ((mx / (WIDTH as f32) * 100.0) / 100.0) as f64,
            });
            gradient_storage.push(GradientStop {
                color: vec![1.0, 0.0, 1.0, 1.0],
                offset: 1.0,
            });
            player
                .player
                .global_inputs_set_gradient("ball", &gradient_storage);
            player
                .player
                .global_inputs_set_vector("wand_pos", &[mx.into(), my.into()]);
        }

        player.update();
        window
            .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window");
        // }
    }
}
