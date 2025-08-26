XCODE_PATH ?= $(shell xcode-select -p 2>/dev/null || echo "/Applications/Xcode.app/Contents/Developer")
MIN_IOS_VERSION ?= 13.0
MIN_MACOS_VERSION ?= 11.0
MIN_TVOS_VERSION ?= 13.0
MIN_VISIONOS_VERSION ?= 1.0
MIN_MACCATALYST_VERSION ?= 13.1

# Default Rust features for Apple builds
FEATURES ?= tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-lottie-expressions,tvg-threads
DEFAULT_FEATURES = tvg-v1,tvg-sw,uniffi

# UniFFI Bindings
BINDINGS_DIR ?= dotlottie-ffi/uniffi-bindings
SWIFT_BINDINGS_DIR ?= $(BINDINGS_DIR)/swift

# Release and packaging variables
RELEASE_DIR ?= release
APPLE_RELEASE_DIR ?= $(RELEASE_DIR)/apple
DOTLOTTIE_PLAYER_DIR ?= dotlottie-player
DOTLOTTIE_PLAYER_APPLE_RELEASE_DIR ?= $(APPLE_RELEASE_DIR)/$(DOTLOTTIE_PLAYER_DIR)

# Framework and library names
DOTLOTTIE_PLAYER_MODULE ?= DotLottiePlayer
DOTLOTTIE_PLAYER_FRAMEWORK := $(DOTLOTTIE_PLAYER_MODULE).framework
DOTLOTTIE_PLAYER_XCFRAMEWORK := $(DOTLOTTIE_PLAYER_MODULE).xcframework
RUNTIME_FFI_LIB_BASE ?= libdotlottie_player
RUNTIME_FFI_DYLIB := $(RUNTIME_FFI_LIB_BASE).dylib

# Framework structure
FRAMEWORK_HEADERS := Headers
FRAMEWORK_MODULES := Modules
MODULE_MAP := module.modulemap
INFO_PLIST := Info.plist

# Apple tools
LIPO := lipo
PLISTBUDDY_EXEC := /usr/libexec/PlistBuddy
INSTALL_NAME_TOOL := install_name_tool
XCODEBUILD := xcodebuild
CODESIGN := codesign

# Code signing variables
# Set CODESIGN_IDENTITY to enable code signing (e.g., "Developer ID Application: Your Name")
# Set KEYCHAIN_PASSWORD if using a custom keychain (used in CI environments)

# Get version information
CRATE_VERSION = $(shell grep -m 1 'version =' dotlottie-ffi/Cargo.toml | grep -o '[0-9][0-9.]*')

COMMIT_HASH := $(shell git rev-parse --short HEAD)

# Apple module map template
define APPLE_MODULE_MAP_FILE
framework module $(MODULE_NAME) {
  umbrella header "$(UMBRELLA_HEADER)"
  export *
  module * { export * }
}
endef

# Code signing function
define perform_codesigning
	@if [ -n "$(CODESIGN_IDENTITY)" ]; then \
		echo "→ Unlocking keychain for signing..."; \
		security unlock-keychain -p "$(KEYCHAIN_PASSWORD)" build.keychain; \
		echo "→ Signing XCFramework with identity: $(CODESIGN_IDENTITY)"; \
		$(CODESIGN) --sign "$(CODESIGN_IDENTITY)" --timestamp --options runtime $(1); \
		$(CODESIGN) --verify --verbose $(1); \
		echo "✓ Code signing completed"; \
	else \
		echo "→ Skipping code signing (no identity provided)"; \
	fi
endef





# Helper function to create framework structure and Info.plist
define create_framework_structure
	@echo "Creating framework structure for $(1)..."
	@rm -rf $(1)/$(DOTLOTTIE_PLAYER_FRAMEWORK)
	@mkdir -p $(1)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_HEADERS)
	@mkdir -p $(1)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_MODULES)
	@echo "Creating Info.plist for $(1)..."
	@$(PLISTBUDDY_EXEC) -c "Add :CFBundleIdentifier string com.dotlottie.$(DOTLOTTIE_PLAYER_MODULE)" \
		-c "Add :CFBundleName string $(DOTLOTTIE_PLAYER_MODULE)" \
		-c "Add :CFBundleDisplayName string $(DOTLOTTIE_PLAYER_MODULE)" \
		-c "Add :CFBundleVersion string $(CRATE_VERSION)" \
		-c "Add :CFBundleShortVersionString string $(CRATE_VERSION)" \
		-c "Add :CFBundlePackageType string FMWK" \
		-c "Add :CFBundleExecutable string $(DOTLOTTIE_PLAYER_MODULE)" \
		-c "Add :MinimumOSVersion string $(2)" \
		-c "Add :CFBundleSupportedPlatforms array" \
		-c "Add :CFBundleSupportedPlatforms:0 string $(3)" \
		$(1)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(INFO_PLIST)
endef

