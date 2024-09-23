use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::rc::Rc;
use std::sync::RwLock;

mod actions;
pub mod errors;
pub mod events;
pub mod listeners;
pub mod parser;
pub mod states;
pub mod transitions;

use actions::Action;
use parser::{ActionJson, ListenerJson, StateJson, TransitionJson};
use states::StateTrait;

use crate::parser::StringNumberBool;
use crate::state_machine::listeners::Listener;
use crate::state_machine::transitions::guard::Guard;
use crate::{Config, DotLottiePlayerContainer, Layout, Mode};

use self::parser::state_machine_parse;
use self::{errors::StateMachineError, events::Event, states::State, transitions::Transition};

pub trait StateMachineObserver: Send + Sync {
    fn on_transition(&self, previous_state: String, new_state: String);
    fn on_state_entered(&self, entering_state: String);
    fn on_state_exit(&self, leaving_state: String);
}

#[derive(PartialEq, Debug)]
pub enum StateMachineStatus {
    Running,
    Paused,
    Stopped,
}

// todo: Remove Rcs, or replace with Rcs
pub struct StateMachine {
    pub global_state: Option<State>,
    pub states: HashMap<String, State>,

    pub listeners: Vec<Listener>,
    pub current_state: Option<Rc<State>>,
    pub player: Option<Rc<RwLock<DotLottiePlayerContainer>>>,
    pub status: StateMachineStatus,

    numeric_trigger: HashMap<String, f32>,
    string_trigger: HashMap<String, String>,
    bool_trigger: HashMap<String, bool>,
    event_trigger: HashMap<String, String>,

    observers: RwLock<Vec<Rc<dyn StateMachineObserver>>>,
}

impl Default for StateMachine {
    fn default() -> StateMachine {
        StateMachine {
            global_state: None,
            states: HashMap::new(),
            listeners: Vec::new(),
            current_state: None,
            player: None,
            numeric_trigger: HashMap::new(),
            string_trigger: HashMap::new(),
            bool_trigger: HashMap::new(),
            event_trigger: HashMap::new(),
            status: StateMachineStatus::Stopped,
            observers: RwLock::new(Vec::new()),
        }
    }
}

impl Display for StateMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMachine")
            .field("global_state", &self.global_state)
            .field("states", &self.states)
            .field("listeners", &self.listeners)
            .field("current_state", &self.current_state)
            .field("numeric_trigger", &self.numeric_trigger)
            .field("string_trigger", &self.string_trigger)
            .field("bool_trigger", &self.bool_trigger)
            .field("event_trigger", &self.event_trigger)
            .field("status", &self.status)
            .finish()
    }
}

// todo: Use a lifetime for the player
impl StateMachine {
    pub fn new(
        state_machine_definition: &str,
        player: Rc<RwLock<DotLottiePlayerContainer>>,
    ) -> Result<StateMachine, StateMachineError> {
        let mut state_machine = StateMachine {
            global_state: None,
            states: HashMap::new(),
            listeners: Vec::new(),
            current_state: None,
            player: Some(player.clone()),
            numeric_trigger: HashMap::new(),
            string_trigger: HashMap::new(),
            bool_trigger: HashMap::new(),
            event_trigger: HashMap::new(),
            status: StateMachineStatus::Stopped,
            observers: RwLock::new(Vec::new()),
        };

        state_machine.create_state_machine(state_machine_definition, &player)
    }

    pub fn subscribe(&self, observer: Rc<dyn StateMachineObserver>) {
        let mut observers = self.observers.write().unwrap();
        observers.push(observer);
    }

    pub fn unsubscribe(&self, observer: &Rc<dyn StateMachineObserver>) {
        self.observers
            .write()
            .unwrap()
            .retain(|o| !Rc::ptr_eq(o, observer));
    }

    pub fn get_numeric_trigger(&self, key: &str) -> Option<f32> {
        self.numeric_trigger.get(key).cloned()
    }

    pub fn get_string_trigger(&self, key: &str) -> Option<String> {
        self.string_trigger.get(key).cloned()
    }

