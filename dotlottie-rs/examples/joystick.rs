#![allow(clippy::print_stdout)]

/// Joystick Example
///
/// This example demonstrates how to use the `set_position_slot` API to build an
/// interactive virtual joystick out of a Lottie animation.
///
/// joystick.json exposes three position slots:
///   - "joystick_bot_pos": the fixed base
///   - "joystick_top_pos": the draggable knob
///   - "triangle_pos": a character controlled by the knob's direction
///
/// Click and hold inside the joystick, then drag - the knob is clamped to a circular
/// region around the base. The triangle behaves like a top-down character controller:
/// its velocity is set by the knob's direction and how far it's pushed, so holding the
/// knob steady in one direction keeps the triangle moving that way, rather than snapping
/// to a fixed offset. Releasing the mouse springs the knob back to center, which decays
/// the triangle's velocity back to zero (it stays wherever it ended up).
use dotlottie_rs::{ColorSpace, LottieProperty, Player};
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::ffi::CString;

mod common;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

// Constants derived from joystick.json's layer transforms.
const JOY_CENTER: [f32; 2] = [438.09, 438.09];
const JOY_TOP_INIT: [f32; 2] = [437.87, 438.12];
const TRIANGLE_INIT: [f32; 2] = [256.00, 270.94];
// Clamping the knob's *center* to joystick_bot's radius (~31.78) means its own edge
// (knob center + knob radius) overhangs the base's edge by exactly the knob's own
// radius - half its width, not the full width.
const MAX_KNOB_RADIUS: f32 = 31.78;
const GRAB_RADIUS: f32 = 31.78;

// How quickly the knob eases back to center after release (per tick, 0..1).
const SPRING_FACTOR: f32 = 0.8;
const SPRING_EPSILON: f32 = 0.05;

// Character-controller speed: pixels/sec the triangle moves at full stick deflection.
const TRIANGLE_SPEED: f32 = 150.0;

fn main() {
    let mut window = Window::new(
        "Joystick Example - click and drag the knob",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    // Create player and load animation
    let mut player = Player::new();
    player.set_loop(true);
    player.set_autoplay(true);

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();

    let animation_data = include_str!("../assets/animations/lottie/joystick.json");

    let c_data = CString::new(animation_data).expect("CString conversion failed");

    if player.load_animation_data(&c_data).is_err() {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Animation loaded successfully!");
    println!("Click and hold inside the joystick, then push a direction to move the triangle");
    println!("Release to let the knob spring back and the triangle coast to a stop");
    println!("Press ESC to quit");

    // The base never moves.
    let _ = player.set_position_slot(
        "joystick_bot_pos",
        LottieProperty::static_value(JOY_TOP_INIT),
    );

    let mut knob_offset = [0.0_f32, 0.0_f32];
    let mut triangle_pos = TRIANGLE_INIT;
    let mut dragging = false;
    let mut mouse_was_down = false;
    let mut mx = 0.0_f32;
    let mut my = 0.0_f32;
    let mut clock = common::Clock::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = clock.dt();

        let mouse_pressed = window.get_mouse_down(MouseButton::Left);

        if let Some((new_mx, new_my)) = window.get_mouse_pos(minifb::MouseMode::Discard) {
            mx = new_mx;
            my = new_my;
        }

        if mouse_pressed && !mouse_was_down {
            let dist = ((mx - JOY_CENTER[0]).powi(2) + (my - JOY_CENTER[1]).powi(2)).sqrt();
            if dist <= GRAB_RADIUS {
                dragging = true;
            }
        } else if !mouse_pressed && mouse_was_down {
            dragging = false;
        }

        mouse_was_down = mouse_pressed;

        let mut offset_changed = false;

        if dragging && mouse_pressed {
            let mut delta = [mx - JOY_CENTER[0], my - JOY_CENTER[1]];
            let len = (delta[0].powi(2) + delta[1].powi(2)).sqrt();
            if len > MAX_KNOB_RADIUS {
                let scale = MAX_KNOB_RADIUS / len;
                delta[0] *= scale;
                delta[1] *= scale;
            }
            knob_offset = delta;
            offset_changed = true;
        } else if knob_offset[0] != 0.0 || knob_offset[1] != 0.0 {
            knob_offset[0] *= SPRING_FACTOR;
            knob_offset[1] *= SPRING_FACTOR;

            if knob_offset[0].abs() < SPRING_EPSILON && knob_offset[1].abs() < SPRING_EPSILON {
                knob_offset = [0.0, 0.0];
            }
            offset_changed = true;
        }

        if offset_changed {
            let knob_pos = [
                JOY_CENTER[0] + knob_offset[0],
                JOY_CENTER[1] + knob_offset[1],
            ];

            let dt_secs = dt / 1000.0;
            triangle_pos[0] += (knob_offset[0] / MAX_KNOB_RADIUS) * TRIANGLE_SPEED * dt_secs;
            triangle_pos[1] += (knob_offset[1] / MAX_KNOB_RADIUS) * TRIANGLE_SPEED * dt_secs;
            triangle_pos[0] = triangle_pos[0].clamp(0.0, WIDTH as f32);
            triangle_pos[1] = triangle_pos[1].clamp(0.0, HEIGHT as f32);

            let _ = player
                .set_position_slot("joystick_top_pos", LottieProperty::static_value(knob_pos));
            let _ = player
                .set_position_slot("triangle_pos", LottieProperty::static_value(triangle_pos));
        }

        if player.tick(dt).unwrap_or(false) {
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!("Example finished!");
}
