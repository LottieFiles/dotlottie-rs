---
default: patch
---

# fix: audio volume uses the left channel

Static audio levels such as `[100, 0]` played silent because ThorVG read the right channel. ThorVG now reads the left one, like lottie-web.
