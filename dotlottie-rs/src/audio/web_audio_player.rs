use rustc_hash::FxHashMap;
use std::sync::Arc;

use clip::Clip;

/// Web audio backend powered by the browser's native `HtmlAudioElement`. Audio
/// bytes play via a `Blob` object URL, so no decoder ships in the wasm binary.
pub struct WebAudioPlayer {
    clips: FxHashMap<String, Clip>,
    /// Global multiplier in [0.0, 1.0], applied on top of per-layer volume.
    global_volume: f32,
    /// Playback rate multiplier (`playbackRate`; the browser preserves pitch).
    rate: f32,
}

impl Default for WebAudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl WebAudioPlayer {
    pub fn new() -> Self {
        Self {
            clips: FxHashMap::default(),
            global_volume: 1.0,
            rate: 1.0,
        }
    }

    pub fn play(&mut self, key: &str, data: Arc<[u8]>, offset_secs: f32, layer_volume: f32) {
        // Drop the old clip first so it pauses before the new one plays.
        self.clips.remove(key);
        if let Some(clip) = Clip::start(
            &data,
            offset_secs,
            layer_volume,
            self.global_volume,
            self.rate,
        ) {
            self.clips.insert(key.to_string(), clip);
        }
    }

    pub fn stop(&mut self, key: &str) {
        self.clips.remove(key);
    }

    pub fn pause_all(&mut self) {
        for clip in self.clips.values() {
            clip.pause();
        }
    }

    pub fn resume_all(&mut self) {
        for clip in self.clips.values() {
            clip.resume();
        }
    }

    pub fn stop_all(&mut self) {
        self.clips.clear();
    }

    /// Update one clip's per-layer volume without restarting it.
    pub fn set_volume(&mut self, key: &str, layer_volume: f32) {
        if let Some(clip) = self.clips.get_mut(key) {
            clip.set_layer_volume(layer_volume, self.global_volume);
        }
    }

    /// Set the playback rate multiplier for live and future elements, clamped
    /// to [0.0625, 16] — Chrome throws NotSupportedError outside that range.
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate.clamp(0.0625, 16.0);
        for clip in self.clips.values() {
            clip.set_rate(self.rate);
        }
    }

    pub fn set_global_volume(&mut self, volume: f32) {
        self.global_volume = volume;
        for clip in self.clips.values() {
            clip.apply_volume(volume);
        }
    }

    pub fn global_volume(&self) -> f32 {
        self.global_volume
    }
}

/// The only code that touches an audio element. Keep the element private:
/// `volume` and `muted` must be written together, by `apply_volume`.
mod clip {
    use crate::audio::{loudness, mime_type};
    use js_sys::{Array, Uint8Array};
    use web_sys::{Blob, BlobPropertyBag, HtmlAudioElement, Url};

    /// One playing clip: an audio element and the object URL it plays from.
    pub(super) struct Clip {
        element: HtmlAudioElement,
        /// Revoked on drop.
        url: String,
        layer_volume: f32,
    }

    impl Clip {
        /// Start playing `data` from `offset_secs`. `None` if the browser
        /// refuses the blob, URL or element.
        pub(super) fn start(
            data: &[u8],
            offset_secs: f32,
            layer_volume: f32,
            global_volume: f32,
            rate: f32,
        ) -> Option<Self> {
            let array = Array::new();
            array.push(&Uint8Array::from(data));

            let props = BlobPropertyBag::new();
            props.set_type(mime_type(data));

            let blob = Blob::new_with_u8_array_sequence_and_options(&array, &props).ok()?;
            let url = Url::create_object_url_with_blob(&blob).ok()?;

            let element = match HtmlAudioElement::new_with_src(&url) {
                Ok(e) => e,
                Err(_) => {
                    let _ = Url::revoke_object_url(&url);
                    return None;
                }
            };

            let clip = Clip {
                element,
                url,
                layer_volume,
            };
            clip.apply_volume(global_volume);
            clip.set_rate(rate);
            if offset_secs > 0.0 {
                clip.element.set_current_time(offset_secs as f64);
            }
            clip.resume();

            Some(clip)
        }

        pub(super) fn set_layer_volume(&mut self, layer_volume: f32, global_volume: f32) {
            self.layer_volume = layer_volume;
            self.apply_volume(global_volume);
        }

        pub(super) fn apply_volume(&self, global_volume: f32) {
            let loudness = loudness(self.layer_volume, global_volume);
            self.element.set_volume(loudness.volume);
            self.element.set_muted(loudness.muted);
        }

        pub(super) fn set_rate(&self, rate: f32) {
            self.element.set_playback_rate(rate as f64);
        }

        pub(super) fn pause(&self) {
            let _ = self.element.pause();
        }

        /// `.play()` returns a Promise; fire-and-forget.
        pub(super) fn resume(&self) {
            let _ = self.element.play();
        }
    }

    /// Media elements keep playing after the Rust handle is dropped.
    impl Drop for Clip {
        fn drop(&mut self) {
            self.pause();
            let _ = Url::revoke_object_url(&self.url);
        }
    }
}
