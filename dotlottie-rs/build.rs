mod wgpu_native {
    use std::env;
    use std::fs;
    use std::io;
    use std::path::PathBuf;
    const WGPU_NATIVE_VERSION: &str = "v25.0.2.1";

    fn artifact_name(target: &str) -> Option<&'static str> {
        match target {
            // macOS
            "aarch64-apple-darwin" => Some("wgpu-macos-aarch64-release"),
            "x86_64-apple-darwin" => Some("wgpu-macos-x86_64-release"),
            // iOS
            "aarch64-apple-ios" => Some("wgpu-ios-aarch64-release"),
            "aarch64-apple-ios-sim" => Some("wgpu-ios-aarch64-simulator-release"),
            "x86_64-apple-ios" => Some("wgpu-ios-x86_64-simulator-release"),
            // Mac Catalyst (runs on macOS hardware)
            "aarch64-apple-ios-macabi" => Some("wgpu-macos-aarch64-release"),
            "x86_64-apple-ios-macabi" => Some("wgpu-macos-x86_64-release"),
            // Linux
            "aarch64-unknown-linux-gnu" => Some("wgpu-linux-aarch64-release"),
            "x86_64-unknown-linux-gnu" => Some("wgpu-linux-x86_64-release"),
            // Android
            "aarch64-linux-android" => Some("wgpu-android-aarch64-release"),
            "x86_64-linux-android" => Some("wgpu-android-x86_64-release"),
            "i686-linux-android" => Some("wgpu-android-i686-release"),
            "armv7-linux-androideabi" => Some("wgpu-android-armv7-release"),
            _ => None,
        }
    }

    /// Returns the persistent cache directory: `$CARGO_HOME/wgpu-native-cache/{version}/`.
    fn cache_dir() -> PathBuf {
        let cargo_home = env::var("CARGO_HOME").unwrap_or_else(|_| {
            let home = env::var("HOME").expect("Neither CARGO_HOME nor HOME is set");
            format!("{home}/.cargo")
        });
        PathBuf::from(cargo_home)
            .join("wgpu-native-cache")
            .join(WGPU_NATIVE_VERSION)
    }

    fn download_file(url: &str, dest: &std::path::Path) -> io::Result<()> {
        let response = minreq::get(url)
            .send()
            .map_err(|e| io::Error::other(format!("Download failed: {url}: {e}")))?;
        if response.status_code < 200 || response.status_code >= 300 {
            return Err(io::Error::other(format!(
                "Download failed: {url}: HTTP {}",
                response.status_code
            )));
        }
        fs::write(dest, response.as_bytes())?;
        Ok(())
    }

    fn extract_zip(zip_path: &std::path::Path, dest_dir: &std::path::Path) -> io::Result<()> {
        fs::create_dir_all(dest_dir)?;
        let file = fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| io::Error::other(format!("Failed to read zip: {e}")))?;
        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| io::Error::other(format!("Failed to read zip entry: {e}")))?;
            let out_path = dest_dir.join(entry.mangled_name());
            if entry.is_dir() {
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut out_file = fs::File::create(&out_path)?;
                io::copy(&mut entry, &mut out_file)?;
            }
        }
        Ok(())
    }

    /// Main entry point. Returns `(include_path, lib_path)`.
    ///
    /// Priority chain:
    /// 1. `WGPU_NATIVE_INCLUDE` + `WGPU_NATIVE_LIB` env vars
    /// 2. Cached download at `$CARGO_HOME/wgpu-native-cache/{version}/{artifact}/`
    /// 3. Fresh download from GitHub
    ///
    pub fn ensure_available(target: &str) -> io::Result<(PathBuf, PathBuf)> {
        println!("cargo:rerun-if-env-changed=WGPU_NATIVE_INCLUDE");
        println!("cargo:rerun-if-env-changed=WGPU_NATIVE_LIB");

        // Priority 1: env var overrides
        if let (Ok(inc), Ok(lib)) = (env::var("WGPU_NATIVE_INCLUDE"), env::var("WGPU_NATIVE_LIB")) {
            return Ok((PathBuf::from(inc), PathBuf::from(lib)));
        }

        let artifact = artifact_name(target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                format!("No wgpu-native artifact for target: {target}"),
            )
        })?;

        let cache = cache_dir().join(artifact);

        // Priority 2: cached download
        if cache.join("include/webgpu/webgpu.h").exists()
            && cache.join("lib/libwgpu_native.a").exists()
        {
            return Ok((cache.join("include"), cache.join("lib")));
        }

        // Priority 3: download
        let url = format!(
            "https://github.com/gfx-rs/wgpu-native/releases/download/{WGPU_NATIVE_VERSION}/{artifact}.zip"
        );

        let download_dir = cache_dir();
        fs::create_dir_all(&download_dir)?;
        let zip_path = download_dir.join(format!("{artifact}.zip"));

        download_file(&url, &zip_path)?;
        extract_zip(&zip_path, &cache)?;

        let _ = fs::remove_file(&zip_path);

        Ok((cache.join("include"), cache.join("lib")))
    }
}

