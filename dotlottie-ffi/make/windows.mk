# Windows build configuration for dotlottie-ffi

# Tool configuration
UNIFFI_BINDGEN_CPP ?= uniffi-bindgen-cpp

# Default Rust features for Windows builds
WINDOWS_FEATURES ?= uniffi,thorvg,thorvg_webp,thorvg_png,thorvg_jpg,thorvg_ttf,thorvg_lottie_expressions

# UniFFI Bindings
BINDINGS_DIR ?= bindings
CPP_BINDINGS_DIR ?= $(BINDINGS_DIR)/cpp

# Release and packaging variables
RELEASE_DIR ?= release
WINDOWS_RELEASE_DIR ?= $(RELEASE_DIR)/windows
DOTLOTTIE_PLAYER_DIR ?= dotlottie-player
DOTLOTTIE_PLAYER_WINDOWS_RELEASE_DIR ?= $(WINDOWS_RELEASE_DIR)/$(DOTLOTTIE_PLAYER_DIR)
DOTLOTTIE_PLAYER_WINDOWS_INCLUDE_DIR ?= $(DOTLOTTIE_PLAYER_WINDOWS_RELEASE_DIR)/include
DOTLOTTIE_PLAYER_WINDOWS_LIB_DIR ?= $(DOTLOTTIE_PLAYER_WINDOWS_RELEASE_DIR)/lib

# Library names and paths
RUNTIME_FFI_LIB_BASE ?= dotlottie_player
RUNTIME_FFI_DLL := $(RUNTIME_FFI_LIB_BASE).dll
RUNTIME_FFI_LIB := $(RUNTIME_FFI_LIB_BASE).lib
RUNTIME_FFI_PDB := $(RUNTIME_FFI_LIB_BASE).pdb

# Get version information
CRATE_VERSION := $(shell grep -m 1 version Cargo.toml | sed 's/.*"\([0-9.]\+\)"/\1/')
COMMIT_HASH := $(shell git rev-parse --short HEAD)

# Windows targets
WINDOWS_TARGETS = x86_64-pc-windows-msvc i686-pc-windows-msvc aarch64-pc-windows-msvc x86_64-pc-windows-gnu

# Windows target mapping
WINDOWS_TARGET_x86_64_msvc = x86_64-pc-windows-msvc
WINDOWS_TARGET_i686_msvc = i686-pc-windows-msvc
WINDOWS_TARGET_aarch64_msvc = aarch64-pc-windows-msvc
WINDOWS_TARGET_x86_64_gnu = x86_64-pc-windows-gnu

# Windows architecture mapping for packaging
WINDOWS_ARCH_x86_64_msvc = x86_64-msvc
WINDOWS_ARCH_i686_msvc = i686-msvc
WINDOWS_ARCH_aarch64_msvc = aarch64-msvc
WINDOWS_ARCH_x86_64_gnu = x86_64-gnu

