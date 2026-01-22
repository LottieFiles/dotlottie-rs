mod thorvg {
    use std::env;
    use std::fs::{self, OpenOptions};
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
        eprintln!("cargo:warning=tvg-wg enabled: {}", cfg!(feature = "tvg-wg"));
        eprintln!("cargo:warning=tvg-gl enabled: {}", cfg!(feature = "tvg-gl"));
        eprintln!("cargo:warning=tvg-sw enabled: {}", cfg!(feature = "tvg-sw"));
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

        // Check if tvg-wg should be enabled for this target
        // Only enable if feature is on AND we have wgpu binaries for the target OR it's Emscripten
        let tvg_wg_requested = cfg!(feature = "tvg-wg");

        let tvg_wg_enabled = if tvg_wg_requested {
            let target = env::var("TARGET").unwrap_or_default();

            // Enable for Emscripten (uses browser WebGPU, no wgpu-native needed)
            if target == "wasm32-unknown-emscripten" {
                eprintln!("cargo:warning=Enabling WebGPU for Emscripten target");
                true
            } else {
                // Native targets need wgpu-native binaries
                matches!(
                    target.as_str(),
                    "aarch64-apple-darwin"
                        | "x86_64-apple-darwin"
                        | "aarch64-apple-ios"
                        | "aarch64-apple-ios-sim"
                        | "x86_64-apple-ios"
                )
            }
        } else {
            false
        };

        if tvg_wg_enabled {
            writeln!(thorvg_config_h, "#define THORVG_WG_RASTER_SUPPORT")?;
            src.push("deps/thorvg/src/renderer/wg_engine");

            let target = env::var("TARGET").unwrap_or_default();

            if target != "wasm32-unknown-emscripten" {
                println!("cargo:rustc-cfg=has_wgpu_binaries");
            } else {
                eprintln!("cargo:warning=Using Emscripten WebGPU (browser-based, no wgpu-native)");
            }
        } else if tvg_wg_requested {
            eprintln!(
                "cargo:warning=tvg-wg requested but target {} doesn't have wgpu binaries",
                target_triple
            );
            eprintln!("cargo:warning=Building without WebGPU support");
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
        eprintln!(
            "cargo:warning=config.h path: {}",
            out_dir.join("config.h").display()
        );
        eprintln!(
            "cargo:warning=config.h exists: {}",
            out_dir.join("config.h").exists()
        );

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

        // Add wgpu headers for type definitions on Emscripten (no linking)
        if tvg_wg_enabled && target_triple == "wasm32-unknown-emscripten" {
            // Use any wgpu include dir - they all have the same webgpu.h with type definitions
            let wgpu_include = PathBuf::from("deps/wgpu/wgpu-macos-aarch64-release/include");
            if wgpu_include.exists() {
                cc_build.include(&wgpu_include);
                eprintln!(
                    "cargo:warning=Using wgpu-native headers for type definitions (Emscripten)"
                );
            }
        }

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

        // Add wgpu_native include path and linking if tvg-wg is enabled (native targets only)
        if tvg_wg_enabled {
            let target = env::var("TARGET").unwrap_or_default();

            // Emscripten uses browser WebGPU - skip wgpu-native linking entirely
            if target != "wasm32-unknown-emscripten" {
                // Map target triple to wgpu library directory
                // Only support targets with matching wgpu-native binaries
                let wgpu_arch_dir = match target.as_str() {
                    "aarch64-apple-darwin" => Some("wgpu-macos-aarch64-release"),
                    "x86_64-apple-darwin" => Some("wgpu-macos-x86_64-release"),
                    "aarch64-apple-ios" => Some("wgpu-ios-aarch64-release"),
                    "aarch64-apple-ios-sim" => Some("wgpu-ios-aarch64-simulator-release"),
                    "x86_64-apple-ios" => Some("wgpu-ios-x86_64-simulator-release"),
                    // Mac Catalyst, tvOS, visionOS need their own wgpu builds
                    // Skipping wgpu for these targets until specific builds are available
                    _ => {
                        eprintln!(
                            "cargo:warning=Target {} doesn't have matching wgpu-native binaries",
                            target
                        );
                        eprintln!("cargo:warning=Skipping wgpu linking. WebGPU features will not be available for this target");
                        eprintln!("cargo:warning=Supported targets: macOS (x86_64, aarch64), iOS device (aarch64), iOS simulator (x86_64, aarch64)");
                        None
                    }
                };

                if let Some(wgpu_arch_dir) = wgpu_arch_dir {
                    // Use architecture-specific path to wgpu-native in deps/wgpu
                    let wgpu_base_path = PathBuf::from("deps/wgpu").join(wgpu_arch_dir);
                    let wgpu_lib_path = wgpu_base_path.join("lib");
                    let wgpu_include_path = wgpu_base_path.join("include");

                    // Add include path if it exists
                    if wgpu_include_path.exists() {
                        cc_build.include(&wgpu_include_path);
                        eprintln!(
                            "cargo:warning=Added WGPU include path: {}",
                            wgpu_include_path.display()
                        );
                    }

                    // Try static library first, then dynamic
                    let static_lib = wgpu_lib_path.join("libwgpu_native.a");
                    let dynamic_lib = wgpu_lib_path.join("libwgpu_native.dylib");

                    if static_lib.exists() {
                        // Link static library
                        // We need to tell cargo where to find it and to link it statically
                        let abs_lib_path = wgpu_lib_path
                            .canonicalize()
                            .expect("Failed to canonicalize wgpu lib path");

                        println!("cargo:rustc-link-search=native={}", abs_lib_path.display());
                        println!("cargo:rustc-link-lib=static=wgpu_native");

                        eprintln!(
                            "cargo:warning=Linking wgpu_native statically from: {}",
                            static_lib.display()
                        );

                        // Link required system frameworks for wgpu-native
                        // Note: Must check TARGET env var, not cfg!, for cross-compilation
                        println!("cargo:rustc-link-lib=framework=Metal");
                        println!("cargo:rustc-link-lib=framework=QuartzCore");
                        println!("cargo:rustc-link-lib=framework=Foundation");
                        if target.contains("-darwin") && !target.contains("macabi") {
                            // macOS (but not Mac Catalyst)
                            println!("cargo:rustc-link-lib=framework=AppKit");
                        } else if target.contains("ios")
                            || target.contains("tvos")
                            || target.contains("visionos")
                            || target.contains("macabi")
                        {
                            // iOS, tvOS, visionOS, or Mac Catalyst
                            println!("cargo:rustc-link-lib=framework=UIKit");
                        }
                    } else if dynamic_lib.exists() {
                        // Link dynamic library and set rpath
                        let abs_lib_path = wgpu_lib_path
                            .canonicalize()
                            .expect("Failed to canonicalize wgpu lib path");
                        println!("cargo:rustc-link-search=native={}", abs_lib_path.display());
                        println!("cargo:rustc-link-lib=dylib=wgpu_native");
                        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", abs_lib_path.display());
                        eprintln!(
                            "cargo:warning=Linking wgpu_native dynamically from: {}",
                            abs_lib_path.display()
                        );

                        // Also link required system frameworks for wgpu-native
                        // Note: Must check TARGET env var, not cfg!, for cross-compilation
                        println!("cargo:rustc-link-lib=framework=Metal");
                        println!("cargo:rustc-link-lib=framework=QuartzCore");
                        println!("cargo:rustc-link-lib=framework=Foundation");
                        if target.contains("-darwin") && !target.contains("macabi") {
                            // macOS (but not Mac Catalyst)
                            println!("cargo:rustc-link-lib=framework=AppKit");
                        } else if target.contains("ios")
                            || target.contains("tvos")
                            || target.contains("visionos")
                            || target.contains("macabi")
                        {
                            // iOS, tvOS, visionOS, or Mac Catalyst
                            println!("cargo:rustc-link-lib=framework=UIKit");
                        }
                    } else {
                        eprintln!(
                            "cargo:warning=WGPU library not found at: {} or {}\n\
                             Make sure to place wgpu-native binaries in deps/wgpu/{}/lib/",
                            static_lib.display(),
                            dynamic_lib.display(),
                            wgpu_arch_dir
                        );
                    }
                } // end if let Some(wgpu_arch_dir)
            } // end if target != "wasm32-unknown-emscripten"
        } // end if tvg_wg_enabled

        cc_build.compile("thorvg");

        let bindings = bindgen::Builder::default()
            .header("deps/thorvg/src/bindings/capi/thorvg_capi.h")
            .generate()
            .expect("Failed to generate bindings");

        bindings.write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))?;

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    if cfg!(feature = "tvg") {
        thorvg::build()?;
    }

    Ok(())
}
