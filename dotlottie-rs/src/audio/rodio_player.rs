use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::io::Cursor;

/// Native audio backend powered by rodio (cpal → CoreAudio on Apple, WASAPI on
/// Windows, ALSA/PulseAudio on Linux, OpenSL ES / AAudio on Android).
pub struct RodioPlayer {
    /// Kept alive so the audio output chain stays open.
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    /// Pre-decoded raw audio bytes, keyed by asset id.
    buffers: HashMap<String, Vec<u8>>,
    /// Active sinks, keyed by asset id.
    sinks: HashMap<String, Sink>,
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
        self.buffers.insert(id.to_string(), data.to_vec());
        true
    }

    /// Rewind and start playing from the stored buffer.
    pub fn play(&mut self, id: &str, volume: f32) {
        self.sinks.remove(id);
        if let Some(data) = self.buffers.get(id) {
            let cursor = Cursor::new(data.clone());
            if let Ok(source) = Decoder::new(cursor) {
                if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                    sink.set_volume(volume);
                    sink.append(source);
                    self.sinks.insert(id.to_string(), sink);
                }
            }
        }
    }

    pub fn pause(&mut self, id: &str) {
        if let Some(sink) = self.sinks.get(id) {
            sink.pause();
        }
    }

    pub fn resume(&mut self, id: &str) {
        if let Some(sink) = self.sinks.get(id) {
            sink.play();
        }
    }

    pub fn stop(&mut self, id: &str) {
        self.sinks.remove(id);
    }

    /// Adjust the volume of a currently-playing sink without stopping it.
    pub fn set_volume(&mut self, id: &str, volume: f32) {
        if let Some(sink) = self.sinks.get(id) {
            sink.set_volume(volume);
        }
    }
}