# Windows packaging function
define WINDOWS_PACKAGE_ARCH
	@echo "Packaging Windows $(1) build..."
	@mkdir -p $(DOTLOTTIE_PLAYER_WINDOWS_INCLUDE_DIR)/$(WINDOWS_ARCH_$(1))
	@mkdir -p $(DOTLOTTIE_PLAYER_WINDOWS_LIB_DIR)/$(WINDOWS_ARCH_$(1))
	
	# Copy C++ bindings headers
	@if [ -d "$(CPP_BINDINGS_DIR)" ] && [ -n "$$(ls -A $(CPP_BINDINGS_DIR)/*.hpp 2>/dev/null || true)" ]; then \
		cp $(CPP_BINDINGS_DIR)/*.hpp $(DOTLOTTIE_PLAYER_WINDOWS_INCLUDE_DIR)/$(WINDOWS_ARCH_$(1))/; \
		echo "Copied C++ header files to $(DOTLOTTIE_PLAYER_WINDOWS_INCLUDE_DIR)/$(WINDOWS_ARCH_$(1))"; \
	fi
	
	# Copy main library files
	@if [ -f "target/$(WINDOWS_TARGET_$(1))/release/$(RUNTIME_FFI_DLL)" ]; then \
		cp target/$(WINDOWS_TARGET_$(1))/release/$(RUNTIME_FFI_DLL) \
		   $(DOTLOTTIE_PLAYER_WINDOWS_LIB_DIR)/$(WINDOWS_ARCH_$(1))/$(RUNTIME_FFI_DLL); \
		echo "Copied DLL for $(1)"; \
	else \
		echo "Warning: $(RUNTIME_FFI_DLL) not found in target/$(WINDOWS_TARGET_$(1))/release/"; \
	fi
	
	@if [ -f "target/$(WINDOWS_TARGET_$(1))/release/$(RUNTIME_FFI_LIB)" ]; then \
		cp target/$(WINDOWS_TARGET_$(1))/release/$(RUNTIME_FFI_LIB) \
		   $(DOTLOTTIE_PLAYER_WINDOWS_LIB_DIR)/$(WINDOWS_ARCH_$(1))/$(RUNTIME_FFI_LIB); \
		echo "Copied LIB for $(1)"; \
	else \
		echo "Warning: $(RUNTIME_FFI_LIB) not found in target/$(WINDOWS_TARGET_$(1))/release/"; \
	fi
	
	# Copy PDB file if available (debug symbols)
	@if [ -f "target/$(WINDOWS_TARGET_$(1))/release/$(RUNTIME_FFI_PDB)" ]; then \
		cp target/$(WINDOWS_TARGET_$(1))/release/$(RUNTIME_FFI_PDB) \
		   $(DOTLOTTIE_PLAYER_WINDOWS_LIB_DIR)/$(WINDOWS_ARCH_$(1))/$(RUNTIME_FFI_PDB); \
		echo "Copied PDB debug symbols for $(1)"; \
	fi
	
	# Copy C++ bindings source files
	@if [ -d "$(CPP_BINDINGS_DIR)" ] && [ -n "$$(ls -A $(CPP_BINDINGS_DIR)/*.cpp 2>/dev/null || true)" ]; then \
		mkdir -p $(DOTLOTTIE_PLAYER_WINDOWS_RELEASE_DIR)/src/cpp; \
		cp $(CPP_BINDINGS_DIR)/*.cpp $(DOTLOTTIE_PLAYER_WINDOWS_RELEASE_DIR)/src/cpp/; \
		echo "Copied C++ source files to $(DOTLOTTIE_PLAYER_WINDOWS_RELEASE_DIR)/src/cpp"; \
	fi
	
	@echo "Windows $(1) packaging completed in $(DOTLOTTIE_PLAYER_WINDOWS_LIB_DIR)/$(WINDOWS_ARCH_$(1))"
endef

# Windows-specific phony targets
.PHONY: windows windows-x86_64-msvc windows-i686-msvc windows-aarch64-msvc windows-x86_64-gnu cpp-bindings install-windows-targets windows-env-info windows-help windows-package windows-clean

# Windows help
windows-help:
	@echo "Windows Build Targets:"
	@echo "======================"
	@echo "  make windows                                      - Build for all Windows architectures"
	@echo "  make windows-x86_64-msvc                         - Build for Windows x86_64 MSVC"
	@echo "  make windows-i686-msvc                           - Build for Windows i686 MSVC"
	@echo "  make windows-aarch64-msvc                        - Build for Windows aarch64 MSVC"
	@echo "  make windows-x86_64-gnu                          - Build for Windows x86_64 GNU"
	@echo "  make windows-clean                               - Clean Windows bindings and release artifacts"
	@echo ""
	@echo "Windows Variables:"
	@echo "=================="
	@echo "  WINDOWS_FEATURES - Rust features to enable (default: $(WINDOWS_FEATURES))"
	@echo ""
	@echo "Windows Examples:"
	@echo "================="
	@echo "  make windows                                      - Build with default features"
	@echo "  make windows WINDOWS_FEATURES=thorvg,uniffi      - Build with custom features"
	@echo "  make windows-x86_64-msvc                         - Build x86_64 MSVC only"
	@echo ""
	@echo "Prerequisites:"
	@echo "=============="
	@echo "  make install-windows-targets                     - Install all required Rust targets"
	@echo "  uniffi-bindgen-cpp                               - Required for C++ bindings generation"

# Generate C++ UniFFI bindings
cpp-bindings:
	@echo "Generating C++ UniFFI bindings..."
	@mkdir -p $(CPP_BINDINGS_DIR)
	rm -rf $(CPP_BINDINGS_DIR)/*
	$(UNIFFI_BINDGEN_CPP) \
		--config uniffi.toml \
		--out-dir $(CPP_BINDINGS_DIR) \
		src/dotlottie_player.udl
	@echo "Applying C++ bindings fixes..."
	@if ls $(CPP_BINDINGS_DIR)/*.hpp >/dev/null 2>&1; then \
		sed -i.bak 's/uint8_t/char/g' $(CPP_BINDINGS_DIR)/*.hpp; \
		rm -f $(CPP_BINDINGS_DIR)/*.bak; \
	fi
	@if ls $(CPP_BINDINGS_DIR)/*.cpp >/dev/null 2>&1; then \
		sed -i.bak 's/uint8_t/char/g' $(CPP_BINDINGS_DIR)/*.cpp; \
		rm -f $(CPP_BINDINGS_DIR)/*.bak; \
	fi
	@echo "C++ bindings generated in $(CPP_BINDINGS_DIR)"

# Build for all Windows architectures (with bindings and packaging)
windows: cpp-bindings $(addprefix windows-,x86_64-msvc i686-msvc aarch64-msvc x86_64-gnu) windows-package
	@echo "All Windows builds completed and packaged!"

# Build for Windows x86_64 MSVC
windows-x86_64-msvc: cpp-bindings windows-check-env
	@echo "Building dotlottie-ffi for Windows x86_64 MSVC..."
	@echo "Target: $(WINDOWS_TARGET_x86_64_msvc)"
	@echo "Features: $(WINDOWS_FEATURES)"
	CC="cl.exe" \
	CXX="cl.exe" \
	cargo build --target $(WINDOWS_TARGET_x86_64_msvc) \
		--release \
		--no-default-features \
		--features $(WINDOWS_FEATURES)
	$(call WINDOWS_PACKAGE_ARCH,x86_64_msvc)

# Build for Windows i686 MSVC
windows-i686-msvc: cpp-bindings windows-check-env
	@echo "Building dotlottie-ffi for Windows i686 MSVC..."
	@echo "Target: $(WINDOWS_TARGET_i686_msvc)"
	@echo "Features: $(WINDOWS_FEATURES)"
	CC="cl.exe" \
	CXX="cl.exe" \
	cargo build --target $(WINDOWS_TARGET_i686_msvc) \
		--release \
		--no-default-features \
		--features $(WINDOWS_FEATURES)
	$(call WINDOWS_PACKAGE_ARCH,i686_msvc)

# Build for Windows aarch64 MSVC
windows-aarch64-msvc: cpp-bindings windows-check-env
	@echo "Building dotlottie-ffi for Windows aarch64 MSVC..."
	@echo "Target: $(WINDOWS_TARGET_aarch64_msvc)"
	@echo "Features: $(WINDOWS_FEATURES)"
	CC="cl.exe" \
	CXX="cl.exe" \
	cargo build --target $(WINDOWS_TARGET_aarch64_msvc) \
		--release \
		--no-default-features \
		--features $(WINDOWS_FEATURES)
	$(call WINDOWS_PACKAGE_ARCH,aarch64_msvc)

# Build for Windows x86_64 GNU
windows-x86_64-gnu: cpp-bindings windows-check-env
	@echo "Building dotlottie-ffi for Windows x86_64 GNU..."
	@echo "Target: $(WINDOWS_TARGET_x86_64_gnu)"
	@echo "Features: $(WINDOWS_FEATURES)"
	CC="x86_64-w64-mingw32-gcc" \
	CXX="x86_64-w64-mingw32-g++" \
	cargo build --target $(WINDOWS_TARGET_x86_64_gnu) \
		--release \
		--no-default-features \
		--features $(WINDOWS_FEATURES)
	$(call WINDOWS_PACKAGE_ARCH,x86_64_gnu)

# Package all Windows builds into a single release
windows-package:
	@echo "Creating Windows release package..."
	@mkdir -p $(WINDOWS_RELEASE_DIR)
	@echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(DOTLOTTIE_PLAYER_WINDOWS_RELEASE_DIR)/version.txt
	@echo "Created version.txt with version $(CRATE_VERSION)-$(COMMIT_HASH)"
	@cd $(WINDOWS_RELEASE_DIR) && \
		rm -f dotlottie-player.windows.tar.gz && \
		tar zcf dotlottie-player.windows.tar.gz *
	@echo "Windows release package completed: $(WINDOWS_RELEASE_DIR)/dotlottie-player.windows.tar.gz"

# Check Windows build environment
windows-check-env:
	@echo "Checking Windows build environment..."
	@if ! command -v $(UNIFFI_BINDGEN_CPP) >/dev/null 2>&1; then \
		echo "Warning: $(UNIFFI_BINDGEN_CPP) not found in PATH"; \
		echo "C++ bindings generation may fail"; \
		echo "Please install uniffi-bindgen-cpp or ensure it's in PATH"; \
	fi
	@if ! command -v cargo >/dev/null 2>&1; then \
		echo "Error: cargo not found"; \
		exit 1; \
	fi

# Install Windows targets if not already installed
install-windows-targets:
	@echo "Installing Windows Rust targets..."
	rustup target add $(WINDOWS_TARGETS)
	@echo "Windows targets installed successfully!"

# Show Windows environment info
windows-env-info: windows-check-env
	@echo "Windows Environment Information:"
	@echo "==============================="
	@echo "Available targets: $(WINDOWS_TARGETS)"
	@echo "C++ bindings directory: $(CPP_BINDINGS_DIR)"
	@echo "UniFFI bindgen C++: $(UNIFFI_BINDGEN_CPP)"
	@echo "Windows features: $(WINDOWS_FEATURES)"
	@echo ""
	@echo "Rust toolchain info:"
	@echo "===================="
	rustc --version
	cargo --version
	@echo ""
	@echo "Windows Rust targets:"
	@echo "===================="
	@for target in $(WINDOWS_TARGETS); do \
		if rustup target list --installed | grep -q $$target; then \
			echo "✓ $$target (installed)"; \
		else \
			echo "✗ $$target (not installed - run 'make install-windows-targets')"; \
		fi; \
	done
	@echo ""
	@echo "UniFFI bindgen C++ status:"
	@echo "========================="
	@if command -v $(UNIFFI_BINDGEN_CPP) >/dev/null 2>&1; then \
		echo "✓ $(UNIFFI_BINDGEN_CPP) found at: $$(which $(UNIFFI_BINDGEN_CPP))"; \
		$(UNIFFI_BINDGEN_CPP) --version 2>/dev/null || echo "Version info not available"; \
	else \
		echo "✗ $(UNIFFI_BINDGEN_CPP) not found in PATH"; \
		echo "Please install uniffi-bindgen-cpp for C++ bindings generation"; \
	fi

# Clean Windows bindings and release artifacts
windows-clean:
	@echo "Cleaning Windows bindings and release artifacts..."
	rm -rf $(CPP_BINDINGS_DIR)
	rm -rf $(WINDOWS_RELEASE_DIR)
	@echo "Windows artifacts cleaned!" 