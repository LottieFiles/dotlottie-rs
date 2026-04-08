# Helper: comma literal for use inside $(call) arguments
comma := ,

# Default Rust features for Windows builds
WINDOWS_FEATURES ?= tvg-webp,tvg-png,tvg-jpg,tvg-ttf,tvg-lottie-expressions,tvg-threads
WINDOWS_DEFAULT_FEATURES = tvg,tvg-sw,c_api,dotlottie,state-machines,theming

ifdef FEATURES
	WINDOWS_FEATURES = $(FEATURES)
endif

# Release and packaging variables
RELEASE_DIR ?= release
WINDOWS_RELEASE_DIR ?= $(RELEASE_DIR)/windows
DOTLOTTIE_PLAYER_DIR ?= dotlottie-player

# Library names (MSVC toolchain output)
WINDOWS_FFI_LIB_BASE ?= dotlottie_rs
WINDOWS_STATIC_LIB := $(WINDOWS_FFI_LIB_BASE).lib
WINDOWS_SHARED_LIB := $(WINDOWS_FFI_LIB_BASE).dll
WINDOWS_IMPORT_LIB := $(WINDOWS_FFI_LIB_BASE).dll.lib
WINDOWS_HEADER_FILE := dotlottie_player.h
WINDOWS_HEADER_DIR := dotlottie-rs/build

# Get version information
CRATE_VERSION ?= $(shell grep -m 1 'version =' dotlottie-rs/Cargo.toml | grep -o '[0-9][0-9.]*')
COMMIT_HASH ?= $(shell git rev-parse --short HEAD)

# Windows target mapping
WINDOWS_TARGET_x86_64 = x86_64-pc-windows-msvc
WINDOWS_TARGET_arm64 = aarch64-pc-windows-msvc

# Windows targets list
WINDOWS_TARGETS = x86_64-pc-windows-msvc aarch64-pc-windows-msvc

# Function to check platform support - only called when Windows targets are invoked
define check_windows_platform_support
$(if $(filter Windows_NT,$(OS)),,$(error "Windows builds require a Windows host system."))
endef

# Build using MSVC environment.
# Detects VS via vswhere, then runs cargo inside a vcvarsall.bat session using a temp .bat file.
# This avoids PATH corruption from mixing Windows semicolons with Git Bash colons.
# Args: $(1) = vcvars arch (x64, arm64), $(2) = Rust target triple,
#        $(3) = cargo features, $(4) = extra cargo flags (optional)
define WINDOWS_CARGO_BUILD
	@echo "-> Configuring MSVC environment for $(1)..."; \
	VSWHERE="/c/Program Files (x86)/Microsoft Visual Studio/Installer/vswhere.exe"; \
	if [ ! -f "$$VSWHERE" ]; then \
		echo "Error: vswhere.exe not found. Please install Visual Studio Build Tools."; \
		echo "  Run: make windows-setup"; \
		exit 1; \
	fi; \
	VS_PATH=$$("$$VSWHERE" -products '*' -latest -property installationPath | tr -d '\r'); \
	if [ -z "$$VS_PATH" ]; then \
		echo "Error: No Visual Studio installation found."; \
		exit 1; \
	fi; \
	VCVARS="$$VS_PATH/VC/Auxiliary/Build/vcvarsall.bat"; \
	if [ ! -f "$$VCVARS" ]; then \
		echo "Error: vcvarsall.bat not found at $$VCVARS"; \
		echo "  Install the 'Desktop development with C++' workload in VS Installer."; \
		exit 1; \
	fi; \
	echo "-> Using VS installation: $$VS_PATH"; \
	VCVARS_WIN=$$(cygpath -w "$$VCVARS"); \
	CARGO_WIN=$$(cygpath -w "$$(which cargo)"); \
	MANIFEST_WIN=$$(cygpath -w "$$(pwd)/dotlottie-rs/Cargo.toml"); \
	TMPBAT=$$(mktemp /tmp/vcbuild_XXXXXX.bat); \
	TMPBAT_WIN=$$(cygpath -w "$$TMPBAT"); \
	printf '@echo off\ncall "%s" %s\nif errorlevel 1 exit /b 1\nif exist "C:\\Program Files\\LLVM\\bin\\libclang.dll" set LIBCLANG_PATH=C:\\Program Files\\LLVM\\bin\n"%s" build --manifest-path "%s" --target %s --release %s --features %s\n' \
		"$$VCVARS_WIN" "$(1)" "$$CARGO_WIN" "$$MANIFEST_WIN" "$(2)" \
		"$(4)" "$(3)" > "$$TMPBAT"; \
	echo "-> Building $(2) with cargo..."; \
	cmd //C "$$TMPBAT_WIN"; \
	BUILD_EXIT=$$?; \
	rm -f "$$TMPBAT"; \
	if [ $$BUILD_EXIT -ne 0 ]; then \
		exit $$BUILD_EXIT; \
	fi
