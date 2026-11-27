---
default: patch
---

# chore: Rust 1.89 and dependency upgrades

The minimum supported Rust version is now 1.89. rodio moves to 0.23, miniz_oxide to 0.9 and wasm-bindgen to 0.2.130. `cbindgen` is only built when the `c_api` feature is on.
