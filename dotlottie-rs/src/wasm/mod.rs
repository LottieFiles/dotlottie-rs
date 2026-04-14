//! WASM-specific modules for `wasm32-unknown-unknown`.
//!
//! This module is auto-compiled when targeting `wasm32` with ThorVG enabled
//! (excluding emscripten, which has its own C runtime and GL/GPU loaders).
//! It provides the glue that ThorVG needs to run in a browser environment
//! where there is no system C runtime, no native OpenGL loader, and no
//! wgpu-native library.

/// cbindgen:ignore
///
/// Minimal libc/C++ runtime stubs for `wasm32-unknown-unknown`.
///
/// Unlike `wasm32-unknown-emscripten` or `wasm32-wasi`, the `unknown-unknown`
/// target has no system C runtime. ThorVG's C++ sources reference libc symbols
/// (`malloc`, `strcmp`, `snprintf`, …) that must be provided by us.
mod stubs;

/// cbindgen:ignore
///
/// WebGL2 FFI stubs that bridge ThorVG's OpenGL calls to `web_sys::WebGl2RenderingContext`.
///
/// ThorVG's GL engine emits standard OpenGL/GLES function calls (`glBindTexture`,
/// `glCreateShader`, …). On `wasm32-unknown-unknown` there is no native GL loader —
/// these stubs forward each call to the browser's WebGL2 API via `web-sys`.
#[cfg(feature = "tvg-gl")]
mod webgl_stubs;

/// cbindgen:ignore
///
/// WebGPU FFI stubs that bridge ThorVG's wgpu-native calls to the browser's WebGPU API.
///
/// ThorVG's WG engine calls wgpu-native C functions (`wgpuDeviceCreateBuffer`,
/// `wgpuRenderPassEncoderDraw`, …). On `wasm32-unknown-unknown` wgpu-native is not
/// available — these stubs forward each call to the browser's WebGPU API via `web-sys`.
#[cfg(feature = "tvg-wg")]
mod webgpu_stubs;

#[cfg(feature = "wasm-bindgen-api")]
pub mod wasm_bindgen_api;
