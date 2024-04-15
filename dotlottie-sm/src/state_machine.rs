use std::{
    borrow::Borrow,
    sync::{Arc, RwLock},
};

use dotlottie_player_core::DotLottiePlayer;

use crate::{
    event::Event,
    state::{State, StateType},
    transition::{self, TransitionTrait},
};

pub struct StateMachine {
    pub states: Vec<Box<StateType>>,
    pub current_state: Arc<RwLock<StateType>>,
}

impl StateMachine {
    pub fn start(&mut self, player: &mut DotLottiePlayer) {
        self.execute_current_state(player)
    }

    pub fn pause(&mut self) {}

    pub fn end(&mut self) {}

    pub fn set_initial_state(&mut self, state: Arc<RwLock<StateType>>) {
        self.current_state = state;
    }

    // pub fn set_player(&mut self, player: &'a mut DotLottiePlayer) {
    //     self.dotlottie_player = Some(player)
    // }

    // pub fn add_state(&mut self, state: Box<dyn State>) {
    //     self.states.push(state);
    // }

    // pub fn execute_all_states(&mut self, player: &mut DotLottiePlayer) {
    //     self.states.iter_mut().for_each(|state| {
    //         state.execute(player);
    //     });
    // }

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

        // if event.downcast_ref::<StringEvent>().is_some() {
        //     // string event
        //     println!(
        //         ">> StringEvent: {:?}",
        //         event.downcast_ref::<StringEvent>().unwrap().value
        //     );
        //     string_event = true;
        // }
        // if event.downcast_ref::<NumericEvent>().is_some() {
        //     // numeric event
        //     println!(
        //         ">> NumericEvent: {:?}",
        //         event.downcast_ref::<NumericEvent>().unwrap().value
        //     );
        //     numeric_event = true;
        // }
        // if event.downcast_ref::<BoolEvent>().is_some() {
        //     // numeric event
        //     println!(
        //         ">> BoolEvent: {:?}",
        //         event.downcast_ref::<BoolEvent>().unwrap().value
        //     );
        //     bool_event = true;
        // }

        // if self.current_state.is_some() {
        let curr_state = self.current_state.clone();
        let state_value_result = curr_state.read();

        if state_value_result.is_ok() {
            let state_value = state_value_result.unwrap();
            let mut iter = state_value.get_transitions().iter();

            let mut tmp_state: Option<Arc<RwLock<StateType>>> = None;

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
                                let target_state = unwrapped_transition.get_target_state();

                                tmp_state = Some(target_state);
                            }
                            Event::StringEvent { value } => {
                                let target_state = unwrapped_transition.get_target_state();

                                tmp_state = Some(target_state);
                            }
                            Event::NumericEvent { value } => {
                                let target_state = unwrapped_transition.get_target_state();

                                tmp_state = Some(target_state);
                            }
                            Event::OnPointerDownEvent { x, y } => todo!(),
                            Event::OnPointerUpEvent { x, y } => todo!(),
                            Event::OnPointerMoveEvent { x, y } => todo!(),
                            Event::OnPointerEnterEvent { x, y } => todo!(),
                            Event::OnPointerExitEvent => todo!(),
                        }

                        // if let Some(_numeric_event) = transition
                        //     .get_event()
                        //     .as_any()
                        //     .downcast_ref::<NumericEvent>()
                        // {
                        //     println!(">> NumericEvent transition");
                        //     if numeric_event {
                        //         let target_state = transition.get_target_state();

                        //         tmp_state = Some(target_state);
                        //     }
                        // }
                        // if let Some(_string_event) = transition
                        //     .get_event()
                        //     .as_any()
                        //     .downcast_ref::<StringEvent>()
                        // {
                        //     if string_event {
                        //         let target_state = transition.get_target_state();

                        //         tmp_state = Some(target_state);
                        //     }
                        // }
                        // if let Some(_bool_event) =
                        //     transition.get_event().as_any().downcast_ref::<BoolEvent>()
                        // {
                        //     if bool_event {
                        //         let target_state = transition.get_target_state();

                        //         tmp_state = Some(target_state);
                        //     }
                        // }
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

    pub fn remove_state(&mut self, state: Box<dyn State>) {
        // self.states.remove(state);
    }
}
