use std::sync::{Arc, RwLock};

use crate::state_machine::events::Event;

pub trait TransitionTrait {
    fn set_target_state(&mut self, target_state: u32);
    fn set_event(&mut self, event: Arc<RwLock<Event>>);

    fn get_target_state(&self) -> u32;
    // fn get_guards(&self) -> &Vec<Box<dyn Event>>;
    fn get_event(&self) -> Arc<RwLock<Event>>;
}

#[derive(Debug)]
pub enum Transition {
    Transition {
        target_state: u32,
        event: Arc<RwLock<Event>>,
    },
}

impl Transition {}

impl TransitionTrait for Transition {
    fn set_target_state(&mut self, state: u32) {
        match self {
            Transition::Transition {
                target_state,
                event: _,
            } => {
                *target_state = state;
            }
        }
    }

    fn get_target_state(&self) -> u32 {
        match self {
            Transition::Transition {
                target_state,
                event: _,
            } => target_state.clone(),
        }
    }

    fn set_event(&mut self, ev: Arc<RwLock<Event>>) {
        match self {
            Transition::Transition {
                target_state: _,
                event,
            } => {
                *event = ev;
            }
        }
    }

    fn get_event(&self) -> Arc<RwLock<Event>> {
        match self {
            Transition::Transition {
                target_state: _,
                event,
            } => event.clone(),
        }
    }
}
