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
.PHONY: linux linux-x86_64 linux-aarch64 cpp-bindings install-linux-targets linux-env-info linux-help

# Linux help
linux-help:
	@echo "Linux Build Targets:"
	@echo "===================="
	@echo "  make cpp-bindings                                 - Generate C++ UniFFI bindings"
	@echo "  make linux                                        - Build for all Linux architectures"
	@echo "  make linux-x86_64                                - Build for Linux x86_64"
	@echo "  make linux-aarch64                               - Build for Linux aarch64"
	@echo "  make linux-clean                                 - Clean Linux bindings and release artifacts"
	@echo ""
	@echo "Linux Variables:"
	@echo "================"
	@echo "  LINUX_FEATURES - Rust features to enable (default: $(LINUX_FEATURES))"
	@echo ""
	@echo "Linux Examples:"
	@echo "==============="
	@echo "  make cpp-bindings"
	@echo "  make linux-x86_64"
	@echo "  make linux LINUX_FEATURES=thorvg,uniffi"
	@echo ""
	@echo "Prerequisites:"
	@echo "=============="
	@echo "  make install-linux-targets                       - Install all required Rust targets"
	@echo "  uniffi-bindgen-cpp                               - Required for C++ bindings generation"

# Generate C++ UniFFI bindings (following main Makefile pattern)
cpp-bindings:
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
linux: cpp-bindings $(addprefix linux-,x86_64 aarch64)

# Linux x86_64
linux-x86_64: cpp-bindings linux-check-env
	@echo "Building dotlottie-ffi for Linux x86_64..."
	@echo "Target: $(LINUX_TARGET_x86_64)"
	@echo "Features: $(LINUX_FEATURES)"
	cargo build --target $(LINUX_TARGET_x86_64) \
		--no-default-features \
		--features $(LINUX_FEATURES) \
		--release

# Linux aarch64
linux-aarch64: cpp-bindings linux-check-env
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

# Show Linux environment info
linux-env-info: linux-check-env
	@echo "Linux Environment Information:"
	@echo "=============================="
	@echo "Available targets: $(LINUX_TARGETS)"
	@echo "C++ bindings directory: $(CPP_BINDINGS_DIR)"
	@echo "UniFFI bindgen C++: $(UNIFFI_BINDGEN_CPP)"
	@echo "Linux features: $(LINUX_FEATURES)"
	@echo ""
	@echo "Release Information:"
	@echo "==================="
	@echo "CRATE_VERSION: $(CRATE_VERSION)"
	@echo "COMMIT_HASH: $(COMMIT_HASH)"
	@echo ""
	@echo "Rust toolchain info:"
	@echo "===================="
	rustc --version
	cargo --version
	@echo ""
	@echo "Linux Rust targets:"
	@echo "==================="
	@for target in $(LINUX_TARGETS); do \
		if rustup target list --installed | grep -q $$target; then \
			echo "✓ $$target (installed)"; \
		else \
			echo "✗ $$target (not installed - run 'make install-linux-targets')"; \
		fi; \
	done
	@echo ""
	@echo "UniFFI bindgen C++ status:"
	@echo "=========================="
	@if command -v $(UNIFFI_BINDGEN_CPP) >/dev/null 2>&1; then \
		echo "✓ $(UNIFFI_BINDGEN_CPP) found at: $$(which $(UNIFFI_BINDGEN_CPP))"; \
		$(UNIFFI_BINDGEN_CPP) --version 2>/dev/null || echo "Version info not available"; \
	else \
		echo "✗ $(UNIFFI_BINDGEN_CPP) not found in PATH"; \
		echo "Please install uniffi-bindgen-cpp for C++ bindings generation"; \
	fi

# Clean Linux bindings and release artifacts
linux-clean:
	@echo "Cleaning Linux bindings and release artifacts..."
	rm -rf $(CPP_BINDINGS_DIR)
	@echo "Linux artifacts cleaned!"
