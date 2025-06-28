# Windows build configuration for dotlottie-ffi

# Tool configuration
UNIFFI_BINDGEN_CPP ?= uniffi-bindgen-cpp

# Default Rust features for Windows builds
WINDOWS_FEATURES ?= uniffi,thorvg,thorvg_webp,thorvg_png,thorvg_jpg,thorvg_ttf,thorvg_lottie_expressions

# UniFFI Bindings
BINDINGS_DIR ?= dotlottie-ffi/uniffi_bindings
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
CRATE_VERSION := $(shell grep -m 1 version dotlottie-ffi/Cargo.toml | sed 's/.*"\([0-9.]\+\)"/\1/')
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
	@mkdir -p $(DOTLOTTIE_PLAYER_WINDOWS_INCLUDE_DIR)/$(WINDOWS_ARCH_$(1))
	@mkdir -p $(DOTLOTTIE_PLAYER_WINDOWS_LIB_DIR)/$(WINDOWS_ARCH_$(1))
	
	# Copy C++ bindings headers
	@if [ -d "$(CPP_BINDINGS_DIR)" ] && [ -n "$$(ls -A $(CPP_BINDINGS_DIR)/*.hpp 2>/dev/null || true)" ]; then \
		cp $(CPP_BINDINGS_DIR)/*.hpp $(DOTLOTTIE_PLAYER_WINDOWS_INCLUDE_DIR)/$(WINDOWS_ARCH_$(1))/; \
	fi
	
	# Copy main library files
	@if [ -f "dotlottie-ffi/target/$(WINDOWS_TARGET_$(1))/release/$(RUNTIME_FFI_DLL)" ]; then \
		cp dotlottie-ffi/target/$(WINDOWS_TARGET_$(1))/release/$(RUNTIME_FFI_DLL) \
		   $(DOTLOTTIE_PLAYER_WINDOWS_LIB_DIR)/$(WINDOWS_ARCH_$(1))/$(RUNTIME_FFI_DLL); \
	else \
		echo "Warning: $(RUNTIME_FFI_DLL) not found in dotlottie-ffi/target/$(WINDOWS_TARGET_$(1))/release/"; \
	fi
	
	@if [ -f "dotlottie-ffi/target/$(WINDOWS_TARGET_$(1))/release/$(RUNTIME_FFI_LIB)" ]; then \
		cp dotlottie-ffi/target/$(WINDOWS_TARGET_$(1))/release/$(RUNTIME_FFI_LIB) \
		   $(DOTLOTTIE_PLAYER_WINDOWS_LIB_DIR)/$(WINDOWS_ARCH_$(1))/$(RUNTIME_FFI_LIB); \
	else \
		echo "Warning: $(RUNTIME_FFI_LIB) not found in dotlottie-ffi/target/$(WINDOWS_TARGET_$(1))/release/"; \
	fi
	
	# Copy PDB file if available (debug symbols)
	@if [ -f "dotlottie-ffi/target/$(WINDOWS_TARGET_$(1))/release/$(RUNTIME_FFI_PDB)" ]; then \
		cp dotlottie-ffi/target/$(WINDOWS_TARGET_$(1))/release/$(RUNTIME_FFI_PDB) \
		   $(DOTLOTTIE_PLAYER_WINDOWS_LIB_DIR)/$(WINDOWS_ARCH_$(1))/$(RUNTIME_FFI_PDB); \
	fi
	
	# Copy C++ bindings source files
	@if [ -d "$(CPP_BINDINGS_DIR)" ] && [ -n "$$(ls -A $(CPP_BINDINGS_DIR)/*.cpp 2>/dev/null || true)" ]; then \
		mkdir -p $(DOTLOTTIE_PLAYER_WINDOWS_RELEASE_DIR)/src/cpp; \
		cp $(CPP_BINDINGS_DIR)/*.cpp $(DOTLOTTIE_PLAYER_WINDOWS_RELEASE_DIR)/src/cpp/; \
	fi
endef

# Windows-specific phony targets
.PHONY: windows windows-x86_64-msvc windows-i686-msvc windows-aarch64-msvc windows-x86_64-gnu windows-install-targets windows-clean



# Generate C++ UniFFI bindings for Windows
windows-cpp-bindings:
	@echo "→ Generating C++ UniFFI bindings..."
	@mkdir -p $(CPP_BINDINGS_DIR)
	@rm -rf $(CPP_BINDINGS_DIR)/*
	@$(UNIFFI_BINDGEN_CPP) \
		--config dotlottie-ffi/uniffi.toml \
		--out-dir $(CPP_BINDINGS_DIR) \
		dotlottie-ffi/src/dotlottie_player.udl >/dev/null
	@if ls $(CPP_BINDINGS_DIR)/*.hpp >/dev/null 2>&1; then \
		sed -i.bak 's/uint8_t/char/g' $(CPP_BINDINGS_DIR)/*.hpp; \
		rm -f $(CPP_BINDINGS_DIR)/*.bak; \
	fi
	@if ls $(CPP_BINDINGS_DIR)/*.cpp >/dev/null 2>&1; then \
		sed -i.bak 's/uint8_t/char/g' $(CPP_BINDINGS_DIR)/*.cpp; \
		rm -f $(CPP_BINDINGS_DIR)/*.bak; \
	fi
	@echo "✓ C++ bindings generated"

# Build for all Windows architectures (with bindings and packaging)
windows: windows-cpp-bindings $(addprefix windows-,x86_64-msvc i686-msvc aarch64-msvc x86_64-gnu) windows-package
	@echo "✓ All Windows builds complete"

# Build for Windows x86_64 MSVC
windows-x86_64-msvc: windows-cpp-bindings windows-check-env
	@echo "→ Building Windows x86_64 MSVC..."
	@CC="cl.exe" \
	CXX="cl.exe" \
	AR="lib.exe" \
	RANLIB="echo" \
	CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER="link.exe" \
	BINDGEN_EXTRA_CLANG_ARGS="" \
	cargo build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--target $(WINDOWS_TARGET_x86_64_msvc) \
		--release \
		--no-default-features \
		--features $(WINDOWS_FEATURES) >/dev/null
	@$(call WINDOWS_PACKAGE_ARCH,x86_64_msvc)
	@echo "✓ Windows x86_64 MSVC build complete"

# Build for Windows i686 MSVC
windows-i686-msvc: windows-cpp-bindings windows-check-env
	@echo "→ Building Windows i686 MSVC..."
	@CC="cl.exe" \
	CXX="cl.exe" \
	AR="lib.exe" \
	RANLIB="echo" \
	CARGO_TARGET_I686_PC_WINDOWS_MSVC_LINKER="link.exe" \
	BINDGEN_EXTRA_CLANG_ARGS="" \
	cargo build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--target $(WINDOWS_TARGET_i686_msvc) \
		--release \
		--no-default-features \
		--features $(WINDOWS_FEATURES) >/dev/null
	@$(call WINDOWS_PACKAGE_ARCH,i686_msvc)
	@echo "✓ Windows i686 MSVC build complete"

# Build for Windows aarch64 MSVC
windows-aarch64-msvc: windows-cpp-bindings windows-check-env
	@echo "→ Building Windows aarch64 MSVC..."
	@CC="cl.exe" \
	CXX="cl.exe" \
	AR="lib.exe" \
	RANLIB="echo" \
	CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_LINKER="link.exe" \
	BINDGEN_EXTRA_CLANG_ARGS="" \
	cargo build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--target $(WINDOWS_TARGET_aarch64_msvc) \
		--release \
		--no-default-features \
		--features $(WINDOWS_FEATURES) >/dev/null
	@$(call WINDOWS_PACKAGE_ARCH,aarch64_msvc)
	@echo "✓ Windows aarch64 MSVC build complete"

# Build for Windows x86_64 GNU
windows-x86_64-gnu: windows-cpp-bindings windows-check-env
	@echo "→ Building Windows x86_64 GNU..."
	@CC="x86_64-w64-mingw32-gcc" \
	CXX="x86_64-w64-mingw32-g++" \
	AR="x86_64-w64-mingw32-ar" \
	RANLIB="x86_64-w64-mingw32-ranlib" \
	CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="x86_64-w64-mingw32-gcc" \
	BINDGEN_EXTRA_CLANG_ARGS="" \
	cargo build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--target $(WINDOWS_TARGET_x86_64_gnu) \
		--release \
		--no-default-features \
		--features $(WINDOWS_FEATURES) >/dev/null
	@$(call WINDOWS_PACKAGE_ARCH,x86_64_gnu)
	@echo "✓ Windows x86_64 GNU build complete"

# Package all Windows builds into a single release
windows-package:
	@echo "→ Creating Windows release package..."
	@mkdir -p $(WINDOWS_RELEASE_DIR)
	@echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(DOTLOTTIE_PLAYER_WINDOWS_RELEASE_DIR)/version.txt
	@echo "✓ Windows release package created: $(WINDOWS_RELEASE_DIR)/"

# Check Windows build environment
windows-check-env:
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
windows-install-targets:
	@echo "→ Installing Windows Rust targets..."
	@rustup target add $(WINDOWS_TARGETS) >/dev/null
	@echo "✓ Windows targets installed"



# Clean Windows bindings and release artifacts
windows-clean:
	@echo "→ Cleaning Windows builds..."
	@rm -rf $(CPP_BINDINGS_DIR)
	@rm -rf $(WINDOWS_RELEASE_DIR)
	@echo "✓ Windows builds cleaned" 