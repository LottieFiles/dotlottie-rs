UNIFFI_BINDGEN_CPP ?= uniffi-bindgen-cpp

# Default Rust features for Linux builds
LINUX_FEATURES ?= uniffi,thorvg,thorvg_webp,thorvg_png,thorvg_jpg,thorvg_ttf,thorvg_lottie_expressions

# UniFFI Bindings
BINDINGS_DIR ?= bindings
CPP_BINDINGS_DIR ?= $(BINDINGS_DIR)/cpp

# Linux targets
LINUX_TARGETS = x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

# Linux target mapping
LINUX_TARGET_x86_64 = x86_64-unknown-linux-gnu
LINUX_TARGET_aarch64 = aarch64-unknown-linux-gnu

# Get version information
CRATE_VERSION := $(shell grep -m 1 version Cargo.toml | sed 's/.*"\([0-9.]\+\)"/\1/')
COMMIT_HASH := $(shell git rev-parse --short HEAD)

# Linux-specific phony targets
.PHONY: linux linux-x86_64 linux-aarch64 install-linux-targets linux-clean



# Generate C++ UniFFI bindings for Linux
linux-cpp-bindings:
	@echo "Generating C++ UniFFI bindings..."
	@mkdir -p $(CPP_BINDINGS_DIR)
	rm -rf $(CPP_BINDINGS_DIR)/*
	$(UNIFFI_BINDGEN_CPP) \
		--config uniffi.toml \
		--out-dir $(CPP_BINDINGS_DIR) \
		src/dotlottie_player.udl
	@echo "Applying C++ bindings fixes..."
	@if ls $(CPP_BINDINGS_DIR)/*.hpp >/dev/null 2>&1; then \
		sed -i.bak 's/uint8_t/char/g' $(CPP_BINDINGS_DIR)/*.hpp; \
		rm -f $(CPP_BINDINGS_DIR)/*.bak; \
	fi
	@if ls $(CPP_BINDINGS_DIR)/*.cpp >/dev/null 2>&1; then \
		sed -i.bak 's/uint8_t/char/g' $(CPP_BINDINGS_DIR)/*.cpp; \
		rm -f $(CPP_BINDINGS_DIR)/*.bak; \
	fi
	@echo "C++ bindings generated in $(CPP_BINDINGS_DIR)"



# Build for all Linux architectures
linux: linux-cpp-bindings $(addprefix linux-,x86_64 aarch64)

# Linux x86_64
linux-x86_64: linux-cpp-bindings linux-check-env
	@echo "Building dotlottie-ffi for Linux x86_64..."
	@echo "Target: $(LINUX_TARGET_x86_64)"
	@echo "Features: $(LINUX_FEATURES)"
	cargo build --target $(LINUX_TARGET_x86_64) \
		--no-default-features \
		--features $(LINUX_FEATURES) \
		--release

# Linux aarch64
linux-aarch64: linux-cpp-bindings linux-check-env
	@echo "Building dotlottie-ffi for Linux aarch64..."
	@echo "Target: $(LINUX_TARGET_aarch64)"
	@echo "Features: $(LINUX_FEATURES)"
	cargo build --target $(LINUX_TARGET_aarch64) \
		--no-default-features \
		--features $(LINUX_FEATURES) \
		--release

# Check Linux build environment
linux-check-env:
	@echo "Checking Linux build environment..."
	@if ! command -v $(UNIFFI_BINDGEN_CPP) >/dev/null 2>&1; then \
		echo "Warning: $(UNIFFI_BINDGEN_CPP) not found in PATH"; \
		echo "C++ bindings generation may fail"; \
		echo "Please install uniffi-bindgen-cpp or ensure it's in PATH"; \
	fi
	@if ! command -v cargo >/dev/null 2>&1; then \
		echo "Error: cargo not found"; \
		exit 1; \
	fi

# Install Linux targets if not already installed
install-linux-targets:
	@echo "Installing Linux Rust targets..."
	rustup target add $(LINUX_TARGETS)
	@echo "Linux targets installed successfully!"



# Clean Linux bindings and release artifacts
linux-clean:
	@echo "Cleaning Linux bindings and release artifacts..."
	rm -rf $(CPP_BINDINGS_DIR)
	@echo "Linux artifacts cleaned!"
