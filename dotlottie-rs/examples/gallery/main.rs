//! Interactive gallery of the DragAndDrop / path-drag demos — one window,
//! one command:
//!
//!   cargo run --example gallery --features dev
//!
//! Left panel: the demo list with short descriptions. Main panel: a live
//! player running the selected demo. Every demo is a declarative pair of
//! (state machine JSON, animation JSON); the gallery's only host logic is
//! forwarding pointer events (plus feeding `cursor_x`/`cursor_y` for the
//! slide-unlock machine, which consumes them).

#![allow(clippy::print_stdout)]

use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
use dotlottie_rs::events::Event;
use dotlottie_rs::{ColorSpace, Player, StateMachineEngine};
use eframe::egui;
use std::ffi::CString;
use std::fs;

const W: usize = 512;
const H: usize = 512;
const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

struct Demo {
    name: &'static str,
    blurb: &'static str,
    sm: &'static str,
    anim: &'static str,
    /// Feed pointer position into cursor_x / cursor_y inputs on move
    /// (the slide-unlock machine consumes cursor_x).
    feed_cursor: bool,
}

const fn demo(name: &'static str, blurb: &'static str, sm: &'static str, anim: &'static str) -> Demo {
    Demo {
        name,
        blurb,
        sm,
        anim,
        feed_cursor: false,
    }
}

fn demos() -> Vec<Demo> {
    vec![
        demo(
            "DnD: star drop",
            "The whole gesture in ONE interaction: shape-accurate pickup, grab offset, snap tween, lock. Pointer events only.",
            "star_drop_dnd",
            "star_drop_static",
        ),
        demo(
            "DnD: tracking dock",
            "track: true \u{2014} after docking, the engine follows the moving zone each tick. Re-grab to un-dock.",
            "star_drop_track",
            "star_drop_moving",
        ),
        demo(
            "DnD: ghost drag",
            "ghost: true \u{2014} a frozen duplicate rides the pointer above everything while the original stays parked; on release the ghost glides to the dock and the slot is written once.",
            "star_drop_ghost",
            "star_drop_static",
        ),
        Demo {
            feed_cursor: true,
            ..demo(
                "Slide to unlock (DnD)",
                "onGrab/onDrop fire events; a state-slot binding wins the write order to pin the thumb to its track.",
                "slide_unlock_dnd",
                "slide_unlock",
            )
        },
        demo(
            "Boundary drag",
            "Free drag clamped inside the rectangle's rendered bounds (its 260% layer scale is honored). Slides along edges.",
            "boundary_drag",
            "boundary_drag",
        ),
        demo(
            "Spiral drag",
            "Branch-local projection: drag the knob around the spiral \u{2014} adjacent turns can't steal it.",
            "spiral_drag",
            "spiral_scrub",
        ),
        demo(
            "Spiral unlock",
            "Drag scrubs the MAIN timeline via segment-relative SetProgress; release early glides home, complete it to unlock.",
            "spiral_unlock",
            "spiral_unlock",
        ),
        demo(
            "Spiral unlock + dock points",
            "Dots on the path are dock points (arc-proximity capture, ratchet fallback, along-path glides). Three states, zero coordinates.",
            "spiral_unlock_zones",
            "spiral_unlock",
        ),
    ]
}

/// A running demo. The engine borrows the player for its whole life, so
/// the player is leaked to 'static on creation and reclaimed on drop,
/// after the engine is gone. The pixel buffer's heap allocation is stable
/// for as long as this struct lives.
struct DemoInstance {
    engine: Option<StateMachineEngine<'static>>,
    player: *mut Player,
    buffer: Vec<u32>,
}

impl DemoInstance {
    fn new(d: &Demo) -> Result<Self, String> {
        let mut buffer = vec![0u32; W * H];

        let player: &'static mut Player = Box::leak(Box::new(Player::new()));
        let player_ptr = player as *mut Player;
        let cleanup = |e: String| {
            unsafe { drop(Box::from_raw(player_ptr)) };
            e
        };

        player
            .set_sw_target(&mut buffer, W as u32, H as u32, ColorSpace::ABGR8888)
            .map_err(|e| cleanup(format!("set_sw_target: {e:?}")))?;

        let anim = fs::read_to_string(format!("{ASSETS_DIR}/animations/lottie/{}.json", d.anim))
            .map_err(|e| cleanup(format!("read animation '{}': {e}", d.anim)))?;
        let c_anim = CString::new(anim).map_err(|e| cleanup(format!("{e}")))?;
        player
            .load_animation_data(&c_anim)
            .map_err(|e| cleanup(format!("load animation: {e:?}")))?;

        let sm = fs::read_to_string(format!("{ASSETS_DIR}/statemachines/{}.json", d.sm))
            .map_err(|e| cleanup(format!("read state machine '{}': {e}", d.sm)))?;
        let mut engine = player
            .state_machine_load_data(&sm)
            .map_err(|e| cleanup(format!("load state machine: {e:?}")))?;
        engine
            .start(&OpenUrlPolicy::default())
            .map_err(|e| cleanup(format!("start: {e:?}")))?;

        Ok(Self {
            engine: Some(engine),
            player: player_ptr,
            buffer,
        })
    }

    fn engine(&mut self) -> &mut StateMachineEngine<'static> {
        self.engine.as_mut().expect("engine present until drop")
    }
}

