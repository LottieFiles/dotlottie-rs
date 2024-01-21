use instant::Instant;
use std::sync::{Arc, RwLock};

use crate::lottie_renderer::{LottieRenderer, LottieRendererError};

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

enum Direction {
    Forward,
    Reverse,
}

#[derive(Clone)]
pub struct Config {
    pub mode: Mode,
    pub loop_animation: bool,
    pub speed: f32,
    pub use_frame_interpolation: bool,
    pub autoplay: bool,
    pub segments: Vec<f32>,
    pub background_color: u32,
}

struct DotLottieRuntime {
    renderer: LottieRenderer,
    playback_state: PlaybackState,
    is_loaded: bool,
    start_time: Instant,
    loop_count: u32,
    config: Config,
    observers: Vec<Arc<dyn Observer>>,
    direction: Direction,
}

impl DotLottieRuntime {
    pub fn new(config: Config) -> Self {
        let direction = match config.mode {
            Mode::Forward => Direction::Forward,
            Mode::Reverse => Direction::Reverse,
            Mode::Bounce => Direction::Forward,
            Mode::ReverseBounce => Direction::Reverse,
        };

        DotLottieRuntime {
            renderer: LottieRenderer::new(),
            playback_state: PlaybackState::Stopped,
            is_loaded: false,
            start_time: Instant::now(),
            loop_count: 0,
            config,
            observers: Vec::new(),
            direction,
        }
    }

    pub fn set_background_color(&mut self, hex_color: u32) -> bool {
        self.config.background_color = hex_color;

        self.renderer.set_background_color(hex_color).is_ok()
    }

    fn start_frame(&self) -> f32 {
        if self.config.segments.len() == 2 {
            self.config.segments[0].max(0.0)
        } else {
            0.0
        }
    }

