use crate::time::{Duration, Instant};
use std::fs;

use crate::poll_events::{DotLottieEvent, EventQueue};
use crate::{
    extract_markers,
    layout::Layout,
    lottie_renderer::{LottieRenderer, LottieRendererError},
    Marker, MarkersMap,
};
use crate::{
    transform_theme_to_lottie_slots, DotLottieManager, Manifest, Renderer, StateMachineEngine,
    StateMachineEngineError,
};

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
    pub loop_count: u32,
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
            loop_count: 0,
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
    active_state_machine_id: String,
    cached_start_end_frame: Option<(f32, f32)>,
    event_queue: EventQueue<DotLottieEvent>,
}

impl DotLottiePlayer {
    #[cfg(feature = "tvg")]
    pub fn new(config: Config, threads: u32) -> Self {
        Self::with_renderer(config, crate::TvgRenderer::new(threads))
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
            active_state_machine_id: String::new(),
            cached_start_end_frame: None,
            event_queue: EventQueue::new(),
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

    fn is_valid_segment(segment: &[f32]) -> bool {
        if segment.len() != 2 {
            return false;
        }
        segment[0] < segment[1]
    }

    fn compute_start_frame(&self) -> f32 {
        if !self.config.marker.is_empty() {
            if let Some((time, _)) = self.markers.get(&self.config.marker) {
                return (*time).max(0.0);
            }
        }

        if Self::is_valid_segment(&self.config.segment) {
            return self.config.segment[0].max(0.0);
        }

        0.0
    }

    fn compute_end_frame(&self) -> f32 {
        if !self.config.marker.is_empty() {
            if let Some((time, duration)) = self.markers.get(&self.config.marker) {
                return (time + duration).min(self.total_frames() - 1.0);
            }
        }

        if Self::is_valid_segment(&self.config.segment) {
            return self.config.segment[1].min(self.total_frames() - 1.0);
        }

        self.total_frames() - 1.0
    }

    fn start_frame(&self) -> f32 {
        if let Some((start, _)) = self.cached_start_end_frame {
            start
        } else {
            self.compute_start_frame()
        }
    }

    fn end_frame(&self) -> f32 {
        if let Some((_, end)) = self.cached_start_end_frame {
            end
        } else {
            self.compute_end_frame()
        }
    }

    fn invalidate_frame_cache(&mut self) {
        let start = self.compute_start_frame();
        let end = self.compute_end_frame();
        self.cached_start_end_frame = Some((start, end));
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

        self.event_queue.push(DotLottieEvent::Play);

        true
    }

    pub fn pause(&mut self) -> bool {
        if self.is_loaded && self.is_playing() {
            self.playback_state = PlaybackState::Paused;
            self.event_queue.push(DotLottieEvent::Pause);
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
            self.event_queue.push(DotLottieEvent::Stop);

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

    fn should_increment_loop(&self) -> bool {
        if !self.config.loop_animation {
            return false;
        }

        // Unlimited looping: always increment
        if self.config.loop_count == 0 {
            return true;
        }

        // Counted looping: increment until reaching the configured count
        self.loop_count < self.config.loop_count
    }

    fn handle_forward_mode(&mut self, next_frame: f32, end_frame: f32) -> f32 {
        if next_frame >= end_frame {
            if self.should_increment_loop() {
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
            if self.should_increment_loop() {
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
                    if self.should_increment_loop() {
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
                    if self.should_increment_loop() {
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

        let is_ok = self.renderer.set_frame(no).is_ok();

        if is_ok {
            self.event_queue
                .push(DotLottieEvent::Frame { frame_no: no });
        }

        is_ok
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

        if is_ok {
            let frame_no = self.current_frame();

            self.event_queue.push(DotLottieEvent::Render { frame_no });

            if self.is_complete() {
                if self.config().loop_animation {
                    let count_complete = self.config().loop_count > 0
                        && self.loop_count() >= self.config().loop_count;

                    if count_complete {
                        // Put the animation in a stop state, otherwise we can keep looping if we call tick()
                        // Do it before emiting complete, otherwise it will pause the animation at the wrong stages in state machines
                        self.stop();
                    }

                    self.event_queue.push(DotLottieEvent::Loop {
                        loop_count: self.loop_count(),
                    });

                    if count_complete {
                        self.event_queue.push(DotLottieEvent::Complete);
                        self.reset_loop_count();
                    }
                } else if !self.config().loop_animation {
                    self.event_queue.push(DotLottieEvent::Complete);
                }
            }
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

    pub fn reset_loop_count(&mut self) {
        self.loop_count = 0;
    }

    pub fn speed(&self) -> f32 {
        self.config.speed
    }

    pub fn buffer(&self) -> &[u32] {
        self.renderer.buffer()
    }

    pub fn animation_size(&self) -> Vec<f32> {
        vec![
            self.renderer.picture_width(),
            self.renderer.picture_height(),
        ]
    }

    pub fn clear(&mut self) {
        self.renderer.clear()
    }

    // Notes: Player doesn't have the state machine
    // Therefor the state machine can't be loaded here, user must use the load methods.
    pub fn set_config(&mut self, new_config: Config) {
        self.update_mode(&new_config);
        self.update_background_color(&new_config);
        self.update_speed(&new_config);
        self.update_loop_animation(&new_config);
        self.update_loop_count(&new_config);
        self.update_marker(&new_config.marker);
        self.update_layout(&new_config.layout);
        self.set_theme(&new_config.theme_id);

        // directly updating fields that don't require special handling
        self.config.use_frame_interpolation = new_config.use_frame_interpolation;

        if Self::is_valid_segment(&new_config.segment) {
            self.config.segment = new_config.segment;
            self.invalidate_frame_cache();
        }
        self.config.autoplay = new_config.autoplay;
        self.config.animation_id = new_config.animation_id;

        if new_config.autoplay {
            self.play();
        } else {
            self.pause();
        }
    }

    pub fn update_marker(&mut self, marker: &String) {
        if self.config.marker == *marker {
            return;
        }

        let markers = self.markers();

        if let Some(marker) = markers.iter().find(|m| m.name == *marker) {
            self.start_time = Instant::now();

            self.config.marker = marker.name.clone();
            self.invalidate_frame_cache();

            self.set_frame(marker.time);

            self.render();
        } else {
            self.config.marker = String::new();
            self.invalidate_frame_cache();
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

    fn update_loop_count(&mut self, new_config: &Config) {
        if self.config.loop_count != new_config.loop_count {
            self.loop_count = 0;
            self.config.loop_count = new_config.loop_count;
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

        self.invalidate_frame_cache();

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
            |renderer, w, h| renderer.load_data(animation_data, w, h),
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

            self.event_queue.push(DotLottieEvent::Load);
            if self.config().autoplay {
                self.play();
            }
        } else {
            self.event_queue.push(DotLottieEvent::LoadError);
        }

        animation_loaded
    }

    pub fn load_animation_path(&mut self, file_path: &str, width: u32, height: u32) -> bool {
        self.dotlottie_manager = None;
        self.active_animation_id.clear();
        self.active_theme_id.clear();

        let load_status = match fs::read_to_string(file_path) {
            Ok(data) => self.load_animation_data(&data, width, height),
            Err(_) => {
                self.event_queue.push(DotLottieEvent::LoadError);
                false
            }
        };

        load_status
    }

    pub fn load_dotlottie_data(&mut self, file_data: &[u8], width: u32, height: u32) -> bool {
        self.active_animation_id.clear();
        self.active_theme_id.clear();

        let loaded = match DotLottieManager::new(file_data) {
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

                    // Returns this result
                    if let Ok(animation_data) = active_animation {
                        self.markers = extract_markers(animation_data.as_str());
                        let animation_loaded = self.load_animation_common(
                            |renderer, w, h| renderer.load_data(&animation_data, w, h),
                            width,
                            height,
                        );

                        if animation_loaded {
                            self.active_animation_id = active_animation_id;
                            if !self.config.theme_id.is_empty() {
                                self.set_theme(&self.config.theme_id.clone());
                            }
                        }

                        animation_loaded
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Err(_) => false,
        };

        // container
        if loaded {
            self.event_queue.push(DotLottieEvent::Load);

            if self.config().autoplay {
                self.play();
            }
        } else {
            self.event_queue.push(DotLottieEvent::LoadError);

            return false;
        }

        loaded
    }

    pub fn load_animation(&mut self, animation_id: &str, width: u32, height: u32) -> bool {
        self.active_animation_id.clear();
        if let Some(manager) = &mut self.dotlottie_manager {
            let animation_data = manager.get_animation(animation_id);

            let ok = match animation_data {
                Ok(animation_data) => self.load_animation_common(
                    |renderer, w, h| renderer.load_data(&animation_data, w, h),
                    width,
                    height,
                ),
                Err(_error) => false,
            };

            if ok {
                self.active_animation_id = animation_id.to_string();

                if !self.config.theme_id.is_empty() {
                    self.set_theme(&self.config.theme_id.clone());
                }

                self.event_queue.push(DotLottieEvent::Load);
                if self.config().autoplay {
                    self.play();
                }
            } else {
                self.event_queue.push(DotLottieEvent::LoadError);
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
                // Enables firing loop_complete if loop_count is enabled.
                // Avoid firing at initial start frame before any loop has completed.
                if self.config().loop_animation && self.config().loop_count > 0 {
                    self.loop_count() > 0 && self.current_frame() <= self.start_frame()
                } else {
                    // Enables firing complete if loop_animation = false
                    self.current_frame() <= self.start_frame()
                        && self.direction == Direction::Reverse
                }
            }
            Mode::ReverseBounce => {
                // Enables firing loop_complete if loop_count is enabled.
                // Avoid firing at initial end frame before any loop has completed.
                if self.config().loop_animation && self.config().loop_count > 0 {
                    self.loop_count() > 0 && self.current_frame() >= self.end_frame()
                } else {
                    // Enables firing complete if loop_animation = false
                    self.current_frame() >= self.end_frame() && self.direction == Direction::Forward
                }
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
        self.config.theme_id.clear();

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
            self.config.theme_id = theme_id.to_string();
        }

        ok
    }

    pub fn reset_theme(&mut self) -> bool {
        self.active_theme_id.clear();
        self.config.theme_id.clear();
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

    pub fn set_quality(&mut self, quality: u8) -> bool {
        self.renderer.set_quality(quality).is_ok()
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

    pub fn get_transform(&self) -> Vec<f32> {
        self.renderer
            .get_transform()
            .unwrap_or([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
            .to_vec()
    }

    pub fn set_transform(&mut self, transform: Vec<f32>) -> bool {
        if transform.len() != 9 {
            return false;
        }
        let transform_array: [f32; 9] = [
            transform[0],
            transform[1],
            transform[2],
            transform[3],
            transform[4],
            transform[5],
            transform[6],
            transform[7],
            transform[8],
        ];
        self.renderer.set_transform(&transform_array).is_ok()
    }

    /// Poll for the next event from the event queue
    ///
    /// Returns Some(event) if an event is available, None if the queue is empty.
    pub fn poll_event(&mut self) -> Option<DotLottieEvent> {
        self.event_queue.poll()
    }

    pub fn tick(&mut self) -> bool {
        if self.is_tweening() {
            self.tween_update(None) && self.render()
        } else {
            let next_frame = self.request_frame();

            let _ = self.set_frame(next_frame);

            let rendered = self.render();

            rendered
        }
    }

    pub fn state_machine_load<'a>(
        &'a mut self,
        state_machine_id: &str,
    ) -> Result<StateMachineEngine<'a>, StateMachineEngineError> {
        let machine = match self.get_state_machine(state_machine_id) {
            Some(m) => m, // String is owned, no borrow issue
            None => return Err(StateMachineEngineError::CreationError),
        };

        self.set_active_state_machine_id(state_machine_id);

        StateMachineEngine::new(&machine, self, None)
    }

    pub fn state_machine_load_data<'a>(
        &'a mut self,
        state_machine: &str,
    ) -> Result<StateMachineEngine<'a>, StateMachineEngineError> {
        StateMachineEngine::new(state_machine, self, None)
    }
}
