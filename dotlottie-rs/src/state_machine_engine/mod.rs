use core::result::Result::Ok;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

pub mod actions;
pub mod errors;
pub mod events;
pub mod listeners;
pub mod security;
pub mod state_machine;
pub mod states;
pub mod transitions;
pub mod triggers;

use actions::{Action, ActionTrait};
use listeners::ListenerTrait;
use state_machine::StateMachine;
use states::StateTrait;
use transitions::guard::GuardTrait;
use transitions::{Transition, TransitionTrait};
use triggers::{Trigger, TriggerManager, TriggerTrait, TriggerValue};

use crate::state_machine_engine::listeners::Listener;
use crate::{
    event_type_name, state_machine_state_check_pipeline, DotLottiePlayerContainer, EventName,
    PointerEvent, StateMachineEngineSecurityError,
};

use self::state_machine::state_machine_parse;
use self::{events::Event, states::State};

pub trait StateMachineObserver: Send + Sync {
    fn on_transition(&self, previous_state: String, new_state: String);
    fn on_state_entered(&self, entering_state: String);
    fn on_state_exit(&self, leaving_state: String);
    fn on_custom_event(&self, message: String);
    fn on_error(&self, error: String);
}

#[derive(PartialEq, Debug)]
pub enum StateMachineEngineStatus {
    Running,
    Paused,
    Stopped,
}

#[derive(Debug, thiserror::Error)]
pub enum StateMachineEngineError {
    #[error("Failed to parse JSON state machine definition.")]
    ParsingError { reason: String },

    #[error("Failed to create StateMachineEngine.")]
    CreationError { reason: String },

    #[error("Event can not be fired as it does not exist.")]
    FireEventError,

    #[error("Infinite loop detected.")]
    InfiniteLoopError,

    #[error("State machine engine is not running.")]
    NotRunningError,

    #[error("Failed to change the current state.")]
    SetStateError,

    #[error(
        "The state: {} has multiple transitions without guards. This is not allowed.",
        state_name
    )]
    SecurityCheckErrorMultipleGuardlessTransitions { state_name: String },

    #[error(
        "The state name: {} has been used multiple times. This is not allowed.",
        state_name
    )]
    SecurityCheckErrorDuplicateStateName { state_name: String },
}

pub struct StateMachineEngine {
    /* We keep references to the StateMachine's States. */
    /* This prevents duplicating the data inside the engine. */
    pub global_state: Option<Rc<State>>,
    pub current_state: Option<Rc<State>>,

    pub player: Option<Rc<RwLock<DotLottiePlayerContainer>>>,
    pub status: StateMachineEngineStatus,

    triggers: TriggerManager,
    event_trigger: HashMap<String, String>,
    curr_event: Option<String>,

    // PointerEnter/PointerExit management
    curr_entered_layer: String,
    listened_layers: Vec<(String, String)>,

    observers: RwLock<Vec<Arc<dyn StateMachineObserver>>>,

    state_machine: StateMachine,

    state_history: Vec<String>,
    max_cycle_count: usize,
    current_cycle_count: usize,
    action_mutated_triggers: bool,
}

impl Default for StateMachineEngine {
    fn default() -> StateMachineEngine {
        StateMachineEngine {
            global_state: None,
            state_machine: StateMachine::default(),
            current_state: None,
            player: None,
            triggers: TriggerManager::new(),
            event_trigger: HashMap::new(),
            curr_event: None,
            curr_entered_layer: "".to_string(),
            listened_layers: Vec::new(),
            status: StateMachineEngineStatus::Stopped,
            observers: RwLock::new(Vec::new()),
            state_history: Vec::new(),
            max_cycle_count: 20,
            current_cycle_count: 0,
            action_mutated_triggers: false,
        }
    }
}

