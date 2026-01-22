use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=src/dotlottie_player.udl");

    if cfg!(feature = "uniffi") {
        uniffi::generate_scaffolding("src/dotlottie_player.udl").unwrap();
    }

    if cfg!(feature = "ffi") {
        // Execute cbindgen
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let config_path = PathBuf::from(&crate_dir).join("cbindgen.toml");
        let config = cbindgen::Config::from_file(config_path).unwrap();

        cbindgen::Builder::new()
            .with_crate(crate_dir)
            .with_config(config)
            .generate()
            .expect("Unable to generate bindings")
            .write_to_file("bindings.h");
    }

    // Link wgpu_native when tvg-wg feature is enabled
    // Note: This affects all targets in the package, including binaries
    // The linker will only pull in symbols that are actually used
    if cfg!(feature = "tvg-wg") {
        let target = env::var("TARGET").unwrap_or_default();

        // Only apply for Apple platforms
        if !target.contains("apple") {
            return;
        }

        // Map target triple to wgpu library directory
        // Only support targets with matching wgpu-native binaries
        let wgpu_arch_dir = match target.as_str() {
            "aarch64-apple-darwin" => "wgpu-macos-aarch64-release",
            "x86_64-apple-darwin" => "wgpu-macos-x86_64-release",
            "aarch64-apple-ios" => "wgpu-ios-aarch64-release",
            "aarch64-apple-ios-sim" => "wgpu-ios-aarch64-simulator-release",
            "x86_64-apple-ios" => "wgpu-ios-x86_64-simulator-release",
            // Mac Catalyst, tvOS, visionOS need their own wgpu builds
            // Skipping wgpu for these targets until specific builds are available
            _ => {
                eprintln!("cargo:warning=[dotlottie-ffi] Target {} doesn't have matching wgpu-native binaries", target);
                eprintln!("cargo:warning=[dotlottie-ffi] Skipping wgpu linking. WebGPU features will not be available for this target");
                eprintln!("cargo:warning=[dotlottie-ffi] Supported targets: macOS (x86_64, aarch64), iOS device (aarch64), iOS simulator (x86_64, aarch64)");
                return;
            }
        };

        // Enable custom cfg for Rust code to conditionally compile WgpuContext
        println!("cargo:rustc-cfg=has_wgpu_binaries");
        eprintln!("cargo:warning=[dotlottie-ffi] Enabled has_wgpu_binaries cfg for target {}", target);

        // Path to wgpu-native in ../dotlottie-rs/deps/wgpu
        let wgpu_base_path = PathBuf::from("../dotlottie-rs/deps/wgpu").join(wgpu_arch_dir);
        let wgpu_lib_path = wgpu_base_path.join("lib");
        let static_lib = wgpu_lib_path.join("libwgpu_native.a");

        if static_lib.exists() {
            let abs_lib_path = wgpu_lib_path
                .canonicalize()
                .expect("Failed to canonicalize wgpu lib path");

            println!("cargo:rustc-link-search=native={}", abs_lib_path.display());
            println!("cargo:rustc-link-lib=static=wgpu_native");

            eprintln!(
                "cargo:warning=[dotlottie-ffi] Linking wgpu_native from: {}",
                static_lib.display()
            );

            // Link required system frameworks for wgpu-native on Apple platforms
            println!("cargo:rustc-link-lib=framework=Metal");
            println!("cargo:rustc-link-lib=framework=QuartzCore");
            println!("cargo:rustc-link-lib=framework=Foundation");
            if target.contains("-darwin") && !target.contains("macabi") {
                // macOS (but not Mac Catalyst)
                println!("cargo:rustc-link-lib=framework=AppKit");
            } else if target.contains("ios") || target.contains("tvos") || target.contains("visionos") || target.contains("macabi") {
                // iOS, tvOS, visionOS, or Mac Catalyst
                println!("cargo:rustc-link-lib=framework=UIKit");
            }
        } else {
            eprintln!(
                "cargo:warning=[dotlottie-ffi] WGPU library not found at: {}\n\
                 This will cause linking errors. Make sure wgpu-native is in ../dotlottie-rs/deps/wgpu/{}/lib/",
                static_lib.display(),
                wgpu_arch_dir
            );
        }
    }
}
