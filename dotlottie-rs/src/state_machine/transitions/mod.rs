pub mod guard;

use std::sync::{Arc, RwLock};

use crate::state_machine::events::InternalEvent;

use self::guard::Guard;

pub trait TransitionTrait {
    fn set_target_state(&mut self, target_state: u32);
    fn set_event(&mut self, event: Arc<RwLock<InternalEvent>>);

    fn get_target_state(&self) -> u32;
    fn get_guards(&self) -> &Vec<Guard>;
    fn get_event(&self) -> Arc<RwLock<InternalEvent>>;
}

#[derive(Debug)]
pub enum Transition {
    Transition {
        target_state: u32,
        event: Arc<RwLock<InternalEvent>>,
        guards: Vec<Guard>,
    },
}

impl TransitionTrait for Transition {
    fn set_target_state(&mut self, state: u32) {
        match self {
            Transition::Transition { target_state, .. } => {
                *target_state = state;
            }
        }
    }

    fn get_target_state(&self) -> u32 {
        match self {
            Transition::Transition { target_state, .. } => *target_state,
        }
    }

    fn set_event(&mut self, ev: Arc<RwLock<InternalEvent>>) {
        match self {
            Transition::Transition { event, .. } => {
                *event = ev;
            }
        }
    }

    fn get_event(&self) -> Arc<RwLock<InternalEvent>> {
        match self {
            Transition::Transition { event, .. } => event.clone(),
        }
    }

    fn get_guards(&self) -> &Vec<Guard> {
        match self {
            Transition::Transition { guards, .. } => guards,
        }
    }
}
