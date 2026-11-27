#![allow(clippy::print_stdout)]
//! Slots: override the values an animation exposes by slot id, at runtime.
//!
//! ```text
//! cargo run --example slots
//! ```

use dotlottie_rs::player::Result;
use dotlottie_rs::{
    Bezier, BezierPath, BezierPathSlot, BezierValue, ColorSlot, ColorValue, GradientSlot,
    GradientStop, ImageSlot, LottieKeyframe, LottieProperty, Player, ScalarSlot, ScalarValue,
    TextDocument, TextKeyframe, TextSlot,
};
use eframe::egui;
use std::f32::consts::{FRAC_PI_2, TAU};
use std::ffi::CString;

mod common;

const BOUNCY_BALL: &str = include_str!("../assets/animations/lottie/bouncy_ball.json");
const GRADIENT: &str = include_str!("../assets/animations/lottie/gradient.json");
const TEXT: &str = include_str!("../assets/animations/lottie/text.json");
const BEZIER_PATH: &str = include_str!("../assets/animations/lottie/bezier_path.json");
const IMAGE: &str = include_str!("../assets/animations/lottie/image.json");

struct Demo {
    name: &'static str,
    animation: &'static str,
    slot_id: &'static str,
    values: usize,
    set_static: fn(&mut Player, &str, usize) -> Result<String>,
    set_animated: Option<fn(&mut Player, &str) -> Result<String>>,
}

const DEMOS: &[Demo] = &[
    Demo {
        name: "color",
        animation: BOUNCY_BALL,
        slot_id: "ball_color",
        values: COLORS.len(),
        set_static: color_static,
        set_animated: Some(color_animated),
    },
    Demo {
        name: "scalar",
        animation: BOUNCY_BALL,
        slot_id: "ball_opacity",
        values: OPACITIES.len(),
        set_static: scalar_static,
        set_animated: Some(scalar_animated),
    },
    Demo {
        name: "vector",
        animation: BOUNCY_BALL,
        slot_id: "ball_scale",
        values: SCALES.len(),
        set_static: vector_static,
        set_animated: Some(vector_animated),
    },
    Demo {
        name: "position",
        animation: BOUNCY_BALL,
        slot_id: "ball_position",
        values: POSITIONS.len(),
        set_static: position_static,
        set_animated: Some(position_animated),
    },
    Demo {
        name: "gradient",
        animation: GRADIENT,
        slot_id: "gradient_fill",
        values: GRADIENTS.len(),
        set_static: gradient_static,
        set_animated: Some(gradient_animated),
    },
    Demo {
        name: "text",
        animation: TEXT,
        slot_id: "my_text",
        values: TEXTS.len(),
        set_static: text_static,
        set_animated: Some(text_animated),
    },
    Demo {
        name: "bezier path",
        animation: BEZIER_PATH,
        slot_id: "shape_path",
        values: SHAPES.len(),
        set_static: bezier_path_static,
        set_animated: Some(bezier_path_animated),
    },
    Demo {
        name: "image",
        animation: IMAGE,
        slot_id: "sun_img",
        values: IMAGES.len(),
        set_static: image_static,
        set_animated: None,
    },
];

fn keyframe<T>(frame: f32, value: T) -> LottieKeyframe<T> {
    LottieKeyframe {
        frame,
        start_value: value,
        in_tangent: None,
        out_tangent: None,
        value_in_tangent: None,
        value_out_tangent: None,
        hold: None,
    }
}

const COLORS: &[(&str, [f32; 3])] = &[
    ("red", [1.0, 0.0, 0.0]),
    ("green", [0.0, 1.0, 0.0]),
    ("blue", [0.0, 0.0, 1.0]),
    ("orange", [1.0, 0.5, 0.0]),
    ("purple", [0.5, 0.0, 1.0]),
];

fn color_static(player: &mut Player, slot_id: &str, index: usize) -> Result<String> {
    let (name, rgb) = COLORS[index];
    player.set_color_slot(slot_id, ColorSlot::new(rgb))?;
    Ok(name.to_string())
}

fn color_animated(player: &mut Player, slot_id: &str) -> Result<String> {
    let slot = ColorSlot::with_keyframes(vec![
        keyframe(0.0, ColorValue([1.0, 0.0, 0.0])),
        keyframe(60.0, ColorValue([0.0, 0.0, 1.0])),
    ]);
    player.set_color_slot(slot_id, slot)?;
    Ok("red -> blue".to_string())
}

const OPACITIES: &[f32] = &[100.0, 60.0, 30.0, 10.0];

fn scalar_static(player: &mut Player, slot_id: &str, index: usize) -> Result<String> {
    let opacity = OPACITIES[index];
    player.set_scalar_slot(slot_id, ScalarSlot::new(opacity))?;
    Ok(format!("opacity {opacity:.0}%"))
}

