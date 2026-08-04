//! Closed-form damped harmonic oscillator. Evaluating x(t)/v(t) analytically keeps the
//! spring deterministic under variable tick rates and makes velocity handoff exact.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringParams {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl Default for SpringParams {
    fn default() -> Self {
        Self {
            stiffness: 100.0,
            damping: 10.0,
            mass: 1.0,
        }
    }
}

impl SpringParams {
    /// Perceptual parametrization: `visual_duration` is roughly the time (seconds) the
    /// spring takes to visually reach the target, `bounce` in [0, 1] trades damping for
    /// overshoot (0 = no bounce, critically damped).
    pub fn from_bounce(visual_duration: f32, bounce: f32) -> Self {
        let duration = visual_duration.max(0.01);
        let bounce = bounce.clamp(0.0, 1.0);
        let omega0 = std::f32::consts::TAU / duration;
        let zeta = (1.0 - bounce).clamp(0.05, 1.0);
        let mass = 1.0;
        let stiffness = omega0 * omega0 * mass;
        let damping = 2.0 * zeta * omega0 * mass;
        Self {
            stiffness,
            damping,
            mass,
        }
    }
}

const REST_DELTA: f32 = 0.005;
const REST_SPEED: f32 = 0.05;

#[derive(Debug, Clone, Copy)]
enum Regime {
    /// zeta < 1: x(t) = e^(-z*w0*t) * (a*cos(wd*t) + b*sin(wd*t))
    Under { wd: f32 },
    /// zeta == 1: x(t) = (a + b*t) * e^(-w0*t)
    Critical,
    /// zeta > 1: x(t) = a*e^(r1*t) + b*e^(r2*t)
    Over { r1: f32, r2: f32 },
}

#[derive(Debug, Clone, Copy)]
pub struct SpringSample {
    pub value: f32,
    pub velocity: f32,
    pub done: bool,
}

#[derive(Debug, Clone)]
pub struct Spring {
    target: f32,
    zeta_omega0: f32,
    regime: Regime,
    a: f32,
    b: f32,
    elapsed: f32,
    rest_scale: f32,
}

