use instant::{Duration, Instant};
use std::sync::RwLock;
use std::{fs, rc::Rc, sync::Arc};

use crate::errors::StateMachineError::ParsingError;
use crate::listeners::ListenerTrait;
use crate::state_machine::events::Event;
use crate::{
    extract_markers,
    layout::Layout,
    lottie_renderer::{LottieRenderer, LottieRendererError},
    Marker, MarkersMap, StateMachine,
};
use crate::{DotLottieError, DotLottieManager, Manifest, ManifestAnimation, Renderer};
use crate::{StateMachineObserver, StateMachineStatus};

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

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
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

#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct Config {
    pub mode: Mode,
    pub loop_animation: bool,
    pub speed: f32,
    pub use_frame_interpolation: bool,
    pub autoplay: bool,
    pub segment: Vec<f32>,
    pub background_color: u32,
    pub layout: Layout,
    pub marker: String,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("mode", &self.mode)
            .field("loop_animation", &self.loop_animation)
            .field("speed", &self.speed)
            .field("use_frame_interpolation", &self.use_frame_interpolation)
            .field("autoplay", &self.autoplay)
            .field("segment", &self.segment)
            .field("background_color", &self.background_color)
            // .field("layout", &self.layout)
            .field("marker", &self.marker)
            .finish()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mode: Mode::Forward,
            loop_animation: false,
            speed: 1.0,
            use_frame_interpolation: true,
            autoplay: false,
            segment: vec![],
            background_color: 0x00000000,
            layout: Layout::default(),
            marker: String::new(),
        }
    }
}

#[repr(C)]
pub struct LayerBoundingBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl From<LayerBoundingBox> for Vec<f32> {
    fn from(bbox: LayerBoundingBox) -> Vec<f32> {
        vec![bbox.x, bbox.y, bbox.w, bbox.h]
    }
}

impl Default for LayerBoundingBox {
    fn default() -> Self {
        LayerBoundingBox {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        }
    }
}

struct DotLottieRuntime {
    renderer: Box<dyn LottieRenderer>,
    playback_state: PlaybackState,
    is_loaded: bool,
    start_time: Instant,
    loop_count: u32,
    config: Config,
    dotlottie_manager: DotLottieManager,
    direction: Direction,
    markers: MarkersMap,
    active_animation_id: String,
    active_theme_id: String,
}

impl DotLottieRuntime {
    #[cfg(feature = "thorvg")]
    pub fn new(config: Config) -> Self {
        Self::with_renderer(
            config,
            crate::TvgRenderer::new(crate::TvgEngine::TvgEngineSw, 0),
        )
    }

