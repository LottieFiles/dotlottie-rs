use serde::Deserialize;
use std::ffi::{CStr, CString};
use std::{fs, mem};

#[cfg(feature = "audio")]
use crate::audio::AudioManager;
use crate::player_state::{Resume, State, TweenOutcome};
use crate::poll_events::{EventQueue, PlayerEvent};
use crate::PlayerError;
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
#[cfg(feature = "audio")]
use rustc_hash::FxHashMap;
#[cfg(feature = "audio")]
use std::cell::RefCell;
#[cfg(feature = "audio")]
use std::rc::Rc;
#[cfg(feature = "audio")]
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Status {
    Idle = 0,
    Playing = 1,
    Paused = 2,
    Stopped = 3,
    Tweening = 4,
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

// This is used to pass the loop complete / complete event to the state machine engine
pub enum CompletionEvent {
    None,
    Completed,
    LoopCompleted,
}

pub struct Player {
    pub(crate) renderer: Box<dyn LottieRenderer>,
    state: State,
    tween_outcome: Option<TweenOutcome>,
    elapsed_frames: f32,
    current_loop_count: u32,
    #[cfg(feature = "dotlottie")]
    dotlottie_manager: Option<DotLottieManager>,
    #[cfg(feature = "audio")]
    audio: Option<Rc<RefCell<AudioManager>>>,
    direction: Direction,
    active_marker: Option<CString>,
    event_queue: EventQueue<PlayerEvent, 16>,
    completion_event: CompletionEvent,
    // Config properties
    mode: Mode,
    loop_animation: bool,
    loop_count: u32,
    speed: f32,
    use_frame_interpolation: bool,
    autoplay: bool,
    layout: Layout,
    #[cfg(feature = "theming")]
    theme_id: Option<CString>,
    #[cfg(feature = "dotlottie")]
    animation_id: Option<CString>,
    #[cfg(feature = "state-machines")]
    state_machine_id: Option<CString>,
}

#[cfg(feature = "tvg")]
impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    #[cfg(feature = "tvg")]
    pub fn new() -> Self {
        Self::with_renderer(crate::TvgRenderer::new(0))
    }

    #[cfg(feature = "tvg")]
    pub fn with_threads(threads: u32) -> Self {
        Self::with_renderer(crate::TvgRenderer::new(threads))
    }

    #[cfg(feature = "tvg")]
    pub fn load_font(name: &str, data: &[u8]) -> Result<(), PlayerError> {
        use crate::lottie_renderer::Renderer;
        crate::TvgRenderer::load_font(name, data).map_err(|_| PlayerError::Unknown)
    }

    #[cfg(feature = "tvg")]
    pub fn unload_font(name: &str) -> Result<(), PlayerError> {
        use crate::lottie_renderer::Renderer;
        crate::TvgRenderer::unload_font(name).map_err(|_| PlayerError::Unknown)
    }

    pub fn with_renderer<R: Renderer>(renderer: R) -> Self {
        Player {
            renderer: <dyn LottieRenderer>::new(renderer),
            state: State::Idle,
            tween_outcome: None,
            elapsed_frames: 0.0,
            current_loop_count: 0,
            mode: Mode::Forward,
            loop_animation: false,
            loop_count: 0,
            speed: 1.0,
            use_frame_interpolation: true,
            autoplay: false,
            layout: Layout::default(),
            #[cfg(feature = "theming")]
            theme_id: None,
            #[cfg(feature = "dotlottie")]
            animation_id: None,
            #[cfg(feature = "dotlottie")]
            dotlottie_manager: None,
            #[cfg(feature = "audio")]
            audio: None,
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
        if let Some(am) = &self.audio {
            am.borrow_mut().set_volume(volume);
        }
    }

    #[cfg(feature = "audio")]
    pub fn audio_volume(&self) -> f32 {
        self.audio.as_ref().map_or(1.0, |am| am.borrow().volume())
    }

    /// Create the audio manager and attach the renderer's audio resolver.
    /// `sources` is empty for raw-JSON animations whose audio is embedded.
    #[cfg(feature = "audio")]
    fn setup_audio(&mut self, sources: FxHashMap<String, Arc<[u8]>>) {
        self.audio = Some(Rc::new(RefCell::new(AudioManager::new(sources))));
        self.attach_audio_resolver();
    }

    #[cfg(feature = "audio")]
    fn attach_audio_resolver(&mut self) {
        let Some(am) = self.audio.as_ref().map(Rc::clone) else {
            return;
        };
        let resolver: crate::lottie_renderer::AudioResolver = Box::new(move |event| {
            if let Ok(mut manager) = am.try_borrow_mut() {
                manager.on_audio(event);
            }
        });
        let _ = self.renderer.set_audio_resolver(Some(resolver));
    }

    #[cfg(all(test, feature = "audio"))]
    pub(crate) fn audio_active_count(&self) -> usize {
        self.audio
            .as_ref()
            .map_or(0, |am| am.borrow().active_layer_count())
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

    #[inline]
    pub fn status(&self) -> Status {
        match self.state {
            State::Idle => Status::Idle,
            State::Stopped => Status::Stopped,
            State::Paused => Status::Paused,
            State::Playing => Status::Playing,
            State::Tweening { .. } => Status::Tweening,
        }
    }

    fn end_tween(&mut self, outcome: TweenOutcome) {
        if let State::Tweening { resume, .. } = self.state {
            self.state = resume.into();
            self.tween_outcome = Some(outcome);
        }
    }

    pub(crate) fn take_tween_outcome(&mut self) -> Option<TweenOutcome> {
        self.tween_outcome.take()
    }

    pub fn play(&mut self) -> Result<(), PlayerError> {
        match &mut self.state {
            State::Idle => return Err(PlayerError::AnimationNotLoaded),
            State::Playing => return Err(PlayerError::InsufficientCondition),
            State::Tweening { resume, .. } => {
                if *resume == Resume::Playing {
                    return Err(PlayerError::InsufficientCondition);
                }
                *resume = Resume::Playing;

                #[cfg(feature = "audio")]
                if let Some(am) = &self.audio {
                    am.borrow_mut().set_playing(true);
                }

                self.event_queue.push(PlayerEvent::Play);
                return Ok(());
            }
            State::Stopped | State::Paused => {}
        }

        if self.is_complete() && matches!(self.state, State::Stopped) {
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

        self.state = State::Playing;

        #[cfg(feature = "audio")]
        if let Some(am) = &self.audio {
            am.borrow_mut().set_playing(true);
        }

        self.event_queue.push(PlayerEvent::Play);

        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), PlayerError> {
        match &mut self.state {
            State::Idle => return Err(PlayerError::AnimationNotLoaded),
            State::Playing => {}
            State::Tweening { resume, .. } if *resume == Resume::Playing => {
                *resume = Resume::Paused;

                #[cfg(feature = "audio")]
                if let Some(am) = &self.audio {
                    am.borrow_mut().set_playing(false);
                }

                self.event_queue.push(PlayerEvent::Pause);
                return Ok(());
            }
            _ => return Err(PlayerError::InsufficientCondition),
        }
        self.state = State::Paused;

        #[cfg(feature = "audio")]
        if let Some(am) = &self.audio {
            am.borrow_mut().set_playing(false);
        }

        self.event_queue.push(PlayerEvent::Pause);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), PlayerError> {
        match self.state {
            State::Idle => return Err(PlayerError::AnimationNotLoaded),
            State::Stopped => return Err(PlayerError::InsufficientCondition),
            State::Tweening { .. } => self.end_tween(TweenOutcome::Cancelled),
            State::Playing | State::Paused => {}
        }

        self.state = State::Stopped;

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
        if let Some(am) = &self.audio {
            am.borrow_mut().stop();
        }

        self.event_queue.push(PlayerEvent::Stop);

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
        if !matches!(self.state, State::Playing) {
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
        if matches!(self.state, State::Playing) {
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
    fn apply_frame(&mut self, no: f32) -> Result<(), PlayerError> {
        if no < self.start_frame() || no > self.end_frame() {
            return Err(PlayerError::InvalidParameter);
        }

        self.renderer.set_frame(no)?;
        self.event_queue.push(PlayerEvent::Frame { frame_no: no });

        Ok(())
    }

    /// Set the frame number and sync playback position.
    ///
    /// Playback will continue from this frame on the next `tick()`.
    pub fn set_frame(&mut self, no: f32) -> Result<(), PlayerError> {
        self.apply_frame(no)?;
        self.elapsed_frames = match self.direction {
            Direction::Forward => no - self.start_frame(),
            Direction::Reverse => self.end_frame() - no,
        };
        Ok(())
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> Result<(), PlayerError> {
        self.renderer.set_viewport(x, y, w, h)?;
        Ok(())
    }

    fn emit_on_complete(&mut self) {
        self.completion_event = CompletionEvent::Completed;
        self.event_queue.push(PlayerEvent::Complete);
    }

    pub fn emit_on_loop(&mut self) {
        self.completion_event = CompletionEvent::LoopCompleted;
        self.event_queue.push(PlayerEvent::Loop {
            loop_count: self.current_loop_count(),
        });
    }

    pub fn render(&mut self) -> Result<(), PlayerError> {
        self.renderer.render()?;

        let frame_no = self.current_frame();

        self.event_queue.push(PlayerEvent::Render { frame_no });

        // Completion logic only applies during active playback — not when the
        // caller renders manually (e.g. scrubbing while paused/stopped).
        if matches!(self.state, State::Playing) && self.is_complete() {
            if self.loop_animation {
                let count_complete =
                    self.loop_count > 0 && self.current_loop_count() >= self.loop_count;

                if count_complete {
                    // Put the animation in a stop state, otherwise we can keep looping if we call tick()
                    // Do it before emiting complete, otherwise it will pause the animation at the wrong stages in state machines
                    let _ = self.stop();
                }

                // Replay audio on loop; the resolver won't re-announce layers
                // that stay in range across the wrap.
                #[cfg(feature = "audio")]
                if !count_complete {
                    if let Some(am) = &self.audio {
                        am.borrow_mut().restart();
                    }
                }

                self.emit_on_loop();

                if count_complete {
                    self.emit_on_complete();
                    self.reset_current_loop_count();
                }
            } else {
                self.state = State::Stopped;
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

    pub fn set_background(&mut self, color: Rgba) -> Result<(), PlayerError> {
        self.renderer
            .set_background(color)
            .map_err(|_| PlayerError::Unknown)
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

    pub fn set_segment(&mut self, segment: Option<crate::Segment>) -> Result<(), PlayerError> {
        self.renderer
            .set_segment(segment)
            .map_err(|_| PlayerError::InvalidParameter)?;
        self.active_marker = None;
        Ok(())
    }

    pub fn segment(&self) -> Result<crate::Segment, PlayerError> {
        self.renderer.segment().map_err(|_| PlayerError::Unknown)
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
    ) -> Result<(), PlayerError> {
        let required_size = (width * height) as usize;
        if buffer.len() < required_size {
            return Err(PlayerError::InvalidParameter);
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
    ) -> Result<(), PlayerError> {
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
    ) -> Result<(), PlayerError> {
        self.renderer
            .set_wg_target(device, instance, target, width, height, target_type)?;
        Ok(())
    }

    fn load_animation_common<F>(&mut self, loader: F) -> Result<(), PlayerError>
    where
        F: FnOnce(&mut dyn LottieRenderer) -> Result<(), LottieRendererError>,
    {
        self.end_tween(TweenOutcome::Cancelled);
        self.state = State::Idle;
        self.elapsed_frames = 0.0;
        self.current_loop_count = 0;

        let loaded = loader(&mut *self.renderer).is_ok();

        if self.renderer.set_layout(&self.layout).is_err() || !loaded {
            return Err(PlayerError::Unknown);
        }

        self.state = State::Stopped;

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

        Ok(())
    }

    pub fn load_animation_data(&mut self, animation_data: &CStr) -> Result<(), PlayerError> {
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
            // Embedded audio is delivered via the resolver, so no source map.
            #[cfg(feature = "audio")]
            self.setup_audio(FxHashMap::default());

            self.event_queue.push(PlayerEvent::Load);
            if self.autoplay {
                let _ = self.play();
            }
        } else {
            self.event_queue.push(PlayerEvent::LoadError);
        }

        result
    }

    pub fn load_animation_path(&mut self, file_path: &CStr) -> Result<(), PlayerError> {
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
                .map_err(|_| PlayerError::InvalidParameter)?;
            let data = fs::read_to_string(path_str).map_err(|_| PlayerError::InvalidParameter)?;
            let c_data = CString::new(data).map_err(|_| PlayerError::InvalidParameter)?;

            self.load_animation_data(&c_data)
        })();

        result.inspect_err(|_| {
            self.event_queue.push(PlayerEvent::LoadError);
        })
    }

    #[cfg(feature = "dotlottie")]
    pub fn load_dotlottie_data(&mut self, file_data: &[u8]) -> Result<(), PlayerError> {
        #[cfg(feature = "dotlottie")]
        {
            self.animation_id = None;
        }
        #[cfg(feature = "theming")]
        {
            self.theme_id = None;
        }
        let manager = DotLottieManager::new(file_data).map_err(|_| PlayerError::Unknown)?;

        let (active_animation, active_animation_id) =
            if let Some(anim_id) = self.animation_id.as_deref().and_then(|c| c.to_str().ok()) {
                (manager.get_animation(anim_id), self.animation_id.clone())
            } else {
                (
                    manager.get_active_animation(),
                    CString::new(manager.active_animation_id()).ok(),
                )
            };

        let animation_data = active_animation.map_err(|_| PlayerError::Unknown)?;

        let animation_data_cstr = CString::new(animation_data).map_err(|_| PlayerError::Unknown)?;

        self.dotlottie_manager = Some(manager);

        let result =
            self.load_animation_common(|renderer| renderer.load_data(&animation_data_cstr));

        if result.is_ok() {
            self.animation_id = active_animation_id;

            #[cfg(feature = "audio")]
            {
                let sources = self
                    .dotlottie_manager
                    .as_ref()
                    .map(|dm| dm.get_audio_sources())
                    .unwrap_or_default();
                self.setup_audio(sources);
            }

            self.event_queue.push(PlayerEvent::Load);

            if self.autoplay {
                let _ = self.play();
            }
        } else {
            self.event_queue.push(PlayerEvent::LoadError);
        }

        Ok(())
    }

    #[cfg(feature = "dotlottie")]
    pub fn load_animation(&mut self, animation_id: &CStr) -> Result<(), PlayerError> {
        let anim_id_str = animation_id
            .to_str()
            .map_err(|_| PlayerError::InvalidParameter)?;

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
                Err(_error) => Err(PlayerError::Unknown),
            };

            if result.is_ok() {
                self.animation_id = Some(animation_id.to_owned());

                #[cfg(feature = "audio")]
                {
                    let sources = self
                        .dotlottie_manager
                        .as_ref()
                        .map(|dm| dm.get_audio_sources())
                        .unwrap_or_default();
                    self.setup_audio(sources);
                }

                #[cfg(feature = "theming")]
                if let Some(ref theme_id_cstr) = saved_theme_id {
                    self.theme_id = None;
                    let _ = self.set_theme(theme_id_cstr);
                }

                self.event_queue.push(PlayerEvent::Load);
                if self.autoplay {
                    let _ = self.play();
                }
            } else {
                self.event_queue.push(PlayerEvent::LoadError);
            }

            result
        } else {
            Err(PlayerError::Unknown)
        }
    }

    pub fn is_complete(&self) -> bool {
        if matches!(self.state, State::Idle | State::Tweening { .. }) {
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
    pub fn set_theme(&mut self, theme_id: &CStr) -> Result<(), PlayerError> {
        if self.theme_id.as_deref() == Some(theme_id) {
            return Ok(());
        }

        if theme_id.is_empty() {
            self.theme_id = None;
            self.renderer.reset_slots();
            return Ok(());
        }

        if self.dotlottie_manager.is_none() {
            return Err(PlayerError::InsufficientCondition);
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
            return Err(PlayerError::InvalidParameter);
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
            return Err(PlayerError::InsufficientCondition);
        }

        let Ok(theme_id_str) = theme_id.to_str() else {
            return Err(PlayerError::InvalidParameter);
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
            .unwrap_or(Err(PlayerError::Unknown));

        if result.is_ok() {
            self.theme_id = Some(theme_id.to_owned());
        }

        result
    }

    #[cfg(feature = "theming")]
    pub fn reset_theme(&mut self) -> Result<(), PlayerError> {
        self.theme_id = None;
        self.renderer.reset_slots();
        Ok(())
    }

    #[cfg(feature = "theming")]
    pub fn set_theme_data(&mut self, theme_data: &CStr) -> Result<(), PlayerError> {
        let theme_data_str = theme_data
            .to_str()
            .map_err(|_| PlayerError::InvalidParameter)?;

        let theme = theme_data_str
            .parse::<crate::theme::Theme>()
            .map_err(|_| PlayerError::InvalidParameter)?;

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
    ) -> Result<(), PlayerError> {
        use crate::lottie_renderer::SlotType;

        for (slot_id, slot_type) in slots {
            match slot_type {
                SlotType::Color(slot) => self.renderer.set_color_slot(&slot_id, slot)?,
                SlotType::Gradient(slot) => self.renderer.set_gradient_slot(&slot_id, slot)?,
                SlotType::Image(slot) => {
                    let slot = self.normalize_image_slot(&slot_id, slot)?;
                    self.renderer.set_image_slot(&slot_id, slot)?
                }
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
    ) -> Result<(), PlayerError> {
        self.renderer.set_color_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn set_gradient_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::GradientSlot,
    ) -> Result<(), PlayerError> {
        self.renderer.set_gradient_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn set_image_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::ImageSlot,
    ) -> Result<(), PlayerError> {
        let slot = self.normalize_image_slot(slot_id, slot)?;
        self.renderer.set_image_slot(slot_id, slot)?;
        Ok(())
    }

    /// Inline package images as `data:` URIs and ensure non-zero `w`/`h`, the
    /// only shape ThorVG parses as an image rather than as audio.
    fn normalize_image_slot(
        &self,
        slot_id: &str,
        mut slot: crate::lottie_renderer::ImageSlot,
    ) -> Result<crate::lottie_renderer::ImageSlot, PlayerError> {
        if !slot.is_embedded() && !slot.is_remote() {
            let file_name = slot
                .file_name()
                .map(str::to_owned)
                .ok_or(PlayerError::InvalidParameter)?;

            let data_url = self
                .resolve_package_image(&file_name)
                .ok_or(PlayerError::InvalidParameter)?;

            slot.inline(data_url);
        }

        if !slot.has_dimensions() {
            if let Some(crate::lottie_renderer::SlotType::Image(default)) =
                self.renderer.default_slot(slot_id)
            {
                if let (Some(width), Some(height)) = (default.width, default.height) {
                    slot = slot.with_dimensions(width, height);
                }
            }
        }

        if !slot.has_dimensions() {
            return Err(PlayerError::InvalidParameter);
        }

        Ok(slot)
    }

    fn normalize_image_slots(
        &self,
        slots: &mut std::collections::BTreeMap<String, crate::lottie_renderer::SlotType>,
    ) -> Result<(), PlayerError> {
        for (slot_id, slot_type) in slots.iter_mut() {
            if let crate::lottie_renderer::SlotType::Image(slot) = slot_type {
                *slot = self.normalize_image_slot(slot_id, slot.clone())?;
            }
        }

        Ok(())
    }

    #[cfg(feature = "dotlottie")]
    fn resolve_package_image(&self, file_name: &str) -> Option<String> {
        self.dotlottie_manager
            .as_ref()?
            .get_image_data_url(file_name)
    }

    #[cfg(not(feature = "dotlottie"))]
    fn resolve_package_image(&self, _file_name: &str) -> Option<String> {
        None
    }

    pub fn set_text_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::TextSlot,
    ) -> Result<(), PlayerError> {
        self.renderer.set_text_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn set_scalar_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::ScalarSlot,
    ) -> Result<(), PlayerError> {
        self.renderer.set_scalar_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn set_vector_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::VectorSlot,
    ) -> Result<(), PlayerError> {
        self.renderer.set_vector_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn set_position_slot(
        &mut self,
        slot_id: &str,
        slot: crate::lottie_renderer::PositionSlot,
    ) -> Result<(), PlayerError> {
        self.renderer.set_position_slot(slot_id, slot)?;
        Ok(())
    }

    pub fn clear_slots(&mut self) -> Result<(), PlayerError> {
        self.renderer.clear_slots()?;
        Ok(())
    }

    pub fn clear_slot(&mut self, slot_id: &str) -> Result<(), PlayerError> {
        self.renderer.clear_slot(slot_id)?;
        Ok(())
    }

    pub fn set_slots(
        &mut self,
        mut slots: std::collections::BTreeMap<String, crate::lottie_renderer::SlotType>,
    ) -> Result<(), PlayerError> {
        self.normalize_image_slots(&mut slots)?;
        self.renderer.set_slots(slots)?;
        Ok(())
    }

    pub fn set_slots_str(&mut self, slots_json: &str) -> Result<(), PlayerError> {
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
                        SlotType::Image(slot) => {
                            let slot = self.normalize_image_slot(&slot_id, slot)?;
                            self.renderer.set_image_slot(&slot_id, slot)?
                        }
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
            Err(_) => Err(PlayerError::InvalidParameter),
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

    pub fn set_slot_str(&mut self, slot_id: &str, json: &str) -> Result<(), PlayerError> {
        if self.renderer.get_slot_type(slot_id) == "image" {
            let parsed = crate::lottie_renderer::slots::parse_slot_from_json("image", json)
                .ok_or(PlayerError::InvalidParameter)?;

            if let crate::lottie_renderer::SlotType::Image(slot) = parsed {
                let slot = self.normalize_image_slot(slot_id, slot)?;
                self.renderer.set_image_slot(slot_id, slot)?;

                return Ok(());
            }
        }

        self.renderer.set_slot_str(slot_id, json)?;
        Ok(())
    }

    pub fn reset_slot(&mut self, slot_id: &str) -> Result<(), PlayerError> {
        self.renderer.reset_slot(slot_id)?;
        Ok(())
    }

    pub fn reset_slots(&mut self) -> bool {
        self.renderer.reset_slots()
    }

    pub fn set_quality(&mut self, quality: u8) -> Result<(), PlayerError> {
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

    pub fn tween(&mut self, to: f32, duration: f32, easing: [f32; 4]) -> Result<(), PlayerError> {
        let resume = match self.state {
            State::Idle => return Err(PlayerError::AnimationNotLoaded),
            State::Tweening { .. } => return Err(PlayerError::InsufficientCondition),
            State::Stopped => Resume::Stopped,
            State::Paused => Resume::Paused,
            State::Playing => Resume::Playing,
        };
        let from = self.current_frame();
        let tween = TweenState::new(from, to, duration, easing)?;
        self.tween_outcome = None;
        self.state = State::Tweening { tween, resume };
        Ok(())
    }

    pub(crate) fn sync_tween_frame(&mut self, frame: f32) {
        self.renderer.sync_current_frame(frame);
        self.elapsed_frames = match self.direction {
            Direction::Forward => frame - self.start_frame(),
            Direction::Reverse => self.end_frame() - frame,
        };
    }

    pub fn tween_advance(&mut self, dt: f32) -> Result<TweenStatus, PlayerError> {
        let (status, progress, from, to) = match &mut self.state {
            State::Tweening { tween, .. } => {
                let (status, progress) = tween.update(dt);
                (status, progress, tween.from, tween.to)
            }
            _ => return Err(PlayerError::InsufficientCondition),
        };

        if let Err(e) = self.renderer.tween(from, to, progress) {
            self.end_tween(TweenOutcome::Cancelled);
            return Err(e.into());
        }

        if status == TweenStatus::Completed {
            self.elapsed_frames = match self.direction {
                Direction::Forward => to - self.start_frame(),
                Direction::Reverse => self.end_frame() - to,
            };
            self.end_tween(TweenOutcome::Completed);
        }

        Ok(status)
    }

    pub fn get_transform(&self) -> Vec<f32> {
        self.renderer
            .get_transform()
            .unwrap_or([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
            .to_vec()
    }

    pub fn set_transform(&mut self, transform: Vec<f32>) -> Result<(), PlayerError> {
        if transform.len() != 9 {
            return Err(PlayerError::InvalidParameter);
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
    pub fn poll_event(&mut self) -> Option<PlayerEvent> {
        self.event_queue.poll()
    }

    /// Advance the animation by `dt` milliseconds and render if the frame changed.
    ///
    /// Returns `Ok(true)` when a new frame was rendered, `Ok(false)` when the
    /// frame was unchanged and rendering was skipped.
    pub fn tick(&mut self, dt: f32) -> Result<bool, PlayerError> {
        let dt = dt.max(0.0);

        if matches!(self.state, State::Tweening { .. }) {
            match self.tween_advance(dt) {
                Ok(_) => {
                    self.render()?;
                    Ok(true)
                }
                Err(e) => Err(e),
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

#[cfg(test)]
#[cfg(all(feature = "tvg", feature = "dotlottie", feature = "audio"))]
mod audio_render_tests {
    use crate::{ColorSpace, Player};

    /// A dotLottie with audio loads, renders, and drives the audio manager.
    #[test]
    fn renders_dotlottie_with_audio_and_resolves_layers() {
        let mut player = Player::new();
        let mut buffer = vec![0u32; 512 * 512];
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();

        let data = include_bytes!("../assets/animations/dotlottie/v2/happy_birthday_audio.lottie");
        assert!(
            player.load_dotlottie_data(data).is_ok(),
            "audio dotLottie loads"
        );
        assert!(player.total_frames() > 0.0, "animation has frames");

        player.set_loop(true);
        player.set_autoplay(true);

        let mut rendered_any = false;
        for _ in 0..20 {
            if player.tick(1000.0 / 60.0).unwrap_or(false) {
                rendered_any = true;
            }
        }
        assert!(rendered_any, "at least one frame rendered");
        assert!(
            player.audio_active_count() > 0,
            "audio resolver should mark layers active during playback"
        );
    }

    fn loaded_audio_player(buffer: &mut [u32]) -> Player {
        let mut player = Player::new();
        player
            .set_sw_target(buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let data = include_bytes!("../assets/animations/dotlottie/v2/happy_birthday_audio.lottie");
        assert!(player.load_dotlottie_data(data).is_ok());
        player.set_loop(true);
        player
    }

    fn tick_some(player: &mut Player) {
        for _ in 0..20 {
            let _ = player.tick(1000.0 / 60.0);
        }
    }

    /// The example's X→P flow: after stop, audio must restart on the next play.
    #[test]
    fn audio_restarts_after_stop_then_play() {
        let mut buffer = vec![0u32; 512 * 512];
        let mut player = loaded_audio_player(&mut buffer);

        player.play().unwrap();
        tick_some(&mut player);
        assert!(
            player.audio_active_count() > 0,
            "audio active on first play"
        );

        player.stop().unwrap();

        player.play().unwrap();
        tick_some(&mut player);
        assert!(
            player.audio_active_count() > 0,
            "audio must re-activate after stop -> play"
        );
    }

    /// The example's S→P flow: pause keeps the active set so resume continues.
    #[test]
    fn audio_survives_pause_resume() {
        let mut buffer = vec![0u32; 512 * 512];
        let mut player = loaded_audio_player(&mut buffer);

        player.play().unwrap();
        tick_some(&mut player);
        let active = player.audio_active_count();
        assert!(active > 0, "audio active while playing");

        player.pause().unwrap();
        assert_eq!(
            player.audio_active_count(),
            active,
            "pause retains the active set"
        );

        player.play().unwrap();
        tick_some(&mut player);
        assert!(
            player.audio_active_count() > 0,
            "audio still active after resume"
        );
    }
}
