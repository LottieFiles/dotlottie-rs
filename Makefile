# dotlottie-ffi Makefile
# Main build orchestrator for dotlottie-ffi across multiple platforms

# Variables that can be overridden
BINDINGS_DIR ?= dotlottie-ffi/uniffi_bindings

.PHONY: all clean help list-platforms

# Default target - MUST be defined before includes to ensure it's the first target
all: help

# Include platform-specific makefiles
include make/android.mk
include make/apple.mk
include make/linux.mk
include make/wasm.mk

# Main help menu
help:
	@echo "dotlottie-ffi Build System"
	@echo "=========================="
	@echo ""
	@echo "Platform Build Targets:"
	@echo "======================="
	@echo "  make android                                      - Build all Android targets"
	@echo "  make apple                                        - Build all Apple targets" 
	@echo "  make linux                                        - Build all Linux targets"
	@echo "  make wasm                                         - Build WASM module"
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
	@echo "  make apple-macos-arm64                            - Build macOS ARM64"
	@echo "  make apple-macos-x86_64                           - Build macOS x86_64"
	@echo "  make apple-ios-arm64                              - Build iOS ARM64"
	@echo "  make apple-ios-x86_64                             - Build iOS x86_64"
	@echo "  make apple-ios-sim-arm64                          - Build iOS Simulator ARM64"
	@echo "  make apple-visionos-arm64                         - Build visionOS ARM64"
	@echo "  make apple-visionos-sim-arm64                     - Build visionOS Simulator ARM64"
	@echo "  make apple-tvos-arm64                             - Build tvOS ARM64"
	@echo "  make apple-tvos-sim-arm64                         - Build tvOS Simulator ARM64"
	@echo ""
	@echo "Linux Targets:"
	@echo "=============="
	@echo "  make linux-x86_64                                 - Build Linux x86_64"
	@echo "  make linux-aarch64                                - Build Linux aarch64"
	@echo ""
	@echo "WASM Setup Targets:"
	@echo "=================="
	@echo "  make wasm-install-emsdk                           - Install and setup Emscripten SDK"
	@echo ""
	@echo "Install Targets:"
	@echo "================"
	@echo "  make install-android-targets                      - Install Android Rust targets"
	@echo "  make install-apple-targets                        - Install Apple Rust targets"
	@echo "  make install-linux-targets                        - Install Linux Rust targets"
	@echo "  make install-wasm-targets                         - Install WASM Rust targets"
	@echo ""
	@echo "Clean Targets:"
	@echo "=============="
	@echo "  make clean                                        - Clean all build artifacts"
	@echo "  make android-clean                                - Clean Android artifacts"
	@echo "  make apple-clean                                  - Clean Apple artifacts"
	@echo "  make linux-clean                                  - Clean Linux artifacts"
	@echo "  make wasm-clean                                   - Clean WASM artifacts"

# List all supported platforms
list-platforms:
	@echo "Supported Platforms:"
	@echo "==================="
	@echo "  android     - Android (ARM64, x86_64, x86, ARMv7)"
	@echo "  apple       - Apple (macOS, iOS, visionOS, tvOS)"
	@echo "  linux       - Linux (x86_64, aarch64)"
	@echo "  wasm        - WebAssembly (Emscripten)"
	@echo ""

# Clean all build artifacts
clean:
	@echo "Cleaning all build artifacts..."
	cargo clean --manifest-path dotlottie-ffi/Cargo.toml
	rm -rf $(BINDINGS_DIR)
	@echo "Clean complete." 