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

        let thorvg_bindings = bindgen::Builder::default()
            .header("deps/thorvg/src/bindings/capi/thorvg_capi.h")
            .generate()
            .expect("Failed to generate bindings");

        thorvg_bindings
            .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))?;

        // Generate jerryscript bindings
        //
        // WASM Target Limitation:
        // Bindgen with libclang fails to generate function declarations for wasm32-unknown-emscripten
        // when parsing headers that use preprocessor-expanded `extern "C" {}` blocks.
        //
        // Root Cause Investigation:
        // - Bindgen uses libclang to parse C headers and extract declarations
        // - The JerryScript headers use `JERRY_C_API_BEGIN`/`END` macros that expand to `extern "C" {}`
        // - When targeting wasm32-unknown-emscripten, libclang incorrectly reports linkage for functions
        //   inside these macro-expanded blocks, causing bindgen to skip them entirely
        // - This results in bindings with ~991 lines of types/constants but ZERO function declarations
        //
        // Attempted Fixes (all failed):
        // 1. Adding `-x c` flag to force C mode → still 0 functions
        // 2. Bypassing BINDGEN_EXTRA_CLANG_ARGS (emscripten sysroot) → still 0 functions
        // 3. Using a `.c` wrapper file instead of `.h` → still 0 functions
        //
        // Conclusion:
        // This is a fundamental limitation of bindgen/libclang when cross-compiling to WASM
        // with emscripten. Manual bindings are the correct and recommended solution for this scenario.
        //
        // References:
        // - Bindgen linkage detection: https://github.com/rust-lang/rust-bindgen/blob/main/bindgen/ir/function.rs#L743-748
        // - Similar cross-compilation issues have been reported in the Rust ecosystem
        //
        if target_triple == "wasm32-unknown-emscripten" {
            // Manual bindings for WASM target
            // Only includes functions actually used by the codebase
            let manual_bindings = r#"
/* Manually generated bindings for wasm32-unknown-emscripten */
/* Bindgen fails to extract functions from macro-expanded extern "C" blocks */

pub type jerry_value_t = u32;
pub type jerry_char_t = u8;
pub type jerry_size_t = u32;
pub type jerry_length_t = u32;
pub type jerry_init_flag_t = u32;

// Constants
pub const JERRY_INIT_EMPTY: u32 = 0;
pub const jerry_init_flag_t_JERRY_INIT_EMPTY: u32 = 0;
pub const jerry_encoding_t_JERRY_ENCODING_UTF8: u32 = 0;

// Function declarations matching JerryScript C API
extern "C" {
    pub fn jerry_init(flags: jerry_init_flag_t);
    pub fn jerry_cleanup();
    pub fn jerry_eval(source_p: *const jerry_char_t, source_size: usize, flags: u32) -> jerry_value_t;
    pub fn jerry_undefined() -> jerry_value_t;
    pub fn jerry_boolean(value: bool) -> jerry_value_t;
    pub fn jerry_number(value: f32) -> jerry_value_t;
    pub fn jerry_string_sz(str_p: *const ::std::os::raw::c_char) -> jerry_value_t;
    pub fn jerry_value_is_undefined(value: jerry_value_t) -> bool;
    pub fn jerry_value_is_number(value: jerry_value_t) -> bool;
    pub fn jerry_value_is_string(value: jerry_value_t) -> bool;
    pub fn jerry_value_is_object(value: jerry_value_t) -> bool;
    pub fn jerry_value_is_exception(value: jerry_value_t) -> bool;
    pub fn jerry_value_as_number(value: jerry_value_t) -> f32;
    pub fn jerry_value_to_string(value: jerry_value_t) -> jerry_value_t;
    pub fn jerry_string_length(value: jerry_value_t) -> jerry_length_t;
    pub fn jerry_string_to_buffer(
        value: jerry_value_t,
        encoding: u32,
        buffer_p: *mut jerry_char_t,
        buffer_size: jerry_size_t,
    ) -> jerry_size_t;
    pub fn jerry_value_free(value: jerry_value_t);
}
"#;
            fs::write(out_dir.join("jerryscript_bindings.rs"), manual_bindings)?;
        } else {
            // Use bindgen for native targets (works correctly on macOS, Linux, etc.)
            let jerryscript_bindings = bindgen::Builder::default()
                .header(
                    "deps/thorvg/src/loaders/lottie/jerryscript/jerry-core/include/jerryscript.h",
                )
                .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
                .generate()
                .expect("Failed to generate jerryscript bindings");

            jerryscript_bindings.write_to_file(out_dir.join("jerryscript_bindings.rs"))?;
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