    pub fn with_renderer<R: Renderer>(config: Config, renderer: R) -> Self {
        let direction = match config.mode {
            Mode::Forward => Direction::Forward,
            Mode::Reverse => Direction::Reverse,
            Mode::Bounce => Direction::Forward,
            Mode::ReverseBounce => Direction::Reverse,
        };

        DotLottieRuntime {
            renderer: <dyn LottieRenderer>::new(renderer),
            playback_state: PlaybackState::Stopped,
            is_loaded: false,
            start_time: Instant::now(),
            loop_count: 0,
            config,
            dotlottie_manager: DotLottieManager::new(None).unwrap(),
            direction,
            markers: MarkersMap::new(),
            active_animation_id: String::new(),
            active_theme_id: String::new(),
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
        if !self.config.marker.is_empty() {
            if let Some((time, _)) = self.markers.get(&self.config.marker) {
                return (*time).max(0.0);
            }
        }

        if self.config.segment.len() == 2 {
            return self.config.segment[0].max(0.0);
        }

        0.0
    }

    fn end_frame(&self) -> f32 {
        if !self.config.marker.is_empty() {
            if let Some((time, duration)) = self.markers.get(&self.config.marker) {
                return (time + duration).min(self.total_frames());
            }
        }

        if self.config.segment.len() == 2 {
            return self.config.segment[1].min(self.total_frames());
        }

        self.total_frames()
    }

    pub fn hit_check(&self, layer_name: &str, x: f32, y: f32) -> bool {
        self.renderer.hit_check(layer_name, x, y).unwrap_or(false)
    }

    pub fn get_layer_bounds(&self, layer_name: &str) -> Vec<f32> {
        let bbox = self.renderer.get_layer_bounds(layer_name);

        match bbox {
            Err(_) => LayerBoundingBox::default().into(),
            Ok(bbox) => LayerBoundingBox {
                x: bbox.0,
                y: bbox.1,
                w: bbox.2,
                h: bbox.3,
            }
            .into(),
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.playback_state, PlaybackState::Playing)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.playback_state, PlaybackState::Paused)
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self.playback_state, PlaybackState::Stopped)
    }

    pub fn play(&mut self) -> bool {
        if !self.is_loaded || self.is_playing() {
            return false;
        }

        if self.is_complete() && self.is_stopped() {
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
        } else {
            self.update_start_time_for_frame(self.current_frame());
        }

        self.playback_state = PlaybackState::Playing;

        true
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

    pub fn size(&self) -> (u32, u32) {
        (self.renderer.width(), self.renderer.height())
    }

    pub fn get_state_machine(&self, state_machine_id: &str) -> Option<String> {
        self.dotlottie_manager
            .get_state_machine(state_machine_id)
            .ok()
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

        // the animation start & end frames (considering the segment)
        let start_frame = self.start_frame();
        let end_frame = self.end_frame();

        // the effective total frames (considering the segment)
        let effective_total_frames = end_frame - start_frame;

        // the effective duration in milliseconds (considering the segment & speed)
        let effective_duration =
            (duration * effective_total_frames / total_frames) / self.config.speed;

        let raw_next_frame = (elapsed_time / effective_duration) * effective_total_frames;

        // update the next frame based on the direction
        let mut next_frame = match self.direction {
            Direction::Forward => start_frame + raw_next_frame,
            Direction::Reverse => end_frame - raw_next_frame,
        };

        // Apply frame interpolation
        next_frame = if self.config.use_frame_interpolation {
            (next_frame * 1000.0).round() / 1000.0
        } else {
            next_frame.round()
        };

        // Clamp the next frame to the start & end frames
        next_frame = next_frame.clamp(start_frame, end_frame);

        // Handle different modes
        next_frame = match self.config.mode {
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

            // estimate elapsed time for current frame based on direction and segment
            let mut elapsed_time_for_frame = match self.direction {
                Direction::Forward => (frame_no - start_frame) * frame_duration,
                Direction::Reverse => (end_frame - frame_no) * frame_duration,
            };

            if elapsed_time_for_frame < 0.0 {
                elapsed_time_for_frame = 0.0;
            }
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

    pub fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> bool {
        self.renderer.set_viewport(x, y, w, h).is_ok()
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

    pub fn segment_duration(&self) -> f32 {
        if self.config.segment.is_empty() {
            self.duration()
        } else {
            let start_frame = self.start_frame();
            let end_frame = self.end_frame();

            let frame_rate = self.total_frames() / self.duration();

            (end_frame - start_frame) / frame_rate
        }
    }

    pub fn current_frame(&self) -> f32 {
        self.renderer.current_frame()
    }

    pub fn loop_count(&self) -> u32 {
        self.loop_count
    }

    pub fn speed(&self) -> f32 {
        self.config.speed
    }

    pub fn buffer(&self) -> &[u32] {
        self.renderer.buffer()
    }

    pub fn clear(&mut self) {
        self.renderer.clear()
    }

    pub fn set_config(&mut self, new_config: Config) {
        self.update_mode(&new_config);
        self.update_background_color(&new_config);
        self.update_speed(&new_config);
        self.update_loop_animation(&new_config);
        self.update_layout(&new_config.layout);

        // directly updating fields that don't require special handling
        self.config.use_frame_interpolation = new_config.use_frame_interpolation;
        self.config.segment = new_config.segment;
        self.config.autoplay = new_config.autoplay;
        self.config.marker = new_config.marker;
    }

    pub fn update_layout(&mut self, layout: &Layout) {
        if self.renderer.set_layout(layout).is_ok() {
            self.config.layout = layout.clone();
        }
    }

    fn update_mode(&mut self, new_config: &Config) {
        if self.config.mode != new_config.mode {
            self.flip_direction_if_needed(new_config.mode);
            self.config.mode = new_config.mode;
        }
    }

    fn flip_direction_if_needed(&mut self, new_mode: Mode) {
        let should_flip = matches!(
            (new_mode, self.direction),
            (Mode::Forward | Mode::Bounce, Direction::Reverse)
                | (Mode::Reverse | Mode::ReverseBounce, Direction::Forward)
        );

        if should_flip {
            self.direction = self.direction.flip();
            self.update_start_time_for_frame(self.current_frame());
        }
    }

    fn update_background_color(&mut self, new_config: &Config) {
        if self.config.background_color != new_config.background_color
            && self
                .renderer
                .set_background_color(new_config.background_color)
                .is_ok()
        {
            self.config.background_color = new_config.background_color;
        }
    }

    fn update_speed(&mut self, new_config: &Config) {
        if self.config.speed != new_config.speed && new_config.speed > 0.0 {
            self.config.speed = new_config.speed;

            self.update_start_time_for_frame(self.current_frame());
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
        F: FnOnce(&mut dyn LottieRenderer, u32, u32) -> Result<(), LottieRendererError>,
    {
        self.clear();
        self.playback_state = PlaybackState::Stopped;
        self.start_time = Instant::now();
        self.loop_count = 0;

        let loaded = loader(&mut *self.renderer, width, height).is_ok()
            && self
                .renderer
                .set_background_color(self.config.background_color)
                .is_ok();

        if self.renderer.set_layout(&self.config.layout).is_err() {
            return false;
        }

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

        loaded
    }

    pub fn load_animation_data(&mut self, animation_data: &str, width: u32, height: u32) -> bool {
        self.active_animation_id.clear();
        self.active_theme_id.clear();

        self.dotlottie_manager = DotLottieManager::new(None).unwrap();

        self.markers = extract_markers(animation_data);

        self.load_animation_common(
            |renderer, w, h| renderer.load_data(animation_data, w, h, false),
            width,
            height,
        )
    }

    pub fn load_animation_path(&mut self, file_path: &str, width: u32, height: u32) -> bool {
        self.active_animation_id.clear();
        self.active_theme_id.clear();

        match fs::read_to_string(file_path) {
            Ok(data) => self.load_animation_data(&data, width, height),
            Err(_) => false,
        }
    }

    pub fn load_dotlottie_data(&mut self, file_data: &[u8], width: u32, height: u32) -> bool {
        self.active_animation_id.clear();
        self.active_theme_id.clear();

        if self.dotlottie_manager.init(file_data).is_err() {
            return false;
        }

        let first_animation: Result<String, DotLottieError> =
            self.dotlottie_manager.get_active_animation();

        let ok = match first_animation {
            Ok(animation_data) => {
                self.markers = extract_markers(animation_data.as_str());

                // For the moment we're ignoring manifest values

                // self.load_playback_settings();
                self.load_animation_common(
                    |renderer, w, h| renderer.load_data(&animation_data, w, h, false),
                    width,
                    height,
                )
            }
            Err(_error) => false,
        };

        if ok {
            self.active_animation_id = self.dotlottie_manager.active_animation_id();
        }

        ok
    }

    pub fn load_animation(&mut self, animation_id: &str, width: u32, height: u32) -> bool {
        self.active_animation_id.clear();

        let animation_data = self.dotlottie_manager.get_animation(animation_id);

        let ok = match animation_data {
            Ok(animation_data) => self.load_animation_common(
                |renderer, w, h| renderer.load_data(&animation_data, w, h, false),
                width,
                height,
            ),
            Err(_error) => false,
        };

        if ok {
            self.active_animation_id = animation_id.to_string();
        }

        ok
    }

    #[allow(dead_code)]
    fn load_playback_settings(&mut self) -> bool {
        let playback_settings_result: Result<ManifestAnimation, DotLottieError> =
            self.dotlottie_manager.active_animation_playback_settings();

        match playback_settings_result {
            Ok(playback_settings) => {
                let speed = playback_settings.speed.unwrap_or(1.0);
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

                self.config.speed = speed;
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
        if !self.is_loaded() {
            return false;
        }
        match self.config.mode {
            Mode::Forward => self.current_frame() >= self.end_frame(),
            Mode::Reverse => self.current_frame() <= self.start_frame(),
            Mode::Bounce => {
                self.current_frame() <= self.start_frame() && self.direction == Direction::Reverse
            }
            Mode::ReverseBounce => {
                self.current_frame() >= self.end_frame() && self.direction == Direction::Forward
            }
        }
    }

    pub fn load_theme(&mut self, theme_id: &str) -> bool {
        self.active_theme_id.clear();

        if theme_id.is_empty() {
            return self.renderer.load_theme_data("").is_ok();
        }

        let ok = self
            .manifest()
            .and_then(|manifest| manifest.themes)
            .map_or(false, |themes| {
                themes
                    .iter()
                    .find(|t| t.id == theme_id)
                    .map_or(false, |theme| {
                        // check if the theme is either global or scoped to the currently active animation
                        let is_global_or_active_animation = theme.animations.is_empty()
                            || theme
                                .animations
                                .iter()
                                .any(|animation| animation == &self.active_animation_id);

                        is_global_or_active_animation
                            && self
                                .dotlottie_manager
                                .get_theme(theme_id)
                                .ok()
                                .and_then(|theme_data| {
                                    self.renderer.load_theme_data(&theme_data).ok()
                                })
                                .is_some()
                    })
            });

        if ok {
            self.active_theme_id = theme_id.to_string();
        }

        ok
    }

    pub fn load_theme_data(&mut self, theme_data: &str) -> bool {
        self.renderer.load_theme_data(theme_data).is_ok()
    }

    pub fn active_animation_id(&self) -> &str {
        &self.active_animation_id
    }

    pub fn active_theme_id(&self) -> &str {
        &self.active_theme_id
    }
}

pub struct DotLottiePlayerContainer {
    runtime: RwLock<DotLottieRuntime>,
    observers: RwLock<Vec<Arc<dyn Observer>>>,
    state_machine: Rc<RwLock<Option<StateMachine>>>,
}

impl DotLottiePlayerContainer {
    #[cfg(feature = "thorvg")]
    pub fn new(config: Config) -> Self {
        DotLottiePlayerContainer {
            runtime: RwLock::new(DotLottieRuntime::new(config)),
            observers: RwLock::new(Vec::new()),
            state_machine: Rc::new(RwLock::new(None)),
        }
    }

    pub fn with_renderer<R: Renderer>(config: Config, renderer: R) -> Self {
        DotLottiePlayerContainer {
            runtime: RwLock::new(DotLottieRuntime::with_renderer(config, renderer)),
            observers: RwLock::new(Vec::new()),
            state_machine: Rc::new(RwLock::new(None)),
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

            if self.config().autoplay {
                self.play();
            }
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

            if self.config().autoplay {
                self.play();
            }
        } else {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load_error();
            });

            return false;
        }

        is_ok
    }

    pub fn load_dotlottie_data(&self, file_data: &[u8], width: u32, height: u32) -> bool {
        let is_ok = self
            .runtime
            .write()
            .is_ok_and(|mut runtime| runtime.load_dotlottie_data(file_data, width, height));

        if is_ok {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load();
            });

            if self.config().autoplay {
                self.play();
            }
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

            if self.config().autoplay {
                self.play();
            }
        } else {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load_error();
            });

            return false;
        }

        is_ok
    }

    pub fn manifest(&self) -> Option<Manifest> {
        self.runtime.read().unwrap().manifest()
    }

    pub fn buffer(&self) -> *const u32 {
        self.runtime.read().unwrap().buffer().as_ptr()
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

    pub fn size(&self) -> (u32, u32) {
        self.runtime.read().unwrap().size()
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

    pub fn segment_duration(&self) -> f32 {
        self.runtime.read().unwrap().segment_duration()
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

                    if let Ok(mut state_machine) = self.state_machine.try_write() {
                        if let Some(sm) = state_machine.as_mut() {
                            sm.post_event(&Event::OnComplete);
                        }
                    }
                }
            }
        }

        ok
    }

    pub fn set_viewport(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        match self.runtime.try_write() {
            Ok(mut runtime) => runtime.set_viewport(x, y, w, h),
            _ => false,
        }
    }

    pub fn resize(&self, width: u32, height: u32) -> bool {
        self.runtime.write().unwrap().resize(width, height)
    }

    pub fn config(&self) -> Config {
        self.runtime.read().unwrap().config()
    }

    pub fn subscribe(&self, observer: Arc<dyn Observer>) {
        self.observers.write().unwrap().push(observer);
    }

    pub fn manifest_string(&self) -> String {
        self.runtime
            .try_read()
            .ok()
            .and_then(|runtime| runtime.manifest())
            .map_or_else(String::new, |manifest| manifest.to_string())
    }

    pub fn is_complete(&self) -> bool {
        self.runtime.read().unwrap().is_complete()
    }

    pub fn unsubscribe(&self, observer: &Arc<dyn Observer>) {
        self.observers
            .write()
            .unwrap()
            .retain(|o| !Arc::ptr_eq(o, observer));
    }

    pub fn load_theme(&self, theme_id: &str) -> bool {
        self.runtime.write().unwrap().load_theme(theme_id)
    }

    pub fn load_theme_data(&self, theme_data: &str) -> bool {
        self.runtime.write().unwrap().load_theme_data(theme_data)
    }

    pub fn animation_size(&self) -> Vec<f32> {
        match self.runtime.try_read() {
            Ok(runtime) => vec![
                runtime.renderer.picture_width(),
                runtime.renderer.picture_height(),
            ],
            _ => vec![0.0, 0.0],
        }
    }

    pub fn hit_check(&self, layer_name: &str, x: f32, y: f32) -> bool {
        self.runtime.read().unwrap().hit_check(layer_name, x, y)
    }

    pub fn get_layer_bounds(&self, layer_name: &str) -> Vec<f32> {
        self.runtime.read().unwrap().get_layer_bounds(layer_name)
    }

    pub fn markers(&self) -> Vec<Marker> {
        self.runtime.read().unwrap().markers()
    }

    pub fn active_animation_id(&self) -> String {
        self.runtime
            .read()
            .unwrap()
            .active_animation_id()
            .to_string()
    }

    pub fn active_theme_id(&self) -> String {
        self.runtime.read().unwrap().active_theme_id().to_string()
    }

    pub fn get_state_machine(&self, state_machine_id: &str) -> Option<String> {
        match self.runtime.try_read() {
            Ok(runtime) => runtime.get_state_machine(state_machine_id),
            Err(_) => None,
        }
    }
}

