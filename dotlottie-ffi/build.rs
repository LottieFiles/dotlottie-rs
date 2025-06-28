use std::{env, path::PathBuf};

fn main() {
    // Always re-run the build script
    println!("cargo:rerun-if-changed=NULL");

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
