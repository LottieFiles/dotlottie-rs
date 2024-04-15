use std::sync::{Arc, RwLock};

use crate::{event::Event, state::State};

pub trait TransitionTrait {
    fn set_target_state(&mut self, target_state: Arc<RwLock<State>>);
    fn set_event(&mut self, event: Arc<RwLock<Event>>);

    fn get_target_state(&self) -> Arc<RwLock<State>>;
    // fn get_guards(&self) -> &Vec<Box<dyn Event>>;
    fn get_event(&self) -> Arc<RwLock<Event>>;
}

pub enum Transition {
    Transition {
        target_state: Arc<RwLock<State>>,
        event: Arc<RwLock<Event>>,
    },
}

impl Transition {}

impl TransitionTrait for Transition {
    fn set_target_state(&mut self, state: Arc<RwLock<State>>) {
        match self {
            Transition::Transition {
                target_state,
                event: _,
            } => {
                *target_state = state;
            }
        }
    }

    fn get_target_state(&self) -> Arc<RwLock<State>> {
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
