use std::time::SystemTime;

use crate::LottieRenderer;

enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

struct Config {
    use_frame_interpolation: bool,
    auto_play: bool,
    _loop: bool,
    speed: f32,
}

pub struct DotLottiePlayer {
    renderer: LottieRenderer,
    playback_state: PlaybackState,
    is_loaded: bool,
    start_time: SystemTime,
    _loop: bool,
    loop_count: u32,
    use_frame_interpolation: bool,
}

impl DotLottiePlayer {
    pub fn new(_loop: bool, use_frame_interpolation: bool) -> Self {
        DotLottiePlayer {
            renderer: LottieRenderer::new(),
            playback_state: PlaybackState::Stopped,
            is_loaded: false,
            start_time: SystemTime::now(),
            _loop,
            loop_count: 0,
            use_frame_interpolation,
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    pub fn is_playing(&self) -> bool {
        match self.playback_state {
            PlaybackState::Playing => true,
            _ => false,
        }
    }

    pub fn is_paused(&self) -> bool {
        match self.playback_state {
            PlaybackState::Paused => true,
            _ => false,
        }
    }

    pub fn is_stopped(&self) -> bool {
        match self.playback_state {
            PlaybackState::Stopped => true,
            _ => false,
        }
    }

    pub fn play(&mut self) -> bool {
        if self.is_loaded && !self.is_playing() {
            self.playback_state = PlaybackState::Playing;
            self.start_time = SystemTime::now();

            true
        } else {
            false
        }
    }

    pub fn pause(&mut self) -> bool {
        if self.is_loaded {
            self.playback_state = PlaybackState::Paused;
            true
        } else {
            false
        }
    }

    pub fn stop(&mut self) -> bool {
        if self.is_loaded {
            self.playback_state = PlaybackState::Stopped;
            self.set_frame(0_f32);

            true
        } else {
            false
        }
    }

    pub fn request_frame(&mut self) -> f32 {
        let current_time = SystemTime::now();

        let elapsed_time = match current_time.duration_since(self.start_time) {
            Ok(n) => n.as_millis(),
            Err(_) => 0,
        } as f32;

        let duration = self.duration() * 1000.0;
        let total_frames = self.total_frames() - 1.0;

        let raw_next_frame = elapsed_time / duration * total_frames;

        let next_frame = if self.use_frame_interpolation {
            raw_next_frame
        } else {
            raw_next_frame.round()
        };

        if next_frame >= total_frames {
            if self._loop {
                self.loop_count += 1;
                self.start_time = SystemTime::now();
                self.set_frame(0_f32);
            } else {
                return total_frames - 1.0;
            }
        }

        next_frame
    }

    pub fn set_frame(&mut self, no: f32) -> bool {
        self.renderer.set_frame(no).is_ok()
    }

    pub fn render(&mut self) -> bool {
        self.renderer.render().is_ok()
    }

    pub fn total_frames(&self) -> f32 {
        self.renderer.total_frames().unwrap_or(0.0)
    }

    pub fn duration(&self) -> f32 {
        self.renderer.duration().unwrap_or(0.0)
    }

    pub fn current_frame(&self) -> f32 {
        self.renderer.current_frame().unwrap_or(0.0)
    }

    pub fn buffer(&self) -> &[u32] {
        &self.renderer.buffer
    }

    pub fn clear(&mut self) -> bool {
        self.renderer.clear(false, true).is_ok()
    }

    pub fn load_animation(&mut self, animation_data: &str, width: u32, height: u32) -> bool {
        let loaded = self
            .renderer
            .load_data(animation_data, width, height, true)
            .is_ok();

        self.is_loaded = loaded;

        loaded
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}
