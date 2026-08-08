use super::MediaPlayer;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 120;
const FPS: f32 = 30.0;
const DURATION: f32 = 2.0;

pub struct SyntheticPlayer {
    pixels: Vec<u8>,
    time: f32,
    filled: Option<u32>,
}

fn frame_color(index: u32) -> (u8, u8, u8) {
    let ramp = index.saturating_mul(8).min(255) as u8;
    (ramp, 255 - ramp, 128)
}

impl SyntheticPlayer {
    pub fn open(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }
        Some(Self {
            pixels: vec![0; (WIDTH * HEIGHT * 4) as usize],
            time: 0.0,
            filled: None,
        })
    }

    fn frame_index(&self) -> u32 {
        (self.time * FPS).round().max(0.0) as u32
    }
}

impl MediaPlayer for SyntheticPlayer {
    fn seek(&mut self, seconds: f32) {
        self.time = seconds.clamp(0.0, DURATION);
        let index = self.frame_index();
        if self.filled == Some(index) {
            return;
        }
        let (r, g, b) = frame_color(index);
        for px in self.pixels.chunks_exact_mut(4) {
            px.copy_from_slice(&[r, g, b, 255]);
        }
        self.filled = Some(index);
    }

    fn info(&self) -> Option<(u32, u32, f32)> {
        Some((WIDTH, HEIGHT, DURATION))
    }

    fn frame(&mut self) -> Option<&[u8]> {
        self.filled?;
        Some(&self.pixels)
    }

    fn time(&self) -> f32 {
        self.time
    }
}
