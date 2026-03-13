use crate::time::{Duration, Instant};
use std::ffi::{CStr, CString};
use std::{fs, mem};

use crate::poll_events::{DotLottieEvent, EventQueue};
use crate::DotLottiePlayerError;
use crate::{
    extract_markers,
    layout::Layout,
    lottie_renderer::{LottieRenderer, LottieRendererError},
    Marker,
};
use crate::{ColorSpace, Renderer};
#[cfg(feature = "dotlottie")]
use crate::{DotLottieManager, Manifest};
#[cfg(feature = "state-machines")]
use crate::{StateMachineEngine, StateMachineEngineError};

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

pub const DEFAULT_BACKGROUND_COLOR: u32 = 0x00000000;

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

// This is used to pass the loop complete / complete event to the state machine engine
pub enum CompletionEvent {
    None,
    Completed,
    LoopCompleted,
}

pub struct DotLottiePlayer {
    renderer: Box<dyn LottieRenderer>,
    playback_state: PlaybackState,
    is_loaded: bool,
    start_time: Instant,
    current_loop_count: u32,
    #[cfg(feature = "dotlottie")]
    dotlottie_manager: Option<DotLottieManager>,
    direction: Direction,
    marker_names: Vec<CString>,
    marker_data: Vec<(f32, f32)>, // (time, duration)
    cached_start_end_frame: Option<(f32, f32)>,
    event_queue: EventQueue<DotLottieEvent>,
    completion_event: CompletionEvent,
    // Playback config properties
    mode: Mode,
    loop_animation: bool,
    loop_count: u32,
    speed: f32,
    use_frame_interpolation: bool,
    autoplay: bool,
    segment: Option<[f32; 2]>,
    background_color: u32,
    layout: Layout,
    marker: Option<usize>, // marker id
    #[cfg(feature = "theming")]
    theme_id: Option<CString>,
    #[cfg(feature = "dotlottie")]
    animation_id: Option<CString>,
    #[cfg(feature = "state-machines")]
    state_machine_id: Option<CString>,
}

#[cfg(feature = "tvg")]
impl Default for DotLottiePlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl DotLottiePlayer {
    #[cfg(feature = "tvg")]
    pub fn new() -> Self {
        Self::with_renderer(crate::TvgRenderer::new(0))
    }

    #[cfg(feature = "tvg")]
    pub fn with_threads(threads: u32) -> Self {
        Self::with_renderer(crate::TvgRenderer::new(threads))
    }

    #[cfg(feature = "tvg")]
    pub fn load_font(name: &str, data: &[u8]) -> Result<(), DotLottiePlayerError> {
        use crate::lottie_renderer::Renderer;
        crate::TvgRenderer::load_font(name, data).map_err(|_| DotLottiePlayerError::Unknown)
    }

    #[cfg(feature = "tvg")]
    pub fn unload_font(name: &str) -> Result<(), DotLottiePlayerError> {
        use crate::lottie_renderer::Renderer;
        crate::TvgRenderer::unload_font(name).map_err(|_| DotLottiePlayerError::Unknown)
    }

    pub fn with_renderer<R: Renderer>(renderer: R) -> Self {
        DotLottiePlayer {
            renderer: <dyn LottieRenderer>::new(renderer),
            playback_state: PlaybackState::Stopped,
            is_loaded: false,
            start_time: Instant::now(),
            current_loop_count: 0,
            mode: Mode::Forward,
            loop_animation: false,
            loop_count: 0,
            speed: 1.0,
            use_frame_interpolation: true,
            autoplay: false,
            segment: None,
            background_color: DEFAULT_BACKGROUND_COLOR,
            layout: Layout::default(),
            marker: None,
            #[cfg(feature = "theming")]
            theme_id: None,
            #[cfg(feature = "dotlottie")]
            animation_id: None,
            #[cfg(feature = "dotlottie")]
            dotlottie_manager: None,
            direction: Direction::Forward,
            marker_names: Vec::new(),
            marker_data: Vec::new(),
            #[cfg(feature = "state-machines")]
            state_machine_id: None,
            cached_start_end_frame: None,
            event_queue: EventQueue::new(),
            completion_event: CompletionEvent::None,
        }
    }

    pub fn markers(&self) -> Vec<Marker> {
        self.marker_names
            .iter()
            .zip(self.marker_data.iter())
            .map(|(name, (time, duration))| Marker {
                name: name.to_string_lossy().into_owned(),
                time: *time,
                duration: *duration,
            })
            .collect()
    }

    pub fn marker_names(&self) -> &[CString] {
        &self.marker_names
    }

    pub fn marker_data(&self) -> &[(f32, f32)] {
        &self.marker_data
    }

    fn find_marker(&self, name: &CStr) -> Option<(usize, f32, f32)> {
        self.marker_names
            .iter()
            .position(|n| n.as_c_str() == name)
            .map(|idx| {
                let (time, duration) = self.marker_data[idx];
                (idx, time, duration)
            })
    }

