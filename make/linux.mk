# Linux targets
# Uses C FFI (ffi feature flag) along with tvg and tvg-sw for thorvg support
# MUST NOT USE UNIFFI feature flag for linux build

# Default Rust features for Linux builds
FEATURES ?= tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-lottie-expressions,tvg-threads
DEFAULT_FEATURES = tvg,tvg-sw,ffi

# Release and packaging variables
RELEASE_DIR ?= release
LINUX_RELEASE_DIR ?= $(RELEASE_DIR)/linux
DOTLOTTIE_PLAYER_DIR ?= dotlottie-player

# Library names
LINUX_FFI_LIB_BASE ?= libdotlottie_player
LINUX_STATIC_LIB := $(LINUX_FFI_LIB_BASE).a
LINUX_SHARED_LIB := $(LINUX_FFI_LIB_BASE).so
LINUX_HEADER_FILE := bindings.h

# Get version information
CRATE_VERSION = $(shell grep -m 1 'version =' dotlottie-ffi/Cargo.toml | grep -o '[0-9][0-9.]*')
COMMIT_HASH := $(shell git rev-parse --short HEAD)

# Linux target mapping
LINUX_TARGET_x86_64 = x86_64-unknown-linux-gnu
LINUX_TARGET_arm64 = aarch64-unknown-linux-gnu

# Linux targets list
LINUX_TARGETS = x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

# Detect host platform
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Function to check platform support - only called when Linux targets are invoked
define check_linux_platform_support
$(if $(filter Linux,$(UNAME_S)),,$(error "Linux builds require a Linux host system. Current system: $(UNAME_S)"))
endef