# Apple SDK paths
MACOS_SDK = $(XCODE_PATH)/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk
IOS_SDK = $(XCODE_PATH)/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk
IOS_SIMULATOR_SDK = $(XCODE_PATH)/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator.sdk
TVOS_SDK = $(XCODE_PATH)/Platforms/AppleTVOS.platform/Developer/SDKs/AppleTVOS.sdk
TVOS_SIMULATOR_SDK = $(XCODE_PATH)/Platforms/AppleTVSimulator.platform/Developer/SDKs/AppleTVSimulator.sdk
VISIONOS_SDK = $(XCODE_PATH)/Platforms/XROS.platform/Developer/SDKs/XROS.sdk
VISIONOS_SIMULATOR_SDK = $(XCODE_PATH)/Platforms/XRSimulator.platform/Developer/SDKs/XRSimulator.sdk

# Apple targets
APPLE_TARGETS = aarch64-apple-darwin x86_64-apple-darwin aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim aarch64-apple-ios-macabi x86_64-apple-ios-macabi aarch64-apple-visionos aarch64-apple-visionos-sim aarch64-apple-tvos aarch64-apple-tvos-sim

# Apple target mapping
APPLE_TARGET_macos_arm64 = aarch64-apple-darwin
APPLE_TARGET_macos_x86_64 = x86_64-apple-darwin
APPLE_TARGET_ios_arm64 = aarch64-apple-ios
APPLE_TARGET_ios_x86_64 = x86_64-apple-ios
APPLE_TARGET_ios_sim_arm64 = aarch64-apple-ios-sim
APPLE_TARGET_maccatalyst_arm64 = aarch64-apple-ios-macabi
APPLE_TARGET_maccatalyst_x86_64 = x86_64-apple-ios-macabi
APPLE_TARGET_visionos_arm64 = aarch64-apple-visionos
APPLE_TARGET_visionos_sim_arm64 = aarch64-apple-visionos-sim
APPLE_TARGET_tvos_arm64 = aarch64-apple-tvos
APPLE_TARGET_tvos_sim_arm64 = aarch64-apple-tvos-sim

# Framework build directories
APPLE_BUILD_DIR := dotlottie-ffi/build/apple
FRAMEWORK_BUILD_DIR := $(APPLE_BUILD_DIR)/frameworks

# Framework type directories
MACOS_FRAMEWORK_DIR := $(FRAMEWORK_BUILD_DIR)/macos
IOS_FRAMEWORK_DIR := $(FRAMEWORK_BUILD_DIR)/ios
IOS_SIMULATOR_FRAMEWORK_DIR := $(FRAMEWORK_BUILD_DIR)/ios-simulator
MACCATALYST_FRAMEWORK_DIR := $(FRAMEWORK_BUILD_DIR)/maccatalyst
VISIONOS_FRAMEWORK_DIR := $(FRAMEWORK_BUILD_DIR)/visionos
VISIONOS_SIMULATOR_FRAMEWORK_DIR := $(FRAMEWORK_BUILD_DIR)/visionos-simulator
TVOS_FRAMEWORK_DIR := $(FRAMEWORK_BUILD_DIR)/tvos
TVOS_SIMULATOR_FRAMEWORK_DIR := $(FRAMEWORK_BUILD_DIR)/tvos-simulator

# Apple-specific phony targets
.PHONY: apple apple-macos apple-ios apple-maccatalyst apple-visionos apple-tvos apple-macos-arm64 apple-macos-x86_64 apple-ios-arm64 apple-ios-x86_64 apple-ios-sim-arm64 apple-maccatalyst-arm64 apple-maccatalyst-x86_64 apple-visionos-arm64 apple-visionos-sim-arm64 apple-tvos-arm64 apple-tvos-sim-arm64 apple-setup apple-clean apple-code-sign



# Swift bindings file target - automatically rebuilds when UDL changes
$(SWIFT_BINDINGS_DIR)/dotlottie_player.swift: dotlottie-ffi/src/dotlottie_player.udl
	@echo "→ Generating Swift UniFFI bindings..."
	@mkdir -p $(SWIFT_BINDINGS_DIR)
	@cargo run \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--no-default-features \
		--features=uniffi/cli,$(DEFAULT_FEATURES),$(FEATURES) \
		--bin uniffi-bindgen \
		generate dotlottie-ffi/src/dotlottie_player.udl \
		--language swift \
		--out-dir $(SWIFT_BINDINGS_DIR) >/dev/null
	@echo "✓ Swift bindings generated"

