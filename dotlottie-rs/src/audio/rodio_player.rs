use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use rustc_hash::FxHashMap;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

/// Native audio backend powered by rodio. Sinks are keyed by audio source; the
/// output stream opens lazily on first playback.
pub struct RodioPlayer {
    /// Kept alive so the audio output chain stays open; `None` until first use.
    stream: Option<OutputStream>,
    sinks: FxHashMap<String, ActiveSink>,
    /// Global multiplier in [0.0, 1.0], applied on top of per-layer volume.
    global_volume: f32,
    /// Playback rate multiplier (rodio resamples, so pitch shifts with it).
    rate: f32,
}

struct ActiveSink {
    sink: Sink,
    /// Per-layer volume in [0.0, 1.0], for re-applying the global multiplier.
    layer_volume: f32,
}

impl Default for RodioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl RodioPlayer {
    pub fn new() -> Self {
        Self {
            stream: None,
            sinks: FxHashMap::default(),
            global_volume: 1.0,
            rate: 1.0,
        }
    }

    /// Connect a sink to the output stream, opening it on demand; `None` if no
    /// audio device is available.
    fn new_sink(&mut self) -> Option<Sink> {
        if self.stream.is_none() {
            self.stream = OutputStreamBuilder::open_default_stream()
                .ok()
                .map(|mut stream| {
                    stream.log_on_drop(false);
                    stream
                });
        }
        self.stream
            .as_ref()
            .map(|stream| Sink::connect_new(stream.mixer()))
    }

    pub fn play(&mut self, key: &str, data: Arc<[u8]>, offset_secs: f32, layer_volume: f32) {
        self.sinks.remove(key);

        let Some(source) = decode(data) else {
            return;
        };
        let Some(sink) = self.new_sink() else {
            return;
        };

        sink.set_volume(layer_volume * self.global_volume);
        sink.set_speed(self.rate);
        // skip_duration approximates a mid-clip start.
        sink.append(source.skip_duration(Duration::from_secs_f32(offset_secs.max(0.0))));

        self.sinks
            .insert(key.to_string(), ActiveSink { sink, layer_volume });
    }

    pub fn stop(&mut self, key: &str) {
        self.sinks.remove(key);
    }

    pub fn pause_all(&mut self) {
        for active in self.sinks.values() {
            active.sink.pause();
        }
    }

    pub fn resume_all(&mut self) {
        for active in self.sinks.values() {
            active.sink.play();
        }
    }

    pub fn stop_all(&mut self) {
        self.sinks.clear();
    }

    /// Update one clip's per-layer volume without restarting it.
    pub fn set_volume(&mut self, key: &str, layer_volume: f32) {
        if let Some(active) = self.sinks.get_mut(key) {
            active.layer_volume = layer_volume;
            active.sink.set_volume(layer_volume * self.global_volume);
        }
    }

    /// Set the playback rate multiplier for live and future sinks.
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
        for active in self.sinks.values() {
            active.sink.set_speed(rate);
        }
    }

    pub fn set_global_volume(&mut self, volume: f32) {
        self.global_volume = volume;
        for active in self.sinks.values() {
            active
                .sink
                .set_volume(active.layer_volume * self.global_volume);
        }
    }

    pub fn global_volume(&self) -> f32 {
        self.global_volume
    }
}

/// Decode `data` in any enabled format. The byte length lets symphonia read MP4
/// files whose index follows the media data.
fn decode(data: Arc<[u8]>) -> Option<Decoder<Cursor<Arc<[u8]>>>> {
    let len = data.len() as u64;
    Decoder::builder()
        .with_data(Cursor::new(data))
        .with_byte_len(len)
        .with_seekable(true)
        .build()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::decode;

    fn decodes(bytes: &'static [u8]) -> bool {
        decode(bytes.into()).is_some_and(|source| source.count() > 0)
    }

    #[test]
    fn decodes_mp3() {
        assert!(decodes(include_bytes!("../../assets/audio/tone.mp3")));
    }

    #[cfg(feature = "audio-wav")]
    #[test]
    fn decodes_wav() {
        assert!(decodes(include_bytes!("../../assets/audio/tone.wav")));
    }

    #[cfg(feature = "audio-flac")]
    #[test]
    fn decodes_flac() {
        assert!(decodes(include_bytes!("../../assets/audio/tone.flac")));
    }

    #[cfg(feature = "audio-ogg")]
    #[test]
    fn decodes_ogg() {
        assert!(decodes(include_bytes!("../../assets/audio/tone.ogg")));
    }

    #[cfg(feature = "audio-m4a")]
    #[test]
    fn decodes_m4a() {
        assert!(decodes(include_bytes!("../../assets/audio/tone.m4a")));
    }
}
