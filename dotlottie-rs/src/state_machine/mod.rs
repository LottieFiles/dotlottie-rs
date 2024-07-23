use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

pub mod errors;
pub mod events;
pub mod listeners;
pub mod parser;
pub mod states;
pub mod transitions;

use crate::parser::StringNumberBool;
use crate::state_machine::listeners::Listener;
use crate::state_machine::states::StateTrait;
use crate::state_machine::transitions::guard::Guard;
use crate::state_machine::transitions::TransitionTrait;
use crate::{Config, DotLottiePlayerContainer, Layout, Mode};

use self::parser::{state_machine_parse, ContextJsonType};
use self::{errors::StateMachineError, events::Event, states::State, transitions::Transition};

pub trait StateMachineObserver: Send + Sync {
    fn on_transition(&self, previous_state: String, new_state: String);
    fn on_state_entered(&self, entering_state: String);
    fn on_state_exit(&self, leaving_state: String);
}

#[derive(PartialEq)]
pub enum StateMachineStatus {
    Running,
    Paused,
    Stopped,
}

pub struct StateMachine {
    pub states: Vec<Arc<RwLock<State>>>,
    pub listeners: Vec<Arc<RwLock<Listener>>>,
    pub current_state: Option<Arc<RwLock<State>>>,
    pub player: Option<Rc<RwLock<DotLottiePlayerContainer>>>,
    pub status: StateMachineStatus,

    numeric_context: HashMap<String, f32>,
    string_context: HashMap<String, String>,
    bool_context: HashMap<String, bool>,

    observers: RwLock<Vec<Arc<dyn StateMachineObserver>>>,
}

impl Default for StateMachine {
    fn default() -> StateMachine {
        StateMachine {
            states: Vec::new(),
            listeners: Vec::new(),
            current_state: None,
            player: None,
            numeric_context: HashMap::new(),
            string_context: HashMap::new(),
            bool_context: HashMap::new(),
            status: StateMachineStatus::Stopped,
            observers: RwLock::new(Vec::new()),
        }
    }
}

