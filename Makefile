.DEFAULT_GOAL := help

# Some basic helpers
define n


endef

# Formatting
RED := $(shell tput setaf 1)
YELLOW := $(shell tput setaf 3)
GREEN := $(shell tput setaf 2)
NC := $(shell tput sgr0)

# Build system types
LINUX_BUILD_PLATFORM := linux
MAC_BUILD_PLATFORM := darwin

X64_64_ARCH := x86_64
AARCH64_ARCH := aarch64
ARM_ARCH := arm64

BUILD_PLATFORM := $(shell uname -s | tr '[:upper:]' '[:lower:]')
ifeq ($(filter $(BUILD_PLATFORM),$(LINUX_BUILD_PLATFORM) $(MAC_BUILD_PLATFORM)),)
  $(error $n $(RED)ERROR$(NC): Your platform ($(GREEN)$(BUILD_PLATFORM)$(NC)) is unrecognized, cannot continue)
endif
BUILD_PLATFORM_ARCH := $(shell uname -m)
ifeq ($(filter $(BUILD_PLATFORM_ARCH),$(X64_64_ARCH) $(AARCH64_ARCH) $(ARM_ARCH)),)
  $(error $n $(RED)ERROR$(NC): Your platform architecture ($(GREEN)$(BUILD_PLATFORM_ARCH)$(NC)) is not supported, cannot continue)
endif

