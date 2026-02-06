//! WebGPU helpers for iOS/macOS
//! Provides API for creating WebGPU context from Metal layer

#![cfg(all(
    feature = "tvg-wg",
    any(target_os = "macos", target_os = "ios"),
    wgpu_native_linked
))]

use std::ffi::c_void;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

// Minimal FFI bindings to wgpu-native v25
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
        pub _padding: u32,
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
        pub _padding: u32,
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

        /// Process events on the instance
        /// Note: Not used in the new condvar-based implementation,
        /// but kept for potential future debugging or alternative approaches
        #[allow(dead_code)]
        pub fn wgpuInstanceProcessEvents(instance: WGPUInstance);

        pub fn wgpuAdapterRequestDevice(
            adapter: WGPUAdapter,
            descriptor: *const WGPUDeviceDescriptor,
            callback_info: WGPURequestDeviceCallbackInfo,
        ) -> WGPUFuture;
        pub fn wgpuDeviceGetQueue(device: WGPUDevice) -> WGPUQueue;
        pub fn wgpuSurfacePresent(surface: WGPUSurface);
        pub fn wgpuInstanceRelease(instance: WGPUInstance);
        pub fn wgpuAdapterRelease(adapter: WGPUAdapter);
        pub fn wgpuDeviceRelease(device: WGPUDevice);
        pub fn wgpuQueueRelease(queue: WGPUQueue);
        pub fn wgpuSurfaceRelease(surface: WGPUSurface);
    }
}

// Thread-local storage for device callback workaround
// This is needed because wgpu-native doesn't pass userdata1 correctly for device callbacks
use std::cell::RefCell;
thread_local! {
    static DEVICE_CALLBACK_RESULT: RefCell<Option<Arc<(Mutex<DeviceCallbackResult>, Condvar)>>> = RefCell::new(None);
}

struct DeviceCallbackResult {
    device: Option<ffi::WGPUDevice>,
    completed: bool,
}

/// Request adapter synchronously using condvar instead of polling
unsafe fn request_adapter_sync(
    instance: ffi::WGPUInstance,
    surface: ffi::WGPUSurface,
) -> Result<ffi::WGPUAdapter, String> {
    struct AdapterResult {
        adapter: Option<ffi::WGPUAdapter>,
        completed: bool,
    }

    let result = Arc::new((
        Mutex::new(AdapterResult {
            adapter: None,
            completed: false,
        }),
        Condvar::new(),
    ));

    let result_clone = result.clone();

    unsafe extern "C" fn adapter_callback(
        status: ffi::WGPURequestAdapterStatus,
        adapter: ffi::WGPUAdapter,
        _message: ffi::WGPUStringView,
        userdata1: *mut c_void,
        _userdata2: *mut c_void,
    ) {
        if !userdata1.is_null() {
            let result = &*(userdata1 as *const (Mutex<AdapterResult>, Condvar));
            let (lock, cvar) = result;
            let mut data = lock.lock().unwrap();

            if status == ffi::WGPU_REQUEST_ADAPTER_STATUS_SUCCESS {
                data.adapter = Some(adapter);
            } else {
                eprintln!("WebGPU adapter request failed with status: {}", status);
            }
            data.completed = true;
            cvar.notify_one();
        } else {
            eprintln!("WARNING: adapter callback userdata1 is null!");
        }
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
        userdata1: Arc::as_ptr(&result_clone) as *mut c_void,
        userdata2: std::ptr::null_mut(),
    };

    ffi::wgpuInstanceRequestAdapter(instance, &adapter_options, callback_info);

    // Wait with timeout using condvar (better than polling)
    let (lock, cvar) = &*result;
    let data = lock.lock().unwrap();

    // Wait for up to 10 seconds (Metal initialization can be slow on first launch)
    let wait_result = cvar
        .wait_timeout_while(data, Duration::from_secs(10), |data| !data.completed)
        .unwrap();

    if wait_result.1.timed_out() {
        return Err("Adapter request timed out after 10 seconds".to_string());
    }

    wait_result
        .0
        .adapter
        .ok_or_else(|| "Failed to get WebGPU adapter".to_string())
}

/// Request device synchronously using condvar instead of polling
///
/// Note: wgpu-native has a bug where it doesn't pass userdata1 correctly for device callbacks,
/// so we use thread_local storage as a workaround.
unsafe fn request_device_sync(adapter: ffi::WGPUAdapter) -> Result<ffi::WGPUDevice, String> {
    let result = Arc::new((
        Mutex::new(DeviceCallbackResult {
            device: None,
            completed: false,
        }),
        Condvar::new(),
    ));

    let result_clone = result.clone();

    // Workaround: Store result in thread_local because wgpu-native doesn't pass userdata1
    DEVICE_CALLBACK_RESULT.with(|r| *r.borrow_mut() = Some(result_clone.clone()));

    unsafe extern "C" fn device_callback(
        status: ffi::WGPURequestDeviceStatus,
        device: ffi::WGPUDevice,
        _message: *const ffi::WGPUStringView,
        _userdata1: *mut c_void,
        _userdata2: *mut c_void,
    ) {
        // Use thread_local because wgpu-native doesn't pass userdata1 correctly
        DEVICE_CALLBACK_RESULT.with(|r| {
            if let Some(result) = r.borrow().as_ref() {
                let (lock, cvar) = &**result;
                let mut data = lock.lock().unwrap();

                if status == ffi::WGPU_REQUEST_DEVICE_STATUS_SUCCESS {
                    data.device = Some(device);
                } else {
                    eprintln!("WebGPU device request failed with status: {}", status);
                }
                data.completed = true;
                cvar.notify_one();
            } else {
                eprintln!("WARNING: device callback thread_local is empty!");
            }
        });
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
        userdata1: Arc::as_ptr(&result_clone) as *mut c_void,
        userdata2: std::ptr::null_mut(),
    };

    ffi::wgpuAdapterRequestDevice(adapter, &device_descriptor, device_callback_info);

    // Wait with timeout using condvar
    let (lock, cvar) = &*result;
    let data = lock.lock().unwrap();

    // Wait for up to 10 seconds
    let wait_result = cvar
        .wait_timeout_while(data, Duration::from_secs(10), |data| !data.completed)
        .unwrap();

    if wait_result.1.timed_out() {
        DEVICE_CALLBACK_RESULT.with(|r| *r.borrow_mut() = None);
        return Err("Device request timed out after 10 seconds".to_string());
    }

    // Clean up thread_local
    DEVICE_CALLBACK_RESULT.with(|r| *r.borrow_mut() = None);

    wait_result
        .0
        .device
        .ok_or_else(|| "Failed to get WebGPU device".to_string())
}

