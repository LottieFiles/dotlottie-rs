// Generate wgpu bindings using bindgen
// Run with: cargo run --bin wgpu_bindings_gen

use std::env;
use std::path::PathBuf;

fn main() {
    let wgpu_header = "deps/wgpu/wgpu-macos-x86_64-release/include/webgpu/webgpu.h";

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header(wgpu_header)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Only generate bindings for functions we need
        .allowlist_function("wgpuCreateInstance")
        .allowlist_function("wgpuInstanceCreateSurface")
        .allowlist_function("wgpuInstanceRequestAdapter")
        .allowlist_function("wgpuInstanceProcessEvents")
        .allowlist_function("wgpuAdapterRequestDevice")
        .allowlist_function("wgpuDeviceGetQueue")
        .allowlist_function("wgpuInstanceRelease")
        .allowlist_function("wgpuAdapterRelease")
        .allowlist_function("wgpuDeviceRelease")
        .allowlist_function("wgpuQueueRelease")
        .allowlist_function("wgpuSurfaceRelease")
        // Allow list types we need
        .allowlist_type("WGPUInstance.*")
        .allowlist_type("WGPUAdapter.*")
        .allowlist_type("WGPUDevice.*")
        .allowlist_type("WGPUQueue.*")
        .allowlist_type("WGPUSurface.*")
        .allowlist_type("WGPUFuture.*")
        .allowlist_type("WGPURequestAdapter.*")
        .allowlist_type("WGPURequestDevice.*")
        .allowlist_type("WGPUStringView.*")
        .allowlist_type("WGPUChainedStruct.*")
        .allowlist_type("WGPUSurfaceSource.*")
        .allowlist_type("WGPUCallbackMode.*")
        .allowlist_type("WGPUPowerPreference.*")
        .allowlist_type("WGPUBackendType.*")
        .allowlist_type("WGPUSType.*")
        .allowlist_var("WGPU_.*")
        .generate()
        .expect("Unable to generate bindings");

    // Write bindings to stdout
    println!("{}", bindings.to_string());
}
