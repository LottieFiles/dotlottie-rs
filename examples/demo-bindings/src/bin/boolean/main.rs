use dotlottie_rs::{
    actions::open_url_policy::OpenUrlPolicy, Config, DotLottiePlayer, GlobalInputsObserver,
};
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::cell::RefCell;
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
        let player = DotLottiePlayer::new(Config {
            autoplay: false,
            loop_animation: false,
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

pub const ANIMATION_NAME: &str = "sm_toggle_button";
pub const BINDING_FILE_NAME: &str = "inputs";
pub const SM_FILE_NAME: &str = "toggleButton";

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
        "[Bindings - Color] S=Switch Mode | R=Remove Theme | T=Apply theme",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    let mut using_animated = true;
    let mut player = Player::new(&format!("./src/bin/boolean/{}.lottie", ANIMATION_NAME));

    // Load binding and set theme on startup
    let binding_file_path = format!("./src/bin/boolean/{}.json", BINDING_FILE_NAME);
    let binding_file_data = std::fs::read_to_string(&binding_file_path).unwrap();
    let l = player.player.state_machine_load(SM_FILE_NAME);
    let s = player.player.state_machine_start(OpenUrlPolicy::default());
    player.player.global_inputs_load_data(&binding_file_data);
    player.player.set_theme("theme");

    let observer: Arc<dyn GlobalInputsObserver + 'static> = Arc::new(DummyGlobalInputsObserver {});
    player.player.global_inputs_subscribe(observer.clone());

    println!("state machine: {}{}", l, s);

    println!("[Info] Controls:");
    println!("  S - Switch between animated/static mode");
    println!("  R - Remove theme");
    println!("  T - Apply theme");
    println!("[Info] Current mode: ANIMATED");

    // Track whether the mouse was down in the previous frame to avoid sending repeated pointer down events
    thread_local! {
        static WAS_LEFT_DOWN: RefCell<bool> = RefCell::new(false);
    }

    let mut toggle = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        player.update();

        let left_down = window.get_mouse_down(MouseButton::Left);

        WAS_LEFT_DOWN.with(|was_down| {
            let mut was_down = was_down.borrow_mut();

            if left_down && !*was_down {
                toggle = !toggle;
                player
                    .player
                    .global_inputs_set_boolean("toggle_global_input", toggle);
            }
            //     player
            //         .player
            //         .state_machine_post_pointer_down_event(10.0, 10.0);
            // }
            *was_down = left_down;
        });

        // R: Remove theme
        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            player.player.reset_theme();
            player.player.global_inputs_remove();
            player.player.set_theme("theme");
            println!("[Debug] Theme removed");
        }

        // T: Apply theme
        if window.is_key_pressed(Key::T, minifb::KeyRepeat::Yes) {
            let binding_file_name = if using_animated {
                "inputs_animated"
            } else {
                "inputs_static"
            };

            player.player.set_theme("theme");

            // Load corresponding binding and apply theme
            let binding_file_path = format!("./src/bin/boolean/{}.json", binding_file_name);
            let binding_file_data = std::fs::read_to_string(&binding_file_path).unwrap();
            let load = player.player.global_inputs_load_data(&binding_file_data);
            println!("[Debug]: Loaded inputs: {}", load);
        }

        // S: Switch between animated and static mode
        if window.is_key_pressed(Key::S, minifb::KeyRepeat::No) {
            using_animated = !using_animated;

            let animation_name = if using_animated {
                ANIMATION_NAME.to_string()
            } else {
                ANIMATION_NAME.replace("_animated", "_static")
            };

            let binding_file_name = if using_animated {
                BINDING_FILE_NAME.to_string()
            } else {
                BINDING_FILE_NAME.replace("animated", "static")
            };

            // Load new animation
            player = Player::new(&format!("./src/bin/boolean/{}.lottie", animation_name));

            // Load corresponding binding and apply theme
            let binding_file_path = format!("./src/bin/boolean/{}.json", binding_file_name);
            let binding_file_data = std::fs::read_to_string(&binding_file_path).unwrap();
            player.player.global_inputs_load_data(&binding_file_data);
            player.player.set_theme("theme");

            // Re-subscribe observer after creating new player
            player.player.global_inputs_subscribe(observer.clone());

            let mode_str = if using_animated { "ANIMATED" } else { "STATIC" };
            println!("[Info] Switched to {} mode", mode_str);
        }

        window
            .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}