endef

# Function to package Windows architecture builds
# Args: $(1) = architecture (x86_64, arm64)
define WINDOWS_PACKAGE_ARCH
	@echo "-> Packaging Windows $(1)..."
	@mkdir -p $(WINDOWS_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/include
	@mkdir -p $(WINDOWS_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/lib

	# Copy header file
	@if [ -f "$(WINDOWS_HEADER_DIR)/$(WINDOWS_HEADER_FILE)" ]; then \
		cp $(WINDOWS_HEADER_DIR)/$(WINDOWS_HEADER_FILE) $(WINDOWS_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/include/$(WINDOWS_HEADER_FILE); \
	else \
		echo "Error: $(WINDOWS_HEADER_FILE) not found in $(WINDOWS_HEADER_DIR)"; \
		exit 1; \
	fi

	# Copy static library
	@if [ -f "dotlottie-rs/target/$(WINDOWS_TARGET_$(1))/release/$(WINDOWS_STATIC_LIB)" ]; then \
		cp dotlottie-rs/target/$(WINDOWS_TARGET_$(1))/release/$(WINDOWS_STATIC_LIB) \
		   $(WINDOWS_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/lib/; \
	else \
		echo "Warning: $(WINDOWS_STATIC_LIB) not found for $(1)"; \
	fi

	# Copy shared library (DLL)
	@if [ -f "dotlottie-rs/target/$(WINDOWS_TARGET_$(1))/release/$(WINDOWS_SHARED_LIB)" ]; then \
		cp dotlottie-rs/target/$(WINDOWS_TARGET_$(1))/release/$(WINDOWS_SHARED_LIB) \
		   $(WINDOWS_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/lib/; \
	else \
		echo "Warning: $(WINDOWS_SHARED_LIB) not found for $(1)"; \
	fi

	# Copy import library (.dll.lib)
	@if [ -f "dotlottie-rs/target/$(WINDOWS_TARGET_$(1))/release/$(WINDOWS_IMPORT_LIB)" ]; then \
		cp dotlottie-rs/target/$(WINDOWS_TARGET_$(1))/release/$(WINDOWS_IMPORT_LIB) \
		   $(WINDOWS_RELEASE_DIR)/$(1)/$(DOTLOTTIE_PLAYER_DIR)/lib/; \
	else \
		echo "Warning: $(WINDOWS_IMPORT_LIB) not found for $(1)"; \
	fi

	# Create version file
	@echo "dlplayer-version=$(CRATE_VERSION)-$(COMMIT_HASH)" > $(WINDOWS_RELEASE_DIR)/$(1)/version.txt

	@echo "Done: Windows $(1) packaging complete"
endef

# Windows-specific phony targets
.PHONY: windows windows-x86_64 windows-arm64 windows-setup windows-clean windows-check-deps

# Build all Windows targets
windows: $(addprefix windows-,x86_64 arm64)
	@echo "Done: All Windows builds complete"

# Build for Windows x86_64
windows-x86_64: windows-check-deps
	$(call check_windows_platform_support)
	@echo "-> Building Windows x86_64..."
	$(call WINDOWS_CARGO_BUILD,x64,$(WINDOWS_TARGET_x86_64),$(WINDOWS_DEFAULT_FEATURES)$(comma)$(WINDOWS_FEATURES),--no-default-features)
	@$(call WINDOWS_PACKAGE_ARCH,x86_64)
	@echo "Done: Windows x86_64 build complete"

# Build for Windows ARM64
windows-arm64: windows-check-deps
	$(call check_windows_platform_support)
	@echo "-> Building Windows ARM64..."
	$(call WINDOWS_CARGO_BUILD,arm64,$(WINDOWS_TARGET_arm64),$(WINDOWS_DEFAULT_FEATURES)$(comma)$(WINDOWS_FEATURES),--no-default-features)
	@$(call WINDOWS_PACKAGE_ARCH,arm64)
	@echo "Done: Windows ARM64 build complete"

# Check for required dependencies
windows-check-deps:
	@echo "-> Checking Windows build dependencies..."
	@if ! command -v cargo >/dev/null 2>&1; then \
		echo "Error: cargo not found. Please install Rust via rustup."; \
		exit 1; \
	fi
	@echo "Done: Build dependencies check complete"

# Install Windows targets and dependencies
windows-setup:
	@echo "-> Setting up Windows build environment..."
	@# Install Rust targets
	@rustup target add $(WINDOWS_TARGETS) 2>/dev/null || true
	@echo "Done: Windows Rust targets installed"
	@echo ""
	@# Check for Visual Studio Build Tools and install via winget if missing
	@VSWHERE="/c/Program Files (x86)/Microsoft Visual Studio/Installer/vswhere.exe"; \
	if [ -f "$$VSWHERE" ] && "$$VSWHERE" -products '*' -latest -property installationPath >/dev/null 2>&1; then \
		VS_PATH=$$("$$VSWHERE" -products '*' -latest -property installationPath | tr -d '\r'); \
		echo "Done: Visual Studio found at $$VS_PATH"; \
	else \
		echo "Visual Studio Build Tools not found."; \
		if command -v winget >/dev/null 2>&1; then \
			echo "-> Installing Visual Studio Build Tools via winget..."; \
			echo "   This will install the 'Desktop development with C++' workload."; \
			winget install Microsoft.VisualStudio.2022.BuildTools \
				--override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended" \
				--accept-source-agreements --accept-package-agreements; \
			if [ $$? -eq 0 ]; then \
				echo "Done: Visual Studio Build Tools installed successfully."; \
				echo "      You may need to restart your terminal for PATH changes to take effect."; \
			else \
				echo "Warning: winget install failed. Please install manually:"; \
				echo "  https://visualstudio.microsoft.com/visual-cpp-build-tools/"; \
			fi; \
		else \
			echo "winget not found. Please install Visual Studio Build Tools manually:"; \
			echo "  https://visualstudio.microsoft.com/visual-cpp-build-tools/"; \
			echo "  Select the 'Desktop development with C++' workload."; \
		fi; \
	fi
	@# Check for LLVM (needed by bindgen for generating Rust bindings)
	@if [ -f "/c/Program Files/LLVM/bin/libclang.dll" ]; then \
		echo "Done: LLVM found at C:\\Program Files\\LLVM"; \
	else \
		echo "LLVM not found (needed by bindgen for Rust binding generation)."; \
		if command -v winget >/dev/null 2>&1; then \
			echo "-> Installing LLVM via winget..."; \
			winget install LLVM.LLVM \
				--accept-source-agreements --accept-package-agreements; \
			if [ $$? -eq 0 ]; then \
				echo "Done: LLVM installed successfully."; \
			else \
				echo "Warning: LLVM install failed. Please install manually from:"; \
				echo "  https://github.com/llvm/llvm-project/releases"; \
			fi; \
		else \
			echo "Please install LLVM manually from:"; \
			echo "  https://github.com/llvm/llvm-project/releases"; \
		fi; \
	fi
	@echo ""
	@echo "Setup complete. Additional notes:"
	@echo "  - For ARM64 cross-compilation: install ARM64 build tools via VS Installer"
	@echo "  - GNU Make is required (via scoop: scoop install make, or choco: choco install make)"

# Clean Windows builds
windows-clean:
	@echo "-> Cleaning Windows builds..."
	@rm -rf $(WINDOWS_RELEASE_DIR)
	@for target in $(WINDOWS_TARGETS); do \
		if [ -d "dotlottie-rs/target/$$target" ]; then \
			rm -rf dotlottie-rs/target/$$target/release; \
		fi; \
	done
	@echo "Done: Windows builds cleaned"
