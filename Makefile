.DEFAULT_GOAL := help

# Some basic helpers
define n


endef

# Directory containing this Makefile
PROJECT_DIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))

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

# Glocal options
export CC := clang
export CXX := clang++

# Build variable(s)
BUILD := build

# Directories for external dependencies and their builds
DEPS_DIR := deps
DEPS_MODULES_DIR := $(DEPS_DIR)/modules
DEPS_BUILD_DIR := $(DEPS_DIR)/build
DEPS_ARTIFACTS_DIR := $(DEPS_DIR)/artifacts

# Android
ANDROID := android

ANDROID_BUILD_PLATFORM := $(BUILD_PLATFORM)-x86_64
ANDROID_NDK_HOME ?= /opt/homebrew/share/android-ndk
ANDROID_API_VERSION ?= 24

# Android Tool chain
AR := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/llvm-ar
AS := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/llvm-as
RANLIB := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/llvm-ranlib
LD := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/ld
STRIP := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/llvm-strip

# Apple
APPLE := apple
DARWIN := darwin
APPLE_BUILD := $(BUILD)/$(APPLE)

APPLE_IOS := ios
APPLE_IOS_PLATFORM := iPhoneOS
APPLE_IOS_SDK ?= iPhoneOS
APPLE_IOS_VERSION_MIN ?= 11.0

APPLE_IOS_SIMULATOR := ios-simulator
APPLE_IOS_SIMULATOR_PLATFORM := iPhoneSimulator
APPLE_IOS_SIMULATOR_SDK ?= iPhoneSimulator

APPLE_MACOSX := macosx
APPLE_MACOSX_PLATFORM := MacOSX
APPLE_MACOSX_SDK ?= MacOSX12

APPLE_IOS_FRAMEWORK_TYPE := $(APPLE_IOS)
APPLE_IOS_SIMULATOR_FRAMEWORK_TYPE := $(APPLE_IOS_SIMULATOR)
APPLE_MACOSX_FRAMEWORK_TYPE := $(APPLE_MACOSX)
APPLE_FRAMEWORK_TYPES := $(APPLE_IOS_FRAMEWORK_TYPE) $(APPLE_IOS_SIMULATOR_FRAMEWORK_TYPE) $(APPLE_MACOSX_FRAMEWORK_TYPE)

# Apple tools
LIPO := lipo
PLISTBUDDY_EXEC := /usr/libexec/PlistBuddy
INSTALL_NAME_TOOL := install_name_tool
XCODEBUILD := xcodebuild

# Wasm
WASM := wasm
WASM_BUILD := $(BUILD)/$(WASM)

EMSDK := emsdk
EMSDK_DIR := $(PROJECT_DIR)/$(DEPS_MODULES_DIR)/$(EMSDK)
EMSDK_VERSION := 3.1.57
EMSDK_ENV := emsdk_env.sh

UNIFFI_BINDGEN_CPP := uniffi-bindgen-cpp
UNIFFI_BINDGEN_CPP_VERSION := v0.6.0+v0.25.0

WASM_MODULE := DotLottiePlayer

# External dependencies
THORVG := thorvg
LIBJPEG_TURBO := libjpeg-turbo
LIBPNG := libpng
ZLIB := zlib
WEBP := libwebp

# External dependency artifacts
MESON_CROSS_FILE := cross.txt
MESON_BUILD_FILE := meson.build
NINJA_BUILD_FILE := build.ninja
THORVG_LIB := libthorvg.a

CMAKE_TOOLCHAIN_FILE := toolchain.cmake
CMAKE_MAKEFILE := Makefile
CMAKE_CACHE := CMakeCache.txt

LIBPNG_LIB := libpng.a
LIBJPEG_TURBO_LIB := libturbojpeg.a
ZLIB_LIB := libz.a
WEBP_LIB := libwebp.a

# Release artifacts will be placed in this directory
RELEASE := release

# Build artifact types
CORE := dotlottie-rs
RUNTIME_FFI := dotlottie-ffi
FMS := dotlottie-fms
DOTLOTTIE_PLAYER := dotlottie-player

# Build artifacts
RUNTIME_FFI_UNIFFI_BINDINGS := uniffi-bindings

RUNTIME_FFI_STATIC_LIB := libdotlottie_player.a
RUNTIME_FFI_LIB := libdotlottie_player.so
RUNTIME_FFI_DYLIB := libdotlottie_player.dylib

DOTLOTTIE_PLAYER_HEADER := dotlottie_player.h
DOTLOTTIE_PLAYER_SWIFT := dotlottie_player.swift
DOTLOTTIE_PLAYER_MODULE := DotLottiePlayer

DOTLOTTIE_PLAYER_FRAMEWORK := $(DOTLOTTIE_PLAYER_MODULE).framework
DOTLOTTIE_PLAYER_XCFRAMEWORK := $(DOTLOTTIE_PLAYER_MODULE).xcframework
FRAMEWORK_HEADERS := Headers
FRAMEWORK_MODULES := Modules
MODULE_MAP := module.modulemap
INFO_PLIST := Info.plist

KOTLIN := kotlin
SWIFT := swift
CPLUSPLUS := cpp

RUNTIME_FFI_ANDROID_ASSETS := assets
DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR := $(RELEASE)/$(ANDROID)/$(DOTLOTTIE_PLAYER)
DOTLOTTIE_PLAYER_ANDROID_SRC_DIR := $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/$(KOTLIN)
DOTLOTTIE_PLAYER_LIB := libuniffi_dotlottie_player.so
DOTLOTTIE_PLAYER_GRADLE_PROPERTIES := gradle.properties

# Dependency build directories for the current machine architecture
LOCAL_ARCH := local-arch

LOCAL_ARCH_BUILD_DIR := $(DEPS_BUILD_DIR)/$(LOCAL_ARCH)
LOCAL_ARCH_ARTIFACTS_DIR := $(DEPS_ARTIFACTS_DIR)/$(LOCAL_ARCH)/usr
LOCAL_ARCH_INCLUDE_DIR := $(LOCAL_ARCH_ARTIFACTS_DIR)/include
LOCAL_ARCH_LIB_DIR := $(LOCAL_ARCH_ARTIFACTS_DIR)/lib
LOCAL_ARCH_LIB64_DIR := $(LOCAL_ARCH_ARTIFACTS_DIR)/lib64

THORVG_LOCAL_ARCH_BUILD_DIR := $(LOCAL_ARCH_BUILD_DIR)/$(THORVG)/build
LIBJPEG_TURBO_LOCAL_ARCH_BUILD_DIR := $(LOCAL_ARCH_BUILD_DIR)/$(LIBJPEG_TURBO)/build
LIBPNG_LOCAL_ARCH_BUILD_DIR := $(LOCAL_ARCH_BUILD_DIR)/$(LIBPNG)/build
ZLIB_LOCAL_ARCH_BUILD_DIR := $(LOCAL_ARCH_BUILD_DIR)/$(ZLIB)/build
WEBP_LOCAL_ARCH_BUILD_DIR := $(LOCAL_ARCH_BUILD_DIR)/$(WEBP)/build

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
system = '$(ANDROID)'
cpu_family = '$(CPU_FAMILY)'
cpu = '$(CPU)'
endian = 'little'
endef

