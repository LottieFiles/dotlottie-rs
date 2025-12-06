use dotlottie_rs::{actions::open_url_policy::OpenUrlPolicy, Config, DotLottiePlayer, Event};
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::time::Instant;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

struct Player {
    player: DotLottiePlayer,
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

        Self {
            player,
            last_update: Instant::now(),
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

pub const ANIMATION_NAME: &str = "test_inputs_ball_numeric_animated";
pub const BINDING_FILE_NAME: &str = "inputs_animated";

fn main() {
    let mut window = Window::new(
        "Lottie Player Demo (ESC to exit, ←/→ to change markers, P to play, S to stop)",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    let mut player = Player::new(&format!("./src/bin/numeric/{}.lottie", ANIMATION_NAME));

    let binding_file_path = format!("./src/bin/numeric/{}.json", BINDING_FILE_NAME);
    let binding_file_data = std::fs::read_to_string(&binding_file_path).expect(&format!(
        "[Error] Failed to read binding file: {}",
        binding_file_path
    ));

    let binding_load = player.player.global_inputs_load_data(&binding_file_data);

    let theme_load = player.player.set_theme("theme");

    println!("[Debug] Parse succeeded: {}", binding_load);
    println!("[Debug] Load succeeded: {}", theme_load);

    window.set_title("[Bindings - Numeric] Press Space to switch between Animated and Static");

    let mut using_animated = true;
    let mut space_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        player.update();

        let space_down = window.is_key_down(Key::Space);

        if space_down && !space_was_down {
            // This block will only run on the *transition* from not pressed to pressed.

            // Extract the static names by replacing _animated with _static
            let mut static_animation_name = ANIMATION_NAME.replace("_animated", "_static");
            let mut static_binding_file_name = BINDING_FILE_NAME.replace("animated", "static");

            if using_animated {
                static_animation_name = ANIMATION_NAME.replace("_animated", "_static");
                static_binding_file_name = BINDING_FILE_NAME.replace("animated", "static");
            } else {
                static_animation_name = ANIMATION_NAME.replace("_static", "_animated");
                static_binding_file_name = BINDING_FILE_NAME.replace("static", "animated");
            }
            using_animated = !using_animated;

            // Load the animation
            player = Player::new(&format!(
                "./src/bin/numeric/{}.lottie",
                static_animation_name
            ));

            let static_binding_file_path =
                format!("./src/bin/numeric/{}.json", static_binding_file_name);
            let static_binding_file_data = std::fs::read_to_string(&static_binding_file_path)
                .expect(&format!(
                    "[Error] Failed to read binding file: {}",
                    static_binding_file_path
                ));

            let static_binding_load = player
                .player
                .global_inputs_load_data(&static_binding_file_data);
            let static_theme_load = player.player.set_theme("theme");
            println!("[Debug] (Switch) Parse succeeded: {}", static_binding_load);
            println!("[Debug] (Switch) Load succeeded: {}", static_theme_load);
        }
        space_was_down = space_down;

        window
            .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}
