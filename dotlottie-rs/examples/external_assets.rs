#![allow(clippy::print_stdout)]

/// Demonstrates `Player::set_asset_resolver`: the animation references two
/// non-embedded images (a remote URL and a bare path) whose bytes are
/// supplied by the resolver.
use dotlottie_rs::{ColorSpace, Player};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;

mod common;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

static RED_PNG: &[u8] = include_bytes!("../assets/images/red.png");
static BLUE_PNG: &[u8] = include_bytes!("../assets/images/blue.png");
const ANIMATION: &str = include_str!("../assets/animations/lottie/external_assets.json");

fn main() {
    let mut window = Window::new(
        "External Asset Resolver Example - ESC to quit",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut player = Player::new();
    player.set_autoplay(true);
    player.set_loop(true);

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ARGB8888)
        .unwrap();

    player.set_asset_resolver(|src| {
        println!("asset resolver asked for: {src}");
        match src {
            "https://example.com/red.png" => Some(RED_PNG.to_vec()),
            s if s.ends_with("textures/blue.png") => Some(BLUE_PNG.to_vec()),
            _ => None,
        }
    });

    let c_data = CString::new(ANIMATION).expect("CString conversion failed");
    if player.load_animation_data(&c_data).is_err() {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Both squares are resolver-supplied: red via URL, blue via path.");
    println!("Press ESC to quit");

    let mut clock = common::Clock::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = clock.dt();
        if player.tick(dt).unwrap_or(false) {
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }
}