define APPLE_CROSS_FILE
[binaries]
cpp = ['clang++', '-arch', '$(ARCH)', '-isysroot', '/Applications/Xcode_13.3.1.app/Contents/Developer/Platforms/$(PLATFORM).platform/Developer/SDKs/$(SDK).sdk']
ld = 'ld'
ar = 'ar'
strip = 'strip'
pkg-config = 'pkg-config'

[properties]
root = '/Applications/Xcode_13.3.1.app/Contents/Developer/Platforms/$(SDK).platform/Developer'
has_function_printf = true

$(if $(filter $(PLATFORM),$(APPLE_IOS_PLATFORM) $(APPLE_IOS_SIMULATOR_PLATFORM)),\
[built-in options]\n\
cpp_args = ['-miphoneos-version-min=$(APPLE_IOS_VERSION_MIN)']\n\
cpp_link_args = ['-miphoneos-version-min=$(APPLE_IOS_VERSION_MIN)']\n\
,)

[host_machine]
system = 'darwin'
subsystem = '$(SUBSYSTEM)'
kernel = 'xnu'
cpu_family = '$(CPU_FAMILY)'
cpu = '$(CPU)'
endian = 'little'
endef

define WASM_CROSS_FILE
[binaries]
cpp = ['$(EMSDK_DIR)/upstream/emscripten/em++.py', '-std=c++20']
ar = '$(EMSDK_DIR)/upstream/emscripten/emar.py'
strip = '-strip'

[properties]
root = '$(EMSDK_DIR)/upstream/emscripten/system'
shared_lib_suffix = 'js'
static_lib_suffix = 'js'
shared_module_suffix = 'js'
exe_suffix = 'js'

[built-in options]
cpp_args = ['-Wshift-negative-value', '-flto', '-Oz', '-ffunction-sections', '-fdata-sections']
cpp_link_args = [
	'-Wl,-u,htons',
	'-Wl,-u,ntohs',
	'-Wl,-u,htonl',
	'-Wshift-negative-value',
	'-flto', '-Os', '--bind', '-sWASM=1',
	'-sALLOW_MEMORY_GROWTH=1',
	'-sFORCE_FILESYSTEM=0',
	'-sMODULARIZE=1',
	'-sEXPORT_NAME=create$(WASM_MODULE)Module',
	'-sEXPORT_ES6=1',
	'-sUSE_ES6_IMPORT_META=0',
	'-sENVIRONMENT=web',
	'-sFILESYSTEM=0',
	'-sDYNAMIC_EXECUTION=0',
	'--no-entry',
	'--strip-all',
	'--emit-tsd=${WASM_MODULE}.d.ts',
	'--minify=0']

[host_machine]
system = '$(SYSTEM)'
cpu_family = '$(CPU_FAMILY)'
cpu = '$(CPU)'
endian = 'little'
endef

# Helper functions
define ANDROID_CMAKE_TOOLCHAIN_FILE
set(CMAKE_SYSTEM_NAME Android)
set(CMAKE_SYSTEM_VERSION $(ANDROID_API_VERSION))
set(CMAKE_ANDROID_ARCH_ABI $(ANDROID_ABI))
set(CMAKE_ANDROID_NDK $(ANDROID_NDK_HOME))
endef

define APPLE_MODULE_MAP_FILE
framework module $(MODULE_NAME) {
  umbrella header "$(UMBRELLA_HEADER)"
  export *
  module * { export * }
}
endef

define WASM_MESON_BUILD_FILE
project('$(WASM_MODULE)', 'cpp')

cc = meson.get_compiler('cpp')
if cc.get_id() == 'emscripten'
    executable('$(WASM_MODULE)',
        [$(shell find $(FFI_BINDINGS_DIR) -name "*.cpp" -exec printf "'%s'," {} \; 2>/dev/null)],
        include_directories: '$(FFI_BINDINGS_DIR)',
        link_args: ['-L$(DEPS_LIB_DIR)', '-L$(FFI_BUILD_DIR)', '-lthorvg', '-ldotlottie_player'],
    )
else
    message('The compiler is not Emscripten.')
endif
endef

define CREATE_OUTPUT_FILE
	mkdir -p $$(dirname $@)
	echo "$$OUTPUT_FILE" > $@
endef

define SETUP_MESON
	meson setup \
		--prefix=/ \
		--backend=ninja \
		-Dloaders="lottie, png, jpg, webp" \
		-Ddefault_library=static \
		-Dbindings=capi \
		-Dlog=$(LOG) \
		-Dstatic=$(STATIC) \
		-Dextra=$(EXTRA) \
		$(CROSS_FILE) "$(THORVG_DEP_SOURCE_DIR)" "$(THORVG_DEP_BUILD_DIR)"
endef

define SETUP_WASM_MESON
  meson setup \
		--prefix=/ \
		--backend=ninja \
		--cross-file "$(CROSS_FILE)" "$(WASM_SRC_DIR)" "$(WASM_BUILD_DIR)"
endef

define NINJA_BUILD
	DESTDIR=$(ARTIFACTS_DIR) ninja -C $(DEP_BUILD_DIR) install
endef

define SETUP_CMAKE
	cmake -DCMAKE_INSTALL_PREFIX=$(DEP_ARTIFACTS_DIR) \
		-DCMAKE_POSITION_INDEPENDENT_CODE=ON \
		-DBUILD_SHARED_LIBS=OFF $(CMAKE_BUILD_SETTINGS) $(PLATFORM) $(TOOLCHAIN_FILE) \
		-B $(DEP_BUILD_DIR) \
		$(DEP_SOURCE_DIR)
endef

define CMAKE_MAKE_BUILD
  $(MAKE) -C $(CMAKE_BUILD_DIR) install
endef

define CMAKE_BUILD
  cmake --build $(CMAKE_BUILD_DIR) --config Release --target install -- $(CMAKE_BUILD_OPTIONS)
endef

define CLEAN_LIBGJPEG
	echo "Removing libjpeg from rm /usr/local/lib/libjpeg*"
	rm -f /usr/local/lib/libjpeg*
endef

define CARGO_BUILD
	source $(EMSDK_DIR)/$(EMSDK)_env.sh && \
		cargo build \
		--manifest-path $(PROJECT_DIR)/Cargo.toml \
		--target $(CARGO_TARGET) \
		--release
endef

define UNIFFI_BINDINGS_BUILD
	rm -rf $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(BINDINGS_LANGUAGE)
	cargo run \
		--manifest-path $(RUNTIME_FFI)/Cargo.toml \
		--features=uniffi/cli \
		--bin uniffi-bindgen \
		generate $(RUNTIME_FFI)/src/dotlottie_player.udl \
		--language $(BINDINGS_LANGUAGE) \
		--out-dir $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(BINDINGS_LANGUAGE)
endef

