use std::ffi::{c_char};

// ============================================================================
// WEBGL C API (WASM-specific)
// Functions for WebGL context management
// ============================================================================

#[cfg(all(target_os = "emscripten", feature = "tvg-gl"))]
mod webgl_api {
    use super::*;

    /// Create a WebGL context for a canvas selector
    ///
    /// # Arguments
    /// * `selector` - CSS selector for the canvas element (e.g., "#myCanvas")
    ///
    /// # Returns
    /// A handle (uintptr_t) to the WebGL context, or 0 on failure
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgl_context_create(selector: *const c_char) -> usize {
        if selector.is_null() {
            return 0;
        }

        // Import Emscripten WebGL functions
        extern "C" {
            fn emscripten_webgl_create_context(
                target: *const c_char,
                attributes: *const EmscriptenWebGLContextAttributes,
            ) -> i32;
        }

        #[repr(C)]
        struct EmscriptenWebGLContextAttributes {
            alpha: bool,
            depth: bool,
            stencil: bool,
            antialias: bool,
            premultiplied_alpha: bool,
            preserve_drawing_buffer: bool,
            power_preference: i32,
            fail_if_major_performance_caveat: bool,
            major_version: i32,
            minor_version: i32,
            enable_extensions_by_default: bool,
            explicit_swap_control: bool,
            proxy_context_to_main_thread: i32,
            render_via_offscreen_back_buffer: bool,
        }

        let attrs = EmscriptenWebGLContextAttributes {
            alpha: true,
            depth: false,
            stencil: false,
            antialias: false,
            premultiplied_alpha: true,
            preserve_drawing_buffer: false,
            power_preference: 0, // default
            fail_if_major_performance_caveat: false,
            major_version: 2,
            minor_version: 0,
            enable_extensions_by_default: true,
            explicit_swap_control: false,
            proxy_context_to_main_thread: 0,
            render_via_offscreen_back_buffer: false,
        };

        let context = emscripten_webgl_create_context(selector, &attrs);
        context as usize
    }

    /// Make a WebGL context current
    ///
    /// # Returns
    /// 0 on success, non-zero on failure
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgl_context_make_current(context: usize) -> i32 {
        extern "C" {
            fn emscripten_webgl_make_context_current(context: i32) -> i32;
        }

        emscripten_webgl_make_context_current(context as i32)
    }

    /// Check if a WebGL context is lost
    ///
    /// # Returns
    /// 1 if context is lost, 0 otherwise
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgl_is_context_lost(context: usize) -> i32 {
        extern "C" {
            fn emscripten_is_webgl_context_lost(context: i32) -> bool;
        }

        if emscripten_is_webgl_context_lost(context as i32) {
            1
        } else {
            0
        }
    }

    /// Destroy a WebGL context
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgl_context_destroy(context: usize) {
        extern "C" {
            fn emscripten_webgl_destroy_context(context: i32);
        }

        emscripten_webgl_destroy_context(context as i32);
    }
}

// ============================================================================
// WEBGPU C API (WASM-specific)
// Functions for WebGPU device/surface management
// ============================================================================

#[cfg(all(target_os = "emscripten", feature = "tvg-wg"))]
mod webgpu_api {
    use super::*;
    use std::sync::Mutex;

    // Dawn/WebGPU FFI declarations for Emscripten
    extern "C" {
        fn emscripten_webgpu_get_device() -> usize;
        fn wgpuCreateInstance(descriptor: *const std::ffi::c_void) -> usize;
        fn wgpuInstanceCreateSurface(instance: usize, descriptor: *const std::ffi::c_void)
            -> usize;
        fn wgpuInstanceRelease(instance: usize);
        fn wgpuAdapterRelease(adapter: usize);
        fn wgpuDeviceRelease(device: usize);
        fn wgpuSurfaceRelease(surface: usize);
    }

    // Surface descriptor for canvas
    #[repr(C)]
    struct WGPUSurfaceDescriptorFromCanvasHTMLSelector {
        chain: WGPUChainedStruct,
        selector: *const c_char,
    }

    #[repr(C)]
    struct WGPUChainedStruct {
        next: *const std::ffi::c_void,
        stype: u32,
    }

    const WGPUSTYPE_SURFACE_DESCRIPTOR_FROM_CANVAS_HTMLSELECTOR: u32 = 0x00000004;

    // Global state for WebGPU
    static WEBGPU_STATE: Mutex<Option<WebGpuState>> = Mutex::new(None);

    struct WebGpuState {
        instance: usize,
        adapter: usize,
        device: usize,
    }

