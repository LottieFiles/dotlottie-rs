BINDINGS_DIR ?= dotlottie-rs/build

.PHONY: all help list-platforms setup clean test clippy fmt native native-opengl native-webgpu native-clean

# Must come before the includes so that it is the default target.
all: help

include make/android.mk
include make/apple.mk
include make/wasm.mk
include make/linux.mk
include make/windows.mk

help:
	@echo "dotlottie-rs build targets"
	@echo ""
	@echo "Platform targets:"
	@echo "  make android                    Build all Android architectures"
	@echo "  make apple                      Build all Apple platforms and the XCFramework"
	@echo "  make wasm                       Build the WASM module (software renderer)"
	@echo "  make linux                      Build Linux x86_64 and ARM64"
	@echo "  make windows                    Build Windows x86_64 and ARM64"
	@echo "  make native                     Build the native library for this host"
	@echo "  make native-opengl              Build the native library with the OpenGL renderer"
	@echo "  make native-webgpu              Build the native library with the WebGPU renderer"
	@echo ""
	@echo "WASM targets:"
	@echo "  make wasm                       Software renderer"
	@echo "  make wasm-webgl                 WebGL2 renderer"
	@echo "  make wasm-webgpu                WebGPU renderer"
	@echo "  make wasm-all                   All three variants"
	@echo ""
	@echo "Android targets:"
	@echo "  make android-aarch64            ARM64"
	@echo "  make android-x86_64             x86_64"
	@echo "  make android-x86                x86"
	@echo "  make android-armv7              ARMv7"
	@echo ""
	@echo "Apple targets:"
	@echo "  make apple-macos                All macOS architectures"
	@echo "  make apple-ios                  All iOS device and simulator architectures"
	@echo "  make apple-maccatalyst          All Mac Catalyst architectures"
	@echo "  make apple-visionos             All visionOS architectures"
	@echo "  make apple-tvos                 All tvOS architectures"
	@echo "  make apple-watchos              All watchOS architectures"
	@echo "  make apple-macos-arm64          macOS ARM64"
	@echo "  make apple-macos-x86_64         macOS x86_64"
	@echo "  make apple-ios-arm64            iOS ARM64"
	@echo "  make apple-ios-x86_64           iOS simulator x86_64"
	@echo "  make apple-ios-sim-arm64        iOS simulator ARM64"
	@echo "  make apple-maccatalyst-arm64    Mac Catalyst ARM64"
	@echo "  make apple-maccatalyst-x86_64   Mac Catalyst x86_64"
	@echo "  make apple-visionos-arm64       visionOS ARM64"
	@echo "  make apple-visionos-sim-arm64   visionOS simulator ARM64"
	@echo "  make apple-tvos-arm64           tvOS ARM64"
	@echo "  make apple-tvos-sim-arm64       tvOS simulator ARM64"
	@echo "  make apple-watchos-arm64        watchOS ARM64"
	@echo "  make apple-watchos-arm64_32     watchOS arm64_32"
	@echo "  make apple-watchos-armv7k       watchOS armv7k"
	@echo "  make apple-watchos-sim-arm64    watchOS simulator ARM64"
	@echo ""
	@echo "Linux targets:"
	@echo "  make linux-x86_64               x86_64"
	@echo "  make linux-arm64                ARM64"
	@echo ""
	@echo "Windows targets:"
	@echo "  make windows-x86_64             x86_64"
	@echo "  make windows-arm64              ARM64"
	@echo ""
	@echo "Setup targets:"
	@echo "  make setup                      Set up every platform"
	@echo "  make android-setup              Install the Android Rust targets"
	@echo "  make apple-setup                Install the Apple Rust targets"
	@echo "  make wasm-setup                 Install the wasm32 target and wasm-bindgen-cli"
	@echo "  make linux-setup                Install the Linux Rust targets"
	@echo "  make windows-setup              Install the Windows Rust targets and MSVC tools"
	@echo ""
	@echo "Development targets:"
	@echo "  make test                       Run the tests on one thread"
	@echo "  make clippy                     Run clippy on all targets"
	@echo "  make fmt                        Run rustfmt"
	@echo "  make list-platforms             List the supported platforms"
	@echo ""
	@echo "Clean targets:"
	@echo "  make clean                      Remove all build artifacts and the Cargo target directory"
	@echo "  make android-clean              Remove Android artifacts"
	@echo "  make apple-clean                Remove Apple artifacts"
	@echo "  make wasm-clean                 Remove WASM artifacts"
	@echo "  make linux-clean                Remove Linux artifacts"
	@echo "  make windows-clean              Remove Windows artifacts"
	@echo "  make native-clean               Remove native artifacts"
	@echo ""

