use rustc_hash::FxHashMap;
use std::sync::Arc;

use crate::renderer::{AudioEvent, AudioSource};

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
use native::Clip as PlatformClip;

#[cfg(any(test, target_arch = "wasm32"))]
mod web;
#[cfg(target_arch = "wasm32")]
use web::Clip as PlatformClip;

/// One playing clip on a platform. Dropping it stops playback.
pub trait AudioClip: Sized {
    /// What clips play through, such as the audio device. Created once per
    /// manager.
    type Device: Default;

    /// `None` if the platform can't play `data`, such as when no audio device
    /// is available.
    fn start(device: &mut Self::Device, data: &Arc<[u8]>, cue: Cue) -> Option<Self>;
    fn pause(&self);
    fn resume(&self);
    fn set_gain(&self, gain: Gain);
    fn set_rate(&self, rate: f32);
}

/// A clip's two volume factors. They stay separate because web mutes when the
/// global factor is 0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gain {
    pub layer: f32,
    pub global: f32,
}

/// Where and how a clip starts.
#[derive(Debug, Clone, Copy)]
pub struct Cue {
    /// Seek position into the clip in seconds.
    pub offset: f32,
    pub gain: Gain,
    pub rate: f32,
}

/// Settings shared by every clip.
#[derive(Debug, Clone, Copy)]
struct Global {
    /// Global volume in [0.0, 1.0].
    volume: f32,
    rate: f32,
}

enum Playback<C> {
    Idle,
    /// Handed to the platform. `None` when the platform couldn't play it, so a
    /// missing audio device doesn't retry every frame.
    Started(Option<C>),
}

/// An audio layer inside its playback range. It stays in the map while paused
/// so resume can replay it.
struct Layer<C> {
    data: Arc<[u8]>,
    /// Seek position into the clip in seconds.
    offset: f32,
    /// Animation time, in seconds, of the clip's in-point. `sync` recomputes
    /// `offset` from it when the playhead jumps.
    clip_start: f32,
    /// Per-layer volume in [0.0, 1.0].
    volume: f32,
    playback: Playback<C>,
}

impl<C: AudioClip> Layer<C> {
    fn start(&mut self, device: &mut C::Device, global: Global) {
        // Drop the old clip so it stops before the new one plays.
        self.playback = Playback::Idle;
        let cue = Cue {
            offset: self.offset,
            gain: self.gain(global),
            rate: global.rate,
        };
        self.playback = Playback::Started(C::start(device, &self.data, cue));
    }

    fn stop(&mut self) {
        self.playback = Playback::Idle;
    }

    fn is_started(&self) -> bool {
        matches!(self.playback, Playback::Started(_))
    }

    fn clip(&self) -> Option<&C> {
        match &self.playback {
            Playback::Started(clip) => clip.as_ref(),
            Playback::Idle => None,
        }
    }

    fn refresh_gain(&self, global: Global) {
        if let Some(clip) = self.clip() {
            clip.set_gain(self.gain(global));
        }
    }

    fn gain(&self, global: Global) -> Gain {
        Gain {
            layer: self.volume,
            global: global.volume,
        }
    }
}

/// The playhead and the renderer's clip offset are rounded separately, so a
/// clip anchored at t=0 can land a few ulps later. Treat that as on time.
const START_SLACK: f32 = 1e-3;

/// Resolves an external audio `src` (normalized packaged path or URL) to bytes.
pub type AudioSourceLookup = Box<dyn Fn(&str) -> Option<Arc<[u8]>>>;

/// Frame-synchronised audio playback driven by the renderer's audio resolver.
/// Plays through rodio on native targets and `HtmlAudioElement` on wasm.
pub struct AudioManager<C: AudioClip = PlatformClip> {
    /// Resolves external audio srcs; embedded audio bytes arrive in the event.
    lookup: AudioSourceLookup,
    layers: FxHashMap<String, Layer<C>>,
    // Declared after `layers` so clips drop before their device.
    device: C::Device,
    global: Global,
    playing: bool,
    /// Playhead position in animation-time seconds.
    playhead: f32,
    /// False while the playhead runs backward, which suspends audio.
    forward: bool,
}

impl<C: AudioClip> AudioManager<C> {
    /// The audio device opens on first playback.
    pub fn new(lookup: AudioSourceLookup) -> Self {
        AudioManager {
            lookup,
            layers: FxHashMap::default(),
            device: C::Device::default(),
            global: Global {
                volume: 1.0,
                rate: 1.0,
            },
            playing: false,
            playhead: 0.0,
            forward: true,
        }
    }

    /// Update the playhead position (animation-time seconds). Call it before
    /// the update that fires resolver events, so new activations anchor to
    /// this position.
    pub fn set_playhead(&mut self, secs: f32) {
        self.playhead = secs;
    }

