use lazy_static::lazy_static;
use std::env;
use std::path::{Path, PathBuf};

// Path for all default artifacts
const DEFAULT_ARTIFACTS_DIR: &str = "../deps/artifacts/local-arch/usr";

lazy_static! {
    // The project root directory
    static ref PROJECT_DIR: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Default artifacts directories
    static ref DEFAULT_INCLUDE_DIR: PathBuf = PathBuf::from(format!("{DEFAULT_ARTIFACTS_DIR}/include"));
    static ref DEFAULT_LIB_DIR: PathBuf = PathBuf::from(&format!("{DEFAULT_ARTIFACTS_DIR}/lib"));
    static ref DEFAULT_LIB64_DIR: PathBuf = PathBuf::from(&format!("{DEFAULT_ARTIFACTS_DIR}/lib64"));

    // Native library dependencies
    static ref NATIVE_STATIC_LIBS: Vec<String> = vec![String::from("thorvg"), String::from("turbojpeg"), String::from("png"), String::from("z")];
    static ref NATIVE_DYNAMIC_LIBS: Vec<String> = vec![String::from("c++")];
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

fn main() {
    let include_dir = find_path("ARTIFACTS_INCLUDE_DIR", &DEFAULT_INCLUDE_DIR, true);
    let lib_dir = find_path("ARTIFACTS_LIB_DIR", &DEFAULT_LIB_DIR, true);
    let lib64_dir = find_path("ARTIFACTS_LIB64_DIR", &DEFAULT_LIB64_DIR, false);
    let bindings_output_path = env::var("OUT_DIR").map(PathBuf::from).unwrap();

    // Add artifacts library directories
    register_link_path(&lib_dir);
    register_link_path(&lib64_dir);

    // Ensure libraries are made available
    NATIVE_STATIC_LIBS.iter().for_each(register_static_lib);
    NATIVE_DYNAMIC_LIBS.iter().for_each(register_dylib);
    println!("cargo:rustc-link-lib=static=thorvg");

    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_dir.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(bindings_output_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
