UNIFFI_BINDGEN_CPP ?= uniffi-bindgen-cpp

# Default Rust features for Linux builds
LINUX_FEATURES ?= uniffi,thorvg,thorvg_webp,thorvg_png,thorvg_jpg,thorvg_ttf,thorvg_lottie_expressions

# Linux toolchain configuration
LINUX_CC_x86_64 ?= gcc
LINUX_CXX_x86_64 ?= g++
LINUX_AR_x86_64 ?= ar
LINUX_RANLIB_x86_64 ?= ranlib

LINUX_CC_aarch64 ?= aarch64-linux-gnu-gcc
LINUX_CXX_aarch64 ?= aarch64-linux-gnu-g++
LINUX_AR_aarch64 ?= aarch64-linux-gnu-ar
LINUX_RANLIB_aarch64 ?= aarch64-linux-gnu-ranlib

# UniFFI Bindings
BINDINGS_DIR ?= dotlottie-ffi/uniffi_bindings
CPP_BINDINGS_DIR ?= $(BINDINGS_DIR)/cpp

# Linux targets
LINUX_TARGETS = x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

# Linux target mapping
LINUX_TARGET_x86_64 = x86_64-unknown-linux-gnu
LINUX_TARGET_aarch64 = aarch64-unknown-linux-gnu

# Get version information
CRATE_VERSION := $(shell grep -m 1 version dotlottie-ffi/Cargo.toml | sed 's/.*"\([0-9.]\+\)"/\1/')
COMMIT_HASH := $(shell git rev-parse --short HEAD)

# Linux-specific phony targets
.PHONY: linux linux-x86_64 linux-aarch64 linux-install-targets linux-package linux-clean



# Generate C++ UniFFI bindings for Linux
linux-cpp-bindings:
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



# Release directories
LINUX_RELEASE_DIR ?= release/linux
LINUX_RELEASE_INCLUDE_DIR ?= $(LINUX_RELEASE_DIR)/include
LINUX_RELEASE_LIB_DIR ?= $(LINUX_RELEASE_DIR)/lib

