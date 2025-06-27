# dotlottie-ffi Makefile
# Main build orchestrator for dotlottie-ffi across multiple platforms

# Variables that can be overridden
BINDINGS_DIR ?= bindings

.PHONY: all clean help list-platforms

# Default target - MUST be defined before includes to ensure it's the first target
all: help

# Include platform-specific makefiles
include make/android.mk
include make/apple.mk
include make/linux.mk
include make/wasm.mk
include make/windows.mk

# Main help menu
help:
	@echo "dotlottie-ffi Build System"
	@echo "=========================="
	@echo ""
	@echo "Platform Help Commands:"
	@echo "======================="
	@echo "  make android-help                                  - Show Android-specific help"
	@echo "  make apple-help                                    - Show Apple-specific help"
	@echo "  make linux-help                                    - Show Linux-specific help"
	@echo "  make wasm-help                                     - Show WASM-specific help"
	@echo "  make windows-help                                  - Show Windows-specific help"
	@echo ""
	@echo "Quick Build Commands:"
	@echo "===================="
	@echo "  make android                						- Build all Android targets"
	@echo "  make apple                                         - Build all Apple targets"
	@echo "  make linux                                         - Build all Linux targets"
	@echo "  make wasm                                          - Build WASM module"
	@echo "  make windows                                       - Build all Windows targets"
	@echo ""
	@echo "Android Targets:"
	@echo "==============="
	@echo "  make android-aarch64      							- Build Android ARM64"
	@echo "  make android-x86_64        						- Build Android x86_64"
	@echo "  make android-x86           						- Build Android x86"
	@echo "  make android-armv7         						- Build Android ARMv7"
	@echo "  make install-android-targets                       - Install Android Rust targets"
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
	@echo "  make install-apple-targets                        - Install Apple Rust targets"
	@echo ""
	@echo "Linux Targets:"
	@echo "=============="
	@echo "  make linux-x86_64                                 - Build Linux x86_64"
	@echo "  make linux-aarch64                                - Build Linux aarch64"
	@echo "  make install-linux-targets                        - Install Linux Rust targets"
	@echo "  make cpp-bindings                                 - Generate C++ bindings"
	@echo ""
	@echo "WASM Targets:"
	@echo "============="
	@echo "  make wasm-install-emsdk                           - Install specific emsdk version"
	@echo "  make wasm-clean                                   - Clean WASM build artifacts"
	@echo "  make install-wasm-targets                         - Install WASM Rust targets and nightly"
	@echo ""
	@echo "Environment Info Commands:"
	@echo "========================="
	@echo "  make android-env-info                             - Show Android environment info"
	@echo "  make apple-env-info                               - Show Apple environment info"
	@echo "  make linux-env-info                               - Show Linux environment info"
	@echo "  make wasm-env-info                                - Show WASM environment info"
	@echo "  make windows-env-info                             - Show Windows environment info"
	@echo ""
	@echo "Windows Targets:"
	@echo "================"
	@echo "  make windows-x86_64-msvc                          - Build Windows x86_64 MSVC"
	@echo "  make windows-i686-msvc                            - Build Windows i686 MSVC"
	@echo "  make windows-aarch64-msvc                         - Build Windows aarch64 MSVC"
	@echo "  make windows-x86_64-gnu                           - Build Windows x86_64 GNU"
	@echo "  make install-windows-targets                      - Install Windows Rust targets"
	@echo ""
	@echo "Utility Commands:"
	@echo "=================="
	@echo "  make clean                                         - Clean all build artifacts"
	@echo "  make list-platforms                               - List all supported platforms"
	@echo ""
	@echo ""
	@echo "Examples:"
	@echo "========="
	@echo "  make android
	@echo "  make wasm-setup && make wasm"
	@echo "  make apple-ios"
	@echo "  make linux-x86_64"

# List all supported platforms
list-platforms:
	@echo "Supported Platforms:"
	@echo "==================="
	@echo "  android     - Android (ARM64, x86_64, x86, ARMv7)"
	@echo "  apple       - Apple (macOS, iOS, visionOS, tvOS)"
	@echo "  linux       - Linux (x86_64, aarch64)"
	@echo "  wasm        - WebAssembly (Emscripten)"
	@echo "  windows     - Windows (x86_64-msvc, i686-msvc, aarch64-msvc, x86_64-gnu)"
	@echo ""

# Clean all build artifacts
clean:
	@echo "Cleaning all build artifacts..."
	cargo clean
	rm -rf $(BINDINGS_DIR)
	@echo "Clean complete."

# Legacy/convenience targets
env-info: android-env-info
