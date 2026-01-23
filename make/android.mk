# Detect host platform first
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Android NDK path
# By default, set for macOS with Homebrew. For other platforms, override ANDROID_NDK_HOME or set it in your environment.
ifeq ($(UNAME_S),Darwin)
    ANDROID_NDK_HOME ?= /opt/homebrew/share/android-ndk
else ifeq ($(UNAME_S),Linux)
    # Common default for Linux (update as needed for your system)
    ANDROID_NDK_HOME ?= /opt/android-ndk
else
    # Unknown platform: require user to set ANDROID_NDK_HOME
    ifndef ANDROID_NDK_HOME
        $(error "Please set ANDROID_NDK_HOME to your Android NDK installation path for $(UNAME_S).")
    endif
endif
API_LEVEL ?= 21

# Minimum required NDK version
MIN_NDK_VERSION = 28

# Default Rust features for Android builds
ANDROID_FEATURES ?= tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-lottie-expressions,tvg-threads

ifdef FEATURES
	ANDROID_FEATURES = $(FEATURES)
endif

# Release and packaging variables
RELEASE_DIR ?= release
ANDROID_RELEASE_DIR ?= $(RELEASE_DIR)/android
LIBCPP_SHARED_LIB ?= libc++_shared.so

# Detect host tag for NDK

# Function to check platform support - only called when Android targets are invoked
define check_android_platform_support
$(if $(filter Darwin Linux,$(UNAME_S)),,$(error "Android builds not supported on $(UNAME_S). Requires macOS or Linux with Android NDK."))
endef

ifeq ($(UNAME_S),Darwin)
    ifeq ($(UNAME_M),arm64)
        HOST_TAG = darwin-x86_64
    else
        HOST_TAG = darwin-x86_64
    endif
else ifeq ($(UNAME_S),Linux)
    HOST_TAG = linux-x86_64
else
    # For unsupported platforms, set a default - error will be thrown when Android targets are used
    HOST_TAG = linux-x86_64
endif

# Android NDK toolchain paths
ANDROID_TOOLCHAIN = $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(HOST_TAG)
ANDROID_AR = $(ANDROID_TOOLCHAIN)/bin/llvm-ar
ANDROID_RANLIB = $(ANDROID_TOOLCHAIN)/bin/llvm-ranlib
ANDROID_STRIP = $(ANDROID_TOOLCHAIN)/bin/llvm-strip

# Android targets
ANDROID_TARGETS = aarch64-linux-android x86_64-linux-android i686-linux-android armv7-linux-androideabi

# Rust target mapping
RUST_TARGET_aarch64 = aarch64-linux-android
RUST_TARGET_x86_64 = x86_64-linux-android
RUST_TARGET_x86 = i686-linux-android
RUST_TARGET_armv7 = armv7-linux-androideabi

# Android ABI mapping
ANDROID_ABI_aarch64 = arm64-v8a
ANDROID_ABI_x86_64 = x86_64
ANDROID_ABI_x86 = x86
ANDROID_ABI_armv7 = armeabi-v7a

# Android libcpp path mapping
LIBCPP_PATH_aarch64 = aarch64-linux-android
LIBCPP_PATH_x86_64 = x86_64-linux-android
LIBCPP_PATH_x86 = i686-linux-android
LIBCPP_PATH_armv7 = arm-linux-androideabi