    pub fn get_bool_trigger(&self, key: &str) -> Option<bool> {
        self.bool_trigger.get(key).cloned()
    }

    pub fn set_numeric_trigger(&mut self, key: &str, value: f32) {
        self.numeric_trigger.insert(key.to_string(), value);
    }

    pub fn set_string_trigger(&mut self, key: &str, value: &str) {
        self.string_trigger
            .insert(key.to_string(), value.to_string());
    }

    pub fn set_bool_trigger(&mut self, key: &str, value: bool) {
        self.bool_trigger.insert(key.to_string(), value);
    }

    fn build_listener(&self, listener_to_build: ListenerJson) -> Listener {
        match listener_to_build {
            ListenerJson::PointerUp {
                layer_name,
                actions,
            } => {
                let mut new_actions: Vec<Action> = Vec::new();

                for action in actions {
                    let new_action = self.build_action(action);

                    new_actions.push(new_action);
                }

                let new_listener = Listener::PointerUp {
                    layer_name,
                    actions: new_actions,
                };

                new_listener
            }
            ListenerJson::PointerDown {
                layer_name,
                actions,
            } => {
                let mut new_actions: Vec<Action> = Vec::new();

                for action in actions {
                    let new_action = self.build_action(action);

                    new_actions.push(new_action);
                }

                let new_listener = Listener::PointerDown {
                    layer_name,
                    actions: new_actions,
                };

                new_listener
            }
            ListenerJson::PointerEnter {
                layer_name,
                actions,
            } => {
                let mut new_actions: Vec<Action> = Vec::new();

                for action in actions {
                    let new_action = self.build_action(action);

                    new_actions.push(new_action);
                }

                let new_listener = Listener::PointerEnter {
                    layer_name,
                    actions: new_actions,
                };

                new_listener
            }
            ListenerJson::PointerExit {
                layer_name,
                actions,
            } => {
                let mut new_actions: Vec<Action> = Vec::new();

                for action in actions {
                    let new_action = self.build_action(action);

                    new_actions.push(new_action);
                }

                let new_listener = Listener::PointerExit {
                    layer_name,
                    actions: new_actions,
                };

                new_listener
            }
            ListenerJson::PointerMove {
                layer_name,
                actions,
            } => {
                let mut new_actions: Vec<Action> = Vec::new();

                for action in actions {
                    let new_action = self.build_action(action);

                    new_actions.push(new_action);
                }
                let new_listener = Listener::PointerMove {
                    layer_name,
                    actions: new_actions,
                };

                new_listener
            }
            ListenerJson::OnComplete {
                state_name,
                actions,
            } => {
                let mut new_actions: Vec<Action> = Vec::new();

                for action in actions {
                    let new_action = self.build_action(action);

                    new_actions.push(new_action);
                }

                let new_listener = Listener::OnComplete {
                    state_name,
                    actions: new_actions,
                };

                new_listener
            }
        }
    }

    fn build_action(&self, action_to_build: ActionJson) -> Action {
        match action_to_build {
            parser::ActionJson::OpenUrl { url } => Action::OpenUrl { url },
            parser::ActionJson::ThemeAction { theme_id, target } => Action::SetTheme { theme_id },
            parser::ActionJson::Increment {
                trigger_name,
                value,
            } => Action::Increment {
                trigger_name,
                value,
            },
            parser::ActionJson::Decrement {
                trigger_name,
                value,
            } => Action::Decrement {
                trigger_name,
                value,
            },
            parser::ActionJson::Toggle { trigger_name } => Action::Toggle { trigger_name },
            parser::ActionJson::SetBoolean {
                trigger_name,
                value,
            } => Action::SetBoolean {
                trigger_name,
                value,
            },
            parser::ActionJson::SetString {
                trigger_name,
                value,
            } => Action::SetString {
                trigger_name,
                value,
            },
            parser::ActionJson::SetNumeric {
                trigger_name,
                value,
            } => Action::SetNumeric {
                trigger_name,
                value,
            },
            parser::ActionJson::Fire { trigger_name } => Action::Fire { trigger_name },
            parser::ActionJson::SetExpression {
                layer_name,
                property_index,
                var_name,
                value,
            } => Action::SetExpression {
                layer_name,
                property_index,
                var_name,
                value,
            },
            parser::ActionJson::SetTheme { theme_id } => Action::SetTheme { theme_id },
            parser::ActionJson::SetFrame { value } => Action::SetFrame { value },
            parser::ActionJson::SetSlot { value } => Action::SetSlot { value },
            parser::ActionJson::FireCustomEvent { value } => Action::FireCustomEvent { value },
        }
    }

