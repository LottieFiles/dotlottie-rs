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

        if cfg!(feature = "tvg-partial") {
            writeln!(thorvg_config_h, "#define THORVG_PARTIAL_RENDER_SUPPORT")?;
        }

        let tvg_sw_enabled = cfg!(feature = "tvg-sw");
        if tvg_sw_enabled {
            writeln!(thorvg_config_h, "#define THORVG_SW_RASTER_SUPPORT")?;
            src.push("deps/thorvg/src/renderer/sw_engine");
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

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    if cfg!(feature = "tvg") {
        thorvg::build()?;
    }

    Ok(())
}
