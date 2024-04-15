use lazy_static::lazy_static;
use std::env;

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
    // Apply build settings
    apply_build_settings(&TARGET_BUILD_SETTINGS);
}
