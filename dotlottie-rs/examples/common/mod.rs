#![allow(dead_code)]

use dotlottie_rs::{ColorSpace, Player};
use eframe::egui;
use std::ffi::CString;
use std::fs;
use std::path::Path;

pub struct Clock {
    last: std::time::Instant,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            last: std::time::Instant::now(),
        }
    }

    pub fn dt(&mut self) -> f32 {
        let now = std::time::Instant::now();
        let dt = (now - self.last).as_secs_f32() * 1000.0;
        self.last = now;
        dt
    }
}

pub fn load_file(player: &mut Player, path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("lottie") => {
            fs::read(path).is_ok_and(|data| player.load_dotlottie_data(&data).is_ok())
        }
        Some("json") => fs::read_to_string(path)
            .ok()
            .and_then(|json| CString::new(json).ok())
            .is_some_and(|json| player.load_animation_data(&json).is_ok()),
        _ => false,
    }
}

pub fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

pub const CANVAS_SIZE: u32 = 512;

pub struct Canvas {
    pub player: Box<Player>,
    buffer: Vec<u32>,
    texture: Option<egui::TextureHandle>,
    clock: Clock,
}

impl Canvas {
    pub fn new() -> Self {
        let mut buffer = vec![0u32; (CANVAS_SIZE * CANVAS_SIZE) as usize];
        let mut player = Box::new(Player::new());

        // ABGR8888 is R, G, B, A in memory: the byte order of an egui texture.
        player
            .set_sw_target(
                &mut buffer,
                CANVAS_SIZE,
                CANVAS_SIZE,
                CANVAS_SIZE,
                ColorSpace::ABGR8888,
            )
            .expect("Failed to set software rendering target");

        Self {
            player,
            buffer,
            texture: None,
            clock: Clock::new(),
        }
    }

    pub fn dt(&mut self) -> f32 {
        self.clock.dt()
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let pixels = self
            .buffer
            .iter()
            .map(|pixel| {
                let [r, g, b, a] = pixel.to_le_bytes();
                egui::Color32::from_rgba_premultiplied(r, g, b, a)
            })
            .collect();
        let image = egui::ColorImage::new([CANVAS_SIZE as usize; 2], pixels);

        let options = egui::TextureOptions::LINEAR;
        let texture = match &mut self.texture {
            Some(texture) => {
                texture.set(image, options);
                texture
            }
            None => self
                .texture
                .insert(ui.ctx().load_texture("canvas", image, options)),
        };

        let side = ui.available_size().min_elem();
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(side), egui::Sense::click_and_drag());

        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        ui.painter()
            .image(texture.id(), rect, uv, egui::Color32::WHITE);
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::GRAY),
            egui::StrokeKind::Outside,
        );

        ui.ctx().request_repaint();

        response
    }

    pub fn to_canvas(pos: egui::Pos2, rect: egui::Rect) -> (f32, f32) {
        let scale = CANVAS_SIZE as f32 / rect.width();
        ((pos.x - rect.min.x) * scale, (pos.y - rect.min.y) * scale)
    }
}

pub fn run(title: &str, app: impl eframe::App + 'static) {
    eframe::run_native(
        title,
        eframe::NativeOptions::default(),
        Box::new(move |_| Ok(Box::new(app))),
    )
    .expect("Failed to run eframe");
}