# Android compiler mapping
ANDROID_CXX_aarch64 = $(ANDROID_TOOLCHAIN)/bin/aarch64-linux-android$(API_LEVEL)-clang++
ANDROID_CXX_x86_64 = $(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang++
ANDROID_CXX_x86 = $(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang++
ANDROID_CXX_armv7 = $(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang++

# Get version information
CRATE_VERSION = $(shell grep -m 1 'version =' dotlottie-rs/Cargo.toml | grep -o '[0-9][0-9.]*')
COMMIT_HASH := $(shell git rev-parse --short HEAD)

# ============================================================================
# Android Build Targets (C API)
# ============================================================================

# Android-specific phony targets
.PHONY: android android-aarch64 android-x86_64 android-x86 android-armv7 android-package android-setup android-clean

# Check if NDK path is valid
android-check-ndk:
	@if [ ! -d "$(ANDROID_NDK_HOME)" ]; then \
		echo "Error: ANDROID_NDK_HOME does not exist: $(ANDROID_NDK_HOME)"; \
		exit 1; \
	fi
	@if [ -f "$(ANDROID_NDK_HOME)/source.properties" ]; then \
		NDK_VERSION=$$(grep "Pkg.Revision" "$(ANDROID_NDK_HOME)/source.properties" | sed 's/.*= *\([0-9]\+\).*/\1/' | head -1); \
		if [ "$$NDK_VERSION" -lt $(MIN_NDK_VERSION) ]; then \
			echo "Error: Android NDK r$(MIN_NDK_VERSION) or higher is required. Found NDK r$$NDK_VERSION"; \
			echo "Please upgrade your Android NDK to r$(MIN_NDK_VERSION) or later."; \
			echo "Current NDK path: $(ANDROID_NDK_HOME)"; \
			echo "You can download the latest NDK from: https://developer.android.com/ndk/downloads"; \
			exit 1; \
		else \
			echo "✓ Using Android NDK r$$NDK_VERSION (meets minimum requirement of r$(MIN_NDK_VERSION))"; \
		fi; \
	else \
		echo "Warning: Could not determine NDK version from $(ANDROID_NDK_HOME)/source.properties"; \
		echo "Ensure you are using Android NDK r$(MIN_NDK_VERSION) or higher."; \
	fi
	@if [ ! -f "$(ANDROID_CXX_aarch64)" ]; then \
		echo "Error: Android toolchain not found. Please check ANDROID_NDK_HOME and API_LEVEL."; \
		echo "Expected: $(ANDROID_CXX_aarch64)"; \
		exit 1; \
	fi

# Install Android targets if not already installed
android-setup:
	@echo "→ Installing Android Rust targets..."
	@rustup target add $(ANDROID_TARGETS) >/dev/null
	@echo "✓ Android targets installed"

# Clean Android bindings and release artifacts
android-clean:
	@echo "→ Cleaning Android builds..."
	@rm -rf $(ANDROID_RELEASE_DIR)
	@echo "✓ Android builds cleaned"

# ============================================================================
# Android C API Targets (using dotlottie-rs)
# ============================================================================

# Android feature set (C API from dotlottie-rs)
ANDROID_DEFAULT_FEATURES = tvg,tvg-sw,c_api
ANDROID_LIB_NAME = libdotlottie_rs.so

# Build for all Android architectures with C API
android: $(addprefix android-,aarch64 x86_64 x86 armv7) android-package
	@echo "✓ All Android C API builds complete"

# Build for Android ARM64
android-aarch64: android-check-ndk
	$(call check_android_platform_support)
	@echo "→ Building Android aarch64..."
	@ANDROID_NDK_HOME="$(ANDROID_NDK_HOME)" \
	CC="$(ANDROID_TOOLCHAIN)/bin/aarch64-linux-android$(API_LEVEL)-clang" \
	CXX="$(ANDROID_TOOLCHAIN)/bin/aarch64-linux-android$(API_LEVEL)-clang++" \
	CLANG_PATH="$(ANDROID_TOOLCHAIN)/bin/aarch64-linux-android$(API_LEVEL)-clang" \
	CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$(ANDROID_TOOLCHAIN)/bin/aarch64-linux-android$(API_LEVEL)-clang" \
	AR="$(ANDROID_AR)" \
	RANLIB="$(ANDROID_RANLIB)" \
	BINDGEN_EXTRA_CLANG_ARGS="-isysroot $(ANDROID_TOOLCHAIN)/sysroot" \
	cargo build \
		--manifest-path dotlottie-rs/Cargo.toml \
		--target $(RUST_TARGET_aarch64) \
		--release \
		--no-default-features \
		--features $(ANDROID_DEFAULT_FEATURES),$(ANDROID_FEATURES)
	@mkdir -p $(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_aarch64)
	@cp dotlottie-rs/target/$(RUST_TARGET_aarch64)/release/$(ANDROID_LIB_NAME) \
		$(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_aarch64)/libdotlottie_player.so
	@cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(HOST_TAG)/sysroot/usr/lib/$(LIBCPP_PATH_aarch64)/$(LIBCPP_SHARED_LIB) \
		$(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_aarch64)/
	@if command -v $(ANDROID_STRIP) >/dev/null 2>&1; then \
		$(ANDROID_STRIP) --strip-unneeded $(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_aarch64)/$(LIBCPP_SHARED_LIB); \
		$(ANDROID_STRIP) --strip-unneeded $(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_aarch64)/libdotlottie_player.so; \
	fi
	@echo "✓ Android aarch64 build complete"

# Build for Android x86_64 with C API
android-x86_64: android-check-ndk
	$(call check_android_platform_support)
	@echo "→ Building Android x86_64 ..."
	@ANDROID_NDK_HOME="$(ANDROID_NDK_HOME)" \
	CC="$(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang" \
	CXX="$(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang++" \
	CLANG_PATH="$(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang" \
	CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang" \
	AR="$(ANDROID_AR)" \
	RANLIB="$(ANDROID_RANLIB)" \
	BINDGEN_EXTRA_CLANG_ARGS="-isysroot $(ANDROID_TOOLCHAIN)/sysroot" \
	cargo build \
		--manifest-path dotlottie-rs/Cargo.toml \
		--target $(RUST_TARGET_x86_64) \
		--release \
		--no-default-features \
		--features $(ANDROID_DEFAULT_FEATURES),$(ANDROID_FEATURES)
	@mkdir -p $(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_x86_64)
	@cp dotlottie-rs/target/$(RUST_TARGET_x86_64)/release/$(ANDROID_LIB_NAME) \
		$(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_x86_64)/libdotlottie_player.so
	@cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(HOST_TAG)/sysroot/usr/lib/$(LIBCPP_PATH_x86_64)/$(LIBCPP_SHARED_LIB) \
		$(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_x86_64)/
	@if command -v $(ANDROID_STRIP) >/dev/null 2>&1; then \
		$(ANDROID_STRIP) --strip-unneeded $(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_x86_64)/$(LIBCPP_SHARED_LIB); \
		$(ANDROID_STRIP) --strip-unneeded $(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_x86_64)/libdotlottie_player.so; \
	fi
	@echo "✓ Android x86_64 build complete"

# Build for Android x86 with C API
android-x86: android-check-ndk
	$(call check_android_platform_support)
	@echo "→ Building Android x86 ..."
	@ANDROID_NDK_HOME="$(ANDROID_NDK_HOME)" \
	CC="$(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang" \
	CXX="$(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang++" \
	CLANG_PATH="$(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang" \
	CARGO_TARGET_I686_LINUX_ANDROID_LINKER="$(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang" \
	AR="$(ANDROID_AR)" \
	RANLIB="$(ANDROID_RANLIB)" \
	BINDGEN_EXTRA_CLANG_ARGS="-isysroot $(ANDROID_TOOLCHAIN)/sysroot" \
	cargo build \
		--manifest-path dotlottie-rs/Cargo.toml \
		--target $(RUST_TARGET_x86) \
		--release \
		--no-default-features \
		--features $(ANDROID_DEFAULT_FEATURES),$(ANDROID_FEATURES)
	@mkdir -p $(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_x86)
	@cp dotlottie-rs/target/$(RUST_TARGET_x86)/release/$(ANDROID_LIB_NAME) \
		$(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_x86)/libdotlottie_player.so
	@cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(HOST_TAG)/sysroot/usr/lib/$(LIBCPP_PATH_x86)/$(LIBCPP_SHARED_LIB) \
		$(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_x86)/
	@if command -v $(ANDROID_STRIP) >/dev/null 2>&1; then \
		$(ANDROID_STRIP) --strip-unneeded $(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_x86)/$(LIBCPP_SHARED_LIB); \
		$(ANDROID_STRIP) --strip-unneeded $(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_x86)/libdotlottie_player.so; \
	fi
	@echo "✓ Android x86 build complete"

# Build for Android ARMv7 with C API
android-armv7: android-check-ndk
	$(call check_android_platform_support)
	@echo "→ Building Android ARMv7 ..."
	@ANDROID_NDK_HOME="$(ANDROID_NDK_HOME)" \
	CC="$(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang" \
	CXX="$(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang++" \
	CLANG_PATH="$(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang" \
	CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang" \
	AR="$(ANDROID_AR)" \
	RANLIB="$(ANDROID_RANLIB)" \
	BINDGEN_EXTRA_CLANG_ARGS="-isysroot $(ANDROID_TOOLCHAIN)/sysroot" \
	cargo build \
		--manifest-path dotlottie-rs/Cargo.toml \
		--target $(RUST_TARGET_armv7) \
		--release \
		--no-default-features \
		--features $(ANDROID_DEFAULT_FEATURES),$(ANDROID_FEATURES)
	@mkdir -p $(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_armv7)
	@cp dotlottie-rs/target/$(RUST_TARGET_armv7)/release/$(ANDROID_LIB_NAME) \
		$(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_armv7)/libdotlottie_player.so
	@cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(HOST_TAG)/sysroot/usr/lib/$(LIBCPP_PATH_armv7)/$(LIBCPP_SHARED_LIB) \
		$(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_armv7)/
	@if command -v $(ANDROID_STRIP) >/dev/null 2>&1; then \
		$(ANDROID_STRIP) --strip-unneeded $(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_armv7)/$(LIBCPP_SHARED_LIB); \
		$(ANDROID_STRIP) --strip-unneeded $(ANDROID_RELEASE_DIR)/jniLibs/$(ANDROID_ABI_armv7)/libdotlottie_player.so; \
	fi
	@echo "✓ Android ARMv7 build complete"

# Package Android C API build with bindings generation
android-package:
	@echo "→ Creating Android  release package..."
	@mkdir -p $(ANDROID_RELEASE_DIR)/include
	@echo "→ Generating C header with cbindgen..."
	@cbindgen --config dotlottie-rs/cbindgen.toml \
		--crate dotlottie-rs \
		--output $(ANDROID_RELEASE_DIR)/include/dotlottie_player.h \
		dotlottie-rs
	@echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(ANDROID_RELEASE_DIR)/version.txt
	@echo "api-type=c-api" >> $(ANDROID_RELEASE_DIR)/version.txt
	@echo "✓ Android release package created: $(ANDROID_RELEASE_DIR)/"
	@echo ""
	@echo "Output structure:"
	@echo "  $(ANDROID_RELEASE_DIR)/"
	@echo "    ├── include/dotlottie_player.h   (C header - generated with cbindgen)"
	@echo "    ├── jniLibs/"
	@echo "    │   ├── arm64-v8a/libdotlottie_player.so"
	@echo "    │   ├── armeabi-v7a/libdotlottie_player.so"
	@echo "    │   ├── x86/libdotlottie_player.so"
	@echo "    │   └── x86_64/libdotlottie_player.so"
	@echo "    └── version.txt"
