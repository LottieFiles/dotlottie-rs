use crate::player::Error as PlayerError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TweenStatus {
    /// The tween is still in progress.
    Tweening,
    /// The tween has completed.
    Completed,
}

/// Elapsed/duration/easing bookkeeping shared by any value tweened over wall-clock time
/// (frame-position tweening here, theme-slot tweening in `renderer::mod`).
pub(crate) struct EasingTimer {
    elapsed: f32,
    duration: f32,
    easing: [f32; 4],
}

impl EasingTimer {
    /// Returns `None` if `duration <= 0` or the easing control points are out of range
    /// (`x1`/`x2` must be in `[0,1]`; `y1`/`y2` just need to be finite — they may fall
    /// outside `[0,1]` for overshoot/bounce curves).
    pub(crate) fn new(duration: f32, easing: [f32; 4]) -> Option<Self> {
        if duration <= 0.0 {
            return None;
        }

        let [x1, y1, x2, y2] = easing;
        if !(0.0..=1.0).contains(&x1)
            || !(0.0..=1.0).contains(&x2)
            || !y1.is_finite()
            || !y2.is_finite()
        {
            return None;
        }

        Some(Self {
            elapsed: 0.0,
            duration,
            easing,
        })
    }

    /// Advance the timer by `dt` milliseconds and compute eased progress.
    /// Returns `(status, progress)` where progress is in [0.0, 1.0]
    /// (or beyond if the easing curve overshoots).
    pub(crate) fn advance(&mut self, dt: f32) -> (TweenStatus, f32) {
        self.elapsed += dt;
        let t = self.elapsed / self.duration;

        if t >= 1.0 {
            (TweenStatus::Completed, 1.0)
        } else {
            let [x1, y1, x2, y2] = self.easing;
            let progress = bezier::cubic_bezier(t, x1, y1, x2, y2);
            (TweenStatus::Tweening, progress)
        }
    }
}

pub(crate) struct TweenState {
    /// Frame the tween started from. ThorVG owns the rendered pose; this only exists so
    /// the player can report an interpolated `current_frame` while tweening.
    pub from: f32,
    pub to: f32,
    timer: EasingTimer,
}

impl TweenState {
    pub fn new(from: f32, to: f32, duration: f32, easing: [f32; 4]) -> Result<Self, PlayerError> {
        let timer = EasingTimer::new(duration, easing).ok_or(PlayerError::InvalidParameter)?;
        Ok(Self { from, to, timer })
    }

    /// Advance the tween by `dt` milliseconds and compute eased progress.
    /// Returns `(status, progress)` where progress is in [0.0, 1.0]
    /// (or beyond if the easing curve overshoots).
    pub fn update(&mut self, dt: f32) -> (TweenStatus, f32) {
        self.timer.advance(dt)
    }
}

mod bezier {
    /// Computes the x-coordinate of the cubic Bézier for parameter `u`.
    /// P0 = 0, P1 = (x1, _), P2 = (x2, _), P3 = 1.
    #[inline]
    fn sample_curve_x(u: f32, x1: f32, x2: f32) -> f32 {
        let inv_u = 1.0 - u;
        3.0 * inv_u * inv_u * u * x1 + 3.0 * inv_u * u * u * x2 + u * u * u
    }

    /// Computes the y-coordinate of the cubic Bézier for parameter `u`.
    /// P0 = 0, P1 = (_, y1), P2 = (_, y2), P3 = 1.
    #[inline]
    fn sample_curve_y(u: f32, y1: f32, y2: f32) -> f32 {
        let inv_u = 1.0 - u;
        3.0 * inv_u * inv_u * u * y1 + 3.0 * inv_u * u * u * y2 + u * u * u
    }

