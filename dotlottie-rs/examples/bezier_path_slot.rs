#![allow(clippy::print_stdout)]
//! Bezier path slot example (experimental).
//!
//! Demonstrates `set_bezier_path_slot`, which replaces the geometry of a path
//! (`ty: "sh"`) property tagged with a slot id. bezier_path.json exposes a
//! triangle under the slot id "shape_path".

use dotlottie_rs::{
    Bezier, BezierPath, BezierPathSlot, BezierValue, ColorSpace, LottieKeyframe, LottieProperty,
    Player,
};
use minifb::{Key, Window, WindowOptions};
use std::f32::consts::{FRAC_PI_2, TAU};
use std::ffi::CString;

mod common;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

/// Lottie interpolates paths point by point, so two keyframes only morph when
/// they have the same vertex count. 12 divides by 3, 4 and 6, so polygon corners
/// land on real vertices.
const POINTS: usize = 12;

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

/// Points on a circle of `radius`, with the cubic handles that round the segments
/// between them: k = 4/3 * tan(θ/4) * r.
fn circle(radius: f32) -> BezierPath {
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

/// Six-pointed star: alternate the outer and inner radius around the circle.
fn star(outer: f32, inner: f32) -> BezierPath {
    let step = TAU / POINTS as f32;
    let vertices = (0..POINTS)
        .map(|i| {
            let radius = if i % 2 == 0 { outer } else { inner };
            let (sin, cos) = (step * i as f32 - FRAC_PI_2).sin_cos();
            [cos * radius, sin * radius]
        })
        .collect();

    BezierPath::polygon(vertices, true)
}

fn triangle() -> BezierPath {
    sampled_polygon(&[[0.0, -80.0], [70.0, 45.0], [-70.0, 45.0]])
}

fn square() -> BezierPath {
    sampled_polygon(&[[-60.0, -60.0], [60.0, -60.0], [60.0, 60.0], [-60.0, 60.0]])
}

fn bezier(x: f32, y: f32) -> Option<Bezier> {
    Some(Bezier {
        x: BezierValue::Single(x),
        y: BezierValue::Single(y),
    })
}

fn morph_keyframe(frame: u32, path: BezierPath) -> LottieKeyframe<BezierPath> {
    LottieKeyframe {
        frame,
        start_value: path,
        // Ease-in-out between shapes.
        in_tangent: bezier(0.58, 1.0),
        out_tangent: bezier(0.42, 0.0),
        value_in_tangent: None,
        value_out_tangent: None,
        hold: None,
    }
}

/// One keyframe per shape across the animation's 60 frames, ending back on the
/// triangle so the loop is seamless.
fn morph_cycle() -> BezierPathSlot {
    LottieProperty::animated(vec![
        morph_keyframe(0, triangle()),
        morph_keyframe(15, square()),
        morph_keyframe(30, star(85.0, 38.0)),
        morph_keyframe(45, circle(70.0)),
        morph_keyframe(60, triangle()),
    ])
}

fn main() {
    let mut window = Window::new(
        "Bezier Path Slot Example - SPACE cycles shapes, M morphs",
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
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();

    let animation_data = include_str!("../assets/animations/lottie/bezier_path.json");
    let c_data = CString::new(animation_data).expect("CString conversion failed");

    if player.load_animation_data(&c_data).is_err() {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Animation loaded successfully!");
    println!("Press M to morph triangle -> square -> star -> circle -> triangle");
    println!("Press SPACE to hold a single shape");
    println!("Press ESC to quit");

    type Shape = (&'static str, fn() -> BezierPath);
    let shapes: [Shape; 4] = [
        ("triangle", triangle),
        ("square", square),
        ("star", || star(85.0, 38.0)),
        ("circle", || circle(70.0)),
    ];
    let mut shape_index = 0usize;
    let mut morphing = true;
    let mut last_key_press = std::time::Instant::now();
    let mut clock = common::Clock::new();

    let _ = player.set_bezier_path_slot("shape_path", morph_cycle());
    println!("Mode: MORPH");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        let dt = clock.dt();
        let debounced = now.duration_since(last_key_press).as_millis() > 200;

        if window.is_key_down(Key::Space) && debounced {
            let (name, build) = shapes[shape_index];
            shape_index = (shape_index + 1) % shapes.len();
            let _ = player.set_bezier_path_slot("shape_path", BezierPathSlot::new(build()));
            morphing = false;
            println!("Mode: STATIC | Shape: {name}");
            last_key_press = now;
        }

        if window.is_key_down(Key::M) && debounced && !morphing {
            let _ = player.set_bezier_path_slot("shape_path", morph_cycle());
            morphing = true;
            println!("Mode: MORPH");
            last_key_press = now;
        }

        if player.tick(dt).unwrap_or(false) {
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!("Example finished!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shape_shares_a_vertex_count() {
        for path in [triangle(), square(), star(85.0, 38.0), circle(70.0)] {
            assert!(path.is_valid());
            assert_eq!(path.vertices.len(), POINTS);
        }
    }
}
