use std::sync::{Arc, RwLock};

use dotlottie_player_core::DotLottiePlayer;

use crate::{
    event::Event,
    state::{State, StateTrait},
    transition::TransitionTrait,
};

pub struct StateMachine {
    pub states: Vec<Arc<RwLock<State>>>,
    pub current_state: Arc<RwLock<State>>,
    pub dotlottie_player: Arc<RwLock<DotLottiePlayer>>,
}

impl StateMachine {
    pub fn start(&mut self) {
        self.execute_current_state()
    }

    pub fn pause(&mut self) {}

    pub fn end(&mut self) {}

    pub fn set_initial_state(&mut self, state: Arc<RwLock<State>>) {
        self.current_state = state;
    }

    pub fn set_player(&mut self, player: Arc<RwLock<DotLottiePlayer>>) {
        self.dotlottie_player = player;
    }

    pub fn get_current_state(&self) -> Arc<RwLock<State>> {
        self.current_state.clone()
    }

    pub fn add_state(&mut self, state: Arc<RwLock<State>>) {
        self.states.push(state);
    }

    pub fn execute_current_state(&mut self) {
        let mut player = self.dotlottie_player.write().unwrap();
        let mut state = self.current_state.write().unwrap();

        state.execute(&mut *player);
    }

    pub fn post_event(&mut self, event: &Event) {
        let mut string_event = false;
        let mut numeric_event = false;
        let mut bool_event = false;

        match event {
            Event::Bool(_) => bool_event = true,
            Event::String(_) => string_event = true,
            Event::Numeric(_) => numeric_event = true,
            Event::OnPointerDown(_, _) => {
                println!(">> OnPointerDownEvent");
            }
            Event::OnPointerUp(_, _) => {
                println!(">> OnPointerUpEvent");
            }
            Event::OnPointerMove(_, _) => {
                println!(">> OnPointerMoveEvent");
            }
            Event::OnPointerEnter(_, _) => {
                println!(">> OnPointerEnterEvent");
            }
            Event::OnPointerExit => {
                println!(">> OnPointerExitEvent");
            }
        }

        let curr_state = self.current_state.clone();
        let state_value_result = curr_state.read();

        if state_value_result.is_ok() {
            let state_value = state_value_result.unwrap();
            let mut iter = state_value.get_transitions().iter();
            let mut tmp_state: i32 = -1;

            // Loop through all transitions of the current state and check if we should transition to another state
            loop {
                match iter.next() {
                    Some(transition) => {
                        let unwrapped_transition = transition.read().unwrap();
                        let transition = &*unwrapped_transition;
                        let event_lock = transition.get_event();
                        let event_data = event_lock.read().unwrap();
                        let transition_event = &*event_data;

                        // Match the transition's event type and compare it to the received event
                        match transition_event {
                            Event::Bool(bool_value) => {
                                let mut received_event_value = false;

                                match event {
                                    Event::Bool(value) => {
                                        received_event_value = *value;
                                    }
                                    _ => {}
                                }

                                // Check the transitions value and compare to the received one to check if we should transition
                                if bool_event && received_event_value == *bool_value {
                                    let target_state = unwrapped_transition.get_target_state();

                                    tmp_state = target_state as i32;
                                }
                            }
                            Event::String(string_value) => {
                                let mut received_event_value = "";

                                match event {
                                    Event::String(value) => {
                                        received_event_value = value;
                                    }
                                    _ => {}
                                }

                                if string_event && received_event_value == string_value {
                                    let target_state = unwrapped_transition.get_target_state();

                                    tmp_state = target_state as i32;
                                }
                            }
                            Event::Numeric(numeric_value) => {
                                let mut received_event_value = 0.0;

                                match event {
                                    Event::Numeric(value) => {
                                        received_event_value = *value;
                                    }
                                    _ => {}
                                }

                                if numeric_event && received_event_value == *numeric_value {
                                    let target_state = unwrapped_transition.get_target_state();

                                    tmp_state = target_state as i32;
                                }
                            }
                            Event::OnPointerDown(_, _) => todo!(),
                            Event::OnPointerUp(_, _) => todo!(),
                            Event::OnPointerMove(_, _) => todo!(),
                            Event::OnPointerEnter(_, _) => todo!(),
                            Event::OnPointerExit => todo!(),
                        }
                    }
                    None => break,
                }
            }

            if tmp_state > -1 {
                let next_state = self.states.get(tmp_state as usize).unwrap();
                self.current_state = next_state.clone();

                self.execute_current_state();
            }
        }
    }

    pub fn remove_state(&mut self, state: Arc<RwLock<State>>) {
        let _ = state;
        // self.states.remove(state);
    }
}
