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
use crate::{Config, DotLottiePlayerContainer, InternalEvent, Layout, Mode, PointerEvent};

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
    pub global_state: Option<Arc<RwLock<State>>>,
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
            global_state: None,
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
            global_state: None,
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
        let mut global_state: Option<Arc<RwLock<State>>> = None;
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
                                theme_id: String::from(""),
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
                        parser::StateJson::GlobalState {
                            name,
                            reset_context,
                            entry_actions: _,
                            exit_actions: _,
                        } => {
                            let new_global_state = State::Global {
                                name,
                                reset_context: reset_context.unwrap_or("".to_string()),
                                transitions: Vec::new(),
                            };

                            let locked_global_state = Arc::new(RwLock::new(new_global_state));

                            global_state = Some(locked_global_state.clone());

                            states.push(locked_global_state);
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
                            let mut new_event: Option<InternalEvent> = None;

                            // Capture which event this transition has
                            if numeric_event.is_some() {
                                let numeric_event = numeric_event.unwrap();
                                new_event = Some(InternalEvent::Numeric {
                                    value: numeric_event.value,
                                });
                                state_to_attach_to = from_state as i32;
                            } else if string_event.is_some() {
                                let string_event = string_event.unwrap();
                                new_event = Some(InternalEvent::String {
                                    value: string_event.value,
                                });
                                state_to_attach_to = from_state as i32;
                            } else if boolean_event.is_some() {
                                let boolean_event = boolean_event.unwrap();
                                new_event = Some(InternalEvent::Bool {
                                    value: boolean_event.value,
                                });
                                state_to_attach_to = from_state as i32;
                            } else if on_complete_event.is_some() {
                                new_event = Some(InternalEvent::OnComplete);
                                state_to_attach_to = from_state as i32;
                            } else if on_pointer_down_event.is_some() {
                                // Default to 0.0 0.0 coordinates
                                let pointer_down_event = on_pointer_down_event.unwrap();

                                if pointer_down_event.target.is_some() {
                                    new_event = Some(InternalEvent::OnPointerDown {
                                        target: pointer_down_event.target,
                                    });
                                } else {
                                    new_event = Some(InternalEvent::OnPointerDown { target: None });
                                }

                                state_to_attach_to = from_state as i32;
                            } else if on_pointer_up_event.is_some() {
                                // Default to 0.0 0.0 coordinates

                                let pointer_up_event = on_pointer_up_event.unwrap();

                                if pointer_up_event.target.is_some() {
                                    new_event = Some(InternalEvent::OnPointerUp {
                                        target: pointer_up_event.target,
                                    });
                                } else {
                                    new_event = Some(InternalEvent::OnPointerUp { target: None });
                                }

                                state_to_attach_to = from_state as i32;
                            } else if on_pointer_enter_event.is_some() {
                                // Default to 0.0 0.0 coordinates
                                let pointer_enter_event = on_pointer_enter_event.unwrap();

                                if pointer_enter_event.target.is_some() {
                                    new_event = Some(InternalEvent::OnPointerEnter {
                                        target: pointer_enter_event.target,
                                    });
                                } else {
                                    new_event =
                                        Some(InternalEvent::OnPointerEnter { target: None });
                                }

                                state_to_attach_to = from_state as i32;
                            } else if on_pointer_exit_event.is_some() {
                                // Default to 0.0 0.0 coordinates
                                let pointer_exit_event = on_pointer_exit_event.unwrap();

                                if pointer_exit_event.target.is_some() {
                                    new_event = Some(InternalEvent::OnPointerExit {
                                        target: pointer_exit_event.target,
                                    });
                                } else {
                                    new_event = Some(InternalEvent::OnPointerExit { target: None });
                                }

                                state_to_attach_to = from_state as i32;
                            } else if on_pointer_move_event.is_some() {
                                // Default to 0.0 0.0 coordinates
                                let pointer_move_event = on_pointer_move_event.unwrap();

                                if pointer_move_event.target.is_some() {
                                    new_event = Some(InternalEvent::OnPointerMove {
                                        target: pointer_move_event.target,
                                    });
                                } else {
                                    new_event = Some(InternalEvent::OnPointerMove { target: None });
                                }

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
                    global_state,
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

    // Return codes
    // 0: Success
    // 1: Failure
    // 2: Play animation
    // 3: Pause animation
    // 4: Request and draw a new single frame of the animation (needed for sync state)
    pub fn execute_current_state(&mut self) -> i32 {
        if self.current_state.is_none() {
            return 1;
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
                return unwrapped_state.execute(
                    self.player.as_mut().unwrap(),
                    &self.string_context,
                    &self.bool_context,
                    &self.numeric_context,
                );
            } else {
                return 1;
            }
        }

        0
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

    fn perform_hit_check(&self, x: f32, y: f32, target: &str) -> bool {
        // A layer name was provided, we need to check if the pointer is within the layer
        let pointer_target = target;

        let player_ref = self.player.as_ref();

        if player_ref.is_some() {
            let player = player_ref.unwrap();
            let player_read = player.try_read();

            match player_read {
                Ok(player) => {
                    let player = &*player;

                    player.intersect(x, y, pointer_target)
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    fn evaluate_transition(&self, transitions: &[Arc<RwLock<Transition>>], event: &Event) -> i32 {
        let mut tmp_state: i32 = -1;
        let iter = transitions.iter();

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
                InternalEvent::Bool { value } => {
                    let bool_value = value;

                    if let Event::Bool { value } = event {
                        if *value == *bool_value {
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
                InternalEvent::String { value } => {
                    let string_value = value;

                    if let Event::String { value } = event {
                        if string_value == value {
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
                InternalEvent::Numeric { value } => {
                    let num_value = value;

                    if let Event::Numeric { value } = event {
                        if *value == *num_value {
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
                InternalEvent::OnComplete => {
                    if let Event::OnComplete = event {
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
                // This is checking the state machine's event, not the passed event
                InternalEvent::OnPointerDown { target } => {
                    if let Event::OnPointerDown { x, y } = event {
                        // If there are guards loop over them and check if theyre verified
                        if !transition_guards.is_empty() {
                            for guard in transition_guards {
                                if self.verify_if_guards_are_met(guard) {
                                    tmp_state = target_state as i32;
                                }
                            }
                        } else if target.is_some() && self.player.is_some() {
                            if self.perform_hit_check(*x, *y, target.as_ref().unwrap()) {
                                tmp_state = target_state as i32;
                            }
                        } else {
                            tmp_state = target_state as i32;
                        }
                    }
                }
                InternalEvent::OnPointerUp { target } => {
                    if let Event::OnPointerUp { x, y } = event {
                        // If there are guards loop over them and check if theyre verified
                        if !transition_guards.is_empty() {
                            for guard in transition_guards {
                                if self.verify_if_guards_are_met(guard) {
                                    tmp_state = target_state as i32;
                                }
                            }
                        } else if target.is_some() && self.player.is_some() {
                            if self.perform_hit_check(*x, *y, target.as_ref().unwrap()) {
                                tmp_state = target_state as i32;
                            }
                        } else {
                            tmp_state = target_state as i32;
                        }
                    }
                }
                InternalEvent::OnPointerMove { target } => {
                    if let Event::OnPointerMove { x, y } = event {
                        // If there are guards loop over them and check if theyre verified
                        if !transition_guards.is_empty() {
                            for guard in transition_guards {
                                if self.verify_if_guards_are_met(guard) {
                                    tmp_state = target_state as i32;
                                }
                            }
                        } else if target.is_some() && self.player.is_some() {
                            if self.perform_hit_check(*x, *y, target.as_ref().unwrap()) {
                                tmp_state = target_state as i32;
                            }
                        } else {
                            tmp_state = target_state as i32;
                        }
                    }
                }
                InternalEvent::OnPointerEnter { target } => {
                    let mut received_event_values = Event::OnPointerEnter { x: 0.0, y: 0.0 };

                    match event {
                        Event::OnPointerEnter { x, y } => {
                            received_event_values = Event::OnPointerEnter { x: *x, y: *y };
                        }
                        Event::OnPointerMove { x, y } => {
                            received_event_values = Event::OnPointerMove { x: *x, y: *y };
                        }
                        _ => {}
                    }

                    // If there are guards loop over them and check if theyre verified
                    if !transition_guards.is_empty() {
                        for guard in transition_guards {
                            if self.verify_if_guards_are_met(guard) {
                                tmp_state = target_state as i32;
                            }
                        }
                    } else if target.is_some() && self.player.is_some() {
                        if self.perform_hit_check(
                            received_event_values.x(),
                            received_event_values.y(),
                            target.as_ref().unwrap(),
                        ) {
                            let current_state_name =
                                if let Some(current_state) = &self.current_state {
                                    if let Ok(state) = current_state.read() {
                                        state.get_name()
                                    } else {
                                        return -1;
                                    }
                                } else {
                                    return -1;
                                };

                            let target_state_name = if let Some(target_state) =
                                self.states.get(target_state as usize)
                            {
                                if let Ok(state) = target_state.read() {
                                    state.get_name()
                                } else {
                                    return 1; // Handle read lock error
                                }
                            } else {
                                return 1; // Handle invalid index
                            };

                            // This prevent the state from transitioning to itself over and over again
                            if current_state_name != target_state_name {
                                tmp_state = target_state as i32;
                            }
                        }
                    } else {
                        tmp_state = target_state as i32;
                    }
                }
                InternalEvent::OnPointerExit { target } => {
                    let mut received_event_values = Event::OnPointerEnter { x: 0.0, y: 0.0 };

                    match event {
                        Event::OnPointerExit { x, y } => {
                            received_event_values = Event::OnPointerExit { x: *x, y: *y };
                        }
                        Event::OnPointerMove { x, y } => {
                            received_event_values = Event::OnPointerMove { x: *x, y: *y };
                        }
                        _ => {}
                    }

                    // If there are guards loop over them and check if theyre verified
                    if !transition_guards.is_empty() {
                        for guard in transition_guards {
                            if self.verify_if_guards_are_met(guard) {
                                tmp_state = target_state as i32;
                            }
                        }
                    } else if target.is_some() && self.player.is_some() {
                        // Check if current state is the target state
                        let current_state_name = if let Some(current_state) = &self.current_state {
                            if let Ok(state) = current_state.read() {
                                state.get_name()
                            } else {
                                return -1;
                            }
                        } else {
                            return -1;
                        };

                        if current_state_name == *target.as_ref().unwrap()
                            && !self.perform_hit_check(
                                received_event_values.x(),
                                received_event_values.y(),
                                target.as_ref().unwrap(),
                            )
                        {
                            tmp_state = target_state as i32;
                        }
                    } else {
                        // Check if coordinates are outside of the player
                        let (width, height) = self.player.as_ref().unwrap().read().unwrap().size();

                        if received_event_values.x() < 0.0
                            || received_event_values.x() > width as f32
                            || received_event_values.y() < 0.0
                            || received_event_values.y() > height as f32
                        {
                            tmp_state = target_state as i32;
                        }
                    }
                }
                InternalEvent::SetNumericContext { key: _, value: _ } => {}
            }
        }

        tmp_state
    }

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
        }

        // Only match with setNumericContext as if this is the case we return early
        // Other event types are handled within self.evaluate_transition
        if let Event::SetNumericContext { key, value } = event {
            self.set_numeric_context(key, *value);

            let s = self.current_state.clone();

            // If current state is a sync state, we need to update the frame
            if let Some(state) = s {
                let unwrapped_state = state.try_read();

                if let Ok(state) = unwrapped_state {
                    let state_value = &*state;

                    if let State::Sync { .. } = state_value {
                        return self.execute_current_state();
                    }
                }
            }

            return 0;
        }

        // Firstly check if we have a global state within the state machine.
        if self.global_state.is_some() {
            let global_state = self.global_state.clone().unwrap();
            let global_state_value = global_state.try_read();

            if global_state_value.is_ok() {
                let state_value = global_state_value.unwrap();
                let tmp_state = self.evaluate_transition(state_value.get_transitions(), event);

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
                        observer
                            .on_state_entered((*next_state.read().unwrap().get_name()).to_string());
                    });

                    return self.execute_current_state();
                }
            }
        }

        // Otherwise we evaluate the transitions of the current state
        let curr_state = self.current_state.clone().unwrap();

        let state_value_result = curr_state.read();

        if state_value_result.is_ok() {
            let state_value = state_value_result.unwrap();

            let tmp_state = self.evaluate_transition(state_value.get_transitions(), event);

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

                return self.execute_current_state();
            }
        }

        1
    }

    pub fn remove_state(&mut self, state: Arc<RwLock<State>>) {
        let _ = state;
        // self.states.remove(state);
    }
}

unsafe impl Send for StateMachine {}
unsafe impl Sync for StateMachine {}
