#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::{
    ffi::CString,
    fs::File,
    io::Read,
    time::{Duration, Instant},
};

use minifb::{Key, Window, WindowOptions};

pub const WIDTH: usize = 1200;
pub const HEIGHT: usize = 1200;

// Todo
// - Accept a buffer containg json data
// - Accept an image buffer

fn update_lottie(animation: *mut Tvg_Animation, canvas: *mut Tvg_Canvas, go_to_frame: &mut u32) {
    let mut total_frame: u32 = 0;
    let mut curr_frame: u32 = 0;

    unsafe { tvg_animation_get_total_frame(animation, &mut total_frame as *mut u32) };
    unsafe { tvg_animation_get_frame(animation, &mut curr_frame as *mut u32) };

    // let new_frame = total_frame * *go_to_frame;

    if *go_to_frame == curr_frame {
        return;
    }
    // println!("new frame {}", new_frame);
    println!("go_to_frame {}", *go_to_frame);

    unsafe { tvg_animation_set_frame(animation, *go_to_frame) };
    unsafe { tvg_canvas_update_paint(canvas, tvg_animation_get_picture(animation)) };

    //Draw the canvas
    unsafe { tvg_canvas_draw(canvas) };
    unsafe { tvg_canvas_sync(canvas) };

    println!("curr frmae: {} ", curr_frame);
    println!("total frame: {} ", total_frame);

    if *go_to_frame >= total_frame {
        *go_to_frame = 0;
    }
}

fn load_file(file_path: &str) -> (*const ::std::os::raw::c_char, u32) {
    println!("Loading file: {}", file_path);

    // Read a file in the local file system
    let mut data_file = File::open(file_path).unwrap();

    // Create an empty mutable string
    let mut file_content = String::new();

    // Copy contents of file to a mutable string
    data_file.read_to_string(&mut file_content).unwrap();

    return (
        file_content.as_ptr() as *const i8,
        file_content.len() as u32,
    );
}

// Tick will update the Lottie once a second has passed since last being called
struct Timer {
    last_update: Instant,
}

impl Timer {
    fn new() -> Self {
        Timer {
            last_update: Instant::now(),
        }
    }

    fn tick(
        &mut self,
        animation: *mut Tvg_Animation,
        canvas: *mut Tvg_Canvas,
        go_to_frame: &mut u32,
    ) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);
        let framerate = 30.0;
        let frame_duration: u64 = (1.0 / framerate * 1000.0) as u64;

        if elapsed >= Duration::from_millis(frame_duration) {
            self.last_update = now; // Update the last update time

            update_lottie(animation, canvas, go_to_frame);

            *go_to_frame += 1;
        }
    }
}

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

    // window.limit_update_rate(Some(std::time::Duration::from_millis(30)));

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
    // let scene = unsafe { tvg_scene_new() };
    // // Generate a round rectangle
    // let rect = unsafe { tvg_shape_new() };
    // unsafe { tvg_shape_append_rect(rect, -235.0, -250.0, 400.0, 400.0, 50.0, 50.0) };
    // unsafe { tvg_shape_set_fill_color(rect, 0, 255, 0, 100) };
    // unsafe { tvg_scene_push(scene, rect) };

    // // Generate a circle
    // let circle = unsafe { tvg_shape_new() };
    // unsafe { tvg_shape_append_circle(circle, -165.0, -150.0, 200.0, 200.0) };
    // unsafe { tvg_shape_set_fill_color(circle, 255, 255, 0, 127) };
    // unsafe { tvg_scene_push(scene, circle) };

    // // Generate a ellipse
    // let ellipse = unsafe { tvg_shape_new() };
    // unsafe { tvg_shape_append_circle(ellipse, 265.0, 250.0, 150.0, 100.0) };
    // unsafe { tvg_shape_set_fill_color(ellipse, 0, 255, 255, 100) };
    // unsafe { tvg_scene_push(scene, ellipse) };

    // // Transform the scene [Not available]
    // unsafe { tvg_paint_translate(scene, 350.0, 350.0) };
    // unsafe { tvg_paint_scale(scene, 0.5) };
    // unsafe { tvg_paint_rotate(scene, 45.0) };

    /*
       Load a Lottie animation
    */
    let animation = unsafe { tvg_animation_new() };
    let mut duration = 0.0;
    let path = "/Users/sam/Projects/LottieFiles/Github/@rust/thorvg-rust-wrapper/src/cartoon.json";

    // Get a raw pointer to the CString's buffer
    let pict_lottie = unsafe { tvg_animation_get_picture(animation) };

    if pict_lottie.is_null() {
        println!("pict is null!");
    }

    // if (unsafe { tvg_picture_load_raw(pict_lottie, load_file(path), 512, 512, false) }

    // if (unsafe { tvg_picture_load(pict_lottie, path_ptr) } != Tvg_Result_TVG_RESULT_SUCCESS) {

    let result = load_file(path);
    let mimetype = CString::new("lottie").expect("Failed to create CString");

    if (unsafe { tvg_picture_load_data(pict_lottie, result.0, result.1, mimetype.as_ptr(), false) }
        != Tvg_Result_TVG_RESULT_SUCCESS)
    {
        println!("Problem with loading an lottie file");

        unsafe { tvg_animation_del(animation) };
    } else {
        unsafe { tvg_paint_scale(pict_lottie, 1.0) };

        let mut total_frame: u32 = 0;
        unsafe { tvg_animation_get_total_frame(animation, &mut total_frame as *mut u32) };
        unsafe { tvg_animation_get_duration(animation, &mut duration) };

        unsafe { tvg_animation_set_frame(animation, 0) };

        unsafe { tvg_canvas_push(canvas, pict_lottie) };

        println!("Duration: {}", duration);

        println!("Total frames: {}", total_frame);
    }

    // unsafe { tvg_canvas_push(canvas, pict_lottie) };
    unsafe { tvg_canvas_draw(canvas) };
    unsafe { tvg_canvas_sync(canvas) };

    let mut go_to_frame: u32 = 0;
    let mut timer = Timer::new();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // for y in 0..HEIGHT {
        //     for x in 0..WIDTH {
        //         let index = y * WIDTH + x;
        //         buffer[index] = 0xff00ff00; // Clear the buffer
        //     }
        // }

        /*
           animation code
        */
        timer.tick(animation, canvas, &mut go_to_frame);

        if window.is_key_down(Key::Up) || window.is_key_down(Key::W) {}
        if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {}
        if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {}
        if window.is_key_down(Key::Right) || window.is_key_down(Key::D) {}

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
