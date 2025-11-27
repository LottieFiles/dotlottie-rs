EMSDK_VERSION ?= 3.1.74
UNIFFI_BINDGEN_CPP ?= uniffi-bindgen-cpp
UNIFFI_BINDGEN_CPP_VERSION ?= v0.7.3+v0.28.3

RUST_TOOLCHAIN ?= nightly-2025-08-01

# Default Rust features for WASM builds
WASM_FEATURES ?= tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-lottie-expressions
WASM_DEFAULT_FEATURES = tvg,tvg-sw,c_api

ifdef FEATURES
	WASM_FEATURES = $(FEATURES)
endif

# WASM/Emscripten configuration
EMSDK := emsdk
EMSDK_DIR := deps/modules/$(EMSDK)
EMSDK_ENV := emsdk_env.sh

# WASM module configuration
WASM_MODULE := dotlottie_player
WASM_TARGET := wasm32-unknown-emscripten
WASM_BUILD_DIR := dotlottie-rs/build/wasm
BUILD_DIR := dotlottie-rs/build

# Get version information
CRATE_VERSION = $(shell grep -m 1 'version =' dotlottie-rs/Cargo.toml | grep -o '[0-9][0-9.]*')
COMMIT_HASH := $(shell git rev-parse --short HEAD)

# Release directories
WASM_RELEASE_DIR ?= release/wasm

ifneq (,$(findstring tvg-simd,$(FEATURES)))
  EMSIMD_FLAGS += -msimd128
endif

# WASM-specific phony targets
.PHONY: wasm wasm-setup wasm-install-emsdk wasm-build-rust wasm-link wasm-package wasm-clean


# Initialize emsdk submodule
wasm-init-submodule:
	@echo "→ Initializing emsdk submodule..."
	@if [ ! -f "$(EMSDK_DIR)/emsdk" ]; then \
		git submodule update --init --recursive $(EMSDK_DIR) >/dev/null; \
	fi
	@echo "✓ emsdk submodule ready"

# Install and activate specific emsdk version
wasm-install-emsdk: wasm-init-submodule
	@echo "→ Installing emsdk $(EMSDK_VERSION)..."
	@cd $(EMSDK_DIR) && \
		./emsdk install $(EMSDK_VERSION) >/dev/null && \
		./emsdk activate $(EMSDK_VERSION) >/dev/null
	@echo "✓ emsdk $(EMSDK_VERSION) installed and activated"

# ============================================================================
# Old UniFFI-based WASM targets (deprecated)
# Preserved for reference - old builds moved to wasm.mk.bak
# Use 'make wasm' for new C API build
# ============================================================================

# Check WASM build environment
wasm-check-env:
	@echo "Checking WASM build environment..."
	@if [ ! -d "$(EMSDK_DIR)" ]; then \
		echo "Error: emsdk not found at $(EMSDK_DIR)"; \
		echo "Run 'make wasm-setup' to initialize emsdk"; \
		exit 1; \
	fi
	@if [ ! -f "$(EMSDK_DIR)/$(EMSDK_ENV)" ]; then \
		echo "Error: emsdk environment not found"; \
		echo "Run 'make wasm-install-emsdk' to install emsdk"; \
		exit 1; \
	fi
	@if ! command -v $(UNIFFI_BINDGEN_CPP) >/dev/null 2>&1; then \
		echo "Warning: $(UNIFFI_BINDGEN_CPP) not found in PATH"; \
		echo "C++ bindings generation may fail"; \
	fi
	@if ! rustup toolchain list | grep -q nightly; then \
		echo "Warning: Rust nightly toolchain not found"; \
		echo "Install with: rustup toolchain install nightly"; \
		echo "WASM build requires nightly for aggressive size optimizations"; \
	fi

# Install WASM Rust target and all dependencies
wasm-setup: wasm-init-submodule wasm-install-emsdk
	@echo "→ Installing Rust nightly toolchain and WASM target..."
	@rustup toolchain install $(RUST_TOOLCHAIN) >/dev/null
	@rustup component add rust-src --toolchain $(RUST_TOOLCHAIN) >/dev/null
	@rustup target add --toolchain $(RUST_TOOLCHAIN) $(WASM_TARGET) >/dev/null
	@echo "✓ WASM targets and nightly toolchain installed"
	@echo "→ Installing uniffi-bindgen-cpp..."
	@cargo install uniffi-bindgen-cpp --git https://github.com/NordSecurity/uniffi-bindgen-cpp --tag $(UNIFFI_BINDGEN_CPP_VERSION) >/dev/null
	@echo "✓ uniffi-bindgen-cpp installed"



# ============================================================================
# New WASM C API Build (Direct C function exports - No C++ wrapper)
# ============================================================================

# New WASM configuration

# Note: C API function export list is auto-generated from the C header during link step