fn scalar_animated(player: &mut Player, slot_id: &str) -> Result<String> {
    let slot = ScalarSlot::with_keyframes(vec![
        keyframe(0.0, ScalarValue(100.0)),
        keyframe(60.0, ScalarValue(20.0)),
    ]);
    player.set_scalar_slot(slot_id, slot)?;
    Ok("opacity 100% -> 20%".to_string())
}

const SCALES: &[[f32; 2]] = &[[100.0, 100.0], [150.0, 150.0], [50.0, 50.0], [150.0, 60.0]];

fn vector_static(player: &mut Player, slot_id: &str, index: usize) -> Result<String> {
    let [x, y] = SCALES[index];
    player.set_vector_slot(slot_id, LottieProperty::static_value([x, y]))?;
    Ok(format!("scale {x:.0}% x {y:.0}%"))
}

fn vector_animated(player: &mut Player, slot_id: &str) -> Result<String> {
    let slot = LottieProperty::animated(vec![
        keyframe(0.0, [50.0, 50.0]),
        keyframe(60.0, [150.0, 150.0]),
    ]);
    player.set_vector_slot(slot_id, slot)?;
    Ok("scale 50% -> 150%".to_string())
}

const POSITIONS: &[[f32; 2]] = &[
    [250.0, 250.0],
    [120.0, 120.0],
    [380.0, 120.0],
    [380.0, 380.0],
    [120.0, 380.0],
];

fn position_static(player: &mut Player, slot_id: &str, index: usize) -> Result<String> {
    let [x, y] = POSITIONS[index];
    player.set_position_slot(slot_id, LottieProperty::static_value([x, y]))?;
    Ok(format!("position {x:.0}, {y:.0}"))
}

fn position_animated(player: &mut Player, slot_id: &str) -> Result<String> {
    let slot = LottieProperty::animated(vec![
        keyframe(0.0, [100.0, 100.0]),
        keyframe(60.0, [400.0, 400.0]),
    ]);
    player.set_position_slot(slot_id, slot)?;
    Ok("position 100, 100 -> 400, 400".to_string())
}

const SUNSET: &[[f32; 3]] = &[[1.0, 0.4, 0.0], [1.0, 0.6, 0.2], [1.0, 0.8, 0.0]];
const OCEAN: &[[f32; 3]] = &[[0.0, 0.3, 0.6], [0.0, 0.5, 0.8], [0.0, 0.8, 1.0]];
const FOREST: &[[f32; 3]] = &[[0.1, 0.3, 0.1], [0.2, 0.6, 0.2], [0.5, 0.8, 0.3]];
const FIRE: &[[f32; 3]] = &[
    [1.0, 0.0, 0.0],
    [1.0, 0.5, 0.0],
    [1.0, 1.0, 0.0],
    [1.0, 1.0, 1.0],
];

const GRADIENTS: &[(&str, &[[f32; 3]])] = &[
    ("sunset", SUNSET),
    ("ocean", OCEAN),
    ("forest", FOREST),
    ("fire", FIRE),
];

fn stops(colors: &[[f32; 3]]) -> Vec<GradientStop> {
    let last = (colors.len() - 1) as f32;
    colors
        .iter()
        .enumerate()
        .map(|(i, &[r, g, b])| GradientStop {
            offset: i as f32 / last,
            color: [r, g, b, 1.0],
        })
        .collect()
}

fn gradient_static(player: &mut Player, slot_id: &str, index: usize) -> Result<String> {
    let (name, colors) = GRADIENTS[index];
    player.set_gradient_slot(slot_id, GradientSlot::new(stops(colors)))?;
    Ok(name.to_string())
}

fn gradient_animated(player: &mut Player, slot_id: &str) -> Result<String> {
    let slot = GradientSlot::with_keyframes(vec![
        keyframe(0.0, stops(SUNSET)),
        keyframe(60.0, stops(OCEAN)),
    ]);
    player.set_gradient_slot(slot_id, slot)?;
    Ok("sunset -> ocean".to_string())
}

const TEXTS: &[(&str, [f32; 4])] = &[
    ("Hello!", [1.0, 0.0, 0.0, 1.0]),
    ("World", [0.0, 0.6, 0.0, 1.0]),
    ("Slots!", [0.0, 0.0, 1.0, 1.0]),
    ("Dynamic", [1.0, 0.0, 1.0, 1.0]),
];

fn text_document(text: &str, fill: [f32; 4]) -> TextDocument {
    TextDocument::new(text)
        .with_font("Arial")
        .with_size(200.0)
        .with_fill_color(fill.to_vec())
}

