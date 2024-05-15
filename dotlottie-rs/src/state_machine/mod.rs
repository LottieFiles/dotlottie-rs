use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

pub mod errors;
pub mod events;
pub mod parser;
pub mod states;
pub mod transitions;

use crate::parser::StringNumberBool;
use crate::state_machine::states::StateTrait;
use crate::state_machine::transitions::guard::Guard;
use crate::state_machine::transitions::TransitionTrait;
use crate::{Config, DotLottiePlayerContainer, Layout, Mode};

use self::parser::{state_machine_parse, ContextJsonType};
use self::{errors::StateMachineError, events::Event, states::State, transitions::Transition};

pub trait StateMachineObserver: Send + Sync {
    fn transition_occured(&self, previous_state: &State, new_state: &State);
    fn on_state_entered(&self, entering_state: &State);
    fn on_state_exit(&self, leaving_state: &State);
}

#[derive(PartialEq)]
pub enum StateMachineStatus {
    Running,
    Paused,
    Stopped,
}

pub struct StateMachine {
    pub states: Vec<Arc<RwLock<State>>>,
    pub current_state: Option<Arc<RwLock<State>>>,
    pub player: Option<Rc<RwLock<DotLottiePlayerContainer>>>,
    pub status: StateMachineStatus,

    numeric_context: HashMap<String, f32>,
    string_context: HashMap<String, String>,
    bool_context: HashMap<String, bool>,

    observers: RwLock<Vec<Arc<dyn StateMachineObserver>>>,
}

impl StateMachine {
    pub fn default() -> StateMachine {
        StateMachine {
            states: Vec::new(),
            current_state: None,
            player: None,
            numeric_context: HashMap::new(),
            string_context: HashMap::new(),
            bool_context: HashMap::new(),
            status: StateMachineStatus::Stopped,
            observers: RwLock::new(Vec::new()),
        }
    }

    pub fn new(
        state_machine_definition: &str,
        player: Rc<RwLock<DotLottiePlayerContainer>>,
    ) -> Result<StateMachine, StateMachineError> {
        let mut state_machine = StateMachine {
            states: Vec::new(),
            current_state: None,
            player: Some(player.clone()),
            numeric_context: HashMap::new(),
            string_context: HashMap::new(),
            bool_context: HashMap::new(),
            status: StateMachineStatus::Stopped,
            observers: RwLock::new(Vec::new()),
        };

        let sm = state_machine.create_state_machine(state_machine_definition, &player);

        match sm {
            Ok(sm) => {
                return Ok(sm);
            }
            Err(err) => {
                return Err(err);
            }
        };
    }

    pub fn subscribe(&self, observer: Arc<dyn StateMachineObserver>) {
        let mut observers = self.observers.write().unwrap();
        observers.push(observer);
    }

    pub fn unsubscribe(&self, observer: &Arc<dyn StateMachineObserver>) {
        self.observers
            .write()
            .unwrap()
            .retain(|o| !Arc::ptr_eq(o, observer));
    }

    pub fn get_numeric_context(&self, key: &str) -> Option<f32> {
        self.numeric_context.get(key).cloned()
    }

    pub fn get_string_context(&self, key: &str) -> Option<String> {
        self.string_context.get(key).cloned()
    }

    pub fn get_bool_context(&self, key: &str) -> Option<bool> {
        self.bool_context.get(key).cloned()
    }

    pub fn set_numeric_context(&mut self, key: &str, value: f32) {
        self.numeric_context.insert(key.to_string(), value);
    }

    pub fn set_string_context(&mut self, key: &str, value: &str) {
        self.string_context
            .insert(key.to_string(), value.to_string());
    }

    pub fn set_bool_context(&mut self, key: &str, value: bool) {
        self.bool_context.insert(key.to_string(), value);
    }

