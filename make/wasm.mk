# ============================================================================
# WASM Build System — wasm32-unknown-unknown + wasm-pack
# ============================================================================
# Prerequisites:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-pack  (or: make wasm-setup)
#
# Usage:
#   make wasm-setup     — install Rust target + wasm-pack
#   make wasm           — software-renderer build  → release/wasm/
#   make wasm-webgl     — WebGL2 build             → release/wasm-webgl/
#   make wasm-webgpu    — WebGPU build             → release/wasm-webgpu/
#   make wasm-all       — all three variants
#   make wasm-clean     — remove build artefacts
# ============================================================================

WASM_BINDGEN_TARGET := wasm32-unknown-unknown
WASM_BINDGEN_COMMON := tvg,tvg-sw,tvg-png,tvg-jpg,tvg-webp,tvg-ttf,tvg-lottie-expressions,dotlottie,theming,state-machines,wasm,wasm-bindgen-api

# Apple's system clang lacks the WebAssembly backend.  Use Homebrew LLVM if
# present, otherwise fall back to whatever clang/clang++ is on PATH.
LLVM_PREFIX := $(shell brew --prefix llvm 2>/dev/null)
ifneq ($(LLVM_PREFIX),)
  WASM_CC  := $(LLVM_PREFIX)/bin/clang
  WASM_CXX := $(LLVM_PREFIX)/bin/clang++
else
  WASM_CC  := clang
  WASM_CXX := clang++
endif

.PHONY: wasm-setup wasm wasm-webgl wasm-webgpu wasm-all wasm-clean

# Install the wasm32-unknown-unknown Rust target and wasm-pack
wasm-setup:
	@rustup target add $(WASM_BINDGEN_TARGET)
	@command -v wasm-pack >/dev/null 2>&1 || curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Software-renderer build — no graphics API required
wasm:
	CC=$(WASM_CC) CXX=$(WASM_CXX) \
		wasm-pack build dotlottie-rs --target web \
		--out-dir ../release/wasm \
		--no-default-features --features $(WASM_BINDGEN_COMMON)

# WebGL2 build
wasm-webgl:
	CC=$(WASM_CC) CXX=$(WASM_CXX) \
		wasm-pack build dotlottie-rs --target web \
		--out-dir ../release/wasm-webgl \
		--no-default-features --features $(WASM_BINDGEN_COMMON),tvg-gl,webgl

# WebGPU build — requires web_sys_unstable_apis cfg for all Gpu* web-sys types
wasm-webgpu:
	CC=$(WASM_CC) CXX=$(WASM_CXX) \
		RUSTFLAGS="--cfg=web_sys_unstable_apis" \
		wasm-pack build dotlottie-rs --target web \
		--out-dir ../release/wasm-webgpu \
		--no-default-features --features $(WASM_BINDGEN_COMMON),tvg-wg,webgpu

# Build all three variants
wasm-all: wasm wasm-webgl wasm-webgpu

# Remove wasm artefacts
wasm-clean:
	@cargo clean --manifest-path dotlottie-rs/Cargo.toml --target $(WASM_BINDGEN_TARGET)
	@rm -rf release/wasm release/wasm-webgl release/wasm-webgpu
