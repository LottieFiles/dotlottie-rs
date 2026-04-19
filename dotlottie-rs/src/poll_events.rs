use std::collections::VecDeque;

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

}
