use std::collections::VecDeque;
use std::ffi::CString;

/// Events emitted by the DotLottie player.
#[derive(Debug, Clone, PartialEq)]
pub enum DotLottieEvent {
    Load,
    LoadError,
    Play,
    Pause,
    Stop,
    Frame { frame_no: f32 },
    Render { frame_no: f32 },
    Loop { loop_count: u32 },
    Complete,
    #[cfg(feature = "audio")]
    AudioPlay { ref_id: String },
    #[cfg(feature = "audio")]
    AudioPause { ref_id: String },
    #[cfg(feature = "audio")]
    AudioStop { ref_id: String },
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

pub const MAX_EVENTS: usize = 256;

/// Bounded event queue.
///
/// - Fixed maximum size ([`MAX_EVENTS`]).
/// - When full, the oldest event is dropped.
/// - Single-threaded; no synchronization overhead.
pub struct EventQueue<T> {
    queue: VecDeque<T>,
    max_size: usize,
}

impl<T> EventQueue<T> {
    /// Creates a new event queue with the default capacity.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(MAX_EVENTS),
            max_size: MAX_EVENTS,
        }
    }

    /// Pushes an event onto the queue.
    ///
    /// If the queue is full, the oldest event is dropped.
    pub fn push(&mut self, event: T) {
        if self.queue.len() >= self.max_size {
            self.queue.pop_front();
        }

        self.queue.push_back(event);
    }

    /// Removes and returns the next event from the queue.
    ///
    /// Returns `Some(event)` if an event is available, `None` if the queue is empty.
    pub fn poll(&mut self) -> Option<T> {
        self.queue.pop_front()
    }

    /// Returns the number of events currently in the queue.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Removes all events from the queue.
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

impl<T> Default for EventQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_queue_basic() {
        let mut queue = EventQueue::<DotLottieEvent>::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        queue.push(DotLottieEvent::Load);
        assert_eq!(queue.len(), 1);

