use std::sync::{Arc, RwLock};

use crate::{
    event::Event,
    state::{State, StateType},
};

pub trait TransitionTrait {
    fn set_target_state(&mut self, target_state: Arc<RwLock<StateType>>);
    fn set_event(&mut self, event: Arc<RwLock<Event>>);

    fn get_target_state(&self) -> Arc<RwLock<StateType>>;
    // fn get_guards(&self) -> &Vec<Box<dyn Event>>;
    fn get_event(&self) -> Arc<RwLock<Event>>;
}

pub enum Transition {
    Transition {
        target_state: Arc<RwLock<StateType>>,
        event: Arc<RwLock<Event>>,
    },
}

impl Transition {}

impl TransitionTrait for Transition {
    fn set_target_state(&mut self, state: Arc<RwLock<StateType>>) {
        match self {
            Transition::Transition {
                target_state,
                event: _,
            } => {
                *target_state = state;
            }
        }
    }

    fn get_target_state(&self) -> Arc<RwLock<StateType>> {
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

// pub trait Transition {
//     fn set_target_state(&mut self, target_state: Arc<RwLock<StateType>>);
//     fn set_event(&mut self, event: Arc<RwLock<Event>>);

//     fn get_target_state(&self) -> Arc<RwLock<StateType>>;
//     // fn get_guards(&self) -> &Vec<Box<dyn Event>>;
//     fn get_event(&self) -> &Event;

//     fn as_any(&self) -> &dyn Any;
// }

// pub struct BaseTransition {
//     target_state: Arc<RwLock<StateType>>,
//     // guards: Vec<Box<dyn Event>>,
//     event: Arc<RwLock<Event>>,
// }

// impl BaseTransition {
//     pub fn new(
//         target_state: Arc<RwLock<StateType>>,
//         // guards: Vec<Box<dyn Event>>,
//         event: Arc<RwLock<Event>>,
//     ) -> Self {
//         Self {
//             target_state,
//             // guards,
//             event,
//         }
//     }
// }

// impl Transition for BaseTransition {
//     fn set_target_state(&mut self, target_state: Arc<RwLock<StateType>>) {
//         self.target_state = target_state;
//     }

//     fn set_event(&mut self, event: Arc<RwLock<Event>>) {
//         self.event = event;
//     }

//     fn get_target_state(&self) -> Arc<RwLock<StateType>> {
//         Arc::clone(&self.target_state)
//     }

//     // Should we return the value of a rwlock ?
//     fn get_event(&self) -> &Event {
//         &*self.event.read().unwrap()
//     }

//     fn as_any(&self) -> &dyn Any {
//         self
//     }
// }
