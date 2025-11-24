use dotlottie_rs::{actions::open_url_policy::OpenUrlPolicy, Config, DotLottiePlayer};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::Instant;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

struct Player {
    player: DotLottiePlayer,
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

        Self {
            player,
            last_update: Instant::now(),
            text_input: String::new(),
        }
    }

    fn update(&mut self) -> bool {
        let updated = self.player.tick();
        self.last_update = Instant::now();
        updated
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
                        .global_inputs_set_text("text_input", &self.text_input);
                } else {
                    // Use a single space instead of empty string to avoid the crash
                    self.player.global_inputs_set_text("text_input", " ");
                }
                None
            }
            _ => None,
        };

        if let Some(c) = char_to_add {
            self.text_input.push(c);
            println!("Current text: '{}'", self.text_input);
            self.player
                .global_inputs_set_text("text_input", &self.text_input);
        }
    }
}

pub const ANIMATION_NAME: &str = "test_inputs_text";
pub const BINDING_FILE_NAME: &str = "inputs";
pub const THEMING_FILE_NAME: &str = "theme";
// pub const SM_FILE_NAME: &str = "starRating";

fn main() {
    let mut window = Window::new(
        "Lottie Player Demo (ESC to exit, Type letters to input text)",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    let mut player = Player::new(&format!("./src/bin/text-input/{}.lottie", ANIMATION_NAME));

    // let binding_file_path = format!("./src/bin/text-input/{}.json", BINDING_FILE_NAME);
    // let binding_file_data = std::fs::read_to_string(&binding_file_path).expect(&format!(
    //     "Failed to read binding file: {}",
    //     binding_file_path
    // ));

    let parse = player.player.global_inputs_load(BINDING_FILE_NAME);
    println!("[Player] Binding Parse succeeded: {}", parse);

    // let theme_file_path = format!("./src/bin/text-input/{}.json", THEMING_FILE_NAME);
    // let theme_file_data = std::fs::read_to_string(&theme_file_path)
    //     .expect(&format!("Failed to read theming file: {}", theme_file_path));

    // println!("THEME DATA: {}", theme_file_data);

    let parse = player.player.set_theme(THEMING_FILE_NAME);
    println!("[Player] Theme Parse succeeded: {}", parse);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .get_keys_pressed(KeyRepeat::No)
            .iter()
            .for_each(|key| player.handle_text_input(*key));

        player.update();
        window
            .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}
