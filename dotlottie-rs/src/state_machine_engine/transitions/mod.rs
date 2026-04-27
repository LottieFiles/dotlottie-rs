pub mod guard;
use guard::Guard;
use serde::Deserialize;

use crate::string::{DotString, DotStringInterner};

pub trait TransitionTrait {
    fn target_state(&self) -> &DotString;
    fn guards(&self) -> &Option<Vec<Guard>>;
    fn easing(&self) -> [f32; 4];
    fn duration(&self) -> f32;
    fn transitions_contain_event(&self) -> bool;
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "type")]
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