impl Drop for DemoInstance {
    fn drop(&mut self) {
        if let Some(engine) = self.engine.take() {
            engine.release();
        }
        // The engine (and its borrow of the player) is gone: reclaim.
        unsafe { drop(Box::from_raw(self.player)) };
    }
}

struct App {
    demos: Vec<Demo>,
    selected: usize,
    instance: Option<DemoInstance>,
    error: Option<String>,
    texture: Option<egui::TextureHandle>,
    left_down: bool,
    last_pos: Option<(f32, f32)>,
}

impl App {
    fn new() -> Self {
        Self {
            demos: demos(),
            selected: 0,
            instance: None,
            error: None,
            texture: None,
            left_down: false,
            last_pos: None,
        }
    }

    fn select(&mut self, index: usize) {
        if index != self.selected || (self.instance.is_none() && self.error.is_none()) {
            self.selected = index;
            self.instance = None;
            self.error = None;
            self.left_down = false;
            self.last_pos = None;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Lazily (re)build the selected demo.
        if self.instance.is_none() && self.error.is_none() {
            match DemoInstance::new(&self.demos[self.selected]) {
                Ok(instance) => self.instance = Some(instance),
                Err(e) => self.error = Some(e),
            }
        }

        let mut clicked_demo = None;
        egui::SidePanel::left("demo_list")
            .default_width(240.0)
            .show(ctx, |ui| {
                ui.heading("State machine demos");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, d) in self.demos.iter().enumerate() {
                        if ui.selectable_label(i == self.selected, d.name).clicked() {
                            clicked_demo = Some(i);
                        }
                        ui.label(egui::RichText::new(d.blurb).small().weak());
                        ui.add_space(6.0);
                    }
                });
            });
        if let Some(i) = clicked_demo {
            self.select(i);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let d = &self.demos[self.selected];
            ui.heading(d.name);
            ui.label(
                egui::RichText::new(format!(
                    "statemachines/{}.json  +  animations/lottie/{}.json",
                    d.sm, d.anim
                ))
                .small()
                .weak(),
            );
            ui.add_space(4.0);

            if let Some(e) = &self.error {
                ui.colored_label(egui::Color32::LIGHT_RED, e);
                return;
            }
            let feed_cursor = d.feed_cursor;
            let Some(instance) = self.instance.as_mut() else {
                return;
            };

            // Drain engine events (transitions etc.) to the terminal.
            while let Some(event) = instance.engine().poll_event() {
                println!("{event:?}");
            }

            // ── Present the frame, capture pointer interaction ───────────
            let dt_ms = ctx.input(|i| i.stable_dt).min(0.1) * 1000.0;

            let image = {
                let bytes: &[u8] = unsafe {
                    std::slice::from_raw_parts(
                        instance.buffer.as_ptr() as *const u8,
                        instance.buffer.len() * 4,
                    )
                };
                egui::ColorImage::from_rgba_unmultiplied([W, H], bytes)
            };
            match &mut self.texture {
                Some(t) => t.set(image, egui::TextureOptions::LINEAR),
                None => {
                    self.texture =
                        Some(ctx.load_texture("preview", image, egui::TextureOptions::LINEAR))
                }
            }
            let texture = self.texture.as_ref().expect("texture just set");

            let response = ui.add(
                egui::Image::new((texture.id(), egui::vec2(W as f32, H as f32)))
                    .sense(egui::Sense::click_and_drag()),
            );
            // Outline the canvas so the pointer-active area is visible
            // (many demos render on a transparent background).
            ui.painter().rect_stroke(
                response.rect.expand(1.0),
                2.0,
                egui::Stroke::new(1.0, ui.visuals().strong_text_color()),
                egui::StrokeKind::Outside,
            );

            // Pointer -> engine, exactly like the standalone examples:
            // moves whenever the position changes, down/up edges of the
            // primary button, optional cursor-input feeding.
            let rect = response.rect;
            let pointer = response
                .hover_pos()
                .or_else(|| response.interact_pointer_pos())
                .map(|p| (p.x - rect.min.x, p.y - rect.min.y))
                .filter(|(x, y)| {
                    *x >= 0.0 && *y >= 0.0 && *x < W as f32 && *y < H as f32
                });

            if let Some((mx, my)) = pointer {
                if self.last_pos != Some((mx, my)) {
                    instance
                        .engine()
                        .post_event(&Event::PointerMove { x: mx, y: my });
                    self.last_pos = Some((mx, my));
                }
                if feed_cursor {
                    let _ = instance.engine().set_numeric_input("cursor_x", mx, true, false);
                    let _ = instance.engine().set_numeric_input("cursor_y", my, false, false);
                }
                let down = ctx.input(|i| i.pointer.primary_down());
                if down && !self.left_down {
                    instance
                        .engine()
                        .post_event(&Event::PointerDown { x: mx, y: my });
                } else if !down && self.left_down {
                    instance
                        .engine()
                        .post_event(&Event::PointerUp { x: mx, y: my });
                }
                self.left_down = down;
            } else if self.left_down {
                // Released or left the canvas mid-drag.
                if let Some((mx, my)) = self.last_pos {
                    instance
                        .engine()
                        .post_event(&Event::PointerUp { x: mx, y: my });
                }
                self.left_down = false;
            }

            let _ = instance.engine().tick(dt_ms);
        });

        // Keep animating.
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([820.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native(
        "dotLottie state machine gallery",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}
