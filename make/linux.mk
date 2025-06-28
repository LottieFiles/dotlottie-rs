UNIFFI_BINDGEN_CPP ?= uniffi-bindgen-cpp

# Default Rust features for Linux builds
LINUX_FEATURES ?= uniffi,thorvg,thorvg_webp,thorvg_png,thorvg_jpg,thorvg_ttf,thorvg_lottie_expressions

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
	@cargo build \
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
	@cargo build \
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
