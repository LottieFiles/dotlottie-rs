use crate::string::DotString;

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

#[derive(Debug, Clone)]
pub enum StateMachineInternalEvent {
    Message { message: DotString },
}
