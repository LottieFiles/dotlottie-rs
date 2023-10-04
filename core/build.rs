use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = PathBuf::from(manifest_dir);
    let root_path = manifest_path.parent().unwrap();

    // Determine the library path based on the target
    let lib_path = root_path.join("build").join(&target);

    println!("LIB PATH::::::: {}", lib_path.display());
    println!("cargo:rustc-link-search=native={}", lib_path.join("lib").display());
    // println!("cargo:rustc-link-lib=static=thorvg");
    println!("cargo:rerun-if-changed=wrapper.h");

    println!("Include --> {}", format!("-I{}", lib_path.join("include").display()));

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", lib_path.join("include").display()))
        .clang_arg(format!("-L{}", lib_path.join("lib").display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
