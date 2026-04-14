use crate::DotLottiePlayerError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TweenStatus {
    /// The tween is still in progress.
    Tweening,
    /// The tween has completed.
    Completed,
}

pub(crate) struct TweenState {
    pub from: f32,
    pub to: f32,
    elapsed: f32,
    duration: f32,
    easing: [f32; 4],
}

impl TweenState {
    pub fn new(
        from: f32,
        to: f32,
        duration: f32,
        easing: [f32; 4],
    ) -> Result<Self, DotLottiePlayerError> {
        if duration <= 0.0 {
            return Err(DotLottiePlayerError::InvalidParameter);
        }

        let [x1, y1, x2, y2] = easing;
        if !(0.0..=1.0).contains(&x1)
            || !(0.0..=1.0).contains(&x2)
            || !y1.is_finite()
            || !y2.is_finite()
        {
            return Err(DotLottiePlayerError::InvalidParameter);
        }

        Ok(Self {
            from,
            to,
            elapsed: 0.0,
            duration,
            easing,
        })
    }

    /// Advance the tween by `dt` milliseconds and compute eased progress.
    /// Returns `(status, progress)` where progress is in [0.0, 1.0]
    /// (or beyond if the easing curve overshoots).
    pub fn update(&mut self, dt: f32) -> (TweenStatus, f32) {
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
    pub(super) fn cubic_bezier(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
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
