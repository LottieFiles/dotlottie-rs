#![allow(clippy::print_stdout)]
//! External assets: images that are not embedded in the animation, supplied
//! through `Player::set_asset_resolver`.
//!
//! ```text
//! cargo run --example external_assets
//! ```

use eframe::egui;
use std::ffi::CString;

mod common;

static RED_PNG: &[u8] = include_bytes!("../assets/images/red.png");
static BLUE_PNG: &[u8] = include_bytes!("../assets/images/blue.png");
const ANIMATION: &str = include_str!("../assets/animations/lottie/external_assets.json");

struct App {
    canvas: common::Canvas,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let dt = self.canvas.dt();
        let _ = self.canvas.player.tick(dt);

        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("Both squares are resolver-supplied: red by URL, blue by path.");
            self.canvas.show(ui);
        });
    }
}

fn main() {
    let mut canvas = common::Canvas::new();
    let player = &mut canvas.player;
    player.set_autoplay(true);
    player.set_loop(true);

    player.set_asset_resolver(|src| {
        println!("asset resolver asked for: {src}");
        match src {
            "https://example.com/red.png" => Some(RED_PNG.to_vec()),
            s if s.ends_with("textures/blue.png") => Some(BLUE_PNG.to_vec()),
            _ => None,
        }
    });

    let animation = CString::new(ANIMATION).expect("animation JSON contains a NUL");
    player
        .load_animation_data(&animation)
        .expect("Failed to load animation");

    common::run("dotLottie external assets", App { canvas });
}
