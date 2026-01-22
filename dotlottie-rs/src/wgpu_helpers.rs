// WebGPU helpers for iOS/macOS
// Provides simple API for creating WebGPU context from Metal layer

// #![cfg(all(feature = "tvg-wg", any(target_os = "macos", target_os = "ios")))]

use std::ffi::c_void;
use std::sync::{Arc, Mutex};

// Minimal FFI bindings to wgpu-native
mod ffi {
    use super::*;

    pub type WGPUInstance = *mut c_void;
    pub type WGPUAdapter = *mut c_void;
    pub type WGPUDevice = *mut c_void;
    pub type WGPUQueue = *mut c_void;
    pub type WGPUSurface = *mut c_void;

    pub type WGPUBackendType = u32;
    pub const WGPU_BACKEND_TYPE_METAL: WGPUBackendType = 0x00000005;

    pub type WGPUPowerPreference = u32;
    pub const WGPU_POWER_PREFERENCE_HIGH_PERFORMANCE: WGPUPowerPreference = 1;

    pub type WGPUFeatureLevel = u32;
    pub const WGPU_FEATURE_LEVEL_CORE: WGPUFeatureLevel = 0x00000002;

    pub type WGPUBool = u32;

    pub type WGPUCallbackMode = u32;
    pub const WGPU_CALLBACK_MODE_ALLOW_PROCESS_EVENTS: WGPUCallbackMode = 0x00000001;
    pub const WGPU_CALLBACK_MODE_ALLOW_SPONTANEOUS: WGPUCallbackMode = 0x00000003;

    pub type WGPURequestAdapterStatus = u32;
    pub const WGPU_REQUEST_ADAPTER_STATUS_SUCCESS: WGPURequestAdapterStatus = 0x00000001;

    pub type WGPURequestDeviceStatus = u32;
    pub const WGPU_REQUEST_DEVICE_STATUS_SUCCESS: WGPURequestDeviceStatus = 0x00000001;

    #[repr(C)]
    pub struct WGPUChainedStruct {
        pub next: *const WGPUChainedStruct,
        pub s_type: u32,
    }

    #[repr(C)]
    pub struct WGPUChainedStructOut {
        pub next: *mut WGPUChainedStructOut,
        pub s_type: u32,
    }

    #[repr(C)]
    pub struct WGPUInstanceCapabilities {
        pub next_in_chain: *mut WGPUChainedStructOut,
        pub timed_wait_any_enable: WGPUBool,
        pub timed_wait_any_max_count: usize,
    }

    #[repr(C)]
    pub struct WGPUInstanceDescriptor {
        pub next_in_chain: *const WGPUChainedStruct,
        pub features: WGPUInstanceCapabilities,
    }

    #[repr(C)]
    pub struct WGPUSurfaceSourceMetalLayer {
        pub chain: WGPUChainedStruct,
        pub layer: *mut c_void,
    }

    #[repr(C)]
    pub struct WGPUSurfaceDescriptor {
        pub next_in_chain: *const WGPUChainedStruct,
        pub label: *const std::os::raw::c_char,
    }

    #[repr(C)]
    pub struct WGPURequestAdapterOptions {
        pub next_in_chain: *const WGPUChainedStruct,
        pub feature_level: WGPUFeatureLevel,
        pub power_preference: WGPUPowerPreference,
        pub force_fallback_adapter: WGPUBool,
        pub backend_type: WGPUBackendType,
        pub compatible_surface: WGPUSurface,
    }

    #[repr(C)]
    pub struct WGPUQueueDescriptor {
        pub next_in_chain: *const WGPUChainedStruct,
        pub label: *const std::os::raw::c_char,
    }

    #[repr(C)]
    pub struct WGPUDeviceDescriptor {
        pub next_in_chain: *const WGPUChainedStruct,
        pub label: *const std::os::raw::c_char,
        pub required_feature_count: usize,
        pub required_features: *const u32,
        pub required_limits: *const c_void,
        pub default_queue: WGPUQueueDescriptor,
    }

    #[repr(C)]
    pub struct WGPUFuture {
        pub id: u64,
    }

    #[repr(C)]
    pub struct WGPUStringView {
        pub data: *const std::os::raw::c_char,
        pub length: usize,
    }

