use lazy_static::lazy_static;
use std::{env, path::PathBuf};

// Target triple for WASM
const WASM32_UNKNOWN_EMSCRIPTEN: &str = "wasm32-unknown-emscripten";

// Target-specifc build settings
struct BuildSettings {
    link_args: Vec<String>,
}

fn is_wasm_build() -> bool {
    match env::var("TARGET") {
        Ok(target) => target == WASM32_UNKNOWN_EMSCRIPTEN,
        Err(_) => panic!("TARGET environment variable not set"),
    }
}

lazy_static! {
    // Native library dependencies
    static ref TARGET_BUILD_SETTINGS: BuildSettings = match is_wasm_build() {
        true => BuildSettings{
            link_args: vec![String::from("--no-entry"), String::from("-sERROR_ON_UNDEFINED_SYMBOLS=0")],
        },
        _ => BuildSettings{
            link_args: vec![],
        },
   };
}

fn register_link_arg(arg: &String) {
    println!("cargo:rustc-link-arg={}", arg);
}

fn apply_build_settings(build_settings: &BuildSettings) {
    build_settings.link_args.iter().for_each(register_link_arg);
}

fn main() {
    // Always re-run the build script
    println!("cargo:rerun-if-changed=NULL");

    // Apply build settings
    apply_build_settings(&TARGET_BUILD_SETTINGS);

    if cfg!(feature = "uniffi") {
        uniffi::generate_scaffolding("src/dotlottie_player.udl").unwrap();
    }

    if cfg!(feature = "ffi") {
        // Execute cbindgen
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let config_path = PathBuf::from(&crate_dir).join("cbindgen.toml");
        let config = cbindgen::Config::from_file(config_path).unwrap();

        cbindgen::Builder::new()
            .with_crate(crate_dir)
            .with_config(config)
            .generate()
            .expect("Unable to generate bindings")
            .write_to_file("bindings.h");
    }
}
