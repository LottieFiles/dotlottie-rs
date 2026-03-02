use hound::WavReader;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::collections::HashMap;
use std::io::Cursor;

struct StreamPlayer {
    samples: Vec<i16>,
    pos: usize,
    volume: f32,
    playing: bool,
}

impl AudioCallback for StreamPlayer {
    type Channel = i16;
    fn callback(&mut self, out: &mut [i16]) {
        if !self.playing {
            out.fill(0);
            return;
        }
        let n = (self.samples.len().saturating_sub(self.pos)).min(out.len());
        for (dst, &src) in out[..n].iter_mut().zip(&self.samples[self.pos..]) {
            *dst = (src as f32 * self.volume) as i16;
        }
        out[n..].fill(0);
        self.pos += n;
        if self.pos >= self.samples.len() {
            self.playing = false;
        }
    }
}

struct AudioBuffer {
    samples: Vec<i16>,
    sample_rate: i32,
    channels: u8,
}

/// Native audio backend powered by SDL2.
///
/// Each asset's `AudioDevice` is opened once in `load()` and kept running
/// (outputting silence until `play()` is called). This eliminates the
/// start-up latency that would occur if a new device were created on every
/// `play()` call, and ensures Core Audio's resampler is fully warm.
pub struct SdlPlayer {
    /// Kept alive so that SDL is not shut down while audio is active.
    _sdl_context: sdl2::Sdl,
    audio_subsystem: sdl2::AudioSubsystem,
    devices: HashMap<String, sdl2::audio::AudioDevice<StreamPlayer>>,
}

impl SdlPlayer {
    pub fn new() -> Result<Self, String> {
        let ctx = sdl2::init().map_err(|e| e.to_string())?;
        let audio = ctx.audio().map_err(|e| e.to_string())?;
        Ok(Self {
            _sdl_context: ctx,
            audio_subsystem: audio,
            devices: HashMap::new(),
        })
    }

    /// Decode audio and open a persistent (silenced) device for this asset.
    /// Returns `true` if decoding and device creation both succeeded.
    pub fn load(&mut self, id: &str, data: &[u8]) -> bool {
        let buf = match decode(data) {
            Some(b) => b,
            None => return false,
        };
        let spec = AudioSpecDesired {
            freq: Some(buf.sample_rate),
            channels: Some(buf.channels),
            samples: None,
        };
        match self.audio_subsystem.open_playback(None, &spec, |_| StreamPlayer {
            samples: buf.samples,
            pos: 0,
            volume: 1.0,
            playing: false,
        }) {
            Ok(dev) => {
                // Resume immediately so Core Audio's output chain is initialised
                // and the resampler is warm before the first play() call.
                dev.resume();
                self.devices.insert(id.to_string(), dev);
                true
            }
            Err(e) => {
                eprintln!("[audio] open_playback failed for '{}': {e}", id);
                false
            }
        }
    }

    /// Rewind and start playing.
    pub fn play(&mut self, id: &str, volume: f32) {
        if let Some(dev) = self.devices.get_mut(id) {
            let mut cb = dev.lock();
            cb.pos = 0;
            cb.volume = volume;
            cb.playing = true;
        }
    }

    /// Pause without rewinding.
    pub fn pause(&mut self, id: &str) {
        if let Some(dev) = self.devices.get_mut(id) {
            let mut cb = dev.lock();
            cb.playing = false;
        }
    }

    /// Resume from the paused position.
    pub fn resume(&mut self, id: &str) {
        if let Some(dev) = self.devices.get_mut(id) {
            let mut cb = dev.lock();
            cb.playing = true;
        }
    }

    /// Stop and rewind.
    pub fn stop(&mut self, id: &str) {
        if let Some(dev) = self.devices.get_mut(id) {
            let mut cb = dev.lock();
            cb.pos = 0;
            cb.playing = false;
        }
    }
}

fn decode(data: &[u8]) -> Option<AudioBuffer> {
    decode_mp3(data).or_else(|| decode_wav(data))
}

fn decode_mp3(data: &[u8]) -> Option<AudioBuffer> {
    // Use minimp3_sys (raw C bindings) directly to avoid the slice-ring-buffer
    // dependency that minimp3 0.5 carries, which uses SysV IPC and does not
    // compile on wasm32-unknown-emscripten.
    let mut dec: minimp3_sys::mp3dec_t = unsafe { std::mem::zeroed() };
    unsafe { minimp3_sys::mp3dec_init(&mut dec) };

    // MINIMP3_MAX_SAMPLES_PER_FRAME = 1152 * 2 (stereo)
    const MAX_SAMPLES: usize = 1152 * 2;
    let mut pcm = [0i16; MAX_SAMPLES];

    let mut all_samples: Vec<i16> = Vec::new();
    let (mut rate, mut ch) = (44100i32, 2u8);
    let mut pos = 0usize;

    loop {
        if pos >= data.len() {
            break;
        }
        let mut info: minimp3_sys::mp3dec_frame_info_t = unsafe { std::mem::zeroed() };
        let samples_per_ch = unsafe {
            minimp3_sys::mp3dec_decode_frame(
                &mut dec,
                data[pos..].as_ptr(),
                (data.len() - pos) as i32,
                pcm.as_mut_ptr(),
                &mut info,
            )
        };
        let frame_bytes = info.frame_bytes as usize;
        if frame_bytes == 0 {
            break;
        }
        pos += frame_bytes;
        if samples_per_ch > 0 {
            let total = samples_per_ch as usize * info.channels as usize;
            all_samples.extend_from_slice(&pcm[..total]);
            rate = info.hz;
            ch = info.channels as u8;
        }
    }

    if all_samples.is_empty() {
        return None;
    }
    Some(AudioBuffer { samples: all_samples, sample_rate: rate, channels: ch })
}

fn decode_wav(data: &[u8]) -> Option<AudioBuffer> {
    let mut reader = WavReader::new(Cursor::new(data)).ok()?;
    let spec = reader.spec();
    let samples: Vec<i16> = reader.samples::<i16>().filter_map(|s| s.ok()).collect();
    if samples.is_empty() {
        return None;
    }
    Some(AudioBuffer {
        samples,
        sample_rate: spec.sample_rate as i32,
        channels: spec.channels as u8,
    })
}
