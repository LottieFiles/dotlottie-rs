#![cfg(all(feature = "tvg-wg", any(target_os = "macos", target_os = "ios")))]

//! Apple-specific ThorVG wgpu fixups.

use std::ffi::c_void;

/// Intercepts ThorVG's wgpuSurfaceConfigure calls (redirected via the
/// tvg_wgpu_surface_fixup.h force-include in build.rs).
///
/// ThorVG hardcodes WGPUPresentMode_Immediate for all Apple targets, but iOS
/// Metal surfaces only support Fifo.  This shim corrects the present mode on
/// iOS before forwarding to the real wgpu-native implementation.
#[no_mangle]
unsafe extern "C" fn _tvg_wgpu_surface_configure_fixup(
    surface: ffi::WGPUSurface,
    config: *const ffi::WGPUSurfaceConfiguration,
) {
    // `mut` is only used on iOS, where the present mode is patched below.
    #[cfg_attr(not(target_os = "ios"), allow(unused_mut))]
    let mut cfg = *config;
    #[cfg(target_os = "ios")]
    {
        cfg.presentMode = ffi::WGPUPresentMode_WGPUPresentMode_Fifo;
    }
    ffi::wgpuSurfaceConfigure(surface, &cfg);
}

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
