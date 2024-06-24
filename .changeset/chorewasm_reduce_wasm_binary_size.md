---
default: minor
---

# chore(wasm): 🤖 reduce WASM binary size

- **WASM Binary Optimization:**

  - Applied the `-Oz` flag with `emcc` for size optimization.
  - Used the compact `emmalloc` allocator.
  - Used the rust nightly toolchain to remove location details and panic string formatting for a smaller binary size.
  - Reduced binary size by ~142 KB (from 1,245,102 bytes to 1,099,243 bytes).

- **JavaScript Glue Optimization:**

  - Enabled the Closure compiler with the `--closure=1` flag.
  - Reduced glue code size by ~36.88 KB (from 67,964 bytes to 30,197 bytes).