    /// Handle an audio layer state change reported by the renderer.
    pub fn on_audio(&mut self, event: AudioEvent) {
        let key = match &event.source {
            AudioSource::External(src) => normalize_src(src),
            // Key embedded layers by their (stable) source pointer.
            AudioSource::Embedded { bytes, .. } => format!("emb:{:p}", bytes.as_ptr()),
        };

        if !event.active {
            self.layers.remove(&key);
            return;
        }

        let volume = event.volume / 100.0;
        let clip_start = self.playhead - event.offset;

        // Volume keyframes re-fire the resolver; update without restarting.
        if let Some(layer) = self.layers.get_mut(&key) {
            layer.volume = volume;
            layer.offset = event.offset;
            layer.clip_start = clip_start;
            layer.refresh_gain(self.global);
            return;
        }

        let data = match event.source {
            AudioSource::External(_) => match (self.lookup)(&key) {
                Some(data) => data,
                None => return,
            },
            AudioSource::Embedded { bytes, .. } => Arc::from(bytes),
        };

        let mut layer = Layer {
            data,
            offset: event.offset,
            clip_start,
            volume,
            playback: Playback::Idle,
        };
        if self.audible() {
            layer.start(&mut self.device, self.global);
        }
        self.layers.insert(key, layer);
    }

    /// Match the player's playback state. Pauses or resumes clips, and starts
    /// any layers that became active while paused.
    pub fn set_playing(&mut self, playing: bool) {
        if self.playing == playing {
            return;
        }
        self.playing = playing;

        if !playing {
            self.clips().for_each(C::pause);
            return;
        }
        if !self.forward {
            return;
        }
        self.clips().for_each(C::resume);
        for layer in self.layers.values_mut() {
            let offset = self.playhead - layer.clip_start;
            if !layer.is_started() && offset >= -START_SLACK {
                layer.offset = offset.max(0.0);
                layer.start(&mut self.device, self.global);
            }
        }
    }

    /// Recompute each clip's position from the playhead after a seek, a
    /// segment start or a loop wrap.
    pub fn sync(&mut self) {
        let audible = self.audible();
        for layer in self.layers.values_mut() {
            let offset = self.playhead - layer.clip_start;
            layer.offset = offset.max(0.0);
            if audible && offset >= -START_SLACK {
                layer.start(&mut self.device, self.global);
            } else {
                layer.stop();
            }
        }
    }

    /// Set the playback rate multiplier, applied to live and future clips.
    pub fn set_rate(&mut self, rate: f32) {
        if rate.is_nan() || rate <= 0.0 {
            return;
        }
        self.global.rate = rate;
        self.clips().for_each(|clip| clip.set_rate(rate));
    }

    /// Suspend audio while the playhead runs backward, and resync when it runs
    /// forward again.
    pub fn set_direction(&mut self, forward: bool) {
        if self.forward == forward {
            return;
        }
        self.forward = forward;
        if forward {
            self.sync();
        } else {
            self.stop_all();
        }
    }

    /// Stop all clips but keep the layers, so `set_playing(true)` can replay
    /// them.
    pub fn stop(&mut self) {
        self.playing = false;
        self.stop_all();
    }

    fn stop_all(&mut self) {
        self.layers.values_mut().for_each(Layer::stop);
    }

    /// Set the global volume multiplier (clamped to [0.0, 1.0], NaN counts as
    /// 0).
    pub fn set_volume(&mut self, volume: f32) {
        self.global.volume = if volume > 0.0 { volume.min(1.0) } else { 0.0 };
        for layer in self.layers.values() {
            layer.refresh_gain(self.global);
        }
    }

    pub fn volume(&self) -> f32 {
        self.global.volume
    }

    fn audible(&self) -> bool {
        self.playing && self.forward
    }

    fn clips(&self) -> impl Iterator<Item = &C> {
        self.layers.values().filter_map(Layer::clip)
    }

    /// Number of audio layers currently within their playback range.
    #[cfg(test)]
    pub(crate) fn active_layer_count(&self) -> usize {
        self.layers.len()
    }

    #[cfg(test)]
    pub(crate) fn layer_offsets(&self) -> Vec<f32> {
        self.layers.values().map(|layer| layer.offset).collect()
    }

    /// Number of active layers whose clip has been handed to the platform.
    #[cfg(test)]
    pub(crate) fn started_layer_count(&self) -> usize {
        self.layers
            .values()
            .filter(|layer| layer.is_started())
            .count()
    }

    #[cfg(test)]
    pub(crate) fn rate(&self) -> f32 {
        self.global.rate
    }

    #[cfg(test)]
    pub(crate) fn is_forward(&self) -> bool {
        self.forward
    }
}

/// Whether `src` names an audio format this build plays: MP3 always, the rest
/// behind their `audio-*` features.
pub(crate) fn is_supported_src(src: &str) -> bool {
    let Some((_, ext)) = src.rsplit_once('.') else {
        return false;
    };
    match ext.to_ascii_lowercase().as_str() {
        "mp3" => true,
        #[cfg(feature = "audio-wav")]
        "wav" => true,
        #[cfg(feature = "audio-flac")]
        "flac" => true,
        #[cfg(feature = "audio-ogg")]
        "ogg" => true,
        #[cfg(feature = "audio-m4a")]
        "m4a" => true,
        _ => false,
    }
}

