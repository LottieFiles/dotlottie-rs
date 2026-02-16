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

    /// Check if WebGPU native libraries are available for the target platform
    fn is_wgpu_supported(target_triple: &str) -> bool {
        // WebGPU is supported on desktop platforms: macOS, Linux, Windows
        target_triple.contains("apple-darwin")
            || target_triple.contains("linux-gnu")
            || target_triple.contains("linux-musl")
            || target_triple.contains("pc-windows")
    }

    pub fn build() -> std::io::Result<()> {
        let target_triple = env::var("TARGET").unwrap_or_default();
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

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

        // Only enable WebGPU renderer if the target platform supports it
        if cfg!(feature = "tvg-wg") && is_wgpu_supported(&target_triple) {
            writeln!(thorvg_config_h, "#define THORVG_WG_RASTER_SUPPORT")?;
            src.push("deps/thorvg/src/renderer/wg_engine");
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

        if cfg!(feature = "tvg-wg") && is_wgpu_supported(&target_triple) {
            let vendored_wgpu_include = PathBuf::from(&crate_dir)
                .join("deps/wgpu")
                .join(&target_triple)
                .join("include");

            let wgpu_include_path = env::var("WGPU_NATIVE_INCLUDE")
                .map(PathBuf::from)
                .unwrap_or(vendored_wgpu_include);

            cc_build.include(&wgpu_include_path);

            if !target_triple.contains("emscripten") {
                // Default paths: {crate_dir}/deps/wgpu/{target}/lib and /include
                let vendored_wgpu_lib = PathBuf::from(&crate_dir)
                    .join("deps/wgpu")
                    .join(&target_triple)
                    .join("lib");

                // Use environment variables or defaults
                let wgpu_lib_path = env::var("WGPU_NATIVE_LIB")
                    .map(PathBuf::from)
                    .unwrap_or(vendored_wgpu_lib);

                let abs_lib_path = wgpu_lib_path
                    .canonicalize()
                    .expect("Failed to canonicalize wgpu lib path");

                println!("cargo:rustc-link-search=native={}", abs_lib_path.display());
                println!("cargo:rustc-link-lib=static=wgpu_native");

                // Link platform-specific frameworks/libraries
                if target_triple.contains("apple") {
                    println!("cargo:rustc-link-lib=framework=Metal");
                    println!("cargo:rustc-link-lib=framework=QuartzCore");
                    println!("cargo:rustc-link-lib=framework=Foundation");
                    println!("cargo:rustc-link-lib=framework=AppKit");
                } else if target_triple.contains("linux") {
                    println!("cargo:rustc-link-lib=vulkan");
                }
            }

            // generate wgpu binding
            bindgen::Builder::default()
                .header(wgpu_include_path.join("webgpu/webgpu.h").to_str().unwrap())
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
                .expect("Failed to generate wgpu bindings")
                .write_to_file(
                    PathBuf::from(env::var("OUT_DIR").unwrap()).join("wgpu_bindings.rs"),
                )?;
        } else if cfg!(feature = "tvg-wg") {
            println!(
                "cargo:warning=WebGPU renderer not available for target '{}', will use software renderer instead",
                target_triple
            );
        }

        for flag in simd_flags {
            cc_build.flag(flag);
        }

        if cfg!(feature = "tvg-threads")
            && std::env::var("CARGO_CFG_UNIX").is_ok()
            && !target_triple.contains("apple")
            && !target_triple.contains("android")
        {
            cc_build.flag("-pthread");
            println!("cargo:rustc-link-lib=pthread");
        }

        cc_build.compile("thorvg");

        let bindings = bindgen::Builder::default()
            .header("deps/thorvg/src/bindings/capi/thorvg_capi.h")
            .generate()
            .expect("Failed to generate bindings");

        bindings.write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))?;

        if cfg!(feature = "c_api") {
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
