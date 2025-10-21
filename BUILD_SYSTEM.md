## Build System

The build system uses GNU `make` to build all artifacts for `android`, `apple`, `wasm`, and `native` platforms.
The system is modular with a main orchestrator `Makefile` and platform-specific makefiles in the `make/` directory.
This documentation provides implementation details to help understand and extend the build system.

The build process works as follows:

1. **Platform Setup**: Initialize and configure build tools for each target platform (NDK for Android, Xcode for Apple, emsdk for WASM)
2. **UniFFI Bindings Generation**: Generate language-specific bindings (Kotlin for Android, Swift for Apple, C++ for WASM)
3. **Rust Library Build**: Build the `dotlottie-ffi` library for each target architecture using platform-specific toolchains
4. **Platform-Specific Packaging**: Create platform-appropriate release artifacts (AAR for Android, XCFramework for Apple, JS/WASM modules for Web)
5. **Release Assembly**: Package all artifacts into the `release/` directory with version information

### Platform-Specific Makefiles

The build system is organized into modular makefiles, each handling a specific platform:

- **Main Makefile**: Orchestrates all builds and provides help/setup targets
- **make/android.mk**: Handles Android builds across multiple architectures (ARM64, x86_64, x86, ARMv7)
- **make/apple.mk**: Manages Apple platform builds (macOS, iOS, tvOS, visionOS, macCatalyst)
- **make/wasm.mk**: Controls WebAssembly builds using Emscripten

Each platform makefile is responsible for:

- Platform-specific environment setup and validation
- UniFFI bindings generation for the target language
- Cross-compilation configuration
- Release artifact packaging

#### Android Build System (`make/android.mk`)

The Android build system supports four architectures: ARM64, x86_64, x86, and ARMv7. Key features include:

- **NDK Integration**: Automatically detects and validates Android NDK installation
- **Kotlin Bindings**: Generates UniFFI Kotlin bindings for Android integration
- **Multi-Architecture Support**: Builds for all Android architectures in parallel
- **Packaging**: Creates Android-ready package structure with JNI libraries and shared dependencies
- **Version Management**: Includes build version and commit hash in release artifacts

Key targets:

- `android`: Builds all Android architectures and packages the release
- `android-{arch}`: Builds specific architecture (e.g., `android-aarch64`)
- `android-setup`: Installs required Rust targets
- `android-clean`: Cleans Android-specific build artifacts

#### Apple Build System (`make/apple.mk`)

The Apple build system handles multiple Apple platforms with comprehensive framework generation:

- **Platform Support**: macOS, iOS, tvOS, visionOS, and macCatalyst
- **Swift Bindings**: Automatic generation of Swift UniFFI bindings
- **Framework Creation**: Builds individual frameworks and combines them into XCFramework
- **Universal Binaries**: Creates universal binaries using `lipo` for multi-architecture support
- **Code Signing**: Optional code signing support for distribution

Key targets:

- `apple`: Builds all Apple platforms and creates XCFramework
- `apple-{platform}`: Builds specific platform (e.g., `apple-ios`)
- `apple-{platform}-{arch}`: Builds specific architecture (e.g., `apple-macos-arm64`)
- `apple-setup`: Installs required Rust targets and toolchain components

#### WASM Build System (`make/wasm.mk`)

The WASM build system creates WebAssembly modules for web deployment:

- **Emscripten Integration**: Uses Emscripten SDK for WASM compilation
- **C++ Bindings**: Generates and compiles UniFFI C++ bindings
- **TypeScript Support**: Generates TypeScript definition files
- **Optimization**: Aggressive size optimization with LTO and closure compiler
- **Self-Contained**: Manages emsdk submodule automatically

Key targets:

- `wasm`: Builds complete WASM module with TypeScript definitions
- `wasm-setup`: Installs emsdk, Rust nightly, and uniffi-bindgen-cpp
- `wasm-clean`: Cleans WASM-specific build artifacts

#### Native Build System

