UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Defaults to the Homebrew NDK on macOS. Set ANDROID_NDK_HOME for any other location.
ifeq ($(UNAME_S),Darwin)
    ANDROID_NDK_HOME ?= /opt/homebrew/share/android-ndk
else ifeq ($(UNAME_S),Linux)
    ANDROID_NDK_HOME ?= /opt/android-ndk
else
    # No default on other hosts. check_android_platform_support requires ANDROID_NDK_HOME.
    ANDROID_NDK_HOME ?=
endif
API_LEVEL ?= 21

MIN_NDK_VERSION = 28

ANDROID_FEATURES ?= tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-otf,tvg-lottie-expressions,tvg-threads

ifdef FEATURES
	ANDROID_FEATURES = $(FEATURES)
endif

RELEASE_DIR ?= release
ANDROID_RELEASE_DIR ?= $(RELEASE_DIR)/android
LIBCPP_SHARED_LIB ?= libc++_shared.so

define check_android_platform_support
$(if $(ANDROID_NDK_HOME),,$(error "Please set ANDROID_NDK_HOME to your Android NDK installation path."))
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
    # Placeholder. check_android_platform_support rejects unsupported hosts.
    HOST_TAG = linux-x86_64
endif

ANDROID_TOOLCHAIN = $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(HOST_TAG)
ANDROID_AR = $(ANDROID_TOOLCHAIN)/bin/llvm-ar
ANDROID_RANLIB = $(ANDROID_TOOLCHAIN)/bin/llvm-ranlib
ANDROID_STRIP = $(ANDROID_TOOLCHAIN)/bin/llvm-strip

ANDROID_TARGETS = aarch64-linux-android x86_64-linux-android i686-linux-android armv7-linux-androideabi

RUST_TARGET_aarch64 = aarch64-linux-android
RUST_TARGET_x86_64 = x86_64-linux-android
RUST_TARGET_x86 = i686-linux-android
RUST_TARGET_armv7 = armv7-linux-androideabi

ANDROID_ABI_aarch64 = arm64-v8a
ANDROID_ABI_x86_64 = x86_64
ANDROID_ABI_x86 = x86
ANDROID_ABI_armv7 = armeabi-v7a

LIBCPP_PATH_aarch64 = aarch64-linux-android
LIBCPP_PATH_x86_64 = x86_64-linux-android
LIBCPP_PATH_x86 = i686-linux-android
LIBCPP_PATH_armv7 = arm-linux-androideabi

ANDROID_CXX_aarch64 = $(ANDROID_TOOLCHAIN)/bin/aarch64-linux-android$(API_LEVEL)-clang++
ANDROID_CXX_x86_64 = $(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang++
ANDROID_CXX_x86 = $(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang++
ANDROID_CXX_armv7 = $(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang++

CRATE_VERSION = $(shell grep -m 1 'version =' dotlottie-rs/Cargo.toml | grep -o '[0-9][0-9.]*')
COMMIT_HASH := $(shell git rev-parse --short HEAD)

.PHONY: android android-aarch64 android-x86_64 android-x86 android-armv7 android-package android-setup android-clean

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

android-setup:
	@echo "→ Installing Android Rust targets..."
	@rustup target add $(ANDROID_TARGETS) >/dev/null
	@echo "✓ Android targets installed"

android-clean:
	@echo "→ Cleaning Android builds..."
	@rm -rf $(ANDROID_RELEASE_DIR)
	@echo "✓ Android builds cleaned"

ANDROID_DEFAULT_FEATURES = tvg,tvg-cpu,tvg-gl,c_api,dotlottie,state-machines,theming
ANDROID_LIB_NAME = libdotlottie_rs.so

android: $(addprefix android-,aarch64 x86_64 x86 armv7) android-package
	@echo "✓ All Android builds complete"

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
	cargo rustc \
		--manifest-path dotlottie-rs/Cargo.toml \
		--crate-type cdylib \
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

android-x86_64: android-check-ndk
	$(call check_android_platform_support)
	@echo "→ Building Android x86_64..."
	@ANDROID_NDK_HOME="$(ANDROID_NDK_HOME)" \
	CC="$(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang" \
	CXX="$(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang++" \
	CLANG_PATH="$(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang" \
	CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang" \
	AR="$(ANDROID_AR)" \
	RANLIB="$(ANDROID_RANLIB)" \
	BINDGEN_EXTRA_CLANG_ARGS="-isysroot $(ANDROID_TOOLCHAIN)/sysroot" \
	cargo rustc \
		--manifest-path dotlottie-rs/Cargo.toml \
		--crate-type cdylib \
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

android-x86: android-check-ndk
	$(call check_android_platform_support)
	@echo "→ Building Android x86..."
	@ANDROID_NDK_HOME="$(ANDROID_NDK_HOME)" \
	CC="$(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang" \
	CXX="$(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang++" \
	CLANG_PATH="$(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang" \
	CARGO_TARGET_I686_LINUX_ANDROID_LINKER="$(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang" \
	AR="$(ANDROID_AR)" \
	RANLIB="$(ANDROID_RANLIB)" \
	BINDGEN_EXTRA_CLANG_ARGS="-isysroot $(ANDROID_TOOLCHAIN)/sysroot" \
	cargo rustc \
		--manifest-path dotlottie-rs/Cargo.toml \
		--crate-type cdylib \
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

android-armv7: android-check-ndk
	$(call check_android_platform_support)
	@echo "→ Building Android ARMv7..."
	@ANDROID_NDK_HOME="$(ANDROID_NDK_HOME)" \
	CC="$(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang" \
	CXX="$(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang++" \
	CLANG_PATH="$(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang" \
	CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang" \
	AR="$(ANDROID_AR)" \
	RANLIB="$(ANDROID_RANLIB)" \
	BINDGEN_EXTRA_CLANG_ARGS="-isysroot $(ANDROID_TOOLCHAIN)/sysroot" \
	cargo rustc \
		--manifest-path dotlottie-rs/Cargo.toml \
		--crate-type cdylib \
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

android-package:
	@echo "→ Creating Android release package..."
	@mkdir -p $(ANDROID_RELEASE_DIR)/include
	@echo "→ Copying C header from build..."
	@cp dotlottie-rs/build/dotlottie_player.h $(ANDROID_RELEASE_DIR)/include/
	@echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(ANDROID_RELEASE_DIR)/version.txt
	@echo "api-type=c-api" >> $(ANDROID_RELEASE_DIR)/version.txt
	@echo "✓ Android release package created: $(ANDROID_RELEASE_DIR)/"
	@echo ""
	@echo "Output structure:"
	@echo "  $(ANDROID_RELEASE_DIR)/"
	@echo "    ├── include/dotlottie_player.h"
	@echo "    ├── jniLibs/"
	@echo "    │   ├── arm64-v8a/libdotlottie_player.so"
	@echo "    │   ├── armeabi-v7a/libdotlottie_player.so"
	@echo "    │   ├── x86/libdotlottie_player.so"
	@echo "    │   └── x86_64/libdotlottie_player.so"
	@echo "    └── version.txt"
