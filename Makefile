# dotlottie-ffi Makefile
# Main build orchestrator for dotlottie-ffi across multiple platforms

# Variables that can be overridden
BINDINGS_DIR ?= dotlottie-ffi/uniffi-bindings

.PHONY: all clean help list-platforms test clippy native native-clean

# Default target - MUST be defined before includes to ensure it's the first target
all: help

# Include platform-specific makefiles
include make/android.mk
include make/apple.mk
include make/wasm.mk
include make/linux.mk

# Main help menu
help:
	@echo "dotlottie-ffi Build System"
	@echo "=========================="
	@echo ""
	@echo "Platform Build Targets:"
	@echo "======================="
	@echo "  make android                                      - Build all Android targets"
	@echo "  make apple                                        - Build all Apple targets" 
	@echo "  make wasm                                         - Build WASM module"
	@echo "  make linux                                        - Build all Linux targets"
	@echo "  make native                                       - Build native (current platform)"
	@echo ""
	@echo "Android Targets:"
	@echo "==============="
	@echo "  make android-aarch64                              - Build Android ARM64"
	@echo "  make android-x86_64                               - Build Android x86_64"
	@echo "  make android-x86                                  - Build Android x86"
	@echo "  make android-armv7                                - Build Android ARMv7"
	@echo ""
	@echo "Apple Targets:"
	@echo "=============="
	@echo "  make apple-macos                                  - Build all macOS targets"
	@echo "  make apple-ios                                    - Build all iOS targets"
	@echo "  make apple-visionos                               - Build all visionOS targets"
	@echo "  make apple-tvos                                   - Build all tvOS targets"
	@echo "  make apple-maccatalyst                            - Build all macCatalyst targets"
	@echo "  make apple-macos-arm64                            - Build macOS ARM64"
	@echo "  make apple-macos-x86_64                           - Build macOS x86_64"
	@echo "  make apple-ios-arm64                              - Build iOS ARM64"
	@echo "  make apple-ios-x86_64                             - Build iOS x86_64"
	@echo "  make apple-ios-sim-arm64                          - Build iOS Simulator ARM64"
	@echo "  make apple-visionos-arm64                         - Build visionOS ARM64"
	@echo "  make apple-visionos-sim-arm64                     - Build visionOS Simulator ARM64"
	@echo "  make apple-tvos-arm64                             - Build tvOS ARM64"
	@echo "  make apple-tvos-sim-arm64                         - Build tvOS Simulator ARM64"
	@echo "  make apple-maccatalyst-arm64                      - Build macCatalyst ARM64"
	@echo "  make apple-maccatalyst-x86_64                     - Build macCatalyst x86_64"
	@echo ""
	@echo "Linux Targets:"
	@echo "=============="
	@echo "  make linux-x86_64                                  - Build Linux x86_64"
	@echo "  make linux-arm64                                    - Build Linux ARM64"
	@echo ""
	@echo "Setup Targets:"
	@echo "=============="
	@echo "  make setup                                        - Setup all platforms"
	@echo "  make android-setup                                - Setup Android environment"
	@echo "  make apple-setup                                  - Setup Apple environment"
	@echo "  make wasm-setup                                   - Setup WASM environment"
	@echo "  make linux-setup                                  - Setup Linux environment"
	@echo ""
	@echo "Clean Targets:"
	@echo "=============="
	@echo "  make clean                                        - Clean all build artifacts"
	@echo "  make android-clean                                - Clean Android artifacts"
	@echo "  make apple-clean                                  - Clean Apple artifacts"
	@echo "  make wasm-clean                                   - Clean WASM artifacts"
	@echo "  make linux-clean                                  - Clean Linux artifacts"
	@echo "  make native-clean                                 - Clean Native artifacts"
	@echo ""
# List all supported platforms
list-platforms:
	@echo "Supported Platforms:"
	@echo "==================="
	@echo "  android     - Android (ARM64, x86_64, x86, ARMv7)"
	@echo "  apple       - Apple (macOS, iOS, visionOS, tvOS, macCatalyst)"
	@echo "  wasm        - WebAssembly (Emscripten)"
	@echo "  linux       - Linux (x86_64, ARM64)"
	@echo "  native      - Native (current platform)"
	@echo ""

# Setup all platforms
setup: android-setup apple-setup wasm-setup linux-setup
	@echo "✓ All platform setup complete"

