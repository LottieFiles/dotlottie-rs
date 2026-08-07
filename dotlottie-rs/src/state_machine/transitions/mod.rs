pub mod guard;

use guard::Guard;

use super::definition::dot_string;
use crate::json::{array_of, f32_array, opt, Value};
use crate::string::{DotString, DotStringInterner};

/// The blend to run while settling into a transition's target state.
/// A zero duration is an instant transition.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Tween {
    /// Milliseconds (authored in seconds).
    pub duration: f32,
    pub easing: [f32; 4],
}

impl Tween {
    pub fn is_instant(&self) -> bool {
        self.duration <= 0.0
    }
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub to_state: DotString,
    pub guards: Option<Vec<Guard>>,
    pub tween: Tween,
}

pub(crate) fn transition_from_json(v: &Value) -> Option<Transition> {
    // `"Transition"` is a zero-duration `"Tweened"`.
    let tween = match v.str_field("type")? {
        "Transition" => Tween::default(),
        "Tweened" => Tween {
            duration: v.f32_field("duration")? * 1000.0,
            easing: f32_array(v.get("easing")?)?,
        },
        _ => return None,
    };
    Some(Transition {
        to_state: dot_string(v.get("toState")?)?,
        guards: opt(v.get("guards"), |g| array_of(g, guard::guard_from_json))?,
        tween,
    })
}

impl Transition {
    pub(crate) fn intern_identifiers(&mut self, interner: &mut DotStringInterner) {
        self.to_state = interner.intern(self.to_state.as_str());
        for guard in self.guards.iter_mut().flatten() {
            guard.intern_identifiers(interner);
        }
    }

    pub fn guards(&self) -> &[Guard] {
        self.guards.as_deref().unwrap_or_default()
    }

    pub fn contains_event_guard(&self) -> bool {
        self.guards()
            .iter()
            .any(|guard| matches!(guard, Guard::Event { .. }))
    }
}
