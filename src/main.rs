#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use minifb::{Key, Window, WindowOptions};

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 900;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Thorvg inside Rust - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_millis(30)));

    /*
     * Init Thorvg
     */
    unsafe { tvg_engine_init(Tvg_Engine_TVG_ENGINE_SW, 0) };

    let canvas = unsafe { tvg_swcanvas_create() };

    let raw_ptr = buffer.as_mut_ptr();

    unsafe {
        tvg_swcanvas_set_target(
            canvas,
            raw_ptr,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            Tvg_Colorspace_TVG_COLORSPACE_ARGB8888,
        );
    };

    // Generate a rounded rectangle
    // let rect = unsafe { tvg_shape_new() };
    // unsafe { tvg_shape_append_rect(rect, 0.0, 0.0, 400.0, 400.0, 50.0, 50.0) };
    // unsafe { tvg_shape_set_fill_color(rect, 10, 70, 100, 100) };
    // unsafe { tvg_canvas_push(canvas, rect) };
    // unsafe { tvg_canvas_draw(canvas) };
    // unsafe { tvg_canvas_sync(canvas) };

    // Generate a scene
    let scene = unsafe { tvg_scene_new() };
    // Generate a round rectangle
    let rect = unsafe { tvg_shape_new() };
    unsafe { tvg_shape_append_rect(rect, -235.0, -250.0, 400.0, 400.0, 50.0, 50.0) };
    unsafe { tvg_shape_set_fill_color(rect, 0, 255, 0, 100) };
    unsafe { tvg_scene_push(scene, rect) };

    // Generate a circle
    let circle = unsafe { tvg_shape_new() };
    unsafe { tvg_shape_append_circle(circle, -165.0, -150.0, 200.0, 200.0) };
    unsafe { tvg_shape_set_fill_color(circle, 255, 255, 0, 127) };
    unsafe { tvg_scene_push(scene, circle) };

    // Generate a ellipse
    let ellipse = unsafe { tvg_shape_new() };
    unsafe { tvg_shape_append_circle(ellipse, 265.0, 250.0, 150.0, 100.0) };
    unsafe { tvg_shape_set_fill_color(ellipse, 0, 255, 255, 100) };
    unsafe { tvg_scene_push(scene, ellipse) };

    // Transform the scene [Not available]
    unsafe { tvg_paint_translate(scene, 350.0, 350.0) };
    unsafe { tvg_paint_scale(scene, 0.5) };
    unsafe { tvg_paint_rotate(scene, 45.0) };

    unsafe { tvg_canvas_push(canvas, scene) };
    unsafe { tvg_canvas_draw(canvas) };
    unsafe { tvg_canvas_sync(canvas) };

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // for y in 0..HEIGHT {
        //     for x in 0..WIDTH {
        //         let index = y * WIDTH + x;
        //         buffer[index] = 0; // Clear the buffer
        //     }
        // }

        if window.is_key_down(Key::Up) || window.is_key_down(Key::W) {}
        if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {}
        if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {}
        if window.is_key_down(Key::Right) || window.is_key_down(Key::D) {}

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