# Swift bindings header file target
$(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h: dotlottie-ffi/src/dotlottie_player.udl
	@echo "→ Generating Swift UniFFI header..."
	@mkdir -p $(SWIFT_BINDINGS_DIR)
	@cargo run \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--no-default-features \
		--features=uniffi/cli,$(DEFAULT_FEATURES),$(FEATURES) \
		--bin uniffi-bindgen \
		generate dotlottie-ffi/src/dotlottie_player.udl \
		--language swift \
		--out-dir $(SWIFT_BINDINGS_DIR) >/dev/null
	@echo "✓ Swift bindings header generated"

# Convenience target that depends on the actual files
swift-bindings: $(SWIFT_BINDINGS_DIR)/dotlottie_player.swift $(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h

# Build for all Apple platforms
apple: swift-bindings $(addprefix apple-,macos ios maccatalyst visionos tvos) apple-package

# Build for all macOS architectures
apple-macos: swift-bindings $(addprefix apple-macos-,arm64 x86_64) $(MACOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)
	@echo "✓ macOS build complete"

# Build for all iOS architectures (device + simulator)
apple-ios: swift-bindings $(addprefix apple-ios-,arm64 x86_64 sim-arm64) $(IOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)
	@echo "✓ iOS build complete"

# Build for all Mac Catalyst architectures
apple-maccatalyst: swift-bindings $(addprefix apple-maccatalyst-,arm64 x86_64) $(MACCATALYST_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)
	@echo "✓ Mac Catalyst build complete"

# Build for all visionOS architectures
apple-visionos: swift-bindings $(addprefix apple-visionos-,arm64 sim-arm64) $(VISIONOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)
	@echo "✓ visionOS build complete"

# Build for all tvOS architectures
apple-tvos: swift-bindings $(addprefix apple-tvos-,arm64 sim-arm64) $(TVOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)
	@echo "✓ tvOS build complete"

# macOS ARM64
apple-macos-arm64: swift-bindings apple-check-xcode
	@echo "→ Building macOS ARM64..."
	@SDKROOT="$(MACOS_SDK)" \
	MACOSX_DEPLOYMENT_TARGET="$(MIN_MACOS_VERSION)" \
	CC="$(shell xcrun -sdk macosx --find clang)" \
	CXX="$(shell xcrun -sdk macosx --find clang++)" \
	AR="$(shell xcrun -sdk macosx --find ar)" \
	RANLIB="$(shell xcrun -sdk macosx --find ranlib)" \
	CFLAGS="-arch arm64 -isysroot $(MACOS_SDK) -mmacosx-version-min=$(MIN_MACOS_VERSION)" \
	CXXFLAGS="-arch arm64 -isysroot $(MACOS_SDK) -mmacosx-version-min=$(MIN_MACOS_VERSION)" \
	CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="$(shell xcrun -sdk macosx --find clang)" \
	cargo build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--target $(APPLE_TARGET_macos_arm64) \
		--release \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES) >/dev/null
	@echo "✓ macOS ARM64 build complete"

# macOS x86_64
apple-macos-x86_64: swift-bindings apple-check-xcode
	@echo "→ Building macOS x86_64..."
	@SDKROOT="$(MACOS_SDK)" \
	MACOSX_DEPLOYMENT_TARGET="$(MIN_MACOS_VERSION)" \
	CC="$(shell xcrun -sdk macosx --find clang)" \
	CXX="$(shell xcrun -sdk macosx --find clang++)" \
	AR="$(shell xcrun -sdk macosx --find ar)" \
	RANLIB="$(shell xcrun -sdk macosx --find ranlib)" \
	CFLAGS="-arch x86_64 -isysroot $(MACOS_SDK) -mmacosx-version-min=$(MIN_MACOS_VERSION)" \
	CXXFLAGS="-arch x86_64 -isysroot $(MACOS_SDK) -mmacosx-version-min=$(MIN_MACOS_VERSION)" \
	CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER="$(shell xcrun -sdk macosx --find clang)" \
	cargo build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--target $(APPLE_TARGET_macos_x86_64) \
		--release \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES) >/dev/null
	@echo "✓ macOS x86_64 build complete"

# iOS ARM64 (device)
apple-ios-arm64: swift-bindings apple-check-xcode
	@echo "→ Building iOS ARM64..."
	@SDKROOT="$(IOS_SDK)" \
	IPHONEOS_DEPLOYMENT_TARGET="$(MIN_IOS_VERSION)" \
	CC="$(shell xcrun -sdk iphoneos --find clang)" \
	CXX="$(shell xcrun -sdk iphoneos --find clang++)" \
	AR="$(shell xcrun -sdk iphoneos --find ar)" \
	RANLIB="$(shell xcrun -sdk iphoneos --find ranlib)" \
	CFLAGS="-arch arm64 -isysroot $(IOS_SDK) -miphoneos-version-min=$(MIN_IOS_VERSION)" \
	CXXFLAGS="-arch arm64 -isysroot $(IOS_SDK) -miphoneos-version-min=$(MIN_IOS_VERSION)" \
	CARGO_TARGET_AARCH64_APPLE_IOS_LINKER="$(shell xcrun -sdk iphoneos --find clang)" \
	cargo build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--target $(APPLE_TARGET_ios_arm64) \
		--release \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES) >/dev/null
	@echo "✓ iOS ARM64 build complete"