# Build Rust library for WASM with C API (NO C++ wrapper needed!)
wasm-build-rust: wasm-check-env
	@echo "→ Building Rust library for WASM (C API - direct export)..."
	@bash -c "source $(EMSDK_DIR)/$(EMSDK_ENV) && \
	CC=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emcc \
	CXX=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++ \
	AR=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emar \
	CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_LINKER=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emcc \
	CXXFLAGS='-isystem $(PWD)/$(EMSDK_DIR)/upstream/emscripten/cache/sysroot/include/c++/v1 -isystem $(PWD)/$(EMSDK_DIR)/upstream/emscripten/cache/sysroot/include' \
	BINDGEN_EXTRA_CLANG_ARGS='-isysroot $(PWD)/$(EMSDK_DIR)/upstream/emscripten/cache/sysroot' \
	RUSTFLAGS='-C panic=abort -C link-arg=--no-entry -C link-arg=-sERROR_ON_UNDEFINED_SYMBOLS=0' \
	cargo +$(RUST_TOOLCHAIN) build \
		--manifest-path dotlottie-rs/Cargo.toml \
		-Z build-std=std,panic_abort \
		-Z build-std-features=panic_immediate_abort \
		--target $(WASM_TARGET) \
		--no-default-features \
		--features $(WASM_DEFAULT_FEATURES),$(WASM_FEATURES) \
		--release"
	@echo "✓ Rust library built for WASM"

# Install npm dependencies for TypeScript support
wasm-install-npm-deps:
	@echo "→ Installing npm dependencies for TypeScript support..."
	@bash -c "source $(EMSDK_DIR)/$(EMSDK_ENV) && \
		cd $(PWD)/$(EMSDK_DIR)/upstream/emscripten && \
		if [ ! -d node_modules ] || [ ! -f node_modules/.bin/tsc ]; then \
			npm install >/dev/null 2>&1; \
		fi"
	@echo "✓ npm dependencies installed"

# Link WASM module - Direct C API
wasm-link: wasm-build-rust  wasm-install-npm-deps
	@echo "→ Linking WASM module (direct C API)..."
	@mkdir -p $(WASM_BUILD_DIR)
	@echo "  Auto-generating export list from C header..."
	@bash -c "source $(EMSDK_DIR)/$(EMSDK_ENV) && \
		C_API_EXPORTED_FUNCTIONS=\$$(grep -o 'dotlottie_[a-z_]*(' $(BUILD_DIR)/dotlottie_player.h | sed 's/(//g' | sort -u | sed 's/^/_/' | paste -sd ',' - | sed 's/,/\",\"/g' | sed 's/^/\"/' | sed 's/\$$/\",\"_malloc\",\"_free\"/') && \
		echo \"  Exporting \$$(echo \$$C_API_EXPORTED_FUNCTIONS | grep -o '_dotlottie' | wc -l | tr -d ' ') C API functions\" && \
		$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emcc \
			-o $(PWD)/$(WASM_BUILD_DIR)/$(WASM_MODULE).js \
			$(PWD)/dotlottie-rs/target/$(WASM_TARGET)/release/libdotlottie_rs.a \
			-Wl,-u,htons \
			-Wl,-u,ntohs \
			-Wl,-u,htonl \
			-flto \
			-Oz \
			-sWASM=1 \
			-sALLOW_MEMORY_GROWTH=1 \
			-sFORCE_FILESYSTEM=0 \
			-sMODULARIZE=1 \
			-sEXPORT_NAME=createDotLottieRuntimeModule \
			-sEXPORT_ES6=1 \
			-sUSE_ES6_IMPORT_META=0 \
			-sDYNAMIC_EXECUTION=0 \
			-sENVIRONMENT=web \
			-sMIN_SAFARI_VERSION=130000 \
			-sFILESYSTEM=0 \
			-sEXPORTED_FUNCTIONS=\"[\$$C_API_EXPORTED_FUNCTIONS]\" \
			-sEXPORTED_RUNTIME_METHODS='[\"ccall\",\"cwrap\",\"getValue\",\"setValue\",\"HEAPU8\",\"HEAPU32\"]' \
			--no-entry \
			--strip-all \
			--closure=1"
	@echo "✓ WASM module linked (direct C API)"

# Package new WASM build
wasm-package: wasm-link
	@echo "→ Creating WASM release package..."
	@mkdir -p $(WASM_RELEASE_DIR)/include

	# Copy WASM module files
	@cp $(WASM_BUILD_DIR)/$(WASM_MODULE).wasm $(WASM_RELEASE_DIR)/
	@cp $(WASM_BUILD_DIR)/$(WASM_MODULE).js $(WASM_RELEASE_DIR)/

	# Create version file
	@echo "dlplayer-version=$(WASM_NEW_CRATE_VERSION)-$(COMMIT_HASH)" > $(WASM_RELEASE_DIR)/version.txt
	@echo "api-type=c-api" >> $(WASM_RELEASE_DIR)/version.txt

	@echo "✓ WASM release package created: $(WASM_RELEASE_DIR)/"
	@echo ""
	@echo "Output structure:"
	@echo "  $(WASM_RELEASE_DIR)/"
	@echo "    ├── $(WASM_MODULE).wasm"
	@echo "    ├── $(WASM_MODULE).js"
	@echo "    └── version.txt"
	@echo ""
	@echo "Usage in JavaScript:"
	@echo "  import createModule from './$(WASM_MODULE).js';"
	@echo "  const Module = await createModule();"
	@echo "  const newPlayer = Module.cwrap('dotlottie_new_player', 'number', ['number']);"

# Main WASM build target (C API - direct export)
wasm: wasm-link wasm-package
	@echo "✓ WASM C API build complete"

# Clean new WASM builds
wasm-clean:
	@echo "→ Cleaning WASM C API builds..."
	@cargo clean --manifest-path dotlottie-rs/Cargo.toml --target $(WASM_TARGET)
	@rm -rf $(WASM_BUILD_DIR)
	@rm -rf $(WASM_RELEASE_DIR)
	@echo "✓ WASM C API builds cleaned"