list-platforms:
	@echo "Supported platforms:"
	@echo "  android     Android (ARM64, x86_64, x86, ARMv7)"
	@echo "  apple       Apple (macOS, iOS, tvOS, visionOS, watchOS, Mac Catalyst)"
	@echo "  wasm        WebAssembly (wasm-bindgen)"
	@echo "  linux       Linux (x86_64, ARM64)"
	@echo "  windows     Windows (x86_64, ARM64)"
	@echo "  native      Native (this host)"
	@echo ""

setup: android-setup apple-setup wasm-setup linux-setup windows-setup
	@echo "✓ All platform setup complete"

clean: native-clean
	@echo "Cleaning all build artifacts..."
	cargo clean --manifest-path dotlottie-rs/Cargo.toml
	rm -rf $(BINDINGS_DIR)
	@echo "✓ Clean complete"

test:
	cargo test --manifest-path dotlottie-rs/Cargo.toml -- --test-threads=1

clippy:
	cargo clippy --manifest-path dotlottie-rs/Cargo.toml --all-targets -- -D clippy::print_stdout

fmt:
	cargo fmt --manifest-path dotlottie-rs/Cargo.toml

NATIVE = native
RELEASE = release
DOTLOTTIE_ROOT = dotlottie-rs
RELEASE_FILE_NAME = dotlottie_player
RUNTIME_HEADER = $(RELEASE_FILE_NAME).h
BUILD_DIR = $(DOTLOTTIE_ROOT)/build
DOTLOTTIE_PLAYER = dotlottie-player

DOTLOTTIE_PLAYER_NATIVE_RELEASE_DIR = $(RELEASE)/$(NATIVE)/$(DOTLOTTIE_PLAYER)
DOTLOTTIE_PLAYER_NATIVE_RELEASE_INCLUDE_DIR = $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_DIR)/include
DOTLOTTIE_PLAYER_NATIVE_RELEASE_LIB_DIR = $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_DIR)/lib

NATIVE_FEATURES = tvg,tvg-cpu,c_api

define NATIVE_RELEASE
	rm -rf $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_DIR)
	mkdir -p $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_INCLUDE_DIR) $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_LIB_DIR)
	cp $(BUILD_DIR)/$(RUNTIME_HEADER) $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_INCLUDE_DIR)/$(RUNTIME_HEADER)
	find $(DOTLOTTIE_ROOT)/target/release/ -maxdepth 1 \( -name '*.so' -or -name '*.dylib' -or -name "*.dll" \) \
		-exec sh -c 'cp "$$1" "$(DOTLOTTIE_PLAYER_NATIVE_RELEASE_LIB_DIR)/lib$(RELEASE_FILE_NAME).$${1##*.}"' _ {} \;
	@echo "   Library: $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_LIB_DIR)/"
	@echo "   Header:  $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_INCLUDE_DIR)/$(RUNTIME_HEADER)"
endef

native-opengl:
	@echo "→ Building native library with the OpenGL renderer..."
	cargo rustc --manifest-path $(DOTLOTTIE_ROOT)/Cargo.toml --crate-type cdylib --features c_api,tvg,tvg-gl,tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-threads,tvg-lottie-expressions --release
	$(NATIVE_RELEASE)
	@echo "✓ Native OpenGL build complete"

native-webgpu:
	@echo "→ Building native library with the WebGPU renderer..."
	cargo rustc --manifest-path $(DOTLOTTIE_ROOT)/Cargo.toml --crate-type cdylib --features c_api,tvg,tvg-wg,tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-threads,tvg-lottie-expressions --release
	$(NATIVE_RELEASE)
	@echo "✓ Native WebGPU build complete"

native:
	@echo "→ Building native library..."
ifeq ($(OS),Windows_NT)
	$(call WINDOWS_CARGO_BUILD,x64,x86_64-pc-windows-msvc,$(NATIVE_FEATURES),)
else
	cargo rustc --manifest-path $(DOTLOTTIE_ROOT)/Cargo.toml --crate-type cdylib --features $(NATIVE_FEATURES) --release
endif
	$(NATIVE_RELEASE)
	@echo "✓ Native build complete"

native-clean:
	@echo "Cleaning native artifacts..."
	rm -rf $(RELEASE)/$(NATIVE)
	@echo "✓ Native clean complete"
