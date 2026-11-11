---
default: patch
---

# fix: regenerate the C header when the C API changes

`build/dotlottie_player.h` was only regenerated when ThorVG sources changed, so an incremental build after a C API change kept the old header.