impl StateMachineEngine {
    pub fn new(
        state_machine_definition: &str,
        player: Rc<RwLock<DotLottiePlayerContainer>>,
        max_cycle_count: Option<usize>,
    ) -> Result<StateMachineEngine, StateMachineEngineError> {
        let mut state_machine = StateMachineEngine {
            global_state: None,
            state_machine: StateMachine::default(),
            current_state: None,
            player: Some(player.clone()),
            triggers: TriggerManager::new(),
            event_trigger: HashMap::new(),
            curr_event: None,
            curr_entered_layer: "".to_string(),
            listened_layers: Vec::new(),
            status: StateMachineEngineStatus::Stopped,
            observers: RwLock::new(Vec::new()),
            state_history: Vec::new(),
            max_cycle_count: max_cycle_count.unwrap_or(20),
            current_cycle_count: 0,
            action_mutated_triggers: false,
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

    // key: The key of the trigger
    // value: The value to set the trigger to
    // run_pipeline: If true, the pipeline will be run after setting the trigger. This is most likely false if called from an action or during initialization.
    // called_from_action: If true, the trigger was set from an action. We need this so that action_mutated_triggers is correctly set.
    pub fn set_numeric_trigger(
        &mut self,
        key: &str,
        value: f32,
        run_pipeline: bool,
        called_from_action: bool,
    ) -> Option<TriggerValue> {
        let ret = self.triggers.set_numeric(key, value);

        if called_from_action {
            self.action_mutated_triggers = true;
        }

        if run_pipeline {
            let _ = self.run_current_state_pipeline();
        }

        ret
    }

    pub fn get_numeric_trigger(&self, key: &str) -> Option<f32> {
        self.triggers.get_numeric(key)
    }

    pub fn set_string_trigger(
        &mut self,
        key: &str,
        value: &str,
        run_pipeline: bool,
        called_from_action: bool,
    ) -> Option<TriggerValue> {
        let ret = self.triggers.set_string(key, value.to_string());

        if called_from_action {
            self.action_mutated_triggers = true;
        }
        if run_pipeline {
            let _ = self.run_current_state_pipeline();
        }

        ret
    }

    pub fn get_string_trigger(&self, key: &str) -> Option<String> {
        self.triggers.get_string(key)
    }

    pub fn set_boolean_trigger(
        &mut self,
        key: &str,
        value: bool,
        run_pipeline: bool,
        called_from_action: bool,
    ) -> Option<TriggerValue> {
        let ret = self.triggers.set_boolean(key, value);

        if called_from_action {
            self.action_mutated_triggers = true;
        }
        if run_pipeline {
            let _ = self.run_current_state_pipeline();
        }

        ret
    }

    pub fn get_boolean_trigger(&self, key: &str) -> Option<bool> {
        self.triggers.get_boolean(key)
    }

    pub fn reset_trigger(&mut self, key: &str, run_pipeline: bool, called_from_action: bool) {
        self.triggers.reset(key);

        if called_from_action {
            self.action_mutated_triggers = true;
        }
        if run_pipeline {
            let _ = self.run_current_state_pipeline();
        }
    }

    pub fn fire(&mut self, event: &str, run_pipeline: bool) -> Result<(), StateMachineEngineError> {
        // If the event is a valid trigger
        if let Some(valid_event) = self.event_trigger.get(event) {
            self.curr_event = Some(valid_event.to_string());

            // Run pipeline is always false if called from an action
            if run_pipeline {
                let _ = self.run_current_state_pipeline();
            }

            return Ok(());
        }

        Err(StateMachineEngineError::FireEventError)
    }

    // Parses the JSON of the state machine definition and creates the states and transitions
    pub fn create_state_machine(
        &mut self,
        sm_definition: &str,
        player: &Rc<RwLock<DotLottiePlayerContainer>>,
    ) -> Result<StateMachineEngine, StateMachineEngineError> {
        let parsed_state_machine = state_machine_parse(sm_definition);
        let mut new_state_machine = StateMachineEngine::default();

        if parsed_state_machine.is_err() {
            println!(
                "Error parsing state machine definition: {:?}",
                parsed_state_machine.err()
            );
            return Err(StateMachineEngineError::ParsingError {
                reason: "Failed to parse state machine definition".to_string(),
            });
        }

        match parsed_state_machine {
            Ok(parsed_state_machine) => {
                let initial_state_index = parsed_state_machine.initial.clone();

                /* Build all trigger variables into hashmaps for easier use */
                if let Some(triggers) = &parsed_state_machine.triggers {
                    for trigger in triggers {
                        match trigger {
                            Trigger::Numeric { name, value } => {
                                new_state_machine.triggers.set_initial_numeric(name, *value);
                            }
                            Trigger::String { name, value } => {
                                new_state_machine
                                    .triggers
                                    .set_initial_string(name, value.to_string());
                            }
                            Trigger::Boolean { name, value } => {
                                new_state_machine.triggers.set_initial_boolean(name, *value);
                            }
                            Trigger::Event { name } => {
                                new_state_machine
                                    .event_trigger
                                    .insert(name.to_string(), name.to_string());
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
                        new_state_machine.global_state = Some(Rc::new(state.clone()));
                    }
                }

                new_state_machine.player = Some(player.clone());
                new_state_machine.state_machine = parsed_state_machine;

                new_state_machine.init_listened_layers();

                // Run the security check pipeline
                let check_report = self.security_check_pipeline(&new_state_machine);

                match check_report {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(StateMachineEngineError::ParsingError {
                            reason: error.to_string(),
                        });
                    }
                }

                let err = new_state_machine.set_current_state(&initial_state_index, false);
                match err {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(StateMachineEngineError::CreationError {
                            reason: error.to_string(),
                        });
                    }
                }

                Ok(new_state_machine)
            }
            Err(error) => Err(StateMachineEngineError::CreationError {
                reason: error.to_string(),
            }),
        }
    }

    fn security_check_pipeline(
        &self,
        state_machine: &StateMachineEngine,
    ) -> Result<(), StateMachineEngineSecurityError> {
        state_machine_state_check_pipeline(state_machine)
    }

    pub fn start(&mut self) {
        if self.status == StateMachineEngineStatus::Running {
            return;
        }
        self.status = StateMachineEngineStatus::Running;

        let _ = self.run_current_state_pipeline();
    }

    pub fn pause(&mut self) {
        self.status = StateMachineEngineStatus::Paused;
    }

    pub fn stop(&mut self) {
        self.status = StateMachineEngineStatus::Stopped;
    }

    pub fn get_current_state(&self) -> Option<Rc<State>> {
        self.current_state.clone()
    }

    pub fn listeners(&self, event_type_filter: Option<String>) -> Vec<&Listener> {
        let mut listeners_clone = Vec::new();
        let filter = event_type_filter.unwrap_or("".to_string());

        if let Some(listeners) = &self.state_machine.listeners {
            for listener in listeners {
                if !filter.is_empty() {
                    // If the filter type and the listener type don't match, skip
                    if filter == listener.type_name() {
                        // Clones the references
                        listeners_clone.push(listener);
                    }
                } else {
                    // No filter used, clone the reference
                    listeners_clone.push(listener);
                }
            }
        }

        listeners_clone
    }

    fn init_listened_layers(&mut self) {
        let mut listeners = vec![];

        listeners.extend(self.listeners(None));

        let mut all_listened_layers: Vec<(String, String)> = vec![];

        // Get every layer we listen to
        for listener in listeners {
            match listener {
                Listener::PointerEnter { layer_name, .. } => {
                    if let Some(layer) = layer_name {
                        all_listened_layers
                            .push((layer.clone(), event_type_name!(PointerEnter).to_string()));
                    }
                }
                Listener::PointerExit { layer_name, .. } => {
                    if let Some(layer) = layer_name {
                        all_listened_layers
                            .push((layer.clone(), event_type_name!(PointerExit).to_string()))
                    }
                }
                Listener::PointerUp { layer_name, .. } => {
                    if let Some(layer) = layer_name {
                        all_listened_layers
                            .push((layer.clone(), event_type_name!(PointerUp).to_string()))
                    }
                }
                Listener::PointerDown { layer_name, .. } => {
                    if let Some(layer) = layer_name {
                        all_listened_layers
                            .push((layer.clone(), event_type_name!(PointerDown).to_string()))
                    }
                }
                _ => {}
            }
        }

        self.listened_layers = all_listened_layers;
    }

    fn get_state(&self, state_name: &str) -> Option<Rc<State>> {
        if let Some(global_state) = &self.global_state {
            if global_state.name() == state_name {
                return Some(global_state.clone());
            }
        }

        for state in self.state_machine.states.iter() {
            if state.name() == state_name {
                return Some(Rc::new(state.clone()));
            }
        }

        None
    }

    // Set the current state to the target state
    // Manage performing entry and exit actions
    // As well as executing the state's type (Currently on PlaybackState has an effect on playback)
    fn set_current_state(
        &mut self,
        state_name: &str,
        called_from_global: bool,
    ) -> Result<(), StateMachineEngineError> {
        let new_state = self.get_state(state_name);

        // We have a new state
        if let Some(new_state) = new_state {
            // Emit transtion occured event
            self.observe_on_transition(&self.get_current_state_name(), &new_state.name());

            // Perform exit actions on the current state if there is one.
            if self.current_state.is_some() {
                let state = self.current_state.take();
                let player = self.player.take();

                // Now use the extracted information
                if let (Some(state), Some(player)) = (state, player) {
                    if !called_from_global {
                        let _ = state.exit(self, &player);
                    }

                    // Don't forget to put things back
                    // new_state becomes the current state
                    self.current_state = Some(state);
                    self.player = Some(player);
                }
            }

            // Emit transtion occured event
            self.observe_on_state_exit(&self.get_current_state_name());

            // Assign the new state to the current_state
            self.current_state = Some(new_state);

            // Emit transtion occured event
            self.observe_on_state_entered(&self.get_current_state_name());

            // Perform entry actions
            // Execute its type of state
            let state = self.current_state.take();
            let player = self.player.take();

            // Now use the extracted information
            if let (Some(state), Some(player)) = (state, player) {
                // Enter the state
                state.enter(self, &player);

                // Don't forget to put things back
                // new_state becomes the current state
                self.current_state = Some(state);
                self.player = Some(player);
            } else {
                return Err(StateMachineEngineError::SetStateError {});
            }

            return Ok(());
        }

        Err(StateMachineEngineError::CreationError {
            reason: format!("Failed to find state: {}", state_name),
        })
    }

    /* Returns the target state, otherwise None */
    /* Todo: Integrate transitions with no guards */
    /* Todo: Integrate if only one transitions with no guard */
    fn evaluate_transitions(
        &self,
        state_to_evaluate: &Rc<State>,
        event: Option<&String>,
    ) -> Option<String> {
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
                                if !guard.numeric_trigger_is_satisfied(&self.triggers) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            transitions::guard::Guard::String { .. } => {
                                if !guard.string_trigger_is_satisfied(&self.triggers) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            transitions::guard::Guard::Boolean { .. } => {
                                if !guard.boolean_trigger_is_satisfied(&self.triggers) {
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
                                    if !guard.event_trigger_is_satisfied(event) {
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

                        return Some(target_state.to_string());
                    }
                }
            }
        }

        // Enforces the rule that a guardless transition should be taken in to account last
        let target_state = guardless_transition?.target_state();
        Some(target_state.to_string())
    }

    fn evaluate_global_state(&mut self) -> bool {
        if let Some(state_to_evaluate) = &self.global_state {
            let target_state =
                self.evaluate_transitions(state_to_evaluate, self.curr_event.as_ref());

            self.curr_event = None;

            if let Some(state) = target_state {
                let success = self.set_current_state(&state, true);

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

    pub fn run_current_state_pipeline(&mut self) -> Result<(), StateMachineEngineError> {
        // Reset cycle count for each pipeline run
        self.current_cycle_count = 0;

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
                    self.stop();
                    self.observe_on_error("Infinite loop detected! Stopping the state machine.");
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
                if self.evaluate_global_state() {
                    // Therfor we need to re-evaluate the global state.
                    // When we entered the state from global, it made on_entry changes.
                    if self.action_mutated_triggers {
                        ignore_global = false;
                        ignore_child = true;

                        tick = true;
                        self.action_mutated_triggers = false;
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
                    let target_state = self
                        .evaluate_transitions(current_state_to_evaluate, self.curr_event.as_ref());

                    self.curr_event = None;

                    if let Some(state) = target_state {
                        let success = self.set_current_state(&state, false);

                        match success {
                            Ok(()) => {
                                // Re-evaluate global state, a trigger was changed
                                if self.action_mutated_triggers {
                                    tick = true;

                                    ignore_global = false;
                                    self.action_mutated_triggers = false;
                                }
                                // Re-evaluate global state, an event was fired
                                else if self.curr_event.is_some() {
                                    tick = true;

                                    ignore_global = false;
                                }
                                // Re-evaluate current state, ignore global since no triggers were changed or events fired
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

    fn manage_explicit_events(&mut self, event: &Event, x: f32, y: f32) {
        let mut actions_to_execute: Vec<Action> = Vec::new();
        let listeners = self.listeners(None);
        let mut entered_layer = self.curr_entered_layer.clone();

        for listener in listeners {
            if listener.type_name() == event.type_name() {
                // User defined a specific layer to check if hit
                if let Some(layer) = listener.get_layer_name() {
                    // Check if the layer was hit, otherwise we ignore this listener
                    if let Some(rc_player) = &self.player {
                        let try_read_lock = rc_player.try_read();

                        if let Ok(player_container) = try_read_lock {
                            // If we have a pointer down event, we need to check if the pointer is outside of the layer
                            if let Event::PointerExit { x, y } = event {
                                if self.curr_entered_layer == *layer
                                    && !player_container.hit_check(&layer, *x, *y)
                                {
                                    entered_layer = "".to_string();
                                    actions_to_execute.extend(listener.get_actions().clone());
                                }
                            } else {
                                // Hit check will return true if the layer was hit
                                if player_container.hit_check(&layer, x, y) {
                                    entered_layer = layer.clone();
                                    actions_to_execute.extend(listener.get_actions().clone());
                                }
                            }
                        }
                    }
                } else {
                    // No layer was specified, add all actions
                    actions_to_execute.extend(listener.get_actions().clone());
                }
            }
        }

        self.curr_entered_layer = entered_layer;

        for action in actions_to_execute {
            // Run the pipeline because listeners are outside of the evaluation pipeline loop
            if let Some(player_ref) = &self.player {
                let _ = action.execute(self, player_ref.clone(), true);
            }
        }
    }

    fn manage_cross_platform_events(&mut self, event: &Event, x: f32, y: f32) {
        let mut actions_to_execute = Vec::new();

        // Manage pointerMove listeners
        if event.type_name() == event_type_name!(PointerMove).to_string() {
            let pointer_move_listeners =
                self.listeners(Some(event_type_name!(PointerMove).to_string()));

            for listener in pointer_move_listeners {
                if let Listener::PointerMove { actions } = listener {
                    actions_to_execute.extend(actions.clone());
                }
            }
        }

        // Check if we've moved the pointer over any of the pointerEnter/Exit listeners
        // If we've changed layers, perform exit actions
        // If we don't hit any layers, perform exit actions
        if let Some(rc_player) = &self.player {
            let try_read_lock = rc_player.try_read();

            if let Ok(player_container) = try_read_lock {
                let mut hit = false;
                let old_layer = self.curr_entered_layer.clone();

                // Loop through all layers we're listening to
                for (layer, event_name) in &self.listened_layers {
                    // We're only interested in the listened layers that need enter / exit event
                    if event_name == event_type_name!(PointerEnter)
                        || event_name == event_type_name!(PointerExit)
                    {
                        if player_container.hit_check(&layer, x, y) {
                            hit = true;

                            // If it's that same current layer, do nothing
                            if self.curr_entered_layer == *layer {
                                break;
                            }

                            self.curr_entered_layer = layer.to_string();

                            // Get all pointer_enter listeners
                            let pointer_enter_listeners =
                                self.listeners(Some(event_type_name!(PointerEnter).to_string()));

                            // Add their actions if their layer name matches the current layer name in loop
                            for listener in pointer_enter_listeners {
                                if let Some(listener_layer_name) = listener.get_layer_name() {
                                    if *listener_layer_name == self.curr_entered_layer {
                                        actions_to_execute.extend(listener.get_actions().clone());
                                    }
                                }
                            }
                        }
                    }
                }

                // We didn't hit any listened layers
                if !hit {
                    self.curr_entered_layer = "".to_string();

                    let pointer_exit_listeners =
                        self.listeners(Some(event_type_name!(PointerExit).to_string()));

                    // Add the actions of every PointerExit listener that depended on the layer we've just exited
                    for listener in pointer_exit_listeners {
                        if let Some(listener_layer_name) = listener.get_layer_name() {
                            // We've exited the desired layer, add its actions to execute
                            if *listener_layer_name == old_layer {
                                actions_to_execute.extend(listener.get_actions().clone());
                            }
                        }
                    }
                }
            }
        }

        for action in actions_to_execute {
            // Run the pipeline because listeners are outside of the evaluation pipeline loop
            if let Some(player_ref) = &self.player {
                let _ = action.execute(self, player_ref.clone(), true);
            }
        }
    }

    // How pointer event are managed depending on the listener's event and the sent event.
    // Since we can't detect PointerMove on mobile, we can still check PointerDown/Up and see if it's entered or exited a layer.
    //
    // | -------------------------------- | ----------------------------- | ----------- |
    // | Listener Event type              | Web                           | Mobile      |
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
    // | ---------------------------------|-------------------------------| ----------- |

    // Notes:
    // Atm, PointerEnter/Exit without layers is not supported on mobile.
    // This is because if we allow pointerDown to activate PointerEnter/Exit,
    // It would override PointerDown with layers, which is not a great experience.
    // With the current setup we can have an action that happens when the cursor is over the canvas
    // and another action that happens when the cursor is over a specific layer.
    fn manage_pointer_event(&mut self, event: &Event, x: f32, y: f32) {
        // This will handle PointerDown, PointerUp, PointerEnter, PointerExit
        if event.type_name() != "PointerMove" {
            self.manage_explicit_events(event, x, y);
        }

        // We're left with PointerMove
        // Also perform checks for PointerDown and PointerUp, a mobile framework could of sent them and validate PointerEnter/Exit listeners.
        if event.type_name() == "PointerMove"
            || event.type_name() == "PointerDown"
            || event.type_name() == "PointerUp"
        {
            self.manage_cross_platform_events(event, x, y);
        }
    }

    fn manage_on_complete_event(&mut self, event: &Event) {
        let listeners = self.listeners(Some(event.type_name()));

        if listeners.is_empty() {
            return;
        }

        let mut actions_to_execute = Vec::new();

        for listener in listeners {
            if let Listener::OnComplete {
                state_name,
                actions,
            } = listener
            {
                if let Some(current_state) = &self.current_state {
                    if current_state.name() == *state_name {
                        actions_to_execute.extend(actions.clone());
                    }
                }
            }
        }

        for action in actions_to_execute {
            // Run the pipeline because listeners are outside of the evaluation pipeline loop
            if let Some(player_ref) = &self.player {
                let _ = action.execute(self, player_ref.clone(), true);
            }
        }
    }

    // Return codes
    // 0: Success
    // 1: Failure
    // 2: Play animation
    // 3: Pause animation
    // 4: Request and draw a new single frame of the animation (needed for sync state)
    pub fn post_event(&mut self, event: &Event) -> i32 {
        if event.type_name().contains("Pointer") {
            self.manage_pointer_event(event, event.x(), event.y());
        } else {
            self.manage_on_complete_event(event);
        }

        0
    }

    /**
     * Force a state change to the target state. Will not trigger an evaluation
     * after entering the target state.
     *
     * @params state_name: The name of the state to change to.
     * @params do_tick: If true, the state machine will run the transition evaluation pipeline after changing the state.
     */
    pub fn override_current_state(&mut self, state_name: &str, do_tick: bool) -> bool {
        let r = self.set_current_state(state_name, false).is_ok();

        if do_tick {
            return self.run_current_state_pipeline().is_ok();
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

    fn observe_on_state_entered(&self, entering_state: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(rc_player) = &self.player {
                let try_read_lock = rc_player.try_read();

                if let Ok(player_container) = try_read_lock {
                    player_container
                        .emit_state_machine_observer_on_state_entered(entering_state.to_string());
                }
            }
        }

        if let Ok(observers) = self.observers.try_read() {
            for observer in observers.iter() {
                observer.on_state_entered(entering_state.to_string());
            }
        }
    }

    fn observe_on_state_exit(&self, leaving_state: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(rc_player) = &self.player {
                let try_read_lock = rc_player.try_read();

                if let Ok(player_container) = try_read_lock {
                    player_container
                        .emit_state_machine_observer_on_state_entered(leaving_state.to_string());
                }
            }
        }
        if let Ok(observers) = self.observers.try_read() {
            for observer in observers.iter() {
                observer.on_state_exit(leaving_state.to_string());
            }
        }
    }

    fn observe_on_transition(&self, previous_state: &str, new_state: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(rc_player) = &self.player {
                let try_read_lock = rc_player.try_read();

                if let Ok(player_container) = try_read_lock {
                    player_container.emit_state_machine_observer_on_transition(
                        previous_state.to_string(),
                        new_state.to_string(),
                    );
                }
            }
        }

        if let Ok(observers) = self.observers.try_read() {
            for observer in observers.iter() {
                observer.on_transition(previous_state.to_string(), new_state.to_string());
            }
        }
    }

    pub fn observe_custom_event(&self, message: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(rc_player) = &self.player {
                let try_read_lock = rc_player.try_read();

                if let Ok(player_container) = try_read_lock {
                    player_container
                        .emit_state_machine_observer_on_custom_message(message.to_string());
                }
            }
        }
        if let Ok(observers) = self.observers.try_read() {
            for observer in observers.iter() {
                observer.on_custom_event(message.to_string());
            }
        }
    }

    pub fn observe_on_error(&self, message: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(rc_player) = &self.player {
                let try_read_lock = rc_player.try_read();

                if let Ok(player_container) = try_read_lock {
                    player_container.emit_state_machine_observer_on_error(message.to_string());
                }
            }
        }
        if let Ok(observers) = self.observers.try_read() {
            for observer in observers.iter() {
                observer.on_error(message.to_string());
            }
        }
    }
}

unsafe impl Send for StateMachineEngine {}
unsafe impl Sync for StateMachineEngine {}
