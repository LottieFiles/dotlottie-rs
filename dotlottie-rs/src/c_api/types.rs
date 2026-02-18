#![allow(clippy::missing_safety_doc)]

#[cfg(feature = "state-machines")]
use bitflags::bitflags;
#[cfg(feature = "state-machines")]
use core::str::FromStr;
#[cfg(feature = "state-machines")]
use std::ffi::c_char;

#[cfg(feature = "state-machines")]
use crate::state_machine_engine::events::Event;

use crate::lottie_renderer::LottieRendererError;
use crate::DotLottiePlayerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum PlaybackStatus {
    Playing = 0,
    Paused = 1,
    Stopped = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum DotLottieResult {
    Success = 0,
    Error = 1,
    InvalidParameter = 2,
    ManifestNotAvailable = 3,
    AnimationNotLoaded = 4,
    InsufficientCondition = 5,
}

impl From<DotLottiePlayerError> for DotLottieResult {
    fn from(err: DotLottiePlayerError) -> Self {
        match err {
            DotLottiePlayerError::Unknown => DotLottieResult::Error,
            DotLottiePlayerError::InvalidParameter => DotLottieResult::InvalidParameter,
            DotLottiePlayerError::ManifestNotAvailable => DotLottieResult::ManifestNotAvailable,
            DotLottiePlayerError::AnimationNotLoaded => DotLottieResult::AnimationNotLoaded,
            DotLottiePlayerError::InsufficientCondition => DotLottieResult::InsufficientCondition,
        }
    }
}

impl From<LottieRendererError> for DotLottieResult {
    fn from(err: LottieRendererError) -> Self {
        match err {
            LottieRendererError::InvalidArgument => DotLottieResult::InvalidParameter,
            LottieRendererError::AnimationNotLoaded => DotLottieResult::AnimationNotLoaded,
            _ => DotLottieResult::Error,
        }
    }
}

impl<E: Into<DotLottieResult>> From<Result<(), E>> for DotLottieResult {
    fn from(result: Result<(), E>) -> Self {
        match result {
            Ok(()) => DotLottieResult::Success,
            Err(e) => e.into(),
        }
    }
}

// This type allows us to work with Interaction Types as bit flags and easily communicate this
// information to the C side
#[cfg(feature = "state-machines")]
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(C)]
    pub(crate) struct InteractionType: u16 {
        const UNSET = 0;

        const POINTER_UP       = 1 << 0;
        const POINTER_DOWN     = 1 << 1;
        const POINTER_ENTER    = 1 << 2;
        const POINTER_EXIT     = 1 << 3;
        const POINTER_MOVE     = 1 << 4;
        const CLICK            = 1 << 5;
        const ON_COMPLETE      = 1 << 6;
        const ON_LOOP_COMPLETE = 1 << 7;
    }
}

#[derive(Debug, Clone)]
#[cfg(feature = "state-machines")]
pub(crate) struct InteractionTypeParseError;

#[cfg(feature = "state-machines")]
impl InteractionType {
    pub fn new(
        interaction_types: &Vec<String>,
    ) -> Result<InteractionType, InteractionTypeParseError> {
        let mut result: InteractionType = InteractionType::UNSET;
        for interaction_type in interaction_types {
            result |= InteractionType::from_str(interaction_type)?;
        }
        Ok(result)
    }
}

#[cfg(feature = "state-machines")]
impl FromStr for InteractionType {
    type Err = InteractionTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PointerUp" => Ok(InteractionType::POINTER_UP),
            "PointerDown" => Ok(InteractionType::POINTER_DOWN),
            "PointerEnter" => Ok(InteractionType::POINTER_ENTER),
            "PointerExit" => Ok(InteractionType::POINTER_EXIT),
            "PointerMove" => Ok(InteractionType::POINTER_MOVE),
            "Click" => Ok(InteractionType::CLICK),
            "OnComplete" => Ok(InteractionType::ON_COMPLETE),
            "OnLoopComplete" => Ok(InteractionType::ON_LOOP_COMPLETE),
            _ => Err(InteractionTypeParseError),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum DotLottieWgpuTargetType {
    Surface = 0,
    Texture = 1,
}

impl DotLottieWgpuTargetType {
    pub fn to_wgpu_target_type(&self) -> crate::lottie_renderer::WgpuTargetType {
        match self {
            DotLottieWgpuTargetType::Surface => crate::lottie_renderer::WgpuTargetType::Surface,
            DotLottieWgpuTargetType::Texture => crate::lottie_renderer::WgpuTargetType::Texture,
        }
    }
}

// Input events for state machine (pointer interactions)
#[allow(dead_code)]
#[repr(C)]
#[cfg(feature = "state-machines")]
pub enum DotLottieEvent {
    PointerDown { x: f32, y: f32 },
    PointerUp { x: f32, y: f32 },
    PointerMove { x: f32, y: f32 },
    PointerEnter { x: f32, y: f32 },
    PointerExit { x: f32, y: f32 },
    Click { x: f32, y: f32 },
    OnComplete,
    OnLoopComplete,
}

#[cfg(feature = "state-machines")]
impl DotLottieEvent {
    pub unsafe fn to_event(&self) -> Event {
        match self {
            DotLottieEvent::PointerDown { x, y } => Event::PointerDown { x: *x, y: *y },
            DotLottieEvent::PointerUp { x, y } => Event::PointerUp { x: *x, y: *y },
            DotLottieEvent::PointerMove { x, y } => Event::PointerMove { x: *x, y: *y },
            DotLottieEvent::PointerEnter { x, y } => Event::PointerEnter { x: *x, y: *y },
            DotLottieEvent::PointerExit { x, y } => Event::PointerExit { x: *x, y: *y },
            DotLottieEvent::Click { x, y } => Event::Click { x: *x, y: *y },
            DotLottieEvent::OnComplete => Event::OnComplete,
            DotLottieEvent::OnLoopComplete => Event::OnLoopComplete,
        }
    }
}

// ============================================================================
// Event System
// ============================================================================

// DotLottie Player Events (output events from polling)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DotLottiePlayerEventType {
    Load = 0,
    LoadError = 1,
    Play = 2,
    Pause = 3,
    Stop = 4,
    Frame = 5,
    Render = 6,
    Loop = 7,
    Complete = 8,
}

