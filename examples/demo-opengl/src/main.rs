use dotlottie_rs::{ColorSpace, Config, DotLottiePlayer};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use std::num::NonZeroU32;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[cfg(target_os = "macos")]
fn get_cgl_context(_context: &PossiblyCurrentContext) -> *mut std::ffi::c_void {
    // On macOS, get the current CGL context
    extern "C" {
        fn CGLGetCurrentContext() -> *mut std::ffi::c_void;
    }
    unsafe { CGLGetCurrentContext() }
}

#[cfg(not(target_os = "macos"))]
fn get_gl_context(_context: &PossiblyCurrentContext) -> *mut std::ffi::c_void {
    std::ptr::null_mut()
}

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

struct App {
    window: Option<Window>,
    gl_context: Option<glutin::context::PossiblyCurrentContext>,
    gl_surface: Option<Surface<WindowSurface>>,
    player: Option<DotLottiePlayer>,
    first_render: bool,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            gl_context: None,
            gl_surface: None,
            player: None,
            first_render: true,
        }
    }

    fn setup_gl(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("dotLottie OpenGL Example")
            .with_inner_size(winit::dpi::PhysicalSize::new(WIDTH, HEIGHT));

        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_transparency(false);

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
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
        let raw_window_handle = window.window_handle().ok().map(|h| h.as_raw());

        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .expect("Failed to create GL context")
        };

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle.unwrap(),
            NonZeroU32::new(WIDTH).unwrap(),
            NonZeroU32::new(HEIGHT).unwrap(),
        );

        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .expect("Failed to create GL surface")
        };

        let gl_context = not_current_gl_context
            .make_current(&gl_surface)
            .expect("Failed to make GL context current");

        // Load OpenGL function pointers - CRITICAL for ThorVG
        // ThorVG's GL backend needs GL functions to be loaded before set_gl_target
        gl::load_with(|symbol| {
            let symbol_cstr = std::ffi::CString::new(symbol).unwrap();
            gl_display.get_proc_address(&symbol_cstr) as *const _
        });

        // Set up OpenGL state for ThorVG rendering
        unsafe {
            gl::Viewport(0, 0, WIDTH as i32, HEIGHT as i32);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);

            // IMPORTANT: Enable blending - ThorVG needs this for alpha compositing
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            // Disable depth test (2D rendering)
            gl::Disable(gl::DEPTH_TEST);

            // Check what framebuffer is bound
            let mut fbo: i32 = -1;
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fbo);
            println!("Current framebuffer binding: {}", fbo);

            // Check for GL errors
            let err = gl::GetError();
            if err != gl::NO_ERROR {
                eprintln!("OpenGL error during setup: 0x{:x}", err);
            }
        }

        println!("✓ OpenGL initialized");
        println!("  Vendor: {}", unsafe {
            std::ffi::CStr::from_ptr(gl::GetString(gl::VENDOR) as *const i8).to_string_lossy()
        });
        println!("  Renderer: {}", unsafe {
            std::ffi::CStr::from_ptr(gl::GetString(gl::RENDERER) as *const i8).to_string_lossy()
        });

        // Initialize player with OpenGL renderer
        let threads = std::thread::available_parallelism().unwrap().get() as u32;
        println!("Using {} threads", threads);

        let mut player = DotLottiePlayer::new(
            Config {
                autoplay: true,
                loop_animation: true,
                ..Default::default()
            },
            threads,
        );

        // IMPORTANT: Call set_gl_target BEFORE loading animation data
        // This configures ThorVG to use OpenGL as the renderer

        // Ensure context is current before getting pointer and calling set_gl_target
        // This is critical for ThorVG's GL canvas initialization
        gl_context.make_current(&gl_surface).unwrap();

        // Ensure all GL operations are complete before getting context
        // This ensures the GL context is fully initialized and ready
        unsafe {
            gl::Flush();
            gl::Finish();

            // Do a test render to ensure GL is working
            gl::Clear(gl::COLOR_BUFFER_BIT);

            let err = gl::GetError();
            if err != gl::NO_ERROR {
                eprintln!("GL error before set_gl_target: 0x{:x}", err);
            }
        }

        // Get the current OpenGL context pointer
        #[cfg(target_os = "macos")]
        let context_ptr = get_cgl_context(&gl_context);
        #[cfg(not(target_os = "macos"))]
        let context_ptr = get_gl_context(&gl_context);

        // Query the actual framebuffer binding instead of assuming 0
        let fbo_id = unsafe {
            let mut fbo: i32 = -1;
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fbo);
            println!("Bound framebuffer during setup: {}", fbo);
            fbo
        };

        println!("Setting GL target with context: {:?}, fbo: {}", context_ptr, fbo_id);

        // Try multiple times if it fails - sometimes GL needs a moment
        let mut success = false;
        for attempt in 1..=3 {
            success = player.set_gl_target(
                context_ptr,
                fbo_id,
                WIDTH,
                HEIGHT,
                ColorSpace::ABGR8888S, // Must be ABGR8888S for GL
            );

            if success {
                break;
            }

            println!("Attempt {} failed, retrying...", attempt);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        if success {
            println!("✓ OpenGL target set successfully");
        } else {
            eprintln!("✗ Failed to set OpenGL target");
        }

        // Now load the animation
        player.load_animation_path("src/bouncy_ball.json", WIDTH, HEIGHT);
        // Start playing (autoplay in config may not work immediately)
        player.play();

        println!("✓ Animation loaded successfully");
        println!("   Total frames: {}", player.total_frames());
        println!("   Duration: {:.2}s", player.duration());

        self.window = Some(window);
        self.gl_context = Some(gl_context);
        self.gl_surface = Some(gl_surface);
        self.player = Some(player);
    }

    fn render(&mut self) {
        if let (Some(player), Some(gl_context), Some(gl_surface)) = (
            self.player.as_mut(),
            self.gl_context.as_ref(),
            self.gl_surface.as_ref(),
        ) {
            // On first render, re-set the GL target to ensure context is properly bound
            // This helps avoid flakiness from context initialization timing issues
            if self.first_render {
                println!("First render - ensuring GL context is bound...");

                // Check current framebuffer
                let mut fbo: i32 = -1;
                unsafe {
                    gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fbo);
                }
                println!("Framebuffer at first render: {}", fbo);

                #[cfg(target_os = "macos")]
                let context_ptr = get_cgl_context(&gl_context);
                #[cfg(not(target_os = "macos"))]
                let context_ptr = get_gl_context(&gl_context);

                // Use the actual bound framebuffer, not 0
                player.set_gl_target(
                    context_ptr,
                    fbo,
                    WIDTH,
                    HEIGHT,
                    ColorSpace::ABGR8888S,
                );

                self.first_render = false;
            }

            // Explicitly bind the default framebuffer before rendering
            // This ensures ThorVG renders to the window's framebuffer
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            }

            let rendered = player.tick();

            // Debug: Check if rendering actually happened
            static mut FRAME_COUNT: u32 = 0;
            unsafe {
                FRAME_COUNT += 1;
                if FRAME_COUNT % 60 == 0 {
                    println!("Frame {}: rendered={}, current_frame={:.1}",
                             FRAME_COUNT, rendered, player.current_frame());

                    // Check for GL errors
                    let err = gl::GetError();
                    if err != gl::NO_ERROR {
                        println!("OpenGL error: 0x{:x}", err);
                    }
                }
            }

            // Ensure all OpenGL commands complete before swap
            // This is critical for ThorVG's async GL rendering
            unsafe {
                gl::Finish();
            }

            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.setup_gl(event_loop);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close requested, exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.render();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    println!("\n╔════════════════════════════════════════════════════╗");
    println!("║     DotLottie OpenGL Renderer Example             ║");
    println!("╚════════════════════════════════════════════════════╝");
    println!("\nThis example demonstrates using set_gl_target() to");
    println!("configure ThorVG to render using OpenGL instead of");
    println!("the software renderer.\n");
    println!("Key points:");
    println!("  • set_gl_target() must be called BEFORE loading");
    println!("    animation data");
    println!("  • ThorVG renders directly to OpenGL framebuffer");
    println!("  • Better performance for complex animations\n");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
