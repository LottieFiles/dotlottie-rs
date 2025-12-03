use dotlottie_rs::{
    actions::open_url_policy::OpenUrlPolicy, Config, DotLottiePlayer, Event, GradientStop,
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
    text_input: String,
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
            text_input: "".to_string(),
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

    fn handle_text_input(&mut self, key: Key) {
        let char_to_add = match key {
            Key::A => Some('a'),
            Key::B => Some('b'),
            Key::C => Some('c'),
            Key::D => Some('d'),
            Key::E => Some('e'),
            Key::F => Some('f'),
            Key::G => Some('g'),
            Key::H => Some('h'),
            Key::I => Some('i'),
            Key::J => Some('j'),
            Key::K => Some('k'),
            Key::L => Some('l'),
            Key::M => Some('m'),
            Key::N => Some('n'),
            Key::O => Some('o'),
            Key::P => Some('p'),
            Key::Q => Some('q'),
            Key::R => Some('r'),
            Key::S => Some('s'),
            Key::T => Some('t'),
            Key::U => Some('u'),
            Key::V => Some('v'),
            Key::W => Some('w'),
            Key::X => Some('x'),
            Key::Y => Some('y'),
            Key::Z => Some('z'),
            Key::Space => Some(' '),
            Key::Backspace => {
                self.text_input.pop();
                println!("Current text: '{}'", self.text_input);
                // Only update if there's still text remaining, or use a space as placeholder
                if !self.text_input.is_empty() {
                    self.player
                        .global_inputs_set_string("text", &self.text_input);
                } else {
                    // Use a single space instead of empty string to avoid the crash
                    self.player.global_inputs_set_string("text", " ");
                }
                None
            }
            _ => None,
        };

        if let Some(c) = char_to_add {
            self.text_input.push(c);
            println!("Current text: '{}'", self.text_input);
            self.player
                .global_inputs_set_string("text", &self.text_input);
        }
    }
}

// pub const ANIMATION_NAME: &str = "test_inputs_ball_color_animated";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";

// pub const ANIMATION_NAME: &str = "test_inputs_ball_gradient_animated";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";

// pub const ANIMATION_NAME: &str = "test_inputs_sheet_gradient_animated";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";

// pub const ANIMATION_NAME: &str = "test_vector_global_input";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";

// ----- TEXT -----
// pub const ANIMATION_NAME: &str = "test_inputs_text_static";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";
// ----- END TEXT -----

// ----- SCALAR -----
// pub const ANIMATION_NAME: &str = "test_inputs_ball_numeric_static";

// pub const ANIMATION_NAME: &str = "test_inputs_ball_numeric_animated";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";
// ----- END SCALAR -----

// ----- GRADIENT -----
pub const ANIMATION_NAME: &str = "test_inputs_sheet_gradient_static";
// pub const ANIMATION_NAME: &str = "test_inputs_sheet_gradient_animated";

pub const BINDING_FILE_NAME: &str = "inputs";
pub const THEMING_FILE_NAME: &str = "theme";
// ----- END GRADIENT -----

// pub const ANIMATION_NAME: &str = "test_inputs_text_animated";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";

// pub const ANIMATION_NAME: &str = "test_inputs_ball_gradient_static";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";

// pub const ANIMATION_NAME: &str = "test_inputs_ball_scalar";
// pub const BINDING_FILE_NAME: &str = "inputs";
// pub const THEMING_FILE_NAME: &str = "theme";

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

    // player.player.global_inputs_load(BINDING_FILE_NAME);

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

    if player.player.is_loaded() {
        let st = player.player.set_theme(THEMING_FILE_NAME);
        println!("Set theme: {}", st);
    }
    // let l = player.player.global_inputs_load(BINDING_FILE_NAME);
    // println!("L : {}", l);

    let mut mx = 0.0;
    let mut my = 0.0;

    let mut left_down = false;
    let mut toggle = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .get_keys_pressed(KeyRepeat::No)
            .iter()
            .for_each(|key| player.handle_text_input(*key));

        let mouse_down = window.get_mouse_down(MouseButton::Left);

        if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
            player.player.global_inputs_load(BINDING_FILE_NAME);
        }
        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            player.player.global_inputs_set_color(
                "ball_start",
                &vec![
                    rand::random::<f32>() * 1.0,
                    rand::random::<f32>() * 1.0,
                    rand::random::<f32>() * 1.0,
                    1.0,
                ],
            );
            // player
            //     .player
            //     .global_inputs_set_color("ball_start", &[0.0, 0.0, 0.0]);
            player
                .player
                .global_inputs_set_color("ball_end", &[1.0, 0.5, 0.3, 1.0]);

            // let mut gradient_storage = vec![];
            // gradient_storage.push(GradientStop {
            //     color: [0.0, 1.0, 1.0, 1.0],
            //     offset: 0.0,
            // });
            // gradient_storage.push(GradientStop {
            //     color: [1.0, 0.0, 1.0, 1.0],
            //     offset: 1.0,
            // });
            // gradient_storage.push(GradientStop {
            //     color: [1.0, 0.0, 1.0, 1.0],
            //     offset: 1.0,
            // });
            // player
            //     .player
            //     .global_inputs_set_gradient("ball", &gradient_storage);

            // Scalar
            player.player.global_inputs_set_numeric("ball", 90.0);

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

        if window.is_key_pressed(Key::Key0, KeyRepeat::No) {
            player
                .player
                .global_inputs_set_color("start_0", &vec![1.0, 0.0, 1.0, 1.0]);
            player
                .player
                .global_inputs_set_color("end_0", &[0.0, 1.0, 0.0, 0.2]);
        }
        if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            player
                .player
                .global_inputs_set_color("start_1", &[1.0, 1.0, 0.0, 1.0]);
            player
                .player
                .global_inputs_set_color("end_1", &[0.0, 0.0, 1.0, 0.5]);
        }
        if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            player
                .player
                .global_inputs_set_color("start_2", &[0.0, 1.0, 1.0, 0.8]);
            player
                .player
                .global_inputs_set_color("end_2", &[1.0, 0.0, 0.0, 1.0]);
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
            // let mut gradient_storage = vec![];
            // gradient_storage.push(GradientStop {
            //     color: [0.0, 1.0, 1.0, 1.0],
            //     offset: 0.0,
            // });
            // gradient_storage.push(GradientStop {
            //     color: [1.0, 0.0, 1.0, 1.0],
            //     offset: ((mx / (WIDTH as f32) * 100.0) / 100.0),
            // });
            // gradient_storage.push(GradientStop {
            //     color: [1.0, 0.0, 1.0, 1.0],
            //     offset: 1.0,
            // });
            // player
            //     .player
            //     .global_inputs_set_gradient("ball", &gradient_storage);
            // println!("setting wand_pos");
            // player
            //     .player
            //     .global_inputs_set_vector("wand_pos", &[mx.into(), my.into()]);
        }

        player.update();
        window
            .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window");
        // }
    }
}