    // Parses the JSON of the state machine definition and creates the states and transitions
    pub fn create_state_machine(
        &mut self,
        sm_definition: &str,
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
    ) -> Result<StateMachine, StateMachineError> {
        let parsed_state_machine = state_machine_parse(sm_definition);

        // todo somehow get the context json without having to parse it again
        // self.json_context = Some(
        //     state_machine_parse(sm_definition)
        //         .unwrap()
        //         .context_variables,
        // );

        let mut states: Vec<Arc<RwLock<State>>> = Vec::new();
        let mut new_state_machine = StateMachine::default();

        match parsed_state_machine {
            Ok(parsed_state_machine) => {
                // Loop through result json states and create objects for each
                for state in parsed_state_machine.states {
                    match state.r#type {
                        parser::StateType::PlaybackState => {
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
                                mode,
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
                                reset_context: state.reset_context.unwrap_or("".to_string()),
                                animation_id: state.animation_id.unwrap_or("".to_string()),
                                width: 1920,
                                height: 1080,
                                transitions: Vec::new(),
                            };

                            states.push(Arc::new(RwLock::new(new_playback_state)));
                        }
                        parser::StateType::SyncState => {}
                        parser::StateType::FinalState => {}
                        parser::StateType::GlobalState => {}
                    }
                }

                // Loop through result transitions and create objects for each
                for transition in parsed_state_machine.transitions {
                    match transition.r#type {
                        parser::TransitionJsonType::Transition => {
                            let target_state_index = transition.to_state;
                            let mut guards_for_transition: Vec<Guard> = Vec::new();

                            // Use the provided index to get the state in the vec we've built
                            if target_state_index >= states.len() as u32 {
                                return Err(StateMachineError::ParsingError {
                                    reason: "Transition has an invalid target state index value!"
                                        .to_string(),
                                });
                            }

                            // Loop through transition guards and create equivalent Guard objects
                            if transition.guards.is_some() {
                                let guards = transition.guards.unwrap();

                                for guard in guards {
                                    let new_guard = Guard {
                                        context_key: guard.context_key,
                                        condition_type: guard.condition_type,
                                        compare_to: guard.compare_to,
                                    };

                                    guards_for_transition.push(new_guard);
                                }
                            }

                            // let mut new_transition: Option<Transition> = None;
                            let mut state_to_attach_to: i32 = -1;
                            let mut new_event: Option<Event> = None;

                            // Capture which event this transition has
                            if transition.numeric_event.is_some() {
                                let numeric_event = transition.numeric_event.unwrap();
                                new_event = Some(Event::Numeric {
                                    value: numeric_event.value,
                                });
                                state_to_attach_to = transition.from_state as i32;
                            } else if transition.string_event.is_some() {
                                let string_event = transition.string_event.unwrap();
                                new_event = Some(Event::String {
                                    value: string_event.value,
                                });
                                state_to_attach_to = transition.from_state as i32;
                            } else if transition.boolean_event.is_some() {
                                let boolean_event = transition.boolean_event.unwrap();
                                new_event = Some(Event::Bool {
                                    value: boolean_event.value,
                                });
                                state_to_attach_to = transition.from_state as i32;
                            } else if transition.on_complete_event.is_some() {
                            } else if transition.on_pointer_down_event.is_some() {
                            } else if transition.on_pointer_up_event.is_some() {
                            } else if transition.on_pointer_enter_event.is_some() {
                            } else if transition.on_pointer_exit_event.is_some() {
                            } else if transition.on_pointer_move_event.is_some() {
                            }
                            // Todo - Add the rest of the event types

                            match new_event {
                                Some(event) => {
                                    let new_transition = Transition::Transition {
                                        target_state: target_state_index,
                                        event: Arc::new(RwLock::new(event)),
                                        guards: guards_for_transition,
                                    };

                                    // Since the target is valid and transition created, we attach it to the state
                                    if state_to_attach_to < states.len() as i32 {
                                        states[state_to_attach_to as usize]
                                            .write()
                                            .unwrap()
                                            .add_transition(new_transition);
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                }

                // Since value can either be a string, int or bool, we need to check the type and set the context accordingly
                for variable in parsed_state_machine.context_variables {
                    match variable.r#type {
                        ContextJsonType::Numeric => match variable.value {
                            StringNumberBool::F32(value) => {
                                new_state_machine.set_numeric_context(&variable.key, value);
                            }
                            _ => {}
                        },
                        ContextJsonType::String => match variable.value {
                            StringNumberBool::String(value) => {
                                new_state_machine.set_string_context(&variable.key, value.as_str());
                            }
                            _ => {}
                        },
                        ContextJsonType::Boolean => match variable.value {
                            StringNumberBool::Bool(value) => {
                                new_state_machine.set_bool_context(&variable.key, value);
                            }
                            _ => {}
                        },
                    }
                }

                // Since value can either be a string, int or bool, we need to check the type and set the context accordingly
                for variable in parsed_state_machine.context_variables {
                    match variable.r#type {
                        ContextJsonType::Numeric => match variable.value {
                            StringNumberBool::F32(value) => {
                                new_state_machine.set_numeric_context(&variable.key, value);
                            }
                            _ => {}
                        },
                        ContextJsonType::String => match variable.value {
                            StringNumberBool::String(value) => {
                                new_state_machine.set_string_context(&variable.key, value.as_str());
                            }
                            _ => {}
                        },
                        ContextJsonType::Boolean => match variable.value {
                            StringNumberBool::Bool(value) => {
                                new_state_machine.set_bool_context(&variable.key, value);
                            }
                            _ => {}
                        },
                    }
                }

                let mut initial_state = None;

                // All states and transitions have been created, we can set the state machine's initial state
                let initial_state_index = parsed_state_machine.descriptor.initial;

                if initial_state_index < states.len() as u32 {
                    initial_state = Some(states[initial_state_index as usize].clone());
                }

                new_state_machine = StateMachine {
                    states,
                    current_state: initial_state,
                    player: Some(player.clone()),
                    numeric_context: new_state_machine.numeric_context,
                    string_context: new_state_machine.string_context,
                    bool_context: new_state_machine.bool_context,
                    status: StateMachineStatus::Stopped,
                    observers: RwLock::new(Vec::new()),
                };

                return Ok(new_state_machine);
            }
            Err(error) => return Err(error),
        }
    }

