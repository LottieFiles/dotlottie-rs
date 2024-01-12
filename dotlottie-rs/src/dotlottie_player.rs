use std::time::SystemTime;

use crate::LottieRenderer;

enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Clone, Copy)]
pub enum Mode {
    Forward,
    Reverse,
}

pub struct Config {
    mode: Mode,
    _loop: bool,
    speed: f32,
    use_frame_interpolation: bool,
    autoplay: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            autoplay: false,
            mode: Mode::Forward,
            _loop: false,
            speed: 1.0,
            use_frame_interpolation: true,
        }
    }

    pub fn mode(&mut self, mode: Mode) -> &mut Self {
        self.mode = mode;
        self
    }

    pub fn _loop(&mut self, _loop: bool) -> &mut Self {
        self._loop = _loop;
        self
    }

    pub fn use_frame_interpolation(&mut self, use_frame_interpolation: bool) -> &mut Self {
        self.use_frame_interpolation = use_frame_interpolation;
        self
    }

    pub fn speed(&mut self, speed: f32) -> &mut Self {
        self.speed = speed;
        self
    }

    pub fn autoplay(&mut self, autoplay: bool) -> &mut Self {
        self.autoplay = autoplay;
        self
    }

    pub fn build(&self) -> Self {
        Config {
            autoplay: self.autoplay,
            mode: self.mode,
            _loop: self._loop,
            speed: self.speed,
            use_frame_interpolation: self.use_frame_interpolation,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

pub struct DotLottiePlayer {
    renderer: LottieRenderer,
    playback_state: PlaybackState,
    is_loaded: bool,
    start_time: SystemTime,
    loop_count: u32,
    config: Config,
}

impl DotLottiePlayer {
    pub fn new(config: Config) -> Self {
        DotLottiePlayer {
            renderer: LottieRenderer::new(),
            playback_state: PlaybackState::Stopped,
            is_loaded: false,
            start_time: SystemTime::now(),
            loop_count: 0,
            config,
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
            match self.config.mode {
                Mode::Forward => {
                    self.set_frame(0_f32);
                }
                Mode::Reverse => {
                    self.set_frame(self.total_frames());
                }
            }

            true
        } else {
            false
        }
    }

    pub fn request_frame(&mut self) -> f32 {
        if !self.is_loaded || !self.is_playing() {
            return self.current_frame();
        }

        let current_time = SystemTime::now();

        let elapsed_time = match current_time.duration_since(self.start_time) {
            Ok(n) => n.as_millis(),
            Err(_) => 0,
        } as f32;

        let duration = (self.duration() * 1000.0) / self.config.speed as f32;
        let total_frames = self.total_frames() - 1.0;

        let raw_next_frame = elapsed_time / duration * total_frames;

        let next_frame = if self.config.use_frame_interpolation {
            raw_next_frame
        } else {
            raw_next_frame.round()
        };

        let next_frame = match self.config.mode {
            Mode::Forward => next_frame,
            Mode::Reverse => total_frames - next_frame,
        };

        let next_frame = match self.config.mode {
            Mode::Forward => {
                if next_frame >= total_frames {
                    if self.config._loop {
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
                    if self.config._loop {
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

    pub fn loop_count(&self) -> u32 {
        self.loop_count
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.config.speed = if speed < 0.0 { 0.0 } else { speed };
    }

    pub fn speed(&self) -> f32 {
        self.config.speed
    }

    pub fn buffer(&self) -> &[u32] {
        &self.renderer.buffer
    }

    pub fn clear(&mut self, free: bool) -> bool {
        self.renderer.clear(free).is_ok()
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    pub fn load_animation_data(&mut self, animation_data: &str, width: u32, height: u32) -> bool {
        let loaded = self
            .renderer
            .load_data(animation_data, width, height, true)
            .is_ok();

        self.is_loaded = loaded;

        let total_frames = self.total_frames();

        match self.config.mode {
            Mode::Forward => {
                self.set_frame(0_f32);
            }
            Mode::Reverse => {
                self.set_frame(total_frames);
            }
        }

        if self.config.autoplay && loaded {
            return self.play();
        }

        loaded
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}
