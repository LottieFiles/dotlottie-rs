use std::collections::VecDeque;

pub const MAX_EVENTS: usize = 256;

/// Bounded queue that drops the oldest event on overflow.
pub struct EventQueue<T> {
    queue: VecDeque<T>,
    max_size: usize,
}

impl<T> EventQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(MAX_EVENTS),
            max_size: MAX_EVENTS,
        }
    }

    pub fn push(&mut self, event: T) {
        if self.queue.len() >= self.max_size {
            self.queue.pop_front();
        }

        self.queue.push_back(event);
    }

    pub fn poll(&mut self) -> Option<T> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

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

    #[derive(Debug, Clone, PartialEq)]
    enum Sample {
        A,
        B(u32),
    }

    #[test]
    fn push_poll_basic() {
        let mut queue = EventQueue::<Sample>::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        queue.push(Sample::A);
        assert_eq!(queue.len(), 1);

        assert_eq!(queue.poll(), Some(Sample::A));
        assert!(queue.is_empty());
    }

    #[test]
    fn fifo_ordering() {
        let mut queue = EventQueue::<Sample>::new();
        queue.push(Sample::B(1));
        queue.push(Sample::B(2));
        queue.push(Sample::B(3));

        assert_eq!(queue.poll(), Some(Sample::B(1)));
        assert_eq!(queue.poll(), Some(Sample::B(2)));
        assert_eq!(queue.poll(), Some(Sample::B(3)));
    }

    #[test]
    fn overflow_drops_oldest() {
        let mut queue = EventQueue::<Sample>::new();
        for i in 0..300 {
            queue.push(Sample::B(i));
        }

        assert_eq!(queue.len(), MAX_EVENTS);
        assert_eq!(queue.poll(), Some(Sample::B(44)));
    }

    #[test]
    fn clear_empties() {
        let mut queue = EventQueue::<Sample>::new();
        queue.push(Sample::A);
        queue.push(Sample::A);
        assert_eq!(queue.len(), 2);

        queue.clear();
        assert!(queue.is_empty());
    }
}
