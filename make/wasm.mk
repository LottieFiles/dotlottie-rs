WASM_TARGET  := wasm32-unknown-unknown
WASM_FEATURES_COMMON := tvg,tvg-sw,tvg-png,tvg-jpg,tvg-webp,tvg-ttf,tvg-lottie-expressions,dotlottie,theming,state-machines,wasm-bindgen-api
WASM_MANIFEST := dotlottie-rs/Cargo.toml
WASM_ARTIFACT := dotlottie-rs/target/$(WASM_TARGET)/release/dotlottie_rs.wasm

LLVM_PREFIX := $(shell brew --prefix llvm 2>/dev/null)
ifneq ($(LLVM_PREFIX),)
  WASM_CC  := $(LLVM_PREFIX)/bin/clang
  WASM_CXX := $(LLVM_PREFIX)/bin/clang++
else
  WASM_CC  := clang
  WASM_CXX := clang++
endif

WASM_LOCKFILE := dotlottie-rs/Cargo.lock
WASM_BINDGEN_VERSION := $(shell grep -A1 'name = "wasm-bindgen"' $(WASM_LOCKFILE) 2>/dev/null | grep version | head -1 | sed 's/.*"\(.*\)"/\1/')

.PHONY: wasm-setup wasm wasm-webgl wasm-webgpu wasm-all wasm-clean

wasm-setup:
	@rustup target add $(WASM_TARGET)
	@if [ ! -f $(WASM_LOCKFILE) ]; then cargo generate-lockfile --manifest-path $(WASM_MANIFEST); fi
	$(eval WASM_BINDGEN_VERSION := $(shell grep -A1 'name = "wasm-bindgen"' $(WASM_LOCKFILE) | grep version | head -1 | sed 's/.*"\(.*\)"/\1/'))
	@cargo install wasm-bindgen-cli --version $(WASM_BINDGEN_VERSION) --locked

define wasm_build
	@mkdir -p $(3)
	CC=$(WASM_CC) CXX=$(WASM_CXX) RUSTFLAGS="$(2)" \
		cargo rustc \
			--manifest-path $(WASM_MANIFEST) \
			--crate-type cdylib \
			--target $(WASM_TARGET) \
			--release \
			--no-default-features \
			--features $(WASM_FEATURES_COMMON)$(if $(1),$(comma)$(1))
	wasm-bindgen $(WASM_ARTIFACT) \
		--out-dir $(3) \
		--target web \
		--typescript
	@if command -v wasm-opt >/dev/null 2>&1; then \
		wasm-opt $(3)/dotlottie_rs_bg.wasm -o $(3)/dotlottie_rs_bg.wasm -O; \
	fi
endef

comma := ,

wasm:
	$(call wasm_build,,,release/wasm)

wasm-webgl:
	$(call wasm_build,tvg-gl$(comma)webgl,,release/wasm-webgl)

wasm-webgpu:
	$(call wasm_build,tvg-wg$(comma)webgpu,--cfg=web_sys_unstable_apis,release/wasm-webgpu)

wasm-all: wasm wasm-webgl wasm-webgpu

wasm-clean:
	@cargo clean --manifest-path $(WASM_MANIFEST) --target $(WASM_TARGET)
	@rm -rf release/wasm release/wasm-webgl release/wasm-webgpu
