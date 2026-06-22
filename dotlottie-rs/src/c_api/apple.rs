//! Apple WebGPU surface-configure shim.
//!
//! Intercepts ThorVG's `wgpuSurfaceConfigure` calls, which are redirected here
//! by the `tvg_wgpu_surface_fixup.h` force-include generated in `build.rs`.
//!
//! ThorVG hardcodes `WGPUPresentMode_Immediate` for all Apple targets, but iOS
//! Metal surfaces only support `Fifo`. This shim corrects the present mode on
//! iOS before forwarding to the real wgpu-native implementation.

/// Drop-in replacement for `wgpuSurfaceConfigure` (see module docs).
#[no_mangle]
unsafe extern "C" fn _tvg_wgpu_surface_configure_fixup(
    surface: ffi::WGPUSurface,
    config: *const ffi::WGPUSurfaceConfiguration,
) {
    #[cfg_attr(not(target_os = "ios"), allow(unused_mut))]
    let mut cfg = *config;
    #[cfg(target_os = "ios")]
    {
        cfg.presentMode = ffi::WGPUPresentMode_WGPUPresentMode_Fifo;
    }
    ffi::wgpuSurfaceConfigure(surface, &cfg);
}

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
