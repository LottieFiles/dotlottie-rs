use core::result::Result::Ok;
use std::collections::HashSet;

pub mod actions;
pub mod errors;
pub mod events;
pub mod inputs;
pub mod interactions;
pub mod security;
pub mod state_machine;
pub mod states;
pub mod transitions;

use actions::open_url_policy::OpenUrlPolicy;
use actions::{Action, ActionTrait};
use inputs::{Input, InputManager, InputTrait, InputValue};
use interactions::InteractionTrait;
use state_machine::StateMachine;
use states::StateTrait;
use transitions::guard::GuardTrait;
use transitions::{Transition, TransitionTrait};

use crate::actions::whitelist::Whitelist;
use crate::events::{EventQueue, StateMachineEvent, StateMachineInternalEvent};
use crate::state_machine_engine::interactions::Interaction;
use crate::{
    event_type_name, state_machine_state_check_pipeline, Config, DotLottiePlayerContainer,
    EventName, PointerEvent, StateMachineEngineSecurityError,
};

use self::state_machine::state_machine_parse;
use self::{events::Event, states::State};

#[derive(PartialEq, Debug)]
pub enum StateMachineEngineStatus {
    Running,
    Tweening,
    Stopped,
}

#[derive(Debug)]
pub enum StateMachineEngineError {
    ParsingError(String),
    CreationError,
    FireEventError,
    InfiniteLoopError,
    NotRunningError,
    SetStateError,
    SecurityCheckErrorMultipleGuardlessTransitions,
    SecurityCheckErrorDuplicateStateName,
}

struct PointerData {
    curr_entered_layer: String,
    listened_layers: Vec<(String, String)>,
    most_recent_event: Option<Event>,
    pointer_x: f32,
    pointer_y: f32,
}

impl Default for PointerData {
    fn default() -> PointerData {
        PointerData {
            curr_entered_layer: "".to_string(),
            listened_layers: Vec::new(),
            most_recent_event: None,
            pointer_x: 0.0,
            pointer_y: 0.0,
        }
    }
}

pub struct StateMachineEngine {
    // For resetting the player config after state machine is stopped
    cached_player_config: Config,

    /* We keep references to the StateMachine's States. */
    /* This prevents duplicating the data inside the engine. */
    pub global_state: Option<State>,
    pub current_state: Option<State>,

    pub status: StateMachineEngineStatus,

    // Open url policy configurations
    pub open_url_requires_user_interaction: bool,
    pub open_url_whitelist: Whitelist,

    pub inputs: InputManager,
    curr_event: Option<String>,

    // PointerEnter/PointerExit management
    pointer_management: PointerData,

    // event queues
    event_queue: EventQueue<StateMachineEvent>,
    internal_event_queue: EventQueue<StateMachineInternalEvent>,

    state_machine: StateMachine,

    state_history: Vec<String>,
    max_cycle_count: usize,
    current_cycle_count: usize,
    action_mutated_inputs: bool,

    // The state to target once blending has finished
    tween_transition_target_state: Option<State>,
}

impl Default for StateMachineEngine {
    fn default() -> StateMachineEngine {
        StateMachineEngine {
            cached_player_config: Config::default(),
            global_state: None,
            state_machine: StateMachine::default(),
            current_state: None,
            open_url_requires_user_interaction: false,
            open_url_whitelist: Whitelist::new(),
            inputs: InputManager::new(),
            curr_event: None,
            pointer_management: PointerData::default(),
            status: StateMachineEngineStatus::Stopped,
            event_queue: EventQueue::new(),
            internal_event_queue: EventQueue::new(),
            state_history: Vec::new(),
            max_cycle_count: 20,
            current_cycle_count: 0,
            action_mutated_inputs: false,
            tween_transition_target_state: None,
        }
    }
}

impl StateMachineEngine {
    pub fn new(
        state_machine_definition: &str,
        player: &mut DotLottiePlayerContainer,
        max_cycle_count: Option<usize>,
    ) -> Result<StateMachineEngine, StateMachineEngineError> {
        // Create an empty state machine object that we'll use to boot up the parser from
        let mut state_machine = StateMachineEngine {
            cached_player_config: Config::default(),
            global_state: None,
            state_machine: StateMachine::default(),
            current_state: None,
            open_url_requires_user_interaction: false,
            open_url_whitelist: Whitelist::new(),
            inputs: InputManager::new(),
            curr_event: None,
            pointer_management: PointerData::default(),
            status: StateMachineEngineStatus::Stopped,
            event_queue: EventQueue::new(),
            internal_event_queue: EventQueue::new(),
            state_history: Vec::new(),
            max_cycle_count: max_cycle_count.unwrap_or(20),
            current_cycle_count: 0,
            action_mutated_inputs: false,
            tween_transition_target_state: None,
        };

        state_machine.create_state_machine(state_machine_definition, player)
    }

