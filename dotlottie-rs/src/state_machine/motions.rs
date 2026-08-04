//! Named motions: reusable choreography defined at the state machine root and
//! referenced by playback states (state-scoped) or `PlayMotion` actions
//! (fire-and-forget). Steps chain on completion by default; `at` joins a step to
//! the current launch batch with a delay offset instead.

use crate::json::{array_of, Value};
use crate::motion::AnimateOptions;
use crate::string::{DotString, DotStringInterner};

use super::definition::dot_string;
use super::motion_ops::{
    animate_options_from_json, keyframes_spec_from_json, props_spec_from_json, KeyframesSpec,
    PropsSpec,
};

/// Batch placement for a step. Absent `at` starts a new batch once the previous
/// batch has fully settled (completion-chained — spring-safe).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum At {
    /// Bare number: seconds from the batch start.
    Offset(f32),
    /// `"<"`: same delay as the previous step in the batch.
    WithPrevious,
    /// `"+x"` / `"-x"`: previous step's delay plus x (clamped to 0).
    AfterPrevious(f32),
}

fn at_from_json(v: &Value) -> Option<At> {
    if let Some(n) = v.as_f32() {
        return Some(At::Offset(n.max(0.0)));
    }
    let s = v.as_str()?;
    if s == "<" {
        return Some(At::WithPrevious);
    }
    if let Some(rest) = s.strip_prefix('+') {
        return rest.parse::<f32>().ok().map(At::AfterPrevious);
    }
    if s.starts_with('-') {
        return s.parse::<f32>().ok().map(At::AfterPrevious);
    }
    None
}

#[derive(Debug, Clone)]
pub enum StepKind {
    Animate {
        target: String,
        keyframes: KeyframesSpec,
        options: AnimateOptions,
    },
    SetNode {
        target: String,
        props: Box<PropsSpec>,
    },
}

#[derive(Debug, Clone)]
pub struct MotionStep {
    pub kind: StepKind,
    pub at: Option<At>,
}

fn step_from_json(v: &Value) -> Option<MotionStep> {
    let target = v.str_field("target")?.to_owned();
    let at = match v.get("at") {
        Some(at) => Some(at_from_json(at)?),
        None => None,
    };
    let kind = match v.str_field("type") {
        Some("SetNode") => StepKind::SetNode {
            target,
            props: Box::new(props_spec_from_json(v.get("props")?)?),
        },
        // "Animate" or untyped: an animate step.
        Some("Animate") | None => StepKind::Animate {
            target,
            keyframes: keyframes_spec_from_json(v.get("keyframes")?)?,
            options: animate_options_from_json(v.get("transition"), v.get("delay")),
        },
        _ => return None,
    };
    Some(MotionStep { kind, at })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Repeat {
    Count(u32),
    Infinite,
}

/// Restart semantics when the motion is (re)triggered while already running.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterruptPolicy {
    /// Stop the live instance's tracks (overrides freeze) and start over.
    Restart,
    /// Keep the live instance; the trigger is a no-op.
    Ignore,
}

#[derive(Debug, Clone)]
pub struct Motion {
    pub name: DotString,
    pub steps: Vec<MotionStep>,
    pub repeat: Repeat,
    pub interrupt: InterruptPolicy,
}

pub(crate) fn motion_from_json(v: &Value) -> Option<Motion> {
    let repeat = match v.get("repeat") {
        None => Repeat::Count(1),
        Some(r) => {
            if r.as_str() == Some("infinite") {
                Repeat::Infinite
            } else {
                // "repeat": n means n extra iterations on top of the first,
                // mirroring motion.dev; 0 (or absent) plays once.
                Repeat::Count(r.as_u32()?.saturating_add(1))
            }
        }
    };
    let interrupt = match v.str_field("interrupt") {
        None | Some("restart") => InterruptPolicy::Restart,
        Some("ignore") => InterruptPolicy::Ignore,
        Some(_) => return None,
    };
    let steps: Vec<MotionStep> = array_of(v.get("steps")?, step_from_json)?;
    if steps.is_empty() {
        return None;
    }
    Some(Motion {
        name: dot_string(v.get("name")?)?,
        steps,
        repeat,
        interrupt,
    })
}

impl Motion {
    pub(crate) fn intern_identifiers(&mut self, interner: &mut DotStringInterner) {
        self.name = interner.intern(self.name.as_str());
    }
}

/// A running motion. `live_groups` holds the driver group ids of the current launch
/// batch; the batch is done when every id has completed.
#[derive(Debug)]
pub struct MotionInstance {
    pub(crate) motion_idx: usize,
    /// Started from a state's `motions` list — interrupted when that state exits.
    pub(crate) state_scoped: bool,
    /// Next step index to launch.
    pub(crate) cursor: usize,
    pub(crate) live_groups: Vec<u64>,
    /// Iterations left, including the current one.
    pub(crate) remaining: Repeat,
}
