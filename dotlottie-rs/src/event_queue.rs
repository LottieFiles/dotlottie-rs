use std::mem::MaybeUninit;

/// Bounded FIFO queue backed by an inline fixed-size array.
///
/// On overflow, the oldest element is dropped to make room for the newest.
pub struct EventQueue<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    head: usize,
    len: usize,
}

impl<T, const N: usize> EventQueue<T, N> {
    pub fn new() -> Self {
        const { assert!(N > 0, "EventQueue capacity N must be > 0") };
        Self {
            buf: [const { MaybeUninit::uninit() }; N],
            head: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, event: T) {
        if self.len == N {
            // SAFETY: len == N means every slot is initialized; head points to a live T.
            unsafe { self.buf[self.head].assume_init_drop() };
            self.head = (self.head + 1) % N;
            self.len -= 1;
        }
        let tail = (self.head + self.len) % N;
        self.buf[tail].write(event);
        self.len += 1;
    }

    pub fn poll(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        // SAFETY: len > 0 guarantees the slot at head is initialized.
        let value = unsafe { self.buf[self.head].assume_init_read() };
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Some(value)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.drop_all();
    }

    fn drop_all(&mut self) {
        while self.len > 0 {
            // SAFETY: len > 0 guarantees the slot at head is initialized.
            unsafe { self.buf[self.head].assume_init_drop() };
            self.head = (self.head + 1) % N;
            self.len -= 1;
        }
        self.head = 0;
    }
}

impl<T, const N: usize> Default for EventQueue<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Drop for EventQueue<T, N> {
    fn drop(&mut self) {
        self.drop_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Debug, Clone, PartialEq)]
    enum Sample {
        A,
        B(u32),
    }

    #[test]
    fn push_poll_basic() {
        let mut queue = EventQueue::<Sample, 8>::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        queue.push(Sample::A);
        assert_eq!(queue.len(), 1);

        assert_eq!(queue.poll(), Some(Sample::A));
        assert!(queue.is_empty());
    }

    #[test]
    fn fifo_ordering() {
        let mut queue = EventQueue::<Sample, 8>::new();
        queue.push(Sample::B(1));
        queue.push(Sample::B(2));
        queue.push(Sample::B(3));

        assert_eq!(queue.poll(), Some(Sample::B(1)));
        assert_eq!(queue.poll(), Some(Sample::B(2)));
        assert_eq!(queue.poll(), Some(Sample::B(3)));
    }

    #[test]
    fn overflow_drops_oldest() {
        const CAP: usize = 16;
        let mut queue = EventQueue::<Sample, CAP>::new();
        for i in 0..300 {
            queue.push(Sample::B(i));
        }

        assert_eq!(queue.len(), CAP);
        assert_eq!(queue.poll(), Some(Sample::B(300 - CAP as u32)));
    }

    #[test]
    fn clear_empties() {
        let mut queue = EventQueue::<Sample, 8>::new();
        queue.push(Sample::A);
        queue.push(Sample::A);
        assert_eq!(queue.len(), 2);

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn custom_capacity() {
        let mut queue = EventQueue::<Sample, 4>::new();
        for i in 0..6 {
            queue.push(Sample::B(i));
        }
        assert_eq!(queue.len(), 4);
        assert_eq!(queue.poll(), Some(Sample::B(2)));
        assert_eq!(queue.poll(), Some(Sample::B(3)));
        assert_eq!(queue.poll(), Some(Sample::B(4)));
        assert_eq!(queue.poll(), Some(Sample::B(5)));
        assert!(queue.is_empty());
    }

    #[test]
    fn drop_runs_destructors_on_clear() {
        let canary = Arc::new(());
        let mut queue = EventQueue::<Arc<()>, 8>::new();
        for _ in 0..5 {
            queue.push(canary.clone());
        }
        assert_eq!(Arc::strong_count(&canary), 6);

        queue.clear();
        assert_eq!(Arc::strong_count(&canary), 1);
    }

    #[test]
    fn drop_runs_destructors_on_drop() {
        let canary = Arc::new(());
        {
            let mut queue = EventQueue::<Arc<()>, 8>::new();
            for _ in 0..5 {
                queue.push(canary.clone());
            }
            assert_eq!(Arc::strong_count(&canary), 6);
        }
        assert_eq!(Arc::strong_count(&canary), 1);
    }

    #[test]
    fn drop_runs_destructors_on_overflow() {
        let canary = Arc::new(());
        let mut queue = EventQueue::<Arc<()>, 4>::new();
        for _ in 0..10 {
            queue.push(canary.clone());
        }
        assert_eq!(queue.len(), 4);
        assert_eq!(Arc::strong_count(&canary), 5);
    }

    #[test]
    fn poll_all_after_overflow_roundtrip() {
        let mut queue = EventQueue::<Sample, 64>::new();
        for i in 0..200u32 {
            queue.push(Sample::B(i));
        }
        assert_eq!(queue.len(), 64);

        for expected in 136u32..200 {
            assert_eq!(queue.poll(), Some(Sample::B(expected)));
        }
        assert!(queue.is_empty());
        assert_eq!(queue.poll(), None);
    }
}