# Function to package Linux architecture builds
# Args: $(1) = architecture (x86_64, arm64)
define LINUX_PACKAGE_ARCH
	@echo "→ Packaging Linux $(1)..."
	@mkdir -p $(LINUX_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/include
	@mkdir -p $(LINUX_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/lib

	# Copy header file
	@if [ -f "dotlottie-ffi/$(LINUX_HEADER_FILE)" ]; then \
		cp dotlottie-ffi/$(LINUX_HEADER_FILE) $(LINUX_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/include/; \
	else \
		echo "Error: $(LINUX_HEADER_FILE) not found"; \
		exit 1; \
	fi

	# Copy static library
	@if [ -f "dotlottie-ffi/target/$(LINUX_TARGET_$(1))/release/$(LINUX_STATIC_LIB)" ]; then \
		cp dotlottie-ffi/target/$(LINUX_TARGET_$(1))/release/$(LINUX_STATIC_LIB) \
		   $(LINUX_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/lib/; \
	else \
		echo "Warning: $(LINUX_STATIC_LIB) not found for $(1)"; \
	fi

	# Copy shared library and strip it
	@if [ -f "dotlottie-ffi/target/$(LINUX_TARGET_$(1))/release/$(LINUX_SHARED_LIB)" ]; then \
		cp dotlottie-ffi/target/$(LINUX_TARGET_$(1))/release/$(LINUX_SHARED_LIB) \
		   $(LINUX_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/lib/; \
		if [ "$(1)" = "arm64" ]; then \
			if command -v aarch64-linux-gnu-strip >/dev/null 2>&1; then \
				aarch64-linux-gnu-strip --strip-unneeded $(LINUX_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/lib/$(LINUX_SHARED_LIB); \
			else \
				echo "Warning: aarch64-linux-gnu-strip not found, skipping strip for $(1)"; \
			fi; \
		else \
			if command -v strip >/dev/null 2>&1; then \
				strip --strip-unneeded $(LINUX_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/lib/$(LINUX_SHARED_LIB); \
			fi; \
		fi; \
	else \
		echo "Warning: $(LINUX_SHARED_LIB) not found for $(1)"; \
	fi

	# Create version file
	@echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(LINUX_RELEASE_DIR)/$(1)/version.txt

	@echo "✓ Linux $(1) packaging complete"
endef

# Linux-specific phony targets
.PHONY: linux linux-x86_64 linux-arm64 linux-setup linux-clean linux-check-deps

# Build all Linux targets
linux: $(addprefix linux-,x86_64 arm64)
	@echo "✓ All Linux builds complete"

# Build for Linux x86_64
linux-x86_64: linux-check-deps
	$(call check_linux_platform_support)
	@echo "→ Building Linux x86_64..."
	@CC="gcc" \
	CXX="g++" \
	AR="ar" \
	RANLIB="ranlib" \
	CFLAGS="-fPIC" \
	CXXFLAGS="-fPIC -std=c++14" \
	cargo build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--target $(LINUX_TARGET_x86_64) \
		--release \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES)
	@$(call LINUX_PACKAGE_ARCH,x86_64)
	@echo "✓ Linux x86_64 build complete"

# Build for Linux ARM64
linux-arm64: linux-check-deps
	$(call check_linux_platform_support)
	@echo "→ Building Linux ARM64..."
	@# Check if cross-compilation toolchain is available
	@if ! command -v aarch64-linux-gnu-gcc >/dev/null 2>&1; then \
		echo "Warning: aarch64-linux-gnu-gcc not found. Install gcc-aarch64-linux-gnu for cross-compilation."; \
		echo "Attempting build with default toolchain..."; \
		CC="gcc" \
		CXX="g++" \
		AR="ar" \
		RANLIB="ranlib" \
		CFLAGS="-fPIC" \
		CXXFLAGS="-fPIC -std=c++14" \
		CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="gcc" \
		cargo build \
			--manifest-path dotlottie-ffi/Cargo.toml \
			--target $(LINUX_TARGET_arm64) \
			--release \
			--no-default-features \
			--features $(DEFAULT_FEATURES),$(FEATURES); \
	else \
		CC="aarch64-linux-gnu-gcc" \
		CXX="aarch64-linux-gnu-g++" \
		AR="aarch64-linux-gnu-ar" \
		RANLIB="aarch64-linux-gnu-ranlib" \
		CFLAGS="-fPIC" \
		CXXFLAGS="-fPIC -std=c++14" \
		CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-linux-gnu-gcc" \
		cargo build \
			--manifest-path dotlottie-ffi/Cargo.toml \
			--target $(LINUX_TARGET_arm64) \
			--release \
			--no-default-features \
			--features $(DEFAULT_FEATURES),$(FEATURES); \
	fi
	@$(call LINUX_PACKAGE_ARCH,arm64)
	@echo "✓ Linux ARM64 build complete"

# Check for required dependencies
linux-check-deps:
	@echo "→ Checking Linux build dependencies..."
	@# Check for required build tools
	@if ! command -v gcc >/dev/null 2>&1; then \
		echo "Error: gcc not found. Please install build-essential or gcc."; \
		exit 1; \
	fi
	@if ! command -v g++ >/dev/null 2>&1; then \
		echo "Error: g++ not found. Please install build-essential or g++."; \
		exit 1; \
	fi
	@if ! command -v ar >/dev/null 2>&1; then \
		echo "Error: ar not found. Please install binutils."; \
		exit 1; \
	fi
	@if ! command -v pkg-config >/dev/null 2>&1; then \
		echo "Warning: pkg-config not found. Some dependencies may fail to link."; \
	fi
	@echo "✓ Build dependencies check complete"

# Install Linux targets and dependencies
linux-setup:
	@echo "→ Setting up Linux build environment..."
	@# Install Rust targets
	@rustup target add $(LINUX_TARGETS) >/dev/null
	@echo ""
	@echo "✓ Linux Rust targets installed"
	@echo ""
	@echo "Additional dependencies you may need to install:"
	@echo "  Ubuntu/Debian: sudo apt-get install build-essential pkg-config"
	@echo "  For ARM64 cross-compilation: sudo apt-get install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu"
	@echo "  Fedora/RHEL: sudo dnf install gcc gcc-c++ make pkg-config"
	@echo "  For ARM64 cross-compilation: sudo dnf install gcc-aarch64-linux-gnu gcc-c++-aarch64-linux-gnu"
	@echo "  Arch Linux: sudo pacman -S base-devel"
	@echo "  For ARM64 cross-compilation: sudo pacman -S aarch64-linux-gnu-gcc"

# Clean Linux builds
linux-clean:
	@echo "→ Cleaning Linux builds..."
	@rm -rf $(LINUX_RELEASE_DIR)
	@# Clean specific target directories
	@for target in $(LINUX_TARGETS); do \
		if [ -d "dotlottie-ffi/target/$$target" ]; then \
			rm -rf dotlottie-ffi/target/$$target/release; \
		fi; \
	done
	@echo "✓ Linux builds cleaned"
