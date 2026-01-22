// hint: this is a workaround as the generated code from uniffi has empty lines after doc comments
#![allow(clippy::empty_line_after_doc_comments)]

pub use dotlottie_rs::*;

pub use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;

pub fn create_default_layout() -> Layout {
    Layout::default()
}

pub fn create_default_open_url_policy() -> OpenUrlPolicy {
    OpenUrlPolicy::default()
}

pub fn create_default_config() -> Config {
    Config::default()
}

pub fn transform_theme_to_lottie_slots(theme_data: &str, animation_id: &str) -> String {
    dotlottie_rs::transform_theme_to_lottie_slots(theme_data, animation_id)
}

pub fn register_font(font_name: &str, font_data: &[u8]) -> bool {
    dotlottie_rs::register_font(font_name, font_data)
}

// Error type for UniFFI
#[derive(Debug, thiserror::Error)]
pub enum DotLottieError {
    #[error("WebGpuContextCreationFailed(message: {message:?})")]
    WebGpuContextCreationFailed { message: String },
}

// WebGPU Context wrapper for UniFFI
// Real implementation when wgpu binaries are available for the target
#[cfg(has_wgpu_binaries)]
pub struct WgpuContext {
    inner: dotlottie_rs::WgpuContext,
}

#[cfg(has_wgpu_binaries)]
impl WgpuContext {
    pub fn from_metal_layer(metal_layer_ptr: u64) -> Result<Self, DotLottieError> {
        println!(
            "🦀 [FFI] from_metal_layer called with ptr: 0x{:x}",
            metal_layer_ptr
        );

        let inner = unsafe {
            dotlottie_rs::WgpuContext::new_from_metal_layer(
                metal_layer_ptr as *mut std::ffi::c_void,
            )
        }
        .map_err(|err_msg| {
            eprintln!("🦀 [FFI] ❌ WebGPU creation failed: {}", err_msg);
            DotLottieError::WebGpuContextCreationFailed { message: err_msg }
        })?;

        println!("🦀 [FFI] ✓ WebGPU context created successfully");
        Ok(WgpuContext { inner })
    }

    pub fn get_pointers(&self) -> Vec<u64> {
        let (device, instance, surface) = self.inner.as_pointers();
        vec![device, instance, surface]
    }
}

// Stub implementation when wgpu binaries are not available for the target
#[cfg(not(has_wgpu_binaries))]
pub struct WgpuContext {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(not(has_wgpu_binaries))]
impl WgpuContext {
    pub fn from_metal_layer(_metal_layer_ptr: u64) -> Result<Self, DotLottieError> {
        // WebGPU context is only available on supported targets with wgpu binaries
        Err(DotLottieError::WebGpuContextCreationFailed {
            message: "WebGPU is not available for this target (no wgpu binaries)".to_string()
        })
    }

    pub fn get_pointers(&self) -> Vec<u64> {
        // Return dummy values since this should never be called
        vec![0, 0, 0]
    }
}
