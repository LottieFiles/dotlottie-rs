use core::ops::{Add, AddAssign, Sub, SubAssign};
pub use core::time::Duration;

unsafe extern "C" {
    #[cfg(target_os = "emscripten")]
    fn emscripten_get_now() -> f64;
}

pub fn now() -> f64 {
    #[cfg(target_os = "emscripten")]
    {
        unsafe { emscripten_get_now() }
    }

    #[cfg(not(target_os = "emscripten"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("System clock was before 1970.")
            .as_secs_f64()
            * 1000.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub struct Instant(Duration);

impl Instant {
    #[inline]
    pub fn now() -> Self {
        Instant(Duration::from_secs_f64(now() / 1000.0))
    }

    #[inline]
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        if earlier.0 > self.0 {
            panic!("`earlier` cannot be later than `self`.");
        }

        self.0 - earlier.0
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        Self::now().saturating_duration_since(*self)
    }

    #[inline]
    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        self.0.checked_sub(duration).map(Instant)
    }

    #[inline]
    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        if earlier.0 > self.0 {
            Duration::ZERO
        } else {
            self.0 - earlier.0
        }
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Duration) -> Self {
        Instant(self.0 + rhs)
    }
}

impl AddAssign<Duration> for Instant {
    #[inline]
    fn add_assign(&mut self, rhs: Duration) {
        self.0 += rhs
    }
}

impl Sub<Duration> for Instant {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Duration) -> Self {
        Instant(self.0 - rhs)
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    #[inline]
    fn sub(self, rhs: Instant) -> Duration {
        self.duration_since(rhs)
    }
}

impl SubAssign<Duration> for Instant {
    #[inline]
    fn sub_assign(&mut self, rhs: Duration) {
        self.0 -= rhs
    }
}
