## Build system

GNU `make` builds all artifacts for the `android`, `apple`, `wasm`, `linux`, `windows`, and `native` targets.
The top-level `Makefile` defines the native, test, and setup targets and includes one makefile per platform from `make/`.
This document describes how those makefiles work so you can change or extend them.

A platform build runs these steps:

1. Check the platform toolchain: the Android NDK, Xcode, MSVC, or clang plus wasm-bindgen-cli for WASM.
2. Generate the C header with cbindgen from the `c_api` feature. WASM builds run wasm-bindgen-cli instead.
3. Build the `dotlottie-rs` library once per target architecture.
4. Package the platform artifacts: `jniLibs/` for Android, an XCFramework for Apple, JS and WASM modules for the web.
5. Write everything to `release/`, along with a version file.

### Platform makefiles

- `Makefile` holds the native, test, clippy, setup, and help targets and includes the files below.
- `make/android.mk` builds Android for ARM64, x86_64, x86, and ARMv7.
- `make/apple.mk` builds macOS, iOS, tvOS, visionOS, watchOS, and Mac Catalyst.
- `make/wasm.mk` builds WebAssembly with cargo and wasm-bindgen-cli.
- `make/linux.mk` builds Linux x86_64 and ARM64.
- `make/windows.mk` builds Windows x86_64 and ARM64 with the MSVC toolchain.

Each platform makefile checks its toolchain, sets up cross-compilation, produces the C header (or wasm-bindgen output), and packages the release.

#### Android (`make/android.mk`)

`android-check-ndk` finds the NDK through `ANDROID_NDK_HOME` and fails if it is older than r28.
Each architecture build compiles `libdotlottie_player.so` with the NDK clang for `API_LEVEL`, copies `libc++_shared.so` next to it, and strips both.
`android-package` then adds the cbindgen header and a `version.txt` that holds the crate version and commit hash.

The output in `release/android/` looks like this:

```
include/dotlottie_player.h
jniLibs/{arm64-v8a,armeabi-v7a,x86,x86_64}/libdotlottie_player.so
version.txt
```

Targets:

- `android` builds all four architectures and packages the release.
- `android-{arch}` builds one architecture, for example `android-aarch64`.
- `android-setup` installs the Rust targets.
- `android-clean` removes Android build artifacts.

#### Apple (`make/apple.mk`)

The Apple build covers macOS, iOS, tvOS, visionOS, watchOS, and Mac Catalyst.
It builds a framework per platform and SDK, merges architectures with `lipo`, and combines the frameworks into one XCFramework.
The cbindgen header is the bridge for Swift and Objective-C.
Mac Catalyst, tvOS, visionOS, and watchOS build with `cargo +nightly` and `-Z build-std`.

Targets:

- `apple` builds every Apple platform and the XCFramework.
- `apple-{platform}` builds one platform, for example `apple-ios`.
- `apple-{platform}-{arch}` builds one architecture, for example `apple-macos-arm64`.
- `apple-setup` installs the Rust targets and toolchain components.

#### WASM (`make/wasm.mk`)

The WASM build targets `wasm32-unknown-unknown`.
It compiles with `cargo rustc`, generates bindings with wasm-bindgen-cli, and runs `wasm-opt -O3` if `wasm-opt` is on `PATH`.

- The `wasm-bindgen-api` feature produces JS and TypeScript bindings for the software, WebGL, and WebGPU renderers.
- wasm-bindgen-cli writes the TypeScript definitions with `--typescript`.
- The output is the `.wasm` binary plus a wasm-bindgen glue file. There is no Emscripten runtime.
- Stable Rust is enough, with no nightly and no EMSDK. ThorVG's C++ needs a clang that can target wasm. The build prefers Homebrew LLVM's clang if it is installed.

Targets:

- `wasm` builds the software renderer into `release/wasm/`.
- `wasm-webgl` builds the WebGL2 renderer into `release/wasm-webgl/`.
- `wasm-webgpu` builds the WebGPU renderer into `release/wasm-webgpu/`.
- `wasm-all` builds all three.
- `wasm-setup` installs the `wasm32-unknown-unknown` target and the wasm-bindgen-cli version that matches Cargo.lock.
- `wasm-clean` removes WASM build artifacts.

#### Native

The native targets build `dotlottie-rs` with the `c_api` feature as a dynamic library for the host platform, along with the cbindgen header.
Use them for local development and testing.

Targets:

- `native` builds with the software renderer.
- `native-opengl` builds with the OpenGL renderer.
- `native-webgpu` builds with the WebGPU renderer.
- `native-clean` removes native build artifacts.

#### Windows (`make/windows.mk`)

The Windows build links against MSVC for x86_64 and ARM64.
The `cc` crate finds `cl.exe` on its own.
Each target produces a static library (`.lib`), a dynamic library (`.dll`), an import library (`.dll.lib`), and the cbindgen header.

Targets:

