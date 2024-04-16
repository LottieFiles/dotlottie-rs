use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use dotlottie_player_core::DotLottiePlayer;

use crate::{event::Event, state::State, state::StateTrait, transition::TransitionTrait};

pub enum ContextValue {
    Numeric(i32),
    String(String),
    Boolean(bool),
}

pub struct StateMachine {
    pub states: Vec<Box<State>>,
    pub current_state: Arc<RwLock<State>>,
    pub context: HashMap<String, ContextValue>,
}

impl StateMachine {
    pub fn start(&mut self, player: &mut DotLottiePlayer) {
        self.execute_current_state(player)
    }

    pub fn pause(&mut self) {}

    pub fn end(&mut self) {}

    pub fn set_initial_state(&mut self, state: Arc<RwLock<State>>) {
        self.current_state = state;
    }

    // pub fn set_player(&mut self, player: &'a mut DotLottiePlayer) {
    //     self.dotlottie_player = Some(player)
    // }

    // pub fn add_state(&mut self, state: Box<dyn State>) {
    //     self.states.push(state);
    // }

    fn set_context(&mut self, key: String, value: ContextValue) {
        self.context.insert(key, value);
    }

    fn get_context(&self, key: &str) -> Option<&ContextValue> {
        self.context.get(key)
    }

    pub fn execute_current_state(&mut self, player: &mut DotLottiePlayer) {
        let current_state = self.current_state.clone();
        let state_value_result = current_state.write();

        if state_value_result.is_ok() {
            state_value_result.unwrap().execute(player);
        }
    }

    pub fn post_event(&mut self, event: &Event) {
        let mut string_event = false;
        let mut numeric_event = false;
        let mut bool_event = false;

        match event {
            Event::BoolEvent { value: _ } => bool_event = true,
            Event::StringEvent { value: _ } => string_event = true,
            Event::NumericEvent { value: _ } => numeric_event = true,
            Event::OnPointerDownEvent { x, y } => {
                println!(">> OnPointerDownEvent");
            }
            Event::OnPointerUpEvent { x, y } => {
                println!(">> OnPointerUpEvent");
            }
            Event::OnPointerMoveEvent { x, y } => {
                println!(">> OnPointerMoveEvent");
            }
            Event::OnPointerEnterEvent { x, y } => {
                println!(">> OnPointerEnterEvent");
            }
            Event::OnPointerExitEvent => {
                println!(">> OnPointerExitEvent");
            }
        }

        // if self.current_state.is_some() {
        let curr_state = self.current_state.clone();
        let state_value_result = curr_state.read();

        if state_value_result.is_ok() {
            let state_value = state_value_result.unwrap();
            let mut iter = state_value.get_transitions().iter();

            let mut tmp_state: Option<Arc<RwLock<State>>> = None;

            loop {
                match iter.next() {
                    Some(transition) => {
                        let unwrapped_transition = transition.read().unwrap();

                        let transition = &*unwrapped_transition;
                        let event_lock = transition.get_event();
                        let event_data = event_lock.read().unwrap();
                        let event = &*event_data;

                        match event {
                            Event::BoolEvent { value: _ } => {
                                if bool_event {
                                    let target_state = unwrapped_transition.get_target_state();

                                    tmp_state = Some(target_state);
                                }
                            }
                            Event::StringEvent { value } => {
                                if string_event {
                                    let target_state = unwrapped_transition.get_target_state();

                                    tmp_state = Some(target_state);
                                }
                            }
                            Event::NumericEvent { value } => {
                                if numeric_event {
                                    let target_state = unwrapped_transition.get_target_state();

                                    tmp_state = Some(target_state);
                                }
                            }
                            Event::OnPointerDownEvent { x, y } => todo!(),
                            Event::OnPointerUpEvent { x, y } => todo!(),
                            Event::OnPointerMoveEvent { x, y } => todo!(),
                            Event::OnPointerEnterEvent { x, y } => todo!(),
                            Event::OnPointerExitEvent => todo!(),
                        }
                    }
                    None => break,
                }
            }

            if tmp_state.is_some() {
                let next_state = tmp_state.unwrap();
                self.current_state = next_state;

                println!(
                    ">> Transitioning to next state {0}",
                    self.current_state.read().unwrap().as_str()
                );
            }
        }
    }

    pub fn remove_state(&mut self, state: Arc<RwLock<State>>) {
        // self.states.remove(state);
    }
}
