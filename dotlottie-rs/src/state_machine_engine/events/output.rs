use std::ffi::CString;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_and_read() {
        let event = StateMachineEvent::Transition {
            previous_state: CString::new("a").unwrap(),
            new_state: CString::new("b").unwrap(),
        };
        if let StateMachineEvent::Transition {
            previous_state,
            new_state,
        } = &event
        {
            assert_eq!(previous_state.to_str().unwrap(), "a");
            assert_eq!(new_state.to_str().unwrap(), "b");
        } else {
            panic!("expected Transition");
        }
    }

    #[test]
    fn internal_event_message() {
        let event = StateMachineInternalEvent::Message {
            message: CString::new("m").unwrap(),
        };
        let StateMachineInternalEvent::Message { message } = &event;
        assert_eq!(message.to_str().unwrap(), "m");
    }
}