# Android
ANDROID_BUILD_PLATFORM := $(BUILD_PLATFORM)-x86_64
ANDROID_NDK_HOME ?= /opt/homebrew/share/android-ndk
ifeq ($(wildcard $(ANDROID_NDK_HOME)/*),)
  $(error $n $(RED)ERROR$(NC): The $(GREEN)ANDROID_NDK_HOME$(NC) ($(ANDROID_NDK_HOME)) environment variable is not set to a usable directory.$n\
             You will need to install it, if necessary, and export or specify this environment variable with its location)
endif
ANDROID_API_VERSION ?= 24

# Android Tool chain
AR := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/llvm-ar
AS := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/llvm-as
RANLIB := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/llvm-ranlib
LD := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/ld
STRIP := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/llvm-strip

# Directories for external dependencies and their builds
DEPS_DIR := deps
DEPS_MODULES_DIR := $(DEPS_DIR)/modules
DEPS_BUILD_DIR := $(DEPS_DIR)/build
DEPS_ARTIFACTS_DIR := $(DEPS_DIR)/artifacts

# External dependencies
THORVG := thorvg
LIBJPEG_TURBO := libjpeg-turbo
LIBPNG := libpng
ZLIB := zlib

# External dependency artifacts
THORVG_CROSS_FILE := cross.txt
THORVG_NINJA_BUILD_FILE := build.ninja
THORVG_LIB := libthorvg.a

CMAKE_TOOLCHAIN_FILE := toolchain.cmake
CMAKE_MAKEFILE := Makefile

LIBPNG_LIB := libpng.a
LIBJPEG_TURBO_LIB := libturbojpeg.a
ZLIB_LIB := libz.a

# Release artifacts will be placed in this directory
RELEASE_DIR := release

# Build artifact types
CORE := core
RUNTIME_FFI := runtime-ffi
DOTLOTTIE_PLAYER := dotlottie-player

# Build artifacts
RUNTIME_FFI_UNIFFI_BINDINGS := uniffi-bindings
RUNTIME_FFI_LIB := libdlplayer.so
RUNTIME_FFI_ASSETS := assets

DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR := $(RELEASE_DIR)/android/$(DOTLOTTIE_PLAYER)
DOTLOTTIE_PLAYER_ANDROID_SRC_DIR := $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/kotlin
DOTLOTTIE_PLAYER_LIB := libuniffi_dotlottie_player.so
DOTLOTTIE_PLAYER_GRADLE_PROPERTIES := gradle.properties

# Dependency build directories for the current machine architecture
LOCAL_ARCH := local-arch

LOCAL_ARCH_BUILD_DIR := $(DEPS_BUILD_DIR)/$(LOCAL_ARCH)
LOCAL_ARCH_ARTIFACTS_DIR := $(DEPS_ARTIFACTS_DIR)/$(LOCAL_ARCH)/usr
LOCAL_ARCH_INCLUDE_DIR := $(LOCAL_ARCH_ARTIFACTS_DIR)/include
LOCAL_ARCH_LIB_DIR := $(LOCAL_ARCH_ARTIFACTS_DIR)/lib

THORVG_LOCAL_ARCH_BUILD_DIR := $(LOCAL_ARCH_BUILD_DIR)/$(THORVG)/build
LIBJPEG_TURBO_LOCAL_ARCH_BUILD_DIR := $(LOCAL_ARCH_BUILD_DIR)/$(LIBJPEG_TURBO)/build
LIBPNG_LOCAL_ARCH_BUILD_DIR := $(LOCAL_ARCH_BUILD_DIR)/$(LIBPNG)/build
ZLIB_LOCAL_ARCH_BUILD_DIR := $(LOCAL_ARCH_BUILD_DIR)/$(ZLIB)/build

# Other build flags for dependencies
ZLIB_LDFLAGS := -Wl,--undefined-version

# Sources
CORE_SRC := $(shell find $(CORE)/src -name "*.rs")
RUNTIME_FFI_SRC := $(shell find $(RUNTIME_FFI)/src -name "*.rs") $(shell find $(RUNTIME_FFI)/src -name "*.udl")

# Helper functions
define ANDROID_CROSS_FILE
[binaries]
cpp        = '$(CPP)'
ar         = '$(AR)'
as         = '$(AS)'
ranlib     = '$(RANLIB)'
ld         = '$(LD)'
strip      = '$(STRIP)'
pkg-config = 'pkg-config'

[host_machine]
system = 'android'
cpu_family = '$(CPU_FAMILY)'
cpu = '$(CPU)'
endian = 'little'
endef

# Helper functions
define ANDROID_CMAKE_TOOLCHAIN_FILE
set(CMAKE_SYSTEM_NAME Android)
set(CMAKE_SYSTEM_VERSION $(ANDROID_API_VERSION))
set(CMAKE_ANDROID_ARCH_ABI $(ANDROID_ARCH_ABI))
set(CMAKE_ANDROID_NDK $(ANDROID_NDK_HOME))
endef

define CREATE_CROSS_FILE
	mkdir -p $(DEP_BUILD_DIR)
	echo "$$CROSS_FILE" > $@
endef

define SETUP_MESON
	meson setup \
		--prefix=/ \
		--backend=ninja \
		-Dlog=true \
		-Dloaders="lottie, png, jpg" \
		-Ddefault_library=static \
		-Dsavers=all \
		-Dbindings=capi \
		$(CROSS_FILE) "$(THORVG_DEP_SOURCE_DIR)" "$(THORVG_DEP_BUILD_DIR)"
endef

define NINJA_BUILD
	DESTDIR=$(ARTIFACTS_DIR) ninja -C $(DEP_BUILD_DIR) install
endef

define SETUP_CMAKE
	cmake -DCMAKE_INSTALL_PREFIX=$(DEP_ARTIFACTS_DIR) -DCMAKE_POSITION_INDEPENDENT_CODE=ON -DBUILD_SHARED_LIBS=OFF $(TOOLCHAIN_FILE) \
		-B $(DEP_BUILD_DIR) \
		$(DEP_SOURCE_DIR)
endef

define CMAKE_BUILD
  $(MAKE) -C $(CMAKE_BUILD_DIR) install
endef

define CARGO_BUILD
	cargo build \
	--manifest-path $(PROJECT_DIR)/Cargo.toml \
	--target $(CARGO_TARGET) \
	--release
endef

define ANDROID_RELEASE
  mkdir -p $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR) $(DOTLOTTIE_PLAYER_ANDROID_SRC_DIR) $(DOTLOTTIE_PLAYER_LIB_DIR)
  cp -r $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/* $(DOTLOTTIE_PLAYER_ANDROID_SRC_DIR)
  cp $(RUNTIME_FFI_TARGET_LIB) $(DOTLOTTIE_PLAYER_LIB_DIR)/$(DOTLOTTIE_PLAYER_LIB)
  cp $(RUNTIME_FFI)/assets/android/* $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)
  echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/$(DOTLOTTIE_PLAYER_GRADLE_PROPERTIES)
endef

define NEW_BUILD_TARGET
# Setup architecture variables
$2 := $1
$2_ANDROID_ARCH := $3
$2_ANDROID_ABI := $4
$2_CPU_FAMILY := $5
$2_CPU := $6

# Setup dependency build variables
$2_DEPS_BUILD_DIR := $(DEPS_BUILD_DIR)/$1

$2_THORVG_DEP_BUILD_DIR := $$($2_DEPS_BUILD_DIR)/$(THORVG)
$2_LIBJPEG_TURBO_DEP_BUILD_DIR := $$($2_DEPS_BUILD_DIR)/$(LIBJPEG_TURBO)
$2_LIBPNG_DEP_BUILD_DIR := $$($2_DEPS_BUILD_DIR)/$(LIBPNG)
$2_ZLIB_DEP_BUILD_DIR := $$($2_DEPS_BUILD_DIR)/$(ZLIB)

$2_DEPS_ARTIFACTS_DIR := $(DEPS_ARTIFACTS_DIR)/$1/usr
$2_DEPS_INCLUDE_DIR := $$($2_DEPS_ARTIFACTS_DIR)/include
$2_DEPS_LIB_DIR := $$($2_DEPS_ARTIFACTS_DIR)/lib

# Setup final artifact variables
$2_RUNTIME_FFI_DEPS_BUILD_DIR := $(RUNTIME_FFI)/target/$1/release
$2_DOTLOTTIE_PLAYER_LIB_DIR := $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/jniLibs/$$($2_ANDROID_ABI)
endef

define NEW_CMAKE_BUILD
# Create toolchain file
$4/../$(CMAKE_TOOLCHAIN_FILE): DEP_BUILD_DIR := $4
$4/../$(CMAKE_TOOLCHAIN_FILE): ANDROID_ARCH_ABI := $$($1_ANDROID_ABI)
$4/../$(CMAKE_TOOLCHAIN_FILE): export CROSS_FILE := $$(ANDROID_CMAKE_TOOLCHAIN_FILE)
$4/../$(CMAKE_TOOLCHAIN_FILE):
	$$(CREATE_CROSS_FILE)

# Setup cmake
$4/$(CMAKE_MAKEFILE): export LDFLAGS := $$($2_LDFLAGS)
$4/$(CMAKE_MAKEFILE): DEP_SOURCE_DIR := $(DEPS_MODULES_DIR)/$3
$4/$(CMAKE_MAKEFILE): DEP_BUILD_DIR := $4
$4/$(CMAKE_MAKEFILE): DEP_ARTIFACTS_DIR := $$($1_DEPS_ARTIFACTS_DIR)
$4/$(CMAKE_MAKEFILE): TOOLCHAIN_FILE := -DCMAKE_TOOLCHAIN_FILE=../$(CMAKE_TOOLCHAIN_FILE)
$4/$(CMAKE_MAKEFILE): $4/../$(CMAKE_TOOLCHAIN_FILE)
	$$(SETUP_CMAKE)

# Build
$$($1_DEPS_LIB_DIR)/$5: CMAKE_BUILD_DIR := $4
$$($1_DEPS_LIB_DIR)/$5: $4/$(CMAKE_MAKEFILE)
	$$(CMAKE_BUILD)
endef

define NEW_DEPS_BUILD
$(eval $(call NEW_CMAKE_BUILD,$1,LIBJPEG_TURBO,$(LIBJPEG_TURBO),$$($1_LIBJPEG_TURBO_DEP_BUILD_DIR),$(LIBJPEG_TURBO_LIB)))
$(eval $(call NEW_CMAKE_BUILD,$1,LIBPNG_LIB,$(LIBPNG),$$($1_LIBPNG_DEP_BUILD_DIR),$(LIBPNG_LIB)))
$(eval $(call NEW_CMAKE_BUILD,$1,ZLIB,$(ZLIB),$$($1_ZLIB_DEP_BUILD_DIR),$(ZLIB_LIB)))

# Create cross file for thorvg
$$($1_THORVG_DEP_BUILD_DIR)/../$(THORVG_CROSS_FILE): SYSROOT := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/sysroot/usr/lib/$$($1_ANDROID_ARCH)/$(ANDROID_API_VERSION)
$$($1_THORVG_DEP_BUILD_DIR)/../$(THORVG_CROSS_FILE): CPP := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/$$($1_ANDROID_ARCH)$(ANDROID_API_VERSION)-clang++
$$($1_THORVG_DEP_BUILD_DIR)/../$(THORVG_CROSS_FILE): CPU_FAMILY := $$($1_CPU_FAMILY)
$$($1_THORVG_DEP_BUILD_DIR)/../$(THORVG_CROSS_FILE): CPU := $$($1_CPU)
$$($1_THORVG_DEP_BUILD_DIR)/../$(THORVG_CROSS_FILE): DEP_BUILD_DIR := $$($1_THORVG_DEP_BUILD_DIR)
$$($1_THORVG_DEP_BUILD_DIR)/../$(THORVG_CROSS_FILE): export CROSS_FILE := $$(ANDROID_CROSS_FILE)
$$($1_THORVG_DEP_BUILD_DIR)/../$(THORVG_CROSS_FILE):
	$$(CREATE_CROSS_FILE)

# Setup meson for thorvg
$$($1_THORVG_DEP_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): export PKG_CONFIG_PATH := $(PWD)/$$($1_DEPS_LIB_DIR)/pkgconfig
$$($1_THORVG_DEP_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): THORVG_DEP_SOURCE_DIR := $(DEPS_MODULES_DIR)/$(THORVG)
$$($1_THORVG_DEP_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): THORVG_DEP_BUILD_DIR := $$($1_THORVG_DEP_BUILD_DIR)
$$($1_THORVG_DEP_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): CROSS_FILE := --cross-file $$($1_THORVG_DEP_BUILD_DIR)/../$(THORVG_CROSS_FILE)
$$($1_THORVG_DEP_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): $$($1_THORVG_DEP_BUILD_DIR)/../$(THORVG_CROSS_FILE)
$$($1_THORVG_DEP_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): $$($1_DEPS_LIB_DIR)/$(LIBJPEG_TURBO_LIB)
$$($1_THORVG_DEP_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): $$($1_DEPS_LIB_DIR)/$(LIBPNG_LIB)
$$($1_THORVG_DEP_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): $$($1_DEPS_LIB_DIR)/$(ZLIB_LIB)
	$$(SETUP_MESON)

# Build thorvg
$$($1_DEPS_LIB_DIR)/$(THORVG_LIB): DEP_BUILD_DIR := $$($1_THORVG_DEP_BUILD_DIR)
$$($1_DEPS_LIB_DIR)/$(THORVG_LIB): ARTIFACTS_DIR := ../../../artifacts/$$($1)/usr
$$($1_DEPS_LIB_DIR)/$(THORVG_LIB): $$($1_THORVG_DEP_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE)
	$$(NINJA_BUILD)
endef

define NEW_ANDROID_BUILD
# Build runtime-ffi
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export ARTIFACTS_INCLUDE_DIR := ../$$($1_DEPS_INCLUDE_DIR)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export ARTIFACTS_LIB_DIR := ../$$($1_DEPS_LIB_DIR)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export ARTIFACTS_LIB64_DIR := ../$$($1_DEPS_LIB_DIR)64
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export CARGO_TARGET := $$($1)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export CARGO_TARGET_$1_LINKER := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/$$($1_ANDROID_ARCH)$(ANDROID_API_VERSION)-clang
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): PROJECT_DIR := $(RUNTIME_FFI)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): $$($1_DEPS_LIB_DIR)/$(THORVG_LIB)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS) $(CORE_SRC)
	$$(CARGO_BUILD)

# Build release
$$($1_DOTLOTTIE_PLAYER_LIB_DIR)/$(DOTLOTTIE_PLAYER_LIB): DOTLOTTIE_PLAYER_LIB_DIR := $$($1_DOTLOTTIE_PLAYER_LIB_DIR)
$$($1_DOTLOTTIE_PLAYER_LIB_DIR)/$(DOTLOTTIE_PLAYER_LIB): RUNTIME_FFI_TARGET_LIB := $$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB)
$$($1_DOTLOTTIE_PLAYER_LIB_DIR)/$(DOTLOTTIE_PLAYER_LIB): CRATE_VERSION := $(shell grep -m 1 version $(RUNTIME_FFI)/Cargo.toml | sed 's/.*"\([0-9.]\+\)"/\1/')
$$($1_DOTLOTTIE_PLAYER_LIB_DIR)/$(DOTLOTTIE_PLAYER_LIB): COMMIT_HASH := $(shell git rev-parse --short HEAD)
$$($1_DOTLOTTIE_PLAYER_LIB_DIR)/$(DOTLOTTIE_PLAYER_LIB): $$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB)
	$$(ANDROID_RELEASE)

.PHONY: $$($1)
$$($1): $$($1_DOTLOTTIE_PLAYER_LIB_DIR)/$(DOTLOTTIE_PLAYER_LIB)

ANDROID_BUILD_TARGETS += $$($1)
endef

define DEFINE_BUILD_TARGET
$(eval $(call NEW_BUILD_TARGET,$1,$(shell echo $(1) | tr '[:lower:]-' '[:upper:]_'),$2,$3,$4,$5))
endef

# Define all build targets
$(eval $(call DEFINE_BUILD_TARGET,aarch64-linux-android,aarch64-linux-android,arm64-v8a,arm,aarch64))
$(eval $(call DEFINE_BUILD_TARGET,armv7-linux-androideabi,armv7a-linux-androideabi,armeabi-v7a,arm,armv7))
$(eval $(call DEFINE_BUILD_TARGET,x86_64-linux-android,x86_64-linux-android,x86_64,x86_64,x86_64))

# Define all deps builds
$(eval $(call NEW_DEPS_BUILD,AARCH64_LINUX_ANDROID))
$(eval $(call NEW_DEPS_BUILD,ARMV7_LINUX_ANDROIDEABI))
$(eval $(call NEW_DEPS_BUILD,X86_64_LINUX_ANDROID))

# Define all android builds
$(eval $(call NEW_ANDROID_BUILD,AARCH64_LINUX_ANDROID))
$(eval $(call NEW_ANDROID_BUILD,ARMV7_LINUX_ANDROIDEABI))
$(eval $(call NEW_ANDROID_BUILD,X86_64_LINUX_ANDROID))

# Local architecture dependencies builds
define NEW_LOCAL_ARCH_CMAKE_BUILD
# Setup cmake for local arch build
$$($1_LOCAL_ARCH_BUILD_DIR)/$(CMAKE_MAKEFILE): DEP_SOURCE_DIR := $(DEPS_MODULES_DIR)/$2
$$($1_LOCAL_ARCH_BUILD_DIR)/$(CMAKE_MAKEFILE): DEP_BUILD_DIR := $$($1_LOCAL_ARCH_BUILD_DIR)
$$($1_LOCAL_ARCH_BUILD_DIR)/$(CMAKE_MAKEFILE): DEP_ARTIFACTS_DIR := $(LOCAL_ARCH_ARTIFACTS_DIR)
$$($1_LOCAL_ARCH_BUILD_DIR)/$(CMAKE_MAKEFILE):
	$$(SETUP_CMAKE)

# Build local arch
$(LOCAL_ARCH_LIB_DIR)/$3: CMAKE_BUILD_DIR := $$($1_LOCAL_ARCH_BUILD_DIR)
$(LOCAL_ARCH_LIB_DIR)/$3: $$($1_LOCAL_ARCH_BUILD_DIR)/$(CMAKE_MAKEFILE)
	$$(CMAKE_BUILD)
endef

# Define local deps builds
$(eval $(call NEW_LOCAL_ARCH_CMAKE_BUILD,LIBJPEG_TURBO,$(LIBJPEG_TURBO),$(LIBJPEG_TURBO_LIB)))
$(eval $(call NEW_LOCAL_ARCH_CMAKE_BUILD,LIBPNG,$(LIBPNG),$(LIBPNG_LIB)))
$(eval $(call NEW_LOCAL_ARCH_CMAKE_BUILD,ZLIB,$(ZLIB),$(ZLIB_LIB)))

# Setup meson for thorvg local arch build
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): export PKG_CONFIG_PATH := $(PWD)/$(LOCAL_ARCH_LIB_DIR)/pkgconfig
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): THORVG_DEP_SOURCE_DIR := $(DEPS_MODULES_DIR)/$(THORVG)
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): THORVG_DEP_BUILD_DIR := $(THORVG_LOCAL_ARCH_BUILD_DIR)
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): $(LOCAL_ARCH_LIB_DIR)/$(LIBJPEG_TURBO_LIB)
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): $(LOCAL_ARCH_LIB_DIR)/$(LIBPNG_LIB)
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE): $(LOCAL_ARCH_LIB_DIR)/$(ZLIB_LIB)
	$(SETUP_MESON)

# Build thorvg local arch
$(LOCAL_ARCH_LIB_DIR)/$(THORVG_LIB): DEP_BUILD_DIR := $(THORVG_LOCAL_ARCH_BUILD_DIR)
$(LOCAL_ARCH_LIB_DIR)/$(THORVG_LIB): ARTIFACTS_DIR := ../../../../artifacts/$(LOCAL_ARCH)/usr
$(LOCAL_ARCH_LIB_DIR)/$(THORVG_LIB): $(THORVG_LOCAL_ARCH_BUILD_DIR)/$(THORVG_NINJA_BUILD_FILE)
	$(NINJA_BUILD)

# Uniffi Bindings
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS): export ARTIFACTS_INCLUDE_DIR := ../$(LOCAL_ARCH_INCLUDE_DIR)
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS): export ARTIFACTS_LIB_DIR := ../$(LOCAL_ARCH_LIB_DIR)
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS): export ARTIFACTS_LIB64_DIR := ../$(LOCAL_ARCH_LIB_DIR)64
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS): $(LOCAL_ARCH_LIB_DIR)/$(THORVG_LIB)
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS): $(RUNTIME_FFI_SRC)
	cargo +nightly run \
		--manifest-path $(RUNTIME_FFI)/Cargo.toml \
		--features=uniffi/cli \
		--bin uniffi-bindgen \
		generate $(RUNTIME_FFI)/src/dlplayer.udl \
		--language kotlin \
		--out-dir $(RUNTIME_FFI)/uniffi-bindings

.PHONY: demo-player
demo-player:
	cargo build --manifest-path demo-player/Cargo.toml

.PHONY: android
android: $(ANDROID_BUILD_TARGETS)

.PHONY: all
all: android

.PHONY: deps
deps:
	@git submodule update --init --recursive

# Cleanup extraneous files from the zlib dependency build...
.PHONY: clean-zlib-build
clean-zlib-build:
	@git --git-dir=$(DEPS_MODULES_DIR)/$(ZLIB)/.git clean -fd &>/dev/null
	@git --git-dir=$(DEPS_MODULES_DIR)/$(ZLIB)/.git checkout . &>/dev/null

.PHONY: clean-deps
clean-deps: clean-zlib-build
	@rm -rf $(DEPS_BUILD_DIR) $(DEPS_ARTIFACTS_DIR)

.PHONY: clean
clean: clean-zlib-build
	@rm -rf $(RELEASE_DIR)
	@cargo clean --manifest-path $(CORE)/Cargo.toml
	@cargo clean --manifest-path $(RUNTIME_FFI)/Cargo.toml
	@rm -rf $(RUNTIME_FFI)/uniffi-bindings

.PHONY: clean-all
clean-all: clean clean-deps

.PHONY: help
help:
	@echo "Welcome to the $(GREEN)dotlottie-player$(NC) build system!"
	@echo
	@echo "The following targets are available for android:"
	@printf "  - $(YELLOW)%s$(NC)\n" $(ANDROID_BUILD_TARGETS)
	@echo
	@echo "$(GREEN)NOTE$(NC): Before building for the first time, use the $(YELLOW)deps$(NC) target to clone"
	@echo "      required submodules to the $(GREEN)deps$(NC) directory."
	@echo
	@echo "      After building a target, you should find your artifacts in the $(GREEN)release$(NC) directory."
	@echo
	@echo "Additionally:"
	@echo "  - Use the $(YELLOW)all$(NC) target to build everything"
	@echo "  - Use the $(YELLOW)clean$(NC) target to clear up all cargo & release files"
	@echo "  - Use the $(YELLOW)clean-deps$(NC) target to clear up all deps builds & artifacts"
	@echo "  - Use the $(YELLOW)clean-all$(NC) target to clear up everything"
	@echo
	@echo "The following tools must be installed before you can build anything:"
	@echo "  - pkg-config"
	@echo "  - cmake"
	@echo "  - meson"
	@echo "  - ninja"
	@echo "  - rust"
	@echo
	@echo "For each target you wish to build, install the rust target using rustup, e.g.:"
	@echo "  $$ rustup target add x86_64-linux-android"