/// WebGPU context that owns instance, device, and surface
pub struct WgpuContext {
    instance: ffi::WGPUInstance,
    adapter: ffi::WGPUAdapter,
    device: ffi::WGPUDevice,
    queue: ffi::WGPUQueue,
    surface: ffi::WGPUSurface,
}

unsafe impl Send for WgpuContext {}
unsafe impl Sync for WgpuContext {}

impl WgpuContext {
    /// Create WebGPU context from Metal layer
    ///
    /// # Arguments
    /// * `metal_layer` - Pointer to CAMetalLayer (from Swift: `layer as UnsafeMutableRawPointer`)
    ///
    /// # Safety
    /// - The metal_layer pointer must be valid and point to a CAMetalLayer object
    /// - **CRITICAL**: This function MUST be called from the main thread on iOS/macOS
    ///   Metal and WebGPU require certain operations to be on the main thread
    ///
    /// # Swift Usage
    /// ```swift
    /// // Ensure you're on the main thread before calling
    /// DispatchQueue.main.async {
    ///     let context = DotLottiePlayer.createWebGPUContext(metalLayer: metalLayer)
    /// }
    /// ```
    pub unsafe fn from_metal_layer(metal_layer: *mut c_void) -> Result<Self, String> {
        if metal_layer.is_null() {
            return Err("Metal layer pointer is null".to_string());
        }

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
            eprintln!("[RUST] ERROR: Failed to create WebGPU instance");
            return Err("Failed to create WebGPU instance".to_string());
        }

        // Create surface from Metal layer
        eprintln!(
            "[RUST] Creating surface from Metal layer {:?}...",
            metal_layer
        );
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
            eprintln!("[RUST] ERROR: Failed to create WebGPU surface");
            ffi::wgpuInstanceRelease(instance);
            return Err("Failed to create WebGPU surface".to_string());
        }
        eprintln!("[RUST] Surface created: {:?}", surface);

        // Request adapter with condvar (no polling loop)
        eprintln!("[RUST] Requesting adapter...");
        let adapter = request_adapter_sync(instance, surface)?;
        eprintln!("[RUST] Adapter obtained: {:?}", adapter);

        if adapter.is_null() {
            ffi::wgpuSurfaceRelease(surface);
            ffi::wgpuInstanceRelease(instance);
            return Err("Failed to get WebGPU adapter".to_string());
        }

        // Request device with condvar (no polling loop)
        eprintln!("[RUST] Requesting device...");
        let device = request_device_sync(adapter)?;
        eprintln!("[RUST] Device obtained: {:?}", device);

        if device.is_null() {
            ffi::wgpuAdapterRelease(adapter);
            ffi::wgpuSurfaceRelease(surface);
            ffi::wgpuInstanceRelease(instance);
            return Err("Failed to get WebGPU device".to_string());
        }

        // Get queue
        let queue = ffi::wgpuDeviceGetQueue(device);
        if queue.is_null() {
            ffi::wgpuDeviceRelease(device);
            ffi::wgpuAdapterRelease(adapter);
            ffi::wgpuSurfaceRelease(surface);
            ffi::wgpuInstanceRelease(instance);
            return Err("Failed to get WebGPU queue".to_string());
        }
        eprintln!("[RUST] Queue obtained: {:?}", queue);

        // Note: Surface configuration is handled by ThorVG's wg_engine
        // We just provide the unconfigured surface, device, and instance
        eprintln!("[RUST] WebGPU context ready (surface will be configured by ThorVG)");

        Ok(WgpuContext {
            instance,
            adapter,
            device,
            queue,
            surface,
        })
    }

    /// Get pointers as u64 for passing to dotlottie_set_wg_target
    pub fn as_pointers(&self) -> (u64, u64, u64) {
        (
            self.device as u64,
            self.instance as u64,
            self.surface as u64,
        )
    }

    /// Present the surface to display rendered content
    /// MUST be called after rendering to actually show the frame on screen
    pub fn present(&self) {
        unsafe {
            // Use a compiler fence to prevent reordering of this call
            // This ensures present is called after all rendering is complete
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
            ffi::wgpuSurfacePresent(self.surface);
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
        }
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