pub struct DotLottiePlayer {
    player: Rc<RwLock<DotLottiePlayerContainer>>,
    state_machine: Rc<RwLock<Option<StateMachine>>>,
}

impl DotLottiePlayer {
    #[cfg(feature = "thorvg")]
    pub fn new(config: Config) -> Self {
        DotLottiePlayer {
            player: Rc::new(RwLock::new(DotLottiePlayerContainer::new(config))),
            state_machine: Rc::new(RwLock::new(None)),
        }
    }

    pub fn with_renderer<R: Renderer>(config: Config, renderer: R) -> Self {
        DotLottiePlayer {
            player: Rc::new(RwLock::new(DotLottiePlayerContainer::with_renderer(
                config, renderer,
            ))),
            state_machine: Rc::new(RwLock::new(None)),
        }
    }

    pub fn load_animation_data(&self, animation_data: &str, width: u32, height: u32) -> bool {
        self.player
            .write()
            .is_ok_and(|runtime| runtime.load_animation_data(animation_data, width, height))
    }

    pub fn get_state_machine(&self) -> Rc<RwLock<Option<StateMachine>>> {
        self.state_machine.clone()
    }

    pub fn hit_check(&self, layer_name: &str, x: f32, y: f32) -> bool {
        self.player.read().unwrap().hit_check(layer_name, x, y)
    }

