ANDROID_NDK_HOME ?= /opt/homebrew/share/android-ndk
API_LEVEL ?= 21

# Minimum required NDK version
MIN_NDK_VERSION = 28

# Default Rust features for Android builds
ANDROID_FEATURES ?= uniffi,thorvg,thorvg_webp,thorvg_png,thorvg_jpg,thorvg_ttf,thorvg_lottie_expressions

# UniFFI Bindings
BINDINGS_DIR ?= bindings
KOTLIN_BINDINGS_DIR ?= $(BINDINGS_DIR)/kotlin

# Release and packaging variables
RELEASE_DIR ?= release
ANDROID_RELEASE_DIR ?= $(RELEASE_DIR)/android
DOTLOTTIE_PLAYER_DIR ?= dotlottie-player
DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR ?= $(ANDROID_RELEASE_DIR)/$(DOTLOTTIE_PLAYER_DIR)
DOTLOTTIE_PLAYER_ANDROID_SRC_DIR ?= $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/kotlin

# Library names and paths
RUNTIME_FFI_LIB_BASE ?= libdotlottie_player
RUNTIME_FFI_LIB ?= $(RUNTIME_FFI_LIB_BASE).so
DOTLOTTIE_PLAYER_LIB ?= libuniffi_dotlottie_player.so
LIBCPP_SHARED_LIB ?= libc++_shared.so

# Assets
GRADLE_PROPERTIES ?= gradle.properties

# Detect host tag for NDK
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

ifeq ($(UNAME_S),Darwin)
    ifeq ($(UNAME_M),arm64)
        HOST_TAG = darwin-x86_64
    else
        HOST_TAG = darwin-x86_64
    endif
else ifeq ($(UNAME_S),Linux)
    HOST_TAG = linux-x86_64
else
    $(error "Unsupported host platform: $(UNAME_S)")
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
CRATE_VERSION := $(shell grep -m 1 version Cargo.toml | sed 's/.*"\([0-9.]\+\)"/\1/')
COMMIT_HASH := $(shell git rev-parse --short HEAD)

