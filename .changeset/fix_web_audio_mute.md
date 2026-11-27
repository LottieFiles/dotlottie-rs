---
default: patch
---

# fix: mute web audio at volume 0

On iOS Safari, `set_audio_volume(0)` did not silence a playing animation, because script cannot change `HTMLMediaElement.volume` there. A volume of 0 now also sets `muted`.
