use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use rustc_hash::FxHashMap;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

/// Native audio backend powered by rodio. Sinks are keyed by audio source; the
/// output stream opens lazily on first playback.
pub struct RodioPlayer {
    /// Kept alive so the audio output chain stays open; `None` until first use.
    stream: Option<(OutputStream, OutputStreamHandle)>,
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

    /// Open the output stream on demand, returning a handle or `None` if no
    /// audio device is available.
    fn handle(&mut self) -> Option<OutputStreamHandle> {
        if self.stream.is_none() {
            self.stream = OutputStream::try_default().ok();
        }
        self.stream.as_ref().map(|(_, handle)| handle.clone())
    }

    pub fn play(&mut self, key: &str, data: Arc<[u8]>, offset_secs: f32, layer_volume: f32) {
        self.sinks.remove(key);

        let Some(handle) = self.handle() else {
            return;
        };
        let Ok(source) = Decoder::new(Cursor::new(data)) else {
            return;
        };
        let Ok(sink) = Sink::try_new(&handle) else {
            return;
        };

        sink.set_volume(layer_volume * self.global_volume);
        sink.set_speed(self.rate);
        // rodio 0.17 has no seek; skip_duration approximates a mid-clip start.
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
