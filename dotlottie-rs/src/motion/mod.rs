//! Imperative motion driver: spring/tween tracks over node properties, ticked by the
//! player clock. Tracks write absolute override values into the renderer's props store;
//! interrupting a live track hands its current value and velocity to the replacement.

mod spring;

pub use spring::{Spring, SpringParams, SpringSample};

use crate::tween::bezier;

pub const EASE_LINEAR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
pub const EASE: [f32; 4] = [0.25, 0.1, 0.25, 1.0];
pub const EASE_IN: [f32; 4] = [0.42, 0.0, 1.0, 1.0];
pub const EASE_OUT: [f32; 4] = [0.0, 0.0, 0.58, 1.0];
pub const EASE_IN_OUT: [f32; 4] = [0.42, 0.0, 0.58, 1.0];

/// Scalar node property a track can drive. Uniform `scale` is lowered to X+Y tracks at
/// `animate` time so interruption stays keyed per scalar. The dotted-key props
/// (`Spot*`, `Clip*`, `TintIntensity`) form the curated animatable allowlist over the
/// grouped `NodeProps` fields; `Value` is a raw driver-clocked number with no node sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Prop {
    X,
    Y,
    Rotate,
    ScaleX,
    ScaleY,
    Opacity,
    Blur,
    TintIntensity,
    SpotCx,
    SpotCy,
    SpotRadius,
    SpotFeather,
    ClipCx,
    ClipCy,
    ClipRadius,
    Value,
}

impl Prop {
    /// Override value meaning "authored animation unchanged".
    pub fn identity(self) -> f32 {
        match self {
            Prop::ScaleX | Prop::ScaleY | Prop::Opacity => 1.0,
            Prop::SpotFeather => 0.4,
            _ => 0.0,
        }
    }

    /// Canonical dotted-key catalog shared by every serialized surface (JS
    /// keyframes objects, state machine actions). Uniform `scale` is caller
    /// sugar lowered to X+Y and deliberately absent here, as is `Value`.
    pub const KEYS: &'static [(&'static str, Prop)] = &[
        ("x", Prop::X),
        ("y", Prop::Y),
        ("rotate", Prop::Rotate),
        ("scaleX", Prop::ScaleX),
        ("scaleY", Prop::ScaleY),
        ("opacity", Prop::Opacity),
        ("blur", Prop::Blur),
        ("tint.intensity", Prop::TintIntensity),
        ("spot.cx", Prop::SpotCx),
        ("spot.cy", Prop::SpotCy),
        ("spot.r", Prop::SpotRadius),
        ("spot.feather", Prop::SpotFeather),
        ("clip.cx", Prop::ClipCx),
        ("clip.cy", Prop::ClipCy),
        ("clip.r", Prop::ClipRadius),
    ];

    pub fn from_key(key: &str) -> Option<Prop> {
        Self::KEYS
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, prop)| *prop)
    }
}

