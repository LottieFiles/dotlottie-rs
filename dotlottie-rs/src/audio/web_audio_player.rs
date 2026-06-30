use js_sys::Array;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use web_sys::{Blob, BlobPropertyBag, HtmlAudioElement, Url};

/// One playing audio source, keyed by audio source in `WebAudioPlayer`.
struct AudioEntry {
    element: HtmlAudioElement,
    /// Object URL, revoked on stop.
    url: String,
    /// Per-layer volume in [0.0, 1.0], for re-applying the global multiplier.
    layer_volume: f32,
}

/// Web audio backend powered by the browser's native `HtmlAudioElement`. MP3
/// bytes play via a `Blob` object URL, so no decoder ships in the wasm binary.
pub struct WebAudioPlayer {
    elements: FxHashMap<String, AudioEntry>,
    /// Global multiplier in [0.0, 1.0], applied on top of per-layer volume.
    global_volume: f32,
}

impl Default for WebAudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl WebAudioPlayer {
    pub fn new() -> Self {
        Self {
            elements: FxHashMap::default(),
            global_volume: 1.0,
        }
    }

    pub fn play(&mut self, key: &str, data: Arc<[u8]>, offset_secs: f32, layer_volume: f32) {
        self.stop(key);

        let uint8_array = js_sys::Uint8Array::from(data.as_ref());
        let array = Array::new();
        array.push(&uint8_array);

        let props = BlobPropertyBag::new();
        props.set_type("audio/mpeg");

        let blob = match Blob::new_with_u8_array_sequence_and_options(&array, &props) {
            Ok(b) => b,
            Err(_) => return,
        };

        let url = match Url::create_object_url_with_blob(&blob) {
            Ok(u) => u,
            Err(_) => return,
        };

        let element = match HtmlAudioElement::new_with_src(&url) {
            Ok(e) => e,
            Err(_) => {
                let _ = Url::revoke_object_url(&url);
                return;
            }
        };

        element.set_volume((layer_volume * self.global_volume) as f64);
        if offset_secs > 0.0 {
            element.set_current_time(offset_secs as f64);
        }

        // `.play()` returns a Promise; fire-and-forget.
        let _ = element.play();

        self.elements.insert(
            key.to_string(),
            AudioEntry {
                element,
                url,
                layer_volume,
            },
        );
    }

    pub fn stop(&mut self, key: &str) {
        if let Some(entry) = self.elements.remove(key) {
            let _ = entry.element.pause();
            let _ = Url::revoke_object_url(&entry.url);
        }
    }

    /// Whether an element for `key` exists and has not finished playing.
    pub fn is_active(&self, key: &str) -> bool {
        self.elements
            .get(key)
            .is_some_and(|entry| !entry.element.ended())
    }

    pub fn pause_all(&mut self) {
        for entry in self.elements.values() {
            let _ = entry.element.pause();
        }
    }

    pub fn resume_all(&mut self) {
        for entry in self.elements.values() {
            let _ = entry.element.play();
        }
    }

    pub fn stop_all(&mut self) {
        for (_, entry) in self.elements.drain() {
            let _ = entry.element.pause();
            let _ = Url::revoke_object_url(&entry.url);
        }
    }

    pub fn set_global_volume(&mut self, volume: f32) {
        self.global_volume = volume;
        for entry in self.elements.values() {
            entry
                .element
                .set_volume((entry.layer_volume * self.global_volume) as f64);
        }
    }

    pub fn global_volume(&self) -> f32 {
        self.global_volume
    }
}