# Clean all build artifacts
clean: native-clean
	@echo "Cleaning all build artifacts..."
	cargo clean --manifest-path dotlottie-ffi/Cargo.toml
	cargo clean --manifest-path dotlottie-rs/Cargo.toml
	rm -rf $(BINDINGS_DIR)
	@echo "Clean complete."

# Run tests
test:
	cargo test --manifest-path dotlottie-rs/Cargo.toml -- --test-threads=1
	cargo test --manifest-path dotlottie-ffi/Cargo.toml -- --test-threads=1

# Run clippy
clippy:
	cargo clippy --manifest-path dotlottie-rs/Cargo.toml --all-targets -- -D clippy::print_stdout
	cargo clippy --manifest-path dotlottie-ffi/Cargo.toml --all-targets -- -D clippy::print_stdout

# Native build variables
NATIVE = native
RELEASE = release
RUNTIME_FFI = dotlottie-ffi
DOTLOTTIE_PLAYER = dotlottie-player
RUNTIME_FFI_HEADER = dotlottie_player.h
NATIVE_FEATURES = ffi,tvg,tvg-sw,tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-threads,tvg-lottie-expressions

DOTLOTTIE_PLAYER_NATIVE_RELEASE_DIR = $(RELEASE)/$(NATIVE)/$(DOTLOTTIE_PLAYER)
DOTLOTTIE_PLAYER_NATIVE_RELEASE_INCLUDE_DIR = $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_DIR)/include
DOTLOTTIE_PLAYER_NATIVE_RELEASE_LIB_DIR = $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_DIR)/lib

# Native release function
define NATIVE_RELEASE
	rm -rf $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_DIR)
	mkdir -p $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_INCLUDE_DIR) $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_LIB_DIR)
	cp $(RUNTIME_FFI)/bindings.h $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_INCLUDE_DIR)/$(RUNTIME_FFI_HEADER)
	find $(RUNTIME_FFI)/target/release/ -maxdepth 1 \( -name '*.so' -or -name '*.dylib' -or -name "*.dll" \) \
		-exec cp {} $(DOTLOTTIE_PLAYER_NATIVE_RELEASE_LIB_DIR) \;
endef

# Build native libraries for the current platform
native:
	@echo "Building native libraries for current platform..."
	cargo build --manifest-path $(RUNTIME_FFI)/Cargo.toml --features $(NATIVE_FEATURES) --release
	$(NATIVE_RELEASE)
	@echo "✓ Native build complete. Artifacts available in $(RELEASE)/$(NATIVE)/"

# Native NEW build variables (using dotlottie-rs c_api)
NATIVE_NEW_FEATURES = tvg,tvg-sw,c_api
NATIVE_NEW_RELEASE_DIR = $(RELEASE)/native-new
NATIVE_NEW_LIB_DIR = $(NATIVE_NEW_RELEASE_DIR)/lib
NATIVE_NEW_INCLUDE_DIR = $(NATIVE_NEW_RELEASE_DIR)/include

# Build native libraries using dotlottie-rs c_api
native-new:
	@echo "Building native libraries with dotlottie-rs c_api..."
	cargo build --manifest-path dotlottie-rs/Cargo.toml --features $(NATIVE_NEW_FEATURES) --release
	@mkdir -p $(NATIVE_NEW_LIB_DIR) $(NATIVE_NEW_INCLUDE_DIR)
	@find dotlottie-rs/target/release/ -maxdepth 1 \( -name 'libdotlottie_rs.so' -or -name 'libdotlottie_rs.dylib' -or -name 'dotlottie_rs.dll' \) -exec cp {} $(NATIVE_NEW_LIB_DIR)/ \;
	@cbindgen --config dotlottie-rs/cbindgen.toml --crate dotlottie-rs --output $(NATIVE_NEW_INCLUDE_DIR)/dotlottie_player.h dotlottie-rs
	@echo "✓ Native NEW build complete. Artifacts available in $(NATIVE_NEW_RELEASE_DIR)/"
	@echo "   Library: $(NATIVE_NEW_LIB_DIR)/"
	@echo "   Header:  $(NATIVE_NEW_INCLUDE_DIR)/dotlottie_player.h"

# Clean native artifacts
native-clean:
	@echo "Cleaning native artifacts..."
	rm -rf $(RELEASE)/$(NATIVE)
	@echo "✓ Native clean complete"