    pub type WGPURequestAdapterCallback = unsafe extern "C" fn(
        status: WGPURequestAdapterStatus,
        adapter: WGPUAdapter,
        message: WGPUStringView,
        userdata1: *mut c_void,
        userdata2: *mut c_void,
    );

    #[repr(C)]
    pub struct WGPURequestAdapterCallbackInfo {
        pub next_in_chain: *const WGPUChainedStruct,
        pub mode: WGPUCallbackMode,
        pub _padding: u32, // Padding for alignment
        pub callback: WGPURequestAdapterCallback,
        pub userdata1: *mut c_void,
        pub userdata2: *mut c_void,
    }

    pub type WGPURequestDeviceCallback = unsafe extern "C" fn(
        status: WGPURequestDeviceStatus,
        device: WGPUDevice,
        message: *const WGPUStringView,
        userdata1: *mut c_void,
        userdata2: *mut c_void,
    );

    #[repr(C)]
    pub struct WGPURequestDeviceCallbackInfo {
        pub next_in_chain: *const WGPUChainedStruct,
        pub mode: WGPUCallbackMode,
        pub _padding: u32, // Padding for alignment
        pub callback: WGPURequestDeviceCallback,
        pub userdata1: *mut c_void,
        pub userdata2: *mut c_void,
    }

    extern "C" {
        pub fn wgpuCreateInstance(descriptor: *const WGPUInstanceDescriptor) -> WGPUInstance;

        pub fn wgpuInstanceCreateSurface(
            instance: WGPUInstance,
            descriptor: *const WGPUSurfaceDescriptor,
        ) -> WGPUSurface;

        pub fn wgpuInstanceRequestAdapter(
            instance: WGPUInstance,
            options: *const WGPURequestAdapterOptions,
            callback_info: WGPURequestAdapterCallbackInfo,
        ) -> WGPUFuture;

        pub fn wgpuInstanceProcessEvents(instance: WGPUInstance);

        pub fn wgpuAdapterRequestDevice(
            adapter: WGPUAdapter,
            descriptor: *const WGPUDeviceDescriptor,
            callback_info: WGPURequestDeviceCallbackInfo,
        ) -> WGPUFuture;

        pub fn wgpuDeviceGetQueue(device: WGPUDevice) -> WGPUQueue;

        // Release functions
        pub fn wgpuInstanceRelease(instance: WGPUInstance);
        pub fn wgpuAdapterRelease(adapter: WGPUAdapter);
        pub fn wgpuDeviceRelease(device: WGPUDevice);
        pub fn wgpuQueueRelease(queue: WGPUQueue);
        pub fn wgpuSurfaceRelease(surface: WGPUSurface);
    }
}

/// WebGPU context that owns the instance, device, and surface
pub struct WgpuContext {
    instance: ffi::WGPUInstance,
    adapter: ffi::WGPUAdapter,
    device: ffi::WGPUDevice,
    queue: ffi::WGPUQueue,
    surface: ffi::WGPUSurface,
}

// Safe because wgpu-native handles are thread-safe opaque pointers
unsafe impl Send for WgpuContext {}
unsafe impl Sync for WgpuContext {}

