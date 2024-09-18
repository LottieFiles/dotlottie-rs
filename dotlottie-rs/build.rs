use conan2::{ConanInstall, ConanVerbosity};
use lazy_static::lazy_static;
use std::env;
use std::path::{Path, PathBuf};

// Artifacts environment variables
const ARTIFACTS_INCLUDE_DIR: &str = "ARTIFACTS_INCLUDE_DIR";
const ARTIFACTS_LIB_DIR: &str = "ARTIFACTS_LIB_DIR";
const ARTIFACTS_LIB64_DIR: &str = "ARTIFACTS_LIB64_DIR";

// Target triple for WASM
const WASM32_UNKNOWN_EMSCRIPTEN: &str = "wasm32-unknown-emscripten";

// Target-specifc build settings
struct BuildSettings {
    static_libs: Vec<String>,
    dynamic_libs: Vec<String>,
    link_args: Vec<String>,
}

fn is_artifacts_provided() -> bool {
    std::env::var(ARTIFACTS_INCLUDE_DIR).is_ok() && std::env::var(ARTIFACTS_LIB_DIR).is_ok()
}

fn is_wasm_build() -> bool {
    match std::env::var("TARGET") {
        Ok(target) => target == WASM32_UNKNOWN_EMSCRIPTEN,
        Err(_) => panic!("TARGET environment variable not set"),
    }
}

fn platform_libs() -> Vec<String> {
    match env::var("HOST") {
        Ok(triple) if triple.contains("apple") => vec![String::from("c++")],
        Ok(_) if std::env::var("CARGO_CFG_UNIX").is_ok() => vec![String::from("stdc++")],
        Ok(_) => vec![],
        Err(_) => panic!("CARGO_CFG_TARGET_VENDOR environment variable not set"),
    }
}

lazy_static! {
    // The project root directory
    static ref PROJECT_DIR: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Native library dependencies
    static ref TARGET_BUILD_SETTINGS: BuildSettings = match is_artifacts_provided() {
        true if is_wasm_build() => BuildSettings{
            static_libs: vec![String::from("thorvg")],
            dynamic_libs: vec![],
            link_args: vec![String::from("--no-entry")],
        },
        true => BuildSettings{
            static_libs: vec![String::from("thorvg"), String::from("turbojpeg"), String::from("png"), String::from("z"), String::from("webp")],
            dynamic_libs: platform_libs(),
            link_args: vec![],
        },
        // Conan build
        _ => BuildSettings{
            static_libs: vec![],
            dynamic_libs: platform_libs(),
            link_args: vec![],
        }
   };
}

fn find_path(var: &str, required: bool) -> PathBuf {
    Some(env::var(var).map(|v| PROJECT_DIR.join(v)).unwrap())
        .filter(|p| !required || p.exists())
        .unwrap()
}

fn register_link_path(lib_path: &Path) {
    if lib_path.exists() {
        println!("cargo:rustc-link-search=native={}", lib_path.display());
    }
}

fn register_static_lib(lib: &String) {
    println!("cargo:rustc-link-lib=static={}", lib);
}

fn register_dylib(lib: &String) {
    println!("cargo:rustc-link-lib=dylib={}", lib);
}

fn register_link_arg(arg: &String) {
    println!("cargo:rustc-link-arg={}", arg);
}

fn apply_build_settings(build_settings: &BuildSettings) {
    build_settings
        .static_libs
        .iter()
        .for_each(register_static_lib);
    build_settings.dynamic_libs.iter().for_each(register_dylib);
    build_settings.link_args.iter().for_each(register_link_arg);
}

fn main() {
    let mut builder = bindgen::Builder::default().header("wrapper.h");
    if is_artifacts_provided() {
        let include_dir = find_path(ARTIFACTS_INCLUDE_DIR, true);
        let lib_dir = find_path(ARTIFACTS_LIB_DIR, true);
        let lib64_dir = find_path(ARTIFACTS_LIB64_DIR, false);

        // Add artifacts library directories
        register_link_path(&lib_dir);
        register_link_path(&lib64_dir);

        // Update bindings builder
        builder = builder.clang_arg(format!("-I{}", include_dir.display()))
    } else {
        // Conan build
        let cargo_instructions = ConanInstall::new()
            .detect_profile()
            .build("missing")
            .verbosity(ConanVerbosity::Error)
            .run()
            .parse();

        // Emit instructions to cargo
        cargo_instructions.emit();

        // Update bindings builder
        for path in cargo_instructions.include_paths() {
            builder = builder.clang_arg(format!("-I{}", path.display()))
        }
    }

    // Apply build settings
    apply_build_settings(&TARGET_BUILD_SETTINGS);

    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings_output_path = env::var("OUT_DIR").map(PathBuf::from).unwrap();
    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(bindings_output_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