# Android packaging function
define ANDROID_PACKAGE_ARCH
	@echo "Packaging Android $(1) build..."
	@mkdir -p $(DOTLOTTIE_PLAYER_ANDROID_SRC_DIR)
	@mkdir -p $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/jniLibs/$(ANDROID_ABI_$(1))
	
	# Copy Kotlin bindings
	@if [ -d "$(KOTLIN_BINDINGS_DIR)" ] && [ -n "$$(ls -A $(KOTLIN_BINDINGS_DIR) 2>/dev/null)" ]; then \
		cp -r $(KOTLIN_BINDINGS_DIR)/* $(DOTLOTTIE_PLAYER_ANDROID_SRC_DIR)/; \
		echo "Copied Kotlin bindings to $(DOTLOTTIE_PLAYER_ANDROID_SRC_DIR)"; \
	fi
	
	# Copy and strip main library (rename libdotlottie_player.so to libuniffi_dotlottie_player.so)
	@if [ -f "target/$(RUST_TARGET_$(1))/release/$(RUNTIME_FFI_LIB)" ]; then \
		cp target/$(RUST_TARGET_$(1))/release/$(RUNTIME_FFI_LIB) \
		   $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/jniLibs/$(ANDROID_ABI_$(1))/$(DOTLOTTIE_PLAYER_LIB).temp; \
		$(ANDROID_STRIP) --strip-unneeded \
		   $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/jniLibs/$(ANDROID_ABI_$(1))/$(DOTLOTTIE_PLAYER_LIB).temp; \
		mv $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/jniLibs/$(ANDROID_ABI_$(1))/$(DOTLOTTIE_PLAYER_LIB).temp \
		   $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/jniLibs/$(ANDROID_ABI_$(1))/$(DOTLOTTIE_PLAYER_LIB); \
		echo "Copied and stripped main library for $(1) ($(RUNTIME_FFI_LIB) -> $(DOTLOTTIE_PLAYER_LIB))"; \
	else \
		echo "Warning: $(RUNTIME_FFI_LIB) not found in target/$(RUST_TARGET_$(1))/release/"; \
	fi
	
	# Copy and strip libc++ shared library
	@if [ -f "$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(HOST_TAG)/sysroot/usr/lib/$(LIBCPP_PATH_$(1))/$(LIBCPP_SHARED_LIB)" ]; then \
		cp $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(HOST_TAG)/sysroot/usr/lib/$(LIBCPP_PATH_$(1))/$(LIBCPP_SHARED_LIB) \
		   $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/jniLibs/$(ANDROID_ABI_$(1))/$(LIBCPP_SHARED_LIB).temp; \
		$(ANDROID_STRIP) --strip-unneeded \
		   $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/jniLibs/$(ANDROID_ABI_$(1))/$(LIBCPP_SHARED_LIB).temp; \
		mv $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/jniLibs/$(ANDROID_ABI_$(1))/$(LIBCPP_SHARED_LIB).temp \
		   $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/jniLibs/$(ANDROID_ABI_$(1))/$(LIBCPP_SHARED_LIB); \
		echo "Copied and stripped libc++ shared library for $(1)"; \
	fi
	
	# Create gradle properties file
	@echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/$(GRADLE_PROPERTIES)
	@echo "Created gradle.properties with version $(CRATE_VERSION)-$(COMMIT_HASH)"
	
	@echo "Android $(1) packaging completed in $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)"
endef

# Android-specific phony targets
.PHONY: android android-aarch64 android-x86_64 android-x86 android-armv7 install-android-targets android-clean

# Generate Kotlin UniFFI bindings
kotlin-bindings:
	@echo "Generating Kotlin UniFFI bindings..."
	@mkdir -p $(KOTLIN_BINDINGS_DIR)
	rm -rf $(KOTLIN_BINDINGS_DIR)/*
	cargo run \
		--release \
		--no-default-features \
		--features=uniffi/cli,thorvg,uniffi \
		--bin uniffi-bindgen \
		generate src/dotlottie_player.udl \
		--language kotlin \
		--out-dir $(KOTLIN_BINDINGS_DIR)
	@echo "Kotlin bindings generated in $(KOTLIN_BINDINGS_DIR)"

# Build for all Android architectures (with bindings and packaging)
android: kotlin-bindings $(addprefix android-,aarch64 x86_64 x86 armv7)
	@echo "All Android builds completed and packaged!"

# Build for Android ARM64
android-aarch64: kotlin-bindings android-check-ndk
	@echo "Building dotlottie-ffi for Android aarch64..."
	@echo "Using NDK: $(ANDROID_NDK_HOME)"
	@echo "API Level: $(API_LEVEL)"
	@echo "Host Tag: $(HOST_TAG)"
	@echo "CXX: $(ANDROID_CXX_aarch64)"
	ANDROID_NDK_HOME="$(ANDROID_NDK_HOME)" \
	CC="$(ANDROID_TOOLCHAIN)/bin/aarch64-linux-android$(API_LEVEL)-clang" \
	CXX="$(ANDROID_TOOLCHAIN)/bin/aarch64-linux-android$(API_LEVEL)-clang++" \
	CLANG_PATH="$(ANDROID_TOOLCHAIN)/bin/aarch64-linux-android$(API_LEVEL)-clang" \
	CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$(ANDROID_TOOLCHAIN)/bin/aarch64-linux-android$(API_LEVEL)-clang" \
	AR="$(ANDROID_AR)" \
	RANLIB="$(ANDROID_RANLIB)" \
	BINDGEN_EXTRA_CLANG_ARGS="-isysroot $(ANDROID_TOOLCHAIN)/sysroot" \
	cargo build  \
		--target $(RUST_TARGET_aarch64) \
		--release \
		--no-default-features \
		--features $(ANDROID_FEATURES)
	$(call ANDROID_PACKAGE_ARCH,aarch64)

# Build for Android x86_64
android-x86_64: kotlin-bindings android-check-ndk
	@echo "Building dotlottie-ffi for Android x86_64..."
	@echo "Using NDK: $(ANDROID_NDK_HOME)"
	@echo "API Level: $(API_LEVEL)"
	@echo "Host Tag: $(HOST_TAG)"
	@echo "CXX: $(ANDROID_CXX_x86_64)"
	ANDROID_NDK_HOME="$(ANDROID_NDK_HOME)" \
	CC="$(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang" \
	CXX="$(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang++" \
	CLANG_PATH="$(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang" \
	CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$(ANDROID_TOOLCHAIN)/bin/x86_64-linux-android$(API_LEVEL)-clang" \
	AR="$(ANDROID_AR)" \
	RANLIB="$(ANDROID_RANLIB)" \
	BINDGEN_EXTRA_CLANG_ARGS="-isysroot $(ANDROID_TOOLCHAIN)/sysroot" \
	cargo build  \
		--target $(RUST_TARGET_x86_64) \
		--release \
		--no-default-features \
		--features $(ANDROID_FEATURES)
	$(call ANDROID_PACKAGE_ARCH,x86_64)

# Build for Android x86
android-x86: kotlin-bindings android-check-ndk
	@echo "Building dotlottie-ffi for Android x86..."
	@echo "Using NDK: $(ANDROID_NDK_HOME)"
	@echo "API Level: $(API_LEVEL)"
	@echo "Host Tag: $(HOST_TAG)"
	@echo "CXX: $(ANDROID_CXX_x86)"
	ANDROID_NDK_HOME="$(ANDROID_NDK_HOME)" \
	CC="$(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang" \
	CXX="$(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang++" \
	CLANG_PATH="$(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang" \
	CARGO_TARGET_I686_LINUX_ANDROID_LINKER="$(ANDROID_TOOLCHAIN)/bin/i686-linux-android$(API_LEVEL)-clang" \
	AR="$(ANDROID_AR)" \
	RANLIB="$(ANDROID_RANLIB)" \
	BINDGEN_EXTRA_CLANG_ARGS="-isysroot $(ANDROID_TOOLCHAIN)/sysroot" \
	cargo build  \
		--target $(RUST_TARGET_x86) \
		--release \
		--no-default-features \
		--features $(ANDROID_FEATURES)
	$(call ANDROID_PACKAGE_ARCH,x86)

# Build for Android ARMv7
android-armv7: kotlin-bindings android-check-ndk
	@echo "Building dotlottie-ffi for Android ARMv7..."
	@echo "Using NDK: $(ANDROID_NDK_HOME)"
	@echo "API Level: $(API_LEVEL)"
	@echo "Host Tag: $(HOST_TAG)"
	@echo "CXX: $(ANDROID_CXX_armv7)"
	ANDROID_NDK_HOME="$(ANDROID_NDK_HOME)" \
	CC="$(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang" \
	CXX="$(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang++" \
	CLANG_PATH="$(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang" \
	CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$(ANDROID_TOOLCHAIN)/bin/armv7a-linux-androideabi$(API_LEVEL)-clang" \
	AR="$(ANDROID_AR)" \
	RANLIB="$(ANDROID_RANLIB)" \
	BINDGEN_EXTRA_CLANG_ARGS="-isysroot $(ANDROID_TOOLCHAIN)/sysroot" \
	cargo build  \
		--target $(RUST_TARGET_armv7) \
		--release \
		--no-default-features \
		--features $(ANDROID_FEATURES)
	$(call ANDROID_PACKAGE_ARCH,armv7)

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
install-android-targets:
	@echo "Installing Android Rust targets..."
	rustup target add $(ANDROID_TARGETS)



# Clean Android bindings and release artifacts
android-clean:
	rm -rf $(KOTLIN_BINDINGS_DIR)
	rm -rf $(BINDINGS_DIR)
	rm -rf $(ANDROID_RELEASE_DIR)
	@echo "Cleaned Android bindings and release directory"