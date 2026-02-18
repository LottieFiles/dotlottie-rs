#![allow(clippy::missing_safety_doc)]

use bitflags::bitflags;
use core::fmt;
use core::str::FromStr;
use std::ffi::{c_char, CStr, CString};
use std::io;

use crate::state_machine_engine::events::Event;

use crate::lottie_renderer::LottieRendererError;
use crate::{
    DotLottiePlayerError, Manifest, ManifestAnimation, ManifestStateMachine, ManifestTheme,
};

pub const DOTLOTTIE_MAX_STR_LENGTH: usize = 512;

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

// Makes transfer data to C nice & easy by making the functionality available generically
pub(crate) trait Transferable<T: Sized>
where
    Self: Sized,
{
    unsafe fn new(value: &T) -> Result<Self, io::Error>;

    // Copy a single value to C
    unsafe fn copy(&self, buffer: *mut Self) {
        std::ptr::copy_nonoverlapping(self as *const Self, buffer, 1);
    }

    // Copy a callection of source values to C, performing the required translation
    unsafe fn copy_all(values: &Vec<T>, buffer: *mut Self) -> Result<(), io::Error> {
        let mut ptr = buffer;
        for value in values {
            let new_value = Self::new(value)?;
            new_value.copy(ptr);
            ptr = ptr.add(1);
        }
        Ok(())
    }

    // Perform a copy, returning the expected exit codes
    unsafe fn transfer(value: &T, result: *mut Self) -> DotLottieResult {
        if result.is_null() {
            // No destination buffer provided
            DotLottieResult::InvalidParameter
        } else if let Ok(value) = Self::new(value) {
            value.copy(result);
            DotLottieResult::Success
        } else {
            DotLottieResult::Error
        }
    }

    // Perform a copy_all, returning the expected exit codes
    unsafe fn transfer_all(
        values: &Vec<T>,
        result: *mut Self,
        size: *mut usize,
    ) -> DotLottieResult {
        if size.is_null() {
            // Size must always be provided
            DotLottieResult::InvalidParameter
        } else if result.is_null() {
            // No buffer provided: just return the size
            *size = values.len();
            DotLottieResult::Success
        } else if *size < values.len() {
            // Both buffer & size have been provided, however,
            // The size of the buffer must be big enough to hold the result
            DotLottieResult::InvalidParameter
        } else if Self::copy_all(values, result).is_ok() {
            // Return back to the user the actual number of items
            *size = values.len();
            DotLottieResult::Success
        } else {
            DotLottieResult::Error
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

impl Transferable<String> for DotLottieString {
    unsafe fn new(s: &String) -> Result<DotLottieString, io::Error> {
        let mut value: [c_char; DOTLOTTIE_MAX_STR_LENGTH] = [0; DOTLOTTIE_MAX_STR_LENGTH];
        DotLottieString::copy(s, value.as_mut_ptr(), DOTLOTTIE_MAX_STR_LENGTH)?;
        Ok(DotLottieString { value })
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

// A wrapper for Option
#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct DotLottieOption<T> {
    pub value: T,
    pub defined: bool,
}

// Specialization for strings
impl Transferable<Option<String>> for DotLottieOption<DotLottieString> {
    unsafe fn new(
        option_value: &Option<String>,
    ) -> Result<DotLottieOption<DotLottieString>, io::Error> {
        let (value, defined) = match option_value {
            Some(s) => (DotLottieString::new(s)?, true),
            _ => (DotLottieString::default(), false),
        };
        Ok(DotLottieOption { value, defined })
    }
}

impl Transferable<String> for DotLottieOption<DotLottieString> {
    unsafe fn new(value: &String) -> Result<DotLottieOption<DotLottieString>, io::Error> {
        Ok(DotLottieOption {
            value: DotLottieString::new(value)?,
            defined: true,
        })
    }
}

// Generic implementation
impl<T: Sized + Default + Copy> Transferable<Option<T>> for DotLottieOption<T> {
    unsafe fn new(option_value: &Option<T>) -> Result<DotLottieOption<T>, io::Error> {
        let (value, defined) = match option_value {
            Some(v) => (*v, true),
            _ => (T::default(), false),
        };
        Ok(DotLottieOption { value, defined })
    }
}

#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct DotLottieManifestAnimation {
    pub id: DotLottieOption<DotLottieString>,
    pub name: DotLottieOption<DotLottieString>,
    pub initial_theme: DotLottieOption<DotLottieString>,
    pub background: DotLottieOption<DotLottieString>,
}

impl Transferable<ManifestAnimation> for DotLottieManifestAnimation {
    unsafe fn new(animation: &ManifestAnimation) -> Result<DotLottieManifestAnimation, io::Error> {
        Ok(DotLottieManifestAnimation {
            id: DotLottieOption::new(&animation.id)?,
            name: DotLottieOption::new(&animation.name)?,
            initial_theme: DotLottieOption::new(&animation.initial_theme)?,
            background: DotLottieOption::new(&animation.background)?,
        })
    }
}

#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct DotLottieManifestTheme {
    pub id: DotLottieString,
    pub name: DotLottieOption<DotLottieString>,
}

impl Transferable<ManifestTheme> for DotLottieManifestTheme {
    unsafe fn new(theme: &ManifestTheme) -> Result<DotLottieManifestTheme, io::Error> {
        Ok(DotLottieManifestTheme {
            id: DotLottieString::new(&theme.id)?,
            name: DotLottieOption::new(&theme.name)?,
        })
    }
}

#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct DotLottieManifestStateMachine {
    pub id: DotLottieString,
    pub name: DotLottieOption<DotLottieString>,
}

impl Transferable<ManifestStateMachine> for DotLottieManifestStateMachine {
    unsafe fn new(
        state_machine: &ManifestStateMachine,
    ) -> Result<DotLottieManifestStateMachine, io::Error> {
        Ok(DotLottieManifestStateMachine {
            id: DotLottieString::new(&state_machine.id)?,
            name: DotLottieOption::new(&state_machine.name)?,
        })
    }
}

#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct DotLottieManifest {
    pub generator: DotLottieOption<DotLottieString>,
    pub version: DotLottieOption<DotLottieString>,
}