- `windows` builds both architectures.
- `windows-x86_64` builds `x86_64-pc-windows-msvc`.
- `windows-arm64` builds `aarch64-pc-windows-msvc`.
- `windows-setup` installs the Rust targets.
- `windows-clean` removes Windows build artifacts.

Prerequisites:

- Visual Studio 2019 or later, or the Build Tools, with the "Desktop development with C++" workload
- GNU Make, from MSYS2, Git Bash, or `choco install make`
- The ARM64 build tools from the VS Installer, to cross-compile for ARM64

### Features and configuration

#### Feature flags

The Android, Apple, Linux, and Windows builds read a `FEATURES` variable that replaces their default extra features.
On Apple, the tvOS, visionOS, and watchOS targets ignore it and always use `APPLE_BASE_FEATURES`.
The WASM and native builds use fixed feature lists.

The features you are most likely to change:

- `c_api`: C API and cbindgen header (Android, Apple, Linux, Windows, native)
- `wasm-bindgen-api`: wasm-bindgen bindings for JS and TS (all WASM builds)
- `webgl`: WebGL renderer, used with `wasm-bindgen-api`
- `webgpu`: WebGPU renderer, used with `wasm-bindgen-api`
- `tvg-webp`, `tvg-png`, `tvg-jpg`: WebP, PNG, and JPEG images
- `tvg-ttf`, `tvg-otf`: TrueType and OpenType fonts
- `tvg-threads`: multi-threaded rendering
- `tvg-lottie-expressions`: Lottie expressions

The crate's default features are `dotlottie`, `state-machines`, and `theming`.
Each makefile adds the renderer backend (`tvg`, `tvg-cpu`, `tvg-gl`, `tvg-wg`) and the binding layer (`c_api` or `wasm-bindgen-api`) itself.

#### Environment variables

Android reads `ANDROID_NDK_HOME`, the NDK path.
It defaults to `/opt/homebrew/share/android-ndk` on macOS and `/opt/android-ndk` on Linux.
`API_LEVEL` sets the Android API level and defaults to `21`.

Apple reads `XCODE_PATH`, which defaults to the output of `xcode-select -p`.
`MIN_IOS_VERSION`, `MIN_MACOS_VERSION`, `MIN_TVOS_VERSION`, `MIN_VISIONOS_VERSION`, `MIN_WATCHOS_VERSION`, and `MIN_MACCATALYST_VERSION` set the deployment targets.

WASM needs no variables.
It uses the `clang` and `clang++` on `PATH`, or Homebrew LLVM if present.

#### Version information

Every release writes a version string built from two values:

- `CRATE_VERSION`, read from `dotlottie-rs/Cargo.toml`
- `COMMIT_HASH`, the current git commit

### Targets

Run `make help` for the full list.

#### Platform targets

- `make android`: all Android architectures (ARM64, x86_64, x86, ARMv7)
- `make apple`: all Apple platforms (macOS, iOS, tvOS, visionOS, watchOS, Mac Catalyst)
- `make wasm`: WASM with the software renderer
- `make wasm-webgl`: WASM with the WebGL2 renderer
- `make wasm-webgpu`: WASM with the WebGPU renderer
- `make wasm-all`: all WASM variants
- `make native`: native library for the host
- `make native-opengl`: native library with the OpenGL renderer
- `make native-webgpu`: native library with the WebGPU renderer
- `make linux`: Linux x86_64 and ARM64
- `make windows`: Windows x86_64 and ARM64

#### Architecture targets

- Android: `android-aarch64`, `android-x86_64`, `android-x86`, `android-armv7`
- Apple: `apple-macos-arm64`, `apple-ios-arm64`, `apple-tvos-sim-arm64`, and the rest of `apple-{platform}-{arch}`
- Linux: `linux-x86_64`, `linux-arm64`
- Windows: `windows-x86_64`, `windows-arm64`

#### Development targets

- `make test` runs `cargo test` with `--test-threads=1`.
- `make clippy` runs clippy.
- `make help` prints the target list.

### Dependencies

#### WASM

- `make wasm-setup` installs wasm-bindgen-cli at the version of the `wasm-bindgen` crate in Cargo.lock.
- `make wasm-setup` also adds the `wasm32-unknown-unknown` target through rustup.
- `wasm-opt` is optional. The build runs it with `-O3` if it is on `PATH`.

#### Android

- Install the Android NDK yourself, r28 or newer.
- `make android-setup` installs the Rust targets.

#### Windows

- Visual Studio Build Tools with the "Desktop development with C++" workload
- GNU Make, from MSYS2, Git Bash, or `choco install make`
- `make windows-setup` installs the Rust targets.

#### Apple

- Xcode
- `make apple-setup` installs the Rust targets.
- A nightly toolchain for Mac Catalyst, tvOS, visionOS, and watchOS

### Setup and cleanup

- `make setup` runs every `{platform}-setup` target.
- `make {platform}-setup` sets up one platform.
- `make list-platforms` lists the supported platforms.
- `make clean` runs `native-clean`, runs `cargo clean`, and removes the bindings directory.
- `make {platform}-clean` removes one platform's artifacts.
