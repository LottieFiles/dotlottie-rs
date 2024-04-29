use std::rc::Rc;
use std::sync::{Arc, RwLock};

pub mod errors;
pub mod events;
pub mod states;
pub mod transitions;

use crate::state_machine::states::StateTrait;
use crate::state_machine::transitions::TransitionTrait;
use crate::{Config, DotLottiePlayerContainer, Layout, Mode};
use dotlottie_fms::state_machine_parse;

use self::{errors::StateMachineError, events::Event, states::State, transitions::Transition};

pub trait StateMachineObserver {
    fn load_animation(&mut self, animation_id: &str);
    fn set_config(&mut self, config: Config);
    fn set_frame(&mut self, frame: f32);
}

pub struct StateMachine {
    pub states: Vec<Arc<RwLock<State>>>,
    pub current_state: Arc<RwLock<State>>,
    pub player: Rc<RwLock<DotLottiePlayerContainer>>,
}

impl StateMachine {
    pub fn new(
        state_machine_definition: &str,
        player: Rc<RwLock<DotLottiePlayerContainer>>,
    ) -> Result<StateMachine, StateMachineError> {
        let state_machine = StateMachine {
            states: Vec::new(),
            current_state: Arc::new(RwLock::new(State::Playback {
                config: Config::default(),
                reset_context: false,
                animation_id: "".to_string(),
                width: 0,
                height: 0,
                transitions: Vec::new(),
            })),
            player: player.clone(),
        };

        let sm = state_machine.parse(state_machine_definition);

        match sm {
            Ok((states, initial_state)) => {
                let new_sm = StateMachine {
                    states,
                    current_state: initial_state,
                    player: player.clone(),
                };

                println!("{:?}", "returning new sm");
                return Ok(new_sm);
            }
            Err(err) => {
                return Err(err);
            }
        };
    }

