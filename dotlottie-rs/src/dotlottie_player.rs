use crate::{
    fms::{DotLottieManager, Manifest},
    lottie_renderer::{LottieRenderer, LottieRendererError, Renderer},
    markers::extract_markers,
    theming::transform_theme_to_lottie_slots,
};
use crate::{
    layout::Layout,
    markers::{Marker, MarkersMap},
};
use instant::{Duration, Instant};
use std::fs;

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
pub enum Direction {
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

pub struct DotLottiePlayer {
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
}

impl DotLottiePlayer {
    #[cfg(feature = "thorvg")]
    pub fn new(config: Config) -> Self {
        Self::with_renderer(
            config,
            crate::lottie_renderer::TvgRenderer::new(
                crate::lottie_renderer::TvgEngine::TvgEngineSw,
                0,
            ),
        )
    }

    pub fn with_renderer<R: Renderer>(config: Config, renderer: R) -> Self {
        let direction = match config.mode {
            Mode::Forward => Direction::Forward,
            Mode::Reverse => Direction::Reverse,
            Mode::Bounce => Direction::Forward,
            Mode::ReverseBounce => Direction::Reverse,
        };

        DotLottiePlayer {
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

    pub fn animation_size(&self) -> (f32, f32) {
        (
            self.renderer.picture_width() as f32,
            self.renderer.picture_height() as f32,
        )
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
            .map_or(false, |themes| {
                themes.iter().any(|theme| theme.id == theme_id)
            });

        if !theme_exists {
            return false;
        }

        let can_set_theme = self.manifest().map_or(false, |manifest| {
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
}