/// Named easing preset → cubic-bezier; unknown names fall back to `EASE`.
pub fn ease_preset(name: &str) -> [f32; 4] {
    match name {
        "linear" => EASE_LINEAR,
        "easeIn" => EASE_IN,
        "easeOut" => EASE_OUT,
        "easeInOut" => EASE_IN_OUT,
        _ => EASE,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Transition {
    Tween { duration: f32, easing: [f32; 4] },
    Spring(SpringParams),
}

impl Default for Transition {
    fn default() -> Self {
        Transition::Tween {
            duration: 0.3,
            easing: EASE,
        }
    }
}

/// One animate() entry: a property and its destination waypoints.
#[derive(Debug, Clone)]
pub struct PropKeyframes {
    pub prop: Prop,
    /// Waypoint values. A single value animates from the current override; multiple
    /// values traverse each in turn (first value is an explicit start).
    pub values: Vec<f32>,
    /// Normalized [0,1] offsets per value; must match `values` length when present.
    pub times: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Default)]
pub struct AnimateOptions {
    pub transition: Transition,
    /// Seconds before the track starts sampling.
    pub delay: f32,
}

enum Solver {
    Spring(Spring),
    Tween {
        values: Vec<f32>,
        times: Vec<f32>,
        duration: f32,
        easing: [f32; 4],
        elapsed: f32,
    },
}

struct Track {
    group: u64,
    target: String,
    prop: Prop,
    solver: Solver,
    delay: f32,
    paused: bool,
    speed: f32,
    last_value: f32,
    last_velocity: f32,
}

impl Track {
    /// Advance by `dt` seconds. Returns `(value, done)`; `None` while delayed.
    fn step(&mut self, dt: f32) -> Option<(f32, bool)> {
        let dt = dt * self.speed;
        if self.delay > 0.0 {
            self.delay -= dt;
            if self.delay > 0.0 {
                return None;
            }
        }
        match &mut self.solver {
            Solver::Spring(spring) => {
                let sample = spring.update(dt);
                self.last_value = sample.value;
                self.last_velocity = sample.velocity;
                Some((sample.value, sample.done))
            }
            Solver::Tween {
                values,
                times,
                duration,
                easing,
                elapsed,
            } => {
                *elapsed += dt;
                let t = (*elapsed / *duration).clamp(0.0, 1.0);
                let [x1, y1, x2, y2] = *easing;
                let eased = bezier::cubic_bezier(t, x1, y1, x2, y2);
                let value = sample_keyframes(values, times, eased);
                self.last_velocity = if dt > 0.0 {
                    (value - self.last_value) / dt
                } else {
                    self.last_velocity
                };
                self.last_value = value;
                Some((value, t >= 1.0))
            }
        }
    }
}

fn sample_keyframes(values: &[f32], times: &[f32], p: f32) -> f32 {
    debug_assert_eq!(values.len(), times.len());
    if values.len() == 1 {
        return values[0];
    }
    if p <= times[0] {
        return values[0];
    }
    for w in times.windows(2).zip(values.windows(2)) {
        let ([t0, t1], [v0, v1]) = (w.0, w.1) else {
            continue;
        };
        if p <= *t1 {
            let span = (t1 - t0).max(1e-6);
            let local = (p - t0) / span;
            return v0 + (v1 - v0) * local;
        }
    }
    *values.last().unwrap()
}

/// A value written into (or cleared from) the props store this tick.
pub enum Write {
    Set(Prop, f32),
    Clear(Prop),
}

#[derive(Default)]
pub struct Driver {
    tracks: Vec<Track>,
    next_group: u64,
    finished_groups: Vec<u64>,
}

impl Driver {
    /// Start an animation group. `current` resolves the present override value per prop
    /// (used as the implicit start when no live track exists for it).
    pub fn animate(
        &mut self,
        target: &str,
        entries: Vec<PropKeyframes>,
        options: AnimateOptions,
        mut current: impl FnMut(Prop) -> Option<f32>,
    ) -> u64 {
        self.next_group += 1;
        let group = self.next_group;

        for entry in entries {
            if entry.values.is_empty() {
                continue;
            }
            let (start, velocity) = self.interrupt(target, entry.prop).unwrap_or_else(|| {
                (
                    current(entry.prop).unwrap_or_else(|| entry.prop.identity()),
                    0.0,
                )
            });

            let solver = match &options.transition {
                Transition::Spring(params) => {
                    // Like tweens, a multi-value entry's first value is an explicit
                    // start; springs ignore intermediate waypoints.
                    let from = if entry.values.len() > 1 {
                        entry.values[0]
                    } else {
                        start
                    };
                    let to = *entry.values.last().unwrap();
                    Solver::Spring(Spring::new(from, to, velocity, *params))
                }
                Transition::Tween { duration, easing } => {
                    let mut values = entry.values.clone();
                    let explicit_start = values.len() > 1;
                    if !explicit_start {
                        values.insert(0, start);
                    }
                    let times = match &entry.times {
                        Some(times) if times.len() == values.len() => times.clone(),
                        _ => even_times(values.len()),
                    };
                    Solver::Tween {
                        values,
                        times,
                        duration: duration.max(1e-3),
                        easing: *easing,
                        elapsed: 0.0,
                    }
                }
            };

            self.tracks.push(Track {
                group,
                target: target.to_owned(),
                prop: entry.prop,
                solver,
                delay: options.delay.max(0.0),
                paused: false,
                speed: 1.0,
                last_value: start,
                last_velocity: velocity,
            });
        }

        group
    }

    /// Remove the live track for `(target, prop)`, returning its current value and
    /// velocity for handoff.
    pub fn interrupt(&mut self, target: &str, prop: Prop) -> Option<(f32, f32)> {
        let idx = self
            .tracks
            .iter()
            .position(|t| t.prop == prop && t.target == target)?;
        let track = self.tracks.swap_remove(idx);
        Some((track.last_value, track.last_velocity))
    }

    /// Remove every track for `target` (reset/removal of the node).
    pub fn interrupt_target(&mut self, target: &str) {
        self.tracks.retain(|t| t.target != target);
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.finished_groups.clear();
    }

    pub fn pause(&mut self, group: u64) {
        self.for_group(group, |t| t.paused = true);
    }

    pub fn resume(&mut self, group: u64) {
        self.for_group(group, |t| t.paused = false);
    }

    pub fn set_speed(&mut self, group: u64, speed: f32) {
        let speed = speed.max(0.0);
        self.for_group(group, |t| t.speed = speed);
    }

    /// Freeze at the current value: tracks vanish, overrides persist.
    pub fn stop(&mut self, group: u64) {
        self.tracks.retain(|t| t.group != group);
    }

    /// Remove the group and clear its overrides back to the authored animation.
    pub fn cancel(&mut self, group: u64, mut sink: impl FnMut(&str, Write)) {
        let mut i = 0;
        while i < self.tracks.len() {
            if self.tracks[i].group == group {
                let track = self.tracks.swap_remove(i);
                sink(&track.target, Write::Clear(track.prop));
            } else {
                i += 1;
            }
        }
    }

    fn for_group(&mut self, group: u64, f: impl Fn(&mut Track)) {
        for track in self.tracks.iter_mut().filter(|t| t.group == group) {
            f(track);
        }
    }

    pub fn is_active(&self) -> bool {
        !self.tracks.is_empty()
    }

    /// Advance all tracks by `dt` seconds, emitting writes. Fully finished groups are
    /// queued for `drain_finished`.
    pub fn tick(&mut self, dt: f32, mut sink: impl FnMut(&str, Write)) {
        let mut i = 0;
        while i < self.tracks.len() {
            let track = &mut self.tracks[i];
            if track.paused {
                i += 1;
                continue;
            }
            match track.step(dt) {
                None => i += 1,
                Some((value, done)) => {
                    sink(&track.target, Write::Set(track.prop, value));
                    if done {
                        let group = track.group;
                        self.tracks.swap_remove(i);
                        if !self.tracks.iter().any(|t| t.group == group) {
                            self.finished_groups.push(group);
                        }
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }

    pub fn drain_finished(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.finished_groups)
    }
}

fn even_times(len: usize) -> Vec<f32> {
    if len <= 1 {
        return vec![0.0];
    }
    (0..len).map(|i| i as f32 / (len - 1) as f32).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_current(_: Prop) -> Option<f32> {
        None
    }

    fn collect(driver: &mut Driver, dt: f32) -> Vec<(String, Prop, f32)> {
        let mut out = Vec::new();
        driver.tick(dt, |target, write| {
            if let Write::Set(prop, value) = write {
                out.push((target.to_owned(), prop, value));
            }
        });
        out
    }

    #[test]
    fn tween_track_reaches_target_and_finishes_group() {
        let mut driver = Driver::default();
        let group = driver.animate(
            "arm",
            vec![PropKeyframes {
                prop: Prop::Rotate,
                values: vec![90.0],
                times: None,
            }],
            AnimateOptions {
                transition: Transition::Tween {
                    duration: 0.5,
                    easing: EASE_LINEAR,
                },
                delay: 0.0,
            },
            no_current,
        );

        let mut last = 0.0;
        for _ in 0..40 {
            for (_, prop, value) in collect(&mut driver, 1.0 / 60.0) {
                assert_eq!(prop, Prop::Rotate);
                last = value;
            }
        }
        assert_eq!(last, 90.0);
        assert!(!driver.is_active());
        assert_eq!(driver.drain_finished(), vec![group]);
    }

    #[test]
    fn keyframe_array_hits_waypoints() {
        let mut driver = Driver::default();
        driver.animate(
            "badge",
            vec![PropKeyframes {
                prop: Prop::ScaleX,
                values: vec![1.0, 1.3, 1.0],
                times: None,
            }],
            AnimateOptions {
                transition: Transition::Tween {
                    duration: 1.0,
                    easing: EASE_LINEAR,
                },
                delay: 0.0,
            },
            no_current,
        );

        let mut max: f32 = 0.0;
        for _ in 0..70 {
            for (_, _, value) in collect(&mut driver, 1.0 / 60.0) {
                max = max.max(value);
            }
        }
        assert!((max - 1.3).abs() < 0.02, "peak {max}");
    }

    #[test]
    fn spring_honors_explicit_start_value() {
        let mut driver = Driver::default();
        driver.animate(
            "n",
            vec![PropKeyframes {
                prop: Prop::X,
                values: vec![50.0, 100.0],
                times: None,
            }],
            AnimateOptions {
                transition: Transition::Spring(SpringParams::default()),
                delay: 0.0,
            },
            no_current,
        );
        let first = collect(&mut driver, 1.0 / 240.0);
        let (_, _, value) = first[0];
        assert!(
            (50.0..70.0).contains(&value),
            "spring should start near the explicit 50, got {value}"
        );
    }

    #[test]
    fn uniform_scale_lowered_by_caller_interrupts_both_axes() {
        let mut driver = Driver::default();
        driver.animate(
            "n",
            vec![
                PropKeyframes {
                    prop: Prop::ScaleX,
                    values: vec![2.0],
                    times: None,
                },
                PropKeyframes {
                    prop: Prop::ScaleY,
                    values: vec![2.0],
                    times: None,
                },
            ],
            AnimateOptions::default(),
            no_current,
        );
        assert!(driver.interrupt("n", Prop::ScaleX).is_some());
        assert!(driver.interrupt("n", Prop::ScaleY).is_some());
        assert!(!driver.is_active());
    }

    #[test]
    fn spring_interrupt_hands_off_velocity() {
        let mut driver = Driver::default();
        driver.animate(
            "arm",
            vec![PropKeyframes {
                prop: Prop::X,
                values: vec![100.0],
                times: None,
            }],
            AnimateOptions {
                transition: Transition::Spring(SpringParams::default()),
                delay: 0.0,
            },
            no_current,
        );
        for _ in 0..20 {
            collect(&mut driver, 1.0 / 60.0);
        }

        // Redirect: the new track must start from the live value, not identity.
        let group2 = driver.animate(
            "arm",
            vec![PropKeyframes {
                prop: Prop::X,
                values: vec![-50.0],
                times: None,
            }],
            AnimateOptions {
                transition: Transition::Spring(SpringParams::default()),
                delay: 0.0,
            },
            no_current,
        );
        let writes = collect(&mut driver, 1.0 / 240.0);
        assert_eq!(writes.len(), 1);
        let value = writes[0].2;
        assert!(
            value > 10.0,
            "expected continuation from mid-flight value, got {value}"
        );
        driver.stop(group2);
        assert!(!driver.is_active());
    }

    #[test]
    fn delay_defers_first_write() {
        let mut driver = Driver::default();
        driver.animate(
            "n",
            vec![PropKeyframes {
                prop: Prop::Y,
                values: vec![10.0],
                times: None,
            }],
            AnimateOptions {
                transition: Transition::Tween {
                    duration: 0.1,
                    easing: EASE_LINEAR,
                },
                delay: 0.2,
            },
            no_current,
        );
        assert!(collect(&mut driver, 0.1).is_empty());
        assert!(collect(&mut driver, 0.05).is_empty());
        assert!(!collect(&mut driver, 0.1).is_empty());
    }

    #[test]
    fn cancel_emits_clears() {
        let mut driver = Driver::default();
        let group = driver.animate(
            "n",
            vec![
                PropKeyframes {
                    prop: Prop::X,
                    values: vec![5.0],
                    times: None,
                },
                PropKeyframes {
                    prop: Prop::Opacity,
                    values: vec![0.5],
                    times: None,
                },
            ],
            AnimateOptions::default(),
            no_current,
        );
        let mut cleared = Vec::new();
        driver.cancel(group, |target, write| {
            if let Write::Clear(prop) = write {
                cleared.push((target.to_owned(), prop));
            }
        });
        assert_eq!(cleared.len(), 2);
        assert!(!driver.is_active());
    }

    #[test]
    fn pause_and_resume() {
        let mut driver = Driver::default();
        let group = driver.animate(
            "n",
            vec![PropKeyframes {
                prop: Prop::X,
                values: vec![10.0],
                times: None,
            }],
            AnimateOptions {
                transition: Transition::Tween {
                    duration: 0.1,
                    easing: EASE_LINEAR,
                },
                delay: 0.0,
            },
            no_current,
        );
        driver.pause(group);
        assert!(collect(&mut driver, 0.5).is_empty());
        assert!(driver.is_active());
        driver.resume(group);
        assert!(!collect(&mut driver, 0.5).is_empty());
        assert!(!driver.is_active());
    }
}
