use dotlottie_rs::{Config, DotLottiePlayer};
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
}

pub const ANIMATION_NAME: &str = "test_inputs_bull_image";
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

    let mut player = Player::new(&format!(
        "./src/bin/image-replacement/{}.lottie",
        ANIMATION_NAME
    ));

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
        player.update();
        window
            .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}
