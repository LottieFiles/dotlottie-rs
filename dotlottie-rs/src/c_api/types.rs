#![allow(clippy::missing_safety_doc)]

use bitflags::bitflags;
use core::fmt;
use core::str::FromStr;
use std::ffi::{c_char, CStr, CString};
use std::io;

use crate::state_machine_engine::events::Event;

use crate::lottie_renderer::LottieRendererError;
use crate::DotLottiePlayerError;

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

// Other constant(s)
pub const DOTLOTTIE_MAX_STR_LENGTH: usize = 512;

// This type allows us to work with Interaction Types as bit flags and easily communicate this
// information to the C side
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
pub(crate) struct InteractionTypeParseError;

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

// A string struct used to ensure a buffer exists and is owned on the client side that can
// accomodate a value of the maximum size we would want to write into it.
#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct DotLottieString {
    pub value: [c_char; DOTLOTTIE_MAX_STR_LENGTH],
}

impl DotLottieString {
    pub unsafe fn read(value: *const c_char) -> Result<CString, io::Error> {
        if value.is_null() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "null pointer"));
        }
        Ok(CStr::from_ptr(value).to_owned())
    }

    // Copy a rust string out into a C string
    pub unsafe fn copy(value: &str, buffer: *mut c_char, size: usize) -> Result<(), io::Error> {
        if buffer.is_null() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "null buffer"));
        }

        let bytes = value.as_bytes();

        // Check for interior null bytes (same check CString::new does)
        if bytes.contains(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "interior null byte",
            ));
        }

        let required_len = bytes.len() + 1; // +1 for null terminator
        if required_len > size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "value too large",
            ));
        }

        // Direct copy - no intermediate allocation
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
        // Add null terminator
        *buffer.add(bytes.len()) = 0;

        Ok(())
    }
}

impl fmt::Display for DotLottieString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cstring = unsafe {
            DotLottieString::read(self.value.as_ptr() as *const c_char).map_err(|_| fmt::Error)?
        };
        let value = cstring.to_str().map_err(|_| fmt::Error)?;
        write!(f, "{value}")
    }
}

impl Default for DotLottieString {
    fn default() -> Self {
        DotLottieString {
            value: [0; DOTLOTTIE_MAX_STR_LENGTH],
        }
    }
}

// Input events for state machine (pointer interactions)
#[allow(dead_code)]
#[repr(C)]
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

// For string-based event data (states, input names, messages)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StateMachineStringData {
    pub str1: [c_char; DOTLOTTIE_MAX_STR_LENGTH],
    pub str2: [c_char; DOTLOTTIE_MAX_STR_LENGTH],
    pub str3: [c_char; DOTLOTTIE_MAX_STR_LENGTH],
}

// For numeric input changes
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StateMachineNumericData {
    pub name: [c_char; DOTLOTTIE_MAX_STR_LENGTH],
    pub old_value: f32,
    pub new_value: f32,
}

// For boolean input changes
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StateMachineBooleanData {
    pub name: [c_char; DOTLOTTIE_MAX_STR_LENGTH],
    pub old_value: bool,
    pub new_value: bool,
}

#[repr(C)]
pub union StateMachineEventData {
    pub strings: StateMachineStringData,
    pub numeric: StateMachineNumericData,
    pub boolean: StateMachineBooleanData,
}

#[repr(C)]
pub struct StateMachineEvent {
    pub event_type: StateMachineEventType,
    pub data: StateMachineEventData,
}

