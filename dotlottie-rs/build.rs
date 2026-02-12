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

            // For Emscripten: ENABLE ThorVG's wg_engine with newer Dawn
            // The user has bumped Emscripten/Dawn version to support the newer API
            if target == "wasm32-unknown-emscripten" {
                eprintln!(
                    "cargo:warning=tvg-wg for WASM: Enabling ThorVG WebGPU renderer with Dawn"
                );
                true // Enable ThorVG's wg_engine for WASM
            } else {
                // Native targets: compile ThorVG's wg_engine if wgpu binaries available
                matches!(
                    target.as_str(),
                    "aarch64-apple-darwin"
                        | "x86_64-apple-darwin"
                        | "aarch64-unknown-linux-gnu"
                        | "x86_64-unknown-linux-gnu"
                )
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
                    "cargo:warning=tvg-wg requested but target {target_triple} doesn't have wgpu binaries"
                );
                eprintln!("cargo:warning=Building without ThorVG WebGPU renderer");
            }
            // For Emscripten, not having ThorVG's renderer is expected and OK
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
            let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let wgpu_emscripten_include = env::var("WGPU_INCLUDE").unwrap();
            let webgpu_include = PathBuf::from(&crate_dir)
                .parent()
                .unwrap()
                .join(&wgpu_emscripten_include);
            cc_build.include(&webgpu_include);
            eprintln!(
                "cargo:warning=Adding WebGPU include path: {}",
                webgpu_include.display()
            );
        }

        // Add WebGPU header include path and link wgpu-native for Apple platforms
        if tvg_wg_enabled {
            let target = env::var("TARGET").unwrap_or_default();

            if matches!(
                target.as_str(),
                "aarch64-apple-darwin"
                    | "x86_64-apple-darwin"
                    | "aarch64-unknown-linux-gnu"
                    | "x86_64-unknown-linux-gnu"
            ) {
                let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

                // Default paths: {crate_dir}/deps/wgpu/{target}/lib and /include
                let default_lib_path = PathBuf::from(&crate_dir)
                    .join("deps/wgpu")
                    .join(&target)
                    .join("lib");
                let default_include_path = PathBuf::from(&crate_dir)
                    .join("deps/wgpu")
                    .join(&target)
                    .join("include");

                // Use environment variables or defaults
                let wgpu_lib_path = env::var("WGPU_NATIVE_LIB")
                    .map(PathBuf::from)
                    .unwrap_or(default_lib_path);
                let wgpu_include_path = env::var("WGPU_INCLUDE")
                    .map(PathBuf::from)
                    .unwrap_or(default_include_path);

                // Verify paths exist
                let lib_path_exists = wgpu_lib_path.exists();
                let include_path_exists = wgpu_include_path.exists();

                // Add include path for wgpu headers
                if include_path_exists {
                    cc_build.include(&wgpu_include_path);
                }

                // Link wgpu-native static library
                if !lib_path_exists {
                    panic!(
                        "tvg-wg feature enabled but wgpu lib path not found: {}\n\
                         Set WGPU_NATIVE_LIB environment variable or place library at default location.",
                        wgpu_lib_path.display()
                    );
                }
                if !include_path_exists {
                    panic!(
                        "tvg-wg feature enabled but wgpu include path not found: {}\n\
                         Set WGPU_INCLUDE environment variable or place headers at default location.",
                        wgpu_include_path.display()
                    );
                }

                let abs_lib_path = wgpu_lib_path
                    .canonicalize()
                    .expect("Failed to canonicalize wgpu lib path");

                println!("cargo:rustc-link-search=native={}", abs_lib_path.display());
                println!("cargo:rustc-link-lib=static=wgpu_native");

                // Link platform-specific frameworks/libraries
                if target.contains("apple") {
                    println!("cargo:rustc-link-lib=framework=Metal");
                    println!("cargo:rustc-link-lib=framework=QuartzCore");
                    println!("cargo:rustc-link-lib=framework=Foundation");
                    println!("cargo:rustc-link-lib=framework=AppKit");
                } else if target.contains("linux") {
                    println!("cargo:rustc-link-lib=vulkan");
                }

                eprintln!(
                    "cargo:warning=Linked wgpu-native from {}",
                    abs_lib_path.display()
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

        cc_build.compile("thorvg");

        let bindings = bindgen::Builder::default()
            .header("deps/thorvg/src/bindings/capi/thorvg_capi.h")
            .generate()
            .expect("Failed to generate bindings");

        bindings.write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))?;

        // Generate WebGPU bindings if tvg-wg is enabled for native platforms
        if tvg_wg_enabled {
            let target = env::var("TARGET").unwrap_or_default();

            if matches!(
                target.as_str(),
                "aarch64-apple-darwin"
                    | "x86_64-apple-darwin"
                    | "aarch64-unknown-linux-gnu"
                    | "x86_64-unknown-linux-gnu"
            ) {
                let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

                // Default path: {crate_dir}/deps/wgpu/{target}/include
                let default_include_path = PathBuf::from(&crate_dir)
                    .join("deps/wgpu")
                    .join(&target)
                    .join("include")
                    .join("webgpu/webgpu.h");

                // Use environment variable or default
                let wgpu_include_path = env::var("WGPU_INCLUDE")
                    .map(PathBuf::from)
                    .unwrap_or(default_include_path);

                let wgpu_header = wgpu_include_path;

                if wgpu_header.exists() {
                    eprintln!(
                        "cargo:warning=Generating WebGPU bindings from {}",
                        wgpu_header.display()
                    );

                    let wgpu_bindings = bindgen::Builder::default()
                        .header(wgpu_header.to_str().unwrap())
                        // Only include WGPU types and functions
                        .allowlist_type("WGPU.*")
                        .allowlist_function("wgpu.*")
                        .allowlist_var("WGPU_.*")
                        // Use libc types
                        .ctypes_prefix("std::os::raw")
                        // Don't generate layout tests (they're huge and we don't need them)
                        .layout_tests(false)
                        // Disable default includes to avoid system header issues
                        .use_core()
                        .generate()
                        .expect("Failed to generate wgpu bindings");

                    wgpu_bindings.write_to_file(
                        PathBuf::from(env::var("OUT_DIR").unwrap()).join("wgpu_bindings.rs"),
                    )?;

                    eprintln!("cargo:warning=WebGPU bindings generated successfully");
                }
            }
        }

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
    if cfg!(feature = "tvg") {
        thorvg::build()?;
    }

    Ok(())
}
