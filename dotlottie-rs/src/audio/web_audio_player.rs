use js_sys::Array;
use std::sync::Arc;
use web_sys::{Blob, BlobPropertyBag, HtmlAudioElement, Url};

/// Web audio backend powered by the browser's native `HtmlAudioElement`.
///
/// Each audio layer gets its own `<audio>` element. Raw MP3 bytes are wrapped
/// in a `Blob` and exposed via an object URL so the browser can decode and play
/// them without bundling any audio decoder in the wasm binary.
pub struct WebAudioPlayer {
    elements: Vec<Option<HtmlAudioElement>>,
    /// Object URLs created via `URL.createObjectURL`; kept so we can revoke
    /// them when a layer stops to avoid leaking browser-side resources.
    object_urls: Vec<Option<String>>,
}

impl WebAudioPlayer {
    pub fn new(layer_count: usize) -> Result<Self, String> {
        Ok(Self {
            elements: (0..layer_count).map(|_| None).collect(),
            object_urls: (0..layer_count).map(|_| None).collect(),
        })
    }

    /// Start playing the given audio data in the slot owned by `layer_idx`.
    pub fn play(&mut self, layer_idx: usize, data: Arc<[u8]>, volume: f32) {
        // Wrap the raw bytes in a Blob with the appropriate MIME type so the
        // browser knows how to decode the audio.
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

        element.set_volume(volume as f64);
        // `.play()` returns a Promise; we fire-and-forget since playback is
        // driven by frame updates rather than a completion callback.
        let _ = element.play();

        self.object_urls[layer_idx] = Some(url);
        self.elements[layer_idx] = Some(element);
    }

    pub fn pause(&mut self, layer_idx: usize) {
        if let Some(elem) = self.elements[layer_idx].as_ref() {
            let _ = elem.pause();
        }
    }

    pub fn resume(&mut self, layer_idx: usize) {
        if let Some(elem) = self.elements[layer_idx].as_ref() {
            let _ = elem.play();
        }
    }

    pub fn stop(&mut self, layer_idx: usize) {
        if let Some(elem) = self.elements[layer_idx].take() {
            let _ = elem.pause();
        }
        if let Some(url) = self.object_urls[layer_idx].take() {
            let _ = Url::revoke_object_url(&url);
        }
    }

    /// Adjust the volume of a currently-playing element without stopping it.
    pub fn set_volume(&mut self, layer_idx: usize, volume: f32) {
        if let Some(elem) = self.elements[layer_idx].as_ref() {
            elem.set_volume(volume as f64);
        }
    }
}
