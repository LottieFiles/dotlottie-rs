use js_sys::Uint8Array;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, GainNode};

/// A currently-playing voice
struct Voice {
    source: AudioBufferSourceNode,
    gain: GainNode,
}

struct Inner {
    ctx: AudioContext,
    buffers: HashMap<usize, AudioBuffer>,
    voices: HashMap<usize, Voice>,
    seq: HashMap<usize, u64>,
}

impl Inner {
    fn bump(&mut self, id: usize) -> u64 {
        let g = self.seq.entry(id).or_insert(0);
        *g += 1;
        *g
    }

    fn stop_source(&mut self, id: usize) {
        if let Some(voice) = self.voices.remove(&id) {
            let _ = web_sys::AudioScheduledSourceNode::stop(&voice.source);
            let _ = voice.source.disconnect();
            let _ = voice.gain.disconnect();
        }
    }

    fn play_now(&mut self, id: usize, volume: f32, offset: f32) {
        let Some(buffer) = self.buffers.get(&id).cloned() else {
            return;
        };
        let Ok(source) = self.ctx.create_buffer_source() else {
            return;
        };
        let Ok(gain) = self.ctx.create_gain() else {
            return;
        };

        source.set_buffer(Some(&buffer));
        source.set_loop(true);
        gain.gain().set_value(volume);

        let _ = source.connect_with_audio_node(&gain);
        let destination = self.ctx.destination();
        let _ = gain.connect_with_audio_node(&destination);
        let duration = buffer.duration();
        let start_offset = if duration > 0.0 {
            (offset as f64).rem_euclid(duration)
        } else {
            0.0
        };
        let _ = source.start_with_when_and_grain_offset(0.0, start_offset);

        self.voices.insert(id, Voice { source, gain });
    }
}

pub struct WebAudioPlayer {
    inner: Rc<RefCell<Inner>>,
}

impl WebAudioPlayer {
    pub fn new() -> Result<Self, String> {
        let ctx = AudioContext::new().map_err(|_| "failed to create AudioContext".to_string())?;
        Ok(Self {
            inner: Rc::new(RefCell::new(Inner {
                ctx,
                buffers: HashMap::new(),
                voices: HashMap::new(),
                seq: HashMap::new(),
            })),
        })
    }

    /// Start (or restart) `id`'s playback at `volume`, `offset` seconds into the clip.
    pub fn start(&mut self, id: usize, data: Arc<[u8]>, volume: f32, offset: f32) {
        let (generation, started_at, promise) = {
            let mut inner = self.inner.borrow_mut();
            let generation = inner.bump(id);
            inner.stop_source(id);
            let _ = inner.ctx.resume();
            if inner.buffers.contains_key(&id) {
                inner.play_now(id, volume, offset);
                return;
            }
            let started_at = inner.ctx.current_time();
            let array_buffer = Uint8Array::from(data.as_ref()).buffer();
            let promise = match inner.ctx.decode_audio_data(&array_buffer) {
                Ok(p) => p,
                Err(_) => return,
            };
            (generation, started_at, promise)
        };

        let inner_rc = self.inner.clone();
        spawn_local(async move {
            let Ok(decoded) = JsFuture::from(promise).await else {
                return;
            };
            let Ok(buffer) = decoded.dyn_into::<AudioBuffer>() else {
                return;
            };
            let mut inner = inner_rc.borrow_mut();
            inner.buffers.insert(id, buffer);
            if inner.seq.get(&id).copied() == Some(generation) {
                let elapsed = (inner.ctx.current_time() - started_at).max(0.0) as f32;
                inner.play_now(id, volume, offset + elapsed);
            }
        });
    }

    pub fn stop(&mut self, id: usize) {
        let mut inner = self.inner.borrow_mut();
        inner.bump(id);
        inner.stop_source(id);
    }

    pub fn stop_all(&mut self) {
        let mut inner = self.inner.borrow_mut();
        let ids: Vec<usize> = inner.voices.keys().copied().collect();
        for id in ids {
            inner.bump(id);
            inner.stop_source(id);
        }
    }

    pub fn pause_all(&mut self) {
        let _ = self.inner.borrow().ctx.suspend();
    }

    pub fn resume_all(&mut self) {
        let _ = self.inner.borrow().ctx.resume();
    }

    pub fn set_volume(&mut self, id: usize, volume: f32) {
        let inner = self.inner.borrow();
        if let Some(voice) = inner.voices.get(&id) {
            voice.gain.gain().set_value(volume);
        }
    }
}