impl WgpuContext {
    /// Create a new WebGPU context from a Metal layer
    ///
    /// # Arguments
    /// * `metal_layer` - Pointer to CAMetalLayer (from Swift: `layer as UnsafeMutableRawPointer`)
    ///
    /// # Safety
    /// The metal_layer pointer must be valid and point to a CAMetalLayer object
    pub unsafe fn new_from_metal_layer(metal_layer: *mut c_void) -> Result<Self, String> {
        println!("🦀 [Rust] new_from_metal_layer called");
        println!("🦀 [Rust] Metal layer pointer: {:p}", metal_layer);

        if metal_layer.is_null() {
            println!("🦀 [Rust] ❌ Metal layer pointer is null");
            return Err("Metal layer pointer is null".to_string());
        }

        // Create instance
        println!("🦀 [Rust] Creating WebGPU instance...");
        let instance_desc = ffi::WGPUInstanceDescriptor {
            next_in_chain: std::ptr::null(),
            features: ffi::WGPUInstanceCapabilities {
                next_in_chain: std::ptr::null_mut(),
                timed_wait_any_enable: 0,
                timed_wait_any_max_count: 0,
            },
        };

        let instance = ffi::wgpuCreateInstance(&instance_desc);
        if instance.is_null() {
            println!("🦀 [Rust] ❌ Failed to create WebGPU instance");
            return Err("Failed to create WebGPU instance".to_string());
        }
        println!("🦀 [Rust] ✓ Instance created: {:p}", instance);

        // Create surface from Metal layer
        println!("🦀 [Rust] Creating WebGPU surface from Metal layer...");
        let metal_source = ffi::WGPUSurfaceSourceMetalLayer {
            chain: ffi::WGPUChainedStruct {
                next: std::ptr::null(),
                s_type: 0x00000004, // WGPUSType_SurfaceSourceMetalLayer
            },
            layer: metal_layer,
        };

        let surface_desc = ffi::WGPUSurfaceDescriptor {
            next_in_chain: &metal_source.chain as *const ffi::WGPUChainedStruct,
            label: std::ptr::null(),
        };

        let surface = ffi::wgpuInstanceCreateSurface(instance, &surface_desc);
        if surface.is_null() {
            println!("🦀 [Rust] ❌ Failed to create WebGPU surface");
            ffi::wgpuInstanceRelease(instance);
            return Err("Failed to create WebGPU surface".to_string());
        }
        println!("🦀 [Rust] ✓ Surface created: {:p}", surface);

        // Request adapter
        println!("🦀 [Rust] Requesting WebGPU adapter...");
        let adapter_result: Arc<Mutex<Option<ffi::WGPUAdapter>>> = Arc::new(Mutex::new(None));
        // Use Box::into_raw to create a stable heap allocation for the callback
        let adapter_result_ptr = Box::into_raw(Box::new(adapter_result.clone()));
        println!("🦀 [Rust] adapter_result_ptr allocated at: {:p}", adapter_result_ptr);
        println!("🦀 [Rust] adapter_result Arc strong_count: {}", Arc::strong_count(&adapter_result));

        unsafe extern "C" fn adapter_callback(
            status: ffi::WGPURequestAdapterStatus,
            adapter: ffi::WGPUAdapter,
            message: ffi::WGPUStringView,
            userdata1: *mut c_void,
            _userdata2: *mut c_void,
        ) {
            println!("🦀 [Rust] ====== Adapter callback invoked ======");
            println!("🦀 [Rust] Callback userdata1 pointer: {:p}", userdata1);
            println!("🦀 [Rust] Callback status: {}", status);
            println!("🦀 [Rust] Callback adapter: {:p}", adapter);

            if userdata1.is_null() {
                eprintln!("🦀 [Rust] ❌ ERROR: userdata1 is null!");
                return;
            }

            println!("🦀 [Rust] About to cast userdata1 to Arc<Mutex<Option<WGPUAdapter>>>...");
            if status == ffi::WGPU_REQUEST_ADAPTER_STATUS_SUCCESS {
                println!("🦀 [Rust] ✓ Adapter request succeeded: {:p}", adapter);
                println!("🦀 [Rust] Dereferencing userdata1...");
                let result =
                    unsafe { &*(userdata1 as *const Arc<Mutex<Option<ffi::WGPUAdapter>>>) };
                println!("🦀 [Rust] Got Arc reference, locking mutex...");
                *result.lock().unwrap() = Some(adapter);
                println!("🦀 [Rust] ✓ Adapter stored successfully");
            } else {
                eprintln!(
                    "🦀 [Rust] ❌ Adapter request failed with status: {}",
                    status
                );
                if !message.data.is_null() && message.length > 0 {
                    let msg_bytes = unsafe {
                        std::slice::from_raw_parts(message.data as *const u8, message.length)
                    };
                    if let Ok(msg_str) = std::str::from_utf8(msg_bytes) {
                        println!("🦀 [Rust] Error message: {}", msg_str);
                    }
                }
            }
            println!("🦀 [Rust] ====== Adapter callback complete ======");
        }

        let adapter_options = ffi::WGPURequestAdapterOptions {
            next_in_chain: std::ptr::null(),
            feature_level: ffi::WGPU_FEATURE_LEVEL_CORE,
            power_preference: ffi::WGPU_POWER_PREFERENCE_HIGH_PERFORMANCE,
            force_fallback_adapter: 0,
            backend_type: ffi::WGPU_BACKEND_TYPE_METAL,
            compatible_surface: surface,
        };

        let callback_info = ffi::WGPURequestAdapterCallbackInfo {
            next_in_chain: std::ptr::null(),
            mode: ffi::WGPU_CALLBACK_MODE_ALLOW_SPONTANEOUS,
            _padding: 0,
            callback: adapter_callback,
            userdata1: adapter_result_ptr as *mut c_void,
            userdata2: std::ptr::null_mut(),
        };

        // Print adapter callback info for comparison
        println!("🦀 [Rust] sizeof(WGPURequestAdapterCallbackInfo) = {}", std::mem::size_of::<ffi::WGPURequestAdapterCallbackInfo>());
        let adapter_bytes = unsafe {
            std::slice::from_raw_parts(
                &callback_info as *const _ as *const u8,
                std::mem::size_of::<ffi::WGPURequestAdapterCallbackInfo>()
            )
        };
        println!("🦀 [Rust] Adapter callback raw bytes: {:02x?}", adapter_bytes);
        println!("🦀 [Rust] Registering adapter callback with userdata1: {:p}", adapter_result_ptr);
        println!("🦀 [Rust] Callback function pointer: {:p}", adapter_callback as *const ());
        println!("🦀 [Rust] callback_info.mode = {}", callback_info.mode);
        println!("🦀 [Rust] callback_info.userdata1 = {:p}", callback_info.userdata1);
        println!("🦀 [Rust] About to call wgpuInstanceRequestAdapter...");
        println!("🦀 [Rust] instance = {:p}", instance);
        println!("🦀 [Rust] adapter_options ptr = {:p}", &adapter_options);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        let _future = ffi::wgpuInstanceRequestAdapter(instance, &adapter_options, callback_info);
        println!("🦀 [Rust] wgpuInstanceRequestAdapter returned, future id: {}", _future.id);

        // Poll for adapter with timeout
        println!("🦀 [Rust] Polling for adapter (max 5 seconds)...");
        let adapter = {
            let start = std::time::Instant::now();
            let mut iterations = 0;
            loop {
                if iterations % 10 == 0 {
                    println!("🦀 [Rust] Polling iteration {}, calling wgpuInstanceProcessEvents...", iterations);
                }
                ffi::wgpuInstanceProcessEvents(instance);
                if iterations % 10 == 0 {
                    println!("🦀 [Rust] wgpuInstanceProcessEvents returned");
                }

                if let Some(adapter) = *adapter_result.lock().unwrap() {
                    println!("🦀 [Rust] Got adapter after {} iterations", iterations);
                    break adapter;
                }

                iterations += 1;
                if start.elapsed().as_secs() > 5 {
                    eprintln!(
                        "🦀 [Rust] ❌ Adapter request timed out after {} iterations",
                        iterations
                    );
                    // Clean up the heap-allocated userdata
                    let _ = unsafe { Box::from_raw(adapter_result_ptr) };
                    ffi::wgpuSurfaceRelease(surface);
                    ffi::wgpuInstanceRelease(instance);
                    return Err("Adapter request timed out".to_string());
                }

                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        };

        // Clean up the heap-allocated userdata
        let _ = unsafe { Box::from_raw(adapter_result_ptr) };

        if adapter.is_null() {
            println!("🦀 [Rust] ❌ Adapter is null after polling");
            ffi::wgpuSurfaceRelease(surface);
            ffi::wgpuInstanceRelease(instance);
            return Err("Failed to get WebGPU adapter".to_string());
        }
        println!("🦀 [Rust] ✓ Adapter validated: {:p}", adapter);

        // Request device
        println!("🦀 [Rust] Requesting WebGPU device...");
        let device_result: Arc<Mutex<Option<ffi::WGPUDevice>>> = Arc::new(Mutex::new(None));

        // WORKAROUND: wgpu-native has a bug where it doesn't pass userdata1 to device callbacks
        // Store the result in a thread-local static instead
        use std::cell::RefCell;
        thread_local! {
            static DEVICE_RESULT: RefCell<Option<Arc<Mutex<Option<ffi::WGPUDevice>>>>> = RefCell::new(None);
        }
        DEVICE_RESULT.with(|r| *r.borrow_mut() = Some(device_result.clone()));

        // Still create the pointer for consistency
        let device_result_ptr = Box::into_raw(Box::new(device_result.clone()));

        unsafe extern "C" fn device_callback(
            status: ffi::WGPURequestDeviceStatus,
            device: ffi::WGPUDevice,
            message: *const ffi::WGPUStringView,
            userdata1: *mut c_void,
            _userdata2: *mut c_void,
        ) {
            use std::cell::RefCell;
            thread_local! {
                static DEVICE_RESULT: RefCell<Option<Arc<Mutex<Option<ffi::WGPUDevice>>>>> = RefCell::new(None);
            }

            println!("🦀 [Rust] ====== Device callback invoked ======");
            println!("🦀 [Rust] Device callback status: {}", status);
            println!("🦀 [Rust] Device pointer: {:p}", device);
            println!("🦀 [Rust] Device userdata1: {:p}", userdata1);

            if status == ffi::WGPU_REQUEST_DEVICE_STATUS_SUCCESS {
                println!("🦀 [Rust] ✓ Device request succeeded");

                // WORKAROUND: Use thread-local instead of userdata1 (which wgpu-native doesn't pass correctly)
                println!("🦀 [Rust] Getting device result from thread-local...");
                DEVICE_RESULT.with(|r| {
                    if let Some(result) = r.borrow().as_ref() {
                        println!("🦀 [Rust] Locking mutex...");
                        *result.lock().unwrap() = Some(device);
                        println!("🦀 [Rust] ✓ Device stored successfully");
                    } else {
                        eprintln!("🦀 [Rust] ❌ ERROR: Device result not found in thread-local!");
                    }
                });
            } else {
                println!("🦀 [Rust] ❌ Device request failed with status: {}", status);
                if !message.is_null() {
                    let msg = unsafe { &*message };
                    if !msg.data.is_null() && msg.length > 0 {
                        let msg_bytes = unsafe {
                            std::slice::from_raw_parts(msg.data as *const u8, msg.length)
                        };
                        if let Ok(msg_str) = std::str::from_utf8(msg_bytes) {
                            println!("🦀 [Rust] Error message: {}", msg_str);
                        }
                    }
                }
            }
            println!("🦀 [Rust] ====== Device callback complete ======");
        }

        let device_descriptor = ffi::WGPUDeviceDescriptor {
            next_in_chain: std::ptr::null(),
            label: std::ptr::null(),
            required_feature_count: 0,
            required_features: std::ptr::null(),
            required_limits: std::ptr::null(),
            default_queue: ffi::WGPUQueueDescriptor {
                next_in_chain: std::ptr::null(),
                label: std::ptr::null(),
            },
        };

        let device_callback_info = ffi::WGPURequestDeviceCallbackInfo {
            next_in_chain: std::ptr::null(),
            mode: ffi::WGPU_CALLBACK_MODE_ALLOW_SPONTANEOUS,
            _padding: 0,
            callback: device_callback,
            userdata1: device_result_ptr as *mut c_void,
            userdata2: std::ptr::null_mut(),
        };

        println!("🦀 [Rust] sizeof(WGPURequestDeviceCallbackInfo) = {}", std::mem::size_of::<ffi::WGPURequestDeviceCallbackInfo>());
        println!("🦀 [Rust] offset of mode = {}", std::mem::offset_of!(ffi::WGPURequestDeviceCallbackInfo, mode));
        println!("🦀 [Rust] offset of _padding = {}", std::mem::offset_of!(ffi::WGPURequestDeviceCallbackInfo, _padding));
        println!("🦀 [Rust] offset of callback = {}", std::mem::offset_of!(ffi::WGPURequestDeviceCallbackInfo, callback));
        println!("🦀 [Rust] offset of userdata1 = {}", std::mem::offset_of!(ffi::WGPURequestDeviceCallbackInfo, userdata1));

        // Print raw bytes of the struct
        let bytes = unsafe {
            std::slice::from_raw_parts(
                &device_callback_info as *const _ as *const u8,
                std::mem::size_of::<ffi::WGPURequestDeviceCallbackInfo>()
            )
        };
        println!("🦀 [Rust] Raw struct bytes: {:02x?}", bytes);

        println!("🦀 [Rust] device_callback_info.userdata1 = {:p}", device_callback_info.userdata1);
        println!("🦀 [Rust] device_callback_info.callback = {:p}", device_callback_info.callback as *const ());
        println!("🦀 [Rust] device_callback_info.mode = {}", device_callback_info.mode);
        println!("🦀 [Rust] device_result_ptr = {:p}", device_result_ptr);
        println!("🦀 [Rust] About to call wgpuAdapterRequestDevice...");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        let _device_future =
            ffi::wgpuAdapterRequestDevice(adapter, &device_descriptor, device_callback_info);
        println!("🦀 [Rust] wgpuAdapterRequestDevice returned, future id: {}", _device_future.id);

        // Poll for device with timeout
        println!("🦀 [Rust] Polling for device (max 5 seconds)...");
        let device = {
            let start = std::time::Instant::now();
            let mut iterations = 0;
            loop {
                ffi::wgpuInstanceProcessEvents(instance);

                if let Some(device) = *device_result.lock().unwrap() {
                    println!("🦀 [Rust] Got device after {} iterations", iterations);
                    break device;
                }

                iterations += 1;
                if start.elapsed().as_secs() > 5 {
                    eprintln!(
                        "🦀 [Rust] ❌ Device request timed out after {} iterations",
                        iterations
                    );
                    // Clean up the heap-allocated userdata and thread-local
                    let _ = unsafe { Box::from_raw(device_result_ptr) };
                    DEVICE_RESULT.with(|r| *r.borrow_mut() = None);
                    ffi::wgpuAdapterRelease(adapter);
                    ffi::wgpuSurfaceRelease(surface);
                    ffi::wgpuInstanceRelease(instance);
                    return Err("Device request timed out".to_string());
                }

                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        };

        println!("🦀 [Rust] About to clean up device_result_ptr...");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        // Clean up the heap-allocated userdata
        let _ = unsafe { Box::from_raw(device_result_ptr) };
        // Clean up the thread-local
        DEVICE_RESULT.with(|r| *r.borrow_mut() = None);
        println!("🦀 [Rust] ✓ device_result_ptr and thread-local cleaned up");

        if device.is_null() {
            println!("🦀 [Rust] ❌ Device is null after polling");
            ffi::wgpuAdapterRelease(adapter);
            ffi::wgpuSurfaceRelease(surface);
            ffi::wgpuInstanceRelease(instance);
            return Err("Failed to get WebGPU device".to_string());
        }
        println!("🦀 [Rust] ✓ Device validated: {:p}", device);

        // Get queue
        println!("🦀 [Rust] Getting device queue...");
        println!("🦀 [Rust] About to call wgpuDeviceGetQueue with device: {:p}", device);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        let queue = ffi::wgpuDeviceGetQueue(device);
        println!("🦀 [Rust] wgpuDeviceGetQueue returned: {:p}", queue);
        if queue.is_null() {
            println!("🦀 [Rust] ❌ Failed to get queue");
            ffi::wgpuDeviceRelease(device);
            ffi::wgpuAdapterRelease(adapter);
            ffi::wgpuSurfaceRelease(surface);
            ffi::wgpuInstanceRelease(instance);
            return Err("Failed to get WebGPU queue".to_string());
        }
        println!("🦀 [Rust] ✓ Queue created: {:p}", queue);
        println!("🦀 [Rust] ✓✓✓ WebGPU context successfully created!");
        println!("🦀 [Rust] Creating WgpuContext struct...");
        std::io::Write::flush(&mut std::io::stdout()).ok();

        let context = WgpuContext {
            instance,
            adapter,
            device,
            queue,
            surface,
        };

        println!("🦀 [Rust] ✓ WgpuContext struct created, returning...");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        Ok(context)
    }

    /// Get pointers as u64 for passing to thorvg
    pub fn as_pointers(&self) -> (u64, u64, u64) {
        (
            self.device as u64,
            self.instance as u64,
            self.surface as u64,
        )
    }
}

impl Drop for WgpuContext {
    fn drop(&mut self) {
        unsafe {
            if !self.queue.is_null() {
                ffi::wgpuQueueRelease(self.queue);
            }
            if !self.device.is_null() {
                ffi::wgpuDeviceRelease(self.device);
            }
            if !self.adapter.is_null() {
                ffi::wgpuAdapterRelease(self.adapter);
            }
            if !self.surface.is_null() {
                ffi::wgpuSurfaceRelease(self.surface);
            }
            if !self.instance.is_null() {
                ffi::wgpuInstanceRelease(self.instance);
            }
        }
    }
}