# iOS x86_64 (simulator)
apple-ios-x86_64: swift-bindings apple-check-xcode
	@echo "→ Building iOS x86_64 simulator..."
	@SDKROOT="$(IOS_SIMULATOR_SDK)" \
	IPHONEOS_DEPLOYMENT_TARGET="$(MIN_IOS_VERSION)" \
	CC="$(shell xcrun -sdk iphonesimulator --find clang)" \
	CXX="$(shell xcrun -sdk iphonesimulator --find clang++)" \
	AR="$(shell xcrun -sdk iphonesimulator --find ar)" \
	RANLIB="$(shell xcrun -sdk iphonesimulator --find ranlib)" \
	CFLAGS="-arch x86_64 -isysroot $(IOS_SIMULATOR_SDK) -miphoneos-version-min=$(MIN_IOS_VERSION)" \
	CXXFLAGS="-arch x86_64 -isysroot $(IOS_SIMULATOR_SDK) -miphoneos-version-min=$(MIN_IOS_VERSION)" \
	CARGO_TARGET_X86_64_APPLE_IOS_LINKER="$(shell xcrun -sdk iphonesimulator --find clang)" \
	cargo build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--target $(APPLE_TARGET_ios_x86_64) \
		--release \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES) >/dev/null
	@echo "✓ iOS x86_64 simulator build complete"

# iOS ARM64 Simulator
apple-ios-sim-arm64: swift-bindings apple-check-xcode
	@echo "→ Building iOS ARM64 simulator..."
	@SDKROOT="$(IOS_SIMULATOR_SDK)" \
	IPHONEOS_DEPLOYMENT_TARGET="$(MIN_IOS_VERSION)" \
	CC="$(shell xcrun -sdk iphonesimulator --find clang)" \
	CXX="$(shell xcrun -sdk iphonesimulator --find clang++)" \
	AR="$(shell xcrun -sdk iphonesimulator --find ar)" \
	RANLIB="$(shell xcrun -sdk iphonesimulator --find ranlib)" \
	CFLAGS="-arch arm64 -isysroot $(IOS_SIMULATOR_SDK) -miphoneos-version-min=$(MIN_IOS_VERSION)" \
	CXXFLAGS="-arch arm64 -isysroot $(IOS_SIMULATOR_SDK) -miphoneos-version-min=$(MIN_IOS_VERSION)" \
	CARGO_TARGET_AARCH64_APPLE_IOS_SIM_LINKER="$(shell xcrun -sdk iphonesimulator --find clang)" \
	cargo build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		--target $(APPLE_TARGET_ios_sim_arm64) \
		--release \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES) >/dev/null
	@echo "✓ iOS ARM64 simulator build complete"

# Mac Catalyst ARM64
apple-maccatalyst-arm64: swift-bindings apple-check-xcode
	@echo "→ Building Mac Catalyst ARM64..."
	@SDKROOT="$(MACOS_SDK)" \
	IPHONEOS_DEPLOYMENT_TARGET="$(MIN_MACCATALYST_VERSION)" \
	CC="$(shell xcrun -sdk macosx --find clang)" \
	CXX="$(shell xcrun -sdk macosx --find clang++)" \
	AR="$(shell xcrun -sdk macosx --find ar)" \
	RANLIB="$(shell xcrun -sdk macosx --find ranlib)" \
	CFLAGS="-arch arm64 -isysroot $(MACOS_SDK) -target arm64-apple-ios$(MIN_MACCATALYST_VERSION)-macabi" \
	CXXFLAGS="-arch arm64 -isysroot $(MACOS_SDK) -target arm64-apple-ios$(MIN_MACCATALYST_VERSION)-macabi" \
	CARGO_TARGET_AARCH64_APPLE_IOS_MACABI_LINKER="$(shell xcrun -sdk macosx --find clang)" \
	cargo +nightly build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		-Z build-std=std,panic_abort \
		--target $(APPLE_TARGET_maccatalyst_arm64) \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES) \
		--release >/dev/null
	@echo "✓ Mac Catalyst ARM64 build complete"

# Mac Catalyst x86_64
apple-maccatalyst-x86_64: swift-bindings apple-check-xcode
	@echo "→ Building Mac Catalyst x86_64..."
	@SDKROOT="$(MACOS_SDK)" \
	IPHONEOS_DEPLOYMENT_TARGET="$(MIN_MACCATALYST_VERSION)" \
	CC="$(shell xcrun -sdk macosx --find clang)" \
	CXX="$(shell xcrun -sdk macosx --find clang++)" \
	AR="$(shell xcrun -sdk macosx --find ar)" \
	RANLIB="$(shell xcrun -sdk macosx --find ranlib)" \
	CFLAGS="-arch x86_64 -isysroot $(MACOS_SDK) -target x86_64-apple-ios$(MIN_MACCATALYST_VERSION)-macabi" \
	CXXFLAGS="-arch x86_64 -isysroot $(MACOS_SDK) -target x86_64-apple-ios$(MIN_MACCATALYST_VERSION)-macabi" \
	CARGO_TARGET_X86_64_APPLE_IOS_MACABI_LINKER="$(shell xcrun -sdk macosx --find clang)" \
	cargo +nightly build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		-Z build-std=std,panic_abort \
		--target $(APPLE_TARGET_maccatalyst_x86_64) \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES) \
		--release >/dev/null
	@echo "✓ Mac Catalyst x86_64 build complete"

