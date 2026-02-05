mod thorvg {
    use std::env;
    use std::fs::{self, create_dir_all, OpenOptions};
    use std::io::Write;
    use std::path::PathBuf;

    /// Returns the C++ standard library that needs to be linked for the current platform.
    ///
    /// This is necessary because when we compile C++ code (either from source or use
    /// prebuilt libraries), we need to link against the platform's C++ standard library
    /// to resolve C++ symbols like std::string, std::mutex, etc.
    fn get_cpp_standard_library() -> Vec<String> {
        let host_target =
            std::env::var("HOST").expect("HOST environment variable should be set by Cargo");

        if is_apple_platform(&host_target) {
            // macOS and iOS use libc++ as the C++ standard library
            vec![String::from("c++")]
        } else if is_unix_platform() {
            // Linux and other Unix systems typically use libstdc++
            vec![String::from("stdc++")]
        } else {
            // Windows and other platforms - C++ stdlib is handled differently
            // (usually linked automatically by the compiler/linker)
            vec![]
        }
    }

    /// Check if we're building for an Apple platform (macOS, iOS, etc.)
    fn is_apple_platform(host_target: &str) -> bool {
        host_target.contains("apple")
    }

    /// Check if we're building for a Unix-like platform
    fn is_unix_platform() -> bool {
        std::env::var("CARGO_CFG_UNIX").is_ok()
    }

    fn collect_files(dir: &str) -> Vec<String> {
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|e| e == "cpp") {
                    files.push(path.to_string_lossy().into_owned());
                }
            }
        }

        files
    }

    pub fn build() -> std::io::Result<()> {
        let target_triple = env::var("TARGET").unwrap_or_default();

        get_cpp_standard_library()
            .iter()
            .for_each(|lib| println!("cargo:rustc-link-lib=dylib={lib}"));

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

        let mut src = vec![
            "deps/thorvg/inc",
            "deps/thorvg/src/common",
            "deps/thorvg/src/bindings/capi",
            "deps/thorvg/src/loaders/lottie",
            "deps/thorvg/src/loaders/raw",
            "deps/thorvg/src/renderer",
        ];

        // thorvg config.h
        let mut thorvg_config_h = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(out_dir.join("config.h"))?;

        writeln!(thorvg_config_h, "#define THORVG_VERSION_STRING \"1.0.0\"")?;
        writeln!(thorvg_config_h, "#define THORVG_LOTTIE_LOADER_SUPPORT")?;
        writeln!(thorvg_config_h, "#define TVG_STATIC")?;
        writeln!(thorvg_config_h, "#define WIN32_LEAN_AND_MEAN")?;

        if cfg!(feature = "tvg-log") {
            writeln!(thorvg_config_h, "#define THORVG_LOG_ENABLED")?;
        }

        if cfg!(feature = "tvg-threads") {
            writeln!(thorvg_config_h, "#define THORVG_THREAD_SUPPORT")?;
        }

        let tvg_sw_enabled = cfg!(feature = "tvg-sw");
        if tvg_sw_enabled {
            writeln!(thorvg_config_h, "#define THORVG_SW_RASTER_SUPPORT")?;
            src.push("deps/thorvg/src/renderer/sw_engine");
        }

        if cfg!(feature = "tvg-gl") {
            writeln!(thorvg_config_h, "#define THORVG_GL_RASTER_SUPPORT")?;
            src.push("deps/thorvg/src/renderer/gl_engine");

            if target_triple == "wasm32-unknown-emscripten" {
                writeln!(thorvg_config_h, "#define THORVG_GL_TARGET_GLES 1")?;
            }
        }

        let tvg_wg_requested = cfg!(feature = "tvg-wg");

        let tvg_wg_enabled = if tvg_wg_requested {
            let target = env::var("TARGET").unwrap_or_default();

            // For Emscripten: ENABLE ThorVG's wg_engine with Dawn
            if target == "wasm32-unknown-emscripten" {
                eprintln!(
                    "cargo:warning=tvg-wg for WASM: Enabling ThorVG WebGPU renderer with Dawn"
                );
                true // Enable ThorVG's wg_engine for WASM
            } else {
                false
            }
        } else {
            false
        };

        if tvg_wg_enabled {
            writeln!(thorvg_config_h, "#define THORVG_WG_RASTER_SUPPORT")?;
            src.push("deps/thorvg/src/renderer/wg_engine");

            println!("cargo:rustc-cfg=has_wgpu_binaries");
        } else if tvg_wg_requested {
            let target = env::var("TARGET").unwrap_or_default();
            if target != "wasm32-unknown-emscripten" {
                eprintln!(
                    "cargo:warning=tvg-wg requested but target {} doesn't have wgpu binaries",
                    target_triple
                );
                eprintln!("cargo:warning=Building without ThorVG WebGPU renderer");
            }
        }

        if cfg!(feature = "tvg-jpg") {
            writeln!(thorvg_config_h, "#define THORVG_JPG_LOADER_SUPPORT")?;
            src.push("deps/thorvg/src/loaders/jpg");
        }

        if cfg!(feature = "tvg-png") {
            writeln!(thorvg_config_h, "#define THORVG_PNG_LOADER_SUPPORT")?;
            src.push("deps/thorvg/src/loaders/png");
        }

        if cfg!(feature = "tvg-webp") {
            writeln!(thorvg_config_h, "#define THORVG_WEBP_LOADER_SUPPORT")?;
            src.push("deps/thorvg/src/loaders/webp");
            src.push("deps/thorvg/src/loaders/webp/dec");
            src.push("deps/thorvg/src/loaders/webp/dsp");
            src.push("deps/thorvg/src/loaders/webp/utils");
            src.push("deps/thorvg/src/loaders/webp/webp");
        }

        if cfg!(feature = "tvg-ttf") {
            writeln!(thorvg_config_h, "#define THORVG_TTF_LOADER_SUPPORT")?;
            src.push("deps/thorvg/src/loaders/ttf");
        }

        if cfg!(feature = "tvg-lottie-expressions") {
            writeln!(thorvg_config_h, "#define THORVG_LOTTIE_EXPRESSIONS_SUPPORT")?;
            src.push("deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/api");
            src.push("deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/ecma/base");
            src.push("deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/ecma/builtin-objects");
            src.push(
                "deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/ecma/builtin-objects/typedarray",
            );
            src.push("deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/ecma/operations");
            src.push("deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/include");
            src.push("deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/jcontext");
            src.push("deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/jmem");
            src.push("deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/jrt");
            src.push("deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/lit");
            src.push("deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/parser/js");
            src.push("deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/parser/regexp");
            src.push("deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/vm");
        }

        // ThorVG SIMD feature (only when tvg-sw AND tvg-simd are enabled)
        let tvg_simd_enabled = cfg!(feature = "tvg-simd");
        let target_triple = env::var("TARGET").unwrap_or_default();

        let mut simd_flags: Vec<&str> = Vec::new();
        if tvg_sw_enabled && tvg_simd_enabled {
            if target_triple.contains("x86_64")
                || target_triple.contains("i686")
                || target_triple.contains("i586")
            {
                // x86/x86_64 → AVX
                writeln!(thorvg_config_h, "#define THORVG_AVX_VECTOR_SUPPORT")?;
                simd_flags.push("-mavx");
            } else if target_triple.contains("aarch64") {
                // aarch64 → NEON baseline (no extra flag needed)
                writeln!(thorvg_config_h, "#define THORVG_NEON_VECTOR_SUPPORT")?;
            } else if target_triple.contains("armv7") {
                // armv7 → NEON
                writeln!(thorvg_config_h, "#define THORVG_NEON_VECTOR_SUPPORT")?;
                simd_flags.push("-mfpu=neon");
            } else if target_triple == "wasm32-unknown-emscripten" {
                // Emscripten → use Wasm SIMD
                // https://emscripten.org/docs/porting/simd.html
                writeln!(thorvg_config_h, "#define THORVG_NEON_VECTOR_SUPPORT")?; // maps to Wasm SIMD in ThorVG
                simd_flags.push("-msimd128");
            }
        }

        thorvg_config_h.flush()?;

        let compiler = env::var("CXX").unwrap_or("clang++".to_string());

        let mut cc_build = cc::Build::new();
        cc_build
            .compiler(compiler)
            .std("c++14")
            .cpp(true)
            .include(&out_dir)
            .includes(&src)
            .files(
                src.iter()
                    .flat_map(|dir| collect_files(dir))
                    .collect::<Vec<_>>(),
            )
            .warnings(false);

        // Add WebGPU header include path for WASM builds
        if tvg_wg_enabled && target_triple == "wasm32-unknown-emscripten" {
            // Use absolute path to ensure it's found
            let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let webgpu_include = PathBuf::from(&crate_dir)
                .parent()
                .unwrap()
                .join("deps/modules/emsdk/upstream/emscripten/cache/ports/emdawnwebgpu/emdawnwebgpu_pkg/webgpu/include");
            cc_build.include(&webgpu_include);
            eprintln!(
                "cargo:warning=Adding WebGPU include path: {}",
                webgpu_include.display()
            );
        }

        // Add WebGPU header include path and link wgpu-native for Apple platforms
        // if tvg_wg_enabled {
        //     let target = env::var("TARGET").unwrap_or_default();

        //     if matches!(
        //         target.as_str(),
        //         "aarch64-apple-darwin"
        //             | "x86_64-apple-darwin"
        //             | "aarch64-apple-ios"
        //             | "aarch64-apple-ios-sim"
        //             | "x86_64-apple-ios"
        //     ) {
        //         let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        //         let wgpu_base = PathBuf::from(&crate_dir).join("deps/wgpu");

        //         // Map target to wgpu library directory
        //         let wgpu_arch_dir = match target.as_str() {
        //             "aarch64-apple-darwin" => "wgpu-macos-aarch64-release",
        //             "x86_64-apple-darwin" => "wgpu-macos-x86_64-release",
        //             "aarch64-apple-ios" => "wgpu-ios-aarch64-release",
        //             "aarch64-apple-ios-sim" => "wgpu-ios-aarch64-simulator-release",
        //             "x86_64-apple-ios" => "wgpu-ios-x86_64-simulator-release",
        //             _ => "",
        //         };

        //         if !wgpu_arch_dir.is_empty() {
        //             let wgpu_lib_path = wgpu_base.join(wgpu_arch_dir).join("lib");
        //             let wgpu_include_path = wgpu_base.join(wgpu_arch_dir).join("include");
        //             let static_lib = wgpu_lib_path.join("libwgpu_native.a");

        //             // Add include path for wgpu headers
        //             if wgpu_include_path.exists() {
        //                 cc_build.include(&wgpu_include_path);
        //             }

        //             // Link wgpu-native static library
        //             if static_lib.exists() {
        //                 let abs_lib_path = wgpu_lib_path
        //                     .canonicalize()
        //                     .expect("Failed to canonicalize wgpu lib path");

        //                 println!("cargo:rustc-link-search=native={}", abs_lib_path.display());
        //                 println!("cargo:rustc-link-lib=static=wgpu_native");

        //                 // Link required Apple frameworks
        //                 println!("cargo:rustc-link-lib=framework=Metal");
        //                 println!("cargo:rustc-link-lib=framework=QuartzCore");
        //                 println!("cargo:rustc-link-lib=framework=Foundation");

        //                 if target.contains("-darwin") && !target.contains("macabi") {
        //                     println!("cargo:rustc-link-lib=framework=AppKit");
        //                 } else if target.contains("ios") {
        //                     println!("cargo:rustc-link-lib=framework=UIKit");
        //                 }

        //                 eprintln!(
        //                     "cargo:warning=Linked wgpu-native from {}",
        //                     abs_lib_path.display()
        //                 );

        //                 // Enable cfg flag for conditional compilation
        //                 println!("cargo:rustc-cfg=wgpu_native_linked");e
        //             } else {
        //                 eprintln!(
        //                     "cargo:warning=wgpu-native library not found at {:?}",
        //                     static_lib
        //                 );
        //                 eprintln!("cargo:warning=WebGPU rendering will not be available");
        //             }
        //         }
        //     }
        // }

        for flag in simd_flags {
            cc_build.flag(flag);
        }

        if cfg!(feature = "tvg-threads") && std::env::var("CARGO_CFG_UNIX").is_ok() {
            let target = std::env::var("TARGET").unwrap_or_default();
            if !target.contains("apple") && !target.contains("android") {
                cc_build.flag("-pthread");
                println!("cargo:rustc-link-lib=pthread");
            }
        }

        cc_build.compile("thorvg");

        let bindings = bindgen::Builder::default()
            .header("deps/thorvg/src/bindings/capi/thorvg_capi.h")
            .generate()
            .expect("Failed to generate bindings");

        bindings.write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))?;

        if cfg!(feature = "c_api") {
            let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            create_dir_all(PathBuf::from(&crate_dir).join("build")).unwrap();
            let header_path = PathBuf::from(&crate_dir).join("build/dotlottie_player.h");
            let config_path = PathBuf::from(&crate_dir).join("cbindgen.toml");
            let config = cbindgen::Config::from_file(config_path).unwrap();

            cbindgen::Builder::new()
                .with_crate(crate_dir)
                .with_config(config)
                .generate()
                .expect("Unable to generate bindings")
                .write_to_file(header_path);
        }

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    // Declare custom cfg names for conditional compilation
    println!("cargo::rustc-check-cfg=cfg(wgpu_native_linked)");

    if cfg!(feature = "tvg") {
        thorvg::build()?;
    }

    Ok(())
}