# Linux packaging function
define LINUX_PACKAGE_ARCH
	@mkdir -p $(LINUX_RELEASE_INCLUDE_DIR)/$(1)
	@mkdir -p $(LINUX_RELEASE_LIB_DIR)/$(1)
	
	# Copy C++ bindings headers
	@if [ -d "$(CPP_BINDINGS_DIR)" ] && [ -n "$$(ls -A $(CPP_BINDINGS_DIR)/*.hpp 2>/dev/null || true)" ]; then \
		cp $(CPP_BINDINGS_DIR)/*.hpp $(LINUX_RELEASE_INCLUDE_DIR)/$(1)/; \
	fi
	
	# Copy shared library
	@if [ -f "dotlottie-ffi/target/$(LINUX_TARGET_$(1))/release/libdotlottie_player.so" ]; then \
		cp dotlottie-ffi/target/$(LINUX_TARGET_$(1))/release/libdotlottie_player.so \
		   $(LINUX_RELEASE_LIB_DIR)/$(1)/libdotlottie_player.so; \
	fi
	
	# Copy static library if available
	@if [ -f "dotlottie-ffi/target/$(LINUX_TARGET_$(1))/release/libdotlottie_player.a" ]; then \
		cp dotlottie-ffi/target/$(LINUX_TARGET_$(1))/release/libdotlottie_player.a \
		   $(LINUX_RELEASE_LIB_DIR)/$(1)/libdotlottie_player.a; \
	fi
	
	# Copy C++ bindings source files
	@if [ -d "$(CPP_BINDINGS_DIR)" ] && [ -n "$$(ls -A $(CPP_BINDINGS_DIR)/*.cpp 2>/dev/null || true)" ]; then \
		mkdir -p $(LINUX_RELEASE_DIR)/src/cpp; \
		cp $(CPP_BINDINGS_DIR)/*.cpp $(LINUX_RELEASE_DIR)/src/cpp/; \
	fi
endef

# Build for all Linux architectures
linux: linux-cpp-bindings $(addprefix linux-,x86_64 aarch64) linux-package
	@echo "✓ All Linux builds complete"

# Linux x86_64
linux-x86_64: linux-cpp-bindings linux-check-env
	@echo "→ Building Linux x86_64..."
	@CC="$(LINUX_CC_x86_64)" \
	CXX="$(LINUX_CXX_x86_64)" \
	AR="$(LINUX_AR_x86_64)" \
	RANLIB="$(LINUX_RANLIB_x86_64)" \
	CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$(LINUX_CC_x86_64)" \
	BINDGEN_EXTRA_CLANG_ARGS="" \
	cargo build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--target $(LINUX_TARGET_x86_64) \
		--no-default-features \
		--features $(LINUX_FEATURES) \
		--release >/dev/null
	@$(call LINUX_PACKAGE_ARCH,x86_64)
	@echo "✓ Linux x86_64 build complete"

# Linux aarch64
linux-aarch64: linux-cpp-bindings linux-check-env
	@echo "→ Building Linux aarch64..."
	@CC="$(LINUX_CC_aarch64)" \
	CXX="$(LINUX_CXX_aarch64)" \
	AR="$(LINUX_AR_aarch64)" \
	RANLIB="$(LINUX_RANLIB_aarch64)" \
	CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="$(LINUX_CC_aarch64)" \
	BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/aarch64-linux-gnu" \
	PKG_CONFIG_ALLOW_CROSS=1 \
	PKG_CONFIG_PATH="/usr/lib/aarch64-linux-gnu/pkgconfig" \
	cargo build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--target $(LINUX_TARGET_aarch64) \
		--no-default-features \
		--features $(LINUX_FEATURES) \
		--release >/dev/null
	@$(call LINUX_PACKAGE_ARCH,aarch64)
	@echo "✓ Linux aarch64 build complete"

# Package all Linux builds
linux-package:
	@echo "→ Creating Linux release package..."
	@mkdir -p $(LINUX_RELEASE_DIR)
	@echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(LINUX_RELEASE_DIR)/version.txt
	
	@echo "✓ Linux release package created: $(LINUX_RELEASE_DIR)/"

# Check Linux build environment
linux-check-env:
	@if ! command -v $(UNIFFI_BINDGEN_CPP) >/dev/null 2>&1; then \
		echo "Warning: $(UNIFFI_BINDGEN_CPP) not found in PATH"; \
		echo "C++ bindings generation may fail"; \
		echo "Please install uniffi-bindgen-cpp or ensure it's in PATH"; \
	fi
	@if ! command -v cargo >/dev/null 2>&1; then \
		echo "Error: cargo not found"; \
		exit 1; \
	fi
	@echo "✓ Checking x86_64 toolchain..."
	@if ! command -v $(LINUX_CC_x86_64) >/dev/null 2>&1; then \
		echo "Error: $(LINUX_CC_x86_64) not found"; \
		echo "Please install build-essential or gcc"; \
		exit 1; \
	fi
	@if ! command -v $(LINUX_CXX_x86_64) >/dev/null 2>&1; then \
		echo "Error: $(LINUX_CXX_x86_64) not found"; \
		echo "Please install build-essential or g++"; \
		exit 1; \
	fi
	@echo "✓ Checking aarch64 cross-compilation toolchain..."
	@if ! command -v $(LINUX_CC_aarch64) >/dev/null 2>&1; then \
		echo "Warning: $(LINUX_CC_aarch64) not found"; \
		echo "For aarch64 cross-compilation, please install:"; \
		echo "  Ubuntu/Debian: sudo apt-get install gcc-aarch64-linux-gnu"; \
		echo "  RHEL/CentOS: sudo yum install gcc-aarch64-linux-gnu"; \
		echo "  Arch: sudo pacman -S aarch64-linux-gnu-gcc"; \
		echo "Skipping aarch64 toolchain validation..."; \
	else \
		if ! command -v $(LINUX_CXX_aarch64) >/dev/null 2>&1; then \
			echo "Warning: $(LINUX_CXX_aarch64) not found"; \
			echo "Please install g++-aarch64-linux-gnu"; \
		fi; \
	fi

# Install Linux targets if not already installed
linux-install-targets:
	@echo "→ Installing Linux Rust targets..."
	@rustup target add $(LINUX_TARGETS) >/dev/null
	@echo "✓ Linux targets installed"



# Clean Linux bindings and release artifacts
linux-clean:
	@echo "→ Cleaning Linux builds..."
	@cargo clean --manifest-path dotlottie-ffi/Cargo.toml >/dev/null
	@rm -rf $(CPP_BINDINGS_DIR)
	@rm -rf $(LINUX_RELEASE_DIR)
	@echo "✓ Linux builds cleaned"
