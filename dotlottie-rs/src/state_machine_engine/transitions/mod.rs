pub mod guard;
use guard::Guard;
use serde::Deserialize;

pub trait TransitionTrait {
    fn get_target_state(&self) -> &str;
    fn get_guards(&self) -> &Option<Vec<Guard>>;
    fn transitions_contain_event(&self) -> bool;
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "type")]
pub enum Transition {
    Transition {
        to_state: String,
        guards: Option<Vec<Guard>>,
    },
}

impl TransitionTrait for Transition {
    fn get_target_state(&self) -> &str {
        match self {
            Transition::Transition { to_state, .. } => to_state,
        }
    }

    fn get_guards(&self) -> &Option<Vec<Guard>> {
        match self {
            Transition::Transition { guards, .. } => guards,
        }
    }

    fn transitions_contain_event(&self) -> bool {
        if let Some(guards) = self.get_guards() {
            for guard in guards {
                if let Guard::Event { .. } = guard {
                    return true;
                }
            }
        }

        false
    }
}
