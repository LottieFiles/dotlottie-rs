MACOS_FEATURES ?= tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-otf,tvg-lottie-expressions,tvg-threads
MACOS_DEFAULT_FEATURES = tvg,tvg-cpu,c_api,dotlottie,state-machines,theming

ifdef FEATURES
	MACOS_FEATURES = $(FEATURES)
endif

RELEASE_DIR ?= release
MACOS_RELEASE_DIR ?= $(RELEASE_DIR)/macos
DOTLOTTIE_PLAYER_DIR ?= dotlottie-player
MIN_MACOS_VERSION ?= 11.0

MACOS_FFI_LIB_BASE ?= libdotlottie_rs
MACOS_STATIC_LIB := $(MACOS_FFI_LIB_BASE).a
MACOS_SHARED_LIB := $(MACOS_FFI_LIB_BASE).dylib
MACOS_HEADER_FILE := dotlottie_player.h
MACOS_HEADER_DIR := dotlottie-rs/build

MACOS_TARGET_arm64 = aarch64-apple-darwin
MACOS_TARGET_x86_64 = x86_64-apple-darwin

MACOS_TARGETS = aarch64-apple-darwin x86_64-apple-darwin

# Args: $(1) = architecture (arm64, x86_64)
define MACOS_BUILD_ARCH
	@echo "→ Building macOS $(1)..."
	@MACOSX_DEPLOYMENT_TARGET="$(MIN_MACOS_VERSION)" \
	cargo rustc \
		--manifest-path dotlottie-rs/Cargo.toml \
		--crate-type staticlib --crate-type cdylib \
		--target $(MACOS_TARGET_$(1)) \
		--release \
		--no-default-features \
		--features $(MACOS_DEFAULT_FEATURES),$(MACOS_FEATURES) \
		-- -C link-arg=-Wl,-install_name,@rpath/$(MACOS_SHARED_LIB)

	@echo "→ Packaging macOS $(1)..."
	@mkdir -p $(MACOS_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/include
	@mkdir -p $(MACOS_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/lib
	@cp $(MACOS_HEADER_DIR)/$(MACOS_HEADER_FILE) $(MACOS_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/include/
	@cp dotlottie-rs/target/$(MACOS_TARGET_$(1))/release/$(MACOS_STATIC_LIB) \
		dotlottie-rs/target/$(MACOS_TARGET_$(1))/release/$(MACOS_SHARED_LIB) \
		$(MACOS_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/lib/
	@echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(MACOS_RELEASE_DIR)/$(1)/version.txt
	@echo "✓ macOS $(1) build complete"
endef

.PHONY: macos macos-arm64 macos-x86_64 macos-setup macos-clean

macos: $(addprefix macos-,arm64 x86_64)
	@echo "✓ All macOS builds complete"

macos-arm64:
	$(call MACOS_BUILD_ARCH,arm64)

macos-x86_64:
	$(call MACOS_BUILD_ARCH,x86_64)

macos-setup:
	@echo "→ Installing macOS Rust targets..."
	@rustup target add $(MACOS_TARGETS) >/dev/null
	@echo "✓ macOS Rust targets installed"

macos-clean:
	@echo "→ Cleaning macOS builds..."
	@rm -rf $(MACOS_RELEASE_DIR)
	@echo "✓ macOS clean complete"
