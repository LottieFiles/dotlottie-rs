EMSDK_VERSION ?= 3.1.74
UNIFFI_BINDGEN_CPP ?= uniffi-bindgen-cpp
UNIFFI_BINDGEN_CPP_VERSION ?= v0.7.2+v0.28.3

# Default Rust features for WASM builds
FEATURES ?= tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-lottie-expressions
DEFAULT_FEATURES = tvg-v1,tvg-sw,uniffi

# WASM/Emscripten configuration
EMSDK := emsdk
EMSDK_DIR := deps/modules/$(EMSDK)
EMSDK_ENV := emsdk_env.sh

# WASM module configuration
WASM_MODULE := DotLottiePlayer
WASM_TARGET := wasm32-unknown-emscripten

# UniFFI Bindings
BINDINGS_DIR ?= dotlottie-ffi/uniffi-bindings
CPP_BINDINGS_DIR ?= $(BINDINGS_DIR)/cpp
WASM_BUILD_DIR := dotlottie-ffi/build/wasm

# Get version information
CRATE_VERSION := $(shell grep -m 1 version dotlottie-ffi/Cargo.toml | sed 's/.*"\([0-9.]\+\)"/\1/')
COMMIT_HASH := $(shell git rev-parse --short HEAD)

# Release directories
WASM_RELEASE_DIR ?= release/wasm

# WASM-specific phony targets
.PHONY: wasm wasm-setup wasm-install-emsdk wasm-package wasm-clean


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

# Generate C++ UniFFI bindings for WASM
wasm-cpp-bindings:
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
	@if [ -f dotlottie-ffi/emscripten_bindings.cpp ]; then \
		cp dotlottie-ffi/emscripten_bindings.cpp $(CPP_BINDINGS_DIR)/.; \
	fi
	@echo "✓ C++ bindings generated"

# Compile WASM C++ sources
wasm-compile-cpp: wasm-cpp-bindings
	@echo "→ Compiling C++ sources..."
	@mkdir -p $(WASM_BUILD_DIR)
	@bash -c "source $(EMSDK_DIR)/$(EMSDK_ENV) && \
		export CC=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emcc && \
		export CXX=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++ && \
		export AR=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emar && \
		export RANLIB=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emranlib && \
		$(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++ \
			-std=c++20 \
			-I$(CPP_BINDINGS_DIR) \
			-Wshift-negative-value \
			-flto \
			-Oz \
			-ffunction-sections \
			-fdata-sections \
			-c dotlottie-ffi/emscripten_bindings.cpp \
			-o $(WASM_BUILD_DIR)/emscripten_bindings.o && \
		$(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++ \
			-std=c++20 \
			-I$(CPP_BINDINGS_DIR) \
			-Wshift-negative-value \
			-flto \
			-Oz \
			-ffunction-sections \
			-fdata-sections \
			-c $(CPP_BINDINGS_DIR)/dotlottie_player.cpp \
			-o $(WASM_BUILD_DIR)/dotlottie_player.o"
	@echo "✓ C++ compilation complete"

# Build Rust library for WASM target
wasm-build-rust: wasm-check-env wasm-cpp-bindings
	@echo "→ Building Rust library (nightly)..."
	@bash -c "source $(EMSDK_DIR)/$(EMSDK_ENV)" && \
	CC=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emcc \
	CXX=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++ \
	AR=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emar \
	RANLIB=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emranlib \
	CLANG_PATH=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emcc \
	CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_LINKER=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emcc \
	BINDGEN_EXTRA_CLANG_ARGS="-isysroot $(PWD)/$(EMSDK_DIR)/upstream/emscripten/cache/sysroot" \
	RUSTFLAGS='-C link-arg=--no-entry' \
	cargo +nightly build \
		--manifest-path dotlottie-ffi/Cargo.toml \
		-Z build-std=std,panic_abort \
		-Z build-std-features=panic_immediate_abort \
		--target $(WASM_TARGET) \
		--no-default-features \
		--features $(DEFAULT_FEATURES),$(FEATURES) \
		--release
	@echo "✓ Rust build complete"

