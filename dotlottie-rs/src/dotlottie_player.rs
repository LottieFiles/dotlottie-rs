use instant::{Duration, Instant};
use std::sync::RwLock;
use std::{fs, rc::Rc, sync::Arc};

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

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
mod wasm_observer_callbacks_ffi {
    extern "C" {
        pub fn observer_on_load(dotlottie_instance_id: u32);
        pub fn observer_on_load_error(dotlottie_instance_id: u32);
        pub fn observer_on_play(dotlottie_instance_id: u32);
        pub fn observer_on_pause(dotlottie_instance_id: u32);
        pub fn observer_on_stop(dotlottie_instance_id: u32);
        pub fn observer_on_frame(dotlottie_instance_id: u32, frame_no: f32);
        pub fn observer_on_render(dotlottie_instance_id: u32, frame_no: f32);
        pub fn observer_on_loop(dotlottie_instance_id: u32, loop_count: u32);
        pub fn observer_on_complete(dotlottie_instance_id: u32);

        pub fn state_machine_observer_on_transition(
            dotlottie_instance_id: u32,
            previous_state_ptr: *const u8,
            previous_state_len: usize,
            new_state_ptr: *const u8,
            new_state_len: usize,
        );

        pub fn state_machine_observer_on_state_entered(
            dotlottie_instance_id: u32,
            entering_state_ptr: *const u8,
            entering_state_len: usize,
        );

        pub fn state_machine_observer_on_state_exit(
            dotlottie_instance_id: u32,
            leaving_state_ptr: *const u8,
            leaving_state_len: usize,
        );

        pub fn state_machine_observer_on_custom_event(
            dotlottie_instance_id: u32,
            message_ptr: *const u8,
            message_len: usize,
        );
    }
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
            state_machine_id: String::new(),
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

    // Notes: Runtime doesn't have the state machine
    // Therefor the state machine can't be loaded here, user must use the load methods.
    pub fn set_config(&mut self, new_config: Config) {
        self.update_mode(&new_config);
        self.update_background_color(&new_config);
        self.update_speed(&new_config);
        self.update_loop_animation(&new_config);
        self.update_layout(&new_config.layout);
        self.set_theme(&new_config.theme_id);

        // directly updating fields that don't require special handling
        self.config.use_frame_interpolation = new_config.use_frame_interpolation;
        self.config.segment = new_config.segment;
        self.config.autoplay = new_config.autoplay;
        self.config.marker = new_config.marker;
        self.config.theme_id = new_config.theme_id;
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

        self.load_animation_common(
            |renderer, w, h| renderer.load_data(animation_data, w, h, false),
            width,
            height,
        )
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
                    let first_animation = manager.get_active_animation();
                    if let Ok(animation_data) = first_animation {
                        self.markers = extract_markers(animation_data.as_str());
                        let animation_loaded = self.load_animation_common(
                            |renderer, w, h| renderer.load_data(&animation_data, w, h, false),
                            width,
                            height,
                        );
                        if animation_loaded && !self.config.theme_id.is_empty() {
                            self.set_theme(&self.config.theme_id.clone());
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
}

pub struct DotLottiePlayerContainer {
    runtime: RwLock<DotLottieRuntime>,
    observers: RwLock<Vec<Arc<dyn Observer>>>,
    state_machine: Rc<RwLock<Option<StateMachineEngine>>>,
    #[cfg(target_arch = "wasm32")]
    instance_id: u32,
}

impl DotLottiePlayerContainer {
    #[cfg(any(feature = "thorvg-v0", feature = "thorvg-v1"))]
    pub fn new(config: Config) -> Self {
        #[cfg(target_arch = "wasm32")]
        static NEXT_INSTANCE_ID: std::sync::atomic::AtomicU32 =
            std::sync::atomic::AtomicU32::new(1);

        DotLottiePlayerContainer {
            runtime: RwLock::new(DotLottieRuntime::new(config)),
            observers: RwLock::new(Vec::new()),
            state_machine: Rc::new(RwLock::new(None)),
            #[cfg(target_arch = "wasm32")]
            instance_id: NEXT_INSTANCE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }

    pub fn with_renderer<R: Renderer>(config: Config, renderer: R) -> Self {
        DotLottiePlayerContainer {
            runtime: RwLock::new(DotLottieRuntime::with_renderer(config, renderer)),
            observers: RwLock::new(Vec::new()),
            state_machine: Rc::new(RwLock::new(None)),
            #[cfg(target_arch = "wasm32")]
            instance_id: 0,
        }
    }

    pub fn emit_on_load(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load();
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                wasm_observer_callbacks_ffi::observer_on_load(self.instance_id);
            }

        }
    }

    pub fn emit_on_load_error(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_load_error();
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                wasm_observer_callbacks_ffi::observer_on_load_error(self.instance_id);
            }
        }
    }

