EMSDK_VERSION ?= 3.1.74
UNIFFI_BINDGEN_CPP ?= uniffi-bindgen-cpp

# Default Rust features for WASM builds
WASM_FEATURES ?= thorvg,thorvg_webp,thorvg_png,thorvg_jpg,thorvg_ttf,thorvg_lottie_expressions,uniffi

# WASM/Emscripten configuration
EMSDK := emsdk
EMSDK_DIR := deps/$(EMSDK)
EMSDK_ENV := emsdk_env.sh

# WASM module configuration
WASM_MODULE := DotLottiePlayer
WASM_TARGET := wasm32-unknown-emscripten

# UniFFI Bindings
BINDINGS_DIR ?= bindings
CPP_BINDINGS_DIR ?= $(BINDINGS_DIR)/cpp
WASM_BUILD_DIR := build/wasm

# Get version information
CRATE_VERSION := $(shell grep -m 1 version Cargo.toml | sed 's/.*"\([0-9.]\+\)"/\1/')
COMMIT_HASH := $(shell git rev-parse --short HEAD)

# WASM-specific phony targets
.PHONY: wasm install-wasm-targets wasm-install-emsdk wasm-clean



# Initialize emsdk submodule
wasm-init-submodule:
	@echo "Initializing emsdk submodule..."
	@if [ ! -f "$(EMSDK_DIR)/emsdk" ]; then \
		git submodule update --init --recursive $(EMSDK_DIR); \
	fi

# Install and activate specific emsdk version
wasm-install-emsdk: wasm-init-submodule
	@echo "Installing emsdk version $(EMSDK_VERSION)..."
	cd $(EMSDK_DIR) && \
		./emsdk install $(EMSDK_VERSION) && \
		./emsdk activate $(EMSDK_VERSION)
	@echo "emsdk $(EMSDK_VERSION) installed and activated"

# Generate C++ UniFFI bindings for WASM
wasm-cpp-bindings:
	@echo "Generating C++ UniFFI bindings for WASM..."
	@mkdir -p $(CPP_BINDINGS_DIR)
	rm -rf $(CPP_BINDINGS_DIR)/*
	$(UNIFFI_BINDGEN_CPP) \
		--config uniffi.toml \
		--out-dir $(CPP_BINDINGS_DIR) \
		src/dotlottie_player.udl
	@echo "Applying C++ bindings fixes for WASM..."
	@if ls $(CPP_BINDINGS_DIR)/*.hpp >/dev/null 2>&1; then \
		sed -i.bak 's/uint8_t/char/g' $(CPP_BINDINGS_DIR)/*.hpp; \
		rm -f $(CPP_BINDINGS_DIR)/*.bak; \
	fi
	@if ls $(CPP_BINDINGS_DIR)/*.cpp >/dev/null 2>&1; then \
		sed -i.bak 's/uint8_t/char/g' $(CPP_BINDINGS_DIR)/*.cpp; \
		rm -f $(CPP_BINDINGS_DIR)/*.bak; \
	fi
	@if [ -f emscripten_bindings.cpp ]; then \
		cp emscripten_bindings.cpp $(CPP_BINDINGS_DIR)/.; \
	fi
	@echo "C++ bindings for WASM generated in $(CPP_BINDINGS_DIR)"

# Compile WASM C++ sources
wasm-compile-cpp: wasm-cpp-bindings
	@echo "Compiling C++ sources for WASM..."
	@mkdir -p $(WASM_BUILD_DIR)
	bash -c "source $(EMSDK_DIR)/$(EMSDK_ENV) && \
		export CC=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emcc && \
		export CXX=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++ && \
		export AR=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emar && \
		$(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++ \
			-std=c++20 \
			-I$(CPP_BINDINGS_DIR) \
			-Wshift-negative-value \
			-flto \
			-Oz \
			-ffunction-sections \
			-fdata-sections \
			-c emscripten_bindings.cpp \
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

# Build Rust library for WASM target
wasm-build-rust: wasm-check-env wasm-cpp-bindings
	@echo "Building Rust library for WASM target..."
	@echo "Target: $(WASM_TARGET)"
	@echo "Features: $(WASM_FEATURES)"
	@echo "Setting up emscripten toolchain..."
	@echo "CC: $(PWD)/$(EMSDK_DIR)/upstream/emscripten/emcc"
	@echo "CXX: $(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++"
	@echo "AR: $(PWD)/$(EMSDK_DIR)/upstream/emscripten/emar"
	bash -c "source $(EMSDK_DIR)/$(EMSDK_ENV) && \
		export CC=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emcc && \
		export CXX=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++ && \
		export AR=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emar && \
		export RUSTFLAGS='-C link-arg=--no-entry' && \
		echo 'Verifying toolchain:' && \
		echo 'CC=' \$$CC && \
		echo 'CXX=' \$$CXX && \
		echo 'AR=' \$$AR && \
		echo 'RUSTFLAGS=' \$$RUSTFLAGS && \
		cargo +nightly build \
			-Z build-std=std,panic_abort \
			-Z build-std-features=panic_immediate_abort \
			--target $(WASM_TARGET) \
			--no-default-features \
			--features $(WASM_FEATURES) \
			--release"

# Link WASM module
wasm-link-module: wasm-build-rust wasm-compile-cpp
	@echo "Linking WASM module..."
	bash -c "source $(EMSDK_DIR)/$(EMSDK_ENV) && \
		export CC=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emcc && \
		export CXX=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++ && \
		export AR=$(PWD)/$(EMSDK_DIR)/upstream/emscripten/emar && \
		$(PWD)/$(EMSDK_DIR)/upstream/emscripten/em++ \
			-std=c++20 \
			-o $(WASM_BUILD_DIR)/$(WASM_MODULE).js \
			$(WASM_BUILD_DIR)/emscripten_bindings.o \
			$(WASM_BUILD_DIR)/dotlottie_player.o \
			target/$(WASM_TARGET)/release/libdotlottie_player.a \
			-Wl,-u,htons \
			-Wl,-u,ntohs \
			-Wl,-u,htonl \
			-Wshift-negative-value \
			-flto \
			-Oz \
			--bind \
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

# Main WASM build target
wasm: wasm-link-module
	@echo "WASM build complete!"
	@echo "Output files should be in $(WASM_BUILD_DIR)/"
	@if [ -f "$(WASM_BUILD_DIR)/$(WASM_MODULE).wasm" ]; then \
		echo "✓ $(WASM_MODULE).wasm generated"; \
	fi
	@if [ -f "$(WASM_BUILD_DIR)/$(WASM_MODULE).js" ]; then \
		echo "✓ $(WASM_MODULE).js generated"; \
	fi

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

# Install WASM Rust target
install-wasm-targets:
	@echo "Installing Rust nightly toolchain and WASM target..."
	rustup toolchain install nightly
	rustup target add $(WASM_TARGET)
	rustup target add --toolchain nightly $(WASM_TARGET)
	@echo "WASM target and nightly toolchain installed successfully!"



# Clean WASM bindings and build artifacts
wasm-clean:
	@echo "Cleaning WASM bindings and build artifacts..."
	rm -rf $(CPP_BINDINGS_DIR)
	rm -rf $(WASM_BUILD_DIR)
	@echo "WASM artifacts cleaned!"