impl Transferable<Manifest> for DotLottieManifest {
    unsafe fn new(manifest: &Manifest) -> Result<DotLottieManifest, io::Error> {
        Ok(DotLottieManifest {
            generator: DotLottieOption::new(&manifest.generator)?,
            version: DotLottieOption::new(&manifest.version)?,
        })
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

/// Transition event data with pointers to state names
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StateMachineTransitionData {
    pub previous_state: *const c_char,
    pub new_state: *const c_char,
}

/// State event data (for StateEntered/StateExit)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StateMachineStateData {
    pub state: *const c_char,
}

/// Message event data (for CustomEvent/Error)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StateMachineMessageData {
    pub message: *const c_char,
}

/// String input change event data
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StateMachineStringInputData {
    pub name: *const c_char,
    pub old_value: *const c_char,
    pub new_value: *const c_char,
}

/// Numeric input change event data
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StateMachineNumericInputData {
    pub name: *const c_char,
    pub old_value: f32,
    pub new_value: f32,
}

/// Boolean input change event data
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StateMachineBooleanInputData {
    pub name: *const c_char,
    pub old_value: bool,
    pub new_value: bool,
}

/// Input fired event data
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StateMachineInputFiredData {
    pub name: *const c_char,
}

/// Union of all possible state machine event data types
#[repr(C)]
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
pub struct StateMachineEvent {
    pub event_type: StateMachineEventType,
    pub data: StateMachineEventData,
}

/// Internal state machine event (for framework use).
/// The message pointer is valid until the next poll call.
#[repr(C)]
pub struct StateMachineInternalEvent {
    pub message: *const c_char,
}
