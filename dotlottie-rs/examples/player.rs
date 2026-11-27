#![allow(clippy::print_stdout)]
//! Player: load `.lottie` archives or Lottie JSON files and drive playback,
//! themes and background. Player events are printed to stdout.
//!
//! ```text
//! cargo run --example player                          # bundled animations
//! cargo run --example player -- path/to/animation.lottie
//! cargo run --example player -- path/to/directory
//! ```
//!
//! Audio layers play, with a volume slider, when built with the `audio` feature:
//!
//! ```text
//! cargo run --example player --features audio -- \
//!     assets/animations/dotlottie/v2/happy_birthday_audio.lottie
//! ```

use dotlottie_rs::{PlayerEvent, Rgba, Status};
use eframe::egui;
use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};

mod common;

fn animation_files(path: &Path) -> Vec<PathBuf> {
    if !path.is_dir() {
        return vec![path.to_path_buf()];
    }

    let mut files: Vec<PathBuf> = fs::read_dir(path)
        .expect("Could not read animations directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext == "lottie" || ext == "json")
        })
        .collect();

    files.sort();
    files
}

struct App {
    canvas: common::Canvas,
    files: Vec<PathBuf>,
    file_index: usize,
    themes: Vec<String>,
    theme: Option<usize>,
    background: [u8; 4],
}

impl App {
    fn new(files: Vec<PathBuf>) -> Self {
        let mut canvas = common::Canvas::new();
        canvas.player.set_autoplay(true);
        canvas.player.set_loop(true);

        let mut app = Self {
            canvas,
            files,
            file_index: 0,
            themes: Vec::new(),
            theme: None,
            background: [0; 4],
        };
        app.open(0);
        app
    }

    fn open(&mut self, index: usize) {
        self.file_index = index;
        self.theme = None;
        self.themes.clear();

        let path = &self.files[index];
        let player = &mut self.canvas.player;

        if !common::load_file(player, path) {
            eprintln!("Failed to load {}", path.display());
            return;
        }

        println!(
            "Loaded {}: {:.0} frames, {:.2}s",
            common::file_name(path),
            player.total_frames(),
            player.duration() / 1000.0,
        );

        if let Some(themes) = player.manifest().and_then(|m| m.themes.as_ref()) {
            self.themes = themes.iter().map(|theme| theme.id.clone()).collect();
        }
    }

    fn set_theme(&mut self, theme: Option<usize>) {
        let player = &mut self.canvas.player;
        let applied = match theme {
            Some(index) => {
                let id = CString::new(self.themes[index].as_str()).expect("theme id has a NUL");
                player.set_theme(&id)
            }
            None => player.reset_theme(),
        };

        match applied {
            Ok(()) => self.theme = theme,
            Err(err) => eprintln!("Failed to set theme: {err}"),
        }
    }

    fn theme_name(&self, theme: Option<usize>) -> &str {
        theme.map_or("none", |index| self.themes[index].as_str())
    }

    fn print_events(&mut self) {
        while let Some(event) = self.canvas.player.poll_event() {
            match event {
                PlayerEvent::Load => println!("[event] load"),
                PlayerEvent::LoadError => println!("[event] load error"),
                PlayerEvent::Play => println!("[event] play"),
                PlayerEvent::Pause => println!("[event] pause"),
                PlayerEvent::Stop => println!("[event] stop"),
                PlayerEvent::Loop { loop_count } => println!("[event] loop #{loop_count}"),
                PlayerEvent::Complete => println!("[event] complete"),
                PlayerEvent::Frame { .. } | PlayerEvent::Render { .. } => {}
            }
        }
    }

    fn file_controls(&mut self, ui: &mut egui::Ui) {
        let count = self.files.len();

        ui.horizontal(|ui| {
            if ui.add_enabled(count > 1, egui::Button::new("<")).clicked() {
                self.open((self.file_index + count - 1) % count);
            }
            if ui.add_enabled(count > 1, egui::Button::new(">")).clicked() {
                self.open((self.file_index + 1) % count);
            }
            ui.label(common::file_name(&self.files[self.file_index]));
        });
    }

    fn playback_controls(&mut self, ui: &mut egui::Ui) {
        let player = &mut self.canvas.player;

        ui.horizontal(|ui| {
            let playing = player.status() == Status::Playing;
            if ui.button(if playing { "Pause" } else { "Play" }).clicked() {
                let _ = if playing {
                    player.pause()
                } else {
                    player.play()
                };
            }
            if ui.button("Stop").clicked() {
                let _ = player.stop();
            }

            let last_frame = (player.total_frames() - 1.0).max(0.0);
            let mut frame = player.current_frame();
            let slider = egui::Slider::new(&mut frame, 0.0..=last_frame).text("frame");
            if ui.add(slider).changed() {
                let _ = player.set_frame(frame);
                let _ = player.render();
            }

            ui.label(format!("{:?}", player.status()));
        });
    }

    fn appearance_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Background");
            if ui
                .color_edit_button_srgba_unmultiplied(&mut self.background)
                .changed()
            {
                let [r, g, b, a] = self.background;
                let _ = self.canvas.player.set_background(Rgba::new(r, g, b, a));
            }

            if !self.themes.is_empty() {
                let mut selected = self.theme;
                egui::ComboBox::from_label("theme")
                    .selected_text(self.theme_name(self.theme))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, None, "none");
                        for index in 0..self.themes.len() {
                            ui.selectable_value(
                                &mut selected,
                                Some(index),
                                self.theme_name(Some(index)),
                            );
                        }
                    });
                if selected != self.theme {
                    self.set_theme(selected);
                }
            }

            #[cfg(feature = "audio")]
            volume_slider(ui, &mut self.canvas.player);
        });
    }
}

#[cfg(feature = "audio")]
fn volume_slider(ui: &mut egui::Ui, player: &mut dotlottie_rs::Player) {
    let mut volume = player.audio_volume();
    let slider = egui::Slider::new(&mut volume, 0.0..=1.0).text("volume");
    if ui.add(slider).changed() {
        player.set_audio_volume(volume);
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let dt = self.canvas.dt();
        let _ = self.canvas.player.tick(dt);
        self.print_events();

        egui::Panel::top("controls").show(ui, |ui| {
            self.file_controls(ui);
            self.playback_controls(ui);
            self.appearance_controls(ui);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            self.canvas.show(ui);
        });
    }
}

fn main() {
    let path = std::env::args().nth(1).map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/animations/dotlottie/v2"),
        PathBuf::from,
    );

    let files = animation_files(&path);
    if files.is_empty() {
        eprintln!("No .lottie or .json files found in {}", path.display());
        std::process::exit(1);
    }

    common::run("dotLottie player", App::new(files));
}
