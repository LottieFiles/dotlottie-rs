use bitflags::bitflags;
use core::fmt;
use core::str::FromStr;
use std::ffi::{c_char, CStr, CString};
use std::io;

use crate::{
    Config, Event, Fit, Layout, Manifest, ManifestAnimation, ManifestStateMachine, ManifestTheme,
    Marker, Mode,
};

// Function return codes
pub const DOTLOTTIE_SUCCESS: i32 = 0;
pub const DOTLOTTIE_ERROR: i32 = 1;
pub const DOTLOTTIE_INVALID_PARAMETER: i32 = 2;
pub const DOTLOTTIE_MANIFEST_NOT_AVAILABLE: i32 = 3;

// Other constant(s)
pub const DOTLOTTIE_MAX_STR_LENGTH: usize = 512;

pub const LISTENER_TYPE_UNSET: u16 = 0;
pub const LISTENER_TYPE_POINTER_UP: u16 = 1 << 0;
pub const LISTENER_TYPE_POINTER_DOWN: u16 = 1 << 1;
pub const LISTENER_TYPE_POINTER_ENTER: u16 = 1 << 2;
pub const LISTENER_TYPE_POINTER_EXIT: u16 = 1 << 3;
pub const LISTENER_TYPE_POINTER_MOVE: u16 = 1 << 4;

// This type allows us to work with Interaction Types as bit flags and easily communicate this
// information to the C side
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(C)]
    pub(crate) struct InteractionType: u16 {
        const UNSET = LISTENER_TYPE_UNSET;

        const POINTER_UP    = LISTENER_TYPE_POINTER_UP;
        const POINTER_DOWN  = LISTENER_TYPE_POINTER_DOWN;
        const POINTER_ENTER = LISTENER_TYPE_POINTER_ENTER;
        const POINTER_EXIT  = LISTENER_TYPE_POINTER_EXIT;
        const POINTER_MOVE  = LISTENER_TYPE_POINTER_MOVE;
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
    unsafe fn transfer(value: &T, result: *mut Self) -> i32 {
        if result.is_null() {
            // No destination buffer provided
            DOTLOTTIE_INVALID_PARAMETER
        } else if let Ok(value) = Self::new(value) {
            value.copy(result);
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_ERROR
        }
    }

    // Perform a copy_all, returning the expected exit codes
    unsafe fn transfer_all(values: &Vec<T>, result: *mut Self, size: *mut usize) -> i32 {
        if size.is_null() {
            // Size must always be provided
            DOTLOTTIE_INVALID_PARAMETER
        } else if result.is_null() {
            // No buffer provided: just return the size
            *size = values.len();
            DOTLOTTIE_SUCCESS
        } else if *size < values.len() {
            // Both buffer & size have been provided, however,
            // The size of the buffer must be big enough to hold the result
            DOTLOTTIE_INVALID_PARAMETER
        } else if Self::copy_all(values, result).is_ok() {
            // Return back to the user the actual number of items
            *size = values.len();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_ERROR
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
    // Read a C string into a rust string
    pub unsafe fn read(value: *const c_char) -> Result<String, io::Error> {
        if value.is_null() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "null pointer"));
        }
        match CStr::from_ptr(value).to_str() {
            Ok(s) => Ok(s.to_owned()),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid utf8 sequence",
            )),
        }
    }

    // Copy a rust string out into a C string
    pub unsafe fn copy(value: &str, buffer: *mut c_char, size: usize) -> Result<(), io::Error> {
        if buffer.is_null() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "null buffer"));
        }
        let native_string = CString::new(value)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "null pointer"))?;
        let bytes = native_string.as_bytes_with_nul();
        if bytes.len() <= size {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "value too large",
            ))
        }
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
        let value = unsafe {
            DotLottieString::read(self.value.as_ptr() as *const c_char).map_err(|_| fmt::Error)?
        };
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

// The following types mirror types in dotlottie-rs for various reasons, e.g. because strings are
// used, unsuitable enum variants, etc. They also typically need to be copied over to C, and
// implementing Transferable helps with this.

#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct DotLottieConfig {
    pub mode: Mode,
    pub loop_animation: bool,
    pub loop_count: u32,
    pub speed: f32,
    pub use_frame_interpolation: bool,
    pub autoplay: bool,
    pub segment_start: f32,
    pub segment_end: f32,
    pub background_color: u32,
    pub layout: DotLottieLayout,
    pub marker: DotLottieString,
    pub theme_id: DotLottieString,
    pub state_machine_id: DotLottieString,
    pub animation_id: DotLottieString,
}

