use crate::time::{Duration, Instant};
use std::sync::RwLock;
use std::{fs, rc::Rc, sync::Arc};

use crate::actions::open_url::OpenUrl;
use crate::state_machine_engine::events::Event;
use crate::{
    extract_markers,
    layout::Layout,
    lottie_renderer::{LottieRenderer, LottieRendererError},
    Marker, MarkersMap, StateMachineEngine,
};
use crate::{
    transform_theme_to_lottie_slots, DotLottieManager, Manifest, Renderer, StateMachineEngineError,
};

use crate::StateMachineObserver;

use crate::StateMachineEngineStatus;

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

#[derive(Debug, Clone, PartialEq)]
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
    pub theme_id: String,
    pub animation_id: String,
    pub state_machine_id: String,
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
            theme_id: String::new(),
            animation_id: String::new(),
            state_machine_id: String::new(),
        }
    }
}

#[repr(C)]
pub struct LayerBoundingBox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub x3: f32,
    pub y3: f32,
    pub x4: f32,
    pub y4: f32,
}

impl From<LayerBoundingBox> for Vec<f32> {
    fn from(bbox: LayerBoundingBox) -> Vec<f32> {
        vec![
            bbox.x1, bbox.y1, bbox.x2, bbox.y2, bbox.x3, bbox.y3, bbox.x4, bbox.y4,
        ]
    }
}

impl Default for LayerBoundingBox {
    fn default() -> Self {
        LayerBoundingBox {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
            x3: 0.0,
            y3: 0.0,
            x4: 0.0,
            y4: 0.0,
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
    dotlottie_manager: Option<DotLottieManager>,
    direction: Direction,
    markers: MarkersMap,
    active_animation_id: String,
    active_theme_id: String,
    active_state_machine_id: String,
}

impl DotLottieRuntime {
    #[cfg(any(feature = "thorvg-v0", feature = "thorvg-v1"))]
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
            dotlottie_manager: None,
            direction,
            markers: MarkersMap::new(),
            active_animation_id: String::new(),
            active_theme_id: String::new(),
            active_state_machine_id: String::new(),
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
                return (time + duration).min(self.total_frames() - 1.0);
            }
        }

        if self.config.segment.len() == 2 {
            return self.config.segment[1].min(self.total_frames() - 1.0);
        }

