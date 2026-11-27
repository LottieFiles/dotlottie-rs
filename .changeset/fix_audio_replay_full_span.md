---
default: patch
---

# fix: replay full-span audio on loop, seek and stop

An audio layer that spans the whole animation played once and then stayed silent on later loops, after `stop()` then `play()`, and after seeking back to the start.