impl StateMachine {
    pub fn new(
        state_machine_definition: &str,
        player: Rc<RwLock<DotLottiePlayerContainer>>,
    ) -> Result<StateMachine, StateMachineError> {
        let mut state_machine = StateMachine {
            states: Vec::new(),
            listeners: Vec::new(),
            current_state: None,
            player: Some(player.clone()),
            numeric_context: HashMap::new(),
            string_context: HashMap::new(),
            bool_context: HashMap::new(),
            status: StateMachineStatus::Stopped,
            observers: RwLock::new(Vec::new()),
        };

        state_machine.create_state_machine(state_machine_definition, &player)
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

        let s = self.current_state.clone();

        // If current state is a sync state, we need to update the frame
        if let Some(state) = s {
            let unwrapped_state = state.try_read();

            if let Ok(state) = unwrapped_state {
                let state_value = &*state;

                if let State::Sync { .. } = state_value {
                    self.execute_current_state();
                }
            }
        }
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
        let mut listeners: Vec<Arc<RwLock<Listener>>> = Vec::new();
        let mut new_state_machine = StateMachine::default();

        match parsed_state_machine {
            Ok(parsed_state_machine) => {
                // Loop through result json states and create objects for each
                for state in parsed_state_machine.states {
                    match state {
                        parser::StateJson::PlaybackState {
                            name,
                            animation_id,
                            r#loop,
                            autoplay,
                            mode,
                            speed,
                            segment,
                            background_color,
                            use_frame_interpolation,
                            reset_context,
                            marker,
                            ..
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
                                segment: segment.unwrap_or(default_config.segment),
                                background_color: background_color
                                    .unwrap_or(default_config.background_color),
                                layout: Layout::default(),
                                marker: marker.unwrap_or(default_config.marker),
                            };

                            // Construct a State with the values we've gathered
                            let new_playback_state = State::Playback {
                                name,
                                config: playback_config,
                                reset_context: reset_context.unwrap_or("".to_string()),
                                animation_id: animation_id.unwrap_or("".to_string()),
                                transitions: Vec::new(),
                            };

                            states.push(Arc::new(RwLock::new(new_playback_state)));
                        }
                        parser::StateJson::SyncState {
                            name,
                            animation_id,
                            background_color,
                            reset_context,
                            frame_context_key,
                            segment,
                            ..
                        } => {
                            let mut config = Config::default();

                            config.background_color =
                                background_color.unwrap_or(config.background_color);
                            config.segment = segment.unwrap_or(config.segment);

                            let new_sync_state = State::Sync {
                                name,
                                frame_context_key,
                                reset_context: reset_context.unwrap_or("".to_string()),
                                animation_id: animation_id.unwrap_or("".to_string()),
                                transitions: Vec::new(),
                                config,
                            };

                            states.push(Arc::new(RwLock::new(new_sync_state)));
                        }
                    }
                }

                // Loop through result transitions and create objects for each
                for transition in parsed_state_machine.transitions {
                    match transition {
                        parser::TransitionJson::Transition {
                            from_state,
                            to_state,
                            guards,
                            numeric_event,
                            string_event,
                            boolean_event,
                            on_complete_event,
                            on_pointer_down_event,
                            on_pointer_up_event,
                            on_pointer_enter_event,
                            on_pointer_exit_event,
                            on_pointer_move_event,
                        } => {
                            let target_state_index = to_state;
                            let mut guards_for_transition: Vec<Guard> = Vec::new();

                            // Use the provided index to get the state in the vec we've built
                            if target_state_index >= states.len() as u32 {
                                return Err(StateMachineError::ParsingError {
                                    reason: "Transition has an invalid target state index value!"
                                        .to_string(),
                                });
                            }

                            // Loop through transition guards and create equivalent Guard objects
                            if guards.is_some() {
                                let guards = guards.unwrap();

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
                            if numeric_event.is_some() {
                                let numeric_event = numeric_event.unwrap();
                                new_event = Some(Event::Numeric {
                                    value: numeric_event.value,
                                });
                                state_to_attach_to = from_state as i32;
                            } else if string_event.is_some() {
                                let string_event = string_event.unwrap();
                                new_event = Some(Event::String {
                                    value: string_event.value,
                                });
                                state_to_attach_to = from_state as i32;
                            } else if boolean_event.is_some() {
                                let boolean_event = boolean_event.unwrap();
                                new_event = Some(Event::Bool {
                                    value: boolean_event.value,
                                });
                                state_to_attach_to = from_state as i32;
                            } else if on_complete_event.is_some() {
                                new_event = Some(Event::OnComplete);
                                state_to_attach_to = from_state as i32;
                            } else if on_pointer_down_event.is_some() {
                                // Default to 0.0 0.0 coordinates
                                // How to manage targets?
                                // let pointer_down_event = on_pointer_down_event.unwrap();
                                // pointer_down_event.target;
                                new_event = Some(Event::OnPointerDown { x: 0.0, y: 0.0 });
                                state_to_attach_to = from_state as i32;
                            } else if on_pointer_up_event.is_some() {
                                // Default to 0.0 0.0 coordinates
                                // How to manage targets?
                                new_event = Some(Event::OnPointerUp { x: 0.0, y: 0.0 });
                                state_to_attach_to = from_state as i32;
                            } else if on_pointer_enter_event.is_some() {
                                // Default to 0.0 0.0 coordinates
                                // How to manage targets?
                                new_event = Some(Event::OnPointerEnter { x: 0.0, y: 0.0 });
                                state_to_attach_to = from_state as i32;
                            } else if on_pointer_exit_event.is_some() {
                                new_event = Some(Event::OnPointerExit {});
                                state_to_attach_to = from_state as i32;
                            } else if on_pointer_move_event.is_some() {
                                // Default to 0.0 0.0 coordinates
                                // How to manage targets?
                                new_event = Some(Event::OnPointerMove { x: 0.0, y: 0.0 });
                                state_to_attach_to = from_state as i32;
                            }
                            if let Some(event) = new_event {
                                let new_transition = Transition::Transition {
                                    target_state: target_state_index,
                                    event: Arc::new(RwLock::new(event)),
                                    guards: guards_for_transition,
                                };

                                // Since the target is valid and transition created, we attach it to the state
                                if state_to_attach_to < states.len() as i32 {
                                    let try_write_state =
                                        states[state_to_attach_to as usize].try_write();

                                    try_write_state
                                        .map_err(|_| StateMachineError::ParsingError {
                                            reason: "Failed to write to state".to_string(),
                                        })?
                                        .add_transition(new_transition);
                                }
                            }
                        }
                    }
                }

                for listener in parsed_state_machine.listeners {
                    match listener.r#type {
                        parser::ListenerJsonType::PointerUp => {
                            let new_listener = Listener::PointerUp {
                                r#type: listeners::ListenerType::PointerUp,
                                target: listener.target,
                                action: listener.action,
                                value: listener.value,
                                context_key: listener.context_key,
                            };

                            listeners.push(Arc::new(RwLock::new(new_listener)));
                        }
                        parser::ListenerJsonType::PointerDown => {
                            let new_listener = Listener::PointerDown {
                                r#type: listeners::ListenerType::PointerDown,
                                target: listener.target,
                                action: listener.action,
                                value: listener.value,
                                context_key: listener.context_key,
                            };

                            listeners.push(Arc::new(RwLock::new(new_listener)));
                        }
                        parser::ListenerJsonType::PointerEnter => {
                            let new_listener = Listener::PointerEnter {
                                r#type: listeners::ListenerType::PointerEnter,
                                target: listener.target,
                                action: listener.action,
                                value: listener.value,
                                context_key: listener.context_key,
                            };

                            listeners.push(Arc::new(RwLock::new(new_listener)));
                        }
                        parser::ListenerJsonType::PointerExit => {
                            let new_listener = Listener::PointerExit {
                                r#type: listeners::ListenerType::PointerExit,
                                target: listener.target,
                                action: listener.action,
                                value: listener.value,
                                context_key: listener.context_key,
                            };

                            listeners.push(Arc::new(RwLock::new(new_listener)));
                        }
                        parser::ListenerJsonType::PointerMove => {
                            let new_listener = Listener::PointerMove {
                                r#type: listeners::ListenerType::PointerMove,
                                target: listener.target,
                                action: listener.action,
                                value: listener.value,
                                context_key: listener.context_key,
                            };

                            listeners.push(Arc::new(RwLock::new(new_listener)));
                        }
                    }
                }

                // Since value can either be a string, int or bool, we need to check the type and set the context accordingly
                for variable in parsed_state_machine.context_variables {
                    match variable.r#type {
                        ContextJsonType::Numeric => {
                            if let StringNumberBool::F32(value) = variable.value {
                                new_state_machine.set_numeric_context(&variable.key, value);
                            }
                        }
                        ContextJsonType::String => {
                            if let StringNumberBool::String(value) = variable.value {
                                new_state_machine.set_string_context(&variable.key, value.as_str());
                            }
                        }
                        ContextJsonType::Boolean => {
                            if let StringNumberBool::Bool(value) = variable.value {
                                new_state_machine.set_bool_context(&variable.key, value);
                            }
                        }
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
                    listeners,
                    current_state: initial_state,
                    player: Some(player.clone()),
                    numeric_context: new_state_machine.numeric_context,
                    string_context: new_state_machine.string_context,
                    bool_context: new_state_machine.bool_context,
                    status: StateMachineStatus::Stopped,
                    observers: RwLock::new(Vec::new()),
                };

                Ok(new_state_machine)
            }
            Err(error) => Err(error),
        }
    }

    pub fn start(&mut self) {
        self.status = StateMachineStatus::Running;
        self.execute_current_state();
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

    pub fn get_listeners(&self) -> &Vec<Arc<RwLock<Listener>>> {
        &self.listeners
    }

    pub fn execute_current_state(&mut self) -> bool {
        if self.current_state.is_none() {
            return false;
        }

        // Check if current_state is not None and execute the state
        if let Some(ref state) = self.current_state {
            let unwrapped_state = state.read().unwrap();
            let reset_key = unwrapped_state.get_reset_context_key();

            if !reset_key.is_empty() {
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
                unwrapped_state.execute(
                    self.player.as_mut().unwrap(),
                    &self.string_context,
                    &self.bool_context,
                    &self.numeric_context,
                );
            }
        }

        true
    }

    fn verify_if_guards_are_met(&self, guard: &Guard) -> bool {
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
        let mut complete_event = false;
        let mut pointer_down_event = false;
        let mut pointer_up_event = false;
        let mut pointer_move_event = false;
        let mut pointer_enter_event = false;
        let mut pointer_exit_event = false;

        match event {
            Event::Bool { value: _ } => bool_event = true,
            Event::String { value: _ } => string_event = true,
            Event::Numeric { value: _ } => numeric_event = true,
            Event::OnPointerDown { x: _, y: _ } => pointer_down_event = true,
            Event::OnPointerUp { x: _, y: _ } => pointer_up_event = true,
            Event::OnPointerMove { x: _, y: _ } => pointer_move_event = true,
            Event::OnPointerEnter { x: _, y: _ } => pointer_enter_event = true,
            Event::OnPointerExit => pointer_exit_event = true,
            Event::OnComplete => complete_event = true,
        }

        if self.current_state.is_none() {
            return;
        }

        let curr_state = self.current_state.clone().unwrap();

        let state_value_result = curr_state.read();

        if state_value_result.is_ok() {
            let state_value = state_value_result.unwrap();
            let iter = state_value.get_transitions().iter();
            let mut tmp_state: i32 = -1;

            for transition in iter {
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

                        if let Event::Bool { value } = event {
                            received_event_value = *value;
                        }

                        // Check the transitions value and compare to the received one to check if we should transition
                        if bool_event && received_event_value == *value {
                            // If there are guards loop over them and check if theyre verified
                            if !transition_guards.is_empty() {
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

                        if let Event::String { value } = event {
                            received_event_value = value;
                        }

                        if string_event && received_event_value == value {
                            // If there are guards loop over them and check if theyre verified
                            if !transition_guards.is_empty() {
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

                        if let Event::Numeric { value } = event {
                            received_event_value = *value;
                        }

                        if numeric_event && received_event_value == *value {
                            // If there are guards loop over them and check if theyre verified
                            if !transition_guards.is_empty() {
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
                    Event::OnComplete => {
                        if complete_event {
                            // If there are guards loop over them and check if theyre verified
                            if !transition_guards.is_empty() {
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
                    Event::OnPointerDown { x: _, y: _ } => {
                        if pointer_down_event {
                            // If there are guards loop over them and check if theyre verified
                            if !transition_guards.is_empty() {
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
                    Event::OnPointerUp { x: _, y: _ } => {
                        if pointer_up_event {
                            // If there are guards loop over them and check if theyre verified
                            if !transition_guards.is_empty() {
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
                    Event::OnPointerMove { x: _, y: _ } => {
                        if pointer_move_event {
                            // If there are guards loop over them and check if theyre verified
                            if !transition_guards.is_empty() {
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
                    Event::OnPointerEnter { x: _, y: _ } => {
                        if pointer_enter_event {
                            // If there are guards loop over them and check if theyre verified
                            if !transition_guards.is_empty() {
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
                    Event::OnPointerExit => {
                        if pointer_exit_event {
                            // If there are guards loop over them and check if theyre verified
                            if !transition_guards.is_empty() {
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
                }
            }

            if tmp_state > -1 {
                let next_state = self.states.get(tmp_state as usize).unwrap();

                // Emit transtion occured event
                self.observers.read().unwrap().iter().for_each(|observer| {
                    observer.on_transition(
                        (*self
                            .current_state
                            .as_ref()
                            .unwrap()
                            .read()
                            .unwrap()
                            .get_name())
                        .to_string(),
                        (*next_state.read().unwrap().get_name()).to_string(),
                    )
                });

                // Emit leaving current state event
                if self.current_state.is_some() {
                    self.observers.read().unwrap().iter().for_each(|observer| {
                        observer.on_state_exit(
                            (*self
                                .current_state
                                .as_ref()
                                .unwrap()
                                .read()
                                .unwrap()
                                .get_name())
                            .to_string(),
                        );
                    });
                }

                self.current_state = Some(next_state.clone());

                // Emit entering a new state
                self.observers.read().unwrap().iter().for_each(|observer| {
                    observer.on_state_entered((*next_state.read().unwrap().get_name()).to_string());
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