    fn build_state(&self, state_to_build: StateJson) -> State {
        match state_to_build {
            StateJson::PlaybackState {
                name,
                transitions,
                animation_id,
                r#loop,
                autoplay,
                mode,
                speed,
                segment,
                background_color,
                use_frame_interpolation,
                entry_actions,
                exit_actions,
            } => {
                let unwrapped_mode = mode.unwrap_or("Forward".to_string());
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
                    mode,
                    loop_animation: r#loop.unwrap_or(default_config.loop_animation),
                    speed: speed.unwrap_or(default_config.speed),
                    use_frame_interpolation: use_frame_interpolation
                        .unwrap_or(default_config.use_frame_interpolation),
                    autoplay: autoplay.unwrap_or(default_config.autoplay),
                    marker: segment.unwrap_or(default_config.marker),
                    background_color: background_color.unwrap_or(default_config.background_color),
                    layout: Layout::default(),
                    segment: [].to_vec(),
                };

                let mut state_entry_actions: Vec<Action> = Vec::new();
                let mut state_exit_actions: Vec<Action> = Vec::new();

                /* Create the entry-actions */
                if let Some(actions) = entry_actions {
                    for action in actions {
                        let new_action = self.build_action(action);

                        state_entry_actions.push(new_action);
                    }
                }

                /* Create the exit-actions */
                if let Some(actions) = exit_actions {
                    for action in actions {
                        let new_action = self.build_action(action);

                        state_exit_actions.push(new_action);
                    }
                }

                // Construct a State with the values we've gathered
                let mut new_playback_state = State::Playback {
                    name,
                    config: playback_config,
                    animation_id: animation_id.unwrap_or("".to_string()),
                    transitions: Vec::new(),
                    entry_actions: Some(state_entry_actions),
                    exit_actions: Some(state_exit_actions),
                };

                // Build the transitions
                for transition in transitions {
                    match transition {
                        TransitionJson::Transition { to_state, guards } => {
                            let new_transition = Transition::Transition {
                                target_state: to_state,
                                guards: Vec::new(),
                            };

                            new_playback_state.add_transition(&new_transition);
                        }
                    }
                }

                new_playback_state
            }
            StateJson::GlobalState {
                name,
                transitions,
                entry_actions,
                exit_actions,
            } => {
                let mut state_entry_actions: Vec<Action> = Vec::new();
                let mut state_exit_actions: Vec<Action> = Vec::new();

                /* Create the entry-actions */
                if let Some(actions) = entry_actions {
                    for action in actions {
                        let new_action = self.build_action(action);

                        state_entry_actions.push(new_action);
                    }
                }

                /* Create the exit-actions */
                if let Some(actions) = exit_actions {
                    for action in actions {
                        let new_action = self.build_action(action);

                        state_exit_actions.push(new_action);
                    }
                }

                let mut new_global_state = State::Global {
                    name,
                    transitions: Vec::new(),
                    entry_actions: Some(state_entry_actions),
                    exit_actions: Some(state_exit_actions),
                };

                // Build the transitions
                for transition in transitions {
                    match transition {
                        TransitionJson::Transition { to_state, guards } => {
                            let new_transition = Transition::Transition {
                                target_state: to_state,
                                guards: Vec::new(),
                            };

                            new_global_state.add_transition(&new_transition);
                        }
                    }
                }

                new_global_state
            }
        }
    }

    // Parses the JSON of the state machine definition and creates the states and transitions
    pub fn create_state_machine(
        &mut self,
        sm_definition: &str,
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
    ) -> Result<StateMachine, StateMachineError> {
        let parsed_state_machine = state_machine_parse(sm_definition);

        if parsed_state_machine.is_err() {
            println!(
                "Error parsing state machine definition: {:?}",
                parsed_state_machine.err()
            );
            return Err(StateMachineError::ParsingError {
                reason: "Failed to parse state machine definition".to_string(),
            });
        }

        // TODO
        // - Report PROPER errors if there are any
        // - Create states and transitions based on the parsed state machine
        // - Run it through a check pipeline to ensure everything is valid. Types based on their actions, inf. loops etc.

        // todo somehow get the trigger json without having to parse it again
        // self.json_trigger = Some(
        //     state_machine_parse(sm_definition)
        //         .unwrap()
        //         .trigger_variables,
        // );

        let mut new_state_machine = StateMachine::default();

        match parsed_state_machine {
            Ok(parsed_state_machine) => {
                /* Build every state and their transitions */
                for state in parsed_state_machine.states {
                    let new_state = self.build_state(state);

                    if let State::Global { .. } = new_state {
                        new_state_machine.global_state = Some(new_state.clone());
                    }

                    /* Insert the newly built state in to the hashmap of the state machine */
                    new_state_machine
                        .states
                        .insert(new_state.get_name().to_string(), new_state);
                }

                /* Build every listener and their action */
                if let Some(listeners) = parsed_state_machine.listeners {
                    for listener in listeners {
                        let new_listener = self.build_listener(listener);

                        new_state_machine.listeners.push(new_listener);
                    }
                }

                /* Build all trigger variables */
                if let Some(triggers) = parsed_state_machine.triggers {
                    for trigger in triggers {
                        match trigger {
                            parser::TriggerJson::Numeric { name, value } => {
                                new_state_machine.set_numeric_trigger(&name, value);
                            }
                            parser::TriggerJson::String { name, value } => {
                                new_state_machine.set_string_trigger(&name, &value);
                            }
                            parser::TriggerJson::Boolean { name, value } => {
                                new_state_machine.set_bool_trigger(&name, value);
                            }
                            parser::TriggerJson::Event { name } => {
                                new_state_machine.event_trigger.insert(name, "".to_string());
                            }
                        }
                    }
                }

                new_state_machine.player = Some(player.clone());

                // All states and transitions have been created, we can set the state machine's initial state
                let initial_state_index = parsed_state_machine.descriptor.initial;

                if let Some(state) = new_state_machine.states.get(&initial_state_index) {
                    /* Create a reference to the state marked as initial */
                    new_state_machine.current_state = Some(Rc::new(state.clone()));
                }

                Ok(new_state_machine)
            }
            Err(error) => return Err(error),
        }
    }

    pub fn start(&mut self) {
        self.status = StateMachineStatus::Running;
        // self.execute_current_state();
    }

    pub fn pause(&mut self) {
        self.status = StateMachineStatus::Paused;
    }

    pub fn end(&mut self) {
        self.status = StateMachineStatus::Stopped;
    }

    pub fn get_current_state(&self) -> Option<Rc<State>> {
        self.current_state.clone()
    }

    // pub fn add_state(&mut self, state: Rc<RwLock<State>>) {
    //     self.states.push(state);
    // }

    pub fn get_listeners(&self) -> &Vec<Listener> {
        &self.listeners
    }

    // Return codes
    // 0: Success
    // 1: Failure
    // 2: Play animation
    // 3: Pause animation
    // 4: Request and draw a new single frame of the animation (needed for sync state)
    // pub fn execute_current_state(&mut self) -> i32 {
    //     if self.current_state.is_none() {
    //         return 1;
    //     }

    //     // Check if current_state is not None and execute the state
    //     if let Some(ref state) = self.current_state {
    //         let unwrapped_state = state;
    //         // let reset_key = unwrapped_state.get_reset_trigger_key();

    //         if !reset_key.is_empty() {
    //             if reset_key == "*" {
    //                 // Todo dont clear reset to their original values from file
    //                 // self.numeric_trigger.clear();
    //                 // self.string_trigger.clear();
    //                 // self.bool_trigger.clear();
    //             } else {
    //                 if self.numeric_trigger.contains_key(reset_key) {
    //                     // self.numeric_trigger.remove(reset_key);
    //                 }

    //                 if self.string_trigger.contains_key(reset_key) {
    //                     // self.string_trigger.remove(reset_key);
    //                 }

    //                 if self.bool_trigger.contains_key(reset_key) {
    //                     // self.bool_trigger.remove(reset_key);
    //                 }
    //             }
    //         }

    //         if self.player.is_some() {
    //             return unwrapped_state.execute(
    //                 self.player.as_mut().unwrap(),
    //                 &self.string_trigger,
    //                 &self.bool_trigger,
    //                 &self.numeric_trigger,
    //                 &self.event_trigger,
    //             );
    //         } else {
    //             return 1;
    //         }
    //     }

    //     0
    // }

    // fn verify_if_guards_are_met(&self, guard: &Guard) -> bool {
    //     match guard.compare_to {
    //         StringNumberBool::String(_) => {
    //             if guard.string_trigger_is_satisfied(&self.string_trigger) {
    //                 return true;
    //             }
    //         }
    //         StringNumberBool::F32(_) => {
    //             if guard.numeric_trigger_is_satisfied(&self.numeric_trigger) {
    //                 return true;
    //             }
    //         }
    //         StringNumberBool::Bool(_) => {
    //             if guard.bool_trigger_is_satisfied(&self.bool_trigger) {
    //                 return true;
    //             }
    //         }
    //     }

    //     false
    // }

    // fn perform_hit_check(&self, target: &str, x: f32, y: f32) -> bool {
    //     // A layer name was provided, we need to check if the pointer is within the layer
    //     let pointer_target = target;

    //     let player_ref = self.player.as_ref();

    //     if player_ref.is_some() {
    //         let player = player_ref.unwrap();
    //         let player_read = player.try_read();

    //         match player_read {
    //             Ok(player) => {
    //                 let player = &*player;

    //                 player.hit_check(pointer_target, x, y)
    //             }
    //             Err(_) => false,
    //         }
    //     } else {
    //         false
    //     }
    // }

    // fn evaluate_transition(&self, transitions: &[Rc<RwLock<Transition>>], event: &Event) -> i32 {
    //     let mut tmp_state: i32 = -1;
    //     let iter = transitions.iter();

    //     for transition in iter {
    //         let unwrapped_transition = transition.read().unwrap();
    //         let target_state = unwrapped_transition.get_target_state();
    //         let transition = &*unwrapped_transition;
    //         let event_lock = transition.get_event();
    //         let event_data = event_lock.read().unwrap();
    //         let transition_event = &*event_data;
    //         let transition_guards = transition.get_guards();

    //         // Match the transition's event type and compare it to the received event
    //         match transition_event {
    //             InternalEvent::Bool { value } => {
    //                 let bool_value = value;

    //                 if let Event::Bool { value } = event {
    //                     if *value == *bool_value {
    //                         // If there are guards loop over them and check if theyre verified
    //                         if !transition_guards.is_empty() {
    //                             for guard in transition_guards {
    //                                 if self.verify_if_guards_are_met(guard) {
    //                                     tmp_state = target_state as i32;
    //                                 }
    //                             }
    //                         } else {
    //                             tmp_state = target_state as i32;
    //                         }
    //                     }
    //                 }
    //             }
    //             InternalEvent::String { value } => {
    //                 let string_value = value;

    //                 if let Event::String { value } = event {
    //                     if string_value == value {
    //                         // If there are guards loop over them and check if theyre verified
    //                         if !transition_guards.is_empty() {
    //                             for guard in transition_guards {
    //                                 if self.verify_if_guards_are_met(guard) {
    //                                     tmp_state = target_state as i32;
    //                                 }
    //                             }
    //                         } else {
    //                             tmp_state = target_state as i32;
    //                         }
    //                     }
    //                 }
    //             }
    //             InternalEvent::Numeric { value } => {
    //                 let num_value = value;

    //                 if let Event::Numeric { value } = event {
    //                     if *value == *num_value {
    //                         // If there are guards loop over them and check if theyre verified
    //                         if !transition_guards.is_empty() {
    //                             for guard in transition_guards {
    //                                 if self.verify_if_guards_are_met(guard) {
    //                                     tmp_state = target_state as i32;
    //                                 }
    //                             }
    //                         } else {
    //                             tmp_state = target_state as i32;
    //                         }
    //                     }
    //                 }
    //             }
    //             InternalEvent::OnComplete => {
    //                 if let Event::OnComplete = event {
    //                     // If there are guards loop over them and check if theyre verified
    //                     if !transition_guards.is_empty() {
    //                         for guard in transition_guards {
    //                             if self.verify_if_guards_are_met(guard) {
    //                                 tmp_state = target_state as i32;
    //                             }
    //                         }
    //                     } else {
    //                         tmp_state = target_state as i32;
    //                     }
    //                 }
    //             }
    //             // This is checking the state machine's event, not the passed event
    //             InternalEvent::OnPointerDown { target } => {
    //                 if let Event::OnPointerDown { x, y } = event {
    //                     // If there are guards loop over them and check if theyre verified
    //                     if !transition_guards.is_empty() {
    //                         for guard in transition_guards {
    //                             if self.verify_if_guards_are_met(guard) {
    //                                 tmp_state = target_state as i32;
    //                             }
    //                         }
    //                     } else if target.is_some() && self.player.is_some() {
    //                         if self.perform_hit_check(target.as_ref().unwrap(), *x, *y) {
    //                             tmp_state = target_state as i32;
    //                         }
    //                     } else {
    //                         tmp_state = target_state as i32;
    //                     }
    //                 }
    //             }
    //             InternalEvent::OnPointerUp { target } => {
    //                 if let Event::OnPointerUp { x, y } = event {
    //                     // If there are guards loop over them and check if theyre verified
    //                     if !transition_guards.is_empty() {
    //                         for guard in transition_guards {
    //                             if self.verify_if_guards_are_met(guard) {
    //                                 tmp_state = target_state as i32;
    //                             }
    //                         }
    //                     } else if target.is_some() && self.player.is_some() {
    //                         if self.perform_hit_check(target.as_ref().unwrap(), *x, *y) {
    //                             tmp_state = target_state as i32;
    //                         }
    //                     } else {
    //                         tmp_state = target_state as i32;
    //                     }
    //                 }
    //             }
    //             InternalEvent::OnPointerMove { target } => {
    //                 if let Event::OnPointerMove { x, y } = event {
    //                     // If there are guards loop over them and check if theyre verified
    //                     if !transition_guards.is_empty() {
    //                         for guard in transition_guards {
    //                             if self.verify_if_guards_are_met(guard) {
    //                                 tmp_state = target_state as i32;
    //                             }
    //                         }
    //                     } else if target.is_some() && self.player.is_some() {
    //                         if self.perform_hit_check(target.as_ref().unwrap(), *x, *y) {
    //                             tmp_state = target_state as i32;
    //                         }
    //                     } else {
    //                         tmp_state = target_state as i32;
    //                     }
    //                 }
    //             }
    //             InternalEvent::OnPointerEnter { target } => {
    //                 let mut received_event_values = Event::OnPointerEnter { x: 0.0, y: 0.0 };

    //                 match event {
    //                     Event::OnPointerEnter { x, y } => {
    //                         received_event_values = Event::OnPointerEnter { x: *x, y: *y };
    //                     }
    //                     Event::OnPointerMove { x, y } => {
    //                         received_event_values = Event::OnPointerMove { x: *x, y: *y };
    //                     }
    //                     _ => {}
    //                 }

    //                 // If there are guards loop over them and check if theyre verified
    //                 if !transition_guards.is_empty() {
    //                     for guard in transition_guards {
    //                         if self.verify_if_guards_are_met(guard) {
    //                             tmp_state = target_state as i32;
    //                         }
    //                     }
    //                 } else if target.is_some() && self.player.is_some() {
    //                     if self.perform_hit_check(
    //                         target.as_ref().unwrap(),
    //                         received_event_values.x(),
    //                         received_event_values.y(),
    //                     ) {
    //                         let current_state_name =
    //                             if let Some(current_state) = &self.current_state {
    //                                 if let Ok(state) = current_state.read() {
    //                                     state.get_name()
    //                                 } else {
    //                                     return -1;
    //                                 }
    //                             } else {
    //                                 return -1;
    //                             };

    //                         let target_state_name = if let Some(target_state) =
    //                             self.states.get(target_state as usize)
    //                         {
    //                             if let Ok(state) = target_state.read() {
    //                                 state.get_name()
    //                             } else {
    //                                 return 1; // Handle read lock error
    //                             }
    //                         } else {
    //                             return 1; // Handle invalid index
    //                         };

    //                         // This prevent the state from transitioning to itself over and over again
    //                         if current_state_name != target_state_name {
    //                             tmp_state = target_state as i32;
    //                         }
    //                     }
    //                 } else {
    //                     tmp_state = target_state as i32;
    //                 }
    //             }
    //             InternalEvent::OnPointerExit { target } => {
    //                 let mut received_event_values = Event::OnPointerEnter { x: 0.0, y: 0.0 };

    //                 match event {
    //                     Event::OnPointerExit { x, y } => {
    //                         received_event_values = Event::OnPointerExit { x: *x, y: *y };
    //                     }
    //                     Event::OnPointerMove { x, y } => {
    //                         received_event_values = Event::OnPointerMove { x: *x, y: *y };
    //                     }
    //                     _ => {}
    //                 }

    //                 // If there are guards loop over them and check if theyre verified
    //                 if !transition_guards.is_empty() {
    //                     for guard in transition_guards {
    //                         if self.verify_if_guards_are_met(guard) {
    //                             tmp_state = target_state as i32;
    //                         }
    //                     }
    //                 } else if target.is_some() && self.player.is_some() {
    //                     // Check if current state is the target state
    //                     let current_state_name = if let Some(current_state) = &self.current_state {
    //                         if let Ok(state) = current_state.read() {
    //                             state.get_name()
    //                         } else {
    //                             return -1;
    //                         }
    //                     } else {
    //                         return -1;
    //                     };

    //                     if current_state_name == *target.as_ref().unwrap()
    //                         && !self.perform_hit_check(
    //                             target.as_ref().unwrap(),
    //                             received_event_values.x(),
    //                             received_event_values.y(),
    //                         )
    //                     {
    //                         tmp_state = target_state as i32;
    //                     }
    //                 } else {
    //                     // Check if coordinates are outside of the player
    //                     let (width, height) = self.player.as_ref().unwrap().read().unwrap().size();

    //                     if received_event_values.x() < 0.0
    //                         || received_event_values.x() > width as f32
    //                         || received_event_values.y() < 0.0
    //                         || received_event_values.y() > height as f32
    //                     {
    //                         tmp_state = target_state as i32;
    //                     }
    //                 }
    //             }
    //             // InternalEvent::SetNumerictrigger { key: _, value: _ } => {}
    //         }
    //     }

    //     tmp_state
    // }

    // Return codes
    // 0: Success
    // 1: Failure
    // 2: Play animation
    // 3: Pause animation
    // 4: Request and draw a new single frame of the animation (needed for sync state)
    pub fn post_event(&mut self, event: &Event) -> i32 {
        if self.status == StateMachineStatus::Stopped || self.status == StateMachineStatus::Paused {
            return 1;
        }

        if self.current_state.is_none() {
            return 1;
        } else {
            return 0;
        }

        // Only match with setNumerictrigger as if this is the case we return early
        // Other event types are handled within self.evaluate_transition
        // if let Event::SetNumerictrigger { key, value } = event {
        //     self.set_numeric_trigger(key, *value);

        //     let s = self.current_state.clone();

        //     // If current state is a sync state, we need to update the frame
        //     if let Some(state) = s {
        //         let unwrapped_state = state.try_read();

        //         if let Ok(state) = unwrapped_state {
        //             let state_value = &*state;

        //             if let State::Sync { .. } = state_value {
        //                 return self.execute_current_state();
        //             }
        //         }
        //     }

        //     return 0;
        // }

        // Firstly check if we have a global state within the state machine.
        // if self.global_state.is_some() {
        //     let global_state = self.global_state.clone().unwrap();
        //     let global_state_value = global_state.try_read();

        //     if global_state_value.is_ok() {
        //         let state_value = global_state_value.unwrap();
        //         let tmp_state = self.evaluate_transition(state_value.get_transitions(), event);

        //         if tmp_state > -1 {
        //             let next_state = self.states.get(tmp_state as usize).unwrap();

        //             // Emit transtion occured event
        //             self.observers.read().unwrap().iter().for_each(|observer| {
        //                 observer.on_transition(
        //                     (*self
        //                         .current_state
        //                         .as_ref()
        //                         .unwrap()
        //                         .read()
        //                         .unwrap()
        //                         .get_name())
        //                     .to_string(),
        //                     (*next_state.read().unwrap().get_name()).to_string(),
        //                 )
        //             });

        //             // Emit leaving current state event
        //             if self.current_state.is_some() {
        //                 self.observers.read().unwrap().iter().for_each(|observer| {
        //                     observer.on_state_exit(
        //                         (*self
        //                             .current_state
        //                             .as_ref()
        //                             .unwrap()
        //                             .read()
        //                             .unwrap()
        //                             .get_name())
        //                         .to_string(),
        //                     );
        //                 });
        //             }

        //             self.current_state = Some(next_state.clone());

        //             // Emit entering a new state
        //             self.observers.read().unwrap().iter().for_each(|observer| {
        //                 observer
        //                     .on_state_entered((*next_state.read().unwrap().get_name()).to_string());
        //             });

        //             return self.execute_current_state();
        //         }
        //     }
        // }

        // // Otherwise we evaluate the transitions of the current state
        // let curr_state = self.current_state.clone().unwrap();

        // let state_value_result = curr_state.read();

        // if state_value_result.is_ok() {
        //     let state_value = state_value_result.unwrap();

        //     let tmp_state = self.evaluate_transition(state_value.get_transitions(), event);

        //     if tmp_state > -1 {
        //         let next_state = self.states.get(tmp_state as usize).unwrap();

        //         // Emit transtion occured event
        //         self.observers.read().unwrap().iter().for_each(|observer| {
        //             observer.on_transition(
        //                 (*self
        //                     .current_state
        //                     .as_ref()
        //                     .unwrap()
        //                     .read()
        //                     .unwrap()
        //                     .get_name())
        //                 .to_string(),
        //                 (*next_state.read().unwrap().get_name()).to_string(),
        //             )
        //         });

        //         // Emit leaving current state event
        //         if self.current_state.is_some() {
        //             self.observers.read().unwrap().iter().for_each(|observer| {
        //                 observer.on_state_exit(
        //                     (*self
        //                         .current_state
        //                         .as_ref()
        //                         .unwrap()
        //                         .read()
        //                         .unwrap()
        //                         .get_name())
        //                     .to_string(),
        //                 );
        //             });
        //         }

        //         self.current_state = Some(next_state.clone());

        //         // Emit entering a new state
        //         self.observers.read().unwrap().iter().for_each(|observer| {
        //             observer.on_state_entered((*next_state.read().unwrap().get_name()).to_string());
        //         });

        //         return self.execute_current_state();
        //     }
        // }
    }

    pub fn remove_state(&mut self, state: Rc<RwLock<State>>) {
        let _ = state;
        // self.states.remove(state);
    }
}

unsafe impl Send for StateMachine {}
unsafe impl Sync for StateMachine {}