        self.total_frames() - 1.0
    }

    pub fn intersect(&self, x: f32, y: f32, layer_name: &str) -> bool {
        self.renderer.intersect(x, y, layer_name).unwrap_or(false)
    }

    pub fn get_layer_bounds(&self, layer_name: &str) -> Vec<f32> {
        let bbox = self.renderer.get_layer_bounds(layer_name);

        match bbox {
            Err(_) => LayerBoundingBox::default().into(),
            Ok(bbox) => LayerBoundingBox {
                x1: bbox[0],
                y1: bbox[1],
                x2: bbox[2],
                y2: bbox[3],
                x3: bbox[4],
                y3: bbox[5],
                x4: bbox[6],
                y4: bbox[7],
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

    pub fn manifest(&self) -> Option<&Manifest> {
        self.dotlottie_manager
            .as_ref()
            .map(|manager| manager.manifest())
    }

    pub fn size(&self) -> (u32, u32) {
        (self.renderer.width(), self.renderer.height())
    }

    pub fn get_state_machine(&self, state_machine_id: &str) -> Option<String> {
        self.dotlottie_manager
            .as_ref()
            .and_then(|manager| manager.get_state_machine(state_machine_id).ok())
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
            (next_frame * 1000.0).round() * 0.001
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
        self.renderer.total_frames().unwrap_or(0.0)
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

    // Notes: Runtime doesn't have the state machine
    // Therefor the state machine can't be loaded here, user must use the load methods.
    pub fn set_config(&mut self, new_config: Config) {
        self.update_mode(&new_config);
        self.update_background_color(&new_config);
        self.update_speed(&new_config);
        self.update_loop_animation(&new_config);
        self.update_marker(&new_config.marker);
        self.update_layout(&new_config.layout);
        self.set_theme(&new_config.theme_id);

        // directly updating fields that don't require special handling
        self.config.use_frame_interpolation = new_config.use_frame_interpolation;
        self.config.segment = new_config.segment;
        self.config.autoplay = new_config.autoplay;
        self.config.theme_id = new_config.theme_id;
        self.config.animation_id = new_config.animation_id;
    }

    pub fn update_marker(&mut self, marker: &String) {
        if self.config.marker == *marker {
            return;
        }

        let markers = self.markers();

        if let Some(marker) = markers.iter().find(|m| m.name == *marker) {
            self.start_time = Instant::now();

            self.config.marker = marker.name.clone();

            self.set_frame(marker.time);

            self.render();
        } else {
            self.config.marker = String::new();
        }
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
        self.dotlottie_manager = None;
        self.active_animation_id.clear();
        self.active_theme_id.clear();

        self.markers = extract_markers(animation_data);

        let animation_loaded = self.load_animation_common(
            |renderer, w, h| renderer.load_data(animation_data, w, h, false),
            width,
            height,
        );

        if animation_loaded {
            if !self.config.animation_id.is_empty() {
                self.active_animation_id = self.config.animation_id.clone();
            }

            let theme_id = self.config.theme_id.clone();
            if !theme_id.is_empty() {
                self.set_theme(&theme_id);
            }
        }

        animation_loaded
    }

    pub fn load_animation_path(&mut self, file_path: &str, width: u32, height: u32) -> bool {
        self.dotlottie_manager = None;
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

        match DotLottieManager::new(file_data) {
            Ok(manager) => {
                self.dotlottie_manager = Some(manager);
                if let Some(manager) = &mut self.dotlottie_manager {
                    let (active_animation, active_animation_id) =
                        if !self.config.animation_id.is_empty() {
                            (
                                manager.get_animation(&self.config.animation_id),
                                self.config.animation_id.clone(),
                            )
                        } else {
                            (
                                manager.get_active_animation(),
                                manager.active_animation_id(),
                            )
                        };

                    if let Ok(animation_data) = active_animation {
                        self.markers = extract_markers(animation_data.as_str());
                        let animation_loaded = self.load_animation_common(
                            |renderer, w, h| renderer.load_data(&animation_data, w, h, false),
                            width,
                            height,
                        );

                        if animation_loaded {
                            self.active_animation_id = active_animation_id;
                            if !self.config.theme_id.is_empty() {
                                self.set_theme(&self.config.theme_id.clone());
                            }
                        }

                        return animation_loaded;
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    pub fn load_animation(&mut self, animation_id: &str, width: u32, height: u32) -> bool {
        self.active_animation_id.clear();
        if let Some(manager) = &mut self.dotlottie_manager {
            let animation_data = manager.get_animation(animation_id);

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
        } else {
            false
        }
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

    pub fn set_theme(&mut self, theme_id: &str) -> bool {
        if self.active_theme_id == theme_id {
            return true;
        }

        if self.dotlottie_manager.is_none() {
            return false;
        }

        self.active_theme_id.clear();

        if theme_id.is_empty() {
            return self.renderer.set_slots("").is_ok();
        }

        let theme_exists = self
            .manifest()
            .and_then(|manifest| manifest.themes.as_ref())
            .is_some_and(|themes| themes.iter().any(|theme| theme.id == theme_id));

        if !theme_exists {
            return false;
        }

        let can_set_theme = self.manifest().is_some_and(|manifest| {
            manifest.animations.iter().any(|animation| {
                animation.themes.is_none()
                    || animation
                        .themes
                        .as_ref()
                        .unwrap()
                        .contains(&theme_id.to_string())
            })
        });

        if !can_set_theme {
            return false;
        }

        let ok = self
            .dotlottie_manager
            .as_mut()
            .and_then(|manager| manager.get_theme(theme_id).ok())
            .and_then(|theme_data| {
                let slots = transform_theme_to_lottie_slots(&theme_data, &self.active_animation_id)
                    .unwrap();
                self.renderer.set_slots(&slots).ok()
            })
            .is_some();

        if ok {
            self.active_theme_id = theme_id.to_string();
        }

        ok
    }

    pub fn reset_theme(&mut self) -> bool {
        self.active_theme_id.clear();
        self.renderer.set_slots("").is_ok()
    }

    pub fn set_theme_data(&mut self, theme_data: &str) -> bool {
        match transform_theme_to_lottie_slots(theme_data, &self.active_animation_id) {
            Ok(slots) => self.renderer.set_slots(&slots).is_ok(),
            Err(_) => false,
        }
    }

    pub fn set_slots(&mut self, slots: &str) -> bool {
        self.renderer.set_slots(slots).is_ok()
    }

    pub fn active_animation_id(&self) -> &str {
        &self.active_animation_id
    }

    pub fn active_theme_id(&self) -> &str {
        &self.active_theme_id
    }

    pub fn active_state_machine_id(&self) -> &str {
        &self.active_state_machine_id
    }

    pub fn set_active_state_machine_id(&mut self, state_machine_id: &str) {
        self.active_state_machine_id = state_machine_id.to_string();
    }

    pub fn tween(&mut self, to: f32, duration: Option<f32>, easing: Option<[f32; 4]>) -> bool {
        self.renderer.tween(to, duration, easing).is_ok()
    }

    pub fn tween_stop(&mut self) -> bool {
        self.renderer.tween_stop().is_ok()
    }

    pub fn tween_to_marker(
        &mut self,
        marker: &str,
        duration: Option<f32>,
        easing: Option<[f32; 4]>,
    ) -> bool {
        let markers = self.markers();
        if let Some(marker) = markers.iter().find(|m| m.name == marker) {
            self.tween(marker.time, duration, easing);
            self.config.marker = marker.name.clone();
            true
        } else {
            false
        }
    }

    pub fn is_tweening(&self) -> bool {
        self.renderer.is_tweening()
    }

    pub fn tween_update(&mut self, progress: Option<f32>) -> bool {
        let ok = self.renderer.tween_update(progress).is_ok();
        if !ok {
            // so after the tweening is completed, we can start calculating the next frame based on the start time
            self.start_time = Instant::now();
        }
        ok
    }
}

pub struct DotLottiePlayerContainer {
    runtime: RwLock<DotLottieRuntime>,
    observers: RwLock<Vec<Arc<dyn Observer>>>,
    state_machine: Rc<RwLock<Option<StateMachineEngine>>>,
}

impl DotLottiePlayerContainer {
    #[cfg(any(feature = "thorvg-v0", feature = "thorvg-v1"))]
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

    pub fn emit_on_load(&self) {
        self.observers.read().unwrap().iter().for_each(|observer| {
            observer.on_load();
        });
    }

    pub fn emit_on_load_error(&self) {
        self.observers.read().unwrap().iter().for_each(|observer| {
            observer.on_load_error();
        });
    }

    pub fn emit_on_play(&self) {
        self.observers.read().unwrap().iter().for_each(|observer| {
            observer.on_play();
        });
    }

    pub fn emit_on_pause(&self) {
        self.observers.read().unwrap().iter().for_each(|observer| {
            observer.on_pause();
        });
    }

    pub fn emit_on_stop(&self) {
        self.observers.read().unwrap().iter().for_each(|observer| {
            observer.on_stop();
        });
    }

    pub fn emit_on_frame(&self, frame_no: f32) {
        self.observers.read().unwrap().iter().for_each(|observer| {
            observer.on_frame(frame_no);
        });
    }

    pub fn emit_on_render(&self, frame_no: f32) {
        self.observers.read().unwrap().iter().for_each(|observer| {
            observer.on_render(frame_no);
        });
    }

    pub fn emit_on_loop(&self, loop_count: u32) {
        self.observers.read().unwrap().iter().for_each(|observer| {
            observer.on_loop(loop_count);
        });

        if let Ok(mut state_machine) = self.state_machine.try_write() {
            if let Some(sm) = state_machine.as_mut() {
                sm.post_event(&Event::OnLoopComplete);
            }
        }
    }

    pub fn emit_on_complete(&self) {
        self.observers.read().unwrap().iter().for_each(|observer| {
            observer.on_complete();
        });

        if let Ok(mut state_machine) = self.state_machine.try_write() {
            if let Some(sm) = state_machine.as_mut() {
                sm.post_event(&Event::OnComplete);
            }
        }
    }

    pub fn load_animation_data(&self, animation_data: &str, width: u32, height: u32) -> bool {
        let is_ok = self
            .runtime
            .write()
            .is_ok_and(|mut runtime| runtime.load_animation_data(animation_data, width, height));

        if is_ok {
            self.emit_on_load();

            if self.config().autoplay {
                self.play();
            }
        } else {
            self.emit_on_load_error();

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
            self.emit_on_load();

            if self.config().autoplay {
                self.play();
            }
        } else {
            self.emit_on_load_error();

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
            self.emit_on_load();

            if self.config().autoplay {
                self.play();
            }
        } else {
            self.emit_on_load_error();

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
            self.emit_on_load();

            if self.config().autoplay {
                self.play();
            }
        } else {
            self.emit_on_load_error();

            return false;
        }

        is_ok
    }

    pub fn manifest(&self) -> Option<Manifest> {
        self.runtime
            .read()
            .ok()
            .and_then(|runtime| runtime.manifest().cloned())
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
            self.emit_on_play();
        }

        ok
    }

    pub fn pause(&self) -> bool {
        let ok = self.runtime.write().unwrap().pause();

        if ok {
            self.emit_on_pause();
        }

        ok
    }

    pub fn stop(&self) -> bool {
        let ok = self.runtime.write().unwrap().stop();

        if ok {
            self.emit_on_stop();
        }

        ok
    }

    pub fn request_frame(&self) -> f32 {
        self.runtime.write().unwrap().request_frame()
    }

    pub fn set_frame(&self, no: f32) -> bool {
        let ok = self.runtime.write().unwrap().set_frame(no);

        if ok {
            self.emit_on_frame(no);
        }

        ok
    }

    pub fn seek(&self, no: f32) -> bool {
        let ok = self.runtime.write().unwrap().seek(no);

        if ok {
            self.emit_on_frame(no);
        }

        ok
    }

    pub fn render(&self) -> bool {
        let ok = self.runtime.write().unwrap().render();

        if ok {
            let frame_no = self.current_frame();

            self.emit_on_render(frame_no);

            if self.is_complete() {
                if self.config().loop_animation {
                    self.emit_on_loop(self.loop_count());
                } else {
                    self.emit_on_complete();
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
            .and_then(|runtime| runtime.manifest().cloned())
            .map_or_else(String::new, |manifest| {
                serde_json::to_string(&manifest).unwrap()
            })
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

    pub fn set_theme(&self, theme_id: &str) -> bool {
        self.runtime.write().unwrap().set_theme(theme_id)
    }

    pub fn reset_theme(&self) -> bool {
        self.runtime.write().unwrap().reset_theme()
    }

    pub fn set_theme_data(&self, theme_data: &str) -> bool {
        self.runtime.write().unwrap().set_theme_data(theme_data)
    }

    pub fn set_slots(&self, slots: &str) -> bool {
        self.runtime.write().unwrap().set_slots(slots)
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

    pub fn intersect(&self, x: f32, y: f32, layer_name: &str) -> bool {
        self.runtime.read().unwrap().intersect(x, y, layer_name)
    }

    pub fn markers(&self) -> Vec<Marker> {
        self.runtime.read().unwrap().markers()
    }

    pub fn get_layer_bounds(&self, layer_name: &str) -> Vec<f32> {
        self.runtime.read().unwrap().get_layer_bounds(layer_name)
    }

    pub fn active_animation_id(&self) -> String {
        self.runtime
            .read()
            .unwrap()
            .active_animation_id()
            .to_string()
    }

    pub fn active_state_machine_id(&self) -> String {
        self.runtime
            .read()
            .unwrap()
            .active_state_machine_id()
            .to_string()
    }

    pub fn set_active_state_machine_id(&self, active_state_machine_id: &str) {
        self.runtime
            .write()
            .unwrap()
            .set_active_state_machine_id(active_state_machine_id);
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

    pub fn state_machine_status(&self) -> String {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if let Some(sm) = state_machine.as_ref() {
                    return sm.status();
                }
            }

            Err(_) => {
                return "".to_string();
            }
        }

        "".to_string()
    }

    pub fn tick(&self) -> bool {
        if self.is_tweening() {
            self.tween_update(None) && self.render()
        } else {
            let next_frame = self.request_frame();
            let sf = self.set_frame(next_frame) && self.render();

            let mut is_sm_still_tweening = false;

            if let Ok(state_machine) = self.state_machine.try_read() {
                let sm = &*state_machine;

                if let Some(sm) = sm {
                    if sm.status == StateMachineEngineStatus::Tweening {
                        is_sm_still_tweening = true;
                    }
                }
            }

            if is_sm_still_tweening {
                if let Ok(mut state_machine) = self.state_machine.try_write() {
                    {
                        if let Some(sm) = state_machine.as_mut() {
                            sm.resume_from_tweening();
                        }
                    }
                }
            }

            sf
        }
    }

    pub fn tween(&self, to: f32, duration: Option<f32>, easing: Option<Vec<f32>>) -> bool {
        // Convert Vec<f32> to [f32; 4]
        let easing = easing.and_then(|e| {
            if e.len() == 4 {
                Some([e[0], e[1], e[2], e[3]])
            } else {
                None
            }
        });

        self.runtime.write().unwrap().tween(to, duration, easing)
    }

    pub fn tween_stop(&self) -> bool {
        self.runtime.write().unwrap().tween_stop()
    }

    pub fn tween_to_marker(
        &self,
        marker: &str,
        duration: Option<f32>,
        easing: Option<Vec<f32>>,
    ) -> bool {
        // Convert Vec<f32> to [f32; 4]
        let easing = easing.and_then(|e| {
            if e.len() == 4 {
                Some([e[0], e[1], e[2], e[3]])
            } else {
                None
            }
        });

        self.runtime
            .write()
            .unwrap()
            .tween_to_marker(marker, duration, easing)
    }

    pub fn is_tweening(&self) -> bool {
        self.runtime.read().unwrap().is_tweening()
    }

    pub fn tween_update(&self, progress: Option<f32>) -> bool {
        self.runtime.write().unwrap().tween_update(progress)
    }
}

pub struct DotLottiePlayer {
    player: Rc<RwLock<DotLottiePlayerContainer>>,
    state_machine: Rc<RwLock<Option<StateMachineEngine>>>,
}

impl DotLottiePlayer {
    #[cfg(any(feature = "thorvg-v0", feature = "thorvg-v1"))]
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

    pub fn get_state_machine(&self, state_machine_id: &str) -> String {
        if let Some(sm) = self
            .player
            .read()
            .unwrap()
            .get_state_machine(state_machine_id)
        {
            return sm;
        }

        "".to_string()
    }

    pub fn intersect(&self, x: f32, y: f32, layer_name: &str) -> bool {
        self.player.read().unwrap().intersect(x, y, layer_name)
    }

    pub fn get_layer_bounds(&self, layer_name: &str) -> Vec<f32> {
        self.player.read().unwrap().get_layer_bounds(layer_name)
    }

    pub fn state_machine_start(&self, open_url: OpenUrl) -> bool {
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
                    return sm.start(&open_url);
                }
            }
            Err(_) => {
                return false;
            }
        }

        false
    }

    pub fn state_machine_stop(&self) -> bool {
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
                    if sm.status == StateMachineEngineStatus::Running {
                        sm.stop();
                    }

                    *state_machine = None;
                }
            }
            Err(_) => return false,
        }

        true
    }

    /// Returns which types of interactions need to be setup.
    /// The frameworks should call the function after calling start_state_machine.
    pub fn state_machine_framework_setup(&self) -> Vec<String> {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if state_machine.is_none() {
                    return vec![];
                }

                let mut interaction_types = vec![];

                if let Some(sm) = state_machine.as_ref() {
                    let interactions = sm.interactions(None);

                    for interaction in interactions {
                        match interaction {
                            crate::interactions::Interaction::PointerUp { .. } => {
                                interaction_types.push("PointerUp".to_string())
                            }
                            crate::interactions::Interaction::PointerDown { .. } => {
                                interaction_types.push("PointerDown".to_string())
                            }
                            crate::interactions::Interaction::PointerEnter { .. } => {
                                // In case framework self detects pointer entering layers, push pointerExit
                                interaction_types.push("PointerEnter".to_string());
                                // We push PointerMove too so that we can do hit detection instead of the framework
                                interaction_types.push("PointerMove".to_string());
                            }
                            crate::interactions::Interaction::PointerMove { .. } => {
                                interaction_types.push("PointerMove".to_string())
                            }
                            crate::interactions::Interaction::PointerExit { .. } => {
                                // In case framework self detects pointer exiting layers, push pointerExit
                                interaction_types.push("PointerExit".to_string());
                                // We push PointerMove too so that we can do hit detection instead of the framework
                                interaction_types.push("PointerMove".to_string());
                            }
                            crate::interactions::Interaction::OnComplete { .. } => {
                                interaction_types.push("OnComplete".to_string())
                            }
                            crate::interactions::Interaction::OnLoopComplete { .. } => {
                                interaction_types.push("OnLoopComplete".to_string())
                            }
                            crate::interactions::Interaction::Click { .. } => {
                                interaction_types.push("Click".to_string());
                            }
                        }
                    }

                    interaction_types.sort();
                    interaction_types.dedup();
                    interaction_types
                } else {
                    vec![]
                }
            }
            Err(_) => vec![],
        }
    }

    // Return codes
    // 0: Success
    // 1: Failure
    pub fn state_machine_post_event(&self, event: &Event) -> i32 {
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

    pub fn state_machine_override_current_state(&self, state_name: &str, do_tick: bool) -> bool {
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
                    sm.override_current_state(state_name, do_tick);
                }
            }
            Err(_) => return false,
        }

        false
    }

    pub fn state_machine_post_click_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::Click { x, y };
        self.state_machine_post_event(&event)
    }

    pub fn state_machine_post_pointer_down_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::PointerDown { x, y };
        self.state_machine_post_event(&event)
    }

    pub fn state_machine_post_pointer_up_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::PointerUp { x, y };
        self.state_machine_post_event(&event)
    }

    pub fn state_machine_post_pointer_move_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::PointerMove { x, y };
        self.state_machine_post_event(&event)
    }

    pub fn state_machine_post_pointer_enter_event(&self, x: f32, y: f32) -> i32 {
        let event = Event::PointerEnter { x, y };
        self.state_machine_post_event(&event)
    }

    pub fn state_machine_post_pointer_exit_event(&self, x: f32, y: f32) -> i32 {
        let event: Event = Event::PointerExit { x, y };
        self.state_machine_post_event(&event)
    }

    pub fn state_machine_set_numeric_input(&self, key: &str, value: f32) -> bool {
        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    let ret = sm.set_numeric_input(key, value, true, false);

                    if ret.is_some() {
                        return true;
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    pub fn state_machine_set_string_input(&self, key: &str, value: &str) -> bool {
        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    let ret = sm.set_string_input(key, value, true, false);

                    if ret.is_some() {
                        return true;
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    pub fn state_machine_set_boolean_input(&self, key: &str, value: bool) -> bool {
        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    let ret = sm.set_boolean_input(key, value, true, false);

                    if ret.is_some() {
                        return true;
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    pub fn state_machine_get_numeric_input(&self, key: &str) -> f32 {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if let Some(sm) = &*state_machine {
                    if let Some(value) = sm.get_numeric_input(key) {
                        return value;
                    }
                }
            }
            Err(_) => {
                return f32::MIN;
            }
        }

        f32::MIN
    }

    pub fn state_machine_get_string_input(&self, key: &str) -> String {
        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    if let Some(value) = sm.get_string_input(key) {
                        return value;
                    }
                }
            }
            Err(_) => return "".to_string(),
        }

        "".to_string()
    }

    pub fn state_machine_get_boolean_input(&self, key: &str) -> bool {
        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    if let Some(value) = sm.get_boolean_input(key) {
                        return value;
                    }
                }
            }
            Err(_) => return false,
        }

        false
    }

    pub fn state_machine_fire_event(&self, event: &str) {
        if let Ok(mut state_machine) = self.state_machine.try_write() {
            if let Some(sm) = state_machine.as_mut() {
                let _ = sm.fire(event, true);
            }
        }
    }

    pub fn load_animation_path(&self, animation_path: &str, width: u32, height: u32) -> bool {
        match self.player.try_write() {
            Ok(player) => {
                let load_status = player.load_animation_path(animation_path, width, height);

                let sm_id = player.config().state_machine_id;

                if !sm_id.is_empty() {
                    drop(player); // Explicitly drop to release the lock before state_machine_load

                    let load = self.state_machine_load(&sm_id);

                    let start = self.state_machine_start(OpenUrl::default());

                    return load && start;
                }

                load_status
            }
            Err(_) => false,
        }
    }

    pub fn load_dotlottie_data(&self, file_data: &[u8], width: u32, height: u32) -> bool {
        match self.player.try_write() {
            Ok(player) => {
                let load_status = player.load_dotlottie_data(file_data, width, height);

                let sm_id = player.config().state_machine_id;

                if !sm_id.is_empty() {
                    drop(player); // Explicitly drop to release the lock before state_machine_load

                    let load = self.state_machine_load(&sm_id);

                    let start = self.state_machine_start(OpenUrl::default());

                    return load && start;
                }

                load_status
            }
            Err(_) => false,
        }
    }

    pub fn load_animation(&self, animation_id: &str, width: u32, height: u32) -> bool {
        match self.player.try_write() {
            Ok(player) => {
                let load_status = player.load_animation(animation_id, width, height);

                let sm_id = player.config().state_machine_id;

                if !sm_id.is_empty() {
                    drop(player); // Explicitly drop to release the lock before state_machine_load

                    let load = self.state_machine_load(&sm_id);

                    let start = self.state_machine_start(OpenUrl::default());

                    return load && start;
                }

                load_status
            }
            Err(_) => false,
        }
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
        self.player.write().unwrap().clear()
    }

    pub fn set_config(&self, config: Config) {
        self.player.write().unwrap().set_config(config)
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
        self.player.write().unwrap().subscribe(observer)
    }

    pub fn state_machine_subscribe(&self, observer: Arc<dyn StateMachineObserver>) -> bool {
        let sm = self.state_machine.try_write();

        match sm {
            Ok(mut sm) => {
                if sm.is_none() {
                    let new_state_machine: StateMachineEngine = StateMachineEngine::default();

                    new_state_machine.subscribe(observer);

                    sm.replace(new_state_machine);

                    return true;
                } else if let Some(sm) = sm.as_mut() {
                    sm.subscribe(observer);
                }
            }
            Err(_) => {
                return false;
            }
        }

        true
    }

    pub fn state_machine_unsubscribe(&self, observer: &Arc<dyn StateMachineObserver>) -> bool {
        let mut sm = self.state_machine.write().unwrap();

        if sm.is_none() {
            return false;
        }

        sm.as_mut().unwrap().unsubscribe(observer);

        true
    }

    // Framework internal state machine observer subscribe function
    // This allows us to send custom internal messages to the frameworks, without polluting the user's observers.
    pub fn state_machine_framework_subscribe(
        &self,
        observer: Arc<dyn StateMachineObserver>,
    ) -> bool {
        let sm = self.state_machine.try_write();

        match sm {
            Ok(mut sm) => {
                if sm.is_none() {
                    let new_state_machine: StateMachineEngine = StateMachineEngine::default();

                    new_state_machine.framework_subscribe(observer);

                    sm.replace(new_state_machine);

                    return true;
                } else if let Some(sm) = sm.as_mut() {
                    sm.framework_subscribe(observer);
                }
            }
            Err(_) => {
                return false;
            }
        }

        true
    }

    pub fn state_machine_framework_unsubscribe(
        &self,
        observer: &Arc<dyn StateMachineObserver>,
    ) -> bool {
        let mut sm = self.state_machine.write().unwrap();

        if sm.is_none() {
            return false;
        }

        sm.as_mut().unwrap().framework_unsubscribe(observer);

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
        self.player.write().unwrap().unsubscribe(observer)
    }

    pub fn set_theme(&self, theme_id: &str) -> bool {
        self.player.write().unwrap().set_theme(theme_id)
    }

    pub fn reset_theme(&self) -> bool {
        self.player.write().unwrap().reset_theme()
    }

    pub fn state_machine_load_data(&self, state_machine: &str) -> bool {
        let new_state_machine = StateMachineEngine::new(state_machine, self.player.clone(), None);

        match new_state_machine {
            Ok(sm) => {
                if let Ok(mut state_machine) = self.state_machine.try_write() {
                    // We've called subscribe before loading a state machine
                    if (*state_machine).is_some() {
                        let tmp_sm = state_machine.as_ref().unwrap();
                        let tmp_sm_observers = tmp_sm.observers.read().unwrap().clone();
                        let tmp_sm_framework_observers =
                            tmp_sm.framework_url_observer.read().unwrap().clone();

                        if let Some(tmp_sm_framework_observers) = tmp_sm_framework_observers {
                            sm.framework_subscribe(tmp_sm_framework_observers.clone());
                        }

                        for observer in tmp_sm_observers {
                            sm.subscribe(observer.clone());
                        }
                    }

                    state_machine.replace(sm);
                }

                let player = self.player.try_write();

                match player {
                    Ok(mut player) => {
                        player.state_machine = self.state_machine.clone();
                        player.set_active_state_machine_id("");
                    }
                    Err(_) => {
                        return false;
                    }
                }
            }
            Err(error) => {
                if let Ok(state_machine) = self.state_machine.read() {
                    // We've called subscribe before loading a state machine
                    // Allows us to emit errors on the listeners
                    if (*state_machine).is_some() {
                        let tmp_sm = state_machine.as_ref().unwrap();

                        match error {
                            StateMachineEngineError::ParsingError => tmp_sm.observe_on_error("ParsingError"),
                            StateMachineEngineError::CreationError => tmp_sm.observe_on_error("CreationError"),
                            StateMachineEngineError::SecurityCheckErrorMultipleGuardlessTransitions => tmp_sm.observe_on_error("SecurityCheckErrorMultipleGuardlessTransitions"),
                            StateMachineEngineError::SecurityCheckErrorDuplicateStateName => tmp_sm.observe_on_error("SecurityCheckErrorDuplicateStateName"),
                            _ => {}
                        }
                    }
                }
                return false;
            }
        }

        true
    }

    pub fn state_machine_load(&self, state_machine_id: &str) -> bool {
        let state_machine_string = self
            .player
            .read()
            .unwrap()
            .get_state_machine(state_machine_id);

        match state_machine_string {
            Some(machine) => {
                let state_machine: Result<StateMachineEngine, StateMachineEngineError> =
                    StateMachineEngine::new(&machine, self.player.clone(), None);

                match state_machine {
                    Ok(sm) => {
                        if let Ok(mut state_machine) = self.state_machine.try_write() {
                            // We've called subscribe before loading a state machine
                            if (*state_machine).is_some() {
                                let tmp_sm = state_machine.as_ref().unwrap();
                                let tmp_sm_observers = tmp_sm.observers.read().unwrap().clone();
                                let tmp_sm_framework_observers =
                                    tmp_sm.framework_url_observer.read().unwrap().clone();

                                if let Some(tmp_sm_framework_observers) = tmp_sm_framework_observers
                                {
                                    sm.framework_subscribe(tmp_sm_framework_observers.clone());
                                }

                                for observer in tmp_sm_observers {
                                    sm.subscribe(observer.clone());
                                }
                            }

                            state_machine.replace(sm);
                        }

                        let player = self.player.try_write();

                        match player {
                            Ok(mut player) => {
                                player.state_machine = self.state_machine.clone();
                                player.set_active_state_machine_id(state_machine_id);
                            }
                            Err(_) => {
                                return false;
                            }
                        }
                    }
                    Err(_) => {
                        return false;
                    }
                }
            }
            None => {
                return false;
            }
        }
        true
    }

    pub fn set_theme_data(&self, theme_data: &str) -> bool {
        self.player.write().unwrap().set_theme_data(theme_data)
    }

    pub fn set_slots(&self, slots: &str) -> bool {
        self.player.write().unwrap().set_slots(slots)
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

    pub fn active_state_machine_id(&self) -> String {
        self.player
            .read()
            .unwrap()
            .active_state_machine_id()
            .to_string()
    }

    pub fn animation_size(&self) -> Vec<f32> {
        self.player.read().unwrap().animation_size()
    }

    pub fn state_machine_current_state(&self) -> String {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if let Some(sm) = state_machine.as_ref() {
                    return sm.get_current_state_name();
                }
            }

            Err(_) => {
                return "".to_string();
            }
        }

        "".to_string()
    }

    pub fn state_machine_status(&self) -> String {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if let Some(sm) = state_machine.as_ref() {
                    return sm.status();
                }
            }

            Err(_) => {
                return "".to_string();
            }
        }

        "".to_string()
    }

    pub fn tick(&self) -> bool {
        self.player.read().unwrap().tick()
    }

    pub fn tween(&self, to: f32, duration: Option<f32>, easing: Option<Vec<f32>>) -> bool {
        self.player.read().unwrap().tween(to, duration, easing)
    }

    pub fn tween_stop(&self) -> bool {
        self.player.read().unwrap().tween_stop()
    }

    pub fn is_tweening(&self) -> bool {
        self.player.read().unwrap().is_tweening()
    }

    pub fn tween_update(&self, progress: Option<f32>) -> bool {
        self.player.read().unwrap().tween_update(progress)
    }

    pub fn tween_to_marker(
        &self,
        marker_name: &str,
        duration: Option<f32>,
        easing: Option<Vec<f32>>,
    ) -> bool {
        self.player
            .read()
            .unwrap()
            .tween_to_marker(marker_name, duration, easing)
    }
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}
