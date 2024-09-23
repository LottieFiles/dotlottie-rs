pub mod guard;
use self::guard::Guard;

pub trait TransitionTrait {
    fn get_target_state(&self) -> &str;
    fn get_guards(&self) -> &Vec<Guard>;
}

#[derive(Clone, Debug)]
pub enum Transition {
    Transition {
        target_state: String,
        guards: Vec<Guard>,
    },
}

impl TransitionTrait for Transition {
    fn get_target_state(&self) -> &str {
        match self {
            Transition::Transition { target_state, .. } => target_state,
        }
    }

    fn get_guards(&self) -> &Vec<Guard> {
        match self {
            Transition::Transition { guards, .. } => guards,
        }
    }
}
