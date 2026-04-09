use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::Cursor;
use std::sync::Arc;

/// Native audio backend powered by rodio (cpal → CoreAudio on Apple, WASAPI on
/// Windows, ALSA/PulseAudio on Linux, OpenSL ES / AAudio on Android).
pub struct RodioPlayer {
    /// Kept alive so the audio output chain stays open.
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    /// One slot per layer. None = not playing, Some(sink) = active sink.
    sinks: Vec<Option<Sink>>,
}

impl RodioPlayer {
    pub fn new(layer_count: usize) -> Result<Self, String> {
        let (_stream, stream_handle) = OutputStream::try_default().map_err(|e| e.to_string())?;
        Ok(Self {
            _stream,
            stream_handle,
            sinks: (0..layer_count).map(|_| None).collect(),
        })
    }

    /// Start playing the given audio data in the sink owned by `layer_idx`.
    pub fn play(&mut self, layer_idx: usize, data: Arc<[u8]>) {
        self.sinks[layer_idx].take();
        let cursor = Cursor::new(data);
        if let Ok(source) = Decoder::new(cursor) {
            if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                sink.append(source);
                self.sinks[layer_idx] = Some(sink);
            }
        }
    }

    pub fn pause(&mut self, layer_idx: usize) {
        if let Some(sink) = self.sinks[layer_idx].as_ref() {
            sink.pause();
        }
    }

    pub fn resume(&mut self, layer_idx: usize) {
        if let Some(sink) = self.sinks[layer_idx].as_ref() {
            sink.play();
        }
    }

    pub fn stop(&mut self, layer_idx: usize) {
        self.sinks[layer_idx].take();
    }

    /// Adjust the volume of a currently-playing sink without stopping it.
    pub fn set_volume(&mut self, layer_idx: usize, volume: f32) {
        if let Some(sink) = self.sinks[layer_idx].as_ref() {
            sink.set_volume(volume);
        }
    }
}
