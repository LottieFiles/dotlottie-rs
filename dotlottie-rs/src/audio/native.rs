//! Native audio through rodio. A playback rate change shifts pitch.

use rodio::{mixer::Mixer, Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use super::{AudioClip, Cue, Gain};

/// The audio device. It opens on the first clip and stays open after that.
#[derive(Default)]
pub struct Output(Option<OutputStream>);

impl Output {
    /// `None` if no audio device is available.
    fn mixer(&mut self) -> Option<&Mixer> {
        if self.0.is_none() {
            self.0 = OutputStreamBuilder::open_default_stream()
                .ok()
                .map(|mut stream| {
                    stream.log_on_drop(false);
                    stream
                });
        }
        self.0.as_ref().map(OutputStream::mixer)
    }
}

pub struct Clip(Sink);

impl AudioClip for Clip {
    type Output = Output;

    fn start(output: &mut Output, data: &Arc<[u8]>, cue: Cue) -> Option<Self> {
        let source = decode(data.clone())?;
        let sink = Sink::connect_new(output.mixer()?);
        sink.set_volume(volume(cue.gain));
        sink.set_speed(cue.rate);
        // skip_duration approximates a mid-clip start.
        sink.append(source.skip_duration(Duration::from_secs_f32(cue.offset.max(0.0))));
        Some(Clip(sink))
    }

    fn pause(&self) {
        self.0.pause();
    }

    fn resume(&self) {
        self.0.play();
    }

    fn set_gain(&self, gain: Gain) {
        self.0.set_volume(volume(gain));
    }

    fn set_rate(&self, rate: f32) {
        self.0.set_speed(rate);
    }
}

fn volume(gain: Gain) -> f32 {
    gain.layer * gain.global
}

/// Decode `data` in any enabled format. The byte length lets symphonia read MP4
/// files whose index follows the media data.
fn decode(data: Arc<[u8]>) -> Option<Decoder<Cursor<Arc<[u8]>>>> {
    let len = data.len() as u64;
    Decoder::builder()
        .with_data(Cursor::new(data))
        .with_byte_len(len)
        .with_seekable(true)
        .build()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::decode;

    fn decodes(bytes: &'static [u8]) -> bool {
        decode(bytes.into()).is_some_and(|source| source.count() > 0)
    }

    #[test]
    fn decodes_mp3() {
        assert!(decodes(include_bytes!("../../assets/audio/tone.mp3")));
    }

    #[cfg(feature = "audio-wav")]
    #[test]
    fn decodes_wav() {
        assert!(decodes(include_bytes!("../../assets/audio/tone.wav")));
    }

    #[cfg(feature = "audio-flac")]
    #[test]
    fn decodes_flac() {
        assert!(decodes(include_bytes!("../../assets/audio/tone.flac")));
    }

    #[cfg(feature = "audio-ogg")]
    #[test]
    fn decodes_ogg() {
        assert!(decodes(include_bytes!("../../assets/audio/tone.ogg")));
    }

    #[cfg(feature = "audio-m4a")]
    #[test]
    fn decodes_m4a() {
        assert!(decodes(include_bytes!("../../assets/audio/tone.m4a")));
    }
}
