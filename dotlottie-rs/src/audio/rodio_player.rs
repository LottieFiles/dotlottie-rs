use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::io::Cursor;

/// Native audio backend powered by rodio.
///
/// Each audio asset gets its own [`Sink`] so multiple assets can play
/// simultaneously. The [`OutputStream`] must be kept alive for the duration of
/// playback — it is stored in `_stream` (prefixed underscore prevents the
/// compiler from dropping it early while still suppressing the unused-variable
/// warning).
pub struct RodioPlayer {
    /// Kept alive to maintain the audio output stream.
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    /// Active sinks keyed by asset id.
    sinks: HashMap<String, Sink>,
    /// Raw audio bytes keyed by asset id, used to create new sinks on play.
    buffers: HashMap<String, Vec<u8>>,
}

impl RodioPlayer {
    pub fn new() -> Result<Self, String> {
        let (_stream, stream_handle) =
            OutputStream::try_default().map_err(|e| e.to_string())?;

        Ok(RodioPlayer {
            _stream,
            stream_handle,
            sinks: HashMap::new(),
            buffers: HashMap::new(),
        })
    }

    /// Store audio bytes for an asset so they can be played later.
    pub fn load(&mut self, id: &str, data: &[u8]) {
        self.buffers.insert(id.to_string(), data.to_vec());
    }

    /// Start (or restart) playback of the asset identified by `id`.
    pub fn play(&mut self, id: &str, volume: f32) {
        // Stop any existing sink for this asset before starting a new one.
        if let Some(sink) = self.sinks.remove(id) {
            sink.stop();
        }

        if let Some(data) = self.buffers.get(id) {
            match Sink::try_new(&self.stream_handle) {
                Ok(sink) => {
                    let cursor = Cursor::new(data.clone());
                    match Decoder::new(cursor) {
                        Ok(source) => {
                            sink.set_volume(volume);
                            sink.append(source);
                            self.sinks.insert(id.to_string(), sink);
                        }
                        Err(_) => {
                            // Decoder failed (unsupported format or corrupt data).
                            // Sink is dropped and playback is silently skipped.
                        }
                    }
                }
                Err(_) => {
                    // Could not create a sink (audio device unavailable, etc.).
                }
            }
        }
    }

    /// Pause a playing sink. Does nothing if the asset is not active.
    pub fn pause(&mut self, id: &str) {
        if let Some(sink) = self.sinks.get(id) {
            sink.pause();
        }
    }

    /// Resume a paused sink. Does nothing if the asset is not active.
    pub fn resume(&mut self, id: &str) {
        if let Some(sink) = self.sinks.get(id) {
            sink.play();
        }
    }

    /// Stop and discard the sink for `id`. Does nothing if not active.
    pub fn stop(&mut self, id: &str) {
        if let Some(sink) = self.sinks.remove(id) {
            sink.stop();
        }
    }
}
