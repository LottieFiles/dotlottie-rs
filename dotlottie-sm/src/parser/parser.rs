use std::sync::{Arc, RwLock};

use crate::event::Event;
use crate::state::StateTrait;
use crate::transition::Transition;
use crate::StateMachine;
use crate::{errors::StateMachineError, state::State};
use dotlottie_player_core::{Config, DotLottiePlayer, Layout, Mode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct DescriptorJson {
    id: String,
    initial: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct StateActionJson {
    r#type: String,
    url: Option<String>,
    target: Option<String>,
    theme_id: Option<String>,
    sound_id: Option<String>,
    message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct StateJson {
    r#type: String,
    animation_id: Option<String>,
    r#loop: Option<bool>,
    autoplay: Option<bool>,
    mode: Option<String>,
    speed: Option<f32>,
    marker: Option<String>,
    segment: Option<Vec<f32>>,
    background_color: Option<u32>,
    frame_interpolation: Option<bool>,
    entry_actions: Option<Vec<StateActionJson>>,
    exit_actions: Option<Vec<StateActionJson>>,
    reset_context: Option<bool>,
    frame_context_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TransitionGuardJson {}

#[derive(Serialize, Deserialize, Debug)]
struct NumericEventJson {
    value: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct StringEventJson {
    value: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BooleanEventJson {
    value: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct OnCompleteEventJson {}

#[derive(Serialize, Deserialize, Debug)]
struct OnPointerDownEventJson {
    target: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OnPointerUpEventJson {
    target: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OnPointerEnterEventJson {
    target: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OnPointerExitEventJson {
    target: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OnPointerMoveEventJson {
    target: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TransitionJson {
    r#type: String,
    from_state: u32,
    to_state: u32,
    guard: Option<Vec<TransitionGuardJson>>,
    numeric_event: Option<NumericEventJson>,
    string_event: Option<StringEventJson>,
    boolean_event: Option<BooleanEventJson>,
    on_complete_event: Option<OnCompleteEventJson>,
    on_pointer_down_event: Option<OnPointerDownEventJson>,
    on_pointer_up_event: Option<OnPointerUpEventJson>,
    on_pointer_enter_event: Option<OnPointerEnterEventJson>,
    on_pointer_exit_event: Option<OnPointerExitEventJson>,
    on_pointer_move_event: Option<OnPointerMoveEventJson>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListenerJson {
    r#type: String,
    target: Option<String>,
    action: Option<String>,
    value: Option<String>,
    context_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ContextJson {
    r#type: String,
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct StateMachineJson {
    descriptor: DescriptorJson,
    states: Vec<StateJson>,
    transitions: Vec<TransitionJson>,
    listeners: Vec<ListenerJson>,
    context_variables: Vec<ContextJson>,
}

pub struct Parser {
    current_state_machine: Option<StateMachineJson>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            current_state_machine: None,
        }
    }

    pub fn parse(&self, json: &str) -> Result<StateMachine, StateMachineError> {
        let result: Result<StateMachineJson, serde_json::Error> = serde_json::from_str(json);
        let player = DotLottiePlayer::new(Config::default());
        let mut states: Vec<Arc<RwLock<State>>> = Vec::new();

        match result {
            Ok(state_machine_json) => {
                // Loop through result states and create objects for each
                for state in state_machine_json.states {
                    match state.r#type.as_str() {
                        "PlaybackState" => {
                            let mut unwrapped_mode = state.mode.unwrap_or("Forward".to_string());
                            let mut mode = {
                                match unwrapped_mode.as_str() {
                                    "Forward" => Mode::Forward,
                                    "Reverse" => Mode::Reverse,
                                    "Bounce" => Mode::Bounce,
                                    "ReverseBounce" => Mode::ReverseBounce,
                                    _ => Mode::Forward,
                                }
                            };

                            // Fill out a config with the state's values, if absent use default config values
                            let playback_config = Config {
                                mode: mode,
                                loop_animation: state
                                    .r#loop
                                    .unwrap_or(Config::default().loop_animation),
                                speed: state.speed.unwrap_or(Config::default().speed),
                                use_frame_interpolation: state
                                    .frame_interpolation
                                    .unwrap_or(Config::default().use_frame_interpolation),
                                autoplay: state.autoplay.unwrap_or(Config::default().autoplay),
                                segment: state.segment.unwrap_or(Config::default().segment),
                                background_color: state
                                    .background_color
                                    .unwrap_or(Config::default().background_color),
                                layout: Layout::default(),
                                marker: state.marker.unwrap_or(Config::default().marker),
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
                for transition in state_machine_json.transitions {
                    match transition.r#type.as_str() {
                        "Transition" => {
                            let target_state_index = transition.to_state;

                            // Use the provided index to get the state in the vec we've built
                            if target_state_index > states.len() as u32 {
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
                let initial_state_index = state_machine_json.descriptor.initial;

                if initial_state_index < states.len() as u32 {
                    initial_state = Some(states[initial_state_index as usize].clone());
                }

                let state_machine: StateMachine = StateMachine {
                    states: states,
                    current_state: initial_state.unwrap(),
                    dotlottie_player: Arc::new(RwLock::new(player)),
                };

                // Loop through listeners and creating objects for each
                // Loop through context variables and creating objects for each

                // for state in state_machine_json.states {
                //     state_machine.add_state(state);
                // }

                // for transition in state_machine_json.transitions {
                //     state_machine.add_transition(transition);
                // }

                // for listener in state_machine_json.listeners {
                //     state_machine.add_listener(listener);
                // }

                // for context in state_machine_json.context_variables {
                //     state_machine.add_context_variable(context);
                // }

                Ok(state_machine)
            }
            Err(err) => Err(StateMachineError::ParsingError {
                reason: format!("Error parsing state machine definition file: {}", err),
            }),
        }
    }
}
