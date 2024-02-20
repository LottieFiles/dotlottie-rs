use lazy_static::lazy_static;
use std::env;
use std::path::{Path, PathBuf};

// Path for all default artifacts
const DEFAULT_ARTIFACTS_DIR: &str = "../deps/artifacts/local-arch/usr";

// Target triple for WASM
const WASM32_UNKNOWN_EMSCRIPTEN: &str = "wasm32-unknown-emscripten";

// Target-specifc build settings
struct BuildSettings {
    static_libs: Vec<String>,
    dynamic_libs: Vec<String>,
    link_args: Vec<String>,
}

fn is_wasm_build() -> bool {
    match std::env::var("TARGET") {
        Ok(target) => target == WASM32_UNKNOWN_EMSCRIPTEN,
        Err(_) => panic!("TARGET environment variable not set"),
    }
}

lazy_static! {
    // The project root directory
    static ref PROJECT_DIR: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Default artifacts directories
    static ref DEFAULT_INCLUDE_DIR: PathBuf = PathBuf::from(format!("{DEFAULT_ARTIFACTS_DIR}/include"));
    static ref DEFAULT_LIB_DIR: PathBuf = PathBuf::from(&format!("{DEFAULT_ARTIFACTS_DIR}/lib"));
    static ref DEFAULT_LIB64_DIR: PathBuf = PathBuf::from(&format!("{DEFAULT_ARTIFACTS_DIR}/lib64"));

    // Native library dependencies
    static ref TARGET_BUILD_SETTINGS: BuildSettings = match is_wasm_build() {
        true => BuildSettings{
            static_libs: vec![String::from("thorvg")],
            dynamic_libs: vec![],
            link_args: vec![String::from("--no-entry")],
        },
        _ => BuildSettings{
            static_libs: vec![String::from("thorvg"), String::from("turbojpeg"), String::from("png"), String::from("z"), String::from("webp")],
            dynamic_libs: vec![String::from("c++")],
            link_args: vec![],
        },
   };
}

fn find_path(var: &str, default: &Path, required: bool) -> PathBuf {
    Some(
        env::var(var)
            .map(|v| PROJECT_DIR.join(v))
            .unwrap_or_else(|_| PROJECT_DIR.join(default)),
    )
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
    let include_dir = find_path("ARTIFACTS_INCLUDE_DIR", &DEFAULT_INCLUDE_DIR, true);
    let lib_dir = find_path("ARTIFACTS_LIB_DIR", &DEFAULT_LIB_DIR, true);
    let lib64_dir = find_path("ARTIFACTS_LIB64_DIR", &DEFAULT_LIB64_DIR, false);
    let bindings_output_path = env::var("OUT_DIR").map(PathBuf::from).unwrap();

    // Add artifacts library directories
    register_link_path(&lib_dir);
    register_link_path(&lib64_dir);

    // Apply build settings
    apply_build_settings(&TARGET_BUILD_SETTINGS);

    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_dir.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(bindings_output_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