    pub fn start(&mut self) {
        self.status = StateMachineStatus::Running;
        self.execute_current_state()
    }

    pub fn pause(&mut self) {
        self.status = StateMachineStatus::Paused;
    }

    pub fn end(&mut self) {
        self.status = StateMachineStatus::Stopped;
    }

    pub fn set_initial_state(&mut self, state: Arc<RwLock<State>>) {
        self.current_state = Some(state);
    }

    pub fn get_current_state(&self) -> Option<Arc<RwLock<State>>> {
        self.current_state.clone()
    }

    pub fn add_state(&mut self, state: Arc<RwLock<State>>) {
        self.states.push(state);
    }

    pub fn execute_current_state(&mut self) {
        if self.current_state.is_none() {
            return;
        }

        // Check if current_state is not None and execute the state
        match self.current_state {
            Some(ref state) => {
                let mut unwrapped_state = state.write().unwrap();
                let reset_key = unwrapped_state.get_reset_context_key();

                if reset_key.len() > 0 {
                    if reset_key == "*" {
                        // Todo dont clear reset to their original values from file
                        // self.numeric_context.clear();
                        // self.string_context.clear();
                        // self.bool_context.clear();
                    } else {
                        if self.numeric_context.contains_key(reset_key) {
                            // self.numeric_context.remove(reset_key);
                        }

                        if self.string_context.contains_key(reset_key) {
                            // self.string_context.remove(reset_key);
                        }

                        if self.bool_context.contains_key(reset_key) {
                            // self.bool_context.remove(reset_key);
                        }
                    }
                }

                if self.player.is_some() {
                    unwrapped_state.execute(&self.player.as_mut().unwrap());
                }
            }
            None => {}
        }
    }

    fn verify_if_guards_are_met(&mut self, guard: &Guard) -> bool {
        match guard.compare_to {
            StringNumberBool::String(_) => {
                if guard.string_context_is_satisfied(&self.string_context) {
                    return true;
                }
            }
            StringNumberBool::F32(_) => {
                if guard.numeric_context_is_satisfied(&self.numeric_context) {
                    return true;
                }
            }
            StringNumberBool::Bool(_) => {
                if guard.bool_context_is_satisfied(&self.bool_context) {
                    return true;
                }
            }
        }

        false
    }