    /// Computes the derivative dx/du for a given u.
    #[inline]
    fn sample_curve_derivative_x(u: f32, x1: f32, x2: f32) -> f32 {
        let inv_u = 1.0 - u;
        3.0 * inv_u * inv_u * x1 + 6.0 * inv_u * u * (x2 - x1) + 3.0 * u * u * (1.0 - x2)
    }

    /// Uses binary subdivision to find a parameter u such that sample_curve_x(u) ≈ t.
    #[inline]
    fn binary_subdivide(t: f32, x1: f32, x2: f32) -> f32 {
        let mut a = 0.0;
        let mut b = 1.0;
        let mut u = t;
        for _ in 0..10 {
            let x = sample_curve_x(u, x1, x2);
            if (x - t).abs() < 1e-6 {
                return u;
            }
            if x > t {
                b = u;
            } else {
                a = u;
            }
            u = (a + b) * 0.5;
        }
        u
    }

    /// Given a linear progress t in [0,1], uses a cubic Bézier easing function to compute
    /// an eased progress value. Output can exceed [0,1] when y-values are outside that range
    /// (e.g., overshoot/bounce easing curves).
    ///
    /// The cubic Bézier is defined by:
    ///   P0 = (0, 0)
    ///   P1 = (x1, y1)
    ///   P2 = (x2, y2)
    ///   P3 = (1, 1)
    pub(crate) fn cubic_bezier(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }

        // First try Newton–Raphson iteration.
        let mut u = t;
        for _ in 0..8 {
            let x = sample_curve_x(u, x1, x2);
            let dx = sample_curve_derivative_x(u, x1, x2);
            if dx.abs() < 1e-6 {
                break;
            }
            let delta = (x - t) / dx;
            u -= delta;
            if delta.abs() < 1e-6 {
                break;
            }
        }

        // Fallback to binary subdivision if necessary.
        if !(0.0..=1.0).contains(&u) {
            u = binary_subdivide(t, x1, x2);
        }
        u = u.clamp(0.0, 1.0);
        sample_curve_y(u, y1, y2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINEAR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    #[test]
    fn easing_timer_rejects_non_positive_duration() {
        assert!(EasingTimer::new(0.0, LINEAR).is_none());
        assert!(EasingTimer::new(-1.0, LINEAR).is_none());
    }

    #[test]
    fn easing_timer_rejects_out_of_range_x_control_points() {
        assert!(EasingTimer::new(100.0, [-0.1, 0.0, 0.5, 1.0]).is_none());
        assert!(EasingTimer::new(100.0, [0.0, 0.0, 1.5, 1.0]).is_none());
    }

    #[test]
    fn easing_timer_accepts_y_control_points_outside_unit_range() {
        assert!(EasingTimer::new(100.0, [0.3, 1.5, 0.6, -0.5]).is_some());
    }

    #[test]
    fn easing_timer_advance_reaches_completed_at_duration() {
        let mut timer = EasingTimer::new(100.0, LINEAR).unwrap();
        let (status, progress) = timer.advance(50.0);
        assert_eq!(status, TweenStatus::Tweening);
        assert!((progress - 0.5).abs() < 1e-6);

        let (status, progress) = timer.advance(50.0);
        assert_eq!(status, TweenStatus::Completed);
        assert_eq!(progress, 1.0);
    }

    #[test]
    fn easing_timer_advance_completes_when_overshooting_duration() {
        let mut timer = EasingTimer::new(100.0, LINEAR).unwrap();
        let (status, progress) = timer.advance(1000.0);
        assert_eq!(status, TweenStatus::Completed);
        assert_eq!(progress, 1.0);
    }

    #[test]
    fn tween_state_new_still_validates_the_same_way() {
        assert!(TweenState::new(0.0, 10.0, 0.0, LINEAR).is_err());
        assert!(TweenState::new(0.0, 10.0, 100.0, [-0.1, 0.0, 0.5, 1.0]).is_err());
        assert!(TweenState::new(0.0, 10.0, 100.0, LINEAR).is_ok());
    }
}
