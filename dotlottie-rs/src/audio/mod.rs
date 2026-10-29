use rustc_hash::FxHashMap;
use std::sync::Arc;

use crate::renderer::{AudioEvent, AudioSource};

#[cfg(all(feature = "audio", not(target_arch = "wasm32")))]
mod rodio_player;
#[cfg(all(feature = "audio", not(target_arch = "wasm32")))]
pub use rodio_player::RodioPlayer;

#[cfg(all(feature = "audio", target_arch = "wasm32"))]
mod web_audio_player;
#[cfg(all(feature = "audio", target_arch = "wasm32"))]
use web_audio_player::WebAudioPlayer;

/// An audio layer within its playback range; retained while paused so it can be
/// replayed on resume.
struct ActiveLayer {
    data: Arc<[u8]>,
    /// Seek position into the clip in seconds.
    offset: f32,
    /// Animation-time second at which the clip's in-point occurs; anchors
    /// offset recomputation when the playhead jumps.
    clip_start: f32,
    /// Per-layer volume in [0.0, 1.0].
    volume: f32,
    /// Whether the clip has been handed to the player for this run.
    started: bool,
}

/// Resolves an external audio `src` (normalized packaged path or URL) to bytes.
pub type AudioSourceLookup = Box<dyn Fn(&str) -> Option<Arc<[u8]>>>;

/// Frame-synchronised audio playback driven by the renderer's audio resolver.
/// Uses rodio on native targets and `HtmlAudioElement` on wasm.
pub struct AudioManager {
    /// Resolves external audio srcs; embedded audio bytes arrive in the event.
    lookup: AudioSourceLookup,
    active: FxHashMap<String, ActiveLayer>,
    playing: bool,
    /// Playhead position in animation-time seconds.
    playhead: f32,
    rate: f32,
    /// False while the playhead runs backward; audio is suspended then.
    forward: bool,
    #[cfg(not(target_arch = "wasm32"))]
    player: RodioPlayer,
    #[cfg(target_arch = "wasm32")]
    player: WebAudioPlayer,
}

impl AudioManager {
    /// The audio device opens lazily on first playback.
    pub fn new(lookup: AudioSourceLookup) -> Self {
        AudioManager {
            lookup,
            active: FxHashMap::default(),
            playing: false,
            playhead: 0.0,
            rate: 1.0,
            forward: true,
            #[cfg(not(target_arch = "wasm32"))]
            player: RodioPlayer::new(),
            #[cfg(target_arch = "wasm32")]
            player: WebAudioPlayer::new(),
        }
    }

    /// Update the playhead position (animation-time seconds). Must be called
    /// before the update that fires resolver events, so activations anchor
    /// correctly.
    pub fn set_playhead(&mut self, secs: f32) {
        self.playhead = secs;
    }

    /// Handle an audio layer state change reported by the renderer.
    pub fn on_audio(&mut self, event: AudioEvent) {
        let key = match &event.source {
            AudioSource::External(src) => normalize_src(src),
            // Key embedded layers by their (stable) source pointer.
            AudioSource::Embedded { bytes, .. } => format!("emb:{:p}", bytes.as_ptr()),
        };

        if !event.active {
            self.active.remove(&key);
            self.player.stop(&key);
            return;
        }

        let volume = event.volume / 100.0;
        let clip_start = self.playhead - event.offset;

        // Volume keyframes re-fire the resolver; update without restarting.
        if let Some(layer) = self.active.get_mut(&key) {
            layer.volume = volume;
            layer.offset = event.offset;
            layer.clip_start = clip_start;
            self.player.set_volume(&key, volume);
            return;
        }

        let data = match event.source {
            AudioSource::External(_) => match (self.lookup)(&key) {
                Some(data) => data,
                None => return,
            },
            AudioSource::Embedded { bytes, .. } => Arc::from(bytes),
        };

        let mut layer = ActiveLayer {
            data,
            offset: event.offset,
            clip_start,
            volume,
            started: false,
        };
        if self.playing && self.forward {
            self.player
                .play(&key, layer.data.clone(), layer.offset, layer.volume);
            layer.started = true;
        }
        self.active.insert(key, layer);
    }

    /// Reflect the player's playback state, resuming/pausing sinks and starting
    /// any layers that became active while paused.
    pub fn set_playing(&mut self, playing: bool) {
        if self.playing == playing {
            return;
        }
        self.playing = playing;

        if !playing {
            self.player.pause_all();
            return;
        }
        if !self.forward {
            return;
        }
        self.player.resume_all();
        for (key, layer) in self.active.iter_mut() {
            if !layer.started && self.playhead >= layer.clip_start {
                layer.offset = self.playhead - layer.clip_start;
                self.player
                    .play(key, layer.data.clone(), layer.offset, layer.volume);
                layer.started = true;
            }
        }
    }

    /// Re-derive each clip's position from the playhead after a jump (seek,
    /// segment start, loop wrap).
    pub fn sync(&mut self) {
        let audible = self.playing && self.forward;
        for (key, layer) in self.active.iter_mut() {
            let offset = self.playhead - layer.clip_start;
            layer.offset = offset.max(0.0);
            if audible && offset >= 0.0 {
                self.player
                    .play(key, layer.data.clone(), layer.offset, layer.volume);
                layer.started = true;
            } else {
                self.player.stop(key);
                layer.started = false;
            }
        }
    }