    /// Poll for the next state machine event
    ///
    /// Returns Some(event) if an event is available, None if the queue is empty.
    /// Events are removed from the queue when polled.
    pub fn poll_event(&mut self) -> Option<StateMachineEvent> {
        self.event_queue.poll()
    }

    /// Poll for the next internal state machine event
    ///
    /// Returns Some(event) if an event is available, None if the queue is empty.
    /// Internal events are for framework use only.
    pub fn poll_internal_event(&mut self) -> Option<StateMachineInternalEvent> {
        self.internal_event_queue.poll()
    }

    // key: The key of the input
    // value: The value to set the input to
    // run_pipeline: If true, the pipeline will be run after setting the input. This is most likely false if called from an action or during initialization.
    // called_from_action: If true, the input was set from an action. We need this so that action_mutated_inputs is correctly set.
    pub fn set_numeric_input(
        &mut self,
        key: &str,
        value: f32,
        _run_pipeline: bool,
        called_from_action: bool,
    ) -> Option<InputValue> {
        // Modifying triggers whilst tweening isn't allowed
        if self.status == StateMachineEngineStatus::Tweening {
            return None;
        }

        let ret = self.inputs.set_numeric(key, value);

        if let Some(InputValue::Numeric(old_value)) = &ret {
            self.observe_numeric_input_value_change(key, *old_value, value);
        }

        if called_from_action {
            self.action_mutated_inputs = true;
        }

        // Note: run_pipeline is tracked but the actual pipeline run happens outside
        // when the caller has access to the player reference

        ret
    }

    pub fn get_numeric_input(&self, key: &str) -> Option<f32> {
        self.inputs.get_numeric(key)
    }

    pub fn set_string_input(
        &mut self,
        key: &str,
        value: &str,
        _run_pipeline: bool,
        called_from_action: bool,
    ) -> Option<InputValue> {
        // Modifying triggers whilst tweening isn't allowed
        if self.status == StateMachineEngineStatus::Tweening {
            return None;
        }

        let ret = self.inputs.set_string(key, value.to_string());

        if let Some(InputValue::String(old_value)) = ret.clone() {
            self.observe_string_input_value_change(key, &old_value, value);
        }

        if called_from_action {
            self.action_mutated_inputs = true;
        }
        // Note: run_pipeline is tracked but the actual pipeline run happens outside
        // when the caller has access to the player reference

        ret
    }

    pub fn get_string_input(&self, key: &str) -> Option<String> {
        self.inputs.get_string(key)
    }

    pub fn set_boolean_input(
        &mut self,
        key: &str,
        value: bool,
        _run_pipeline: bool,
        called_from_action: bool,
    ) -> Option<InputValue> {
        // Modifying triggers whilst tweening isn't allowed
        if self.status == StateMachineEngineStatus::Tweening {
            return None;
        }

        let ret = self.inputs.set_boolean(key, value);

        if let Some(InputValue::Boolean(old_value)) = ret.clone() {
            self.observe_boolean_input_value_change(key, old_value, value);
        }

        if called_from_action {
            self.action_mutated_inputs = true;
        }
        // Note: run_pipeline is tracked but the actual pipeline run happens outside
        // when the caller has access to the player reference

        ret
    }

    pub fn get_boolean_input(&self, key: &str) -> Option<bool> {
        self.inputs.get_boolean(key)
    }

    pub fn reset_input(&mut self, key: &str, _run_pipeline: bool, called_from_action: bool) {
        // Modifying triggers whilst tweening isn't allowed
        if self.status != StateMachineEngineStatus::Running {
            return;
        }

        let ret = self.inputs.reset(key);

        if let Some((old_value, new_value)) = ret {
            match old_value {
                InputValue::Numeric(old_value) => {
                    if let InputValue::Numeric(new_value) = new_value {
                        self.observe_numeric_input_value_change(key, old_value, new_value);
                    }
                }
                InputValue::String(old_value) => {
                    if let InputValue::String(new_value) = new_value {
                        self.observe_string_input_value_change(key, &old_value, &new_value);
                    }
                }
                InputValue::Boolean(old_value) => {
                    if let InputValue::Boolean(new_value) = new_value {
                        self.observe_boolean_input_value_change(key, old_value, new_value);
                    }
                }
                InputValue::Event(_) => {}
            }
        }

        if called_from_action {
            self.action_mutated_inputs = true;
        }
        // Note: run_pipeline is tracked but the actual pipeline run happens outside
        // when the caller has access to the player reference
    }

