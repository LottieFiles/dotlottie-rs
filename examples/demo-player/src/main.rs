use dotlottie_rs::{Config, DotLottiePlayer};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::ContextAttributesBuilder;
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

use std::ffi::c_void;
use std::num::NonZeroU32;
use std::time::Instant;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;
const EASE_LINEAR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

#[cfg(target_os = "macos")]
fn get_cgl_context(context: &impl glutin::context::AsRawContext) -> *mut c_void {
    use glutin::context::RawContext;
    match context.raw_context() {
        RawContext::Cgl(cgl) => cgl as *mut c_void,
        _ => std::ptr::null_mut(),
    }
}

struct Player {
    player: DotLottiePlayer,
    current_marker: usize,
    last_update: Instant,
}

impl Player {
    fn new(animation_path: &str, gl_context_ptr: *mut c_void) -> Self {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            loop_animation: true,
            ..Default::default()
        });

        // Set the GL context BEFORE loading
        player.set_gl_context(gl_context_ptr, 0, WIDTH, HEIGHT);

        let is_dotlottie = animation_path.ends_with(".lottie");

        if is_dotlottie {
            let data = std::fs::read(animation_path).unwrap();
            player.load_dotlottie_data(&data, WIDTH, HEIGHT);
        } else {
            player.load_animation_path(animation_path, WIDTH, HEIGHT);
        }

        for marker in player.markers() {
            println!("Marker '{}' at frame {}", marker.name, marker.time);
        }

        if let Some(marker) = player.markers().first() {
            let mut config = player.config();
            config.marker = marker.name.clone();
            player.set_config(config);
        }

        Self {
            player,
            current_marker: 0,
            last_update: Instant::now(),
        }
    }

    fn update(&mut self) -> bool {
        self.player.tick()
    }

    fn play_marker(&mut self, index: usize) {
        let markers = self.player.markers();
        if index >= markers.len() || index == self.current_marker {
            return;
        }

        let marker = &markers[index];
        self.player
            .tween_to_marker(&marker.name, Some(1.0), Some(EASE_LINEAR.to_vec()));
        println!("Playing marker: '{}'", marker.name);
        let mut config = self.player.config();
        config.marker = marker.name.clone();
        self.player.set_config(config);

        self.current_marker = index;
    }

    fn next_marker(&mut self) {
        if self.player.is_tweening() {
            return;
        }
        let next = (self.current_marker + 1) % self.player.markers().len();
        self.play_marker(next);
    }
}

fn main() {
    println!("\nDemo Player Controls:");
    println!("  P - Play");
    println!("  S - Stop");
    println!("  → - Next marker");
    println!("  ESC - Exit\n");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let window_builder = WindowBuilder::new()
        .with_title("DotLottie GL Demo (ESC to exit)")
        .with_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT));

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);

    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            configs
                .reduce(|accum, config| {
                    if config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .unwrap();

    let window = window.unwrap();
    let raw_window_handle = window.raw_window_handle();
    let gl_display = gl_config.display();

    let context_attrs = ContextAttributesBuilder::new().build(Some(raw_window_handle));

    let gl_context = unsafe {
        gl_display
            .create_context(&gl_config, &context_attrs)
            .unwrap()
    };

    let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(WIDTH).unwrap(),
        NonZeroU32::new(HEIGHT).unwrap(),
    );

    let gl_surface = unsafe {
        gl_display
            .create_window_surface(&gl_config, &surface_attrs)
            .unwrap()
    };

    let gl_context = gl_context.make_current(&gl_surface).unwrap();
    gl_surface
        .set_swap_interval(&gl_context, SwapInterval::DontWait)
        .ok();

    #[cfg(target_os = "macos")]
    let cgl_context_ptr = get_cgl_context(&gl_context);

    #[cfg(not(target_os = "macos"))]
    let cgl_context_ptr: *mut c_void = std::ptr::null_mut();

    println!("GL Context pointer: {:?}", cgl_context_ptr);

    let mut player = Player::new("src/cartoon.json", cgl_context_ptr);

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(key),
                            ..
                        },
                    ..
                } => match key {
                    KeyCode::Escape => elwt.exit(),
                    KeyCode::KeyP => {
                        player.player.play();
                    }
                    KeyCode::KeyS => {
                        player.player.stop();
                    }
                    KeyCode::ArrowRight => player.next_marker(),
                    _ => {}
                },
                WindowEvent::RedrawRequested => {
                    player.update();
                    gl_surface.swap_buffers(&gl_context).unwrap();
                    window.request_redraw();
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        })
        .unwrap();
}
