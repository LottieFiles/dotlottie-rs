use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
mod rodio_player;
#[cfg(not(target_arch = "wasm32"))]
use rodio_player::RodioPlayer as Backend;

#[cfg(target_arch = "wasm32")]
mod web_audio_player;
#[cfg(target_arch = "wasm32")]
use web_audio_player::WebAudioPlayer as Backend;

/// An audio playback-state.
pub struct AudioEvent {
    pub id: usize,
    pub active: bool,
    pub volume: f32,
    pub offset: f32,
    pub data: Option<Arc<[u8]>>,
}

#[derive(Default)]
pub struct AudioBridge {
    queue: Mutex<VecDeque<AudioEvent>>,
}

impl AudioBridge {
    pub fn push(&self, event: AudioEvent) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push_back(event);
        }
    }

    fn drain(&self) -> Vec<AudioEvent> {
        match self.queue.lock() {
            Ok(mut queue) => queue.drain(..).collect(),
            Err(_) => Vec::new(),
        }
    }

    fn clear(&self) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.clear();
        }
    }
}

/// An active voice
struct Voice {
    data: Arc<[u8]>,
    volume: f32,
    base_frame: f32,
    base_offset: f32,
}

/// Drives playback from engine events
pub struct AudioManager {
    bridge: Arc<AudioBridge>,
    backend: Option<Backend>,
    voices: HashMap<usize, Voice>,
    global_volume: f32,
}

impl AudioManager {
    pub fn new(bridge: Arc<AudioBridge>) -> Self {
        Self {
            bridge,
            backend: None,
            voices: HashMap::new(),
            global_volume: 1.0,
        }
    }

    fn backend(&mut self) -> Option<&mut Backend> {
        if self.backend.is_none() {
            self.backend = Backend::new().ok();
        }
        self.backend.as_mut()
    }

    /// Drain engine events into the active-voice set
    pub fn sync(&mut self, playing: bool, frame: f32) {
        let global = self.global_volume;
        for event in self.bridge.drain() {
            if event.active {
                if let Some(voice) = self.voices.get_mut(&event.id) {
                    voice.volume = event.volume;
                    if playing {
                        if let Some(backend) = self.backend() {
                            backend.set_volume(event.id, event.volume * global);
                        }
                    }
                } else if let Some(data) = event.data {
                    self.voices.insert(
                        event.id,
                        Voice {
                            data: data.clone(),
                            volume: event.volume,
                            base_frame: frame,
                            base_offset: event.offset,
                        },
                    );
                    if playing {
                        if let Some(backend) = self.backend() {
                            backend.start(event.id, data, event.volume * global, event.offset);
                        }
                    }
                }
            } else if self.voices.remove(&event.id).is_some() {
                if playing {
                    if let Some(backend) = self.backend() {
                        backend.stop(event.id);
                    }
                }
            }
        }
    }

    /// (Re)start active voice.
    pub fn reposition(&mut self, frame: f32, fps: f32) {
        if fps <= 0.0 || self.voices.is_empty() {
            return;
        }
        let global = self.global_volume;
        let voices: Vec<(usize, Arc<[u8]>, f32, f32)> = self
            .voices
            .iter()
            .map(|(id, voice)| {
                let offset = voice.base_offset + (frame - voice.base_frame) / fps;
                (*id, voice.data.clone(), voice.volume, offset)
            })
            .collect();
        if let Some(backend) = self.backend() {
            for (id, data, volume, offset) in voices {
                backend.start(id, data, volume * global, offset);
            }
        }
    }

    pub fn pause(&mut self) {
        if let Some(backend) = self.backend.as_mut() {
            backend.pause_all();
        }
    }

    pub fn resume(&mut self) {
        if let Some(backend) = self.backend.as_mut() {
            backend.resume_all();
        }
    }

    pub fn stop(&mut self) {
        self.bridge.clear();
        if let Some(backend) = self.backend.as_mut() {
            backend.stop_all();
        }
    }

    /// Set the global volume multiplier (clamped to `[0.0, 1.0]`).
    pub fn set_volume(&mut self, volume: f32) {
        self.global_volume = volume.clamp(0.0, 1.0);
        let global = self.global_volume;
        let updates: Vec<(usize, f32)> = self
            .voices
            .iter()
            .map(|(id, voice)| (*id, voice.volume))
            .collect();
        if let Some(backend) = self.backend.as_mut() {
            for (id, volume) in updates {
                backend.set_volume(id, volume * global);
            }
        }
    }

    pub fn volume(&self) -> f32 {
        self.global_volume
    }
}
