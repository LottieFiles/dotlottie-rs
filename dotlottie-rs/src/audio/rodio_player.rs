use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

/// Native audio backend powered by rodio
pub struct RodioPlayer {
    /// Kept alive so the audio output chain stays open.
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sinks: HashMap<usize, Sink>,
}

impl RodioPlayer {
    pub fn new() -> Result<Self, String> {
        let (_stream, stream_handle) = OutputStream::try_default().map_err(|e| e.to_string())?;
        Ok(Self {
            _stream,
            stream_handle,
            sinks: HashMap::new(),
        })
    }

    pub fn start(&mut self, id: usize, data: Arc<[u8]>, volume: f32, offset: f32) {
        let cursor = Cursor::new(data);
        if let Ok(source) = Decoder::new(cursor) {
            if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                sink.set_volume(volume);
                if offset > 0.0 {
                    sink.append(source.skip_duration(Duration::from_secs_f32(offset)));
                } else {
                    sink.append(source);
                }
                self.sinks.insert(id, sink);
            }
        }
    }

    pub fn stop(&mut self, id: usize) {
        self.sinks.remove(&id);
    }

    pub fn stop_all(&mut self) {
        self.sinks.clear();
    }

    pub fn pause_all(&mut self) {
        for sink in self.sinks.values() {
            sink.pause();
        }
    }

    pub fn resume_all(&mut self) {
        for sink in self.sinks.values() {
            sink.play();
        }
    }

    /// Adjust the volume of a currently-playing voice without stopping it.
    pub fn set_volume(&mut self, id: usize, volume: f32) {
        if let Some(sink) = self.sinks.get(&id) {
            sink.set_volume(volume);
        }
    }
}