# Install npm dependencies for TypeScript support
wasm-install-npm-deps:
	@echo "→ Installing npm dependencies for TypeScript support..."
	@bash -c "source $(EMSDK_DIR)/$(EMSDK_ENV) && \
		cd $(PWD)/$(EMSDK_DIR)/upstream/emscripten && \
		if [ ! -d node_modules ] || [ ! -f node_modules/.bin/tsc ]; then \
			npm install >/dev/null 2>&1; \
		fi"
	@echo "✓ npm dependencies installed"

# Link WASM module
wasm-link-module: wasm-build-rust wasm-compile-cpp wasm-install-npm-deps
	@echo "→ Linking WASM module..."
	@bash -c "source $(EMSDK_DIR)/$(EMSDK_ENV) && \
		export CC=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emcc && \
		export CXX=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++ && \
		export AR=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emar && \
		export RANLIB=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emranlib && \
		$(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++ \
			-std=c++20 \
			-o $(PWD)/$(WASM_BUILD_DIR)/$(WASM_MODULE).js \
			$(PWD)/$(WASM_BUILD_DIR)/emscripten_bindings.o \
			$(PWD)/$(WASM_BUILD_DIR)/dotlottie_player.o \
			$(PWD)/dotlottie-ffi/target/$(WASM_TARGET)/release/libdotlottie_player.a \
			-Wl,-u,htons \
			-Wl,-u,ntohs \
			-Wl,-u,htonl \
			-Wshift-negative-value \
			-flto \
			-Oz \
			--bind \
			--emit-tsd $(PWD)/$(WASM_BUILD_DIR)/$(WASM_MODULE).d.ts \
			-sWASM=1 \
			-sALLOW_MEMORY_GROWTH=1 \
			-sFORCE_FILESYSTEM=0 \
			-sMODULARIZE=1 \
			-sEXPORT_NAME=create$(WASM_MODULE)Module \
			-sEXPORT_ES6=1 \
			-sUSE_ES6_IMPORT_META=0 \
			-sDYNAMIC_EXECUTION=0 \
			-sENVIRONMENT=web \
			-sFILESYSTEM=0 \
			--no-entry \
			--strip-all \
			--closure=1"
	@echo "✓ WASM module linked"

# Main WASM build target
wasm: wasm-link-module wasm-package
	@echo "✓ WASM build and packaging complete"

# Package WASM build
wasm-package: wasm-link-module
	@echo "→ Creating WASM release package..."
	@mkdir -p $(WASM_RELEASE_DIR)
	
	# Copy WASM module files
	@if [ -f "$(WASM_BUILD_DIR)/$(WASM_MODULE).wasm" ]; then \
		cp $(WASM_BUILD_DIR)/$(WASM_MODULE).wasm $(WASM_RELEASE_DIR)/; \
	fi
	@if [ -f "$(WASM_BUILD_DIR)/$(WASM_MODULE).js" ]; then \
		cp $(WASM_BUILD_DIR)/$(WASM_MODULE).js $(WASM_RELEASE_DIR)/; \
	fi
	@if [ -f "$(WASM_BUILD_DIR)/$(WASM_MODULE).d.ts" ]; then \
		cp $(WASM_BUILD_DIR)/$(WASM_MODULE).d.ts $(WASM_RELEASE_DIR)/; \
	fi
	
	# Create version file
	@echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(WASM_RELEASE_DIR)/version.txt
	
	@echo "✓ WASM release package created: $(WASM_RELEASE_DIR)/"

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
	@rustup toolchain install nightly >/dev/null
	@rustup component add rust-src --toolchain nightly >/dev/null
	@rustup target add --toolchain nightly $(WASM_TARGET) >/dev/null
	@echo "✓ WASM targets and nightly toolchain installed"
	@echo "→ Installing uniffi-bindgen-cpp..."
	@cargo install uniffi-bindgen-cpp --git https://github.com/NordSecurity/uniffi-bindgen-cpp --tag $(UNIFFI_BINDGEN_CPP_VERSION) >/dev/null
	@echo "✓ uniffi-bindgen-cpp installed"

# Clean WASM bindings and build artifacts
wasm-clean:
	@echo "→ Cleaning WASM builds..."
	cargo clean --manifest-path dotlottie-ffi/Cargo.toml
	@rm -rf $(CPP_BINDINGS_DIR)
	@rm -rf $(WASM_BUILD_DIR)
	@rm -rf $(WASM_RELEASE_DIR)
	@echo "✓ WASM builds cleaned"

