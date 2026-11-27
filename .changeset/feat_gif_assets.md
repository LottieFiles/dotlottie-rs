---
default: minor
---

# feat: GIF image assets

Adds a `tvg-gif` feature that enables ThorVG's GIF loader. GIF image assets then render, embedded or external. An animated GIF shows its first frame only.

The feature is off by default. The loader does not limit the image size a GIF declares, so enable it only for trusted assets.