    fn end_frame(&self) -> f32 {
        if self.config.segments.len() == 2 {
            self.config.segments[1].min(self.total_frames())
        } else {
            self.total_frames()
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

            let start_frame = self.start_frame();
            let end_frame = self.end_frame();

            match self.config.mode {
                Mode::Forward | Mode::Bounce => {
                    self.set_frame(start_frame);
                }
                Mode::Reverse | Mode::ReverseBounce => {
                    self.set_frame(end_frame);
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

        let elapsed_time = self.start_time.elapsed().as_secs_f32();

        // the animation total frames
        let total_frames = self.total_frames();

        // the animation duration in seconds
        let duration = self.duration();

        // the animation start & end frames (considering the segments)
        let start_frame = self.start_frame();
        let end_frame = self.end_frame();

        // the effective total frames (considering the segments)
        let effective_total_frames = end_frame - start_frame;

        // the effective duration in milliseconds (considering the segments & speed)
        let effective_duration =
            (duration * effective_total_frames / total_frames) / self.config.speed;

        let raw_next_frame = (elapsed_time / effective_duration) * effective_total_frames;

        // update the next frame based on the direction
        let next_frame = match self.direction {
            Direction::Forward => start_frame + raw_next_frame,
            Direction::Reverse => end_frame - raw_next_frame,
        };

        let next_frame = if self.config.use_frame_interpolation {
            next_frame
        } else {
            next_frame.round()
        };

        // to ensure the next_frame won't go beyond the start & end frames
        let next_frame = next_frame.clamp(start_frame, end_frame);

        let next_frame = match self.config.mode {
            Mode::Forward => self.handle_forward_mode(next_frame, start_frame, end_frame),
            Mode::Reverse => self.handle_reverse_mode(next_frame, start_frame, end_frame),
            Mode::Bounce => self.handle_bounce_mode(next_frame, start_frame, end_frame),
            Mode::ReverseBounce => {
                self.handle_reverse_bounce_mode(next_frame, start_frame, end_frame)
            }
        };

        next_frame
    }

    fn handle_forward_mode(&mut self, next_frame: f32, start_frame: f32, end_frame: f32) -> f32 {
        if next_frame >= end_frame {
            if self.config.loop_animation {
                self.loop_count += 1;
                self.start_time = Instant::now();

                start_frame
            } else {
                end_frame
            }
        } else {
            next_frame
        }
    }

    fn handle_reverse_mode(&mut self, next_frame: f32, start_frame: f32, end_frame: f32) -> f32 {
        if next_frame <= start_frame {
            if self.config.loop_animation {
                self.loop_count += 1;
                self.start_time = Instant::now();

                end_frame
            } else {
                start_frame
            }
        } else {
            next_frame
        }
    }

    fn handle_bounce_mode(&mut self, next_frame: f32, start_frame: f32, end_frame: f32) -> f32 {
        match self.direction {
            Direction::Forward => {
                if next_frame >= end_frame {
                    self.direction = Direction::Reverse;
                    self.start_time = Instant::now();

                    end_frame
                } else {
                    next_frame
                }
            }
            Direction::Reverse => {
                if next_frame <= start_frame {
                    if self.config.loop_animation {
                        self.loop_count += 1;
                        self.direction = Direction::Forward;
                        self.start_time = Instant::now();
                    }

                    start_frame
                } else {
                    next_frame
                }
            }
        }
    }

    fn handle_reverse_bounce_mode(
        &mut self,
        next_frame: f32,
        start_frame: f32,
        end_frame: f32,
    ) -> f32 {
        match self.direction {
            Direction::Reverse => {
                if next_frame <= start_frame {
                    self.direction = Direction::Forward;
                    self.start_time = Instant::now();
                    start_frame
                } else {
                    next_frame
                }
            }
            Direction::Forward => {
                if next_frame >= end_frame {
                    if self.config.loop_animation {
                        self.loop_count += 1;
                        self.direction = Direction::Reverse;
                        self.start_time = Instant::now();
                    }

                    end_frame
                } else {
                    next_frame
                }
            }
        }
    }

    pub fn set_frame(&mut self, no: f32) -> bool {
        self.renderer.set_frame(no).is_ok()
    }

    pub fn render(&mut self) -> bool {
        self.renderer.render().is_ok()
    }

    pub fn total_frames(&self) -> f32 {
        match self.renderer.total_frames() {
            Ok(total_frames) => total_frames - 1.0,
            Err(_) => 0.0,
        }
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
        self.config.speed = if speed < 0.0 { 1.0 } else { speed };
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

    fn load_animation_common<F>(&mut self, loader: F, width: u32, height: u32) -> bool
    where
        F: FnOnce(&mut LottieRenderer, u32, u32) -> Result<(), LottieRendererError>,
    {
        let loaded = loader(&mut self.renderer, width, height).is_ok()
            && self
                .renderer
                .set_background_color(self.config.background_color)
                .is_ok();

        self.is_loaded = loaded;

        let start_frame = self.start_frame();
        let end_frame = self.end_frame();

        match self.config.mode {
            Mode::Forward | Mode::Bounce => {
                self.set_frame(start_frame);
                self.direction = Direction::Forward;
            }
            Mode::Reverse | Mode::ReverseBounce => {
                self.set_frame(end_frame);
                self.direction = Direction::Reverse;
            }
        }

        if self.config.autoplay && loaded {
            self.play();
        }

        loaded
    }

    pub fn load_animation_data(&mut self, animation_data: &str, width: u32, height: u32) -> bool {
        self.load_animation_common(
            |renderer, w, h| renderer.load_data(animation_data, w, h, false),
            width,
            height,
        )
    }

    pub fn load_animation_path(&mut self, animation_path: &str, width: u32, height: u32) -> bool {
        self.load_animation_common(
            |renderer, w, h| renderer.load_path(animation_path, w, h),
            width,
            height,
        )
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

    pub fn set_background_color(&self, hex_color: u32) -> bool {
        self.runtime
            .write()
            .unwrap()
            .set_background_color(hex_color)
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}