        let event = queue.poll();
        assert_eq!(event, Some(DotLottieEvent::Load));
        assert!(queue.is_empty());
    }

    #[test]
    fn test_frame_events() {
        let mut queue = EventQueue::<DotLottieEvent>::new();

        queue.push(DotLottieEvent::Frame { frame_no: 10.0 });
        queue.push(DotLottieEvent::Frame { frame_no: 11.0 });
        queue.push(DotLottieEvent::Frame { frame_no: 12.0 });

        assert_eq!(queue.len(), 3);

        assert_eq!(queue.poll(), Some(DotLottieEvent::Frame { frame_no: 10.0 }));
        assert_eq!(queue.poll(), Some(DotLottieEvent::Frame { frame_no: 11.0 }));
        assert_eq!(queue.poll(), Some(DotLottieEvent::Frame { frame_no: 12.0 }));
    }

    #[test]
    fn test_render_events() {
        let mut queue = EventQueue::<DotLottieEvent>::new();

        queue.push(DotLottieEvent::Render { frame_no: 5.0 });
        queue.push(DotLottieEvent::Render { frame_no: 6.0 });

        assert_eq!(queue.len(), 2);

        assert_eq!(queue.poll(), Some(DotLottieEvent::Render { frame_no: 5.0 }));
        assert_eq!(queue.poll(), Some(DotLottieEvent::Render { frame_no: 6.0 }));
    }

    #[test]
    fn test_multiple_event_types() {
        let mut queue = EventQueue::<DotLottieEvent>::new();

        queue.push(DotLottieEvent::Load);
        queue.push(DotLottieEvent::Play);
        queue.push(DotLottieEvent::Frame { frame_no: 10.0 });

        assert_eq!(queue.len(), 3);
    }

    #[test]
    fn test_queue_overflow() {
        let mut queue = EventQueue::<DotLottieEvent>::new();

        for i in 0..300 {
            queue.push(DotLottieEvent::Loop { loop_count: i });
        }

        assert_eq!(queue.len(), MAX_EVENTS);

        let event = queue.poll();
        assert_eq!(event, Some(DotLottieEvent::Loop { loop_count: 44 }));
    }

    #[test]
    fn test_clear() {
        let mut queue = EventQueue::<DotLottieEvent>::new();

        queue.push(DotLottieEvent::Load);
        queue.push(DotLottieEvent::Play);
        assert_eq!(queue.len(), 2);

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_state_machine_event_cstring() {
        // Test Start event
        let event = StateMachineEvent::Start;
        assert!(matches!(event, StateMachineEvent::Start));

        // Test Stop event
        let event = StateMachineEvent::Stop;
        assert!(matches!(event, StateMachineEvent::Stop));

        // Test Transition event with CString
        let event = StateMachineEvent::Transition {
            previous_state: CString::new("state_a").unwrap(),
            new_state: CString::new("state_b").unwrap(),
        };
        if let StateMachineEvent::Transition {
            previous_state,
            new_state,
        } = &event
        {
            assert_eq!(previous_state.to_str().unwrap(), "state_a");
            assert_eq!(new_state.to_str().unwrap(), "state_b");
        } else {
            panic!("Expected Transition variant");
        }

        // Test StateEntered event
        let event = StateMachineEvent::StateEntered {
            state: CString::new("entered_state").unwrap(),
        };
        if let StateMachineEvent::StateEntered { state } = &event {
            assert_eq!(state.to_str().unwrap(), "entered_state");
        } else {
            panic!("Expected StateEntered variant");
        }

        // Test NumericInputChange event
        let event = StateMachineEvent::NumericInputChange {
            name: CString::new("speed").unwrap(),
            old_value: 1.0,
            new_value: 2.5,
        };
        if let StateMachineEvent::NumericInputChange {
            name,
            old_value,
            new_value,
        } = &event
        {
            assert_eq!(name.to_str().unwrap(), "speed");
            assert_eq!(*old_value, 1.0);
            assert_eq!(*new_value, 2.5);
        } else {
            panic!("Expected NumericInputChange variant");
        }

        // Test BooleanInputChange event
        let event = StateMachineEvent::BooleanInputChange {
            name: CString::new("enabled").unwrap(),
            old_value: false,
            new_value: true,
        };
        if let StateMachineEvent::BooleanInputChange {
            name,
            old_value,
            new_value,
        } = &event
        {
            assert_eq!(name.to_str().unwrap(), "enabled");
            assert!(!old_value);
            assert!(new_value);
        } else {
            panic!("Expected BooleanInputChange variant");
        }
    }

    #[test]
    fn test_state_machine_event_empty_strings() {
        // Test with empty strings (should produce valid empty CStrings)
        let event = StateMachineEvent::Transition {
            previous_state: CString::new("").unwrap(),
            new_state: CString::new("").unwrap(),
        };
        if let StateMachineEvent::Transition {
            previous_state,
            new_state,
        } = &event
        {
            assert_eq!(previous_state.to_str().unwrap(), "");
            assert_eq!(new_state.to_str().unwrap(), "");
            // Verify CStrings have valid content (empty string is "\0")
            assert_eq!(previous_state.as_bytes_with_nul().len(), 1);
            assert_eq!(new_state.as_bytes_with_nul().len(), 1);
        } else {
            panic!("Expected Transition variant");
        }
    }

    #[test]
    fn test_state_machine_internal_event() {
        let event = StateMachineInternalEvent::Message {
            message: CString::new("test message").unwrap(),
        };
        let StateMachineInternalEvent::Message { message } = &event;
        assert_eq!(message.to_str().unwrap(), "test message");
    }

    #[test]
    fn test_cstring_pointer_stability() {
        // Verify that pointers remain valid while event is alive
        let event = StateMachineEvent::Error {
            message: CString::new("an error occurred").unwrap(),
        };

        if let StateMachineEvent::Error { message } = &event {
            let ptr = message.as_ptr();
            // Read through pointer to verify it's valid
            let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
            assert_eq!(cstr.to_str().unwrap(), "an error occurred");
        } else {
            panic!("Expected Error variant");
        }
    }
}
