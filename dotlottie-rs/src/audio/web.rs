//! Browser audio through `HtmlAudioElement`. Clip bytes play from a `Blob`
//! object URL, so the wasm binary ships no decoder. The rules below are plain
//! functions so native `cargo test` covers them.

#[cfg(target_arch = "wasm32")]
pub use element::Clip;

/// MIME type of encoded audio bytes, sniffed from the container header.
pub(super) fn mime_type(data: &[u8]) -> &'static str {
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct Loudness {
    volume: f64,
    muted: bool,
}

/// `HTMLMediaElement.volume` throws outside [0, 1] and on NaN, so each factor
/// is clamped and NaN counts as 0. iOS Safari ignores `volume` writes, so a
/// global volume of 0 also mutes.
fn loudness(layer_volume: f32, global_volume: f32) -> Loudness {
    let unit = |v: f32| if v > 0.0 { v.min(1.0) } else { 0.0 };
    let global = unit(global_volume);
    Loudness {
        volume: (unit(layer_volume) * global) as f64,
        muted: global == 0.0,
    }
}

/// Chrome throws NotSupportedError for a `playbackRate` outside [0.0625, 16].
fn playback_rate(rate: f32) -> f64 {
    rate.clamp(0.0625, 16.0) as f64
}

/// The only code that touches an audio element. `set_gain` writes `volume`
/// and `muted` together.
#[cfg(target_arch = "wasm32")]
mod element {
    use js_sys::{Array, Uint8Array};
    use std::sync::Arc;
    use web_sys::{Blob, BlobPropertyBag, HtmlAudioElement, Url};

    use super::{loudness, mime_type, playback_rate};
    use crate::audio::{AudioClip, Cue, Gain};

    pub struct Clip {
        element: HtmlAudioElement,
        /// Revoked on drop.
        url: String,
    }

    impl AudioClip for Clip {
        type Output = ();

        fn start(_: &mut (), data: &Arc<[u8]>, cue: Cue) -> Option<Self> {
            let array = Array::new();
            array.push(&Uint8Array::from(&data[..]));

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

            let clip = Clip { element, url };
            clip.set_gain(cue.gain);
            clip.set_rate(cue.rate);
            if cue.offset > 0.0 {
                clip.element.set_current_time(cue.offset as f64);
            }
            clip.resume();
            Some(clip)
        }

        fn pause(&self) {
            let _ = self.element.pause();
        }

        /// Ignores the Promise that `.play()` returns.
        fn resume(&self) {
            let _ = self.element.play();
        }

        fn set_gain(&self, gain: Gain) {
            let loudness = loudness(gain.layer, gain.global);
            self.element.set_volume(loudness.volume);
            self.element.set_muted(loudness.muted);
        }

        fn set_rate(&self, rate: f32) {
            self.element.set_playback_rate(playback_rate(rate));
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

#[cfg(test)]
mod tests {
    use super::{loudness, playback_rate, Loudness};

    #[test]
    fn loudness_stays_in_range_and_mutes_at_zero_global_volume() {
        let cases = [
            // (layer, global, volume, muted)
            (1.0, 1.0, 1.0, false),
            (0.5, 0.5, 0.25, false),
            (1.0, 0.0, 0.0, true),
            (0.0, 1.0, 0.0, false),
            (2.0, 0.5, 0.5, false),
            (-1.0, 1.0, 0.0, false),
            (1.0, -1.0, 0.0, true),
            (f32::NAN, 1.0, 0.0, false),
            (1.0, f32::NAN, 0.0, true),
            (1.0, 2.0, 1.0, false),
        ];
        for (layer, global, volume, muted) in cases {
            assert_eq!(
                loudness(layer, global),
                Loudness { volume, muted },
                "{layer} * {global}"
            );
        }
    }

    #[test]
    fn playback_rate_stays_in_the_range_browsers_accept() {
        assert_eq!(playback_rate(1.0), 1.0);
        assert_eq!(playback_rate(0.01), 0.0625);
        assert_eq!(playback_rate(100.0), 16.0);
    }
}
