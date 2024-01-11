use std::time::SystemTime;

use crate::LottieRenderer;

enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

pub enum Mode {
    Forward,
    Reverse,
}

pub struct DotLottiePlayer {
    renderer: LottieRenderer,
    playback_state: PlaybackState,
    is_loaded: bool,
    start_time: SystemTime,
    _loop: bool,
    loop_count: u32,
    use_frame_interpolation: bool,
    speed: f32,
    mode: Mode,
}

impl DotLottiePlayer {
    pub fn new(mode: Mode, _loop: bool, use_frame_interpolation: bool, speed: f32) -> Self {
        DotLottiePlayer {
            renderer: LottieRenderer::new(),
            playback_state: PlaybackState::Stopped,
            is_loaded: false,
            start_time: SystemTime::now(),
            _loop,
            loop_count: 0,
            use_frame_interpolation,
            speed: if speed < 0.0 { 0.0 } else { speed },
            mode,
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

        let duration = (self.duration() * 1000.0) / self.speed as f32;
        let total_frames = self.total_frames() - 1.0;

        let raw_next_frame = elapsed_time / duration * total_frames;

        let next_frame = if self.use_frame_interpolation {
            raw_next_frame
        } else {
            raw_next_frame.round()
        };

        let next_frame = match self.mode {
            Mode::Forward => next_frame,
            Mode::Reverse => total_frames - next_frame,
        };

        let next_frame = match self.mode {
            Mode::Forward => {
                if next_frame >= total_frames {
                    if self._loop {
                        self.loop_count += 1;
                        self.start_time = SystemTime::now();
                        0.0
                    } else {
                        total_frames
                    }
                } else {
                    next_frame
                }
            }
            Mode::Reverse => {
                if next_frame <= 0.0 {
                    if self._loop {
                        self.loop_count += 1;
                        self.start_time = SystemTime::now();
                        total_frames
                    } else {
                        0.0
                    }
                } else {
                    next_frame
                }
            }
        };

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

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = if speed < 0.0 { 0.0 } else { speed };
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn buffer(&self) -> &[u32] {
        &self.renderer.buffer
    }

    pub fn clear(&mut self) -> bool {
        self.renderer.clear(false, true).is_ok()
    }

    pub fn load_animation_data(&mut self, animation_data: &str, width: u32, height: u32) -> bool {
        let loaded = self
            .renderer
            .load_data(animation_data, width, height, true)
            .is_ok();

        self.is_loaded = loaded;

        let total_frames = self.total_frames();

        match self.mode {
            Mode::Forward => {
                self.set_frame(0_f32);
            }
            Mode::Reverse => {
                self.set_frame(total_frames);
            }
        }

        loaded
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}
