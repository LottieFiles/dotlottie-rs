use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

/// Native audio backend powered by rodio (cpal → CoreAudio on Apple, WASAPI on
/// Windows, ALSA/PulseAudio on Linux, OpenSL ES / AAudio on Android).
pub struct RodioPlayer {
    /// Kept alive so the audio output chain stays open.
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    /// Pre-decoded raw audio bytes, keyed by asset id.
    buffers: HashMap<String, Arc<[u8]>>,
    /// Active sinks, keyed by layer index so that multiple layers sharing the
    /// same asset each have an independent sink.
    sinks: HashMap<usize, Sink>,
}

impl RodioPlayer {
    pub fn new() -> Result<Self, String> {
        let (_stream, stream_handle) = OutputStream::try_default().map_err(|e| e.to_string())?;
        Ok(Self {
            _stream,
            stream_handle,
            buffers: HashMap::new(),
            sinks: HashMap::new(),
        })
    }

    /// Store raw audio bytes for later playback.
    pub fn load(&mut self, id: &str, data: &[u8]) -> bool {
        self.buffers
            .insert(id.to_string(), Arc::<[u8]>::from(data.to_vec()));
        true
    }

    /// Start playing the asset identified by `ref_id` in a sink owned by `layer_idx`.
    pub fn play(&mut self, layer_idx: usize, ref_id: &str, volume: f32) {
        self.sinks.remove(&layer_idx);
        if let Some(data) = self.buffers.get(ref_id) {
            let cursor = Cursor::new(data.clone());
            if let Ok(source) = Decoder::new(cursor) {
                if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                    sink.set_volume(volume);
                    sink.append(source);
                    self.sinks.insert(layer_idx, sink);
                }
            }
        }
    }

    pub fn pause(&mut self, layer_idx: usize) {
        if let Some(sink) = self.sinks.get(&layer_idx) {
            sink.pause();
        }
    }

    pub fn resume(&mut self, layer_idx: usize) {
        if let Some(sink) = self.sinks.get(&layer_idx) {
            sink.play();
        }
    }

    pub fn stop(&mut self, layer_idx: usize) {
        self.sinks.remove(&layer_idx);
    }

    /// Adjust the volume of a currently-playing sink without stopping it.
    pub fn set_volume(&mut self, layer_idx: usize, volume: f32) {
        if let Some(sink) = self.sinks.get(&layer_idx) {
            sink.set_volume(volume);
        }
    }
}
