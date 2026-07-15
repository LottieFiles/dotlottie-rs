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
    /// Seek position in seconds.
    offset: f32,
    /// Per-layer volume in [0.0, 1.0].
    volume: f32,
}

/// Frame-synchronised audio playback driven by the renderer's audio resolver.
/// Uses rodio on native targets and `HtmlAudioElement` on wasm.
pub struct AudioManager {
    /// External audio bytes keyed by packaged path (e.g. `u/clip.mp3`); empty
    /// when audio is embedded in the JSON (bytes then arrive in the event).
    sources: FxHashMap<String, Arc<[u8]>>,
    active: FxHashMap<String, ActiveLayer>,
    playing: bool,
    #[cfg(not(target_arch = "wasm32"))]
    player: RodioPlayer,
    #[cfg(target_arch = "wasm32")]
    player: WebAudioPlayer,
}

impl AudioManager {
    /// `sources` maps packaged audio paths to bytes (empty for embedded audio).
    /// The audio device opens lazily on first playback.
    pub fn new(sources: FxHashMap<String, Arc<[u8]>>) -> Self {
        AudioManager {
            sources,
            active: FxHashMap::default(),
            playing: false,
            #[cfg(not(target_arch = "wasm32"))]
            player: RodioPlayer::new(),
            #[cfg(target_arch = "wasm32")]
            player: WebAudioPlayer::new(),
        }
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

        let data = match event.source {
            AudioSource::External(_) => match self.resolve_source(&key) {
                Some(data) => data,
                None => return,
            },
            AudioSource::Embedded { bytes, .. } => Arc::from(bytes),
        };

        let layer = ActiveLayer {
            data,
            offset: event.offset,
            volume: event.volume / 100.0,
        };
        if self.playing {
            self.player
                .play(&key, layer.data.clone(), layer.offset, layer.volume);
        }
        self.active.insert(key, layer);
    }

    /// Look up external audio bytes by normalized `src`, falling back to file name.
    fn resolve_source(&self, normalized: &str) -> Option<Arc<[u8]>> {
        if let Some(bytes) = self.sources.get(normalized) {
            return Some(bytes.clone());
        }
        let base = file_name(normalized);
        self.sources
            .iter()
            .find(|(k, _)| file_name(k) == base)
            .map(|(_, bytes)| bytes.clone())
    }

    /// Reflect the player's playback state, resuming/pausing sinks and starting
    /// any layers that became active while paused.
    pub fn set_playing(&mut self, playing: bool) {
        if self.playing == playing {
            return;
        }
        self.playing = playing;

        if playing {
            self.player.resume_all();
            for (key, layer) in &self.active {
                if !self.player.is_active(key) {
                    self.player
                        .play(key, layer.data.clone(), layer.offset, layer.volume);
                }
            }
        } else {
            self.player.pause_all();
        }
    }

    /// Stop all sinks. The active set is kept so `set_playing(true)` can replay it.
    pub fn stop(&mut self) {
        self.playing = false;
        self.player.stop_all();
    }

    /// Restart every in-range layer from the start of its clip (used on loop).
    pub fn restart(&mut self) {
        if !self.playing {
            return;
        }
        for (key, layer) in &self.active {
            self.player.play(key, layer.data.clone(), 0.0, layer.volume);
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
}

/// Normalize a resolver `src` for use as a lookup key: drop leading slashes the
/// renderer prefixes onto packaged paths (e.g. `//u/clip.mp3` → `u/clip.mp3`).
fn normalize_src(src: &str) -> String {
    src.trim_start_matches('/').to_string()
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}