    pub fn emit_on_play(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_play();
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                wasm_observer_callbacks_ffi::observer_on_play(self.instance_id);
            }
        }
    }

    pub fn emit_on_pause(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_pause();
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                wasm_observer_callbacks_ffi::observer_on_pause(self.instance_id);
            }
        }
    }

    pub fn emit_on_stop(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_stop();
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                wasm_observer_callbacks_ffi::observer_on_stop(self.instance_id);
            }
        }
    }

    pub fn emit_on_frame(&self, frame_no: f32) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_frame(frame_no);
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                wasm_observer_callbacks_ffi::observer_on_frame(self.instance_id, frame_no);
            }
        }
    }

    pub fn emit_on_render(&self, frame_no: f32) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_render(frame_no);
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                wasm_observer_callbacks_ffi::observer_on_render(self.instance_id, frame_no);
            }
        }
    }

    pub fn emit_on_loop(&self, loop_count: u32) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_loop(loop_count);
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                wasm_observer_callbacks_ffi::observer_on_loop(self.instance_id, loop_count);
            }
        }
    }

    pub fn emit_on_complete(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.observers.read().unwrap().iter().for_each(|observer| {
                observer.on_complete();
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                wasm_observer_callbacks_ffi::observer_on_complete(self.instance_id);
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

    #[cfg(target_arch = "wasm32")]
    pub fn instance_id(&self) -> u32 {
        self.instance_id
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

        return "".to_string();
    }

    pub fn hit_check(&self, layer_name: &str, x: f32, y: f32) -> bool {
        self.player.read().unwrap().hit_check(layer_name, x, y)
    }

    pub fn get_layer_bounds(&self, layer_name: &str) -> Vec<f32> {
        self.player.read().unwrap().get_layer_bounds(layer_name)
    }

    pub fn state_machine_start(&self) -> bool {
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

                        // nullify the current state machine
                        *state_machine = None;
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
                    let listeners = sm.listeners(None);

                    for listener in listeners {
                        match listener {
                            crate::listeners::Listener::PointerUp { .. } => {
                                listener_types.push("PointerUp".to_string())
                            }
                            crate::listeners::Listener::PointerDown { .. } => {
                                listener_types.push("PointerDown".to_string())
                            }
                            crate::listeners::Listener::PointerEnter { .. } => {
                                // In case framework self detects pointer entering layers, push pointerExit
                                listener_types.push("PointerEnter".to_string());
                                // We push PointerMove too so that we can do hit detection instead of the framework
                                listener_types.push("PointerMove".to_string());
                            }
                            crate::listeners::Listener::PointerMove { .. } => {
                                listener_types.push("PointerMove".to_string())
                            }
                            crate::listeners::Listener::PointerExit { .. } => {
                                // In case framework self detects pointer exiting layers, push pointerExit
                                listener_types.push("PointerExit".to_string());
                                // We push PointerMove too so that we can do hit detection instead of the framework
                                listener_types.push("PointerMove".to_string());
                            }
                            crate::listeners::Listener::OnComplete { .. } => {
                                listener_types.push("OnComplete".to_string())
                            }
                        }
                    }

                    listener_types.sort();
                    listener_types.dedup();
                    listener_types
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

    pub fn state_machine_set_numeric_trigger(&self, key: &str, value: f32) -> bool {
        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    let ret = sm.set_numeric_trigger(key, value, true, false);

                    if ret.is_some() {
                        return true;
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    pub fn state_machine_set_string_trigger(&self, key: &str, value: &str) -> bool {
        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    let ret = sm.set_string_trigger(key, value, true, false);

                    if ret.is_some() {
                        return true;
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    pub fn state_machine_set_boolean_trigger(&self, key: &str, value: bool) -> bool {
        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    let ret = sm.set_boolean_trigger(key, value, true, false);

                    if ret.is_some() {
                        return true;
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    pub fn state_machine_get_numeric_trigger(&self, key: &str) -> f32 {
        match self.state_machine.try_read() {
            Ok(state_machine) => {
                if let Some(sm) = &*state_machine {
                    if let Some(value) = sm.get_numeric_trigger(key) {
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

    pub fn state_machine_get_string_trigger(&self, key: &str) -> String {
        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    if let Some(value) = sm.get_string_trigger(key) {
                        return value;
                    }
                }
            }
            Err(_) => return "".to_string(),
        }

        "".to_string()
    }

    pub fn state_machine_get_boolean_trigger(&self, key: &str) -> bool {
        match self.state_machine.try_write() {
            Ok(mut state_machine) => {
                if let Some(sm) = state_machine.as_mut() {
                    if let Some(value) = sm.get_boolean_trigger(key) {
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

                    let start = self.state_machine_start();

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

                    let start = self.state_machine_start();

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

                    let start = self.state_machine_start();

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

    #[cfg(not(target_arch = "wasm32"))]
    pub fn state_machine_subscribe(&self, observer: Arc<dyn StateMachineObserver>) -> bool {
        let mut sm = self.state_machine.write().unwrap();

        if sm.is_none() {
            return false;
        }
        sm.as_mut().unwrap().subscribe(observer);

        true
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn state_machine_unsubscribe(&self, observer: &Arc<dyn StateMachineObserver>) -> bool {
        let mut sm = self.state_machine.write().unwrap();

        if sm.is_none() {
            return false;
        }

        sm.as_mut().unwrap().unsubscribe(observer);

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
        let state_machine = StateMachineEngine::new(state_machine, self.player.clone(), None);

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
                    player.set_active_state_machine_id("");
                    return true;
                }
                Err(_) => {
                    return false;
                }
            }
        }

        false
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

    #[cfg(target_arch = "wasm32")]
    pub fn instance_id(&self) -> u32 {
        self.player.read().unwrap().instance_id()
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
}

unsafe impl Send for DotLottiePlayer {}
unsafe impl Sync for DotLottiePlayer {}
