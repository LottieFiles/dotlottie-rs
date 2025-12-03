use dotlottie_rs::{actions::open_url_policy::OpenUrlPolicy, Config, DotLottiePlayer, Event};
use minifb::{Key, KeyRepeat, MouseButton, Window, WindowOptions};
use std::time::Instant;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;
const EASE_LINEAR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

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

pub const ANIMATION_NAME: &str = "magic_wand";
pub const BINDING_FILE_NAME: &str = "magic_wand_binding";

fn main() {
    let mut window = Window::new(
        "Lottie Player Demo (ESC to exit, ←/→ to change markers, P to play, S to stop)",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    let mut player = Player::new(&format!("./src/bin/magic-wand/{}.lottie", ANIMATION_NAME));

    let binding_file_path = format!("./src/bin/magic-wand/{}.json", BINDING_FILE_NAME);
    let binding_file_data = std::fs::read_to_string(&binding_file_path).expect(&format!(
        "Failed to read binding file: {}",
        binding_file_path
    ));

    let parse = player.player.global_inputs_load_data(&binding_file_data);
    println!("Parse succeeded: {}", parse);

    let loaded = player.player.state_machine_load("wand_sm");
    println!("[SM] Loaded? {}", loaded);

    let started = player.player.state_machine_start(OpenUrlPolicy::default());
    println!("[SM] Started? {}", started);

    let st = player.player.set_theme("wand");
    println!("Set theme: {}", st);

    let mut mx = 0.0;
    let mut my = 0.0;
    let mut left_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mouse_down = window.get_mouse_down(MouseButton::Left);
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
            player
                .player
                .global_inputs_set_vector("wand_pos", &[mx.into(), my.into()]);
        });
        player.update();
        window
            .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}