fn text_static(player: &mut Player, slot_id: &str, index: usize) -> Result<String> {
    let (text, fill) = TEXTS[index];
    player.set_text_slot(slot_id, TextSlot::with_document(text_document(text, fill)))?;
    Ok(format!("\"{text}\""))
}

fn text_animated(player: &mut Player, slot_id: &str) -> Result<String> {
    let slot = TextSlot::with_keyframes(vec![
        TextKeyframe {
            frame: 0.0,
            text_document: text_document("Hello", [1.0, 0.0, 0.0, 1.0]),
        },
        TextKeyframe {
            frame: 30.0,
            text_document: text_document("World", [0.0, 0.0, 1.0, 1.0]),
        },
    ]);
    player.set_text_slot(slot_id, slot)?;
    Ok("\"Hello\" -> \"World\"".to_string())
}

/// Lottie interpolates paths point by point, so two keyframes only morph when
/// they have the same vertex count. 12 divides by 3, 4 and 6, so polygon corners
/// land on real vertices.
const POINTS: usize = 12;

type Shape = (&'static str, fn() -> BezierPath);

const SHAPES: &[Shape] = &[
    ("triangle", triangle),
    ("square", square),
    ("star", star),
    ("circle", circle),
];

/// Walk a polygon's outline and drop a vertex every `POINTS / corners` steps.
fn sampled_polygon(corners: &[[f32; 2]]) -> BezierPath {
    let per_side = POINTS / corners.len();
    let mut vertices = Vec::with_capacity(POINTS);

    for (i, from) in corners.iter().enumerate() {
        let to = corners[(i + 1) % corners.len()];
        for step in 0..per_side {
            let t = step as f32 / per_side as f32;
            vertices.push([
                from[0] + (to[0] - from[0]) * t,
                from[1] + (to[1] - from[1]) * t,
            ]);
        }
    }

    BezierPath::polygon(vertices, true)
}

fn triangle() -> BezierPath {
    sampled_polygon(&[[0.0, -80.0], [70.0, 45.0], [-70.0, 45.0]])
}

fn square() -> BezierPath {
    sampled_polygon(&[[-60.0, -60.0], [60.0, -60.0], [60.0, 60.0], [-60.0, 60.0]])
}

/// Six-pointed star: alternate the outer and inner radius around the circle.
fn star() -> BezierPath {
    let step = TAU / POINTS as f32;
    let vertices = (0..POINTS)
        .map(|i| {
            let radius = if i % 2 == 0 { 85.0 } else { 38.0 };
            let (sin, cos) = (step * i as f32 - FRAC_PI_2).sin_cos();
            [cos * radius, sin * radius]
        })
        .collect();

    BezierPath::polygon(vertices, true)
}

/// Points on a circle, with the cubic handles that round the segments between
/// them: k = 4/3 * tan(θ/4) * r.
fn circle() -> BezierPath {
    let radius = 70.0;
    let step = TAU / POINTS as f32;
    let handle = 4.0 / 3.0 * (step / 4.0).tan() * radius;

    let mut vertices = Vec::with_capacity(POINTS);
    let mut in_tangents = Vec::with_capacity(POINTS);
    let mut out_tangents = Vec::with_capacity(POINTS);

    for i in 0..POINTS {
        let (sin, cos) = (step * i as f32 - FRAC_PI_2).sin_cos();
        vertices.push([cos * radius, sin * radius]);
        in_tangents.push([sin * handle, -cos * handle]);
        out_tangents.push([-sin * handle, cos * handle]);
    }

    BezierPath {
        vertices,
        in_tangents,
        out_tangents,
        closed: true,
    }
}

fn ease(x: f32, y: f32) -> Option<Bezier> {
    Some(Bezier {
        x: BezierValue::Single(x),
        y: BezierValue::Single(y),
    })
}

fn bezier_path_static(player: &mut Player, slot_id: &str, index: usize) -> Result<String> {
    let (name, build) = SHAPES[index];
    player.set_bezier_path_slot(slot_id, BezierPathSlot::new(build()))?;
    Ok(name.to_string())
}

/// One eased keyframe per shape across the animation's 60 frames, ending back
/// on the triangle so the loop is seamless.
fn bezier_path_animated(player: &mut Player, slot_id: &str) -> Result<String> {
    let shapes = [triangle(), square(), star(), circle(), triangle()];
    let keyframes = shapes
        .into_iter()
        .enumerate()
        .map(|(i, path)| LottieKeyframe {
            in_tangent: ease(0.58, 1.0),
            out_tangent: ease(0.42, 0.0),
            ..keyframe(i as f32 * 15.0, path)
        })
        .collect();

    player.set_bezier_path_slot(slot_id, LottieProperty::animated(keyframes))?;
    Ok("triangle -> square -> star -> circle".to_string())
}

/// Tiny 24x24 solid-color PNGs, as data URIs.
const IMAGES: &[(&str, &str)] = &[
    ("magenta", "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAIAAABvFaqvAAAAH0lEQVR42mO4pjGNKohh1KBRg0YNGjVo1KBRgwbeIADJIY0uOOudVwAAAABJRU5ErkJggg=="),
    ("teal", "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAIAAABvFaqvAAAAHklEQVR42mNgWLGCOmjUoFGDRg0aNWjUoFGDBt4gABRe9B/QJEtQAAAAAElFTkSuQmCC"),
    ("amber", "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAIAAABvFaqvAAAAH0lEQVR42mP4ukqOKohh1KBRg0YNGjVo1KBRgwbeIADadOluhHvNaQAAAABJRU5ErkJggg=="),
];

/// The sun asset in image.json is 1362x1362; match it so a replacement fills
/// the same box.
const IMAGE_SIZE: u32 = 1362;

fn image_static(player: &mut Player, slot_id: &str, index: usize) -> Result<String> {
    let (name, src) = IMAGES[index];
    let slot = ImageSlot::from_src(src.to_string()).with_dimensions(IMAGE_SIZE, IMAGE_SIZE);
    player.set_image_slot(slot_id, slot)?;
    Ok(name.to_string())
}

struct App {
    canvas: common::Canvas,
    demo: usize,
    value: usize,
    status: String,
}

impl App {
    fn new() -> Self {
        let mut canvas = common::Canvas::new();
        canvas.player.set_autoplay(true);
        canvas.player.set_loop(true);

        let mut app = Self {
            canvas,
            demo: 0,
            value: 0,
            status: String::new(),
        };
        app.select(0);
        app
    }

    fn select(&mut self, demo: usize) {
        self.demo = demo;
        self.value = 0;

        let animation = CString::new(DEMOS[demo].animation).expect("animation JSON has a NUL");
        self.canvas
            .player
            .load_animation_data(&animation)
            .expect("Failed to load animation");

        self.set_static();
    }

    fn set_static(&mut self) {
        let demo = &DEMOS[self.demo];
        let outcome = (demo.set_static)(&mut self.canvas.player, demo.slot_id, self.value);
        self.report(outcome);
    }

    fn report(&mut self, outcome: Result<String>) {
        self.status = outcome.unwrap_or_else(|err| format!("failed: {err}"));
        println!("[{}] {}", DEMOS[self.demo].name, self.status);
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let dt = self.canvas.dt();
        let _ = self.canvas.player.tick(dt);

        egui::Panel::left("slot types").show(ui, |ui| {
            ui.heading("Slot type");
            for (index, demo) in DEMOS.iter().enumerate() {
                if ui.selectable_label(index == self.demo, demo.name).clicked() {
                    self.select(index);
                }
            }
        });

        egui::CentralPanel::default().show(ui, |ui| {
            let demo = &DEMOS[self.demo];

            ui.horizontal(|ui| {
                if ui.button("Next value").clicked() {
                    self.value = (self.value + 1) % demo.values;
                    self.set_static();
                }

                if let Some(set_animated) = demo.set_animated {
                    if ui.button("Keyframed").clicked() {
                        let outcome = set_animated(&mut self.canvas.player, demo.slot_id);
                        self.report(outcome);
                    }
                }

                if ui.button("Reset").clicked() {
                    let outcome = self
                        .canvas
                        .player
                        .reset_slot(demo.slot_id)
                        .map(|()| "reset to the authored value".to_string());
                    self.report(outcome);
                }
            });

            ui.label(format!("\"{}\": {}", demo.slot_id, self.status));
            self.canvas.show(ui);
        });
    }
}

fn main() {
    common::run("dotLottie slots", App::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_demo_applies_its_values() {
        let mut canvas = common::Canvas::new();
        let player = &mut canvas.player;

        for demo in DEMOS {
            let animation = CString::new(demo.animation).unwrap();
            player.load_animation_data(&animation).unwrap();

            for index in 0..demo.values {
                (demo.set_static)(player, demo.slot_id, index)
                    .unwrap_or_else(|err| panic!("{} value {index}: {err}", demo.name));
            }
            if let Some(set_animated) = demo.set_animated {
                set_animated(player, demo.slot_id)
                    .unwrap_or_else(|err| panic!("{} keyframes: {err}", demo.name));
            }
            player
                .reset_slot(demo.slot_id)
                .unwrap_or_else(|err| panic!("{} reset: {err}", demo.name));
        }
    }

    #[test]
    fn every_shape_shares_a_vertex_count() {
        for (_, build) in SHAPES {
            let path = build();
            assert!(path.is_valid());
            assert_eq!(path.vertices.len(), POINTS);
        }
    }
}