# visionOS ARM64 (device)
apple-visionos-arm64: swift-bindings apple-check-xcode
	@echo "→ Building visionOS ARM64 (nightly)..."
	@SDKROOT="$(VISIONOS_SDK)" \
	XROS_DEPLOYMENT_TARGET="$(MIN_VISIONOS_VERSION)" \
	CC="$(shell xcrun -sdk xros --find clang)" \
	CXX="$(shell xcrun -sdk xros --find clang++)" \
	AR="$(shell xcrun -sdk xros --find ar)" \
	RANLIB="$(shell xcrun -sdk xros --find ranlib)" \
	CFLAGS="-arch arm64 -isysroot $(VISIONOS_SDK) -target arm64-apple-xros$(MIN_VISIONOS_VERSION)" \
	CXXFLAGS="-arch arm64 -isysroot $(VISIONOS_SDK) -target arm64-apple-xros$(MIN_VISIONOS_VERSION)" \
	CARGO_TARGET_AARCH64_APPLE_VISIONOS_LINKER="$(shell xcrun -sdk xros --find clang)" \
	cargo +nightly build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		-Z build-std=std,panic_abort \
		--target $(APPLE_TARGET_visionos_arm64) \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES) \
		--release >/dev/null
	@echo "✓ visionOS ARM64 build complete"

# visionOS ARM64 Simulator
apple-visionos-sim-arm64: swift-bindings apple-check-xcode
	@echo "→ Building visionOS ARM64 simulator (nightly)..."
	@SDKROOT="$(VISIONOS_SIMULATOR_SDK)" \
	XROS_DEPLOYMENT_TARGET="$(MIN_VISIONOS_VERSION)" \
	CC="$(shell xcrun -sdk xrsimulator --find clang)" \
	CXX="$(shell xcrun -sdk xrsimulator --find clang++)" \
	AR="$(shell xcrun -sdk xrsimulator --find ar)" \
	RANLIB="$(shell xcrun -sdk xrsimulator --find ranlib)" \
	CFLAGS="-arch arm64 -isysroot $(VISIONOS_SIMULATOR_SDK) -target arm64-apple-xros$(MIN_VISIONOS_VERSION)-simulator" \
	CXXFLAGS="-arch arm64 -isysroot $(VISIONOS_SIMULATOR_SDK) -target arm64-apple-xros$(MIN_VISIONOS_VERSION)-simulator" \
	CARGO_TARGET_AARCH64_APPLE_VISIONOS_SIM_LINKER="$(shell xcrun -sdk xrsimulator --find clang)" \
	cargo +nightly build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		-Z build-std=std,panic_abort \
		--target $(APPLE_TARGET_visionos_sim_arm64) \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES) \
		--release >/dev/null
	@echo "✓ visionOS ARM64 simulator build complete"

# tvOS ARM64 (device)
apple-tvos-arm64: swift-bindings apple-check-xcode
	@echo "→ Building tvOS ARM64 (nightly)..."
	@SDKROOT="$(TVOS_SDK)" \
	TVOS_DEPLOYMENT_TARGET="$(MIN_TVOS_VERSION)" \
	CC="$(shell xcrun -sdk appletvos --find clang)" \
	CXX="$(shell xcrun -sdk appletvos --find clang++)" \
	AR="$(shell xcrun -sdk appletvos --find ar)" \
	RANLIB="$(shell xcrun -sdk appletvos --find ranlib)" \
	CFLAGS="-arch arm64 -isysroot $(TVOS_SDK) -mtvos-version-min=$(MIN_TVOS_VERSION)" \
	CXXFLAGS="-arch arm64 -isysroot $(TVOS_SDK) -mtvos-version-min=$(MIN_TVOS_VERSION)" \
	CARGO_TARGET_AARCH64_APPLE_TVOS_LINKER="$(shell xcrun -sdk appletvos --find clang)" \
	cargo +nightly build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		-Z build-std=std,panic_abort \
		--target $(APPLE_TARGET_tvos_arm64) \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES) \
		--release >/dev/null
	@echo "✓ tvOS ARM64 build complete"

# tvOS ARM64 Simulator
apple-tvos-sim-arm64: swift-bindings apple-check-xcode
	@echo "→ Building tvOS ARM64 simulator (nightly)..."
	@SDKROOT="$(TVOS_SIMULATOR_SDK)" \
	TVOS_DEPLOYMENT_TARGET="$(MIN_TVOS_VERSION)" \
	CC="$(shell xcrun -sdk appletvsimulator --find clang)" \
	CXX="$(shell xcrun -sdk appletvsimulator --find clang++)" \
	AR="$(shell xcrun -sdk appletvsimulator --find ar)" \
	RANLIB="$(shell xcrun -sdk appletvsimulator --find ranlib)" \
	CFLAGS="-arch arm64 -isysroot $(TVOS_SIMULATOR_SDK) -mtvos-version-min=$(MIN_TVOS_VERSION)" \
	CXXFLAGS="-arch arm64 -isysroot $(TVOS_SIMULATOR_SDK) -mtvos-version-min=$(MIN_TVOS_VERSION)" \
	CARGO_TARGET_AARCH64_APPLE_TVOS_SIM_LINKER="$(shell xcrun -sdk appletvsimulator --find clang)" \
	cargo +nightly build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		-Z build-std=std,panic_abort \
		--target $(APPLE_TARGET_tvos_sim_arm64) \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES) \
		--release >/dev/null
	@echo "✓ tvOS ARM64 simulator build complete"