impl Spring {
    pub fn new(from: f32, to: f32, velocity: f32, params: SpringParams) -> Self {
        let mass = params.mass.max(1e-3);
        let stiffness = params.stiffness.max(1e-3);
        let damping = params.damping.max(0.0);

        let omega0 = (stiffness / mass).sqrt();
        let zeta = damping / (2.0 * (stiffness * mass).sqrt());
        let x0 = from - to;
        let v0 = velocity;
        let zeta_omega0 = zeta * omega0;

        let (regime, a, b) = if (zeta - 1.0).abs() < 1e-3 {
            (Regime::Critical, x0, v0 + omega0 * x0)
        } else if zeta < 1.0 {
            let wd = omega0 * (1.0 - zeta * zeta).sqrt();
            (Regime::Under { wd }, x0, (v0 + zeta_omega0 * x0) / wd)
        } else {
            let disc = omega0 * (zeta * zeta - 1.0).sqrt();
            let r1 = -zeta_omega0 + disc;
            let r2 = -zeta_omega0 - disc;
            let c1 = (v0 - r2 * x0) / (r1 - r2);
            (Regime::Over { r1, r2 }, c1, x0 - c1)
        };

        // Rest thresholds scale with travel so tiny (opacity) and large (pixel)
        // ranges both settle sensibly.
        let rest_scale = x0.abs().max(1.0);

        Self {
            target: to,
            zeta_omega0,
            regime,
            a,
            b,
            elapsed: 0.0,
            rest_scale,
        }
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    fn displacement_at(&self, t: f32) -> (f32, f32) {
        match self.regime {
            Regime::Under { wd } => {
                let decay = (-self.zeta_omega0 * t).exp();
                let (sin, cos) = (wd * t).sin_cos();
                let x = decay * (self.a * cos + self.b * sin);
                let v = decay
                    * ((self.b * wd - self.a * self.zeta_omega0) * cos
                        - (self.a * wd + self.b * self.zeta_omega0) * sin);
                (x, v)
            }
            Regime::Critical => {
                let w0 = self.zeta_omega0;
                let decay = (-w0 * t).exp();
                let x = (self.a + self.b * t) * decay;
                let v = (self.b - w0 * (self.a + self.b * t)) * decay;
                (x, v)
            }
            Regime::Over { r1, r2 } => {
                let e1 = (r1 * t).exp();
                let e2 = (r2 * t).exp();
                let x = self.a * e1 + self.b * e2;
                let v = self.a * r1 * e1 + self.b * r2 * e2;
                (x, v)
            }
        }
    }

    /// Advance by `dt` seconds.
    pub fn update(&mut self, dt: f32) -> SpringSample {
        self.elapsed += dt.max(0.0);
        let (x, v) = self.displacement_at(self.elapsed);

        let done = x.abs() < REST_DELTA * self.rest_scale && v.abs() < REST_SPEED * self.rest_scale;

        if done {
            SpringSample {
                value: self.target,
                velocity: 0.0,
                done: true,
            }
        } else {
            SpringSample {
                value: self.target + x,
                velocity: v,
                done: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(spring: &mut Spring, seconds: f32, dt: f32) -> SpringSample {
        let mut sample = spring.update(0.0);
        let steps = (seconds / dt) as usize;
        for _ in 0..steps {
            sample = spring.update(dt);
        }
        sample
    }

    #[test]
    fn converges_to_target() {
        let mut s = Spring::new(0.0, 100.0, 0.0, SpringParams::default());
        let sample = run(&mut s, 5.0, 1.0 / 60.0);
        assert!(sample.done);
        assert_eq!(sample.value, 100.0);
    }

    #[test]
    fn critically_damped_never_overshoots() {
        // zeta == 1: damping = 2*sqrt(k*m)
        let params = SpringParams {
            stiffness: 100.0,
            damping: 20.0,
            mass: 1.0,
        };
        let mut s = Spring::new(0.0, 1.0, 0.0, params);
        for _ in 0..600 {
            let sample = s.update(1.0 / 60.0);
            assert!(sample.value <= 1.0 + 1e-4, "overshot: {}", sample.value);
        }
    }

    #[test]
    fn underdamped_overshoots() {
        let params = SpringParams {
            stiffness: 300.0,
            damping: 5.0,
            mass: 1.0,
        };
        let mut s = Spring::new(0.0, 1.0, 0.0, params);
        let mut max = 0.0f32;
        for _ in 0..600 {
            max = max.max(s.update(1.0 / 60.0).value);
        }
        assert!(max > 1.05, "expected overshoot, max {max}");
    }

    #[test]
    fn overdamped_converges_monotonically() {
        let params = SpringParams {
            stiffness: 100.0,
            damping: 40.0,
            mass: 1.0,
        };
        let mut s = Spring::new(0.0, 1.0, 0.0, params);
        let mut prev = 0.0f32;
        for _ in 0..1200 {
            let sample = s.update(1.0 / 60.0);
            assert!(sample.value >= prev - 1e-5);
            prev = sample.value;
            if sample.done {
                break;
            }
        }
        assert_eq!(prev, 1.0);
    }

    #[test]
    fn deterministic_under_variable_dt() {
        let params = SpringParams {
            stiffness: 200.0,
            damping: 12.0,
            mass: 1.0,
        };
        let mut fine = Spring::new(0.0, 50.0, 10.0, params);
        let mut coarse = Spring::new(0.0, 50.0, 10.0, params);
        let mut fine_sample = fine.update(0.0);
        for _ in 0..100 {
            fine_sample = fine.update(0.01);
        }
        let coarse_sample = coarse.update(1.0);
        assert!((fine_sample.value - coarse_sample.value).abs() < 1e-3);
        assert!((fine_sample.velocity - coarse_sample.velocity).abs() < 1e-2);
    }

    #[test]
    fn velocity_handoff_is_continuous() {
        let params = SpringParams::default();
        let mut first = Spring::new(0.0, 100.0, 0.0, params);
        let mut sample = first.update(0.0);
        for _ in 0..30 {
            sample = first.update(1.0 / 60.0);
        }
        // Redirect mid-flight to a new target, seeded with current value+velocity.
        let mut second = Spring::new(sample.value, -50.0, sample.velocity, params);
        let s0 = second.update(0.0);
        assert!((s0.value - sample.value).abs() < 1e-4);
        let s1 = second.update(1.0 / 120.0);
        // The first instants keep moving in the inherited direction.
        assert!((s1.value - s0.value).signum() == sample.velocity.signum());
    }

    #[test]
    fn from_bounce_regimes() {
        let no_bounce = SpringParams::from_bounce(0.5, 0.0);
        let zeta = no_bounce.damping / (2.0 * (no_bounce.stiffness * no_bounce.mass).sqrt());
        assert!((zeta - 1.0).abs() < 1e-3);

        let bouncy = SpringParams::from_bounce(0.5, 0.5);
        let zeta = bouncy.damping / (2.0 * (bouncy.stiffness * bouncy.mass).sqrt());
        assert!((zeta - 0.5).abs() < 1e-3);

        let mut s = Spring::new(0.0, 1.0, 0.0, bouncy);
        let sample = run(&mut s, 5.0, 1.0 / 60.0);
        assert!(sample.done);
    }
}