mod thorvg {
    use std::env;
    use std::fs::{self, create_dir_all, OpenOptions};
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};

    const EMSCRIPTEN_VERSION: &str = "3.1.70";
    const WEBGPU_HEADERS_REV: &str = "bac5208";

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

    pub(super) fn collect_files(dir: &str) -> Vec<String> {
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

    /// Verify that the CXX compiler is clang++ >= 16 (required for wasm32-unknown-unknown).
    fn verify_clang_version(compiler: &str) {
        let output = std::process::Command::new(compiler)
            .arg("--version")
            .output()
            .unwrap_or_else(|e| {
                panic!(
                    "failed to run `{compiler} --version`: {e}\n\
                    Set CXX to a clang++ >= 16 with the wasm32-unknown-unknown \
                    backend (e.g. Homebrew LLVM: CXX=/opt/homebrew/opt/llvm/bin/clang++)"
                )
            });
        let stdout = std::str::from_utf8(&output.stdout).unwrap_or("");
        let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
        let version = stdout
            .lines()
            .chain(stderr.lines())
            .find_map(|l| {
                l.split_once("clang version ").and_then(|(_, ver)| {
                    ver.split_once('.')
                        .and_then(|(major, _)| major.parse::<u32>().ok())
                })
            })
            .unwrap_or_else(|| {
                panic!(
                    "failed to parse version from `{compiler} --version`\n\
                    exit status: {}\nstdout:\n{stdout}\nstderr:\n{stderr}\n\
                    Ensure CXX points to a working clang++ >= 16.",
                    output.status
                )
            });
        if version < 16 {
            panic!(
                "clang++ {version} is too old; wasm32-unknown-unknown requires clang++ >= 16.\n\
                Set CXX to a newer clang++ (e.g. /opt/homebrew/opt/llvm/bin/clang++)"
            );
        }
    }

    /// Download and cache Emscripten system headers (libc++, musl, WebGPU) into `out_dir`.
    /// Returns the path to the emscripten directory.
    fn setup_emscripten_headers(out_dir: &Path) -> io::Result<PathBuf> {
        let emscripten_dir = out_dir.join("emscripten");

        if !emscripten_dir.exists() {
            let url = format!(
                "https://github.com/emscripten-core/emscripten/archive/refs/tags/{EMSCRIPTEN_VERSION}.zip"
            );
            let response = minreq::get(&url)
                .send()
                .map_err(|e| io::Error::other(format!("Failed to download emscripten: {e}")))?;
            if response.status_code < 200 || response.status_code >= 300 {
                return Err(io::Error::other(format!(
                    "Failed to download emscripten: HTTP {}",
                    response.status_code
                )));
            }
            let zip_path = out_dir.join("emscripten.zip");
            fs::write(&zip_path, response.as_bytes())?;

            let file = fs::File::open(&zip_path)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| io::Error::other(format!("Failed to open emscripten zip: {e}")))?;
            archive
                .extract(out_dir)
                .map_err(|e| io::Error::other(format!("Failed to extract emscripten zip: {e}")))?;
            fs::remove_file(&zip_path)?;
            fs::rename(
                out_dir.join(format!("emscripten-{EMSCRIPTEN_VERSION}")),
                &emscripten_dir,
            )?;

            // Write stub html5_webgl.h (ThorVG's GL engine needs this)
            let html5_webgl_path = emscripten_dir.join("system/include/emscripten/html5_webgl.h");
            fs::write(
                &html5_webgl_path,
                "#pragma once\n\
                typedef void* EMSCRIPTEN_WEBGL_CONTEXT_HANDLE;\n\
                typedef int   EMSCRIPTEN_RESULT;\n\
                #ifdef __cplusplus\n\
                extern \"C\" {\n\
                #endif\n\
                EMSCRIPTEN_WEBGL_CONTEXT_HANDLE emscripten_webgl_get_current_context(void);\n\
                EMSCRIPTEN_RESULT emscripten_webgl_make_context_current(EMSCRIPTEN_WEBGL_CONTEXT_HANDLE context);\n\
                #ifdef __cplusplus\n\
                }\n\
                #endif\n",
            )?;

            // Download WebGPU headers
            let webgpu_header_dir = emscripten_dir.join("system/include/webgpu");
            fs::create_dir_all(&webgpu_header_dir)?;
            let webgpu_url = format!(
                "https://raw.githubusercontent.com/webgpu-native/webgpu-headers/{WEBGPU_HEADERS_REV}/webgpu.h"
            );
            let wgpu_resp = minreq::get(&webgpu_url)
                .send()
                .map_err(|e| io::Error::other(format!("Failed to download webgpu.h: {e}")))?;
            fs::write(webgpu_header_dir.join("webgpu.h"), wgpu_resp.as_bytes())?;
        }

        Ok(emscripten_dir)
    }

    pub fn build() -> io::Result<()> {
        let target_triple = env::var("TARGET").unwrap_or_default();
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

        let is_wasm = target_triple.starts_with("wasm32-unknown-");
        let is_wasm_unknown = target_triple == "wasm32-unknown-unknown";

        let compiler = env::var("CXX").unwrap_or_else(|_| "clang++".to_string());

        // wasm32-unknown-unknown needs clang++ >= 16 and emscripten system headers
        if is_wasm_unknown {
            verify_clang_version(&compiler);
        }
        let emscripten_dir = if is_wasm_unknown {
            Some(setup_emscripten_headers(&out_dir)?)
        } else {
            None
        };

        // C++ standard library linking
        if is_wasm_unknown {
            // Prevent cc from linking stdc++ — no C++ runtime available
            env::set_var("CXXSTDLIB", "");
        } else {
            get_cpp_standard_library()
                .iter()
                .for_each(|lib| println!("cargo:rustc-link-lib=dylib={lib}"));
        }

        // Source directories (identical for all targets)
        let mut src = vec![
            "deps/thorvg/inc",
            "deps/thorvg/src/common",
            "deps/thorvg/src/bindings/capi",
            "deps/thorvg/src/loaders/lottie",
            "deps/thorvg/src/loaders/raw",
            "deps/thorvg/src/renderer",
        ];

        // --- config.h generation ---
        let mut thorvg_config_h = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(out_dir.join("config.h"))?;

        writeln!(thorvg_config_h, "#define THORVG_VERSION_STRING \"1.0.1\"")?;
        writeln!(thorvg_config_h, "#define THORVG_LOTTIE_LOADER_SUPPORT")?;
        writeln!(thorvg_config_h, "#define TVG_STATIC")?;
        writeln!(thorvg_config_h, "#define WIN32_LEAN_AND_MEAN")?;

        if !is_wasm {
            writeln!(thorvg_config_h, "#define THORVG_FILE_IO_SUPPORT 1")?;
        }

        if cfg!(feature = "tvg-log") {
            writeln!(thorvg_config_h, "#define THORVG_LOG_ENABLED")?;
        }

        if cfg!(feature = "tvg-threads") && !is_wasm {
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

            if is_wasm {
                writeln!(thorvg_config_h, "#define THORVG_GL_TARGET_GLES 1")?;
            } else {
                writeln!(thorvg_config_h, "#define THORVG_GL_TARGET_GL 1")?;
            }
        }

        if cfg!(feature = "tvg-wg") {
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

        // SIMD — skip entirely for wasm32-unknown-unknown (no SIMD support there yet)
        let tvg_simd_enabled = cfg!(feature = "tvg-simd");
        let mut simd_flags: Vec<&str> = Vec::new();
        if tvg_sw_enabled && tvg_simd_enabled && !is_wasm_unknown {
            if target_triple.contains("x86_64")
                || target_triple.contains("i686")
                || target_triple.contains("i586")
            {
                writeln!(thorvg_config_h, "#define THORVG_AVX_VECTOR_SUPPORT")?;
                simd_flags.push("-mavx");
            } else if target_triple.contains("aarch64") {
                writeln!(thorvg_config_h, "#define THORVG_NEON_VECTOR_SUPPORT")?;
            } else if target_triple.contains("armv7") {
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

        // --- cc::Build setup ---
        let mut cc_build = cc::Build::new();
        cc_build
            .compiler(&compiler)
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

        // wasm32-unknown-unknown: add emscripten system headers and defines
        if let Some(ref emscripten_dir) = emscripten_dir {
            cc_build
                .include(emscripten_dir.join("system/lib/libcxx/include"))
                .include(emscripten_dir.join("system/lib/libc/musl/arch/emscripten"))
                .include(emscripten_dir.join("system/lib/libc/musl/include"))
                .include(emscripten_dir.join("system/include"))
                .include(emscripten_dir.join("system/lib/pthread"))
                .define("__EMSCRIPTEN__", None);

            let target_features_str = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
            if target_features_str.split(',').any(|f| f == "atomics") {
                cc_build.flag("-matomics");
            }
            if target_features_str.split(',').any(|f| f == "bulk-memory") {
                cc_build.flag("-mbulk-memory");
            }
        }

        // wgpu-native — not used for wasm32-unknown-unknown
        if cfg!(feature = "tvg-wg") && !is_wasm_unknown {
            let (wgpu_include_path, wgpu_lib_path) =
                crate::wgpu_native::ensure_available(&target_triple)
                    .expect("Failed to obtain wgpu-native. Set WGPU_NATIVE_LIB and WGPU_NATIVE_INCLUDE env vars, or check your network connection.");

            cc_build.include(&wgpu_include_path);

            let abs_lib_path = wgpu_lib_path
                .canonicalize()
                .expect("Failed to canonicalize wgpu lib path");

            println!("cargo:rustc-link-search=native={}", abs_lib_path.display());
            println!("cargo:rustc-link-lib=static=wgpu_native");

            // Link platform-specific frameworks/libraries
            if target_triple.contains("apple") || target_triple.contains("ios") {
                println!("cargo:rustc-link-lib=framework=Metal");
                println!("cargo:rustc-link-lib=framework=QuartzCore");
                println!("cargo:rustc-link-lib=framework=Foundation");
                if target_triple.contains("darwin") {
                    // macOS
                    println!("cargo:rustc-link-lib=framework=AppKit");
                } else {
                    // iOS, Mac Catalyst
                    println!("cargo:rustc-link-lib=framework=UIKit");
                }
            } else if target_triple.contains("linux") && !target_triple.contains("android") {
                println!("cargo:rustc-link-lib=vulkan");
            }

            bindgen::Builder::default()
                .header(wgpu_include_path.join("webgpu/webgpu.h").to_str().unwrap())
                .allowlist_type("WGPU.*")
                .allowlist_function("wgpu.*")
                .allowlist_var("WGPU_.*")
                .ctypes_prefix("std::os::raw")
                .layout_tests(false)
                .use_core()
                .generate()
                .expect("Failed to generate wgpu bindings")
                .write_to_file(
                    PathBuf::from(env::var("OUT_DIR").unwrap()).join("wgpu_bindings.rs"),
                )?;
        }

        for flag in &simd_flags {
            cc_build.flag(flag);
        }

        if cfg!(feature = "tvg-threads")
            && !is_wasm
            && env::var("CARGO_CFG_UNIX").is_ok()
            && !target_triple.contains("apple")
            && !target_triple.contains("android")
        {
            cc_build.flag("-pthread");
            println!("cargo:rustc-link-lib=pthread");
        }

        cc_build.compile("thorvg");

        // Generate ThorVG C API bindings
        let bindings = bindgen::Builder::default()
            .header("deps/thorvg/src/bindings/capi/thorvg_capi.h")
            .generate()
            .expect("Failed to generate bindings");
        bindings.write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))?;

        // cbindgen — only for native builds with c_api feature
        if cfg!(feature = "c_api") && !is_wasm_unknown {
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
