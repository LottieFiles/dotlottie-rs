pub mod guard;
use guard::Guard;
use serde::Deserialize;

pub trait TransitionTrait {
    fn get_target_state(&self) -> &str;
    fn get_guards(&self) -> &Option<Vec<Guard>>;
}

#[derive(Deserialize, Debug, Clone)]
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
}