    impl Default for WebGpuState {
        fn default() -> Self {
            Self {
                instance: 0,
                adapter: 0,
                device: 0,
            }
        }
    }

    /// Get the WebGPU adapter handle
    ///
    /// # Returns
    /// Handle to the adapter, or 0 if not available
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgpu_get_adapter() -> usize {
        let state = WEBGPU_STATE.lock().unwrap();
        if let Some(ref s) = *state {
            return s.adapter;
        }
        0
    }

    /// Get the WebGPU device handle
    ///
    /// # Returns
    /// Handle to the device, or fallback to emscripten's device
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgpu_get_device() -> usize {
        let state = WEBGPU_STATE.lock().unwrap();
        if let Some(ref s) = *state {
            if s.device != 0 {
                return s.device;
            }
        }

        // Fallback to emscripten's device
        emscripten_webgpu_get_device()
    }

    /// Get the WebGPU instance handle
    ///
    /// # Returns
    /// Handle to the instance
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgpu_get_instance() -> usize {
        let mut state = WEBGPU_STATE.lock().unwrap();

        // Initialize state if needed
        if state.is_none() {
            *state = Some(WebGpuState::default());
        }

        if let Some(ref mut s) = *state {
            if s.instance == 0 {
                // Create a WebGPU instance using Dawn/Emscripten
                // Pass NULL descriptor for default configuration
                let instance = wgpuCreateInstance(std::ptr::null());
                s.instance = instance;

                if instance == 0 {
                    eprintln!("[WebGPU] Warning: Failed to create WebGPU instance");
                }
            }
            return s.instance;
        }

        0
    }

    /// Create a WebGPU surface for a canvas
    ///
    /// # Arguments
    /// * `canvas_selector` - CSS selector for the canvas element (e.g., "#canvas")
    ///
    /// # Returns
    /// Handle to the surface (0 on failure)
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgpu_get_surface(canvas_selector: *const c_char) -> usize {
        if canvas_selector.is_null() {
            eprintln!("[WebGPU] Cannot create surface: selector is null");
            return 0;
        }

        // Ensure instance exists
        let instance = dotlottie_webgpu_get_instance();
        if instance == 0 {
            eprintln!("[WebGPU] Cannot create surface: instance not initialized");
            return 0;
        }

        // Create surface descriptor with canvas selector
        let canvas_descriptor = WGPUSurfaceDescriptorFromCanvasHTMLSelector {
            chain: WGPUChainedStruct {
                next: std::ptr::null(),
                stype: WGPUSTYPE_SURFACE_DESCRIPTOR_FROM_CANVAS_HTMLSELECTOR,
            },
            selector: canvas_selector,
        };

        let surface_descriptor = WGPUSurfaceDescriptor {
            next_in_chain: &canvas_descriptor.chain as *const _ as *const std::ffi::c_void,
            label: std::ptr::null(),
        };

        let surface = wgpuInstanceCreateSurface(
            instance,
            &surface_descriptor as *const _ as *const std::ffi::c_void,
        );

        if surface == 0 {
            eprintln!("[WebGPU] Failed to create surface from selector");
        } else {
            println!("[WebGPU] Created surface: 0x{:x}", surface);
        }

        surface
    }

    /// Clean up WebGPU resources
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgpu_cleanup() {
        let mut state = WEBGPU_STATE.lock().unwrap();
        *state = None;
    }

    /// Release a WebGPU instance
    ///
    /// # Arguments
    /// * `instance` - Handle to the instance to release
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_wgpu_instance_release(instance: usize) {
        if instance != 0 {
            wgpuInstanceRelease(instance);
        }
    }

    /// Release a WebGPU adapter
    ///
    /// # Arguments
    /// * `adapter` - Handle to the adapter to release
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_wgpu_adapter_release(adapter: usize) {
        if adapter != 0 {
            wgpuAdapterRelease(adapter);
        }
    }

    /// Release a WebGPU device
    ///
    /// # Arguments
    /// * `device` - Handle to the device to release
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_wgpu_device_release(device: usize) {
        if device != 0 {
            wgpuDeviceRelease(device);
        }
    }

    /// Release a WebGPU surface
    ///
    /// # Arguments
    /// * `surface` - Handle to the surface to release
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_wgpu_surface_release(surface: usize) {
        if surface != 0 {
            wgpuSurfaceRelease(surface);
        }
    }

    #[repr(C)]
    struct WGPUSurfaceDescriptor {
        next_in_chain: *const std::ffi::c_void,
        label: *const c_char,
    }
}