# Framework creation targets
$(MACOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK): apple-macos-arm64 apple-macos-x86_64
	@echo "→ Creating macOS framework..."
	@$(call create_framework_structure,$(MACOS_FRAMEWORK_DIR),$(MIN_MACOS_VERSION),MacOSX)
	@rm -f $(MACOS_FRAMEWORK_DIR)/$(RUNTIME_FFI_DYLIB)
	@$(LIPO) -create dotlottie-ffi/target/$(APPLE_TARGET_macos_arm64)/release/$(RUNTIME_FFI_DYLIB) dotlottie-ffi/target/$(APPLE_TARGET_macos_x86_64)/release/$(RUNTIME_FFI_DYLIB) -o $(MACOS_FRAMEWORK_DIR)/$(RUNTIME_FFI_DYLIB)
	@cp $(MACOS_FRAMEWORK_DIR)/$(RUNTIME_FFI_DYLIB) $(MACOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@if [ -f "$(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h" ]; then \
		cp $(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h $(MACOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_HEADERS)/$(DOTLOTTIE_PLAYER_MODULE).h; \
	fi
	@echo 'framework module $(DOTLOTTIE_PLAYER_MODULE) {' > $(MACOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  umbrella header "$(DOTLOTTIE_PLAYER_MODULE).h"' >> $(MACOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  export *' >> $(MACOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  module * { export * }' >> $(MACOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '}' >> $(MACOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@cp $(MACOS_FRAMEWORK_DIR)/$(MODULE_MAP) $(MACOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_MODULES)/
	@$(INSTALL_NAME_TOOL) -id @rpath/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE) $(MACOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@echo "✓ macOS framework created"

$(IOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK): apple-ios-arm64
	@echo "Creating iOS framework..."
	$(call create_framework_structure,$(IOS_FRAMEWORK_DIR),$(MIN_IOS_VERSION),iPhoneOS)
	@echo "Creating iOS binary..."
	cp dotlottie-ffi/target/$(APPLE_TARGET_ios_arm64)/release/$(RUNTIME_FFI_DYLIB) $(IOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@if [ -f "$(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h" ]; then \
		cp $(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h $(IOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_HEADERS)/$(DOTLOTTIE_PLAYER_MODULE).h; \
	fi
	@echo "Creating module map for iOS..."
	@echo 'framework module $(DOTLOTTIE_PLAYER_MODULE) {' > $(IOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  umbrella header "$(DOTLOTTIE_PLAYER_MODULE).h"' >> $(IOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  export *' >> $(IOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  module * { export * }' >> $(IOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '}' >> $(IOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	cp $(IOS_FRAMEWORK_DIR)/$(MODULE_MAP) $(IOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_MODULES)/
	$(INSTALL_NAME_TOOL) -id @rpath/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE) $(IOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@echo "iOS framework created: $(IOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)"

$(IOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK): apple-ios-x86_64 apple-ios-sim-arm64
	@echo "Creating iOS Simulator framework..."
	$(call create_framework_structure,$(IOS_SIMULATOR_FRAMEWORK_DIR),$(MIN_IOS_VERSION),iPhoneSimulator)
	@echo "Creating universal binary for iOS Simulator..."
	@rm -f $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(RUNTIME_FFI_DYLIB)
	$(LIPO) -create dotlottie-ffi/target/$(APPLE_TARGET_ios_x86_64)/release/$(RUNTIME_FFI_DYLIB) dotlottie-ffi/target/$(APPLE_TARGET_ios_sim_arm64)/release/$(RUNTIME_FFI_DYLIB) -o $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(RUNTIME_FFI_DYLIB)
	cp $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(RUNTIME_FFI_DYLIB) $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@if [ -f "$(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h" ]; then \
		cp $(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_HEADERS)/$(DOTLOTTIE_PLAYER_MODULE).h; \
	fi
	@echo "Creating module map for iOS Simulator..."
	@echo 'framework module $(DOTLOTTIE_PLAYER_MODULE) {' > $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  umbrella header "$(DOTLOTTIE_PLAYER_MODULE).h"' >> $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  export *' >> $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  module * { export * }' >> $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '}' >> $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	cp $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP) $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_MODULES)/
	$(INSTALL_NAME_TOOL) -id @rpath/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE) $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@echo "iOS Simulator framework created: $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)"

$(MACCATALYST_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK): apple-maccatalyst-arm64 apple-maccatalyst-x86_64
	@echo "→ Creating Mac Catalyst framework..."
	@$(call create_framework_structure,$(MACCATALYST_FRAMEWORK_DIR),$(MIN_MACCATALYST_VERSION),MacOSX)
	@rm -f $(MACCATALYST_FRAMEWORK_DIR)/$(RUNTIME_FFI_DYLIB)
	@$(LIPO) -create dotlottie-ffi/target/$(APPLE_TARGET_maccatalyst_arm64)/release/$(RUNTIME_FFI_DYLIB) dotlottie-ffi/target/$(APPLE_TARGET_maccatalyst_x86_64)/release/$(RUNTIME_FFI_DYLIB) -o $(MACCATALYST_FRAMEWORK_DIR)/$(RUNTIME_FFI_DYLIB)
	@cp $(MACCATALYST_FRAMEWORK_DIR)/$(RUNTIME_FFI_DYLIB) $(MACCATALYST_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@if [ -f "$(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h" ]; then \
		cp $(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h $(MACCATALYST_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_HEADERS)/$(DOTLOTTIE_PLAYER_MODULE).h; \
	fi
	@echo 'framework module $(DOTLOTTIE_PLAYER_MODULE) {' > $(MACCATALYST_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  umbrella header "$(DOTLOTTIE_PLAYER_MODULE).h"' >> $(MACCATALYST_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  export *' >> $(MACCATALYST_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  module * { export * }' >> $(MACCATALYST_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '}' >> $(MACCATALYST_FRAMEWORK_DIR)/$(MODULE_MAP)
	@cp $(MACCATALYST_FRAMEWORK_DIR)/$(MODULE_MAP) $(MACCATALYST_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_MODULES)/
	@$(INSTALL_NAME_TOOL) -id @rpath/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE) $(MACCATALYST_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@echo "✓ Mac Catalyst framework created"

$(VISIONOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK): apple-visionos-arm64
	@echo "Creating visionOS framework..."
	$(call create_framework_structure,$(VISIONOS_FRAMEWORK_DIR),$(MIN_VISIONOS_VERSION),XROS)
	@echo "Creating visionOS binary..."
	cp dotlottie-ffi/target/$(APPLE_TARGET_visionos_arm64)/release/$(RUNTIME_FFI_DYLIB) $(VISIONOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@if [ -f "$(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h" ]; then \
		cp $(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h $(VISIONOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_HEADERS)/$(DOTLOTTIE_PLAYER_MODULE).h; \
	fi
	@echo "Creating module map for visionOS..."
	@echo 'framework module $(DOTLOTTIE_PLAYER_MODULE) {' > $(VISIONOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  umbrella header "$(DOTLOTTIE_PLAYER_MODULE).h"' >> $(VISIONOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  export *' >> $(VISIONOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  module * { export * }' >> $(VISIONOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '}' >> $(VISIONOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	cp $(VISIONOS_FRAMEWORK_DIR)/$(MODULE_MAP) $(VISIONOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_MODULES)/
	$(INSTALL_NAME_TOOL) -id @rpath/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE) $(VISIONOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@echo "visionOS framework created: $(VISIONOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)"

$(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK): apple-visionos-sim-arm64
	@echo "Creating visionOS Simulator framework..."
	$(call create_framework_structure,$(VISIONOS_SIMULATOR_FRAMEWORK_DIR),$(MIN_VISIONOS_VERSION),XRSimulator)
	@echo "Creating visionOS Simulator binary..."
	cp dotlottie-ffi/target/$(APPLE_TARGET_visionos_sim_arm64)/release/$(RUNTIME_FFI_DYLIB) $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@if [ -f "$(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h" ]; then \
		cp $(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_HEADERS)/$(DOTLOTTIE_PLAYER_MODULE).h; \
	fi
	@echo "Creating module map for visionOS Simulator..."
	@echo 'framework module $(DOTLOTTIE_PLAYER_MODULE) {' > $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  umbrella header "$(DOTLOTTIE_PLAYER_MODULE).h"' >> $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  export *' >> $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  module * { export * }' >> $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '}' >> $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	cp $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP) $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_MODULES)/
	$(INSTALL_NAME_TOOL) -id @rpath/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE) $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@echo "visionOS Simulator framework created: $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)"

$(TVOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK): apple-tvos-arm64
	@echo "Creating tvOS framework..."
	$(call create_framework_structure,$(TVOS_FRAMEWORK_DIR),$(MIN_TVOS_VERSION),AppleTVOS)
	@echo "Creating tvOS binary..."
	cp dotlottie-ffi/target/$(APPLE_TARGET_tvos_arm64)/release/$(RUNTIME_FFI_DYLIB) $(TVOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@if [ -f "$(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h" ]; then \
		cp $(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h $(TVOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_HEADERS)/$(DOTLOTTIE_PLAYER_MODULE).h; \
	fi
	@echo "Creating module map for tvOS..."
	@echo 'framework module $(DOTLOTTIE_PLAYER_MODULE) {' > $(TVOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  umbrella header "$(DOTLOTTIE_PLAYER_MODULE).h"' >> $(TVOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  export *' >> $(TVOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  module * { export * }' >> $(TVOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '}' >> $(TVOS_FRAMEWORK_DIR)/$(MODULE_MAP)
	cp $(TVOS_FRAMEWORK_DIR)/$(MODULE_MAP) $(TVOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_MODULES)/
	$(INSTALL_NAME_TOOL) -id @rpath/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE) $(TVOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@echo "tvOS framework created: $(TVOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)"

$(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK): apple-tvos-sim-arm64
	@echo "Creating tvOS Simulator framework..."
	$(call create_framework_structure,$(TVOS_SIMULATOR_FRAMEWORK_DIR),$(MIN_TVOS_VERSION),AppleTVSimulator)
	@echo "Creating tvOS Simulator binary..."
	cp dotlottie-ffi/target/$(APPLE_TARGET_tvos_sim_arm64)/release/$(RUNTIME_FFI_DYLIB) $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@if [ -f "$(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h" ]; then \
		cp $(SWIFT_BINDINGS_DIR)/$(DOTLOTTIE_PLAYER_MODULE).h $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_HEADERS)/$(DOTLOTTIE_PLAYER_MODULE).h; \
	fi
	@echo "Creating module map for tvOS Simulator..."
	@echo 'framework module $(DOTLOTTIE_PLAYER_MODULE) {' > $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  umbrella header "$(DOTLOTTIE_PLAYER_MODULE).h"' >> $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  export *' >> $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '  module * { export * }' >> $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	@echo '}' >> $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP)
	cp $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(MODULE_MAP) $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(FRAMEWORK_MODULES)/
	$(INSTALL_NAME_TOOL) -id @rpath/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE) $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)/$(DOTLOTTIE_PLAYER_MODULE)
	@echo "tvOS Simulator framework created: $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)"

# Create all frameworks
apple-frameworks: $(MACOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) $(IOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) $(VISIONOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) $(TVOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) $(MACCATALYST_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK)
	@echo "✓ All Apple frameworks created"

# Code signing target
apple-code-sign:
	@echo "→ Code signing XCFramework..."
	$(call perform_codesigning,$(APPLE_RELEASE_DIR)/$(DOTLOTTIE_PLAYER_XCFRAMEWORK))

# Package Apple release
apple-package: apple-frameworks
	@echo "→ Creating Apple release package..."
	@mkdir -p $(APPLE_RELEASE_DIR)
	@rm -rf $(APPLE_RELEASE_DIR)/*
	@$(XCODEBUILD) -create-xcframework \
		-framework $(MACOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) \
		-framework $(IOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) \
		-framework $(IOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) \
		-framework $(MACCATALYST_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) \
		-framework $(VISIONOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) \
		-framework $(VISIONOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) \
		-framework $(TVOS_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) \
		-framework $(TVOS_SIMULATOR_FRAMEWORK_DIR)/$(DOTLOTTIE_PLAYER_FRAMEWORK) \
		-output $(APPLE_RELEASE_DIR)/$(DOTLOTTIE_PLAYER_XCFRAMEWORK) >/dev/null
	@if [ -f "$(SWIFT_BINDINGS_DIR)/dotlottie_player.swift" ]; then \
		cp $(SWIFT_BINDINGS_DIR)/dotlottie_player.swift $(APPLE_RELEASE_DIR)/; \
	fi
	
	# Code sign the XCFramework
	$(call perform_codesigning,$(APPLE_RELEASE_DIR)/$(DOTLOTTIE_PLAYER_XCFRAMEWORK))
	
	# Create version file and final tarball
	@echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(APPLE_RELEASE_DIR)/version.txt
	
	@echo "✓ Apple release package created: $(APPLE_RELEASE_DIR)/"

# Check if Xcode is available
apple-check-xcode:
	@if [ ! -d "$(XCODE_PATH)" ]; then \
		echo "Error: Xcode not found at $(XCODE_PATH)"; \
		echo "Please install Xcode or set XCODE_PATH to the correct location"; \
		exit 1; \
	fi
	@if [ ! -d "$(MACOS_SDK)" ]; then \
		echo "Error: macOS SDK not found at $(MACOS_SDK)"; \
		echo "Please ensure Xcode Command Line Tools are installed"; \
		exit 1; \
	fi

# Install Apple targets if not already installed
apple-setup:
	@echo "→ Installing Apple Rust targets..."
	@rustup target add aarch64-apple-darwin x86_64-apple-darwin aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim aarch64-apple-ios-macabi x86_64-apple-ios-macabi >/dev/null
	@rustup component add rust-src --toolchain nightly
	@echo "✓ Apple targets installed"


# Clean all Apple builds
apple-clean:
	@echo "→ Cleaning Apple builds..."
	@cargo clean --manifest-path dotlottie-ffi/Cargo.toml >/dev/null 2>&1
	@rm -rf $(SWIFT_BINDINGS_DIR)
	@rm -rf $(APPLE_BUILD_DIR)
	@rm -rf $(APPLE_RELEASE_DIR)
	@echo "✓ Apple builds cleaned"