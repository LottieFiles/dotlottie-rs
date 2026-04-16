use serde::Deserialize;
use std::ffi::{CStr, CString};
use std::{fs, mem};

#[cfg(feature = "audio")]
use crate::audio::AudioManager;
use crate::poll_events::{DotLottieEvent, EventQueue};
use crate::DotLottiePlayerError;
use crate::{
    layout::Layout,
    lottie_renderer::{LottieRenderer, LottieRendererError},
    tween::{TweenState, TweenStatus},
    Marker,
};
use crate::{ColorSpace, Renderer, Rgba};
#[cfg(feature = "dotlottie")]
use crate::{DotLottieManager, Manifest};
#[cfg(feature = "state-machines")]
use crate::{StateMachineEngine, StateMachineEngineError};

pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
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
    elapsed_frames: f32,
    current_loop_count: u32,
    #[cfg(feature = "dotlottie")]
    dotlottie_manager: Option<DotLottieManager>,
    #[cfg(feature = "audio")]
    audio_manager: Option<AudioManager>,
    direction: Direction,
    active_marker: Option<CString>,
    event_queue: EventQueue<DotLottieEvent>,
    completion_event: CompletionEvent,
    // Playback config properties
    mode: Mode,
    loop_animation: bool,
    loop_count: u32,
    speed: f32,
    use_frame_interpolation: bool,
    autoplay: bool,
    layout: Layout,
    tween_state: Option<TweenState>,
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
            elapsed_frames: 0.0,
            current_loop_count: 0,
            mode: Mode::Forward,
            loop_animation: false,
            loop_count: 0,
            speed: 1.0,
            use_frame_interpolation: true,
            autoplay: false,
            layout: Layout::default(),
            tween_state: None,
            #[cfg(feature = "theming")]
            theme_id: None,
            #[cfg(feature = "dotlottie")]
            animation_id: None,
            #[cfg(feature = "dotlottie")]
            dotlottie_manager: None,
            #[cfg(feature = "audio")]
            audio_manager: None,
            direction: Direction::Forward,
            active_marker: None,
            #[cfg(feature = "state-machines")]
            state_machine_id: None,
            event_queue: EventQueue::new(),
            completion_event: CompletionEvent::None,
        }
    }

    pub fn markers(&self) -> &[Marker] {
        self.renderer.markers()
    }

    /// Set the global audio volume multiplier (clamped to [0.0, 1.0]).
    /// Applied on top of per-layer volume; takes effect immediately.
    #[cfg(feature = "audio")]
    pub fn set_audio_volume(&mut self, volume: f32) {
        if let Some(am) = &mut self.audio_manager {
            am.set_volume(volume);
        }
    }

    #[cfg(feature = "audio")]
    pub fn audio_volume(&self) -> f32 {
        self.audio_manager.as_ref().map_or(1.0, |am| am.volume())
    }

    pub fn pop_completion_event(&mut self) -> CompletionEvent {
        mem::replace(&mut self.completion_event, CompletionEvent::None)
    }

    #[inline]
    pub(crate) fn start_frame(&self) -> f32 {
        self.renderer.segment().map_or(0.0, |seg| seg.start)
    }

    #[inline]
    pub(crate) fn end_frame(&self) -> f32 {
        self.renderer.segment().map_or(0.0, |seg| seg.end)
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

    #[inline]
    pub fn is_playing(&self) -> bool {
        matches!(self.playback_state, PlaybackState::Playing)
    }

    #[inline]
    pub fn is_paused(&self) -> bool {
        matches!(self.playback_state, PlaybackState::Paused)
    }

    #[inline]
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
            self.elapsed_frames = 0.0;
            match self.mode {
                Mode::Forward | Mode::Bounce => {
                    let _ = self.apply_frame(self.start_frame());
                    self.direction = Direction::Forward;
                }
                Mode::Reverse | Mode::ReverseBounce => {
                    let _ = self.apply_frame(self.end_frame());
                    self.direction = Direction::Reverse;
                }
            }
        } else {
            let frame = self.current_frame();
            self.elapsed_frames = match self.direction {
                Direction::Forward => frame - self.start_frame(),
                Direction::Reverse => self.end_frame() - frame,
            };
        }

        self.playback_state = PlaybackState::Playing;

        #[cfg(feature = "audio")]
        if let Some(am) = &mut self.audio_manager {
            am.play();
        }

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

        #[cfg(feature = "audio")]
        if let Some(am) = &mut self.audio_manager {
            am.pause();
        }

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
                let _ = self.apply_frame(start_frame);
            }
            Mode::Reverse | Mode::ReverseBounce => {
                let _ = self.apply_frame(end_frame);
            }
        }

        self.elapsed_frames = 0.0;

        #[cfg(feature = "audio")]
        if let Some(am) = &mut self.audio_manager {
            am.stop();
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

    fn next_frame(&mut self) -> f32 {
        if !self.is_loaded || !self.is_playing() {
            return self.current_frame();
        }

        let start_frame = self.start_frame();
        let end_frame = self.end_frame();

        // elapsed_frames is already in frame-space (scaled by fps in advance_frames)
        let mut next_frame = match self.direction {
            Direction::Forward => start_frame + self.elapsed_frames,
            Direction::Reverse => end_frame - self.elapsed_frames,
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

    #[inline]
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

    #[inline]
    fn handle_forward_mode(&mut self, next_frame: f32, end_frame: f32) -> f32 {
        if next_frame >= end_frame {
            if self.should_increment_loop() {
                self.current_loop_count += 1;
                self.elapsed_frames = 0.0;
            }

            end_frame
        } else {
            next_frame
        }
    }

    #[inline]
    fn handle_reverse_mode(&mut self, next_frame: f32, start_frame: f32) -> f32 {
        if next_frame <= start_frame {
            if self.should_increment_loop() {
                self.current_loop_count += 1;
                self.elapsed_frames = 0.0;
            }

            start_frame
        } else {
            next_frame
        }
    }

    #[inline]
    fn handle_bounce_mode(&mut self, next_frame: f32, start_frame: f32, end_frame: f32) -> f32 {
        match self.direction {
            Direction::Forward => {
                if next_frame >= end_frame {
                    self.direction = Direction::Reverse;
                    self.elapsed_frames = 0.0;

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
                        self.elapsed_frames = 0.0;
                    }

                    start_frame
                } else {
                    next_frame
                }
            }
        }
    }

    #[inline]
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
                    self.elapsed_frames = 0.0;
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
                        self.elapsed_frames = 0.0;
                    }

                    end_frame
                } else {
                    next_frame
                }
            }
        }
    }

    #[inline]
    fn advance_frames(&mut self, dt: f32) {
        if self.is_playing() {
            let duration = self.duration();
            if duration > 0.0 {
                let fps = self.total_frames() / duration;
                self.elapsed_frames += dt * self.speed * fps;
            }
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
    /// Internal: set the renderer frame, push events, sync audio.
    /// Does NOT update `elapsed_frames`.
    fn apply_frame(&mut self, no: f32) -> Result<(), DotLottiePlayerError> {
        if no < self.start_frame() || no > self.end_frame() {
            return Err(DotLottiePlayerError::InvalidParameter);
        }

        self.renderer.set_frame(no)?;
        self.event_queue
            .push(DotLottieEvent::Frame { frame_no: no });

        #[cfg(feature = "audio")]
        if self.is_playing() {
            if let Some(am) = &mut self.audio_manager {
                am.update(no);
            }
        }

        Ok(())
    }

    /// Set the frame number and sync playback position.
    ///
    /// Playback will continue from this frame on the next `tick()`.
    pub fn set_frame(&mut self, no: f32) -> Result<(), DotLottiePlayerError> {
        self.apply_frame(no)?;
        self.elapsed_frames = match self.direction {
            Direction::Forward => no - self.start_frame(),
            Direction::Reverse => self.end_frame() - no,
        };
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

        let frame_no = self.current_frame();

        self.event_queue.push(DotLottieEvent::Render { frame_no });

        // Completion logic only applies during active playback — not when the
        // caller renders manually (e.g. scrubbing while paused/stopped).
        if self.is_playing() && self.is_complete() {
            if self.loop_animation {
                let count_complete =
                    self.loop_count > 0 && self.current_loop_count() >= self.loop_count;

                if count_complete {
                    // Put the animation in a stop state, otherwise we can keep looping if we call tick()
                    // Do it before emiting complete, otherwise it will pause the animation at the wrong stages in state machines
                    let _ = self.stop();
                }

                // Reset audio state so that audio layers re-trigger on the next loop.
                #[cfg(feature = "audio")]
                if let Some(am) = &mut self.audio_manager {
                    am.stop();
                }

                self.emit_on_loop();

                if count_complete {
                    self.emit_on_complete();
                    self.reset_current_loop_count();
                }
            } else {
                self.playback_state = PlaybackState::Stopped;
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
        if let Some(name) = marker_name {
            if self.active_marker.as_deref() == Some(name) {
                return;
            }

            let seg = self
                .renderer
                .markers()
                .iter()
                .find(|m| m.name.as_c_str() == name)
                .map(|m| m.segment);

            if let Some(seg) = seg {
                let _ = self.renderer.set_segment(Some(seg));
                self.active_marker = Some(name.to_owned());

                let frame = self.current_frame();
                if frame < seg.start || frame > seg.end {
                    self.elapsed_frames = 0.0;
                    let _ = self.apply_frame(seg.start);
                }
            }
        } else {
            let _ = self.renderer.set_segment(None);
            self.active_marker = None;
        }
    }

    pub fn active_marker(&self) -> Option<&CStr> {
        self.active_marker.as_deref()
    }

    pub fn set_layout(&mut self, layout: Layout) -> Result<(), LottieRendererError> {
        self.renderer.set_layout(&layout)?;

        self.layout = layout;

        Ok(())
    }

    pub fn layout(&self) -> Layout {
        self.layout
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
            let frame = self.current_frame();
            self.direction = self.direction.flip();
            self.elapsed_frames = match self.direction {
                Direction::Forward => frame - self.start_frame(),
                Direction::Reverse => self.end_frame() - frame,
            };
        }
    }

    pub fn set_background(&mut self, color: Rgba) -> Result<(), DotLottiePlayerError> {
        self.renderer
            .set_background(color)
            .map_err(|_| DotLottiePlayerError::Unknown)
    }

    pub fn background(&self) -> Rgba {
        self.renderer.background()
    }

    pub fn set_speed(&mut self, speed: f32) {
        if self.speed != speed && speed > 0.0 {
            self.speed = speed;
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

    pub fn set_segment(
        &mut self,
        segment: Option<crate::Segment>,
    ) -> Result<(), DotLottiePlayerError> {
        self.renderer
            .set_segment(segment)
            .map_err(|_| DotLottiePlayerError::InvalidParameter)?;
        self.active_marker = None;
        Ok(())
    }

    pub fn segment(&self) -> Result<crate::Segment, DotLottiePlayerError> {
        self.renderer
            .segment()
            .map_err(|_| DotLottiePlayerError::Unknown)
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
    /// `display` and `surface` may hold null pointers on platforms that do not require them
    /// (e.g., macOS CGL only needs `context`). On EGL-based platforms (Android, Linux) all
    /// three handles are typically required.
    ///
    /// All handles must remain valid while the player is using them and the GL context must be
    /// current on the calling thread when rendering.
    pub fn set_gl_target<
        D: crate::lottie_renderer::GlDisplay,
        S: crate::lottie_renderer::GlSurface,
        C: crate::lottie_renderer::GlContext,
    >(
        &mut self,
        display: &D,
        surface: &S,
        context: &C,
        id: i32,
        width: u32,
        height: u32,
    ) -> Result<(), DotLottiePlayerError> {
        self.renderer
            .set_gl_target(display, surface, context, id, width, height)?;
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
        self.renderer
            .set_wg_target(device, instance, target, width, height, target_type)?;
        Ok(())
    }

    fn load_animation_common<F>(&mut self, loader: F) -> Result<(), DotLottiePlayerError>
    where
        F: FnOnce(&mut dyn LottieRenderer) -> Result<(), LottieRendererError>,
    {
        self.clear();
        self.playback_state = PlaybackState::Stopped;
        self.elapsed_frames = 0.0;
        self.current_loop_count = 0;

        let loaded = loader(&mut *self.renderer).is_ok();

        if self.renderer.set_layout(&self.layout).is_err() {
            return Err(DotLottiePlayerError::Unknown);
        }

        self.is_loaded = loaded;

        let start_frame = self.start_frame();
        let end_frame = self.end_frame();

        match self.mode {
            Mode::Forward | Mode::Bounce => {
                let _ = self.apply_frame(start_frame);
                self.direction = Direction::Forward;
            }
            Mode::Reverse | Mode::ReverseBounce => {
                let _ = self.apply_frame(end_frame);
                self.direction = Direction::Reverse;
            }
        }

        let _ = self.renderer.render();

        if loaded {
            Ok(())
        } else {
            Err(DotLottiePlayerError::Unknown)
        }
    }

    pub fn load_animation_data(
        &mut self,
        animation_data: &CStr,
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

        let result = self.load_animation_common(|renderer| renderer.load_data(animation_data));

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

    pub fn load_animation_path(&mut self, file_path: &CStr) -> Result<(), DotLottiePlayerError> {
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

            self.load_animation_data(&c_data)
        })();

        result.inspect_err(|_| {
            self.event_queue.push(DotLottieEvent::LoadError);
        })
    }

    #[cfg(feature = "dotlottie")]
    pub fn load_dotlottie_data(&mut self, file_data: &[u8]) -> Result<(), DotLottiePlayerError> {
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

        let animation_data_cstr =
            CString::new(animation_data).map_err(|_| DotLottiePlayerError::Unknown)?;

        self.dotlottie_manager = Some(manager);

        #[cfg(feature = "audio")]
        {
            self.audio_manager = self
                .dotlottie_manager
                .as_ref()
                .and_then(|dm| dm.get_audio_assets())
                .and_then(|(assets, layers)| AudioManager::with_assets(assets, layers));
        }

        let result =
            self.load_animation_common(|renderer| renderer.load_data(&animation_data_cstr));

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
    pub fn load_animation(&mut self, animation_id: &CStr) -> Result<(), DotLottiePlayerError> {
        let anim_id_str = animation_id
            .to_str()
            .map_err(|_| DotLottiePlayerError::InvalidParameter)?;

        if let Some(manager) = &mut self.dotlottie_manager {
            #[cfg(feature = "theming")]
            let saved_theme_id = self.theme_id.clone();

            let lookup_id = if anim_id_str.is_empty() {
                manager.active_animation_id()
            } else {
                anim_id_str.to_string()
            };
            let animation_data = manager.get_animation(&lookup_id);

            let result = match animation_data {
                Ok(animation_data) => {
                    let animation_data_cstr =
                        CString::new(animation_data).expect("Failed to create CString");
                    self.load_animation_common(|renderer| renderer.load_data(&animation_data_cstr))
                }
                Err(_error) => Err(DotLottiePlayerError::Unknown),
            };

            if result.is_ok() {
                self.animation_id = Some(animation_id.to_owned());

                #[cfg(feature = "audio")]
                {
                    self.audio_manager = self
                        .dotlottie_manager
                        .as_ref()
                        .and_then(|dm| dm.get_audio_assets())
                        .and_then(|(assets, layers)| AudioManager::with_assets(assets, layers));
                }

                #[cfg(feature = "theming")]
                if let Some(ref theme_id_cstr) = saved_theme_id {
                    self.theme_id = None;
                    let _ = self.set_theme(theme_id_cstr);
                }

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
                .clear_slots()
                .map_err(|_| DotLottiePlayerError::Unknown)?;
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
        duration: f32,
        easing: [f32; 4],
    ) -> Result<(), DotLottiePlayerError> {
        if self.is_tweening() {
            return Err(DotLottiePlayerError::InsufficientCondition);
        }
        let from = self.current_frame();
        self.tween_state = Some(TweenState::new(from, to, duration, easing)?);
        Ok(())
    }

    #[inline]
    pub fn is_tweening(&self) -> bool {
        self.tween_state.is_some()
    }

    pub(crate) fn sync_tween_frame(&mut self, frame: f32) {
        self.renderer.sync_current_frame(frame);
        self.elapsed_frames = match self.direction {
            Direction::Forward => frame - self.start_frame(),
            Direction::Reverse => self.end_frame() - frame,
        };
    }

    pub fn tween_advance(&mut self, dt: f32) -> Result<TweenStatus, DotLottiePlayerError> {
        let tween = self
            .tween_state
            .as_mut()
            .ok_or(DotLottiePlayerError::InsufficientCondition)?;

        let (status, progress) = tween.update(dt);
        let from = tween.from;
        let to = tween.to;

        self.renderer.tween(from, to, progress)?;

        if status == TweenStatus::Completed {
            self.elapsed_frames = match self.direction {
                Direction::Forward => to - self.start_frame(),
                Direction::Reverse => self.end_frame() - to,
            };
            self.tween_state = None;
        }

        Ok(status)
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

    /// Advance the animation by `dt` milliseconds and render if the frame changed.
    ///
    /// Returns `Ok(true)` when a new frame was rendered, `Ok(false)` when the
    /// frame was unchanged and rendering was skipped.
    pub fn tick(&mut self, dt: f32) -> Result<bool, DotLottiePlayerError> {
        let dt = dt.max(0.0);

        if self.is_tweening() {
            match self.tween_advance(dt) {
                Ok(_) => {
                    self.render()?;
                    Ok(true)
                }
                Err(e) => {
                    self.tween_state = None;
                    Err(e)
                }
            }
        } else {
            self.advance_frames(dt);
            let next_frame = self.next_frame();

            if next_frame == self.current_frame() && !self.renderer.updated() {
                return Ok(false);
            }

            if next_frame != self.current_frame() {
                let _ = self.apply_frame(next_frame);
            }
            self.render()?;
            Ok(true)
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
