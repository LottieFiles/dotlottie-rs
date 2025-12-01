use dotlottie_rs::{Config, DotLottiePlayer, GradientStop};
use minifb::{Key, Window, WindowOptions};
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

pub const ANIMATION_NAME: &str = "test_inputs_ball_gradient";
pub const BINDING_FILE_NAME: &str = "inputs";
pub const THEMING_FILE_NAME: &str = "theme";

fn main() {
    let mut window = Window::new(
        "Lottie Player Demo (ESC to exit, ←/→ to change markers, P to play, S to stop)",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    let mut player = Player::new(&format!("./src/bin/testbed/{}.lottie", ANIMATION_NAME));

    let load = player.player.global_inputs_load(BINDING_FILE_NAME);

    let st = player.player.set_theme(THEMING_FILE_NAME);
    println!("Inputs load: {}", load);
    println!("Set theme: {}", st);

    let mut mx = 0.0;
    let mut my = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
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
                color: [0.0, 1.0, 1.0, 1.0],
                offset: 0.0,
            });
            gradient_storage.push(GradientStop {
                color: [1.0, 0.0, 1.0, 1.0],
                offset: ((mx / (WIDTH as f32) * 100.0) / 100.0),
            });
            gradient_storage.push(GradientStop {
                color: [1.0, 0.0, 1.0, 1.0],
                offset: 1.0,
            });
            player
                .player
                .global_inputs_set_gradient("ball", &gradient_storage);
        }

        player.update();
        window
            .update_with_buffer(player.frame_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}
