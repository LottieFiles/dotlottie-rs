use bitflags::bitflags;
use core::fmt;
use core::str::FromStr;
use std::ffi::{c_char, CStr, CString};
use std::io;
use std::sync::Arc;

use dotlottie_rs::{
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

pub type OnOp = unsafe extern "C" fn();

// Function pointer types for observers
pub type OnFrameOp = unsafe extern "C" fn(f32);
pub type OnRenderOp = unsafe extern "C" fn(f32);
pub type OnLoopOp = unsafe extern "C" fn(u32);

#[repr(C)]
pub struct Observer {
    pub on_load_op: OnOp,
    pub on_load_error_op: OnOp,
    pub on_play_op: OnOp,
    pub on_pause_op: OnOp,
    pub on_stop_op: OnOp,
    pub on_frame_op: OnFrameOp,
    pub on_render_op: OnRenderOp,
    pub on_loop_op: OnLoopOp,
    pub on_complete_op: OnOp,
}

impl dotlottie_rs::Observer for Observer {
    fn on_load(&self) {
        unsafe { (self.on_load_op)() }
    }
    fn on_load_error(&self) {
        unsafe { (self.on_load_error_op)() }
    }
    fn on_play(&self) {
        unsafe { (self.on_play_op)() }
    }
    fn on_pause(&self) {
        unsafe { (self.on_pause_op)() }
    }
    fn on_stop(&self) {
        unsafe { (self.on_stop_op)() }
    }
    fn on_frame(&self, frame_no: f32) {
        unsafe { (self.on_frame_op)(frame_no) }
    }
    fn on_render(&self, frame_no: f32) {
        unsafe { (self.on_render_op)(frame_no) }
    }
    fn on_loop(&self, loop_count: u32) {
        unsafe { (self.on_loop_op)(loop_count) }
    }
    fn on_complete(&self) {
        unsafe { (self.on_complete_op)() }
    }
}

impl Observer {
    pub unsafe fn as_observer(&mut self) -> Arc<dyn dotlottie_rs::Observer> {
        Arc::from(Box::from_raw(self as *mut dyn dotlottie_rs::Observer))
    }
}

// Function pointer types for state machine observers
pub type OnTransitionOp = unsafe extern "C" fn(*const c_char, *const c_char);
pub type OnStateEnteredOp = unsafe extern "C" fn(*const c_char);
pub type OnStateExitOp = unsafe extern "C" fn(*const c_char);
pub type OnStateCustomEventOp = unsafe extern "C" fn(*const c_char);
pub type OnStateErrorOp = unsafe extern "C" fn(*const c_char);
pub type OnStateMachineStartOp = unsafe extern "C" fn();
pub type OnStateMachineStopOp = unsafe extern "C" fn();
pub type OnStringInputValueChangeOp =
    unsafe extern "C" fn(*const c_char, *const c_char, *const c_char);
pub type OnNumericInputValueChangeOp = unsafe extern "C" fn(*const c_char, f32, f32);
pub type OnBooleanInputValueChangeOp = unsafe extern "C" fn(*const c_char, bool, bool);
pub type OnInputFiredOp = unsafe extern "C" fn(*const c_char);

#[repr(C)]
pub struct StateMachineObserver {
    pub on_transition_op: OnTransitionOp,
    pub on_state_entered_op: OnStateEnteredOp,
    pub on_state_exit_op: OnStateExitOp,
    pub on_state_custom_event_op: OnStateCustomEventOp,
    pub on_state_error_op: OnStateErrorOp,
    pub on_state_machine_start_op: OnStateMachineStartOp,
    pub on_state_machine_stop_op: OnStateMachineStopOp,
    pub on_string_input_value_change_op: OnStringInputValueChangeOp,
    pub on_numeric_input_value_change_op: OnNumericInputValueChangeOp,
    pub on_boolean_input_value_change_op: OnBooleanInputValueChangeOp,
    pub on_input_fired_op: OnInputFiredOp,
}

impl dotlottie_rs::StateMachineObserver for StateMachineObserver {
    fn on_transition(&self, previous_state: String, new_state: String) {
        if let (Ok(previous_state), Ok(new_state)) =
            (CString::new(previous_state), CString::new(new_state))
        {
            unsafe {
                (self.on_transition_op)(
                    previous_state.as_bytes_with_nul().as_ptr() as *const c_char,
                    new_state.as_bytes_with_nul().as_ptr() as *const c_char,
                )
            }
        }
    }

    fn on_state_entered(&self, entering_state: String) {
        if let Ok(entering_state) = CString::new(entering_state) {
            unsafe {
                (self.on_state_entered_op)(
                    entering_state.as_bytes_with_nul().as_ptr() as *const c_char
                )
            }
        }
    }

    fn on_state_exit(&self, leaving_state: String) {
        if let Ok(leaving_state) = CString::new(leaving_state) {
            unsafe {
                (self.on_state_exit_op)(leaving_state.as_bytes_with_nul().as_ptr() as *const c_char)
            }
        }
    }

    fn on_custom_event(&self, message: String) {
        if let Ok(message) = CString::new(message) {
            unsafe {
                (self.on_state_custom_event_op)(
                    message.as_bytes_with_nul().as_ptr() as *const c_char
                )
            }
        }
    }

    fn on_error(&self, message: String) {
        if let Ok(message) = CString::new(message) {
            unsafe {
                (self.on_state_error_op)(message.as_bytes_with_nul().as_ptr() as *const c_char)
            }
        }
    }

    fn on_start(&self) {
        unsafe { (self.on_state_machine_start_op)() }
    }

    fn on_stop(&self) {
        unsafe { (self.on_state_machine_stop_op)() }
    }

    fn on_string_input_value_change(
        &self,
        input_name: String,
        old_value: String,
        new_value: String,
    ) {
        if let (Ok(input_name), Ok(old_value), Ok(new_value)) = (
            CString::new(input_name),
            CString::new(old_value),
            CString::new(new_value),
        ) {
            unsafe {
                (self.on_string_input_value_change_op)(
                    input_name.as_bytes_with_nul().as_ptr() as *const c_char,
                    old_value.as_bytes_with_nul().as_ptr() as *const c_char,
                    new_value.as_bytes_with_nul().as_ptr() as *const c_char,
                )
            }
        }
    }

    fn on_numeric_input_value_change(&self, input_name: String, old_value: f32, new_value: f32) {
        if let Ok(input_name) = CString::new(input_name) {
            unsafe {
                (self.on_numeric_input_value_change_op)(
                    input_name.as_bytes_with_nul().as_ptr() as *const c_char,
                    old_value,
                    new_value,
                )
            }
        }
    }

    fn on_boolean_input_value_change(&self, input_name: String, old_value: bool, new_value: bool) {
        if let Ok(input_name) = CString::new(input_name) {
            unsafe {
                (self.on_boolean_input_value_change_op)(
                    input_name.as_bytes_with_nul().as_ptr() as *const c_char,
                    old_value,
                    new_value,
                )
            }
        }
    }

    fn on_input_fired(&self, input_name: String) {
        if let Ok(input_name) = CString::new(input_name) {
            unsafe {
                (self.on_input_fired_op)(input_name.as_bytes_with_nul().as_ptr() as *const c_char)
            }
        }
    }
}

impl StateMachineObserver {
    pub unsafe fn as_observer(&mut self) -> Arc<dyn dotlottie_rs::StateMachineObserver> {
        Arc::from(Box::from_raw(
            self as *mut dyn dotlottie_rs::StateMachineObserver,
        ))
    }
}

pub type OnMessageOp = unsafe extern "C" fn(*const c_char);

#[repr(C)]
pub struct InternalStateMachineObserver {
    pub on_message_op: OnMessageOp,
}

impl dotlottie_rs::InternalStateMachineObserver for InternalStateMachineObserver {
    fn on_message(&self, message: String) {
        if let Ok(message) = CString::new(message) {
            unsafe { (self.on_message_op)(message.as_bytes_with_nul().as_ptr() as *const c_char) }
        }
    }
}

impl InternalStateMachineObserver {
    pub unsafe fn as_observer(&mut self) -> Arc<dyn dotlottie_rs::InternalStateMachineObserver> {
        Arc::from(Box::from_raw(
            self as *mut dyn dotlottie_rs::InternalStateMachineObserver,
        ))
    }
}
