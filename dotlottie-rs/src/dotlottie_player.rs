use instant::Instant;
use std::sync::{Arc, RwLock};

use crate::LottieRenderer;

pub trait Observer: Send + Sync {
    fn on_load(&self);
    fn on_play(&self);
    fn on_pause(&self);
    fn on_stop(&self);
    fn on_frame(&self, frame_no: f32);
    fn on_render(&self, frame_no: f32);
    fn on_loop(&self, loop_count: u32);
}

pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Clone, Copy)]
pub enum Mode {
    Forward,
    Reverse,
    Bounce,
    ReverseBounce,
}

#[derive(Clone, Copy)]
pub struct Config {
    pub mode: Mode,
    pub loop_animation: bool,
    pub speed: f32,
    pub use_frame_interpolation: bool,
    pub autoplay: bool,
}

struct DotLottieRuntime {
    renderer: LottieRenderer,
    playback_state: PlaybackState,
    is_loaded: bool,
    start_time: Instant,
    loop_count: u32,
    config: Config,
    observers: Vec<Arc<dyn Observer>>,
}

impl DotLottieRuntime {
    pub fn new(config: Config) -> Self {
        DotLottieRuntime {
            renderer: LottieRenderer::new(),
            playback_state: PlaybackState::Stopped,
            is_loaded: false,
            start_time: Instant::now(),
            loop_count: 0,
            config,
            observers: Vec::new(),
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
            self.start_time = Instant::now();

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
                _ => {}
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

        let elapsed_time = self.start_time.elapsed().as_secs_f32();

        let duration = self.duration() / self.config.speed;
        let total_frames = self.total_frames() - 1.0;

        let raw_next_frame = (elapsed_time / duration) * total_frames;

        let next_frame = if self.config.use_frame_interpolation {
            raw_next_frame
        } else {
            raw_next_frame.round()
        };

        let next_frame = match self.config.mode {
            Mode::Forward => next_frame,
            Mode::Reverse => total_frames - next_frame,
            _ => next_frame,
        };

        let next_frame = match self.config.mode {
            Mode::Forward => {
                if next_frame >= total_frames {
                    if self.config.loop_animation {
                        self.loop_count += 1;
                        self.start_time = Instant::now();
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
                    if self.config.loop_animation {
                        self.loop_count += 1;
                        self.start_time = Instant::now();
                        total_frames
                    } else {
                        0.0
                    }
                } else {
                    next_frame
                }
            }
            _ => next_frame,
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

    pub fn clear(&mut self) {
        self.renderer.clear()
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    pub fn load_animation_path(&mut self, animation_path: &str, width: u32, height: u32) -> bool {
        let loaded = self
            .renderer
            .load_path(animation_path, width, height)
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
            _ => {}
        }

        if self.config.autoplay && loaded {
            return self.play();
        }

        loaded
    }

    pub fn load_animation_data(&mut self, animation_data: &str, width: u32, height: u32) -> bool {
        let loaded = self
            .renderer
            .load_data(animation_data, width, height, false)
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
            _ => {}
        }

        if self.config.autoplay && loaded {
            return self.play();
        }

        loaded
    }

    pub fn resize(&mut self, width: u32, height: u32) -> bool {
        self.renderer.resize(width, height).is_ok()
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }

    pub fn subscribe(&mut self, observer: Arc<dyn Observer>) {
        self.observers.push(observer);
    }
}

pub struct DotLottiePlayer {
    runtime: RwLock<DotLottieRuntime>,
}

impl DotLottiePlayer {
    pub fn new(config: Config) -> Self {
        DotLottiePlayer {
            runtime: RwLock::new(DotLottieRuntime::new(config)),
        }
    }

    pub fn load_animation_data(&self, animation_data: &str, width: u32, height: u32) -> bool {
        self.runtime
            .write()
            .unwrap()
            .load_animation_data(animation_data, width, height)
    }

    pub fn load_animation_path(&self, animation_path: &str, width: u32, height: u32) -> bool {
        self.runtime
            .write()
            .unwrap()
            .load_animation_path(animation_path, width, height)
    }

    pub fn buffer_ptr(&self) -> u64 {
        self.runtime.read().unwrap().buffer().as_ptr().cast::<u32>() as u64
    }

    pub fn buffer_len(&self) -> u64 {
        self.runtime.read().unwrap().buffer().len() as u64
    }

    pub fn clear(&self) {
        self.runtime.write().unwrap().clear();
    }

    pub fn set_config(&self, config: Config) {
        self.runtime.write().unwrap().set_config(config);
    }

    pub fn set_speed(&self, speed: f32) {
        self.runtime.write().unwrap().set_speed(speed);
    }

    pub fn speed(&self) -> f32 {
        self.runtime.read().unwrap().speed()
    }

    pub fn total_frames(&self) -> f32 {
        self.runtime.read().unwrap().total_frames()
    }

    pub fn duration(&self) -> f32 {
        self.runtime.read().unwrap().duration()
    }

    pub fn current_frame(&self) -> f32 {
        self.runtime.read().unwrap().current_frame()
    }

    pub fn loop_count(&self) -> u32 {
        self.runtime.read().unwrap().loop_count()
    }

    pub fn is_loaded(&self) -> bool {
        self.runtime.read().unwrap().is_loaded()
    }

    pub fn is_playing(&self) -> bool {
        self.runtime.read().unwrap().is_playing()
    }

    pub fn is_paused(&self) -> bool {
        self.runtime.read().unwrap().is_paused()
    }

    pub fn is_stopped(&self) -> bool {
        self.runtime.read().unwrap().is_stopped()
    }

    pub fn play(&self) -> bool {
        self.runtime.write().unwrap().play()
    }

    pub fn pause(&self) -> bool {
        self.runtime.write().unwrap().pause()
    }

    pub fn stop(&self) -> bool {
        self.runtime.write().unwrap().stop()
    }

    pub fn request_frame(&self) -> f32 {
        self.runtime.write().unwrap().request_frame()
    }

    pub fn set_frame(&self, no: f32) -> bool {
        self.runtime.write().unwrap().set_frame(no)
    }

    pub fn render(&self) -> bool {
        self.runtime.write().unwrap().render()
    }

    pub fn resize(&self, width: u32, height: u32) -> bool {
        self.runtime.write().unwrap().resize(width, height)
    }

    pub fn config(&self) -> Config {
        self.runtime.read().unwrap().config()
    }

    pub fn subscribe(&self, observer: Arc<dyn Observer>) {
        self.runtime.write().unwrap().subscribe(observer);
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}