For local development and testing, the native build system creates C libraries:

- **FFI Library**: Builds `dotlottie-ffi` as a dynamic library
- **C Headers**: Generates cbindgen-compatible headers
- **Platform Detection**: Automatically builds for the current platform

Key targets:

- `native`: Builds native library for current platform
- `native-clean`: Cleans native build artifacts

### Build Features and Configuration

The build system includes several configurable features:

#### Feature Flags

All platforms support configurable Rust features through the `FEATURES` variable:

- `tvg-webp`: WebP image format support
- `tvg-png`: PNG image format support
- `tvg-jpg`: JPEG image format support
- `tvg-ttf`: TrueType font support
- `tvg-lottie-expressions`: Lottie expression evaluation support

Default features include:

- `tvg`: ThorVG renderer backend
- `tvg-sw`: Software rendering backend
- `uniffi`: UniFFI bindings support

#### Environment Variables

Platform-specific environment variables can be overridden:

**Android:**

- `ANDROID_NDK_HOME`: Path to Android NDK (default: `/opt/homebrew/share/android-ndk`)
- `API_LEVEL`: Android API level (default: `21`)

**Apple:**

- `XCODE_PATH`: Path to Xcode installation
- `MIN_IOS_VERSION`, `MIN_MACOS_VERSION`, etc.: Minimum OS versions

**WASM:**

- `EMSDK_VERSION`: Emscripten SDK version (default: `3.1.74`)

#### Version Management

All builds include version information from:

- `CRATE_VERSION`: Extracted from `dotlottie-ffi/Cargo.toml`
- `COMMIT_HASH`: Current git commit hash

This information is embedded in release artifacts for traceability.

### Available Build Targets

The build system provides a comprehensive set of targets accessible via `make help`:

#### Platform Targets

- `make android`: Build all Android architectures (ARM64, x86_64, x86, ARMv7)
- `make apple`: Build all Apple platforms (macOS, iOS, tvOS, visionOS, macCatalyst)
- `make wasm`: Build WebAssembly module with TypeScript definitions
- `make native`: Build native library for current platform

#### Architecture-Specific Targets

- **Android**: `android-aarch64`, `android-x86_64`, `android-x86`, `android-armv7`
- **Apple**: `apple-macos-arm64`, `apple-ios-arm64`, `apple-tvos-sim-arm64`, etc.

#### Development Targets

- `make test`: Run all tests with single-threaded execution
- `make clippy`: Run Rust linter with strict settings
- `make help`: Display comprehensive help menu

### Platform Setup and Dependencies

The build system automatically manages platform-specific dependencies:

#### WASM Dependencies

- **emsdk submodule**: Automatically initialized and configured for WASM builds
- **uniffi-bindgen-cpp**: Installed via `make wasm-setup` for C++ binding generation
- **Node.js dependencies**: TypeScript compiler installed within emsdk environment

#### Android Dependencies

- **Android NDK**: Must be installed separately (minimum version r28)
- **Rust targets**: Automatically installed via `make android-setup`

#### Apple Dependencies

- **Xcode**: Required for all Apple platform builds
- **Rust targets**: Multiple targets installed via `make apple-setup`
- **Nightly toolchain**: Required for newer Apple platforms (visionOS, tvOS)

### Build Management

#### Setup Commands

- `make setup`: Configures all platforms
- `make {platform}-setup`: Configures specific platform
- `make list-platforms`: Shows all supported platforms

#### Incremental Builds

The build system supports efficient incremental builds:

- **Cargo caching**: Rust builds leverage Cargo's incremental compilation
- **Platform isolation**: Each platform builds independently
- **Artifact reuse**: Previously built binaries are reused when possible

#### Cleanup Commands

Different cleanup levels are available:

- `make clean`: Removes all build artifacts and Cargo cache
- `make {platform}-clean`: Cleans specific platform artifacts
- `make native-clean`: Cleans only native build artifacts

The modular design means you can clean and rebuild individual platforms without affecting others.
