#![cfg(all(feature = "tvg-wg", any(target_os = "macos"), wgpu_native_linked))]

/// Create WebGPU context from Metal layer (macOS/iOS only)
///
/// # Arguments
/// * `metal_layer` - Pointer to CAMetalLayer from Swift
///
/// # Returns
/// * Opaque pointer to WgpuContext, or NULL on failure
///
/// # Safety
/// The metal_layer pointer must be valid and point to a CAMetalLayer object
#[no_mangle]
pub unsafe extern "C" fn dotlottie_create_wgpu_context_from_metal_layer(
    metal_layer: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
    match WgpuContext::from_metal_layer(metal_layer) {
        Ok(context) => Box::into_raw(Box::new(context)) as *mut std::ffi::c_void,
        Err(e) => {
            eprintln!("Failed to create WebGPU context: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get WebGPU pointers from context (device, instance, surface)
///
/// # Arguments
/// * `context` - Opaque pointer from dotlottie_create_wgpu_context_from_metal_layer
/// * `out_device` - Output pointer for device
/// * `out_instance` - Output pointer for instance
/// * `out_surface` - Output pointer for surface
///
/// # Safety
/// context must be a valid pointer from dotlottie_create_wgpu_context_from_metal_layer
#[no_mangle]
pub unsafe extern "C" fn dotlottie_wgpu_context_get_pointers(
    context: *const std::ffi::c_void,
    out_device: *mut u64,
    out_instance: *mut u64,
    out_surface: *mut u64,
) {
    if context.is_null() || out_device.is_null() || out_instance.is_null() || out_surface.is_null()
    {
        return;
    }

    let ctx = &*(context as *const WgpuContext);
    let (device, instance, surface) = ctx.as_pointers();
    *out_device = device;
    *out_instance = instance;
    *out_surface = surface;
}

/// Free WebGPU context
///
/// # Arguments
/// * `context` - Opaque pointer from dotlottie_create_wgpu_context_from_metal_layer
///
/// # Safety
/// context must be a valid pointer and will be invalid after this call
#[no_mangle]
pub unsafe extern "C" fn dotlottie_free_wgpu_context(context: *mut std::ffi::c_void) {
    if !context.is_null() {
        let _ = Box::from_raw(context as *mut WgpuContext);
    }
}

/// Present WebGPU surface to display rendered frame
///
/// CRITICAL: Must be called after rendering to show the frame on screen.
/// Without this call, rendering happens off-screen but never displays.
///
/// # Arguments
/// * `context` - Opaque pointer from dotlottie_create_wgpu_context_from_metal_layer
///
/// # Safety
/// context must be a valid pointer from dotlottie_create_wgpu_context_from_metal_layer
#[cfg(all(feature = "tvg-wg", any(target_os = "macos"), wgpu_native_linked))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_wgpu_context_present(context: *const std::ffi::c_void) {
    if context.is_null() {
        return;
    }

    let ctx = &*(context as *const WgpuContext);
    ctx.present();
}

use std::ffi::c_void;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

// WebGPU FFI bindings auto-generated from webgpu.h by bindgen
#[allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    improper_ctypes
)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/wgpu_bindings.rs"));
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

            if status == ffi::WGPURequestAdapterStatus_WGPURequestAdapterStatus_Success {
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
        nextInChain: std::ptr::null(),
        featureLevel: ffi::WGPUFeatureLevel_WGPUFeatureLevel_Core,
        powerPreference: ffi::WGPUPowerPreference_WGPUPowerPreference_HighPerformance,
        forceFallbackAdapter: 0,
        backendType: ffi::WGPUBackendType_WGPUBackendType_Metal,
        compatibleSurface: surface,
    };

    let callback_info = ffi::WGPURequestAdapterCallbackInfo {
        nextInChain: std::ptr::null(),
        mode: ffi::WGPUCallbackMode_WGPUCallbackMode_AllowSpontaneous,
        callback: Some(adapter_callback),
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
        _message: ffi::WGPUStringView,
        _userdata1: *mut c_void,
        _userdata2: *mut c_void,
    ) {
        // Use thread_local because wgpu-native doesn't pass userdata1 correctly
        DEVICE_CALLBACK_RESULT.with(|r| {
            if let Some(result) = r.borrow().as_ref() {
                let (lock, cvar) = &**result;
                let mut data = lock.lock().unwrap();

                if status == ffi::WGPURequestDeviceStatus_WGPURequestDeviceStatus_Success {
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
        nextInChain: std::ptr::null(),
        label: ffi::WGPUStringView {
            data: std::ptr::null(),
            length: 0,
        },
        requiredFeatureCount: 0,
        requiredFeatures: std::ptr::null(),
        requiredLimits: std::ptr::null(),
        defaultQueue: ffi::WGPUQueueDescriptor {
            nextInChain: std::ptr::null(),
            label: ffi::WGPUStringView {
                data: std::ptr::null(),
                length: 0,
            },
        },
        deviceLostCallbackInfo: ffi::WGPUDeviceLostCallbackInfo {
            nextInChain: std::ptr::null(),
            mode: ffi::WGPUCallbackMode_WGPUCallbackMode_AllowSpontaneous,
            callback: None,
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        },
        uncapturedErrorCallbackInfo: ffi::WGPUUncapturedErrorCallbackInfo {
            nextInChain: std::ptr::null(),
            callback: None,
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        },
    };

    let device_callback_info = ffi::WGPURequestDeviceCallbackInfo {
        nextInChain: std::ptr::null(),
        mode: ffi::WGPUCallbackMode_WGPUCallbackMode_AllowSpontaneous,
        callback: Some(device_callback),
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
            nextInChain: std::ptr::null(),
            features: ffi::WGPUInstanceCapabilities {
                nextInChain: std::ptr::null_mut(),
                timedWaitAnyEnable: 0,
                timedWaitAnyMaxCount: 0,
            },
        };

        let instance = ffi::wgpuCreateInstance(&instance_desc);
        if instance.is_null() {
            return Err("Failed to create WebGPU instance".to_string());
        }

        // Create surface from Metal layer
        let metal_source = ffi::WGPUSurfaceSourceMetalLayer {
            chain: ffi::WGPUChainedStruct {
                next: std::ptr::null(),
                sType: 0x00000004, // WGPUSType_SurfaceSourceMetalLayer
            },
            layer: metal_layer,
        };

        let surface_desc = ffi::WGPUSurfaceDescriptor {
            nextInChain: &metal_source.chain as *const ffi::WGPUChainedStruct,
            label: ffi::WGPUStringView {
                data: std::ptr::null(),
                length: 0,
            },
        };

        let surface = ffi::wgpuInstanceCreateSurface(instance, &surface_desc);
        if surface.is_null() {
            ffi::wgpuInstanceRelease(instance);
            return Err("Failed to create WebGPU surface".to_string());
        }

        // Request adapter with condvar (no polling loop)
        let adapter = request_adapter_sync(instance, surface)?;

        if adapter.is_null() {
            ffi::wgpuSurfaceRelease(surface);
            ffi::wgpuInstanceRelease(instance);
            return Err("Failed to get WebGPU adapter".to_string());
        }

        // Request device with condvar (no polling loop)
        let device = request_device_sync(adapter)?;

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

        // Note: Surface configuration is handled by ThorVG's wg_engine
        // We just provide the unconfigured surface, device, and instance

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
            let status = ffi::wgpuSurfacePresent(self.surface);
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);

            // Check if present succeeded
            if status != ffi::WGPUStatus_WGPUStatus_Success {
                eprintln!("Warning: wgpuSurfacePresent failed with status: {}", status);
            }
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
