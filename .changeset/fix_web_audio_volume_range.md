---
default: patch
---

# fix: web audio levels outside 0–100% no longer break the player

An audio layer above 100% or below 0 made `HTMLMediaElement.volume` throw, which left the player unusable. The web backend now clamps the level to [0, 1].
