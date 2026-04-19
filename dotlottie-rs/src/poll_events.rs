use std::ffi::CString;

pub use crate::event_queue::EventQueue;

/// Events emitted by the DotLottie player.
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerEvent {
    Load,
    LoadError,
    Play,
    Pause,
    Stop,
    Frame { frame_no: f32 },
    Render { frame_no: f32 },
    Loop { loop_count: u32 },
    Complete,
}

#[derive(Debug, Clone)]
pub enum StateMachineEvent {
    Start,
    Stop,
    Transition {
        previous_state: CString,
        new_state: CString,
    },
    StateEntered {
        state: CString,
    },
    StateExit {
        state: CString,
    },
    CustomEvent {
        message: CString,
    },
    Error {
        message: CString,
    },
    StringInputChange {
        name: CString,
        old_value: CString,
        new_value: CString,
    },
    NumericInputChange {
        name: CString,
        old_value: f32,
        new_value: f32,
    },
    BooleanInputChange {
        name: CString,
        old_value: bool,
        new_value: bool,
    },
    InputFired {
        name: CString,
    },
}

#[derive(Debug, Clone)]
pub enum StateMachineInternalEvent {
    Message { message: CString },
}
