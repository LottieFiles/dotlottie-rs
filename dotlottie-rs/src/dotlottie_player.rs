use instant::{Duration, Instant};
use std::{
    fs,
    sync::{Arc, RwLock},
};

use dotlottie_fms::{DotLottieError, DotLottieManager, Manifest, ManifestAnimation};

use crate::{
    extract_markers,
    lottie_renderer::{LottieRenderer, LottieRendererError},
    Marker, MarkersMap,
};

pub trait Observer: Send + Sync {
    fn on_load(&self);
    fn on_load_error(&self);
    fn on_play(&self);
    fn on_pause(&self);
    fn on_stop(&self);
    fn on_frame(&self, frame_no: f32);
    fn on_render(&self, frame_no: f32);
    fn on_loop(&self, loop_count: u32);
    fn on_complete(&self);
}

pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Forward,
    Reverse,
    Bounce,
    ReverseBounce,
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Forward,
    Reverse,
}

impl Direction {
    fn flip(&self) -> Self {
        match self {
            Direction::Forward => Direction::Reverse,
            Direction::Reverse => Direction::Forward,
        }
    }
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
    pub marker: String,
}

struct DotLottieRuntime {
    renderer: LottieRenderer,
    playback_state: PlaybackState,
    is_loaded: bool,
    start_time: Instant,
    loop_count: u32,
    config: Config,
    dotlottie_manager: DotLottieManager,
    direction: Direction,
    markers: MarkersMap,
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
            dotlottie_manager: DotLottieManager::new(None).unwrap(),
            direction,
            markers: MarkersMap::new(),
        }
    }

    pub fn markers(&self) -> Vec<Marker> {
        self.markers
            .iter()
            .map(|(name, (time, duration))| Marker {
                name: name.to_string(),
                time: *time,
                duration: *duration,
            })
            .collect()
    }

    fn start_frame(&self) -> f32 {
        let start_frame: f32 = {
            if !self.config.marker.is_empty() {
                if let Some((time, _)) = self.markers.get(&self.config.marker) {
                    return *time;
                }
            }

            if self.config.segments.len() == 2 {
                return self.config.segments[0];
            }

            0.0
        };

        start_frame.clamp(0.0, self.total_frames())
    }

    fn end_frame(&self) -> f32 {
        let end_frame: f32 =
            {
                if !self.config.marker.is_empty() {
                    if let Some((time, duration)) = self.markers.get(&self.config.marker) {
                        return time + duration;
                    }
                }

                if self.config.segments.len() == 2 {
                    return self.config.segments[1];
                }

                self.total_frames()
            };

        end_frame.clamp(0.0, self.total_frames())
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
            if self.is_paused() {
                self.update_start_time_for_frame(self.current_frame());
            } else {
                self.start_time = Instant::now();
                match self.config.mode {
                    Mode::Forward | Mode::Bounce => {
                        self.set_frame(self.start_frame());
                        self.direction = Direction::Forward;
                    }
                    Mode::Reverse | Mode::ReverseBounce => {
                        self.set_frame(self.end_frame());
                        self.direction = Direction::Reverse;
                    }
                }
            }

            self.playback_state = PlaybackState::Playing;

            true
        } else {
            false
        }
    }

    pub fn pause(&mut self) -> bool {
        if self.is_loaded && self.is_playing() {
            self.playback_state = PlaybackState::Paused;

            true
        } else {
            false
        }
    }

    pub fn stop(&mut self) -> bool {
        if self.is_loaded && !self.is_stopped() {
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

    pub fn manifest(&self) -> Option<Manifest> {
        self.dotlottie_manager.manifest()
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

        let next_frame =
            if self.config.use_frame_interpolation {
                next_frame
            } else {
                next_frame.round()
            };

        // to ensure the next_frame won't go beyond the start & end frames
        let next_frame = next_frame.clamp(start_frame, end_frame);

        let next_frame = match self.config.mode {
            Mode::Forward => self.handle_forward_mode(next_frame, end_frame),
            Mode::Reverse => self.handle_reverse_mode(next_frame, start_frame),
            Mode::Bounce => self.handle_bounce_mode(next_frame, start_frame, end_frame),
            Mode::ReverseBounce => {
                self.handle_reverse_bounce_mode(next_frame, start_frame, end_frame)
            }
        };

        next_frame
    }

    fn handle_forward_mode(&mut self, next_frame: f32, end_frame: f32) -> f32 {
        if next_frame >= end_frame {
            if self.config.loop_animation {
                self.loop_count += 1;
                self.start_time = Instant::now();
            }

            end_frame
        } else {
            next_frame
        }
    }

    fn handle_reverse_mode(&mut self, next_frame: f32, start_frame: f32) -> f32 {
        if next_frame <= start_frame {
            if self.config.loop_animation {
                self.loop_count += 1;
                self.start_time = Instant::now();
            }

            start_frame
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

    fn update_start_time_for_frame(&mut self, frame_no: f32) {
        let start_frame = self.start_frame();
        let end_frame = self.end_frame();

        let total_frames = self.total_frames();
        let duration = self.duration();
        let effective_total_frames = end_frame - start_frame;

        if duration.is_finite() && duration > 0.0 && self.config.speed > 0.0 {
            let effective_duration =
                (duration * effective_total_frames / total_frames) / self.config.speed;

            let frame_duration = effective_duration / effective_total_frames;

            // estimate elapsed time for current frame based on direction and segments
            let elapsed_time_for_frame = match self.direction {
                Direction::Forward => (frame_no - start_frame) * frame_duration,
                Direction::Reverse => (end_frame - frame_no) * frame_duration,
            };

            // update start_time to account for the already elapsed time
            if let Some(start_time) =
                Instant::now().checked_sub(Duration::from_secs_f32(elapsed_time_for_frame))
            {
                self.start_time = start_time;
            } else {
                self.start_time = Instant::now();
            }
        } else {
            self.start_time = Instant::now();
        }
    }

    /// Set the frame number to be rendered next.
    ///
    /// # Arguments
    ///
    /// * `no` - The frame number to set.
    ///
    /// # Returns
    ///
    /// Returns `true` if the frame number is valid and updated and `false` otherwise.
    ///
    /// The frame number is considered valid if it's within the range of the start and end frames.
    ///
    /// This function does not update the start time for the new frame assuming it's already managed by the `request_frame` method in the animation loop.
    /// It's the responsibility of the caller to update the start time if needed.
    ///
    pub fn set_frame(&mut self, no: f32) -> bool {
        if no < self.start_frame() || no > self.end_frame() {
            return false;
        }

        self.renderer.set_frame(no).is_ok()
    }

    /// Seek to a specific frame number.
    ///
    /// # Arguments
    ///
    /// * `no` - The frame number to seek to.
    ///
    /// # Returns
    ///
    /// Returns `true` if the frame number is valid and updated and `false` otherwise.
    ///
    /// The frame number is considered valid if it's within the range of the start and end frames.
    ///
    /// The start time is updated based on the new frame number.
    ///
    pub fn seek(&mut self, no: f32) -> bool {
        let is_ok = self.set_frame(no);

        if is_ok {
            self.update_start_time_for_frame(no);
        }

        is_ok
    }

    pub fn render(&mut self) -> bool {
        let is_ok = self.renderer.render().is_ok();

        // rendered the last frame successfully
        if is_ok && self.is_complete() && !self.config.loop_animation {
            self.playback_state = PlaybackState::Stopped;
        }

        is_ok
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

    pub fn speed(&self) -> f32 {
        self.config.speed
    }

    pub fn buffer(&self) -> &[u32] {
        &self.renderer.buffer
    }

    pub fn clear(&mut self) {
        self.renderer.clear()
    }

    pub fn set_config(&mut self, new_config: Config) {
        self.update_mode(&new_config);
        self.update_background_color(&new_config);
        self.update_speed(&new_config);
        self.update_loop_animation(&new_config);

        // directly updating fields that don't require special handling
        self.config.use_frame_interpolation = new_config.use_frame_interpolation;
        self.config.segments = new_config.segments;
        self.config.autoplay = new_config.autoplay;
        self.config.marker = new_config.marker;
    }

    fn update_mode(&mut self, new_config: &Config) {
        if self.config.mode != new_config.mode {
            self.flip_direction_if_needed(new_config.mode);
            self.config.mode = new_config.mode;
        }
    }

    fn flip_direction_if_needed(&mut self, new_mode: Mode) {
        let should_flip = match (new_mode, self.direction) {
            (Mode::Forward | Mode::Bounce, Direction::Reverse)
            | (Mode::Reverse | Mode::ReverseBounce, Direction::Forward) => true,
            _ => false,
        };

        if should_flip {
            self.direction = self.direction.flip();
            self.update_start_time_for_frame(self.current_frame());
        }
    }

    fn update_background_color(&mut self, new_config: &Config) {
        if self.config.background_color != new_config.background_color {
            if self
                .renderer
                .set_background_color(new_config.background_color)
                .is_ok()
            {
                self.config.background_color = new_config.background_color;
            }
        }
    }

    fn update_speed(&mut self, new_config: &Config) {
        if self.config.speed != new_config.speed && new_config.speed > 0.0 {
            self.config.speed = new_config.speed;
        }
    }

    fn update_loop_animation(&mut self, new_config: &Config) {
        if self.config.loop_animation != new_config.loop_animation {
            self.loop_count = 0;
            self.config.loop_animation = new_config.loop_animation;
        }
    }

    fn load_animation_common<F>(&mut self, loader: F, width: u32, height: u32) -> bool
    where
        F: FnOnce(&mut LottieRenderer, u32, u32) -> Result<(), LottieRendererError>,
    {
        self.clear();
        self.playback_state = PlaybackState::Stopped;
        self.start_time = Instant::now();
        self.loop_count = 0;

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
        self.dotlottie_manager = DotLottieManager::new(None).unwrap();

        self.markers = extract_markers(animation_data).unwrap_or_default();

        self.load_animation_common(
            |renderer, w, h| renderer.load_data(animation_data, w, h, false),
            width,
            height,
        )
    }

    pub fn load_animation_path(&mut self, file_path: &str, width: u32, height: u32) -> bool {
        match fs::read_to_string(file_path) {
            Ok(data) => self.load_animation_data(&data, width, height),
            Err(_) => false,
        }
    }

    pub fn load_dotlottie_data(&mut self, file_data: &Vec<u8>, width: u32, height: u32) -> bool {
        if self.dotlottie_manager.init(file_data.clone()).is_err() {
            return false;
        }

        let first_animation: Result<String, DotLottieError> =
            self.dotlottie_manager.get_active_animation();

        match first_animation {
            Ok(animation_data) => {
                self.markers = extract_markers(animation_data.as_str()).unwrap_or_default();

                // For the moment we're ignoring manifest values

                // self.load_playback_settings();
                self.load_animation_common(
                    |renderer, w, h| renderer.load_data(&animation_data, w, h, false),
                    width,
                    height,
                )
            }
            Err(_error) => false,
        }
    }

    pub fn load_animation(&mut self, animation_id: &str, width: u32, height: u32) -> bool {
        let animation_data = self.dotlottie_manager.get_animation(animation_id);

        match animation_data {
            Ok(animation_data) => self.load_animation_common(
                |renderer, w, h| renderer.load_data(&animation_data, w, h, false),
                width,
                height,
            ),
            Err(_error) => false,
        }
    }

    #[allow(dead_code)]
    fn load_playback_settings(&mut self) -> bool {
        let playback_settings_result: Result<ManifestAnimation, DotLottieError> =
            self.dotlottie_manager.active_animation_playback_settings();

        match playback_settings_result {
            Ok(playback_settings) => {
                let speed = playback_settings.speed.unwrap_or(1);
                let loop_animation = playback_settings.r#loop.unwrap_or(false);
                let direction = playback_settings.direction.unwrap_or(1);
                let autoplay = playback_settings.autoplay.unwrap_or(false);
                let play_mode = playback_settings.playMode.unwrap_or("normal".to_string());

                let mode = match play_mode.as_str() {
                    "normal" => Mode::Forward,
                    "reverse" => Mode::Reverse,
                    "bounce" => Mode::Bounce,
                    "reverseBounce" => Mode::ReverseBounce,
                    _ => Mode::Forward,
                };

                self.config.speed = speed as f32;
                self.config.autoplay = autoplay;
                self.config.mode = if play_mode == "normal" {
                    if direction == 1 {
                        Mode::Forward
                    } else {
                        Mode::Reverse
                    }
                } else {
                    mode
                };
                self.config.loop_animation = loop_animation;
            }
            Err(_error) => return false,
        }

        true
    }

    pub fn resize(&mut self, width: u32, height: u32) -> bool {
        self.renderer.resize(width, height).is_ok()
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }

    pub fn is_complete(&self) -> bool {
        match self.config.mode {
            Mode::Forward | Mode::ReverseBounce => self.current_frame() >= self.end_frame(),
            Mode::Reverse | Mode::Bounce => self.current_frame() <= self.start_frame(),
        }
    }
}

pub struct DotLottiePlayer {
    runtime: RwLock<DotLottieRuntime>,
    observers: RwLock<Vec<Arc<dyn Observer>>>,
}

impl DotLottiePlayer {
    pub fn new(config: Config) -> Self {
        DotLottiePlayer {
            runtime: RwLock::new(DotLottieRuntime::new(config)),
            observers: RwLock::new(Vec::new()),
        }
    }

    pub fn load_animation_data(&self, animation_data: &str, width: u32, height: u32) -> bool {
        let is_ok = self
            .runtime
            .write()
            .is_ok_and(|mut runtime| runtime.load_animation_data(animation_data, width, height));

        if is_ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load();
            });
        } else {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load_error();
            });

            return false;
        }

        is_ok
    }

    pub fn load_animation_path(&self, animation_path: &str, width: u32, height: u32) -> bool {
        let is_ok = self
            .runtime
            .write()
            .is_ok_and(|mut runtime| runtime.load_animation_path(animation_path, width, height));

        if is_ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load();
            });
        } else {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load_error();
            });

            return false;
        }

        is_ok
    }

    pub fn load_dotlottie_data(&self, file_data: &Vec<u8>, width: u32, height: u32) -> bool {
        let is_ok = self
            .runtime
            .write()
            .is_ok_and(|mut runtime| runtime.load_dotlottie_data(file_data, width, height));

        if is_ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load();
            });
        } else {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load_error();
            });

            return false;
        }

        is_ok
    }

    pub fn load_animation(&self, animation_id: &str, width: u32, height: u32) -> bool {
        let is_ok = self
            .runtime
            .write()
            .is_ok_and(|mut runtime| runtime.load_animation(animation_id, width, height));

        if is_ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load();
            });
        } else {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load_error();
            });

            return false;
        }

        is_ok
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn manifest(&self) -> Option<Manifest> {
        self.runtime.read().unwrap().manifest()
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
        let ok = self.runtime.write().unwrap().play();

        if ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_play();
            });
        }

        ok
    }

    pub fn pause(&self) -> bool {
        let ok = self.runtime.write().unwrap().pause();

        if ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_pause();
            });
        }

        ok
    }

    pub fn stop(&self) -> bool {
        let ok = self.runtime.write().unwrap().stop();

        if ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_stop();
            });
        }

        ok
    }

    pub fn request_frame(&self) -> f32 {
        self.runtime.write().unwrap().request_frame()
    }

    pub fn set_frame(&self, no: f32) -> bool {
        let ok = self.runtime.write().unwrap().set_frame(no);

        if ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_frame(no);
            });
        }

        ok
    }

    pub fn seek(&self, no: f32) -> bool {
        let ok = self.runtime.write().unwrap().seek(no);

        if ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_frame(no);
            });
        }

        ok
    }

    pub fn render(&self) -> bool {
        let ok = self.runtime.write().unwrap().render();

        if ok {
            let frame_no = self.current_frame();

            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_render(frame_no);
            });

            if self.is_complete() {
                if self.config().loop_animation {
                    self.observers.read().unwrap().iter().for_each(|observer| {
                        observer.on_loop(self.loop_count());
                    });
                } else {
                    self.observers.read().unwrap().iter().for_each(|observer| {
                        observer.on_complete();
                    });
                }
            }
        }

        ok
    }

    pub fn resize(&self, width: u32, height: u32) -> bool {
        self.runtime.write().unwrap().resize(width, height)
    }

    pub fn config(&self) -> Config {
        self.runtime.read().unwrap().config()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn subscribe(&self, observer: Arc<dyn Observer>) {
        self.observers.write().unwrap().push(observer);
    }

    pub fn manifest_string(&self) -> String {
        self.runtime.read().unwrap().manifest().unwrap().to_string()
    }

    pub fn is_complete(&self) -> bool {
        self.runtime.read().unwrap().is_complete()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn unsubscribe(&self, observer: &Arc<dyn Observer>) {
        self.observers
            .write()
            .unwrap()
            .retain(|o| !Arc::ptr_eq(o, observer));
    }

    pub fn markers(&self) -> Vec<Marker> {
        self.runtime.read().unwrap().markers()
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}
