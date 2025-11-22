use std::collections::VecDeque;

/// SDL-style events for DotLottie player
#[derive(Debug, Clone, PartialEq)]
pub enum DotLottieEvent {
    // Player lifecycle events
    Load,
    LoadError,

    // Playback control events
    Play,
    Pause,
    Stop,

    // Frame events (can coalesce)
    Frame { frame_no: f32 },
    Render { frame_no: f32 },

    // Loop/complete events
    Loop { loop_count: u32 },
    Complete,
}

/// State machine events
#[derive(Debug, Clone, PartialEq)]
pub enum StateMachineEvent {
    // Lifecycle
    Start,
    Stop,

    // State transitions
    Transition {
        previous_state: String,
        new_state: String
    },
    StateEntered {
        state: String
    },
    StateExit {
        state: String
    },

    // Custom events and errors
    CustomEvent {
        message: String
    },
    Error {
        message: String
    },

    // Input value changes
    StringInputChange {
        name: String,
        old_value: String,
        new_value: String
    },
    NumericInputChange {
        name: String,
        old_value: f32,
        new_value: f32
    },
    BooleanInputChange {
        name: String,
        old_value: bool,
        new_value: bool
    },

    // Event input fired
    InputFired {
        name: String
    },
}

/// Internal state machine events (for framework use)
#[derive(Debug, Clone, PartialEq)]
pub enum StateMachineInternalEvent {
    Message {
        message: String
    },
}

pub const MAX_EVENTS: usize = 256;

/// Trait for events that can be coalesced to save queue space
pub trait CoalescableEvent: Sized {
    /// Returns true if this event can coalesce with (replace) the other event
    fn can_coalesce_with(&self, other: &Self) -> bool;
}

impl CoalescableEvent for DotLottieEvent {
    fn can_coalesce_with(&self, other: &Self) -> bool {
        match (self, other) {
            // Frame events coalesce with other frame events
            (DotLottieEvent::Frame { .. }, DotLottieEvent::Frame { .. }) => true,
            // Render events coalesce with other render events
            (DotLottieEvent::Render { .. }, DotLottieEvent::Render { .. }) => true,
            // All other events don't coalesce
            _ => false,
        }
    }
}

impl CoalescableEvent for StateMachineEvent {
    fn can_coalesce_with(&self, _other: &Self) -> bool {
        // State machine events don't coalesce
        false
    }
}

impl CoalescableEvent for StateMachineInternalEvent {
    fn can_coalesce_with(&self, _other: &Self) -> bool {
        // Internal events don't coalesce
        false
    }
}

/// Event queue with bounded size and coalescing support
///
/// This queue follows SDL's event system design:
/// - Fixed maximum size (256 events)
/// - When full, oldest events are dropped
/// - Consecutive frame/render events coalesce to save space
/// - Single-threaded (no synchronization overhead)
pub struct EventQueue<T: CoalescableEvent> {
    queue: VecDeque<T>,
    max_size: usize,
}

impl<T: CoalescableEvent + Clone> EventQueue<T> {
    /// Create a new event queue with default capacity (256 events)
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(MAX_EVENTS),
            max_size: MAX_EVENTS,
        }
    }

    /// Push an event onto the queue
    ///
    /// If the last event in the queue can coalesce with this event,
    /// it will be replaced instead of adding a new entry.
    /// If the queue is full, the oldest event will be dropped.
    pub fn push(&mut self, event: T) {
        // Try to coalesce with last event
        if let Some(last) = self.queue.back_mut() {
            if last.can_coalesce_with(&event) {
                *last = event;
                return;
            }
        }

        // Queue full - drop oldest event
        if self.queue.len() >= self.max_size {
            self.queue.pop_front();
        }

        self.queue.push_back(event);
    }

    /// Poll for the next event (removes it from the queue)
    ///
    /// Returns Some(event) if an event is available, None if queue is empty.
    /// This follows SDL's SDL_PollEvent pattern.
    pub fn poll(&mut self) -> Option<T> {
        self.queue.pop_front()
    }

    /// Get the number of events currently in the queue
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Clear all events from the queue
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

impl<T: CoalescableEvent + Clone> Default for EventQueue<T> {
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
    fn test_frame_coalescing() {
        let mut queue = EventQueue::<DotLottieEvent>::new();

        queue.push(DotLottieEvent::Frame { frame_no: 10.0 });
        queue.push(DotLottieEvent::Frame { frame_no: 11.0 });
        queue.push(DotLottieEvent::Frame { frame_no: 12.0 });

        // Should only have one frame event (coalesced)
        assert_eq!(queue.len(), 1);

        let event = queue.poll();
        assert_eq!(event, Some(DotLottieEvent::Frame { frame_no: 12.0 }));
    }

    #[test]
    fn test_render_coalescing() {
        let mut queue = EventQueue::<DotLottieEvent>::new();

        queue.push(DotLottieEvent::Render { frame_no: 5.0 });
        queue.push(DotLottieEvent::Render { frame_no: 6.0 });

        // Should only have one render event (coalesced)
        assert_eq!(queue.len(), 1);

        let event = queue.poll();
        assert_eq!(event, Some(DotLottieEvent::Render { frame_no: 6.0 }));
    }

    #[test]
    fn test_no_coalescing_different_events() {
        let mut queue = EventQueue::<DotLottieEvent>::new();

        queue.push(DotLottieEvent::Load);
        queue.push(DotLottieEvent::Play);
        queue.push(DotLottieEvent::Frame { frame_no: 10.0 });

        // All events should be separate
        assert_eq!(queue.len(), 3);
    }

    #[test]
    fn test_queue_overflow() {
        let mut queue = EventQueue::<DotLottieEvent>::new();

        // Fill beyond capacity
        for i in 0..300 {
            queue.push(DotLottieEvent::Loop { loop_count: i });
        }

        // Should not exceed max size
        assert_eq!(queue.len(), MAX_EVENTS);

        // First event should be Loop { loop_count: 44 } (300 - 256 = 44)
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
}
