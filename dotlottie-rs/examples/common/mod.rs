/// Tracks wall-clock deltas between consecutive calls to `dt()`.
///
/// Replaces the repetitive `last_tick` / `Instant::now()` boilerplate
/// that every example needs to feed `player.tick(dt)`.
pub struct Clock {
    last: std::time::Instant,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            last: std::time::Instant::now(),
        }
    }

    /// Returns milliseconds elapsed since the previous call (or since construction).
    pub fn dt(&mut self) -> f32 {
        let now = std::time::Instant::now();
        let dt = (now - self.last).as_secs_f32() * 1000.0;
        self.last = now;
        dt
    }
}
