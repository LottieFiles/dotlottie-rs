#![allow(warnings)]

use dotlottie_rs::{
    Animation, ColorSpace, Renderer, TvgAnimation, TvgBlendMethod, TvgEngine, TvgMatrix,
    TvgRenderer,
};
use instant::Instant;
use minifb::{Key, KeyRepeat, Window, WindowOptions};

const CANVAS_WIDTH: usize = 300;
const CANVAS_HEIGHT: usize = 300;

fn get_next_frame(current_time: f32, fps: f32, total_frame: f32, duration: f32) -> f32 {
    let next_frame = (current_time * fps) % total_frame;
    next_frame
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut frame_buffer = vec![0; CANVAS_WIDTH * CANVAS_HEIGHT * 4];

    let mut bg_animation = TvgAnimation::default();
    let mut bird_animation = TvgAnimation::default();

    let mut renderer = TvgRenderer::default();

    renderer
        .set_target(
            &mut frame_buffer,
            CANVAS_WIDTH as u32,
            CANVAS_WIDTH as u32,
            CANVAS_HEIGHT as u32,
            ColorSpace::ARGB8888S,
        )
        .unwrap();

    renderer.push_animation(&bg_animation)?;
    renderer.push_animation(&bird_animation)?;

    let bg_animation_data = include_str!("../bg.json");
    bg_animation.load_data(bg_animation_data, "lottie", false)?;

    let bird_animation_data = include_str!("../bird.json");
    bird_animation.load_data(bird_animation_data, "lottie", false)?;

    bird_animation.set_marker("flap")?;

    let mut window = Window::new(
        "demo - ESC to exit",
        CANVAS_WIDTH,
        CANVAS_HEIGHT,
        WindowOptions::default(),
    )?;

    let bg_animation_fps = bg_animation.get_total_frame()? / bg_animation.get_duration()?;
    let bird_animation_fps = bird_animation.get_total_frame()? / bird_animation.get_duration()?;

    let bg_animation_duration = bg_animation.get_duration()?;
    let bird_animation_duration = bird_animation.get_duration()?;

    let start_time = Instant::now();

    let mut updated = false;
    let mut bird_velocity = 0.0;
    let gravity = 0.05;
    let lift = -2.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let elapsed = now.duration_since(start_time).as_secs_f32();

        let next_frame1 = get_next_frame(
            elapsed % bg_animation_duration,
            bg_animation_fps,
            bg_animation.get_total_frame()?,
            bg_animation_duration,
        );
        let next_frame2 = get_next_frame(
            elapsed % bird_animation_duration,
            bird_animation_fps,
            bird_animation.get_total_frame()?,
            bird_animation_duration,
        );

        if bg_animation.set_frame(next_frame1).is_ok() {
            updated = true;
        }
        if bird_animation.set_frame(next_frame2).is_ok() {
            updated = true;
        }

        if updated {
            renderer.render()?;
            updated = false;
        }

        window.update_with_buffer(&frame_buffer, CANVAS_WIDTH, CANVAS_HEIGHT)?;

        if window.is_key_pressed(Key::Space, KeyRepeat::Yes) {
            bird_velocity = lift;
        } else {
            bird_velocity += gravity;
        }

        let (tx, ty) = bird_animation.get_translate()?;
        let new_ty = ty + bird_velocity;

        let (_, _, _, bird_height) = bird_animation.get_bounds()?;

        if new_ty < 0.0 {
            bird_animation.set_translate(tx, 0.0)?;
            bird_velocity = 0.0;
        } else if new_ty + bird_height < CANVAS_HEIGHT as f32 {
            bird_animation.set_translate(tx, new_ty)?;
        } else {
            bird_animation.set_translate(tx, CANVAS_HEIGHT as f32 - bird_height)?;
            bird_velocity = 0.0;
        }
    }

    Ok(())
}
