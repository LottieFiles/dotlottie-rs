#![allow(clippy::print_stdout)]

/// Time Remapping Example
///
/// Drives a precomp layer's Lottie time-remap (`tm`) property through a float slot.
/// Drag the scrubber at the bottom (or use the arrow keys) to control the precomp's
/// *inner* timeline, while the marker in the top right keeps running on *outer* time.
///
/// The dial is drawn by ThorVG at the remapped time; the scrubber is drawn here from
/// our own `inner_time`. They are independent paths to the same number, so a broken
/// remap shows up as the two disagreeing.
///
/// Three things to know about `tm`:
/// - The value is in seconds, not frames. ThorVG feeds it to `frameAtTime()`, which
///   reduces to `seconds * fr` (60 here), clamped at zero.
/// - A negative value means "no remap"; the property defaults to -1.
/// - Only precomp layers honour it.
use dotlottie_rs::{ColorSpace, Player, ScalarSlot};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use std::ffi::CString;

mod common;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

const SLOT_ID: &str = "inner_time";

/// The precomp's inner timeline: 180 frames at 60 fps.
const INNER_DURATION: f32 = 3.0;

/// Seconds of inner time scrubbed per second of arrow key held.
const SCRUB_RATE: f32 = 1.0;

const TRACK_X: usize = 90;
const TRACK_W: usize = 420;
const TRACK_Y: usize = 560;
const TRACK_H: usize = 10;
const KNOB_R: i32 = 11;

/// Vertical slack around the track that still counts as grabbing it.
const GRAB_PAD: f32 = 24.0;

const COLOR_TRACK: u32 = rgb(0x3a, 0x3f, 0x4a);
const COLOR_FILL: u32 = rgb(0x26, 0xa6, 0xff);
const COLOR_KNOB: u32 = rgb(0xf2, 0xf4, 0xf8);

/// Packs into the `ColorSpace::ABGR8888` layout the player renders into.
const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    0xff00_0000 | ((b as u32) << 16) | ((g as u32) << 8) | r as u32
}

fn main() {
    let mut window = Window::new(
        "Time Remap Example - drag the scrubber, or LEFT/RIGHT, R resets",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut player = Player::new();
    player.set_loop(true);
    player.set_autoplay(true);

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();

    let animation_data = include_str!("../assets/animations/lottie/time_remap.json");
    let c_data = CString::new(animation_data).expect("CString conversion failed");

    if player.load_animation_data(&c_data).is_err() {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Animation loaded successfully!");
    println!("The green marker runs on OUTER time and keeps moving on its own.");
    println!("The dial lives in a precomp whose time you control.");
    println!();
    println!("Drag the scrubber, or LEFT/RIGHT to scrub, R to reset, ESC to quit");

    let mut inner_time = 0.0_f32;
    let _ = player.set_scalar_slot(SLOT_ID, ScalarSlot::new(inner_time));
    println!("inner_time {inner_time:.2}s / {INNER_DURATION:.2}s");

    let mut clock = common::Clock::new();
    let mut last_print = std::time::Instant::now();
    let mut dragging = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = clock.dt();

        let mut target = inner_time;

        if window.get_mouse_down(MouseButton::Left) {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
                // Once a drag starts, keep following the mouse even outside the track.
                if dragging || on_track(my) {
                    dragging = true;
                    target = time_at(mx);
                }
            }
        } else {
            dragging = false;
        }

        if !dragging {
            target = if window.is_key_down(Key::R) {
                0.0
            } else {
                let mut scrub = 0.0;
                if window.is_key_down(Key::Right) {
                    scrub += SCRUB_RATE * dt / 1000.0;
                }
                if window.is_key_down(Key::Left) {
                    scrub -= SCRUB_RATE * dt / 1000.0;
                }
                (inner_time + scrub).clamp(0.0, INNER_DURATION)
            };
        }

        if target != inner_time {
            inner_time = target;
            let _ = player.set_scalar_slot(SLOT_ID, ScalarSlot::new(inner_time));

            let now = std::time::Instant::now();
            if now.duration_since(last_print).as_millis() > 100 {
                println!("inner_time {inner_time:.2}s / {INNER_DURATION:.2}s");
                last_print = now;
            }
        }

        // The player repaints the whole canvas, so the scrubber is redrawn every frame.
        if player.tick(dt).unwrap_or(false) {
            draw_scrubber(&mut buffer, inner_time);
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        } else {
            window.update();
        }
    }

    println!("Example finished!");
}

fn on_track(my: f32) -> bool {
    my >= TRACK_Y as f32 - GRAB_PAD && my <= (TRACK_Y + TRACK_H) as f32 + GRAB_PAD
}

fn time_at(mx: f32) -> f32 {
    let t = (mx - TRACK_X as f32) / TRACK_W as f32;
    (t * INNER_DURATION).clamp(0.0, INNER_DURATION)
}

fn draw_scrubber(buffer: &mut [u32], inner_time: f32) {
    let progress = (inner_time / INNER_DURATION).clamp(0.0, 1.0);
    let filled = (TRACK_W as f32 * progress) as usize;

    fill_rect(buffer, TRACK_X, TRACK_Y, TRACK_W, TRACK_H, COLOR_TRACK);
    fill_rect(buffer, TRACK_X, TRACK_Y, filled, TRACK_H, COLOR_FILL);
    fill_disc(
        buffer,
        (TRACK_X + filled) as i32,
        (TRACK_Y + TRACK_H / 2) as i32,
        KNOB_R,
        COLOR_KNOB,
    );
}

fn fill_rect(buffer: &mut [u32], x: usize, y: usize, w: usize, h: usize, color: u32) {
    let stride = WIDTH as usize;
    for row in y..(y + h).min(HEIGHT as usize) {
        for col in x..(x + w).min(stride) {
            buffer[row * stride + col] = color;
        }
    }
}

fn fill_disc(buffer: &mut [u32], cx: i32, cy: i32, r: i32, color: u32) {
    let stride = WIDTH as usize;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy > r * r {
                continue;
            }
            let (x, y) = (cx + dx, cy + dy);
            if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
                buffer[y as usize * stride + x as usize] = color;
            }
        }
    }
}