#[repr(C)]
pub union DotLottiePlayerEventData {
    pub frame_no: f32,   // For Frame and Render events
    pub loop_count: u32, // For Loop event
}

#[repr(C)]
pub struct DotLottiePlayerEvent {
    pub event_type: DotLottiePlayerEventType,
    pub data: DotLottiePlayerEventData,
}

impl From<crate::DotLottieEvent> for DotLottiePlayerEvent {
    fn from(event: crate::DotLottieEvent) -> Self {
        match event {
            crate::DotLottieEvent::Load => DotLottiePlayerEvent {
                event_type: DotLottiePlayerEventType::Load,
                data: DotLottiePlayerEventData { frame_no: 0.0 },
            },
            crate::DotLottieEvent::LoadError => DotLottiePlayerEvent {
                event_type: DotLottiePlayerEventType::LoadError,
                data: DotLottiePlayerEventData { frame_no: 0.0 },
            },
            crate::DotLottieEvent::Play => DotLottiePlayerEvent {
                event_type: DotLottiePlayerEventType::Play,
                data: DotLottiePlayerEventData { frame_no: 0.0 },
            },
            crate::DotLottieEvent::Pause => DotLottiePlayerEvent {
                event_type: DotLottiePlayerEventType::Pause,
                data: DotLottiePlayerEventData { frame_no: 0.0 },
            },
            crate::DotLottieEvent::Stop => DotLottiePlayerEvent {
                event_type: DotLottiePlayerEventType::Stop,
                data: DotLottiePlayerEventData { frame_no: 0.0 },
            },
            crate::DotLottieEvent::Frame { frame_no } => DotLottiePlayerEvent {
                event_type: DotLottiePlayerEventType::Frame,
                data: DotLottiePlayerEventData { frame_no },
            },
            crate::DotLottieEvent::Render { frame_no } => DotLottiePlayerEvent {
                event_type: DotLottiePlayerEventType::Render,
                data: DotLottiePlayerEventData { frame_no },
            },
            crate::DotLottieEvent::Loop { loop_count } => DotLottiePlayerEvent {
                event_type: DotLottiePlayerEventType::Loop,
                data: DotLottiePlayerEventData { loop_count },
            },
            crate::DotLottieEvent::Complete => DotLottiePlayerEvent {
                event_type: DotLottiePlayerEventType::Complete,
                data: DotLottiePlayerEventData { frame_no: 0.0 },
            },
        }
    }
}

// State Machine Events
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg(feature = "state-machines")]
pub enum StateMachineEventType {
    StateMachineStart = 0,
    StateMachineStop = 1,
    StateMachineTransition = 2,
    StateMachineStateEntered = 3,
    StateMachineStateExit = 4,
    StateMachineCustomEvent = 5,
    StateMachineError = 6,
    StateMachineStringInputChange = 7,
    StateMachineNumericInputChange = 8,
    StateMachineBooleanInputChange = 9,
    StateMachineInputFired = 10,
}

/// Transition event data with pointers to state names
#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(feature = "state-machines")]
pub struct StateMachineTransitionData {
    pub previous_state: *const c_char,
    pub new_state: *const c_char,
}

/// State event data (for StateEntered/StateExit)
#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(feature = "state-machines")]
pub struct StateMachineStateData {
    pub state: *const c_char,
}

/// Message event data (for CustomEvent/Error)
#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(feature = "state-machines")]
pub struct StateMachineMessageData {
    pub message: *const c_char,
}

/// String input change event data
#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(feature = "state-machines")]
pub struct StateMachineStringInputData {
    pub name: *const c_char,
    pub old_value: *const c_char,
    pub new_value: *const c_char,
}

/// Numeric input change event data
#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(feature = "state-machines")]
pub struct StateMachineNumericInputData {
    pub name: *const c_char,
    pub old_value: f32,
    pub new_value: f32,
}

/// Boolean input change event data
#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(feature = "state-machines")]
pub struct StateMachineBooleanInputData {
    pub name: *const c_char,
    pub old_value: bool,
    pub new_value: bool,
}

/// Input fired event data
#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(feature = "state-machines")]
pub struct StateMachineInputFiredData {
    pub name: *const c_char,
}

/// Union of all possible state machine event data types
#[repr(C)]
#[cfg(feature = "state-machines")]
pub union StateMachineEventData {
    pub transition: StateMachineTransitionData,
    pub state: StateMachineStateData,
    pub message: StateMachineMessageData,
    pub string_input: StateMachineStringInputData,
    pub numeric_input: StateMachineNumericInputData,
    pub boolean_input: StateMachineBooleanInputData,
    pub input_fired: StateMachineInputFiredData,
}

/// State machine event with type tag and data union.
/// String pointers are valid until the next poll call.
#[repr(C)]
#[cfg(feature = "state-machines")]
pub struct StateMachineEvent {
    pub event_type: StateMachineEventType,
    pub data: StateMachineEventData,
}

/// Internal state machine event (for framework use).
/// The message pointer is valid until the next poll call.
#[repr(C)]
#[cfg(feature = "state-machines")]
pub struct StateMachineInternalEvent {
    pub message: *const c_char,
}