    pub fn fire(&mut self, event: &str, _run_pipeline: bool) -> Result<(), StateMachineEngineError> {
        // If the event is a valid input
        if let Some(valid_event) = self.inputs.get_event(event) {
            self.observe_on_input_fired(&valid_event);

            self.curr_event = Some(valid_event.to_string());

            // Note: run_pipeline is tracked but the actual pipeline run happens outside
            // when the caller has access to the player reference

            return Ok(());
        }

        Err(StateMachineEngineError::FireEventError)
    }

    // Parses the JSON of the state machine definition and creates the states and transitions
    pub fn create_state_machine(
        &mut self,
        sm_definition: &str,
        player: &mut DotLottiePlayerContainer,
    ) -> Result<StateMachineEngine, StateMachineEngineError> {
        let parsed_state_machine = state_machine_parse(sm_definition);
        let mut new_state_machine = StateMachineEngine::default();
        if parsed_state_machine.is_err() {
            let message = match parsed_state_machine.err() {
                Some(e) => format!("Parsing error: {e:?}"),
                None => "Parsing error: Unknown error".to_string(),
            };

            self.observe_on_error(message.as_str());

            return Err(StateMachineEngineError::ParsingError(message));
        }

        match parsed_state_machine {
            Ok(parsed_state_machine) => {
                /* Build all input variables into hashmaps for easier use */
                if let Some(inputs) = &parsed_state_machine.inputs {
                    for input in inputs {
                        match input {
                            Input::Numeric { name, value } => {
                                new_state_machine.inputs.set_initial_numeric(name, *value);
                            }
                            Input::String { name, value } => {
                                new_state_machine
                                    .inputs
                                    .set_initial_string(name, value.to_string());
                            }
                            Input::Boolean { name, value } => {
                                new_state_machine.inputs.set_initial_boolean(name, *value);
                            }
                            Input::Event { name } => {
                                new_state_machine.inputs.set_initial_event(name, name);
                            }
                        }
                    }
                }

                /*
                   Set the reference to the global state so that we can easily
                   Access it when evaluating transitions
                */
                for state in &parsed_state_machine.states {
                    if let State::GlobalState { .. } = state {
                        new_state_machine.global_state = Some(state.clone());
                    }
                }

                new_state_machine.state_machine = parsed_state_machine;

                new_state_machine.cached_player_config = player.config();

                new_state_machine.init_listened_layers();

                // Run the security check pipeline
                let check_report = self.security_check_pipeline(&new_state_machine);

                match check_report {
                    Ok(_) => {}
                    Err(error) => {
                        let message = format!("Load: {error:?}");

                        self.observe_on_error(message.as_str());

                        return Err(StateMachineEngineError::CreationError);
                    }
                }

                Ok(new_state_machine)
            }
            Err(_error) => Err(StateMachineEngineError::CreationError),
        }
    }

    fn security_check_pipeline(
        &self,
        state_machine: &StateMachineEngine,
    ) -> Result<(), StateMachineEngineSecurityError> {
        state_machine_state_check_pipeline(state_machine)
    }

    pub fn start(&mut self, player: &mut DotLottiePlayerContainer, open_url: &OpenUrlPolicy) -> bool {
        // Reset to first frame
        player.stop();
        // Remove all playback settings but preserve use_frame_interpolation and layout
        let current_config = player.config();
        let reset_config = Config {
            use_frame_interpolation: current_config.use_frame_interpolation,
            layout: current_config.layout,
            ..Config::default()
        };
        player.set_config(reset_config);

        // Start can still be called even if load failed. If load failed initial and states will be empty.
        if self.state_machine.initial.is_empty() || self.state_machine.states.is_empty() {
            return false;
        }

        self.open_url_requires_user_interaction = open_url.require_user_interaction;

        if !open_url.whitelist.is_empty() {
            let mut whitelist = Whitelist::new();

            // Add patterns to whitelist
            for entry in &open_url.whitelist {
                let _ = whitelist.add(entry);
            }

            self.open_url_whitelist = whitelist;
        }

        let initial = &self.state_machine.initial.clone();

        let err = self.set_current_state(initial, None, false, player);
        match err {
            Ok(_) => {}
            Err(error) => {
                let message = format!("Error setting initial state: {error:?}");

                self.observe_on_error(message.as_str());

                return false;
            }
        }

        if self.status == StateMachineEngineStatus::Running {
            return true;
        }

        self.observe_on_start();

        self.status = StateMachineEngineStatus::Running;

        let _ = self.run_current_state_pipeline(player);

        true
    }

    pub fn stop(&mut self, player: &mut DotLottiePlayerContainer) {
        self.status = StateMachineEngineStatus::Stopped;

        self.observe_on_stop();

        player.set_config(self.cached_player_config.clone());
    }

    pub fn status(&self) -> String {
        match self.status {
            StateMachineEngineStatus::Running => "Running".to_string(),
            StateMachineEngineStatus::Tweening => "Tweening".to_string(),
            StateMachineEngineStatus::Stopped => "Stopped".to_string(),
        }
    }