    pub fn pop_completion_event(&mut self) -> CompletionEvent {
        mem::replace(&mut self.completion_event, CompletionEvent::None)
    }

    fn is_valid_segment(segment: &[f32]) -> bool {
        segment[0] < segment[1]
    }

    fn compute_start_frame(&self) -> f32 {
        if let Some(idx) = &self.marker {
            if let Some((time, _)) = self.marker_data.get(*idx) {
                return time.max(0.0);
            }
        }

        if let Some(segment) = self.segment {
            return segment[0].max(0.0);
        }

        0.0
    }

    fn compute_end_frame(&self) -> f32 {
        let max_frame = self.total_frames() - 1.0;

        if let Some(idx) = &self.marker {
            if let Some((time, duration)) = self.marker_data.get(*idx) {
                return (time + duration).min(max_frame);
            }
        }

        if let Some(segment) = self.segment {
            return segment[1].min(max_frame);
        }

        max_frame
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

    pub fn play(&mut self) -> Result<(), DotLottiePlayerError> {
        if !self.is_loaded {
            return Err(DotLottiePlayerError::AnimationNotLoaded);
        }
        if self.is_playing() {
            return Err(DotLottiePlayerError::InsufficientCondition);
        }

        if self.is_complete() && self.is_stopped() {
            self.start_time = Instant::now();
            match self.mode {
                Mode::Forward | Mode::Bounce => {
                    let _ = self.set_frame(self.start_frame());
                    self.direction = Direction::Forward;
                }
                Mode::Reverse | Mode::ReverseBounce => {
                    let _ = self.set_frame(self.end_frame());
                    self.direction = Direction::Reverse;
                }
            }
        } else {
            self.update_start_time_for_frame(self.current_frame());
        }

        self.playback_state = PlaybackState::Playing;

        self.event_queue.push(DotLottieEvent::Play);

        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), DotLottiePlayerError> {
        if !self.is_loaded {
            return Err(DotLottiePlayerError::AnimationNotLoaded);
        }
        if !self.is_playing() {
            return Err(DotLottiePlayerError::InsufficientCondition);
        }
        self.playback_state = PlaybackState::Paused;
        self.event_queue.push(DotLottieEvent::Pause);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), DotLottiePlayerError> {
        if !self.is_loaded {
            return Err(DotLottiePlayerError::AnimationNotLoaded);
        }
        if self.is_stopped() {
            return Err(DotLottiePlayerError::InsufficientCondition);
        }

        self.playback_state = PlaybackState::Stopped;

        let start_frame = self.start_frame();
        let end_frame = self.end_frame();

        match self.mode {
            Mode::Forward | Mode::Bounce => {
                let _ = self.set_frame(start_frame);
            }
            Mode::Reverse | Mode::ReverseBounce => {
                let _ = self.set_frame(end_frame);
            }
        }
        self.event_queue.push(DotLottieEvent::Stop);

        Ok(())
    }

    #[cfg(feature = "dotlottie")]
    pub fn manifest(&self) -> Option<&Manifest> {
        self.dotlottie_manager
            .as_ref()
            .map(|manager| manager.manifest())
    }

    pub fn size(&self) -> (u32, u32) {
        (self.renderer.width(), self.renderer.height())
    }

    #[cfg(feature = "state-machines")]
    pub fn get_state_machine(&self, state_machine_id: &CStr) -> Option<String> {
        let id_str = state_machine_id.to_str().ok()?;

        self.dotlottie_manager
            .as_ref()
            .and_then(|manager| manager.get_state_machine(id_str).ok())
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
        let effective_duration = (duration * effective_total_frames / total_frames) / self.speed;

        let raw_next_frame = (elapsed_time / effective_duration) * effective_total_frames;

        // update the next frame based on the direction
        let mut next_frame = match self.direction {
            Direction::Forward => start_frame + raw_next_frame,
            Direction::Reverse => end_frame - raw_next_frame,
        };

        // Apply frame interpolation
        next_frame = if self.use_frame_interpolation {
            (next_frame * 1000.0).round() * 0.001
        } else {
            next_frame.round()
        };

        // Clamp the next frame to the start & end frames
        next_frame = next_frame.clamp(start_frame, end_frame);

        // Handle different modes
        next_frame = match self.mode {
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
        if !self.loop_animation {
            return false;
        }

        // Unlimited looping: always increment
        if self.loop_count == 0 {
            return true;
        }

        // Counted looping: increment until reaching the configured count
        self.current_loop_count < self.loop_count
    }

    fn handle_forward_mode(&mut self, next_frame: f32, end_frame: f32) -> f32 {
        if next_frame >= end_frame {
            if self.should_increment_loop() {
                self.current_loop_count += 1;
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
                self.current_loop_count += 1;
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
                        self.current_loop_count += 1;
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
                        self.current_loop_count += 1;
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

        if duration.is_finite() && duration > 0.0 && self.speed > 0.0 {
            let effective_duration =
                (duration * effective_total_frames / total_frames) / self.speed;

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
    /// Returns `Ok(())` if the frame number is valid and updated and an error variant otherwise.
    ///
    /// The frame number is considered valid if it's within the range of the start and end frames.
    ///
    /// This function does not update the start time for the new frame assuming it's already managed by the `request_frame` method in the animation loop.
    /// It's the responsibility of the caller to update the start time if needed.
    ///
    pub fn set_frame(&mut self, no: f32) -> Result<(), DotLottiePlayerError> {
        if no < self.start_frame() || no > self.end_frame() {
            return Err(DotLottiePlayerError::InvalidParameter);
        }

        self.renderer.set_frame(no)?;
        self.event_queue
            .push(DotLottieEvent::Frame { frame_no: no });
        Ok(())
    }

    /// Seek to a specific frame number.
    ///
    /// # Arguments
    ///
    /// * `no` - The frame number to seek to.
    ///
    /// # Returns
    ///
    /// Returns `DotLottieResult::Success` if the frame number is valid and updated and an error variant otherwise.
    ///
    /// The frame number is considered valid if it's within the range of the start and end frames.
    ///
    /// The start time is updated based on the new frame number.
    ///
    pub fn seek(&mut self, no: f32) -> Result<(), DotLottiePlayerError> {
        self.set_frame(no)?;
        self.update_start_time_for_frame(no);
        Ok(())
    }

    pub fn set_viewport(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) -> Result<(), DotLottiePlayerError> {
        self.renderer.set_viewport(x, y, w, h)?;
        Ok(())
    }

    fn emit_on_complete(&mut self) {
        self.completion_event = CompletionEvent::Completed;
        self.event_queue.push(DotLottieEvent::Complete);
    }

    pub fn emit_on_loop(&mut self) {
        self.completion_event = CompletionEvent::LoopCompleted;
        self.event_queue.push(DotLottieEvent::Loop {
            loop_count: self.current_loop_count(),
        });
    }

    pub fn render(&mut self) -> Result<(), DotLottiePlayerError> {
        self.renderer.render()?;

        // rendered the last frame successfully
        if self.is_complete() && !self.loop_animation {
            self.playback_state = PlaybackState::Stopped;
        }

        let frame_no = self.current_frame();

        self.event_queue.push(DotLottieEvent::Render { frame_no });

        if self.is_complete() {
            if self.loop_animation {
                let count_complete =
                    self.loop_count > 0 && self.current_loop_count() >= self.loop_count;

                if count_complete {
                    // Put the animation in a stop state, otherwise we can keep looping if we call tick()
                    // Do it before emiting complete, otherwise it will pause the animation at the wrong stages in state machines
                    let _ = self.stop();
                }

                self.emit_on_loop();

                if count_complete {
                    self.emit_on_complete();
                    self.reset_current_loop_count();
                }
            } else if !self.loop_animation {
                self.emit_on_complete();
            }
        }

        Ok(())
    }

    pub fn total_frames(&self) -> f32 {
        self.renderer.total_frames().unwrap_or(0.0)
    }

    pub fn duration(&self) -> f32 {
        self.renderer.duration().unwrap_or(0.0)
    }

    pub fn segment_duration(&self) -> f32 {
        // If segment is None, returns animation duration
        if self.segment.is_none() {
            return self.duration();
        };

        let start_frame = self.start_frame();
        let end_frame = self.end_frame();

        let frames_to_seconds = self.duration() / self.total_frames();

        (end_frame - start_frame) * frames_to_seconds
    }

    pub fn current_frame(&self) -> f32 {
        self.renderer.current_frame()
    }

    pub fn current_loop_count(&self) -> u32 {
        self.current_loop_count
    }

    pub fn reset_current_loop_count(&mut self) {
        self.current_loop_count = 0;
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

    pub fn set_marker(&mut self, marker_name: Option<&CStr>) {
        // Get current marker name from index
        let current_name = self
            .marker
            .and_then(|idx| self.marker_names.get(idx))
            .map(|n| n.as_c_str());
        if current_name == marker_name {
            return;
        }

        if let Some(name) = marker_name {
            if let Some((idx, time, _)) = self.find_marker(name) {
                self.start_time = Instant::now();
                self.marker = Some(idx);
                self.invalidate_frame_cache();

                let _ = self.set_frame(time);
                let _ = self.render();
            } else {
                self.marker = None;
                self.invalidate_frame_cache();
            }
        } else {
            self.marker = None;
            self.invalidate_frame_cache();
        }
    }

    pub fn marker(&self) -> Option<&CStr> {
        self.marker
            .and_then(|idx| self.marker_names.get(idx))
            .map(|n| n.as_c_str())
    }

    pub fn set_layout(&mut self, layout: Layout) -> Result<(), LottieRendererError> {
        self.renderer.set_layout(&layout)?;

        self.layout = layout;

        Ok(())
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn set_mode(&mut self, mode: Mode) {
        if self.mode != mode {
            self.flip_direction_if_needed(mode);
            self.mode = mode;
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
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

    pub fn set_background_color(&mut self, color: Option<u32>) -> Result<(), DotLottiePlayerError> {
        let new_color = color.unwrap_or(DEFAULT_BACKGROUND_COLOR);

        if self.background_color == new_color {
            return Ok(());
        }

        if self.renderer.set_background_color(new_color).is_ok() {
            self.background_color = new_color;
            Ok(())
        } else {
            Err(DotLottiePlayerError::Unknown)
        }
    }

    pub fn background_color(&self) -> u32 {
        self.background_color
    }

    pub fn set_speed(&mut self, speed: f32) {
        if self.speed != speed && speed > 0.0 {
            self.speed = speed;

            self.update_start_time_for_frame(self.current_frame());
        }
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn set_loop(&mut self, loop_animation: bool) {
        if self.loop_animation != loop_animation {
            self.current_loop_count = 0;
            self.loop_animation = loop_animation;
        }
    }

    pub fn loop_animation(&self) -> bool {
        self.loop_animation
    }

    pub fn set_loop_count(&mut self, loop_count: u32) {
        if self.loop_count != loop_count {
            self.current_loop_count = 0;
            self.loop_count = loop_count;
        }
    }

    pub fn loop_count(&self) -> u32 {
        self.loop_count
    }

    pub fn set_autoplay(&mut self, autoplay: bool) {
        self.autoplay = autoplay;
        if autoplay {
            let _ = self.play();
        } else {
            let _ = self.pause();
        }
    }

    pub fn autoplay(&self) -> bool {
        self.autoplay
    }

    pub fn set_use_frame_interpolation(&mut self, enabled: bool) {
        self.use_frame_interpolation = enabled;
    }

    pub fn use_frame_interpolation(&self) -> bool {
        self.use_frame_interpolation
    }

    pub fn set_segment(&mut self, segment: Option<[f32; 2]>) -> Result<(), DotLottiePlayerError> {
        if self.segment == segment {
            return Ok(());
        }

        if let Some(seg) = &segment {
            if !Self::is_valid_segment(seg) {
                return Err(DotLottiePlayerError::InvalidParameter);
            }
        }

        self.segment = segment;
        self.invalidate_frame_cache();

        Ok(())
    }

    pub fn segment(&self) -> Option<[f32; 2]> {
        self.segment
    }

    /// Set software rendering target using a safe Rust slice.
    ///
    /// This is the preferred safe API. The buffer must be large enough to hold
    /// width * height pixels.
    ///
    /// # Returns
    /// `Err(InvalidParameter)` if the buffer is too small, `Err` on setup failure.
    pub fn set_sw_target(
        &mut self,
        buffer: &mut [u32],
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Result<(), DotLottiePlayerError> {
        let required_size = (width * height) as usize;
        if buffer.len() < required_size {
            return Err(DotLottiePlayerError::InvalidParameter);
        }

        let stride = width;
        self.renderer
            .set_sw_target(buffer, stride, width, height, color_space)?;

        Ok(())
    }

    /// Set OpenGL rendering target.
    ///
    /// The GL context must remain valid while the player is using it and must be
    /// current on the calling thread when rendering.
    pub fn set_gl_target<C: crate::lottie_renderer::GlContext>(
        &mut self,
        context: &C,
        id: i32,
        width: u32,
        height: u32,
    ) -> Result<(), DotLottiePlayerError> {
        unsafe {
            self.renderer
                .set_gl_target(context.as_ptr(), id, width, height)?;
        }

        Ok(())
    }

    /// Set WebGPU rendering target.
    ///
    /// All WebGPU objects must remain valid while the player is using them.
    #[allow(clippy::too_many_arguments)]
    pub fn set_wg_target<
        D: crate::lottie_renderer::WgpuDevice,
        I: crate::lottie_renderer::WgpuInstance,
        T: crate::lottie_renderer::WgpuTarget,
    >(
        &mut self,
        device: &D,
        instance: &I,
        target: &T,
        width: u32,
        height: u32,
        target_type: crate::lottie_renderer::WgpuTargetType,
    ) -> Result<(), DotLottiePlayerError> {
        unsafe {
            self.renderer.set_wg_target(
                device.as_ptr(),
                instance.as_ptr(),
                target.as_ptr(),
                width,
                height,
                target_type,
            )?;
        }

        Ok(())
    }

    fn load_animation_common<F>(
        &mut self,
        loader: F,
        width: u32,
        height: u32,
    ) -> Result<(), DotLottiePlayerError>
    where
        F: FnOnce(&mut dyn LottieRenderer, u32, u32) -> Result<(), LottieRendererError>,
    {
        self.clear();
        self.playback_state = PlaybackState::Stopped;
        self.start_time = Instant::now();
        self.current_loop_count = 0;

        let loaded = loader(&mut *self.renderer, width, height).is_ok()
            && self
                .renderer
                .set_background_color(self.background_color)
                .is_ok();

        if self.renderer.set_layout(&self.layout).is_err() {
            return Err(DotLottiePlayerError::Unknown);
        }

        self.is_loaded = loaded;

        self.invalidate_frame_cache();

        let start_frame = self.start_frame();
        let end_frame = self.end_frame();

        match self.mode {
            Mode::Forward | Mode::Bounce => {
                let _ = self.set_frame(start_frame);
                self.direction = Direction::Forward;
            }
            Mode::Reverse | Mode::ReverseBounce => {
                let _ = self.set_frame(end_frame);
                self.direction = Direction::Reverse;
            }
        }

        if loaded {
            Ok(())
        } else {
            Err(DotLottiePlayerError::Unknown)
        }
    }

    pub fn load_animation_data(
        &mut self,
        animation_data: &CStr,
        width: u32,
        height: u32,
    ) -> Result<(), DotLottiePlayerError> {
        #[cfg(feature = "dotlottie")]
        {
            self.dotlottie_manager = None;
            self.animation_id = None;
        }
        #[cfg(feature = "theming")]
        {
            self.theme_id = None;
        }

        // Convert to &str only for marker extraction (JSON parsing)
        if let Ok(data_str) = animation_data.to_str() {
            let (names, data) = extract_markers(data_str);
            self.marker_names = names;
            self.marker_data = data;
        }

        let result = self.load_animation_common(
            |renderer, w, h| renderer.load_data(animation_data, w, h),
            width,
            height,
        );

        if result.is_ok() {
            self.event_queue.push(DotLottieEvent::Load);
            if self.autoplay {
                let _ = self.play();
            }
        } else {
            self.event_queue.push(DotLottieEvent::LoadError);
        }

        result
    }

    pub fn load_animation_path(
        &mut self,
        file_path: &CStr,
        width: u32,
        height: u32,
    ) -> Result<(), DotLottiePlayerError> {
        #[cfg(feature = "dotlottie")]
        {
            self.dotlottie_manager = None;
            self.animation_id = None;
        }
        #[cfg(feature = "theming")]
        {
            self.theme_id = None;
        }
        let result = (|| {
            let path_str = file_path
                .to_str()
                .map_err(|_| DotLottiePlayerError::InvalidParameter)?;
            let data =
                fs::read_to_string(path_str).map_err(|_| DotLottiePlayerError::InvalidParameter)?;
            let c_data = CString::new(data).map_err(|_| DotLottiePlayerError::InvalidParameter)?;

            self.load_animation_data(&c_data, width, height)
        })();

        result.inspect_err(|_| {
            self.event_queue.push(DotLottieEvent::LoadError);
        })
    }

    #[cfg(feature = "dotlottie")]
    pub fn load_dotlottie_data(
        &mut self,
        file_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), DotLottiePlayerError> {
        #[cfg(feature = "dotlottie")]
        {
            self.animation_id = None;
        }
        #[cfg(feature = "theming")]
        {
            self.theme_id = None;
        }
        let manager =
            DotLottieManager::new(file_data).map_err(|_| DotLottiePlayerError::Unknown)?;

        let (active_animation, active_animation_id) =
            if let Some(anim_id) = self.animation_id.as_deref().and_then(|c| c.to_str().ok()) {
                (manager.get_animation(anim_id), self.animation_id.clone())
            } else {
                (
                    manager.get_active_animation(),
                    CString::new(manager.active_animation_id()).ok(),
                )
            };

        let animation_data = active_animation.map_err(|_| DotLottiePlayerError::Unknown)?;

        let (names, data) = extract_markers(&animation_data);
        self.marker_names = names;
        self.marker_data = data;

        let animation_data_cstr =
            CString::new(animation_data).map_err(|_| DotLottiePlayerError::Unknown)?;

        self.dotlottie_manager = Some(manager);

        let result = self.load_animation_common(
            |renderer, w, h| renderer.load_data(&animation_data_cstr, w, h),
            width,
            height,
        );

        if result.is_ok() {
            self.animation_id = active_animation_id;
        }

        if result.is_ok() {
            self.event_queue.push(DotLottieEvent::Load);

            if self.autoplay {
                let _ = self.play();
            }
        } else {
            self.event_queue.push(DotLottieEvent::LoadError);
        }

        Ok(())
    }

    #[cfg(feature = "dotlottie")]
    pub fn load_animation(
        &mut self,
        animation_id: &CStr,
        width: u32,
        height: u32,
    ) -> Result<(), DotLottiePlayerError> {
        let anim_id_str = animation_id
            .to_str()
            .map_err(|_| DotLottiePlayerError::InvalidParameter)?;

        if let Some(manager) = &mut self.dotlottie_manager {
            let lookup_id = if anim_id_str.is_empty() {
                manager.active_animation_id()
            } else {
                anim_id_str.to_string()
            };
            let animation_data = manager.get_animation(&lookup_id);

            let result = match animation_data {
                Ok(animation_data) => {
                    let (names, data) = extract_markers(&animation_data);
                    self.marker_names = names;
                    self.marker_data = data;

                    let animation_data_cstr =
                        CString::new(animation_data).expect("Failed to create CString");
                    self.load_animation_common(
                        |renderer, w, h| renderer.load_data(&animation_data_cstr, w, h),
                        width,
                        height,
                    )
                }
                Err(_error) => Err(DotLottiePlayerError::Unknown),
            };

            if result.is_ok() {
                self.animation_id = Some(animation_id.to_owned());

                self.event_queue.push(DotLottieEvent::Load);
                if self.autoplay {
                    let _ = self.play();
                }
            } else {
                self.event_queue.push(DotLottieEvent::LoadError);
            }

            result
        } else {
            Err(DotLottiePlayerError::Unknown)
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), DotLottiePlayerError> {
        self.renderer.resize(width, height)?;
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        if !self.is_loaded() {
            return false;
        }

        match self.mode {
            Mode::Forward => self.current_frame() >= self.end_frame(),
            Mode::Reverse => self.current_frame() <= self.start_frame(),
            Mode::Bounce => {
                // Enables firing loop_complete if loop_count is enabled.
                // Avoid firing at initial start frame before any loop has completed.
                if self.loop_animation && self.loop_count > 0 {
                    self.current_loop_count() > 0 && self.current_frame() <= self.start_frame()
                } else {
                    // Enables firing complete if loop_animation = false
                    self.current_frame() <= self.start_frame()
                        && self.direction == Direction::Reverse
                }
            }
            Mode::ReverseBounce => {
                // Enables firing loop_complete if loop_count is enabled.
                // Avoid firing at initial end frame before any loop has completed.
                if self.loop_animation && self.loop_count > 0 {
                    self.current_loop_count() > 0 && self.current_frame() >= self.end_frame()
                } else {
                    // Enables firing complete if loop_animation = false
                    self.current_frame() >= self.end_frame() && self.direction == Direction::Forward
                }
            }
        }
    }

    #[cfg(feature = "theming")]
    pub fn set_theme(&mut self, theme_id: &CStr) -> Result<(), DotLottiePlayerError> {
        if self.theme_id.as_deref() == Some(theme_id) {
            return Ok(());
        }

        if theme_id.is_empty() {
            self.theme_id = None;
            self.renderer
                .clear_slots()                .map_err(|_| DotLottiePlayerError::Unknown)?;
            return Ok(());
        }

        if self.dotlottie_manager.is_none() {
            return Err(DotLottiePlayerError::InsufficientCondition);
        }

        let theme_exists = self
            .manifest()
            .and_then(|manifest| manifest.themes.as_ref())
            .is_some_and(|themes| {
                themes
                    .iter()
                    .any(|theme| theme.id.as_bytes() == theme_id.to_bytes())
            });

        if !theme_exists {
            return Err(DotLottiePlayerError::InvalidParameter);
        }

        let can_set_theme = self.manifest().is_some_and(|manifest| {
            manifest.animations.iter().any(|animation| {
                match &animation.themes {
                    None => true, // Animation supports all themes
                    Some(themes) => themes.iter().any(|id| id.as_bytes() == theme_id.to_bytes()),
                }
            })
        });

        if !can_set_theme {
            return Err(DotLottiePlayerError::InsufficientCondition);
        }

        let Ok(theme_id_str) = theme_id.to_str() else {
            return Err(DotLottiePlayerError::InvalidParameter);
        };

        let result = self
            .dotlottie_manager
            .as_mut()
            .and_then(|manager| manager.get_theme(theme_id_str).ok())
            .map(|theme| {
                let anim_id_str = self
                    .animation_id
                    .as_deref()
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");

                let slots = theme.to_slot_types(anim_id_str);
                self.apply_slot_types(slots)
            })
            .unwrap_or(Err(DotLottiePlayerError::Unknown));

        if result.is_ok() {
            self.theme_id = Some(theme_id.to_owned());
        }

        result
    }

    #[cfg(feature = "theming")]
    pub fn reset_theme(&mut self) -> Result<(), DotLottiePlayerError> {
        self.theme_id = None;
        self.renderer.clear_slots()?;
        Ok(())
    }

    #[cfg(feature = "theming")]
    pub fn set_theme_data(&mut self, theme_data: &CStr) -> Result<(), DotLottiePlayerError> {
        let theme_data_str = theme_data
            .to_str()
            .map_err(|_| DotLottiePlayerError::InvalidParameter)?;

        let theme = theme_data_str
            .parse::<crate::theme::Theme>()
            .map_err(|_| DotLottiePlayerError::InvalidParameter)?;

        let anim_id_str = self
            .animation_id
            .as_deref()
            .and_then(|c| c.to_str().ok())
            .unwrap_or("");

        let slots = theme.to_slot_types(anim_id_str);

        self.apply_slot_types(slots)
    }

    #[cfg(feature = "theming")]
    fn apply_slot_types(
        &mut self,
        slots: std::collections::BTreeMap<String, crate::lottie_renderer::SlotType>,
    ) -> Result<(), DotLottiePlayerError> {
        use crate::lottie_renderer::SlotType;

        for (slot_id, slot_type) in slots {
            match slot_type {
                SlotType::Color(slot) => self.renderer.set_color_slot(&slot_id, slot)?,
                SlotType::Gradient(slot) => self.renderer.set_gradient_slot(&slot_id, slot)?,
                SlotType::Image(slot) => self.renderer.set_image_slot(&slot_id, slot)?,
                SlotType::Text(slot) => self.renderer.set_text_slot(&slot_id, slot)?,
                SlotType::Scalar(slot) => self.renderer.set_scalar_slot(&slot_id, slot)?,
                SlotType::Vector(slot) => self.renderer.set_vector_slot(&slot_id, slot)?,
                SlotType::Position(slot) => self.renderer.set_position_slot(&slot_id, slot)?,
            };
        }

        Ok(())
    }

    pub fn set_color_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::ColorSlot,
    ) -> Result<(), DotLottiePlayerError> {
        self.renderer.set_color_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn set_gradient_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::GradientSlot,
    ) -> Result<(), DotLottiePlayerError> {
        self.renderer.set_gradient_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn set_image_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::ImageSlot,
    ) -> Result<(), DotLottiePlayerError> {
        self.renderer.set_image_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn set_text_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::TextSlot,
    ) -> Result<(), DotLottiePlayerError> {
        self.renderer.set_text_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn set_scalar_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::ScalarSlot,
    ) -> Result<(), DotLottiePlayerError> {
        self.renderer.set_scalar_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn set_vector_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::VectorSlot,
    ) -> Result<(), DotLottiePlayerError> {
        self.renderer.set_vector_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn set_position_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::PositionSlot,
    ) -> Result<(), DotLottiePlayerError> {
        self.renderer.set_position_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn clear_slots(&mut self) -> Result<(), DotLottiePlayerError> {
        self.renderer.clear_slots()?;
        Ok(())
    }

    pub fn clear_slot(&mut self, slot_id: &str) -> Result<(), DotLottiePlayerError> {
        self.renderer.clear_slot(slot_id)?;
        Ok(())
    }

    pub fn set_slots(
        &mut self,
        slots: std::collections::BTreeMap<String, crate::lottie_renderer::SlotType>,
    ) -> Result<(), DotLottiePlayerError> {
        self.renderer.set_slots(slots)?;
        Ok(())
    }

    pub fn set_slots_str(&mut self, slots_json: &str) -> Result<(), DotLottiePlayerError> {
        use crate::lottie_renderer::slots::slots_from_json_string;

        if slots_json.is_empty() {
            return self.clear_slots();
        }

        match slots_from_json_string(slots_json) {
            Ok(slots) => {
                for (slot_id, slot_type) in slots {
                    use crate::lottie_renderer::SlotType;

                    match slot_type {
                        SlotType::Color(slot) => self.renderer.set_color_slot(&slot_id, slot)?,
                        SlotType::Gradient(slot) => {
                            self.renderer.set_gradient_slot(&slot_id, slot)?
                        }
                        SlotType::Image(slot) => self.renderer.set_image_slot(&slot_id, slot)?,
                        SlotType::Text(slot) => self.renderer.set_text_slot(&slot_id, slot)?,
                        SlotType::Scalar(slot) => self.renderer.set_scalar_slot(&slot_id, slot)?,
                        SlotType::Vector(slot) => self.renderer.set_vector_slot(&slot_id, slot)?,
                        SlotType::Position(slot) => {
                            self.renderer.set_position_slot(&slot_id, slot)?
                        }
                    };
                }
                Ok(())
            }
            Err(_) => Err(DotLottiePlayerError::InvalidParameter),
        }
    }

    pub fn get_slot_ids(&self) -> Vec<String> {
        self.renderer.get_slot_ids()
    }

    pub fn get_slot_type(&self, slot_id: &str) -> String {
        self.renderer.get_slot_type(slot_id)
    }

    pub fn get_slot_str(&self, slot_id: &str) -> String {
        self.renderer.get_slot_str(slot_id)
    }

    pub fn get_slots_str(&self) -> String {
        self.renderer.get_slots_str()
    }

    pub fn set_slot_str(&mut self, slot_id: &str, json: &str) -> Result<(), DotLottiePlayerError> {
        self.renderer.set_slot_str(slot_id, json)?;
        Ok(())
    }

    pub fn reset_slot(&mut self, slot_id: &str) -> Result<(), DotLottiePlayerError> {
        self.renderer.reset_slot(slot_id)?;
        Ok(())
    }

    pub fn reset_slots(&mut self) -> bool {
        self.renderer.reset_slots()
    }

    pub fn set_quality(&mut self, quality: u8) -> Result<(), DotLottiePlayerError> {
        self.renderer.set_quality(quality)?;
        Ok(())
    }

    #[cfg(feature = "dotlottie")]
    pub fn animation_id(&self) -> Option<&CStr> {
        Some(self.animation_id.as_ref()?)
    }

    #[cfg(feature = "theming")]
    pub fn theme_id(&self) -> Option<&CStr> {
        self.theme_id.as_deref()
    }

    #[cfg(feature = "state-machines")]
    pub fn state_machine_id(&self) -> Option<&CStr> {
        self.state_machine_id.as_deref()
    }

    pub fn tween(
        &mut self,
        to: f32,
        duration: Option<f32>,
        easing: Option<[f32; 4]>,
    ) -> Result<(), DotLottiePlayerError> {
        self.renderer.tween(to, duration, easing)?;
        Ok(())
    }

    pub fn tween_stop(&mut self) -> Result<(), DotLottiePlayerError> {
        self.renderer.tween_stop()?;
        Ok(())
    }

    pub fn tween_to_marker(
        &mut self,
        marker: &CStr,
        duration: Option<f32>,
        easing: Option<[f32; 4]>,
    ) -> Result<(), DotLottiePlayerError> {
        if let Some((idx, time, _)) = self.find_marker(marker) {
            self.tween(time, duration, easing)?;

            self.marker = Some(idx);

            self.invalidate_frame_cache();

            Ok(())
        } else {
            Err(DotLottiePlayerError::InvalidParameter)
        }
    }

    pub fn is_tweening(&self) -> bool {
        self.renderer.is_tweening()
    }

    pub fn tween_update(&mut self, progress: Option<f32>) -> Result<bool, DotLottiePlayerError> {
        match self.renderer.tween_update(progress) {
            Ok(still_tweening) => {
                if !still_tweening {
                    // Tween completed — reset start_time so tick() calculates the next frame correctly
                    self.start_time = Instant::now();
                }
                Ok(still_tweening)
            }
            Err(e) => {
                self.start_time = Instant::now();
                Err(e.into())
            }
        }
    }

    pub fn get_transform(&self) -> Vec<f32> {
        self.renderer
            .get_transform()
            .unwrap_or([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
            .to_vec()
    }

    pub fn set_transform(&mut self, transform: Vec<f32>) -> Result<(), DotLottiePlayerError> {
        if transform.len() != 9 {
            return Err(DotLottiePlayerError::InvalidParameter);
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
        self.renderer.set_transform(&transform_array)?;
        Ok(())
    }

    /// Poll for the next event from the event queue
    ///
    /// Returns Some(event) if an event is available, None if the queue is empty.
    pub fn poll_event(&mut self) -> Option<DotLottieEvent> {
        self.event_queue.poll()
    }

    pub fn tick(&mut self) -> Result<(), DotLottiePlayerError> {
        if self.is_tweening() {
            match self.tween_update(None) {
                Ok(_) => return self.render(),
                Err(e) => {
                    // Clear tween state to prevent infinite error loops
                    // (e.g., manual-progress tween where tick provides no progress)
                    let _ = self.tween_stop();
                    return Err(e);
                }
            }
        } else {
            let next_frame = self.request_frame();

            let _ = self.set_frame(next_frame);

            self.render()
        }
    }

    #[cfg(feature = "state-machines")]
    pub fn state_machine_load<'a>(
        &'a mut self,
        state_machine_id: &CStr,
    ) -> Result<StateMachineEngine<'a>, StateMachineEngineError> {
        let machine = match self.get_state_machine(state_machine_id) {
            Some(m) => m, // String is owned, no borrow issue
            None => return Err(StateMachineEngineError::CreationError),
        };

        self.state_machine_id = Some(state_machine_id.to_owned());

        StateMachineEngine::new(&machine, self, None)
    }

    #[cfg(feature = "state-machines")]
    pub fn state_machine_load_data<'a>(
        &'a mut self,
        state_machine: &str,
    ) -> Result<StateMachineEngine<'a>, StateMachineEngineError> {
        StateMachineEngine::new(state_machine, self, None)
    }
}