    pub fn post_event(&mut self, event: &Event) {
        if self.status == StateMachineStatus::Stopped || self.status == StateMachineStatus::Paused {
            return;
        }

        let mut string_event = false;
        let mut numeric_event = false;
        let mut bool_event = false;

        match event {
            Event::Bool { value: _ } => bool_event = true,
            Event::String { value: _ } => string_event = true,
            Event::Numeric { value: _ } => numeric_event = true,
            Event::OnPointerDown { x: _, y: _ } => {
                println!(">> OnPointerDownEvent");
            }
            Event::OnPointerUp { x: _, y: _ } => {
                println!(">> OnPointerUpEvent");
            }
            Event::OnPointerMove { x: _, y: _ } => {
                println!(">> OnPointerMoveEvent");
            }
            Event::OnPointerEnter { x: _, y: _ } => {
                println!(">> OnPointerEnterEvent");
            }
            Event::OnPointerExit => {
                println!(">> OnPointerExitEvent");
            }
        }

        if self.current_state.is_none() {
            return;
        }

        let curr_state = self.current_state.clone().unwrap();
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
                        let target_state = unwrapped_transition.get_target_state();
                        let transition = &*unwrapped_transition;
                        let event_lock = transition.get_event();
                        let event_data = event_lock.read().unwrap();
                        let transition_event = &*event_data;
                        let transition_guards = transition.get_guards();

                        // Match the transition's event type and compare it to the received event
                        match transition_event {
                            Event::Bool { value } => {
                                let mut received_event_value = false;

                                match event {
                                    Event::Bool { value } => {
                                        received_event_value = *value;
                                    }
                                    _ => {}
                                }

                                // Check the transitions value and compare to the received one to check if we should transition
                                if bool_event && received_event_value == *value {
                                    // If there are guards loop over them and check if theyre verified
                                    if transition_guards.len() > 0 {
                                        for guard in transition_guards {
                                            if self.verify_if_guards_are_met(guard) {
                                                tmp_state = target_state as i32;
                                            }
                                        }
                                    } else {
                                        tmp_state = target_state as i32;
                                    }
                                }
                            }
                            Event::String { value } => {
                                let mut received_event_value = "";

                                match event {
                                    Event::String { value } => {
                                        received_event_value = value;
                                    }
                                    _ => {}
                                }

                                if string_event && received_event_value == value {
                                    // If there are guards loop over them and check if theyre verified
                                    if transition_guards.len() > 0 {
                                        for guard in transition_guards {
                                            if self.verify_if_guards_are_met(guard) {
                                                tmp_state = target_state as i32;
                                            }
                                        }
                                    } else {
                                        tmp_state = target_state as i32;
                                    }
                                }
                            }
                            Event::Numeric { value } => {
                                let mut received_event_value = 0.0;

                                match event {
                                    Event::Numeric { value } => {
                                        received_event_value = *value;
                                    }
                                    _ => {}
                                }

                                if numeric_event && received_event_value == *value {
                                    // If there are guards loop over them and check if theyre verified
                                    if transition_guards.len() > 0 {
                                        for guard in transition_guards {
                                            if self.verify_if_guards_are_met(guard) {
                                                tmp_state = target_state as i32;
                                            }
                                        }
                                    } else {
                                        tmp_state = target_state as i32;
                                    }
                                }
                            }
                            Event::OnPointerDown { x: _, y: _ } => todo!(),
                            Event::OnPointerUp { x: _, y: _ } => todo!(),
                            Event::OnPointerMove { x: _, y: _ } => todo!(),
                            Event::OnPointerEnter { x: _, y: _ } => todo!(),
                            Event::OnPointerExit => todo!(),
                        }
                    }
                    None => break,
                }
            }

            if tmp_state > -1 {
                let next_state = self.states.get(tmp_state as usize).unwrap();

                // Emit transtion occured event
                self.observers.read().unwrap().iter().for_each(|observer| {
                    observer.transition_occured(
                        &*self.current_state.as_ref().unwrap().read().unwrap(),
                        &*next_state.read().unwrap(),
                    )
                });

                // Emit leaving current state event
                if self.current_state.is_some() {
                    self.observers.read().unwrap().iter().for_each(|observer| {
                        observer
                            .on_state_exit(&*self.current_state.as_ref().unwrap().read().unwrap());
                    });
                }

                self.current_state = Some(next_state.clone());

                // Emit entering a new state
                self.observers.read().unwrap().iter().for_each(|observer| {
                    observer.on_state_entered(&*next_state.read().unwrap());
                });

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