    /// Set the playback rate multiplier, applied to live and future clips.
    pub fn set_rate(&mut self, rate: f32) {
        if rate <= 0.0 {
            return;
        }
        self.rate = rate;
        self.player.set_rate(rate);
    }

    /// Suspend audio while the playhead runs backward; resync on forward.
    pub fn set_direction(&mut self, forward: bool) {
        if self.forward == forward {
            return;
        }
        self.forward = forward;
        if forward {
            self.sync();
        } else {
            self.stop_all();
        }
    }

    /// Stop all sinks. The active set is kept so `set_playing(true)` can replay it.
    pub fn stop(&mut self) {
        self.playing = false;
        self.stop_all();
    }

    /// Stop every sink and mark its layer as needing a fresh start.
    fn stop_all(&mut self) {
        self.player.stop_all();
        for layer in self.active.values_mut() {
            layer.started = false;
        }
    }

    /// Set the global volume multiplier (clamped to [0.0, 1.0]).
    pub fn set_volume(&mut self, volume: f32) {
        self.player.set_global_volume(volume.clamp(0.0, 1.0));
    }

    pub fn volume(&self) -> f32 {
        self.player.global_volume()
    }

    /// Number of audio layers currently within their playback range.
    #[cfg(test)]
    pub(crate) fn active_layer_count(&self) -> usize {
        self.active.len()
    }

    #[cfg(test)]
    pub(crate) fn layer_offsets(&self) -> Vec<f32> {
        self.active.values().map(|layer| layer.offset).collect()
    }

    /// Number of active layers whose clip has been handed to the player.
    #[cfg(test)]
    pub(crate) fn started_layer_count(&self) -> usize {
        self.active.values().filter(|layer| layer.started).count()
    }

    #[cfg(test)]
    pub(crate) fn rate(&self) -> f32 {
        self.rate
    }

    #[cfg(test)]
    pub(crate) fn is_forward(&self) -> bool {
        self.forward
    }
}

/// Whether `src` names an audio format this build plays: MP3 always, the rest
/// behind their `audio-*` features.
pub(crate) fn is_supported_src(src: &str) -> bool {
    let Some((_, ext)) = src.rsplit_once('.') else {
        return false;
    };
    match ext.to_ascii_lowercase().as_str() {
        "mp3" => true,
        #[cfg(feature = "audio-wav")]
        "wav" => true,
        #[cfg(feature = "audio-flac")]
        "flac" => true,
        #[cfg(feature = "audio-ogg")]
        "ogg" => true,
        #[cfg(feature = "audio-m4a")]
        "m4a" => true,
        _ => false,
    }
}

/// MIME type of encoded audio bytes, sniffed from the container header.
#[cfg(any(test, target_arch = "wasm32"))]
fn mime_type(data: &[u8]) -> &'static str {
    match data {
        #[cfg(feature = "audio-wav")]
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'A', b'V', b'E', ..] => "audio/wav",
        #[cfg(feature = "audio-flac")]
        [b'f', b'L', b'a', b'C', ..] => "audio/flac",
        #[cfg(feature = "audio-ogg")]
        [b'O', b'g', b'g', b'S', ..] => "audio/ogg",
        #[cfg(feature = "audio-m4a")]
        [_, _, _, _, b'f', b't', b'y', b'p', ..] => "audio/mp4",
        _ => "audio/mpeg",
    }
}

/// Normalize a resolver `src` for use as a lookup key: drop leading slashes the
/// renderer prefixes onto packaged paths (e.g. `//u/clip.mp3` → `u/clip.mp3`).
fn normalize_src(src: &str) -> String {
    src.trim_start_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::{is_supported_src, mime_type};

    const TONES: [(&str, &[u8], &str, bool); 5] = [
        (
            "mp3",
            include_bytes!("../../assets/audio/tone.mp3"),
            "audio/mpeg",
            true,
        ),
        (
            "wav",
            include_bytes!("../../assets/audio/tone.wav"),
            "audio/wav",
            cfg!(feature = "audio-wav"),
        ),
        (
            "flac",
            include_bytes!("../../assets/audio/tone.flac"),
            "audio/flac",
            cfg!(feature = "audio-flac"),
        ),
        (
            "ogg",
            include_bytes!("../../assets/audio/tone.ogg"),
            "audio/ogg",
            cfg!(feature = "audio-ogg"),
        ),
        (
            "m4a",
            include_bytes!("../../assets/audio/tone.m4a"),
            "audio/mp4",
            cfg!(feature = "audio-m4a"),
        ),
    ];

    #[test]
    fn supports_only_enabled_formats() {
        for (ext, _, _, enabled) in TONES {
            assert_eq!(is_supported_src(&format!("u/tone.{ext}")), enabled, "{ext}");
            assert_eq!(
                is_supported_src(&format!("u/TONE.{}", ext.to_uppercase())),
                enabled,
                "{ext}"
            );
        }
        assert!(!is_supported_src("u/tone"));
        assert!(!is_supported_src("u/tone.aac"));
    }

    #[test]
    fn sniffs_enabled_containers_and_defaults_to_mpeg() {
        for (ext, bytes, mime, enabled) in TONES {
            let expected = if enabled { mime } else { "audio/mpeg" };
            assert_eq!(mime_type(bytes), expected, "{ext}");
        }
        assert_eq!(mime_type(b"RIFF"), "audio/mpeg");
    }
}
