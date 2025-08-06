mod thorvg {
    use conan2::{ConanInstall, ConanVerbosity};
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

        if cfg!(feature = "tvg-log") {
            writeln!(thorvg_config_h, "#define THORVG_LOG_ENABLED")?;
        }

        if cfg!(feature = "tvg-sw") {
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

        thorvg_config_h.flush()?;

        let compiler = env::var("CXX").unwrap_or("clang++".to_string());


        cc::Build::new()
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
            .warnings(false)
            .compile("thorvg");

        let bindings = bindgen::Builder::default()
            .header("deps/thorvg/src/bindings/capi/thorvg_capi.h")
            .generate()
            .expect("Failed to generate bindings");

        bindings.write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))?;

        Ok(())
    }

    pub fn conan_build() -> std::io::Result<()> {
        let cargo_instructions = ConanInstall::new()
            .detect_profile()
            .build("missing")
            .verbosity(ConanVerbosity::Error)
            .run()
            .parse();

        // Emit instructions to cargo
        cargo_instructions.emit();

        // Link platform-specific C++ standard library
        get_cpp_standard_library()
            .iter()
            .for_each(|lib| println!("cargo:rustc-link-lib=dylib={lib}"));

        // Create bindings
        let mut bindings_builder = bindgen::Builder::default()
            .header("wrapper.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

        // Add conan include paths to bindgen
        for path in cargo_instructions.include_paths() {
            bindings_builder = bindings_builder.clang_arg(format!("-I{}", path.display()));
        }

        println!("cargo:rerun-if-changed=wrapper.h");
        let bindings = bindings_builder
            .generate()
            .expect("Failed to generate bindings");

        bindings.write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))?;

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    if cfg!(feature = "tvg-v1") {
        thorvg::build()?;
    } else if cfg!(feature = "tvg-v0") {
        thorvg::conan_build()?;
    }

    Ok(())
}
