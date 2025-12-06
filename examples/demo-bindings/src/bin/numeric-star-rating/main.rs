use dotlottie_rs::{
    actions::open_url_policy::OpenUrlPolicy, Config, DotLottiePlayer, GlobalInputsObserver,
};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::sync::Arc;
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

pub const ANIMATION_NAME: &str = "sm_star_rating";
pub const BINDING_FILE_NAME: &str = "inputs";
pub const SM_FILE_NAME: &str = "starRating";

struct DummyGlobalInputsObserver;

impl GlobalInputsObserver for DummyGlobalInputsObserver {
    fn on_color_global_input_value_change(
        &self,
        global_input_name: String,
        old_value: Vec<f32>,
        new_value: Vec<f32>,
    ) {
        println!(
            "[global input event] color_input_value_change ==> {} : {:?} -> {:?}",
            global_input_name, old_value, new_value
        );
    }

    fn on_gradient_global_input_value_change(
        &self,
        global_input_name: String,
        old_value: Vec<f32>,
        new_value: Vec<f32>,
    ) {
        println!(
            "[global input event] gradient_input_value_change ==> {} : {:?} -> {:?}",
            global_input_name, old_value, new_value
        );
    }

    fn on_numeric_global_input_value_change(
        &self,
        global_input_name: String,
        old_value: f32,
        new_value: f32,
    ) {
        println!(
            "[global input event] numeric_input_value_change ==> {} : {} -> {}",
            global_input_name, old_value, new_value
        );
    }

    fn on_boolean_global_input_value_change(
        &self,
        global_input_name: String,
        old_value: bool,
        new_value: bool,
    ) {
        println!(
            "[global input event] boolean_input_value_change ==> {} : {} -> {}",
            global_input_name, old_value, new_value
        );
    }

    fn on_string_global_input_value_change(
        &self,
        global_input_name: String,
        old_value: String,
        new_value: String,
    ) {
        println!(
            "[global input event] string_input_value_change ==> {} : {} -> {}",
            global_input_name, old_value, new_value
        );
    }

    fn on_vector_global_input_value_change(
        &self,
        global_input_name: String,
        old_value: [f32; 2],
        new_value: [f32; 2],
    ) {
        println!(
            "[global input event] vector_input_value_change ==> {} : [{}, {}] -> [{}, {}]",
            global_input_name, old_value[0], old_value[1], new_value[0], new_value[1]
        );
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

    let mut player = Player::new(&format!(
        "./src/bin/numeric-star-rating/{}.lottie",
        ANIMATION_NAME
    ));

    player.player.global_inputs_load(BINDING_FILE_NAME);

    let observer: Arc<dyn GlobalInputsObserver + 'static> = Arc::new(DummyGlobalInputsObserver {});
    player.player.global_inputs_subscribe(observer);

    let sml = player.player.state_machine_load(SM_FILE_NAME);
    let sms = player.player.state_machine_start(OpenUrlPolicy::default());

    println!("State machine loaded: {}", sml);
    println!("State machine started: {}", sms);

    player.player.global_inputs_load(BINDING_FILE_NAME);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            player.player.global_inputs_set_numeric("curr_star", 1.0);
        }
        if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            player.player.global_inputs_set_numeric("curr_star", 2.0);
        }
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            player.player.global_inputs_set_numeric("curr_star", 3.0);
        }
        if window.is_key_pressed(Key::Key4, KeyRepeat::No) {
            player.player.global_inputs_set_numeric("curr_star", 4.0);
        }
        if window.is_key_pressed(Key::Key5, KeyRepeat::No) {
            player.player.global_inputs_set_numeric("curr_star", 5.0);
        }

        player.update();
        window
            .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}
