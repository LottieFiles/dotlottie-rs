/*
* 🚨 To run this example, use:
 * cargo run --example webgpu --features tvg-wg
*/
#![allow(clippy::print_stdout)]

mod common;

// This example requires WebGPU support, which is only available with specific features
// ==============================================================================
// WebGPU Implementation (only compiled when features are available)
// ==============================================================================
#[cfg(all(feature = "tvg-wg", target_os = "macos"))]
mod webgpu_impl {
    use dotlottie_rs::c_api::apple::WgpuContext;
    use dotlottie_rs::{DotLottiePlayer, WgpuDevice, WgpuInstance, WgpuTarget, WgpuTargetType};
    use std::ffi::CString;

    // Wrapper types for WebGPU pointers
    struct WebGpuDevice(*mut std::ffi::c_void);
    impl WgpuDevice for WebGpuDevice {
        fn as_ptr(&self) -> *mut std::ffi::c_void {
            self.0
        }

        unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
            Self(ptr)
        }
    }

    struct WebGpuInstance(*mut std::ffi::c_void);
    impl WgpuInstance for WebGpuInstance {
        fn as_ptr(&self) -> *mut std::ffi::c_void {
            self.0
        }

        unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
            Self(ptr)
        }
    }

    struct WebGpuSurface(*mut std::ffi::c_void);
    impl WgpuTarget for WebGpuSurface {
        fn as_ptr(&self) -> *mut std::ffi::c_void {
            self.0
        }

        unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
            Self(ptr)
        }
    }

    #[cfg(target_os = "macos")]
    use objc2::rc::Retained;
    #[cfg(target_os = "macos")]
    use objc2::runtime::AnyObject;
    #[cfg(target_os = "macos")]
    use objc2::{msg_send, msg_send_id, ClassType};
    #[cfg(target_os = "macos")]
    use objc2_app_kit::{NSView, NSWindow};
    #[cfg(target_os = "macos")]
    use objc2_foundation::{CGRect, CGSize};
    #[cfg(target_os = "macos")]
    use objc2_quartz_core::CAMetalLayer;
    #[cfg(target_os = "macos")]
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use winit::application::ApplicationHandler;
    use winit::event::WindowEvent;
    use winit::event_loop::ActiveEventLoop;
    use winit::window::{Window, WindowId};

    pub const WIDTH: u32 = 600;
    pub const HEIGHT: u32 = 600;

    #[cfg(target_os = "macos")]
    fn get_metal_layer(window: &Window) -> *mut std::ffi::c_void {
        unsafe {
            let window_handle = window.window_handle().unwrap();
            let raw_handle = window_handle.as_raw();

            if let RawWindowHandle::AppKit(handle) = raw_handle {
                // Get NSView from window handle
                let ns_view = handle.ns_view.as_ptr() as *mut NSView;
                let ns_view = &*ns_view;

                // Get NSWindow from NSView
                let ns_window: Option<Retained<NSWindow>> = msg_send_id![ns_view, window];
                let ns_window = ns_window.expect("Failed to get NSWindow");

                // Get content view
                let content_view: Option<Retained<NSView>> = msg_send_id![&ns_window, contentView];
                let content_view = content_view.expect("Failed to get content view");

                // Get or create the layer
                let layer: Option<Retained<AnyObject>> = msg_send_id![&content_view, layer];

                if let Some(layer) = layer {
                    // Check if it's already a CAMetalLayer
                    let is_metal_layer: bool =
                        msg_send![&*layer, isKindOfClass: CAMetalLayer::class()];

                    if is_metal_layer {
                        return Retained::as_ptr(&layer) as *mut std::ffi::c_void;
                    }
                }

                // Create a new CAMetalLayer
                let metal_layer = CAMetalLayer::new();

                // Set the layer's drawable size to match the window
                // This is critical for rendering to display properly
                let bounds: CGRect = msg_send![&content_view, bounds];
                let backing_scale_factor: f64 = msg_send![&ns_window, backingScaleFactor];
                let drawable_width = (bounds.size.width * backing_scale_factor) as f64;
                let drawable_height = (bounds.size.height * backing_scale_factor) as f64;
                let drawable_size = CGSize {
                    width: drawable_width,
                    height: drawable_height,
                };
                let _: () = msg_send![&metal_layer, setDrawableSize: drawable_size];

                let _: () = msg_send![&content_view, setLayer: &*metal_layer];
                let _: () = msg_send![&content_view, setWantsLayer: true];

                // Return the layer pointer (need to leak it to keep it alive)
                let ptr = Retained::as_ptr(&metal_layer) as *mut std::ffi::c_void;
                std::mem::forget(metal_layer);
                return ptr;
            }
        }

        std::ptr::null_mut()
    }

    #[cfg(not(target_os = "macos"))]
    fn get_metal_layer(_window: &Window) -> *mut std::ffi::c_void {
        eprintln!("WebGPU example only supports macOS currently");
        std::ptr::null_mut()
    }

    struct App {
        window: Option<Window>,
        player: Option<DotLottiePlayer>,
        #[cfg(all(feature = "tvg-wg", target_os = "macos"))]
        wgpu_context: Option<WgpuContext>,
        current_width: u32,
        current_height: u32,
        clock: super::common::Clock,
    }

    impl App {
        fn new() -> Self {
            Self {
                window: None,
                player: None,
                #[cfg(all(feature = "tvg-wg", target_os = "macos"))]
                wgpu_context: None,
                current_width: 0,
                current_height: 0,
                clock: super::common::Clock::new(),
            }
        }

        fn setup_webgpu(&mut self, event_loop: &ActiveEventLoop) {
            let window_attributes = Window::default_attributes()
                .with_title("dotLottie WebGPU Example")
                .with_inner_size(winit::dpi::PhysicalSize::new(WIDTH, HEIGHT));

            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");

            // Get the Metal layer from the window
            let metal_layer = get_metal_layer(&window);

            if metal_layer.is_null() {
                eprintln!("Failed to get Metal layer from window");
                return;
            }

            println!("✓ Metal layer obtained: {:?}", metal_layer);

            // Get actual window size (handles DPI scaling automatically)
            let window_size = window.inner_size();
            let width = window_size.width;
            let height = window_size.height;

            println!(
                "Window size: {}x{} (logical: {}x{})",
                width, height, WIDTH, HEIGHT
            );

            // Create WebGPU context using the helper from dotlottie-rs
            #[cfg(target_os = "macos")]
            {
                let wgpu_context = match unsafe { WgpuContext::from_metal_layer(metal_layer) } {
                    Ok(ctx) => ctx,
                    Err(e) => {
                        eprintln!("Failed to create WebGPU context: {}", e);
                        return;
                    }
                };

                let (device, instance, surface) = wgpu_context.as_pointers();
                println!("✓ WebGPU context created");

                let mut player = DotLottiePlayer::new();
                player.set_loop(true);
                player.set_autoplay(true);

                // IMPORTANT: Call set_wg_target BEFORE loading animation data
                // Use actual window size to handle DPI scaling
                let wgpu_device = WebGpuDevice(device as *mut std::ffi::c_void);
                let wgpu_instance = WebGpuInstance(instance as *mut std::ffi::c_void);
                let wgpu_surface = WebGpuSurface(surface as *mut std::ffi::c_void);

                let success = player
                    .set_wg_target(
                        &wgpu_device,
                        &wgpu_instance,
                        &wgpu_surface,
                        width,
                        height,
                        WgpuTargetType::Surface,
                    )
                    .is_ok();

                if success {
                    println!("✓ WebGPU target set successfully");
                } else {
                    eprintln!("✗ Failed to set WebGPU target");
                    return;
                }

                // Load animation with actual window size
                let animation_data = include_str!("../assets/animations/lottie/bouncy_ball.json");

                let c_data = CString::new(animation_data).expect("CString conversion failed");

                if player.load_animation_data(&c_data).is_err() {
                    eprintln!("Failed to load animation");
                    return;
                }

                let _ = player.play();

                println!("✓ Animation loaded successfully");
                println!("   Total frames: {}", player.total_frames());
                println!("   Duration: {:.2}s", player.duration());

                self.window = Some(window);
                self.player = Some(player);
                self.wgpu_context = Some(wgpu_context);
                self.current_width = width;
                self.current_height = height;
            }

            #[cfg(not(target_os = "macos"))]
            {
                eprintln!("WebGPU example only supports macOS currently");
            }
        }

        fn render(&mut self) {
            #[cfg(target_os = "macos")]
            {
                if let (Some(player), Some(wgpu_context)) =
                    (self.player.as_mut(), &self.wgpu_context)
                {
                    let dt = self.clock.dt();
                    let _ = player.tick(dt);

                    // CRITICAL: Present the surface to display the rendered frame
                    // Without this, rendering happens but nothing appears on screen
                    wgpu_context.present();
                }
            }

            #[cfg(not(target_os = "macos"))]
            {
                // Non-macOS platforms not yet supported
            }
        }

        fn handle_resize(&mut self, new_width: u32, new_height: u32) {
            // Skip if size hasn't changed or is invalid
            if new_width == 0 || new_height == 0 {
                return;
            }

            if new_width == self.current_width && new_height == self.current_height {
                return;
            }

            println!(
                "Window resized: {}x{} -> {}x{}",
                self.current_width, self.current_height, new_width, new_height
            );

            #[cfg(target_os = "macos")]
            {
                if let (Some(player), Some(wgpu_context)) =
                    (self.player.as_mut(), &self.wgpu_context)
                {
                    let (device, instance, surface) = wgpu_context.as_pointers();

                    // Reconfigure WebGPU target with new size
                    let wgpu_device = WebGpuDevice(device as *mut std::ffi::c_void);
                    let wgpu_instance = WebGpuInstance(instance as *mut std::ffi::c_void);
                    let wgpu_surface = WebGpuSurface(surface as *mut std::ffi::c_void);

                    let success = player
                        .set_wg_target(
                            &wgpu_device,
                            &wgpu_instance,
                            &wgpu_surface,
                            new_width,
                            new_height,
                            WgpuTargetType::Surface,
                        )
                        .is_ok();

                    if success {
                        // Reload animation with new size
                        let path = CString::new("src/bouncy_ball.json").unwrap();
                        let _ = player.load_animation_path(&path);
                        let _ = player.play();

                        self.current_width = new_width;
                        self.current_height = new_height;

                        println!("✓ Resized to {}x{}", new_width, new_height);
                    } else {
                        eprintln!("✗ Failed to resize WebGPU target");
                    }
                }
            }
        }
    }

    impl ApplicationHandler for App {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_none() {
                self.setup_webgpu(event_loop);
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
                WindowEvent::Resized(new_size) => {
                    self.handle_resize(new_size.width, new_size.height);
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
        println!("║     DotLottie WebGPU Renderer Example             ║");
        println!("╚════════════════════════════════════════════════════╝");
        println!("\nThis example demonstrates using set_wg_target() to");
        println!("configure ThorVG to render using WebGPU instead of");
        println!("the software renderer.\n");
        println!("Key points:");
        println!("  • set_wg_target() must be called BEFORE loading");
        println!("    animation data");
        println!("  • ThorVG renders using Metal backend via WebGPU");
        println!("  • Better performance for complex animations\n");

        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = App::new();
        event_loop.run_app(&mut app).unwrap();
    }
} // end of webgpu_impl module

// ==============================================================================
// Main functions (selected by feature gates)
// ==============================================================================
#[cfg(all(feature = "tvg-wg", target_os = "macos"))]
fn main() {
    webgpu_impl::run();
}

#[cfg(not(all(feature = "tvg-wg", target_os = "macos")))]
fn main() {
    eprintln!("This example requires:");
    eprintln!("  - Feature 'tvg-wg' to be enabled");
    eprintln!("  - macOS or iOS target");
    eprintln!("  - wgpu-native libraries must be present at build time");
    eprintln!("\nRun with:");
    eprintln!("  cargo run --example webgpu --features c_api,tvg,tvg-wg,tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-threads,tvg-lottie-expressions");
}