    pub fn get_current_state(&self) -> Option<State> {
        self.current_state.clone()
    }

    pub fn interactions(&self, event_type_filter: Option<String>) -> Vec<&Interaction> {
        let mut interactions_clone = Vec::new();
        let filter = event_type_filter.unwrap_or("".to_string());

        if let Some(interactions) = &self.state_machine.interactions {
            for interaction in interactions {
                if !filter.is_empty() {
                    // If the filter type and the interaction type don't match, skip
                    if filter == interaction.type_name() {
                        // Clones the references
                        interactions_clone.push(interaction);
                    }
                } else {
                    // No filter used, clone the reference
                    interactions_clone.push(interaction);
                }
            }
        }

        interactions_clone
    }

    fn init_listened_layers(&mut self) {
        let mut interactions = vec![];

        interactions.extend(self.interactions(None));

        let mut all_listened_layers: Vec<(String, String)> = vec![];

        // Get every layer we listen to
        for interaction in interactions {
            match interaction {
                Interaction::PointerEnter {
                    layer_name: Some(layer),
                    ..
                } => {
                    all_listened_layers
                        .push((layer.clone(), event_type_name!(PointerEnter).to_string()));
                }
                Interaction::PointerExit {
                    layer_name: Some(layer),
                    ..
                } => all_listened_layers
                    .push((layer.clone(), event_type_name!(PointerExit).to_string())),
                Interaction::PointerUp {
                    layer_name: Some(layer),
                    ..
                } => all_listened_layers
                    .push((layer.clone(), event_type_name!(PointerUp).to_string())),
                Interaction::PointerDown {
                    layer_name: Some(layer),
                    ..
                } => all_listened_layers
                    .push((layer.clone(), event_type_name!(PointerDown).to_string())),
                _ => {}
            }
        }

        self.pointer_management.listened_layers = all_listened_layers;
    }

    fn get_state(&self, state_name: &str) -> Option<State> {
        if let Some(global_state) = &self.global_state {
            if global_state.name() == state_name {
                return Some(global_state.clone());
            }
        }

        for state in self.state_machine.states.iter() {
            if state.name() == state_name {
                return Some(state.clone());
            }
        }

        None
    }

    pub fn resume_from_tweening(&mut self, player: &mut DotLottiePlayerContainer) {
        if self.status != StateMachineEngineStatus::Tweening {
            return;
        }

        self.status = StateMachineEngineStatus::Running;

        if let Some(target_state) = &self.tween_transition_target_state {
            // Assign the new state to the current_state
            self.current_state = Some(target_state.clone());

            self.tween_transition_target_state = None;

            // Emit transtion occured event
            self.observe_on_state_entered(&self.get_current_state_name());

            // Perform entry actions
            // Execute its type of state
            let state = self.current_state.take();

            // Now use the extracted information
            if let Some(state) = state {
                // Enter the state
                let _ = state.enter(self, player);

                // Don't forget to put things back
                // new_state becomes the current state
                self.current_state = Some(state);
            }
        }
    }

    // Set the current state to the target state
    // Manage performing entry and exit actions
    // As well as executing the state's type (Currently on PlaybackState has an effect on playback)
    fn set_current_state(
        &mut self,
        state_name: &str,
        causing_transition: Option<&Transition>,
        called_from_global: bool,
        player: &mut DotLottiePlayerContainer,
    ) -> Result<(), StateMachineEngineError> {
        let new_state = self.get_state(state_name);
        // We have a new state
        if let Some(new_state) = new_state {
            // Emit transtion occured event
            self.observe_on_transition(&self.get_current_state_name(), &new_state.name());
            // Perform exit actions on the current state if there is one.
            if self.current_state.is_some() {
                let state = self.current_state.take();
                // Now use the extracted information
                if let Some(state) = state {
                    if !called_from_global {
                        let _ = state.exit(self, player);
                    }
                    // Don't forget to put things back
                    // new_state becomes the current state
                    self.current_state = Some(state);
                }
            }
            // Emit transtion occured event
            self.observe_on_state_exit(&self.get_current_state_name());

            // Since the blended transition will take time
            // We have to save the target state and do the final transition when tweening has completed
            // The state machine is alerted of tweening finishing because the player calls the resume_from_tweening() method
            //  Note: If the tweened transition targets a State without a segment, it will not tween and the target state is treated it usually would.
            if let Some(causing_transition) = causing_transition {
                // If we dealing with a tweened transition
                if let Transition::Tweened { .. } = causing_transition {
                    // Clone segment before match to avoid partial move
                    let segment_clone = match &new_state {
                        State::PlaybackState { segment, .. } => segment.clone(),
                        _ => None,
                    };
                    match &new_state {
                        // If we're transitioning to a PlaybackState, grab the start segment
                        State::PlaybackState { .. } => {
                            if let Some(target_segment) = segment_clone {
                                self.tween_transition_target_state =
                                    Some(new_state.clone());
                                // Tweening is activated and the state machine has been paused whilst it transitions
                                self.status = StateMachineEngineStatus::Tweening;

                                player.tween_to_marker(
                                    &target_segment,
                                    Some(causing_transition.duration()),
                                    Some(causing_transition.easing().to_vec()),
                                );

                                return Ok(());
                            }
                        }
                        // If we're transitioning to a GlobalState, do nothing
                        State::GlobalState { .. } => {
                            return Ok(());
                        }
                    }
                }
            }

            // Assign the new state to the current_state
            self.current_state = Some(new_state);

            // Emit transtion occured event
            self.observe_on_state_entered(&self.get_current_state_name());
            // Perform entry actions
            // Execute its type of state
            let state = self.current_state.take();
            // Now use the extracted information
            if let Some(state) = state {
                // Enter the state
                let _ = state.enter(self, player);
                // Don't forget to put things back
                // new_state becomes the current state
                self.current_state = Some(state);
            } else {
                return Err(StateMachineEngineError::SetStateError);
            }
            return Ok(());
        }
        Err(StateMachineEngineError::CreationError)
    }

