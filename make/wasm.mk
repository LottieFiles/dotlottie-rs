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
WASM_BINDGEN_COMMON := tvg,tvg-sw,tvg-png,tvg-jpg,tvg-ttf,dotlottie,theming,state-machines,wasm,wasm-bindgen-api

# sed -i behaves differently on macOS (BSD) vs Linux (GNU)
ifeq ($(shell uname),Darwin)
  SED_I := sed -i ''
else
  SED_I := sed -i
endif

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

# Post-process wasm-bindgen output to fix two issues:
# 1. wasm-bindgen >=0.2 generates `import * as __wbg_star0 from 'env'` unconditionally
#    even when the wasm binary has no env imports.  Browsers reject the bare 'env'
#    specifier, so strip the two dead lines.
# 2. The default module path fallback `new URL('...wasm', import.meta.url)` causes
#    Webpack/Next.js to try resolving the .wasm file at build time, breaking bundled
#    consumers. Replace with a throw since DotLottieWasmLoader always provides a URL.
define strip_env_import
	$(SED_I) \
		-e '/^import \* as __wbg_star0 from .env.;/d' \
		-e '/imports\[.env.\] = __wbg_star0;/d' \
		-e "s|module_or_path = new URL('dotlottie_rs_bg.wasm', import.meta.url);|throw new Error('WASM module URL must be provided via DotLottieWasmLoader or setWasmUrl().');|" \
		$(1)/dotlottie_rs.js
endef

# Software-renderer build — no graphics API required
wasm:
	CC=$(WASM_CC) CXX=$(WASM_CXX) \
		wasm-pack build dotlottie-rs --target web \
		--out-dir ../release/wasm \
		--no-default-features --features $(WASM_BINDGEN_COMMON)
	$(call strip_env_import,release/wasm)

# WebGL2 build
wasm-webgl:
	CC=$(WASM_CC) CXX=$(WASM_CXX) \
		wasm-pack build dotlottie-rs --target web \
		--out-dir ../release/wasm-webgl \
		--no-default-features --features $(WASM_BINDGEN_COMMON),tvg-gl,webgl
	$(call strip_env_import,release/wasm-webgl)

# WebGPU build — requires web_sys_unstable_apis cfg for all Gpu* web-sys types
wasm-webgpu:
	CC=$(WASM_CC) CXX=$(WASM_CXX) \
		RUSTFLAGS="--cfg=web_sys_unstable_apis" \
		wasm-pack build dotlottie-rs --target web \
		--out-dir ../release/wasm-webgpu \
		--no-default-features --features $(WASM_BINDGEN_COMMON),tvg-wg,webgpu
	$(call strip_env_import,release/wasm-webgpu)

# Build all three variants
wasm-all: wasm wasm-webgl wasm-webgpu

# Remove wasm artefacts
wasm-clean:
	@cargo clean --manifest-path dotlottie-rs/Cargo.toml --target $(WASM_BINDGEN_TARGET)
	@rm -rf release/wasm release/wasm-webgl release/wasm-webgpu
