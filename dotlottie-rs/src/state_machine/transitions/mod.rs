pub mod guard;
use guard::Guard;

use crate::json::{array_of, f32_array, opt, Value};
use crate::state_machine::definition::dot_string;
use crate::string::{DotString, DotStringInterner};

pub trait TransitionTrait {
    fn target_state(&self) -> &DotString;
    fn guards(&self) -> &Option<Vec<Guard>>;
    fn easing(&self) -> [f32; 4];
    fn duration(&self) -> f32;
    fn transitions_contain_event(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum Transition {
    Transition {
        to_state: DotString,
        guards: Option<Vec<Guard>>,
    },
    Tweened {
        to_state: DotString,
        guards: Option<Vec<Guard>>,
        duration: f32,
        easing: [f32; 4],
    },
}

pub(crate) fn transition_from_json(v: &Value) -> Option<Transition> {
    let to_state = dot_string(v.get("toState")?)?;
    let guards = opt(v.get("guards"), |g| array_of(g, guard::guard_from_json))?;
    Some(match v.str_field("type")? {
        "Transition" => Transition::Transition { to_state, guards },
        "Tweened" => Transition::Tweened {
            to_state,
            guards,
            duration: v.f32_field("duration")?,
            easing: f32_array(v.get("easing")?)?,
        },
        _ => return None,
    })
}

impl Transition {
    pub(crate) fn intern_identifiers(&mut self, interner: &mut DotStringInterner) {
        let (to_state, guards) = match self {
            Transition::Transition { to_state, guards } => (to_state, guards),
            Transition::Tweened {
                to_state, guards, ..
            } => (to_state, guards),
        };
        *to_state = interner.intern(to_state.as_str());
        if let Some(guards) = guards {
            for g in guards {
                g.intern_identifiers(interner);
            }
        }
    }
}

impl TransitionTrait for Transition {
    fn target_state(&self) -> &DotString {
        match self {
            Transition::Transition { to_state, .. } => to_state,
            Transition::Tweened { to_state, .. } => to_state,
        }
    }

    fn guards(&self) -> &Option<Vec<Guard>> {
        match self {
            Transition::Transition { guards, .. } => guards,
            Transition::Tweened { guards, .. } => guards,
        }
    }

    fn easing(&self) -> [f32; 4] {
        match self {
            Transition::Transition { .. } => [0.0, 0.0, 0.0, 0.0],
            Transition::Tweened { easing, .. } => *easing,
        }
    }

    fn duration(&self) -> f32 {
        match self {
            Transition::Transition { .. } => 0.0,
            Transition::Tweened { duration, .. } => *duration * 1000.0,
        }
    }

    fn transitions_contain_event(&self) -> bool {
        if let Some(guards) = self.guards() {
            for guard in guards {
                if let Guard::Event { .. } = guard {
                    return true;
                }
            }
        }

        false
    }
}