define UNIFFI_BINDINGS_CPP_BUILD
	rm -rf $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(CPLUSPLUS)
	$(UNIFFI_BINDGEN_CPP) \
		--config $(RUNTIME_FFI)/uniffi.toml \
		--out-dir $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(CPLUSPLUS) \
		$(RUNTIME_FFI)/src/dotlottie_player_cpp.udl
	sed -i .bak 's/uint8_t/char/g' $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(CPLUSPLUS)/*
	cp $(RUNTIME_FFI)/emscripten_bindings.cpp $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(CPLUSPLUS)/.
endef

define ANDROID_RELEASE
  mkdir -p $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR) $(DOTLOTTIE_PLAYER_ANDROID_SRC_DIR) $(DOTLOTTIE_PLAYER_LIB_DIR)
  cp -r $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(KOTLIN)/* $(DOTLOTTIE_PLAYER_ANDROID_SRC_DIR)
  cp $(RUNTIME_FFI_TARGET_LIB) $(DOTLOTTIE_PLAYER_LIB_DIR)/$(DOTLOTTIE_PLAYER_LIB)
  cp $(RUNTIME_FFI)/$(RUNTIME_FFI_ANDROID_ASSETS)/$(ANDROID)/* $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)
  echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/$(DOTLOTTIE_PLAYER_GRADLE_PROPERTIES)
	cd $(RELEASE)/$(ANDROID) && \
		rm -f $(DOTLOTTIE_PLAYER).$(ANDROID).tar.gz && \
		tar zcf $(DOTLOTTIE_PLAYER).$(ANDROID).tar.gz *
endef

define LIPO_CREATE
	rm -f $@
	mkdir -p $$(dirname $@)
	$(LIPO) \
		-create $(LIBS) \
		-o $@
endef

define CREATE_FRAMEWORK
	rm -rf $(BASE_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) $(RELEASE)/$(APPLE)/$(DOTLOTTIE_PLAYER_XCFRAMEWORK)
	mkdir -p $(BASE_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/{$(FRAMEWORK_HEADERS),$(FRAMEWORK_MODULES)}
	cp $(BASE_DIR)/$(RUNTIME_FFI_DYLIB) $(BASE_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	cp $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(SWIFT)/$(DOTLOTTIE_PLAYER_MODULE).h $(BASE_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_HEADERS)/$(DOTLOTTIE_PLAYER_HEADER)
	cp $(RUNTIME_FFI)/$(APPLE_BUILD)/$(MODULE_MAP) $(BASE_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_MODULES)

	$(PLISTBUDDY_EXEC) -c "Add :CFBundleIdentifier string com.dotlottie.$(DOTLOTTIE_PLAYER_MODULE)" \
                     -c "Add :CFBundleName string $(DOTLOTTIE_PLAYER_MODULE)" \
                     -c "Add :CFBundleDisplayName string $(DOTLOTTIE_PLAYER_MODULE)" \
                     -c "Add :CFBundleVersion string 1.0.0" \
                     -c "Add :CFBundleShortVersionString string 1.0.0" \
                     -c "Add :CFBundlePackageType string FMWK" \
                     -c "Add :CFBundleExecutable string $(DOTLOTTIE_PLAYER_MODULE)" \
                     -c "Add :MinimumOSVersion string 15.4" \
                     -c "Add :CFBundleSupportedPlatforms array" \
										 $(foreach platform,$(PLIST_DISABLE),-c "Add :CFBundleSupportedPlatforms:0 string $(platform)" ) \
										 $(foreach platform,$(PLIST_ENABLE),-c "Add :CFBundleSupportedPlatforms:1 string $(platform)" ) \
                     $(BASE_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(INFO_PLIST)

	$(INSTALL_NAME_TOOL) -id @rpath/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE) $(BASE_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
endef

define APPLE_RELEASE
	rm -rf $(RELEASE)/$(APPLE)
	mkdir -p $(RELEASE)/$(APPLE)
  $(XCODEBUILD) -create-xcframework \
                $$(find $(RUNTIME_FFI)/$(APPLE_BUILD) -type d -depth 2 | sed 's/^/-framework /' | tr '\n' ' ') \
                -output $(RELEASE)/$(APPLE)/$(DOTLOTTIE_PLAYER_XCFRAMEWORK)
	cp $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(SWIFT)/$(DOTLOTTIE_PLAYER_SWIFT) $(RELEASE)/$(APPLE)/.
	cd $(RELEASE)/$(APPLE) && \
		rm -f $(DOTLOTTIE_PLAYER).$(DARWIN).tar.gz && \
		tar zcf $(DOTLOTTIE_PLAYER).$(DARWIN).tar.gz *
endef

define WASM_RELEASE
	rm -rf $(RELEASE)/$(WASM)
	mkdir -p $(RELEASE)/$(WASM)
	cp $(RUNTIME_FFI)/$(WASM_BUILD)/$(BUILD)/$(WASM_MODULE).wasm \
		$(RELEASE)/$(WASM)
	cp $(RUNTIME_FFI)/$(WASM_BUILD)/$(BUILD)/$(WASM_MODULE).d.ts \
		$(RELEASE)/$(WASM)
	cp $(RUNTIME_FFI)/$(WASM_BUILD)/$(BUILD)/$(WASM_MODULE).js \
		$(RELEASE)/$(WASM)/$(WASM_MODULE).mjs
	cd $(RELEASE)/$(WASM) && \
		rm -f $(DOTLOTTIE_PLAYER).$(WASM).tar.gz && \
		tar zcf $(DOTLOTTIE_PLAYER).$(WASM).tar.gz *
endef

# $1: rust target triple, e.g. aarch64-linux-android
# $2: rust target triple in screaming snake case, e.g. AARCH64_LINUX_ANDROID
# $3: build specific, i.e. android/apple, target
# $4: build specific, i.e. android/apple, abi
# $5: CPU Family, e.g. arm
# $6: CPU, e.g. aarch64
define NEW_BUILD_TARGET
# Setup architecture variables
$2 := $1
$2_ARCH := $3
$2_ABI := $4
$2_CPU_FAMILY := $5
$2_CPU := $6

# Setup dependency build variables
$2_DEPS_BUILD_DIR := $(DEPS_BUILD_DIR)/$1

$2_THORVG_DEP_BUILD_DIR := $$($2_DEPS_BUILD_DIR)/$(THORVG)
$2_LIBJPEG_TURBO_DEP_BUILD_DIR := $$($2_DEPS_BUILD_DIR)/$(LIBJPEG_TURBO)
$2_LIBPNG_DEP_BUILD_DIR := $$($2_DEPS_BUILD_DIR)/$(LIBPNG)
$2_ZLIB_DEP_BUILD_DIR := $$($2_DEPS_BUILD_DIR)/$(ZLIB)
$2_WEBP_DEP_BUILD_DIR := $$($2_DEPS_BUILD_DIR)/$(WEBP)

$2_DEPS_ARTIFACTS_DIR := $(DEPS_ARTIFACTS_DIR)/$1/usr
$2_DEPS_INCLUDE_DIR := $$($2_DEPS_ARTIFACTS_DIR)/include
$2_DEPS_LIB_DIR := $$($2_DEPS_ARTIFACTS_DIR)/lib
$2_DEPS_LIB64_DIR := $$($2_DEPS_ARTIFACTS_DIR)/lib64
endef

define NEW_APPLE_TARGET
# Setup Apple-specific variables
$1_SUBSYSTEM := $2
$1_PLATFORM := $3
$1_SDK := $4

# Setup the framework type for this target
$(if $(filter $3,$(APPLE_IOS_PLATFORM)),\
$1_FRAMEWORK_TYPE := $(APPLE_IOS_FRAMEWORK_TYPE)
APPLE_IOS_FRAMEWORK_TARGETS += $$($1))

$(if $(filter $3,$(APPLE_IOS_SIMULATOR_PLATFORM)),\
$1_FRAMEWORK_TYPE := $(APPLE_IOS_SIMULATOR_FRAMEWORK_TYPE)
APPLE_IOS_SIMULATOR_FRAMEWORK_TARGETS += $$($1))

$(if $(filter $3,$(APPLE_MACOSX_PLATFORM)),\
$1_FRAMEWORK_TYPE := $(APPLE_MACOSX_FRAMEWORK_TYPE)
APPLE_MACOSX_FRAMEWORK_TARGETS += $$($1))
endef

define NEW_ANDROID_CMAKE_BUILD
# Create toolchain file
$4/../$(CMAKE_TOOLCHAIN_FILE): DEP_BUILD_DIR := $4
$4/../$(CMAKE_TOOLCHAIN_FILE): ANDROID_ABI := $$($1_ABI)
$4/../$(CMAKE_TOOLCHAIN_FILE): export OUTPUT_FILE := $$(ANDROID_CMAKE_TOOLCHAIN_FILE)
$4/../$(CMAKE_TOOLCHAIN_FILE):
	$$(CREATE_OUTPUT_FILE)

# Setup cmake
$4/$(CMAKE_MAKEFILE): export LDFLAGS := $$($2_LDFLAGS)
$4/$(CMAKE_MAKEFILE): DEP_SOURCE_DIR := $(DEPS_MODULES_DIR)/$3
$4/$(CMAKE_MAKEFILE): DEP_BUILD_DIR := $4
$4/$(CMAKE_MAKEFILE): DEP_ARTIFACTS_DIR := $$($1_DEPS_ARTIFACTS_DIR)
$4/$(CMAKE_MAKEFILE): CMAKE_BUILD_SETTINGS := -DANDROID_NDK=$(ANDROID_NDK_HOME) -DANDROID_ABI=$$($1_ABI)
$4/$(CMAKE_MAKEFILE): TOOLCHAIN_FILE := -DCMAKE_TOOLCHAIN_FILE=../$(CMAKE_TOOLCHAIN_FILE)
$4/$(CMAKE_MAKEFILE): $4/../$(CMAKE_TOOLCHAIN_FILE)
	$$(SETUP_CMAKE)

# Build
$$($1_DEPS_LIB_DIR)/$5: CMAKE_BUILD_DIR := $4
$$($1_DEPS_LIB_DIR)/$5: $4/$(CMAKE_MAKEFILE)
	$$(CMAKE_MAKE_BUILD)
endef

define NEW_APPLE_CMAKE_BUILD
# Setup cmake
$4/$(CMAKE_CACHE): DEP_SOURCE_DIR := $(DEPS_MODULES_DIR)/$3
$4/$(CMAKE_CACHE): DEP_BUILD_DIR := $4
$4/$(CMAKE_CACHE): DEP_ARTIFACTS_DIR := $$($1_DEPS_ARTIFACTS_DIR)
$4/$(CMAKE_CACHE): CMAKE_BUILD_SETTINGS := -GXcode -DCMAKE_MACOSX_BUNDLE=NO
$4/$(CMAKE_CACHE): PLATFORM := -DPLATFORM=$$($1_ARCH)
$4/$(CMAKE_CACHE): TOOLCHAIN_FILE := -DCMAKE_TOOLCHAIN_FILE=$(PWD)/$(DEPS_MODULES_DIR)/ios-cmake/ios.toolchain.cmake
$4/$(CMAKE_CACHE):
	$$(SETUP_CMAKE)

# Build
$(call CLEAN_LIBGJPEG)
$$($1_DEPS_LIB_DIR)/$5: CMAKE_BUILD_DIR := $4
$$($1_DEPS_LIB_DIR)/$5: CMAKE_BUILD_OPTIONS := $(if $(filter $($1_SUBSYSTEM),$(APPLE_IOS)),CODE_SIGNING_ALLOWED=NO,)
$$($1_DEPS_LIB_DIR)/$5: $4/$(CMAKE_CACHE)
	$$(CMAKE_BUILD)
endef

define NEW_ANDROID_CROSS_FILE
# Create cross file for thorvg
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE): SYSROOT := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/sysroot/usr/lib/$$($1_ARCH)/$(ANDROID_API_VERSION)
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE): CPP := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/$$($1_ARCH)$(ANDROID_API_VERSION)-clang++
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE): CPU_FAMILY := $$($1_CPU_FAMILY)
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE): CPU := $$($1_CPU)
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE): export OUTPUT_FILE := $$(ANDROID_CROSS_FILE)
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE):
	$$(CREATE_OUTPUT_FILE)
endef

define NEW_APPLE_CROSS_FILE
# Create cross file for thorvg
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE): ARCH := $$($1_ABI)
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE): PLATFORM := $$($1_PLATFORM)
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE): SDK := $$($1_SDK)
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE): SUBSYSTEM := $$($1_SUBSYSTEM)
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE): CPU_FAMILY := $$($1_CPU_FAMILY)
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE): CPU := $$($1_CPU)
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE): export OUTPUT_FILE := $$(APPLE_CROSS_FILE)
$$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE):
	$$(CREATE_OUTPUT_FILE)
endef

define NEW_WASM_CROSS_FILE
# Create cross file for thorvg
$2/$(MESON_CROSS_FILE): SYSTEM := $3
$2/$(MESON_CROSS_FILE): CPU_FAMILY := $$($1_CPU_FAMILY)
$2/$(MESON_CROSS_FILE): CPU := $$($1_CPU)
$2/$(MESON_CROSS_FILE): export OUTPUT_FILE := $$(WASM_CROSS_FILE)
$2/$(MESON_CROSS_FILE):
	$$(CREATE_OUTPUT_FILE)
endef

define NEW_THORVG_BUILD
# Setup meson for thorvg
$$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE): export PKG_CONFIG_PATH := $(PWD)/$$($1_DEPS_LIB_DIR)/pkgconfig:$(PWD)/$$($1_DEPS_LIB64_DIR)/pkgconfig
$$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE): THORVG_DEP_SOURCE_DIR := $(DEPS_MODULES_DIR)/$(THORVG)
$$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE): THORVG_DEP_BUILD_DIR := $$($1_THORVG_DEP_BUILD_DIR)
$$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE): CROSS_FILE := --cross-file $$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE)
$$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE): LOG := $2
$$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE): STATIC := $3
$$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE): EXTRA := $4
$$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE): $$($1_THORVG_DEP_BUILD_DIR)/../$(MESON_CROSS_FILE)
$(if $(filter $3,false),
$$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE): $$($1_DEPS_LIB_DIR)/$(LIBJPEG_TURBO_LIB)
$$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE): $$($1_DEPS_LIB_DIR)/$(LIBPNG_LIB)
$$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE): $$($1_DEPS_LIB_DIR)/$(WEBP_LIB)
$$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE): $$($1_DEPS_LIB_DIR)/$(ZLIB_LIB),)
	$$(SETUP_MESON)

# Build thorvg
$$($1_DEPS_LIB_DIR)/$(THORVG_LIB): DEP_BUILD_DIR := $$($1_THORVG_DEP_BUILD_DIR)
$$($1_DEPS_LIB_DIR)/$(THORVG_LIB): ARTIFACTS_DIR := ../../../artifacts/$$($1)/usr
$$($1_DEPS_LIB_DIR)/$(THORVG_LIB): $$($1_THORVG_DEP_BUILD_DIR)/$(NINJA_BUILD_FILE)
	$$(NINJA_BUILD)
endef

define NEW_ANDROID_DEPS_BUILD
$(eval $(call NEW_ANDROID_CMAKE_BUILD,$1,LIBJPEG_TURBO,$(LIBJPEG_TURBO),$$($1_LIBJPEG_TURBO_DEP_BUILD_DIR),$(LIBJPEG_TURBO_LIB)))
$(eval $(call NEW_ANDROID_CMAKE_BUILD,$1,LIBPNG_LIB,$(LIBPNG),$$($1_LIBPNG_DEP_BUILD_DIR),$(LIBPNG_LIB)))
$(eval $(call NEW_ANDROID_CMAKE_BUILD,$1,ZLIB,$(ZLIB),$$($1_ZLIB_DEP_BUILD_DIR),$(ZLIB_LIB)))
$(eval $(call NEW_ANDROID_CMAKE_BUILD,$1,WEBP,$(WEBP),$$($1_WEBP_DEP_BUILD_DIR),$(WEBP_LIB)))
$(eval $(call NEW_ANDROID_CROSS_FILE,$1))
$(eval $(call NEW_THORVG_BUILD,$1,false,false,"lottie_expressions"))
endef

define NEW_APPLE_DEPS_BUILD
$(eval $(call NEW_APPLE_CMAKE_BUILD,$1,LIBJPEG_TURBO,$(LIBJPEG_TURBO),$$($1_LIBJPEG_TURBO_DEP_BUILD_DIR),$(LIBJPEG_TURBO_LIB)))
$(eval $(call NEW_APPLE_CMAKE_BUILD,$1,LIBPNG_LIB,$(LIBPNG),$$($1_LIBPNG_DEP_BUILD_DIR),$(LIBPNG_LIB)))
$(eval $(call NEW_APPLE_CMAKE_BUILD,$1,ZLIB,$(ZLIB),$$($1_ZLIB_DEP_BUILD_DIR),$(ZLIB_LIB)))
$(eval $(call NEW_APPLE_CMAKE_BUILD,$1,WEBP,$(WEBP),$$($1_WEBP_DEP_BUILD_DIR),$(WEBP_LIB)))
$(eval $(call NEW_APPLE_CROSS_FILE,$1))
$(eval $(call NEW_THORVG_BUILD,$1,false,false,"lottie_expressions"))
endef

define NEW_WASM_DEPS_BUILD
$(eval $(call NEW_WASM_CROSS_FILE,$1,$$($1_THORVG_DEP_BUILD_DIR)/..,windows))
$(eval $(call NEW_THORVG_BUILD,$1,false,true,"lottie_expressions"))
endef

define NEW_ANDROID_BUILD
# Setup final artifact variables
$1_RUNTIME_FFI_DEPS_BUILD_DIR := $(RUNTIME_FFI)/target/$$($1)/release
$1_DOTLOTTIE_PLAYER_LIB_DIR := $(DOTLOTTIE_PLAYER_ANDROID_RELEASE_DIR)/src/main/jniLibs/$$($1_ABI)

# Build dotlottie-ffi
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export ARTIFACTS_INCLUDE_DIR := ../$$($1_DEPS_INCLUDE_DIR)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export ARTIFACTS_LIB_DIR := ../$$($1_DEPS_LIB_DIR)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export ARTIFACTS_LIB64_DIR := ../$$($1_DEPS_LIB_DIR)64
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export CARGO_TARGET := $$($1)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export CARGO_TARGET_$1_LINKER := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(ANDROID_BUILD_PLATFORM)/bin/$$($1_ARCH)$(ANDROID_API_VERSION)-clang
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): PROJECT_DIR := $(RUNTIME_FFI)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): $$($1_DEPS_LIB_DIR)/$(THORVG_LIB)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(KOTLIN) $(CORE_SRC)
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

define NEW_APPLE_BUILD
# Setup final artifact variables
$1_RUNTIME_FFI_DEPS_BUILD_DIR := $(RUNTIME_FFI)/target/$$($1)/release

# Build dotlottie-ffi
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export ARTIFACTS_INCLUDE_DIR := ../$$($1_DEPS_INCLUDE_DIR)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export ARTIFACTS_LIB_DIR := ../$$($1_DEPS_LIB_DIR)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export ARTIFACTS_LIB64_DIR := ../$$($1_DEPS_LIB_DIR)64
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): export CARGO_TARGET := $$($1)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): PROJECT_DIR := $(RUNTIME_FFI)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): $$($1_DEPS_LIB_DIR)/$(THORVG_LIB)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(SWIFT)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB): $(CORE_SRC)
	$$(CARGO_BUILD)

# Ensure that the release artifact depends on this build
$(RUNTIME_FFI)/$(APPLE_BUILD)/$$($1_FRAMEWORK_TYPE)/$(RUNTIME_FFI_DYLIB): $$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_LIB)

.PHONY: $$($1)
$$($1): $(RUNTIME_FFI)/$(APPLE_BUILD)/$$($1_FRAMEWORK_TYPE)/$(DOTLOTTIE_PLAYER_FRAMEWORK) $(RELEASE)/$(APPLE)/$(DOTLOTTIE_PLAYER_XCFRAMEWORK)

APPLE_BUILD_TARGETS += $$($1)
endef

# $1: framework type
# $2: framework targets
# $3: plist enable
# $4: plist disable
define NEW_APPLE_FRAMEWORK
# Build lipo library
$(RUNTIME_FFI)/$(APPLE_BUILD)/$1/$(RUNTIME_FFI_DYLIB): ALL_LIBS := $$(foreach target,$2,$(RUNTIME_FFI)/target/$$(target)/release/$(RUNTIME_FFI_DYLIB))
$(RUNTIME_FFI)/$(APPLE_BUILD)/$1/$(RUNTIME_FFI_DYLIB): LIBS = $$(foreach lib,$$(ALL_LIBS),$$(wildcard $$(lib)))
$(RUNTIME_FFI)/$(APPLE_BUILD)/$1/$(RUNTIME_FFI_DYLIB):
	$$(LIPO_CREATE)

# Build framework & xcframework
$(RUNTIME_FFI)/$(APPLE_BUILD)/$1/$(DOTLOTTIE_PLAYER_FRAMEWORK): BASE_DIR := $(RUNTIME_FFI)/$(APPLE_BUILD)/$1
$(RUNTIME_FFI)/$(APPLE_BUILD)/$1/$(DOTLOTTIE_PLAYER_FRAMEWORK): PLIST_ENABLE := $3
$(RUNTIME_FFI)/$(APPLE_BUILD)/$1/$(DOTLOTTIE_PLAYER_FRAMEWORK): PLIST_DISABLE := $4
$(RUNTIME_FFI)/$(APPLE_BUILD)/$1/$(DOTLOTTIE_PLAYER_FRAMEWORK): $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(SWIFT) $(RUNTIME_FFI)/$(APPLE_BUILD)/$(MODULE_MAP)
$(RUNTIME_FFI)/$(APPLE_BUILD)/$1/$(DOTLOTTIE_PLAYER_FRAMEWORK): $(RUNTIME_FFI)/$(APPLE_BUILD)/$1/$(RUNTIME_FFI_DYLIB)
	$$(CREATE_FRAMEWORK)
	$$(APPLE_RELEASE)
endef

define NEW_WASM_BUILD
# Setup final artifact variables
$1_RUNTIME_FFI_DEPS_BUILD_DIR := $(RUNTIME_FFI)/target/$$($1)/release

# Build dotlottie-ffi
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_STATIC_LIB): export ARTIFACTS_INCLUDE_DIR := ../$$($1_DEPS_INCLUDE_DIR)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_STATIC_LIB): export ARTIFACTS_LIB_DIR := ../$$($1_DEPS_LIB_DIR)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_STATIC_LIB): export ARTIFACTS_LIB64_DIR := ../$$($1_DEPS_LIB_DIR)64
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_STATIC_LIB): export CARGO_TARGET := $$($1)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_STATIC_LIB): PROJECT_DIR := $(RUNTIME_FFI)
$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_STATIC_LIB): $$($1_DEPS_LIB_DIR)/$(THORVG_LIB)
	$$(CARGO_BUILD)

# Setup WASM build cross file
$(call NEW_WASM_CROSS_FILE,$1,$(RUNTIME_FFI)/$(WASM_BUILD),emscripten)

# Setup WASM meson build
$(RUNTIME_FFI)/$(WASM_BUILD)/$(MESON_BUILD_FILE): DEPS_INCLUDE_DIR := $(PROJECT_DIR)/$$($1_DEPS_INCLUDE_DIR)
$(RUNTIME_FFI)/$(WASM_BUILD)/$(MESON_BUILD_FILE): DEPS_LIB_DIR := $(PROJECT_DIR)/$$($1_DEPS_LIB_DIR)
$(RUNTIME_FFI)/$(WASM_BUILD)/$(MESON_BUILD_FILE): FFI_BUILD_DIR := $(PROJECT_DIR)/$$($1_RUNTIME_FFI_DEPS_BUILD_DIR)
$(RUNTIME_FFI)/$(WASM_BUILD)/$(MESON_BUILD_FILE): FFI_BINDINGS_DIR := $(PROJECT_DIR)/$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(CPLUSPLUS)
$(RUNTIME_FFI)/$(WASM_BUILD)/$(MESON_BUILD_FILE): export OUTPUT_FILE = $$(WASM_MESON_BUILD_FILE)
$(RUNTIME_FFI)/$(WASM_BUILD)/$(MESON_BUILD_FILE): $$($1_RUNTIME_FFI_DEPS_BUILD_DIR)/$(RUNTIME_FFI_STATIC_LIB)
$(RUNTIME_FFI)/$(WASM_BUILD)/$(MESON_BUILD_FILE): $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(CPLUSPLUS)
$(RUNTIME_FFI)/$(WASM_BUILD)/$(MESON_BUILD_FILE): $(RUNTIME_FFI)/$(WASM_BUILD)/$(MESON_CROSS_FILE)
	$$(CREATE_OUTPUT_FILE)

# Setup meson for WASM
$(RUNTIME_FFI)/$(WASM_BUILD)/$(NINJA_BUILD_FILE): WASM_SRC_DIR := $(RUNTIME_FFI)/$(WASM_BUILD)
$(RUNTIME_FFI)/$(WASM_BUILD)/$(NINJA_BUILD_FILE): WASM_BUILD_DIR := $(RUNTIME_FFI)/$(WASM_BUILD)/$(BUILD)
$(RUNTIME_FFI)/$(WASM_BUILD)/$(NINJA_BUILD_FILE): CROSS_FILE := $(RUNTIME_FFI)/$(WASM_BUILD)/$(MESON_CROSS_FILE)
$(RUNTIME_FFI)/$(WASM_BUILD)/$(NINJA_BUILD_FILE): $(RUNTIME_FFI)/$(WASM_BUILD)/$(MESON_BUILD_FILE)
	$$(SETUP_WASM_MESON)

# Build release
$(RELEASE)/$(WASM)/$(WASM_MODULE).wasm $(RELEASE)/$(WASM)/$(WASM_MODULE).js: DEP_BUILD_DIR := $(RUNTIME_FFI)/$(WASM_BUILD)/$(BUILD)
$(RELEASE)/$(WASM)/$(WASM_MODULE).wasm $(RELEASE)/$(WASM)/$(WASM_MODULE).js: ARTIFACTS_DIR := $(RELEASE)/$(WASM)
$(RELEASE)/$(WASM)/$(WASM_MODULE).wasm $(RELEASE)/$(WASM)/$(WASM_MODULE).js: $(RUNTIME_FFI)/$(WASM_BUILD)/$(NINJA_BUILD_FILE)
	$$(NINJA_BUILD)
	$$(WASM_RELEASE)

.PHONY: $$($1)
$$($1): $(RELEASE)/$(WASM)/$(WASM_MODULE).wasm $(RELEASE)/$(WASM)/$(WASM_MODULE).js

WASM_BUILD_TARGETS += $$($1)
endef

define TARGET_PREFIX
$(shell echo $(1) | tr '[:lower:]-' '[:upper:]_')
endef

define DEFINE_TARGET
$(eval $(call NEW_BUILD_TARGET,$1,$(call TARGET_PREFIX,$1),$2,$3,$4,$5))
endef

define DEFINE_APPLE_TARGET
$(eval $(call DEFINE_TARGET,$1,$2,$3,$4,$5))
$(eval $(call NEW_APPLE_TARGET,$(call TARGET_PREFIX,$1),$6,$7,$8))
endef

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
$(eval $(call NEW_LOCAL_ARCH_CMAKE_BUILD,WEBP,$(WEBP),$(WEBP_LIB)))

# Setup meson for thorvg local arch build
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE): export PKG_CONFIG_PATH := $(PWD)/$(LOCAL_ARCH_LIB_DIR)/pkgconfig:$(PWD)/$(LOCAL_ARCH_LIB64_DIR)
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE): THORVG_DEP_SOURCE_DIR := $(DEPS_MODULES_DIR)/$(THORVG)
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE): THORVG_DEP_BUILD_DIR := $(THORVG_LOCAL_ARCH_BUILD_DIR)
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE): LOG := false
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE): STATIC := false
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE): EXTRA := lottie_expressions
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE): $(LOCAL_ARCH_LIB_DIR)/$(LIBJPEG_TURBO_LIB)
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE): $(LOCAL_ARCH_LIB_DIR)/$(LIBPNG_LIB)
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE): $(LOCAL_ARCH_LIB_DIR)/$(ZLIB_LIB)
$(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE): $(LOCAL_ARCH_LIB_DIR)/$(WEBP_LIB)
	$(SETUP_MESON)

# Build thorvg local arch
$(LOCAL_ARCH_LIB_DIR)/$(THORVG_LIB): DEP_BUILD_DIR := $(THORVG_LOCAL_ARCH_BUILD_DIR)
$(LOCAL_ARCH_LIB_DIR)/$(THORVG_LIB): ARTIFACTS_DIR := ../../../../artifacts/$(LOCAL_ARCH)/usr
$(LOCAL_ARCH_LIB_DIR)/$(THORVG_LIB): $(THORVG_LOCAL_ARCH_BUILD_DIR)/$(NINJA_BUILD_FILE)
	$(NINJA_BUILD)

# Uniffi Bindings - kotlin
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(KOTLIN): export ARTIFACTS_INCLUDE_DIR := ../$(LOCAL_ARCH_INCLUDE_DIR)
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(KOTLIN): export ARTIFACTS_LIB_DIR := ../$(LOCAL_ARCH_LIB_DIR)
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(KOTLIN): export ARTIFACTS_LIB64_DIR := ../$(LOCAL_ARCH_LIB_DIR)64
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(KOTLIN): BINDINGS_LANGUAGE := $(KOTLIN)
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(KOTLIN): $(LOCAL_ARCH_LIB_DIR)/$(THORVG_LIB)
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(KOTLIN): $(RUNTIME_FFI_SRC)
	$(UNIFFI_BINDINGS_BUILD)

# Uniffi Bindings - swift
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(SWIFT): export ARTIFACTS_INCLUDE_DIR := ../$(LOCAL_ARCH_INCLUDE_DIR)
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(SWIFT): export ARTIFACTS_LIB_DIR := ../$(LOCAL_ARCH_LIB_DIR)
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(SWIFT): export ARTIFACTS_LIB64_DIR := ../$(LOCAL_ARCH_LIB_DIR)64
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(SWIFT): BINDINGS_LANGUAGE := $(SWIFT)
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(SWIFT): $(LOCAL_ARCH_LIB_DIR)/$(THORVG_LIB)
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(SWIFT): $(RUNTIME_FFI_SRC)
	$(UNIFFI_BINDINGS_BUILD)

# Uniffi Bindings - cpp (for wasm)
$(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(CPLUSPLUS): $(RUNTIME_FFI_SRC)
	$(UNIFFI_BINDINGS_CPP_BUILD)

# Define all android targets
$(eval $(call DEFINE_TARGET,aarch64-linux-android,aarch64-linux-android,arm64-v8a,arm,aarch64))
$(eval $(call DEFINE_TARGET,armv7-linux-androideabi,armv7a-linux-androideabi,armeabi-v7a,arm,armv7))
$(eval $(call DEFINE_TARGET,x86_64-linux-android,x86_64-linux-android,x86_64,x86_64,x86_64))

# Define all android deps builds
$(eval $(call NEW_ANDROID_DEPS_BUILD,AARCH64_LINUX_ANDROID))
$(eval $(call NEW_ANDROID_DEPS_BUILD,ARMV7_LINUX_ANDROIDEABI))
$(eval $(call NEW_ANDROID_DEPS_BUILD,X86_64_LINUX_ANDROID))

# Define all android builds
$(eval $(call NEW_ANDROID_BUILD,AARCH64_LINUX_ANDROID))
$(eval $(call NEW_ANDROID_BUILD,ARMV7_LINUX_ANDROIDEABI))
$(eval $(call NEW_ANDROID_BUILD,X86_64_LINUX_ANDROID))

# Define all apple targets
$(eval $(call DEFINE_APPLE_TARGET,aarch64-apple-darwin,MAC_ARM64,arm64,arm,aarch64,$(APPLE_MACOSX),$(APPLE_MACOSX_PLATFORM),$(APPLE_MACOSX_SDK)))
$(eval $(call DEFINE_APPLE_TARGET,x86_64-apple-darwin,MAC,x86_64,x86_64,x86_64,$(APPLE_MACOSX),$(APPLE_MACOSX_PLATFORM),$(APPLE_MACOSX_SDK)))
$(eval $(call DEFINE_APPLE_TARGET,aarch64-apple-ios,OS64,arm64,arm,aarch64,$(APPLE_IOS),$(APPLE_IOS_PLATFORM),$(APPLE_IOS_SDK)))
$(eval $(call DEFINE_APPLE_TARGET,x86_64-apple-ios,SIMULATOR64,x86_64,x86_64,x86_64,$(APPLE_IOS),$(APPLE_IOS_SIMULATOR_PLATFORM),$(APPLE_IOS_SIMULATOR_SDK)))
$(eval $(call DEFINE_APPLE_TARGET,aarch64-apple-ios-sim,SIMULATORARM64,arm64,arm,aarch64,$(APPLE_IOS),$(APPLE_IOS_SIMULATOR_PLATFORM),$(APPLE_IOS_SIMULATOR_SDK)))

# Define all apple deps builds
$(eval $(call NEW_APPLE_DEPS_BUILD,AARCH64_APPLE_DARWIN))
$(eval $(call NEW_APPLE_DEPS_BUILD,X86_64_APPLE_DARWIN))
$(eval $(call NEW_APPLE_DEPS_BUILD,AARCH64_APPLE_IOS))
$(eval $(call NEW_APPLE_DEPS_BUILD,X86_64_APPLE_IOS))
$(eval $(call NEW_APPLE_DEPS_BUILD,AARCH64_APPLE_IOS_SIM))

# Define all apple builds
$(eval $(call NEW_APPLE_BUILD,AARCH64_APPLE_DARWIN))
$(eval $(call NEW_APPLE_BUILD,X86_64_APPLE_DARWIN))
$(eval $(call NEW_APPLE_BUILD,AARCH64_APPLE_IOS))
$(eval $(call NEW_APPLE_BUILD,X86_64_APPLE_IOS))
$(eval $(call NEW_APPLE_BUILD,AARCH64_APPLE_IOS_SIM))

# Define all apple framework builds (for release)
$(eval $(call NEW_APPLE_FRAMEWORK,$(APPLE_IOS_FRAMEWORK_TYPE),$(APPLE_IOS_FRAMEWORK_TARGETS),$(APPLE_IOS_PLATFORM),))
$(eval $(call NEW_APPLE_FRAMEWORK,$(APPLE_IOS_SIMULATOR_FRAMEWORK_TYPE),$(APPLE_IOS_SIMULATOR_FRAMEWORK_TARGETS),$(APPLE_IOS_SIMULATOR_PLATFORM),))
$(eval $(call NEW_APPLE_FRAMEWORK,$(APPLE_MACOSX_FRAMEWORK_TYPE),$(APPLE_MACOSX_FRAMEWORK_TARGETS),$(APPLE_MACOSX_PLATFORM),))

# Define WASM targets
$(eval $(call DEFINE_TARGET,wasm32-unknown-emscripten,emscripten,emscripten,x86,i686))

# Define WASM deps builds
$(eval $(call NEW_WASM_DEPS_BUILD,WASM32_UNKNOWN_EMSCRIPTEN))

# Define all WASM builds
$(eval $(call NEW_WASM_BUILD,WASM32_UNKNOWN_EMSCRIPTEN))

# Build apple module-map file
$(RUNTIME_FFI)/$(APPLE_BUILD)/$(MODULE_MAP): MODULE_NAME := $(DOTLOTTIE_PLAYER_MODULE)
$(RUNTIME_FFI)/$(APPLE_BUILD)/$(MODULE_MAP): UMBRELLA_HEADER := $(DOTLOTTIE_PLAYER_HEADER)
$(RUNTIME_FFI)/$(APPLE_BUILD)/$(MODULE_MAP): export OUTPUT_FILE := $(APPLE_MODULE_MAP_FILE)
$(RUNTIME_FFI)/$(APPLE_BUILD)/$(MODULE_MAP): $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)/$(SWIFT)
	$(CREATE_OUTPUT_FILE)

.PHONY: demo-player
demo-player: $(LOCAL_ARCH_LIB_DIR)/$(THORVG_LIB)
	cargo build --manifest-path demo-player/Cargo.toml

.PHONY: $(ANDROID)
$(ANDROID): $(ANDROID_BUILD_TARGETS)

.PHONY: $(APPLE)
$(APPLE): $(APPLE_BUILD_TARGETS)

.PHONY: pre-make-wasm
pre-make-wasm:
	@echo "Copy Cargo.wasm.toml to Cargo.toml..."
	@cp $(RUNTIME_FFI)/Cargo.wasm.toml $(RUNTIME_FFI)/Cargo.toml

.PHONY: post-make-wasm
post-make-wasm:
	@echo "Reset Cargo.toml..."
	@git -C $(RUNTIME_FFI) checkout -- Cargo.toml

.PHONY: $(WASM)
$(WASM):
	@$(MAKE) pre-make-wasm
	@$(MAKE) $(WASM_BUILD_TARGETS)
	@$(MAKE) post-make-wasm

.PHONY: all
all: $(APPLE) $(ANDROID) $(WASM)

.PHONY: deps
deps:
	@git submodule update --init --recursive

# Cleanup extraneous files from the zlib dependency build...
.PHONY: clean-build
clean-build:
	@git --git-dir=$(DEPS_MODULES_DIR)/$(ZLIB)/.git clean -fd &>/dev/null
	@git --git-dir=$(DEPS_MODULES_DIR)/$(ZLIB)/.git checkout . &>/dev/null

.PHONY: clean-deps
clean-deps: clean-build
	@rm -rf $(DEPS_BUILD_DIR) $(DEPS_ARTIFACTS_DIR)

.PHONY: clean
clean: clean-build
	@rm -rf $(RELEASE)
	@cargo clean --manifest-path $(CORE)/Cargo.toml
	@cargo clean --manifest-path $(FMS)/Cargo.toml
	@cargo clean --manifest-path $(RUNTIME_FFI)/Cargo.toml
	@rm -rf $(RUNTIME_FFI)/$(RUNTIME_FFI_UNIFFI_BINDINGS)
	@rm -rf $(RUNTIME_FFI)/$(BUILD)

.PHONY: distclean
distclean: clean clean-deps

.PHONY: mac-setup
mac-setup: export EMSDK_VERSION := $(EMSDK_VERSION)
mac-setup: export UNIFFI_BINDGEN_CPP_VERSION:= $(UNIFFI_BINDGEN_CPP_VERSION)
mac-setup:
	@./.$@.sh

.PHONY: test
test: test-all

.PHONY: test-all
test-all:
	$(info $(YELLOW)Running tests for workspace$(NC))
	@cargo test --manifest-path $(CORE)/Cargo.toml -- --test-threads=1 
	@cargo test --manifest-path $(FMS)/Cargo.toml -- --test-threads=1 
	@cargo test --manifest-path $(RUNTIME_FFI)/Cargo.toml -- --test-threads=1 

.PHONY: bench
bench:
	$(info $(YELLOW)Running benchmarks for workspace$(NC))
	cargo bench --manifest-path $(CORE)/Cargo.toml
	cargo bench --manifest-path $(FMS)/Cargo.toml
	cargo bench --manifest-path $(RUNTIME_FFI)/Cargo.toml

.PHONY: clippy
clippy:
	$(info $(YELLOW)Running clippy for workspace$(NC))
	cargo clippy --manifest-path $(CORE)/Cargo.toml --all-targets --all-features
	# fms has a lot of clippy warnings and errors, so we're ignoring them for now
	# cargo clippy --manifest-path $(FMS)/Cargo.toml --all-targets --all-features
	cargo clippy --manifest-path $(RUNTIME_FFI)/Cargo.toml --all-targets --all-features

.PHONY: help
help:
	@echo "Welcome to the $(GREEN)dotlottie-player$(NC) build system!"
	@echo
	@echo "$(YELLOW)*************************************************************************************************$(NC)"
	@echo "$(YELLOW)NOTE$(NC): If you are a $(GREEN)mac$(NC) user, run $(YELLOW)make mac-setup$(NC) the very first time before performing any builds."
	@echo "      This will ensure your local machine has all the required tools installed."
	@echo
	@echo "      After building a target, you should find your artifacts in the $(GREEN)release$(NC) directory."
	@echo "$(YELLOW)*************************************************************************************************$(NC)"
	@echo
	@echo "$(GREEN)-------------------------------------------------------------------------------------------------$(NC)"
	@echo "The following targets are available for $(GREEN)android$(NC):"
	@printf "  - $(YELLOW)%s$(NC)\n" $(ANDROID_BUILD_TARGETS)
	@echo
	@echo "Use the $(YELLOW)android$(NC) target to build all android targets."
	@echo "$(GREEN)-------------------------------------------------------------------------------------------------$(NC)"
	@echo
	@echo "$(GREEN)-------------------------------------------------------------------------------------------------$(NC)"
	@echo "The following targets are available for $(GREEN)apple$(NC):"
	@printf "  - $(YELLOW)%s$(NC)\n" $(APPLE_BUILD_TARGETS)
	@echo
	@echo "Use the $(YELLOW)apple$(NC) target to build all apple targets."
	@echo "$(GREEN)-------------------------------------------------------------------------------------------------$(NC)"
	@echo
	@echo "$(GREEN)-------------------------------------------------------------------------------------------------$(NC)"
	@echo "The following targets are available for $(GREEN)wasm$(NC):"
	@printf "  - $(YELLOW)%s$(NC)\n" $(WASM_BUILD_TARGETS)
	@echo
	@echo "Use the $(YELLOW)wasm$(NC) target to build all wasm targets."
	@echo "$(GREEN)-------------------------------------------------------------------------------------------------$(NC)"
	@echo
	@echo "The following are make targets you might also find useful:"
	@echo "  - $(YELLOW)demo-player$(NC) - build the demo player"
	@echo "  - $(YELLOW)all$(NC)         - build everything (will take a while on the first run)"
	@echo "  - $(YELLOW)clean$(NC)       - clean up all cargo & release files"
	@echo "  - $(YELLOW)clean-deps$(NC)  - clean up all native dependency builds & artifacts"
	@echo "  - $(YELLOW)clean-build$(NC) - clean up any extraneous build files (useful for ensuring a clean working directory)"
	@echo "  - $(YELLOW)distclean$(NC)   - clean up everything"
	@echo "  - $(YELLOW)test$(NC)        - run all tests"
	@echo "  - $(YELLOW)bench$(NC)       - run all benchmarks"
	@echo "  - $(YELLOW)clippy$(NC)      - run clippy on all projects"
	@echo
	@echo
