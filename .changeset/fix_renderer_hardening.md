---
default: patch
---

# fix: renderer crashes and stalled rendering

- `set_sw_target` rejects a buffer that is too small or a size that overflows, and `dotlottie_set_sw_target` returns `InvalidParameter` for a NULL buffer.
- A slot or theme rule whose id is not in the animation is ignored. Before, every later render failed.
