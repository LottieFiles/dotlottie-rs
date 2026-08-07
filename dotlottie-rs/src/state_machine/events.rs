//! State machine events: [`Event`] comes in from the host, and
//! [`StateMachineEvent`] / [`StateMachineInternalEvent`] go back out.

use crate::string::DotString;

/// The discriminant shared by [`Event`] and
/// [`Interaction`](super::interactions::Interaction).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    PointerDown,
    PointerUp,
    PointerMove,
    PointerEnter,
    PointerExit,
    Click,
    OnComplete,
    OnLoopComplete,
}

impl EventKind {
    /// The name used by the interaction schema.
    pub fn as_str(self) -> &'static str {
        match self {
            EventKind::PointerDown => "PointerDown",
            EventKind::PointerUp => "PointerUp",
            EventKind::PointerMove => "PointerMove",
            EventKind::PointerEnter => "PointerEnter",
            EventKind::PointerExit => "PointerExit",
            EventKind::Click => "Click",
            EventKind::OnComplete => "OnComplete",
            EventKind::OnLoopComplete => "OnLoopComplete",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    PointerDown { x: f32, y: f32 },
    PointerUp { x: f32, y: f32 },
    PointerMove { x: f32, y: f32 },
    PointerEnter { x: f32, y: f32 },
    PointerExit { x: f32, y: f32 },
    Click { x: f32, y: f32 },
    OnComplete,
    OnLoopComplete,
}

impl Event {
    pub fn kind(&self) -> EventKind {
        match self {
            Event::PointerDown { .. } => EventKind::PointerDown,
            Event::PointerUp { .. } => EventKind::PointerUp,
            Event::PointerMove { .. } => EventKind::PointerMove,
            Event::PointerEnter { .. } => EventKind::PointerEnter,
            Event::PointerExit { .. } => EventKind::PointerExit,
            Event::Click { .. } => EventKind::Click,
            Event::OnComplete => EventKind::OnComplete,
            Event::OnLoopComplete => EventKind::OnLoopComplete,
        }
    }

    /// Pointer position, or the origin for the player-driven events.
    pub fn position(&self) -> (f32, f32) {
        match self {
            Event::PointerDown { x, y }
            | Event::PointerUp { x, y }
            | Event::PointerMove { x, y }
            | Event::PointerEnter { x, y }
            | Event::PointerExit { x, y }
            | Event::Click { x, y } => (*x, *y),
            Event::OnComplete | Event::OnLoopComplete => (0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StateMachineEvent {
    Start,
    Stop,
    Transition {
        previous_state: DotString,
        new_state: DotString,
    },
    StateEntered {
        state: DotString,
    },
    StateExit {
        state: DotString,
    },
    CustomEvent {
        message: DotString,
    },
    Error {
        message: DotString,
    },
    StringInputChange {
        name: DotString,
        old_value: DotString,
        new_value: DotString,
    },
    NumericInputChange {
        name: DotString,
        old_value: f32,
        new_value: f32,
    },
    BooleanInputChange {
        name: DotString,
        old_value: bool,
        new_value: bool,
    },
    InputFired {
        name: DotString,
    },
}

/// Framework-only channel. Currently carries only `OpenUrl` requests.
#[derive(Debug, Clone)]
pub enum StateMachineInternalEvent {
    Message { message: DotString },
}