impl StateMachineEvent {
    pub unsafe fn from_rust(event: crate::StateMachineEvent) -> Result<Self, io::Error> {
        match event {
            crate::StateMachineEvent::Start => Ok(StateMachineEvent {
                event_type: StateMachineEventType::StateMachineStart,
                data: StateMachineEventData {
                    strings: StateMachineStringData {
                        str1: [0; DOTLOTTIE_MAX_STR_LENGTH],
                        str2: [0; DOTLOTTIE_MAX_STR_LENGTH],
                        str3: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    },
                },
            }),
            crate::StateMachineEvent::Stop => Ok(StateMachineEvent {
                event_type: StateMachineEventType::StateMachineStop,
                data: StateMachineEventData {
                    strings: StateMachineStringData {
                        str1: [0; DOTLOTTIE_MAX_STR_LENGTH],
                        str2: [0; DOTLOTTIE_MAX_STR_LENGTH],
                        str3: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    },
                },
            }),
            crate::StateMachineEvent::Transition {
                previous_state,
                new_state,
            } => {
                let mut data = StateMachineStringData {
                    str1: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str2: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str3: [0; DOTLOTTIE_MAX_STR_LENGTH],
                };
                DotLottieString::copy(
                    &previous_state,
                    data.str1.as_mut_ptr(),
                    DOTLOTTIE_MAX_STR_LENGTH,
                )?;
                DotLottieString::copy(
                    &new_state,
                    data.str2.as_mut_ptr(),
                    DOTLOTTIE_MAX_STR_LENGTH,
                )?;
                Ok(StateMachineEvent {
                    event_type: StateMachineEventType::StateMachineTransition,
                    data: StateMachineEventData { strings: data },
                })
            }
            crate::StateMachineEvent::StateEntered { state } => {
                let mut data = StateMachineStringData {
                    str1: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str2: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str3: [0; DOTLOTTIE_MAX_STR_LENGTH],
                };
                DotLottieString::copy(&state, data.str1.as_mut_ptr(), DOTLOTTIE_MAX_STR_LENGTH)?;
                Ok(StateMachineEvent {
                    event_type: StateMachineEventType::StateMachineStateEntered,
                    data: StateMachineEventData { strings: data },
                })
            }
            crate::StateMachineEvent::StateExit { state } => {
                let mut data = StateMachineStringData {
                    str1: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str2: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str3: [0; DOTLOTTIE_MAX_STR_LENGTH],
                };
                DotLottieString::copy(&state, data.str1.as_mut_ptr(), DOTLOTTIE_MAX_STR_LENGTH)?;
                Ok(StateMachineEvent {
                    event_type: StateMachineEventType::StateMachineStateExit,
                    data: StateMachineEventData { strings: data },
                })
            }
            crate::StateMachineEvent::CustomEvent { message } => {
                let mut data = StateMachineStringData {
                    str1: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str2: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str3: [0; DOTLOTTIE_MAX_STR_LENGTH],
                };
                DotLottieString::copy(&message, data.str1.as_mut_ptr(), DOTLOTTIE_MAX_STR_LENGTH)?;
                Ok(StateMachineEvent {
                    event_type: StateMachineEventType::StateMachineCustomEvent,
                    data: StateMachineEventData { strings: data },
                })
            }
            crate::StateMachineEvent::Error { message } => {
                let mut data = StateMachineStringData {
                    str1: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str2: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str3: [0; DOTLOTTIE_MAX_STR_LENGTH],
                };
                DotLottieString::copy(&message, data.str1.as_mut_ptr(), DOTLOTTIE_MAX_STR_LENGTH)?;
                Ok(StateMachineEvent {
                    event_type: StateMachineEventType::StateMachineError,
                    data: StateMachineEventData { strings: data },
                })
            }
            crate::StateMachineEvent::StringInputChange {
                name,
                old_value,
                new_value,
            } => {
                let mut data = StateMachineStringData {
                    str1: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str2: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str3: [0; DOTLOTTIE_MAX_STR_LENGTH],
                };
                DotLottieString::copy(&name, data.str1.as_mut_ptr(), DOTLOTTIE_MAX_STR_LENGTH)?;
                DotLottieString::copy(
                    &old_value,
                    data.str2.as_mut_ptr(),
                    DOTLOTTIE_MAX_STR_LENGTH,
                )?;
                DotLottieString::copy(
                    &new_value,
                    data.str3.as_mut_ptr(),
                    DOTLOTTIE_MAX_STR_LENGTH,
                )?;
                Ok(StateMachineEvent {
                    event_type: StateMachineEventType::StateMachineStringInputChange,
                    data: StateMachineEventData { strings: data },
                })
            }
            crate::StateMachineEvent::NumericInputChange {
                name,
                old_value,
                new_value,
            } => {
                let mut data = StateMachineNumericData {
                    name: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    old_value,
                    new_value,
                };
                DotLottieString::copy(&name, data.name.as_mut_ptr(), DOTLOTTIE_MAX_STR_LENGTH)?;
                Ok(StateMachineEvent {
                    event_type: StateMachineEventType::StateMachineNumericInputChange,
                    data: StateMachineEventData { numeric: data },
                })
            }
            crate::StateMachineEvent::BooleanInputChange {
                name,
                old_value,
                new_value,
            } => {
                let mut data = StateMachineBooleanData {
                    name: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    old_value,
                    new_value,
                };
                DotLottieString::copy(&name, data.name.as_mut_ptr(), DOTLOTTIE_MAX_STR_LENGTH)?;
                Ok(StateMachineEvent {
                    event_type: StateMachineEventType::StateMachineBooleanInputChange,
                    data: StateMachineEventData { boolean: data },
                })
            }
            crate::StateMachineEvent::InputFired { name } => {
                let mut data = StateMachineStringData {
                    str1: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str2: [0; DOTLOTTIE_MAX_STR_LENGTH],
                    str3: [0; DOTLOTTIE_MAX_STR_LENGTH],
                };
                DotLottieString::copy(&name, data.str1.as_mut_ptr(), DOTLOTTIE_MAX_STR_LENGTH)?;
                Ok(StateMachineEvent {
                    event_type: StateMachineEventType::StateMachineInputFired,
                    data: StateMachineEventData { strings: data },
                })
            }
        }
    }
}

// Internal State Machine Events (for framework use)
#[repr(C)]
pub struct StateMachineInternalEvent {
    pub message: [c_char; DOTLOTTIE_MAX_STR_LENGTH],
}

impl StateMachineInternalEvent {
    pub unsafe fn from_rust(event: crate::StateMachineInternalEvent) -> Result<Self, io::Error> {
        match event {
            crate::StateMachineInternalEvent::Message { message } => {
                let mut data = StateMachineInternalEvent {
                    message: [0; DOTLOTTIE_MAX_STR_LENGTH],
                };
                DotLottieString::copy(
                    &message,
                    data.message.as_mut_ptr(),
                    DOTLOTTIE_MAX_STR_LENGTH,
                )?;
                Ok(data)
            }
        }
    }
}
