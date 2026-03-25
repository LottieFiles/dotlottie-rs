/*
* 🚨 To run this example, use:
 * cargo run --example opengl --features tvg-gl
*/
#![allow(clippy::print_stdout)]

// This example requires OpenGL support, which is only available with specific features
// ==============================================================================
// OpenGL Implementation (only compiled when features are available)
// ==============================================================================
#[cfg(feature = "tvg-gl")]
mod opengl_impl {
    use dotlottie_rs::{
        DotLottiePlayer,
        GlContext as DotGlContext,
        GlDisplay as DotGlDisplay,
        GlSurface as DotGlSurface,
    };
    use glutin::config::ConfigTemplateBuilder;
    use glutin::context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext};
    use glutin::display::GetGlDisplay;
    use glutin::prelude::*;
    use glutin::surface::{Surface, SurfaceAttributesBuilder, WindowSurface};
    use glutin_winit::DisplayBuilder;
    use raw_window_handle::HasWindowHandle;
    use std::ffi::CString;
    use std::num::NonZeroU32;
    use winit::application::ApplicationHandler;
    use winit::event::WindowEvent;
    use winit::event_loop::ActiveEventLoop;
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

    // Wrapper for the platform display handle (e.g. EGLDisplay on Linux/Android).
    // Pass null on platforms that don't require it (macOS CGL, WGL).
    struct OpenGLDisplay(*mut std::ffi::c_void);

    impl DotGlDisplay for OpenGLDisplay {
        fn as_ptr(&self) -> *mut std::ffi::c_void {
            self.0
        }

        unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
            Self(ptr)
        }
    }

    // Wrapper for the platform surface handle (e.g. EGLSurface on Linux/Android).
    // Pass null on platforms that don't require it (macOS CGL, WGL).
    struct OpenGLSurface(*mut std::ffi::c_void);

    impl DotGlSurface for OpenGLSurface {
        fn as_ptr(&self) -> *mut std::ffi::c_void {
            self.0
        }

        unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
            Self(ptr)
        }
    }

    // Wrapper type for OpenGL context pointer
    struct OpenGLContext(*mut std::ffi::c_void);

    impl DotGlContext for OpenGLContext {
        fn as_ptr(&self) -> *mut std::ffi::c_void {
            self.0
        }

        unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
            Self(ptr)
        }
    }

    const WIDTH: u32 = 600;
    const HEIGHT: u32 = 600;

    pub struct App {
        window: Option<Window>,
        gl_context: Option<glutin::context::PossiblyCurrentContext>,
        gl_surface: Option<Surface<WindowSurface>>,
        player: Option<DotLottiePlayer>,
        first_render: bool,
    }

    impl App {
        pub fn new() -> Self {
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

            let display_builder =
                DisplayBuilder::new().with_window_attributes(Some(window_attributes));

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
                println!("Current framebuffer binding: {fbo}");

                // Check for GL errors
                let err = gl::GetError();
                if err != gl::NO_ERROR {
                    eprintln!("OpenGL error during setup: 0x{err:x}");
                }
            }

            let mut player = DotLottiePlayer::new();
            player.set_loop(true);
            player.set_autoplay(true);

            // IMPORTANT: Call set_gl_target BEFORE loading animation data
            // This configures ThorVG to use OpenGL as the renderer

            // Ensure context is current before getting pointer and calling set_gl_target
            // This is critical for ThorVG's GL canvas initialization
            gl_context.make_current(&gl_surface).unwrap();

            // CRITICAL: Do MULTIPLE render cycles to ensure GL context is fully initialized
            // On macOS, GL contexts can be flaky and need multiple swaps to be truly ready
            for i in 0..3 {
                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::Flush();
                    gl::Finish();
                }
                gl_surface.swap_buffers(&gl_context).unwrap();
                gl_context.make_current(&gl_surface).unwrap();

                // Small delay to let GL stabilize
                std::thread::sleep(std::time::Duration::from_millis(16));

                unsafe {
                    let err = gl::GetError();
                    if err != gl::NO_ERROR {
                        eprintln!("GL error during warmup cycle {i}: 0x{err:x}");
                    }
                }
            }
            println!("✓ GL context warmup complete");

            // Get the current OpenGL context pointer
            #[cfg(target_os = "macos")]
            let context_ptr = get_cgl_context(&gl_context);
            #[cfg(not(target_os = "macos"))]
            let context_ptr = get_gl_context(&gl_context);

            // Query the actual framebuffer binding instead of assuming 0
            let fbo_id = unsafe {
                let mut fbo: i32 = -1;
                gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fbo);
                println!("Bound framebuffer during setup: {fbo}");
                fbo
            };

            println!("Setting GL target with context: {context_ptr:?}, fbo: {fbo_id}");

            // Try multiple times if it fails - sometimes GL needs a moment
            let mut success = false;
            // On macOS (CGL), display and surface handles are not required — pass null.
            // On EGL platforms (Linux/Android), pass the EGLDisplay and EGLSurface here.
            let gl_display = OpenGLDisplay(std::ptr::null_mut());
            let gl_surface_handle = OpenGLSurface(std::ptr::null_mut());
            let gl_ctx = OpenGLContext(context_ptr);

            for attempt in 1..=5 {
                // Ensure context is current before each attempt
                gl_context.make_current(&gl_surface).unwrap();

                // Do a test render before each attempt
                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::Finish();
                }

                success = player
                    .set_gl_target(
                        &gl_display,
                        &gl_surface_handle,
                        &gl_ctx,
                        fbo_id,
                        WIDTH,
                        HEIGHT,
                    )
                    .is_ok();

                if success {
                    println!("✓ OpenGL target set successfully on attempt {attempt}");

                    // Verify it worked by doing a test render
                    unsafe {
                        gl::Clear(gl::COLOR_BUFFER_BIT);
                        gl::Finish();
                    }
                    gl_surface.swap_buffers(&gl_context).unwrap();
                    gl_context.make_current(&gl_surface).unwrap();

                    break;
                }

                println!("⚠ Attempt {attempt} failed, retrying...");
                std::thread::sleep(std::time::Duration::from_millis(50));
            }

            if !success {
                eprintln!("✗ Failed to set OpenGL target after 5 attempts");
                eprintln!("   This usually means GL context initialization failed");
                return;
            }

            let animation_data = include_str!("../assets/animations/lottie/bouncy_ball.json");

            let c_data = CString::new(animation_data).expect("CString conversion failed");

            if player.load_animation_data(&c_data, WIDTH, HEIGHT).is_err() {
                eprintln!("Failed to load animation");
                return;
            }

            // DON'T call play() yet - wait until first render
            // Otherwise animation advances during setup and we miss the first frames

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
                // On first render, start playback
                // No need to re-set GL target - we did aggressive initialization during setup
                if self.first_render {
                    println!("First render - starting playback from frame 0...");

                    // Start playing from the beginning
                    let _ = player.set_frame(0.0);
                    let _ = player.play();

                    self.first_render = false;
                }

                // Explicitly bind the default framebuffer before rendering
                // This ensures ThorVG renders to the window's framebuffer
                unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                    // CRITICAL: Clear the framebuffer before rendering to prevent flickering
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }

                let rendered = player.tick().is_ok();

                // Debug: Check if rendering actually happened
                static mut FRAME_COUNT: u32 = 0;
                unsafe {
                    FRAME_COUNT += 1;
                    let frame_count = FRAME_COUNT;
                    if frame_count % 60 == 0 {
                        println!(
                            "Frame {frame_count}: rendered={rendered}, current_frame={:.1}",
                            player.current_frame()
                        );

                        // Check for GL errors
                        let err = gl::GetError();
                        if err != gl::NO_ERROR {
                            println!("OpenGL error: 0x{err:x}");
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

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _id: WindowId,
            event: WindowEvent,
        ) {
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

    pub fn run() {
        use winit::event_loop::{ControlFlow, EventLoop};

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
} // end of opengl_impl module

// ==============================================================================
// Main functions (selected by feature gates)
// ==============================================================================
#[cfg(feature = "tvg-gl")]
fn main() {
    opengl_impl::run();
}

#[cfg(not(feature = "tvg-gl"))]
fn main() {
    eprintln!("This example requires:");
    eprintln!("  - Feature 'tvg-gl' to be enabled");
    eprintln!();
    eprintln!("Run with:");
    eprintln!("  cargo run --example opengl --features c_api,tvg,tvg-gl,tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-threads,tvg-lottie-expressions");
}
