---
default: minor
---

# feat: wav, flac, ogg and m4a audio

Adds opt-in `audio-wav`, `audio-flac`, `audio-ogg` and `audio-m4a` features alongside the default MP3. Each enables the matching rodio decoder on native targets and the format's blob MIME type on the web. rodio is bumped to 0.21, which decodes `.m4a` files that rodio 0.17 panicked on.