    pub fn get_layer_bounds(&self, layer_name: &str) -> Vec<f32> {
        self.player.read().unwrap().get_layer_bounds(layer_name)
    }

    // If you are in an environment that does not support events
    // Call isPlaying() to know if the state machine started playback within the first state
    pub fn start_state_machine(&self) -> bool {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return false;
                }
            }
            Err(_) => {
                return false;
            }
        }

        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    sm.start();
                }
            }
            Err(_) => {
                return false;
            }
        }

        true
    }

    pub fn stop_state_machine(&self) -> bool {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return false;
                }
            }
            Err(_) => return false,
        }

        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    if sm.status == StateMachineStatus::Running {
                        sm.end();
                    } else {
                        return false;
                    }
                }
            }
            Err(_) => return false,
        }

        true
    }

    /// Returns which types of listeners need to be setup.
    /// The frameworks should call the function after calling start_state_machine.
    pub fn state_machine_framework_setup(&self) -> Vec<String> {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return vec![];
                }

                let mut listener_types = vec![];

                if let Some(sm) = state_machine.as_ref() {
                    let listeners = sm.get_listeners();

                    for listener in listeners {
                        match listener.try_read() {
                            Ok(listener) => {
                                if !listener_types.contains(&listener.get_type().to_string()) {
                                    listener_types.push(listener.get_type().to_string());
                                }
                            }
                            Err(_) => return vec![],
                        }
                    }
                    listener_types
                } else {
                    vec![]
                }
            }
            Err(_) => vec![],
        }
    }

    pub fn set_state_machine_numeric_context(&self, key: &str, value: f32) -> bool {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return false;
                }
            }
            Err(_) => return false,
        }

        let sm_write = self.state_machine.try_write();

        match sm_write {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    sm.set_numeric_context(key, value);
                }
            }
            Err(_) => return false,
        }

        true
    }

    pub fn set_state_machine_string_context(&self, key: &str, value: &str) -> bool {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return false;
                }
            }
            Err(_) => return false,
        }

        let sm_write = self.state_machine.try_write();

        match sm_write {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    sm.set_string_context(key, value);
                }
            }
            Err(_) => return false,
        }

        true
    }

    pub fn set_state_machine_boolean_context(&self, key: &str, value: bool) -> bool {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return false;
                }
            }
            Err(_) => return false,
        }

        let sm_write = self.state_machine.try_write();

        match sm_write {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    sm.set_bool_context(key, value);
                }
            }
            Err(_) => return false,
        }

        true
    }

    // Return codes
    // 0: Success
    // 1: Failure
    // 2: Play animation
    // 3: Pause animation
    // 4: Request and draw a new single frame of the animation (needed for sync state)
    pub fn post_event(&self, event: &Event) -> i32 {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return 1;
                }
            }
            Err(_) => return 1,
        }

        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    return sm.post_event(event);
                }
            }
            Err(_) => return 1,
        }

        1
    }

    pub fn post_bool_event(&self, value: bool) -> i32 {
        let event = Event::Bool { value };
        self.post_event(&event)
    }

    pub fn post_string_event(&self, value: &str) -> i32 {
        let event = Event::String {
            value: value.to_string(),
        };
        self.post_event(&event)
    }

    pub fn post_numeric_event(&self, value: f32) -> i32 {
        let event = Event::Numeric { value };
        self.post_event(&event)
    }

    pub fn post_pointer_down_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::OnPointerDown { x, y };
        self.post_event(&event)
    }

    pub fn post_pointer_up_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::OnPointerUp { x, y };
        self.post_event(&event)
    }

    pub fn post_pointer_move_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::OnPointerMove { x, y };
        self.post_event(&event)
    }

    pub fn post_pointer_enter_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::OnPointerEnter { x, y };
        self.post_event(&event)
    }

    pub fn post_pointer_exit_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::OnPointerExit { x, y };
        self.post_event(&event)
    }

    pub fn post_set_numeric_context(&self, key: &str, value: f32) -> i32 {
        let event = Event::SetNumericContext {
            key: key.to_string(),
            value,
        };

        self.post_event(&event)
    }

    pub fn load_animation_path(&self, animation_path: &str, width: u32, height: u32) -> bool {
        self.player
            .write()
            .is_ok_and(|runtime| runtime.load_animation_path(animation_path, width, height))
    }

    pub fn load_dotlottie_data(&self, file_data: &[u8], width: u32, height: u32) -> bool {
        self.player
            .write()
            .is_ok_and(|runtime| runtime.load_dotlottie_data(file_data, width, height))
    }

    pub fn load_animation(&self, animation_id: &str, width: u32, height: u32) -> bool {
        self.player
            .write()
            .is_ok_and(|runtime| runtime.load_animation(animation_id, width, height))
    }

    pub fn manifest(&self) -> Option<Manifest> {
        self.player.read().unwrap().manifest()
    }

    pub fn buffer(&self) -> *const u32 {
        self.player.read().unwrap().buffer()
    }

    pub fn buffer_ptr(&self) -> u64 {
        self.player.read().unwrap().buffer_ptr()
    }

    pub fn buffer_len(&self) -> u64 {
        self.player.read().unwrap().buffer_len()
    }

    pub fn clear(&self) {
        self.player.write().unwrap().clear();
    }

    pub fn set_config(&self, config: Config) {
        self.player.write().unwrap().set_config(config);
    }

    pub fn speed(&self) -> f32 {
        self.player.read().unwrap().speed()
    }

    pub fn total_frames(&self) -> f32 {
        self.player.read().unwrap().total_frames()
    }

    pub fn duration(&self) -> f32 {
        self.player.read().unwrap().duration()
    }

    pub fn current_frame(&self) -> f32 {
        self.player.read().unwrap().current_frame()
    }

    pub fn loop_count(&self) -> u32 {
        self.player.read().unwrap().loop_count()
    }

    pub fn is_loaded(&self) -> bool {
        self.player.read().unwrap().is_loaded()
    }

    pub fn is_playing(&self) -> bool {
        self.player.read().unwrap().is_playing()
    }

    pub fn is_paused(&self) -> bool {
        self.player.read().unwrap().is_paused()
    }

    pub fn is_stopped(&self) -> bool {
        self.player.read().unwrap().is_stopped()
    }

    pub fn segment_duration(&self) -> f32 {
        self.player.read().unwrap().segment_duration()
    }

    pub fn set_viewport(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        self.player.write().unwrap().set_viewport(x, y, w, h)
    }

    pub fn play(&self) -> bool {
        self.player.write().unwrap().play()
    }

    pub fn pause(&self) -> bool {
        self.player.write().unwrap().pause()
    }

    pub fn stop(&self) -> bool {
        self.player.write().unwrap().stop()
    }

    pub fn request_frame(&self) -> f32 {
        self.player.write().unwrap().request_frame()
    }

    pub fn set_frame(&self, no: f32) -> bool {
        self.player.write().unwrap().set_frame(no)
    }

    pub fn seek(&self, no: f32) -> bool {
        self.player.write().unwrap().seek(no)
    }

    pub fn render(&self) -> bool {
        self.player.read().unwrap().render()
    }

    pub fn resize(&self, width: u32, height: u32) -> bool {
        self.player.write().unwrap().resize(width, height)
    }

    pub fn config(&self) -> Config {
        self.player.read().unwrap().config()
    }

    pub fn subscribe(&self, observer: Arc<dyn Observer>) {
        self.player.write().unwrap().subscribe(observer);
    }

    pub fn state_machine_subscribe(&self, observer: Arc<dyn StateMachineObserver>) -> bool {
        let mut sm = self.state_machine.write().unwrap();

        if sm.is_none() {
            return false;
        }
        sm.as_mut().unwrap().subscribe(observer);

        true
    }

    pub fn state_machine_unsubscribe(&self, observer: Arc<dyn StateMachineObserver>) -> bool {
        let mut sm = self.state_machine.write().unwrap();

        if sm.is_none() {
            return false;
        }

        sm.as_mut().unwrap().unsubscribe(&observer);

        true
    }

    pub fn manifest_string(&self) -> String {
        self.player
            .try_read()
            .map_or_else(|_| String::new(), |player| player.manifest_string())
    }

    pub fn is_complete(&self) -> bool {
        self.player.read().unwrap().is_complete()
    }

    pub fn unsubscribe(&self, observer: &Arc<dyn Observer>) {
        self.player.write().unwrap().unsubscribe(observer);
    }

    pub fn load_theme(&self, theme_id: &str) -> bool {
        self.player.write().unwrap().load_theme(theme_id)
    }

    pub fn load_state_machine_data(&self, state_machine: &str) -> bool {
        let state_machine = StateMachine::new(state_machine, self.player.clone());

        if state_machine.is_ok() {
            match self.state_machine.try_write() {
                Ok(mut sm) => {
                    sm.replace(state_machine.unwrap());
                }
                Err(_) => {
                    return false;
                }
            }

            let player = self.player.try_write();

            match player {
                Ok(mut player) => {
                    player.state_machine = self.state_machine.clone();
                }
                Err(_) => {
                    return false;
                }
            }
        }

        true
    }

    pub fn load_state_machine(&self, state_machine_id: &str) -> bool {
        let state_machine_string = self
            .player
            .read()
            .unwrap()
            .get_state_machine(state_machine_id);

        match state_machine_string {
            Some(machine) => {
                let state_machine = StateMachine::new(&machine, self.player.clone());

                if state_machine.is_ok() {
                    match self.state_machine.try_write() {
                        Ok(mut sm) => {
                            sm.replace(state_machine.unwrap());
                        }
                        Err(_) => {
                            return false;
                        }
                    }

                    let player = self.player.try_write();

                    match player {
                        Ok(mut player) => {
                            player.state_machine = self.state_machine.clone();
                        }
                        Err(_) => {
                            return false;
                        }
                    }
                } else if let Err(ParsingError { reason: _ }) = state_machine {
                    return false;
                }
            }
            None => {
                return false;
            }
        }
        true
    }

    pub fn load_theme_data(&self, theme_data: &str) -> bool {
        self.player.write().unwrap().load_theme_data(theme_data)
    }

    pub fn markers(&self) -> Vec<Marker> {
        self.player.read().unwrap().markers()
    }

    pub fn active_animation_id(&self) -> String {
        self.player
            .read()
            .unwrap()
            .active_animation_id()
            .to_string()
    }

    pub fn active_theme_id(&self) -> String {
        self.player.read().unwrap().active_theme_id().to_string()
    }

    pub fn animation_size(&self) -> Vec<f32> {
        self.player.read().unwrap().animation_size()
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}