/// Normalize a resolver `src` into a lookup key by dropping the leading slashes
/// the renderer adds to packaged paths, so `//u/clip.mp3` becomes `u/clip.mp3`.
fn normalize_src(src: &str) -> String {
    src.trim_start_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::{is_supported_src, web::mime_type, AudioClip, AudioManager, Cue, Gain};
    use crate::renderer::{AudioEvent, AudioSource};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Arc;

    const TONES: [(&str, &[u8], &str, bool); 5] = [
        (
            "mp3",
            include_bytes!("../../assets/audio/tone.mp3"),
            "audio/mpeg",
            true,
        ),
        (
            "wav",
            include_bytes!("../../assets/audio/tone.wav"),
            "audio/wav",
            cfg!(feature = "audio-wav"),
        ),
        (
            "flac",
            include_bytes!("../../assets/audio/tone.flac"),
            "audio/flac",
            cfg!(feature = "audio-flac"),
        ),
        (
            "ogg",
            include_bytes!("../../assets/audio/tone.ogg"),
            "audio/ogg",
            cfg!(feature = "audio-ogg"),
        ),
        (
            "m4a",
            include_bytes!("../../assets/audio/tone.m4a"),
            "audio/mp4",
            cfg!(feature = "audio-m4a"),
        ),
    ];

    #[test]
    fn supports_only_enabled_formats() {
        for (ext, _, _, enabled) in TONES {
            assert_eq!(is_supported_src(&format!("u/tone.{ext}")), enabled, "{ext}");
            assert_eq!(
                is_supported_src(&format!("u/TONE.{}", ext.to_uppercase())),
                enabled,
                "{ext}"
            );
        }
        assert!(!is_supported_src("u/tone"));
        assert!(!is_supported_src("u/tone.aac"));
    }

    #[test]
    fn sniffs_enabled_containers_and_defaults_to_mpeg() {
        for (ext, bytes, mime, enabled) in TONES {
            let expected = if enabled { mime } else { "audio/mpeg" };
            assert_eq!(mime_type(bytes), expected, "{ext}");
        }
        assert_eq!(mime_type(b"RIFF"), "audio/mpeg");
    }

    #[derive(Debug, PartialEq)]
    enum Call {
        Start(Gain),
        Gain(Gain),
        Drop,
    }

    #[derive(Default)]
    struct FakeDevice {
        calls: Rc<RefCell<Vec<Call>>>,
        no_device: bool,
    }

    struct FakeClip(Rc<RefCell<Vec<Call>>>);

    impl AudioClip for FakeClip {
        type Device = FakeDevice;

        fn start(device: &mut FakeDevice, _: &Arc<[u8]>, cue: Cue) -> Option<Self> {
            if device.no_device {
                return None;
            }
            device.calls.borrow_mut().push(Call::Start(cue.gain));
            Some(FakeClip(device.calls.clone()))
        }
        fn pause(&self) {}
        fn resume(&self) {}
        fn set_gain(&self, gain: Gain) {
            self.0.borrow_mut().push(Call::Gain(gain));
        }
        fn set_rate(&self, _: f32) {}
    }

    impl Drop for FakeClip {
        fn drop(&mut self) {
            self.0.borrow_mut().push(Call::Drop);
        }
    }

    const BYTES: &[u8] = b"clip";

    fn playing_manager() -> AudioManager<FakeClip> {
        let mut am = AudioManager::<FakeClip>::new(Box::new(|_| None));
        am.set_playing(true);
        am.on_audio(AudioEvent {
            source: AudioSource::Embedded {
                bytes: BYTES,
                mime: None,
            },
            offset: 0.0,
            volume: 50.0,
            active: true,
        });
        am
    }

    fn calls(am: &AudioManager<FakeClip>) -> std::cell::Ref<'_, Vec<Call>> {
        am.device.calls.borrow()
    }

    #[test]
    fn restart_drops_the_old_clip_before_starting_the_new_one() {
        let mut am = playing_manager();
        am.sync();
        let gain = Gain {
            layer: 0.5,
            global: 1.0,
        };
        assert_eq!(
            *calls(&am),
            [Call::Start(gain), Call::Drop, Call::Start(gain)]
        );
    }

    #[test]
    fn global_volume_reaches_live_clips_and_new_ones() {
        let mut am = playing_manager();
        am.set_volume(0.0);
        am.sync();
        let muted = Gain {
            layer: 0.5,
            global: 0.0,
        };
        assert_eq!(
            calls(&am)[1..],
            [Call::Gain(muted), Call::Drop, Call::Start(muted)]
        );
    }

    #[test]
    fn a_clip_without_a_device_still_counts_as_started() {
        let mut am = AudioManager::<FakeClip>::new(Box::new(|_| None));
        am.device.no_device = true;
        am.set_playing(true);
        am.on_audio(AudioEvent {
            source: AudioSource::Embedded {
                bytes: BYTES,
                mime: None,
            },
            offset: 0.0,
            volume: 100.0,
            active: true,
        });
        assert_eq!(am.started_layer_count(), 1);
        assert!(calls(&am).is_empty());
    }
}