    // Parses the JSON of the state machine definition and creates the states and transitions
    pub fn parse(
        self,
        sm_definition: &str,
    ) -> Result<(Vec<Arc<RwLock<State>>>, Arc<RwLock<State>>), StateMachineError> {
        // let parser = dotlottie_fms::::new();
        let parsed_state_machine = state_machine_parse(sm_definition);
        let mut states: Vec<Arc<RwLock<State>>> = Vec::new();

        match parsed_state_machine {
            Ok(parsed_state_machine) => {
                // Loop through result states and create objects for each
                for state in parsed_state_machine.states {
                    match state.r#type.as_str() {
                        "PlaybackState" => {
                            let unwrapped_mode = state.mode.unwrap_or("Forward".to_string());
                            let mode = {
                                match unwrapped_mode.as_str() {
                                    "Forward" => Mode::Forward,
                                    "Reverse" => Mode::Reverse,
                                    "Bounce" => Mode::Bounce,
                                    "ReverseBounce" => Mode::ReverseBounce,
                                    _ => Mode::Forward,
                                }
                            };

                            let default_config = Config::default();

                            // Fill out a config with the state's values, if absent use default config values
                            let playback_config = Config {
                                mode: mode,
                                loop_animation: state
                                    .r#loop
                                    .unwrap_or(default_config.loop_animation),
                                speed: state.speed.unwrap_or(default_config.speed),
                                use_frame_interpolation: state
                                    .use_frame_interpolation
                                    .unwrap_or(default_config.use_frame_interpolation),
                                autoplay: state.autoplay.unwrap_or(default_config.autoplay),
                                segment: state.segment.unwrap_or(default_config.segment),
                                background_color: state
                                    .background_color
                                    .unwrap_or(default_config.background_color),
                                layout: Layout::default(),
                                marker: state.marker.unwrap_or(default_config.marker),
                            };

                            // Construct a State with the values we've gathered
                            let new_playback_state = State::Playback {
                                config: playback_config,
                                reset_context: state.reset_context.unwrap_or(false),
                                animation_id: state.animation_id.unwrap_or("".to_string()),
                                width: 1920,
                                height: 1080,
                                transitions: Vec::new(),
                            };

                            states.push(Arc::new(RwLock::new(new_playback_state)));
                        }
                        "SyncState" => {}
                        "FinalState" => {}
                        "GlobalState" => {}
                        _ => {}
                    }
                }

                // Loop through result transitions and create objects for each
                for transition in parsed_state_machine.transitions {
                    match transition.r#type.as_str() {
                        "Transition" => {
                            let target_state_index = transition.to_state;

                            // Use the provided index to get the state in the vec we've built
                            if target_state_index >= states.len() as u32 {
                                return Err(StateMachineError::ParsingError {
                                    reason: "Transition has an invalid target state index value!"
                                        .to_string(),
                                });
                            }

                            // Capture which event this transition has
                            if transition.numeric_event.is_some() {
                                let numeric_event = transition.numeric_event.unwrap();
                                let new_event = Event::Numeric(numeric_event.value);

                                let new_transition = Transition::Transition {
                                    target_state: target_state_index,
                                    event: Arc::new(RwLock::new(new_event)),
                                };

                                // Since the target is valid and transition created, we attach it to the state
                                let state_to_attch_to = transition.from_state;

                                if state_to_attch_to < states.len() as u32 {
                                    states[state_to_attch_to as usize]
                                        .write()
                                        .unwrap()
                                        .add_transition(new_transition);
                                    println!(
                                        "{}",
                                        states[state_to_attch_to as usize].write().unwrap()
                                    );
                                }
                            } else if transition.string_event.is_some() {
                                let string_event = transition.string_event.unwrap();
                                let new_event = Event::String(string_event.value);

                                let new_transition = Transition::Transition {
                                    target_state: target_state_index,
                                    event: Arc::new(RwLock::new(new_event)),
                                };

                                // Since the target is valid and transition created, we attach it to the state
                                let state_to_attch_to = transition.from_state;

                                if state_to_attch_to < states.len() as u32 {
                                    states[state_to_attch_to as usize]
                                        .write()
                                        .unwrap()
                                        .add_transition(new_transition);

                                    println!(
                                        "{}",
                                        states[state_to_attch_to as usize].write().unwrap()
                                    );
                                }
                            } else if transition.boolean_event.is_some() {
                                let boolean_event = transition.boolean_event.unwrap();
                                let new_event = Event::Bool(boolean_event.value);

                                let new_transition = Transition::Transition {
                                    target_state: target_state_index,
                                    event: Arc::new(RwLock::new(new_event)),
                                };

                                // Since the target is valid and transition created, we attach it to the state
                                let state_to_attch_to = transition.from_state;

                                if state_to_attch_to < states.len() as u32 {
                                    states[state_to_attch_to as usize]
                                        .write()
                                        .unwrap()
                                        .add_transition(new_transition);

                                    println!(
                                        "{}",
                                        states[state_to_attch_to as usize].write().unwrap()
                                    );
                                }
                            } else if transition.on_complete_event.is_some() {
                            } else if transition.on_pointer_down_event.is_some() {
                            } else if transition.on_pointer_up_event.is_some() {
                            } else if transition.on_pointer_enter_event.is_some() {
                            } else if transition.on_pointer_exit_event.is_some() {
                            } else if transition.on_pointer_move_event.is_some() {
                            }
                            // Todo - Add the rest of the event types
                        }
                        _ => {}
                    }
                }

                let mut initial_state = None;

                // All states and transitions have been created, we can set the state machine's initial state
                let initial_state_index = parsed_state_machine.descriptor.initial;

                if initial_state_index < states.len() as u32 {
                    initial_state = Some(states[initial_state_index as usize].clone());
                }

                return Ok((states, initial_state.unwrap()));
            }
            Err(error) => Err(StateMachineError::ParsingError {
                reason: error.to_string(),
            }),
        }
    }

    pub fn start(&mut self) {
        println!("{:?}", "Starting state machine");
        self.execute_current_state()
    }

    pub fn pause(&mut self) {}

    pub fn end(&mut self) {}

    pub fn set_initial_state(&mut self, state: Arc<RwLock<State>>) {
        self.current_state = state;
    }

    pub fn get_current_state(&self) -> Arc<RwLock<State>> {
        self.current_state.clone()
    }

    pub fn add_state(&mut self, state: Arc<RwLock<State>>) {
        self.states.push(state);
    }

    pub fn execute_current_state(&mut self) {
        let mut state = self.current_state.write().unwrap();

        state.execute(&self.player);
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

                                println!("Received event value: {}", received_event_value);
                                println!("String event value: {}", string_value);

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

unsafe impl Send for StateMachine {}
unsafe impl Sync for StateMachine {}
