use std::env;
use std::path::PathBuf;

fn main() {
    // println!("cargo:rustc-link-search=../final-build/lib/");
    println!("cargo:rustc-link-search=/usr/local/lib/");
    // Tell cargo to tell rustc to link the system
    // shared library.

    // NOTE: Static linking not working correctly
    // println!("cargo:rustc-link-lib=static=thorvg");
    println!("cargo:rustc-link-lib=thorvg");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

// use std::env;
// use std::path::PathBuf;

// fn main() {
//     let target = env::var("TARGET").unwrap_or("x86_64-apple-darwin".to_string());
//     let out_dir = env::var("OUT_DIR").unwrap();
//     let out_path = PathBuf::from(&out_dir);
//     let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
//     let manifest_path = PathBuf::from(manifest_dir);
//     let root_path = manifest_path.parent().unwrap();

//     // Determine the library path based on the target
//     let lib_path = root_path.join("./final-build/");

//     println!(
//         "cargo:rustc-link-search=native={}",
//         lib_path.join("lib").display()
//     );

//     println!("cargo:rustc-link-lib=static=thorvg");
//     println!("cargo:rustc-link-lib=static=thorvg");
//     // println!("cargo:rustc-link-args=pthread");

//     // Adjust linking type based on target
//     // "aarch64-apple-darwin" "x86_64-apple-ios" "aarch64-apple-ios-sim" "aarch64-apple-ios" "aarch64-linux-android" "armv7-linux-androideabi"
//     // match target.as_str() {
//     //     "x86_64-apple-ios" => println!("cargo:rustc-link-lib=dylib=thorvg"),
//     //     "aarch64-apple-ios-sim" => println!("cargo:rustc-link-lib=dylib=thorvg"),
//     //     "aarch64-apple-ios" => println!("cargo:rustc-link-lib=dylib=thorvg"),
//     //     _ => println!("cargo:rustc-link-lib=static=thorvg"),
//     // }

//     println!("cargo:rerun-if-changed=wrapper.h");

//     let bindings = bindgen::Builder::default()
//         .header("wrapper.h")
//         .clang_arg(format!("-I{}", lib_path.join("include").display()))
//         .parse_callbacks(Box::new(bindgen::CargoCallbacks))
//         .generate()
//         .expect("Unable to generate bindings");

//     bindings
//         .write_to_file(out_path.join("bindings.rs"))
//         .expect("Couldn't write bindings!");
// }