impl Transferable<Config> for DotLottieConfig {
    unsafe fn new(config: &Config) -> Result<DotLottieConfig, io::Error> {
        let (segment_start, segment_end) = match config.segment[..] {
            [start, end] => (start, end),
            _ => (-1.0, -1.0),
        };
        Ok(DotLottieConfig {
            mode: config.mode,
            loop_animation: config.loop_animation,
            loop_count: config.loop_count,
            speed: config.speed,
            use_frame_interpolation: config.use_frame_interpolation,
            autoplay: config.autoplay,
            segment_start,
            segment_end,
            background_color: config.background_color,
            layout: DotLottieLayout::new(&config.layout),
            marker: DotLottieString::new(&config.marker)?,
            theme_id: DotLottieString::new(&config.theme_id)?,
            state_machine_id: DotLottieString::new(&config.state_machine_id)?,
            animation_id: DotLottieString::new(&config.animation_id)?,
        })
    }
}

impl DotLottieConfig {
    pub unsafe fn to_config(&self) -> Result<Config, io::Error> {
        Ok(Config {
            mode: self.mode,
            loop_animation: self.loop_animation,
            loop_count: self.loop_count,
            speed: self.speed,
            use_frame_interpolation: self.use_frame_interpolation,
            autoplay: self.autoplay,
            segment: if self.segment_start >= 0f32 && self.segment_end >= 0f32 {
                vec![self.segment_start, self.segment_end]
            } else {
                vec![]
            },
            background_color: self.background_color,
            layout: self.layout.to_layout(),
            marker: self.marker.to_string(),
            theme_id: self.theme_id.to_string(),
            state_machine_id: self.state_machine_id.to_string(),
            animation_id: self.animation_id.to_string(),
        })
    }
}

#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct DotLottieMarker {
    pub name: DotLottieString,
    pub duration: f32,
    pub time: f32,
}

impl Transferable<Marker> for DotLottieMarker {
    unsafe fn new(marker: &Marker) -> Result<DotLottieMarker, io::Error> {
        Ok(DotLottieMarker {
            name: DotLottieString::new(&marker.name)?,
            duration: marker.duration,
            time: marker.time,
        })
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

#[derive(Clone, PartialEq)]
#[repr(C)]
pub enum DotLottieFit {
    Contain,
    Fill,
    Cover,
    FitWidth,
    FitHeight,
    Void,
}

impl DotLottieFit {
    pub fn new(fit: Fit) -> DotLottieFit {
        match fit {
            Fit::Contain => DotLottieFit::Contain,
            Fit::Fill => DotLottieFit::Fill,
            Fit::Cover => DotLottieFit::Cover,
            Fit::FitWidth => DotLottieFit::FitWidth,
            Fit::FitHeight => DotLottieFit::FitHeight,
            Fit::None => DotLottieFit::Void,
        }
    }

    pub fn to_fit(&self) -> Fit {
        match self {
            DotLottieFit::Contain => Fit::Contain,
            DotLottieFit::Fill => Fit::Fill,
            DotLottieFit::Cover => Fit::Cover,
            DotLottieFit::FitWidth => Fit::FitWidth,
            DotLottieFit::FitHeight => Fit::FitHeight,
            DotLottieFit::Void => Fit::None,
        }
    }
}

#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct DotLottieLayout {
    pub fit: DotLottieFit,
    pub align_x: f32,
    pub align_y: f32,
}

impl DotLottieLayout {
    pub fn new(layout: &Layout) -> DotLottieLayout {
        let (align_x, align_y) = match layout.align[..] {
            [align_x, align_y] => (align_x, align_y),
            _ => (-1.0, -1.0),
        };
        DotLottieLayout {
            fit: DotLottieFit::new(layout.fit),
            align_x,
            align_y,
        }
    }

    pub fn to_layout(&self) -> Layout {
        Layout {
            fit: self.fit.to_fit(),
            align: if self.align_x >= 0f32 && self.align_y >= 0f32 {
                vec![self.align_x, self.align_y]
            } else {
                vec![]
            },
        }
    }
}

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