    // Returns: The target state and the causing transition
    fn evaluate_transitions(
        &self,
        state_to_evaluate: &State,
        event: Option<&String>,
    ) -> Option<(String, Transition)> {
        let transitions = state_to_evaluate.transitions();
        let mut guardless_transition: Option<&Transition> = None;

        for transition in transitions {
            if transition.guards().is_none() || transition.guards().as_ref().unwrap().is_empty() {
                guardless_transition = Some(transition);
            }
            // If in the transitions we need an event, and there wasn't one fired, don't run the checks.
            // If there wasn't an event needed, but we are sending an event, still do the checks.

            // Guards on a transition are evaluated in order of priority, all of them have to be valid to transition (&& not ||).
            else if (transition.transitions_contain_event() && event.is_some())
                || (!transition.transitions_contain_event() && event.is_none())
            {
                if let Some(guards) = transition.guards() {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            transitions::guard::Guard::Numeric { .. } => {
                                if !guard.numeric_input_is_satisfied(&self.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            transitions::guard::Guard::String { .. } => {
                                if !guard.string_input_is_satisfied(&self.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            transitions::guard::Guard::Boolean { .. } => {
                                if !guard.boolean_input_is_satisfied(&self.inputs) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            transitions::guard::Guard::Event { .. } => {
                                /* If theres a guard, but no event has been fired, we can't validate any guards. */
                                if event.is_none() {
                                    all_guards_satisfied = false;
                                    break;
                                }

                                if let Some(event) = event {
                                    if !guard.event_input_is_satisfied(event) {
                                        all_guards_satisfied = false;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    /* If all guard are satsified, take the transition as they are in order of priority inside the vec */
                    if all_guards_satisfied {
                        let target_state = transition.target_state();

                        return Some((target_state.to_string(), transition.clone()));
                    }
                }
            }
        }

        // Enforces the rule that a guardless transition should be taken in to account last
        let target_state = guardless_transition?.target_state();
        Some((target_state.to_string(), guardless_transition?.clone()))
    }

    fn evaluate_global_state(&mut self, player: &mut DotLottiePlayerContainer) -> bool {
        if let Some(state_to_evaluate) = &self.global_state {
            if let Some((target_state, causing_transition)) =
                self.evaluate_transitions(state_to_evaluate, self.curr_event.as_ref())
            {
                self.curr_event = None;

                // Prevent re-entering the current state again
                if target_state == self.get_current_state_name() {
                    return false;
                }

                let success =
                    self.set_current_state(&target_state, Some(&causing_transition), true, player);

                match success {
                    Ok(()) => {
                        return true;
                    }
                    Err(_) => {
                        return false;
                    }
                }
            }
        }
        false
    }

    pub fn run_current_state_pipeline(&mut self, player: &mut DotLottiePlayerContainer) -> Result<(), StateMachineEngineError> {
        // Reset cycle count for each pipeline run
        self.current_cycle_count = 0;

        // If the state machine is tweening, don't run the pipeline
        if self.status == StateMachineEngineStatus::Tweening {
            return Ok(());
        }

        // If the state machine is not running, or there is no current state, return an error
        // Otherwise this will block the pipeline in a loop
        if self.status != StateMachineEngineStatus::Running
            || (self.current_state.is_none() && self.global_state.is_none())
        {
            return Err(StateMachineEngineError::NotRunningError);
        }

        let mut tick = true;

        let mut ignore_global = false;

        while tick {
            // Safety fallback to prevent infinite loops
            tick = false;
            let mut ignore_child = false;

            // --------------- Start infinite loop detection
            if let Some(_cycle) = self.detect_cycle() {
                self.current_cycle_count += 1;

                if self.current_cycle_count >= self.max_cycle_count {
                    self.stop(player);
                    self.observe_on_error("InfiniteLoop");
                    return Err(StateMachineEngineError::InfiniteLoopError);
                }

                // Clear the history to allow for detecting new cycles
                self.state_history.clear();
            }

            // Record the current state
            if let Some(state) = &self.current_state {
                self.state_history.push(state.name().to_string());
            }

            // --------------- End infinite loop detection

            // Check if there is a global state
            // If there is, evaluate the transitions of the global state first
            if !ignore_global {
                // Global state returned true meaning it changed the current state
                if self.evaluate_global_state(player) {
                    // Check the current state, if its tweening, stop immediately
                    if self.status == StateMachineEngineStatus::Tweening {
                        break;
                    }
                    // Therfor we need to re-evaluate the global state.
                    // When we entered the state from global, it made on_entry changes.
                    if self.action_mutated_inputs {
                        ignore_global = false;
                        ignore_child = true;

                        tick = true;
                        self.action_mutated_inputs = false;
                    }
                    if self.curr_event.is_some() {
                        ignore_global = false;
                        ignore_child = true;

                        tick = true;
                    }
                }
            }

            if !ignore_child {
                if let Some(current_state_to_evaluate) = &self.current_state {
                    if let Some((target_state, causing_transition)) = self
                        .evaluate_transitions(current_state_to_evaluate, self.curr_event.as_ref())
                    {
                        self.curr_event = None;

                        let success =
                            self.set_current_state(&target_state, Some(&causing_transition), false, player);

                        match success {
                            Ok(()) => {
                                // Check the current state, if its tweening, stop immediately
                                if self.status == StateMachineEngineStatus::Tweening {
                                    break;
                                }
                                // Re-evaluate global state, a input was changed
                                if self.action_mutated_inputs {
                                    tick = true;

                                    ignore_global = false;
                                    self.action_mutated_inputs = false;
                                }
                                // Re-evaluate global state, an event was fired
                                else if self.curr_event.is_some() {
                                    tick = true;

                                    ignore_global = false;
                                }
                                // Re-evaluate current state, ignore global since no inputs were changed or events fired
                                else {
                                    tick = true;

                                    ignore_global = true;
                                }
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                }
            }
        }

        self.curr_event = None;

        Ok(())
    }

    fn detect_cycle(&self) -> Option<Vec<String>> {
        let mut seen = HashSet::new();
        let mut cycle = Vec::new();

        for state in self.state_history.iter().rev() {
            if !seen.insert(state) {
                // We've found the start of a cycle
                let cycle_start = state;
                cycle.push(cycle_start.clone());

                for s in self.state_history.iter().rev() {
                    if s == cycle_start {
                        break;
                    }
                    cycle.push(s.clone());
                }

                cycle.reverse();
                return Some(cycle);
            }
        }

        None
    }

    fn manage_explicit_events(&mut self, event: &Event, x: f32, y: f32, player: &mut DotLottiePlayerContainer) {
        let mut actions_to_execute: Vec<Action> = Vec::new();
        let interactions = self.interactions(None);
        let mut entered_layer = self.pointer_management.curr_entered_layer.clone();

        for interaction in interactions {
            if interaction.type_name() == event.type_name() {
                // User defined a specific layer to check if hit
                if let Some(layer) = interaction.get_layer_name() {
                    // If we have a pointer down event, we need to check if the pointer is outside of the layer
                    if let Event::PointerExit { x, y } = event {
                        if self.pointer_management.curr_entered_layer == *layer
                            && !player.intersect(*x, *y, &layer)
                        {
                            entered_layer = "".to_string();
                            actions_to_execute.extend(interaction.get_actions().clone());
                        }
                    } else {
                        // Hit check will return true if the layer was hit
                        if player.intersect(x, y, &layer) {
                            entered_layer = layer.clone();
                            actions_to_execute.extend(interaction.get_actions().clone());
                        }
                    }
                } else {
                    // No layer was specified, add all actions
                    actions_to_execute.extend(interaction.get_actions().clone());
                }
            }
        }

        self.pointer_management.curr_entered_layer = entered_layer;

        for action in actions_to_execute {
            // Run the pipeline because interactions are outside of the evaluation pipeline loop
            let _ = action.execute(self, player, true, false);
        }
    }

    fn manage_cross_platform_events(&mut self, event: &Event, x: f32, y: f32, player: &mut DotLottiePlayerContainer) {
        let mut actions_to_execute = Vec::new();

        // Manage pointerMove interactions
        if event.type_name() == *"PointerMove" {
            let pointer_move_interactions =
                self.interactions(Some(event_type_name!(PointerMove).to_string()));

            for interaction in pointer_move_interactions {
                if let Interaction::PointerMove { actions } = interaction {
                    actions_to_execute.extend(actions.clone());
                }
            }
        }

        // Check if we've moved the pointer over any of the pointerEnter/Exit interactions
        // If we've changed layers, perform exit actions
        // If we don't hit any layers, perform exit actions
        let mut hit = false;
        let old_layer = self.pointer_management.curr_entered_layer.clone();

        // Loop through all layers we're listening to
        for (layer, event_name) in &self.pointer_management.listened_layers.clone() {
            // We're only interested in the listened layers that need enter / exit event
            if (event_name == event_type_name!(PointerEnter)
                || event_name == event_type_name!(PointerExit))
                && player.intersect(x, y, layer)
            {
                hit = true;

                // If it's that same current layer, do nothing
                if self.pointer_management.curr_entered_layer == *layer {
                    break;
                }

                self.pointer_management.curr_entered_layer = layer.to_string();

                // Get all pointer_enter interactions
                let pointer_enter_interactions =
                    self.interactions(Some(event_type_name!(PointerEnter).to_string()));

                // Add their actions if their layer name matches the current layer name in loop
                for interaction in pointer_enter_interactions {
                    if let Some(interaction_layer_name) = interaction.get_layer_name() {
                        if *interaction_layer_name
                            == self.pointer_management.curr_entered_layer
                        {
                            actions_to_execute.extend(interaction.get_actions().clone());
                        }
                    }
                }
            }
        }

        // We didn't hit any listened layers
        if !hit {
            self.pointer_management.curr_entered_layer = "".to_string();

            let pointer_exit_interactions =
                self.interactions(Some(event_type_name!(PointerExit).to_string()));

            // Add the actions of every PointerExit interaction that depended on the layer we've just exited
            for interaction in pointer_exit_interactions {
                if let Some(interaction_layer_name) = interaction.get_layer_name() {
                    // We've exited the desired layer, add its actions to execute
                    if *interaction_layer_name == old_layer {
                        actions_to_execute.extend(interaction.get_actions().clone());
                    }
                }
            }
        }

        for action in actions_to_execute {
            // Run the pipeline because interactions are outside of the evaluation pipeline loop
            let _ = action.execute(self, player, true, false);
        }
    }

    // How pointer event are managed depending on the interaction's event and the sent event.
    // Since we can't detect PointerMove on mobile, we can still check PointerDown/Up and see if it's entered or exited a layer.
    //
    // | -------------------------------- | ----------------------------- | ----------- |
    // | Interaction Event type              | Web                           | Mobile      |
    // | -------------------------------- | ----------------------------- | ----------- |
    // | PointerDown (No Layer)           | PointerDown                   | PointerDown |
    // | PointerDown (With Layer)         | PointerDown                   | PointerDown |
    // | PointerUp (No Layer)             | PointerUp                     | PointerUp   |
    // | PointerUp (With Layer)           | PointerUp                     | PointerUp   |
    // | PointerMove (No Layer)           | PointerMove                   | PointerDown |
    // | PointerEnter (No Layer)          | PointerEnter                  | Not avail.  |
    // | PointerEnter (With Layer)        | PointerMove + PointerEnter    | PointerDown |
    // | PointerExit (No Layer)           | PointerExit                   | Not avail.  |
    // | PointerExit (With Layer)         | PointerMove + PointerExit     | PointerUp   |
    // | Click (With Layer)               | Click                         | Tap         |
    // | Click (No Layer)                 | Click                         | Tap         |
    // | ---------------------------------|-------------------------------| ----------- |

    // Notes:
    // Atm, PointerEnter/Exit without layers is not supported on mobile.
    // This is because if we allow pointerDown to activate PointerEnter/Exit,
    // It would override PointerDown with layers, which is not a great experience.
    // With the current setup we can have an action that happens when the cursor is over the canvas
    // and another action that happens when the cursor is over a specific layer.
    fn manage_pointer_event(&mut self, event: &Event, x: f32, y: f32, player: &mut DotLottiePlayerContainer) {
        self.pointer_management.pointer_x = x;
        self.pointer_management.pointer_y = y;

        // This will handle PointerDown, PointerUp, PointerEnter, PointerExit, Click
        if event.type_name() != "PointerMove" {
            self.manage_explicit_events(event, x, y, player);
        }

        // We're left with PointerMove
        // Also perform checks for PointerDown and PointerUp, a mobile framework could of sent them and validate PointerEnter/Exit interactions.
        if event.type_name() == "PointerMove"
            || event.type_name() == "PointerDown"
            || event.type_name() == "PointerUp"
        {
            self.manage_cross_platform_events(event, x, y, player);
        }
    }

    fn manage_player_events(&mut self, event: &Event, player: &mut DotLottiePlayerContainer) {
        let interactions = self.interactions(Some(event.type_name()));

        if interactions.is_empty() {
            return;
        }

        let mut actions_to_execute = Vec::new();

        for interaction in interactions {
            if let Interaction::OnComplete {
                state_name,
                actions,
            } = interaction
            {
                if let Some(current_state) = &self.current_state {
                    if current_state.name() == *state_name {
                        actions_to_execute.extend(actions.clone());
                    }
                }
            }
            if let Interaction::OnLoopComplete {
                state_name,
                actions,
            } = interaction
            {
                if let Some(current_state) = &self.current_state {
                    if current_state.name() == *state_name {
                        actions_to_execute.extend(actions.clone());
                    }
                }
            }
        }

        for action in actions_to_execute {
            // Run the pipeline because interactions are outside of the evaluation pipeline loop
            let _ = action.execute(self, player, true, false);
        }
    }

    pub fn post_event(&mut self, event: &Event, player: &mut DotLottiePlayerContainer) {
        self.pointer_management.most_recent_event = Some(event.clone());

        if event.type_name().contains("Pointer") || event.type_name().contains("Click") {
            self.manage_pointer_event(event, event.x(), event.y(), player);
        } else {
            self.manage_player_events(event, player);
        }
    }

    /**
     * Force a state change to the target state. Will not input an evaluation
     * after entering the target state.
     *
     * @params state_name: The name of the state to change to.
     * @params do_tick: If true, the state machine will run the transition evaluation pipeline after changing the state.
     */
    pub fn override_current_state(&mut self, state_name: &str, do_tick: bool, player: &mut DotLottiePlayerContainer) -> bool {
        let r = self.set_current_state(state_name, None, false, player).is_ok();

        if do_tick {
            return self.run_current_state_pipeline(player).is_ok();
        }

        r
    }

    pub fn get_state_machine(&self) -> &StateMachine {
        &self.state_machine
    }

    pub fn get_current_state_name(&self) -> String {
        if let Some(state) = &self.current_state {
            return state.name();
        }

        "".to_string()
    }

    fn observe_on_state_entered(&mut self, entering_state: &str) {
        self.event_queue.push(StateMachineEvent::StateEntered {
            state: entering_state.to_string(),
        });
    }

    fn observe_on_state_exit(&mut self, leaving_state: &str) {
        self.event_queue.push(StateMachineEvent::StateExit {
            state: leaving_state.to_string(),
        });
    }

    fn observe_on_transition(&mut self, previous_state: &str, new_state: &str) {
        self.event_queue.push(StateMachineEvent::Transition {
            previous_state: previous_state.to_string(),
            new_state: new_state.to_string(),
        });
    }

    pub fn observe_internal_event(&mut self, message: &str) {
        self.internal_event_queue.push(StateMachineInternalEvent::Message {
            message: message.to_string(),
        });
    }

    pub fn observe_custom_event(&mut self, message: &str) {
        self.event_queue.push(StateMachineEvent::CustomEvent {
            message: message.to_string(),
        });
    }

    pub fn observe_on_error(&mut self, message: &str) {
        self.event_queue.push(StateMachineEvent::Error {
            message: message.to_string(),
        });
    }

    pub fn observe_string_input_value_change(
        &mut self,
        input_name: &str,
        old_value: &str,
        new_value: &str,
    ) {
        if old_value == new_value {
            return;
        }
        self.event_queue.push(StateMachineEvent::StringInputChange {
            name: input_name.to_string(),
            old_value: old_value.to_string(),
            new_value: new_value.to_string(),
        });
    }

    pub fn observe_numeric_input_value_change(
        &mut self,
        input_name: &str,
        old_value: f32,
        new_value: f32,
    ) {
        if old_value == new_value {
            return;
        }
        self.event_queue.push(StateMachineEvent::NumericInputChange {
            name: input_name.to_string(),
            old_value,
            new_value,
        });
    }

    pub fn observe_boolean_input_value_change(
        &mut self,
        input_name: &str,
        old_value: bool,
        new_value: bool,
    ) {
        if old_value == new_value {
            return;
        }
        self.event_queue.push(StateMachineEvent::BooleanInputChange {
            name: input_name.to_string(),
            old_value,
            new_value,
        });
    }

    pub fn observe_on_start(&mut self) {
        self.event_queue.push(StateMachineEvent::Start);
    }

    pub fn observe_on_stop(&mut self) {
        self.event_queue.push(StateMachineEvent::Stop);
    }

    pub fn observe_on_input_fired(&mut self, input_name: &str) {
        self.event_queue.push(StateMachineEvent::InputFired {
            name: input_name.to_string(),
        });
    }
}

