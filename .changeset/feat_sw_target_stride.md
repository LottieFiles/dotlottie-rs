---
default: minor
---

# feat: stride for the software target

`Player::set_sw_target` and `dotlottie_set_sw_target` take a `stride` argument before `width`. Pass the width to keep the old behaviour. A larger stride targets a region of a wider pixel buffer, so several players can share one buffer.

This changes both signatures: existing callers must add the argument.
