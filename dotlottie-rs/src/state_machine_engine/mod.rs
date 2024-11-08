use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
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
use triggers::Trigger;

use crate::state_machine_engine::listeners::Listener;
use crate::{
    state_machine_state_check_pipeline, DotLottiePlayerContainer, EventName, PointerEvent,
    StateMachineEngineSecurityError,
};

use self::state_machine::state_machine_parse;
use self::{events::Event, states::State};

pub trait StateMachineObserver: Send + Sync {
    fn on_transition(&self, previous_state: String, new_state: String);
    fn on_state_entered(&self, entering_state: String);
    fn on_state_exit(&self, leaving_state: String);
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
    // pub listeners: Vec<Listener>,

    /* We keep references to the StateMachine's States. */
    /* This prevents duplicating the data inside the engine. */
    pub global_state: Option<Rc<State>>,
    pub current_state: Option<Rc<State>>,

    pub player: Option<Rc<RwLock<DotLottiePlayerContainer>>>,
    pub status: StateMachineEngineStatus,

    numeric_trigger: HashMap<String, f32>,
    string_trigger: HashMap<String, String>,
    boolean_trigger: HashMap<String, bool>,
    event_trigger: HashMap<String, String>,

    observers: RwLock<Vec<Arc<dyn StateMachineObserver>>>,

    state_machine: StateMachine,

    state_history: Vec<String>,
    max_cycle_count: usize,
    current_cycle_count: usize,
    action_mutated_triggers: bool,

    action_fired_event: Option<String>,
}

impl Default for StateMachineEngine {
    fn default() -> StateMachineEngine {
        StateMachineEngine {
            global_state: None,
            state_machine: StateMachine::default(),
            current_state: None,
            player: None,
            numeric_trigger: HashMap::new(),
            string_trigger: HashMap::new(),
            boolean_trigger: HashMap::new(),
            event_trigger: HashMap::new(),
            status: StateMachineEngineStatus::Stopped,
            observers: RwLock::new(Vec::new()),
            state_history: Vec::new(),
            max_cycle_count: 20,
            current_cycle_count: 0,
            action_mutated_triggers: false,
            action_fired_event: None,
        }
    }
}

impl Display for StateMachineEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMachine")
            .field("global_state", &self.global_state)
            // .field("states", &self.states)
            .field("listeners", &self.state_machine.listeners)
            .field("current_state", &self.current_state)
            .field("numeric_trigger", &self.numeric_trigger)
            .field("string_trigger", &self.string_trigger)
            .field("boolean_trigger", &self.boolean_trigger)
            .field("event_trigger", &self.event_trigger)
            .field("status", &self.status)
            .finish()
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
            numeric_trigger: HashMap::new(),
            string_trigger: HashMap::new(),
            boolean_trigger: HashMap::new(),
            event_trigger: HashMap::new(),
            status: StateMachineEngineStatus::Stopped,
            observers: RwLock::new(Vec::new()),
            state_history: Vec::new(),
            max_cycle_count: max_cycle_count.unwrap_or(20),
            current_cycle_count: 0,
            action_mutated_triggers: false,
            action_fired_event: None,
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

    pub fn get_numeric_trigger(&self, key: &str) -> Option<f32> {
        self.numeric_trigger.get(key).cloned()
    }

    pub fn get_string_trigger(&self, key: &str) -> Option<String> {
        self.string_trigger.get(key).cloned()
    }

    pub fn get_boolean_trigger(&self, key: &str) -> Option<bool> {
        self.boolean_trigger.get(key).cloned()
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
    ) -> Option<f32> {
        let ret = self.numeric_trigger.insert(key.to_string(), value);

        if called_from_action {
            self.action_mutated_triggers = true;
        }

        if run_pipeline {
            let _ = self.run_current_state_pipeline(None);
        }
        ret
    }

    pub fn set_string_trigger(
        &mut self,
        key: &str,
        value: &str,
        run_pipeline: bool,
        called_from_action: bool,
    ) -> Option<String> {
        let ret = self
            .string_trigger
            .insert(key.to_string(), value.to_string());

        if called_from_action {
            self.action_mutated_triggers = true;
        }
        if run_pipeline {
            let _ = self.run_current_state_pipeline(None);
        }

        ret
    }

    pub fn set_boolean_trigger(
        &mut self,
        key: &str,
        value: bool,
        run_pipeline: bool,
        called_from_action: bool,
    ) -> Option<bool> {
        let ret = self.boolean_trigger.insert(key.to_string(), value);

        if called_from_action {
            self.action_mutated_triggers = true;
        }
        if run_pipeline {
            let _ = self.run_current_state_pipeline(None);
        }

        ret
    }

    pub fn fire(&mut self, event: &str, run_pipeline: bool) -> Result<(), StateMachineEngineError> {
        if let Some(_event) = self.event_trigger.get(event) {
            if run_pipeline {
                let _ = self.run_current_state_pipeline(Some(&event.to_string()));
            } else {
                self.action_fired_event = Some(event.to_string());
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
                let initial_state_index = parsed_state_machine.descriptor.initial.clone();

                /* Build all trigger variables into hashmaps for easier use */
                if let Some(triggers) = &parsed_state_machine.triggers {
                    for trigger in triggers {
                        match trigger {
                            Trigger::Numeric { name, value } => {
                                new_state_machine.set_numeric_trigger(&name, *value, false, false);
                            }
                            Trigger::String { name, value } => {
                                new_state_machine.set_string_trigger(&name, &value, false, false);
                            }
                            Trigger::Boolean { name, value } => {
                                new_state_machine.set_boolean_trigger(&name, *value, false, false);
                            }
                            Trigger::Event { name } => {
                                new_state_machine
                                    .event_trigger
                                    .insert(name.to_string(), "".to_string());
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

                let err = new_state_machine.set_current_state(&initial_state_index);
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

        // Todo: Report errors in proper way

        // Todo: Run a checking pipeline
        // - Check all state names are unique
        // - Check for infinite loops
        // - Check for unreachable states
        // - Check for unreachable transitions
        // self.runCheckingPipeline(state_machine);

        // Todo: Implement the restore action. Save the original values of triggers.
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

        let _ = self.run_current_state_pipeline(None);
    }

    pub fn pause(&mut self) {
        self.status = StateMachineEngineStatus::Paused;
    }

    pub fn end(&mut self) {
        self.status = StateMachineEngineStatus::Stopped;
    }

    pub fn get_current_state(&self) -> Option<Rc<State>> {
        self.current_state.clone()
    }

    pub fn listeners(&self, filter: Option<String>) -> Vec<&Listener> {
        let mut listeners_clone = Vec::new();
        let filter = filter.unwrap_or("".to_string());

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
    fn set_current_state(&mut self, state_name: &str) -> Result<(), StateMachineEngineError> {
        let new_state = self.get_state(&state_name);

        if new_state.is_some() {
            // We have a new state

            // Perform exit actions on the current state if there is one.
            if self.current_state.is_some() {
                let state = self.current_state.take();
                let player = self.player.take();

                // Now use the extracted information
                if let (Some(state), Some(player)) = (state, player) {
                    let _ = state.exit(self, &player);

                    // Don't forget to put things back
                    // new_state becomes the current state
                    self.current_state = Some(state);
                    self.player = Some(player);
                }
            }

            // // Assign the new state to the current_state
            self.current_state = new_state;

            // Perform entry actions
            // Execute its type of state
            let state = self.current_state.take();
            let player = self.player.take();

            // Now use the extracted information
            if let (Some(state), Some(player)) = (state, player) {
                let _ = state.enter(self, &player);

                state.execute(&player);

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
            // Events might be passed even if not needed because they are passed down to child states.
            else if (transition.transitions_contain_event() && event.is_some())
                || (!transition.transitions_contain_event())
            {
                if let Some(guards) = transition.guards() {
                    let mut all_guards_satisfied = true;

                    for guard in guards {
                        match guard {
                            transitions::guard::Guard::Numeric { .. } => {
                                if !guard.numeric_trigger_is_satisfied(&self.numeric_trigger) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            transitions::guard::Guard::String { .. } => {
                                if !guard.string_trigger_is_satisfied(&self.string_trigger) {
                                    all_guards_satisfied = false;
                                    break;
                                }
                            }
                            transitions::guard::Guard::Boolean { .. } => {
                                if !guard.boolean_trigger_is_satisfied(&self.boolean_trigger) {
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
        return Some(target_state.to_string());
    }

    // Return codes
    // 0: Success
    // 1: Failure
    // 2: Play animation
    // 3: Pause animation
    // 4: Request and draw a new single frame of the animation (needed for sync state)
    pub fn run_current_state_pipeline(
        &mut self,
        event: Option<&String>,
    ) -> Result<(), StateMachineEngineError> {
        // Reset cycle count for each pipeline run
        self.current_cycle_count = 0;
        let mut ignore_global = false;

        // If the state machine is not running, or there is no current state, return an error
        // Otherwise this will block the pipeline in a loop
        if self.status != StateMachineEngineStatus::Running
            || (self.current_state.is_none() && self.global_state.is_none())
        {
            return Err(StateMachineEngineError::NotRunningError);
        }

        // Start drilling down on the current state and it's transitions
        // As long as there are transitions evaluating to true, we continue the loop
        let mut tick = true;

        while tick {
            // Safety fallback to prevent infinite loops
            tick = false;

            // Infinite loop detection
            if let Some(_cycle) = self.detect_cycle() {
                self.current_cycle_count += 1;

                if self.current_cycle_count >= self.max_cycle_count {
                    println!("🚨 Infinite loop detected, ending state machine.");
                    self.end();
                    return Err(StateMachineEngineError::InfiniteLoopError);
                }

                // Clear the history to allow for detecting new cycles
                self.state_history.clear();
            }

            // Record the current state
            if let Some(state) = &self.current_state {
                self.state_history.push(state.name().to_string());
            }

            // Check if there is a global state
            // If there is, evaluate the transitions of the global state first
            if !ignore_global {
                if let Some(state_to_evaluate) = &self.global_state {
                    let target_state = if self.action_fired_event.is_some() {
                        self.evaluate_transitions(
                            state_to_evaluate,
                            self.action_fired_event.as_ref(),
                        )
                    } else {
                        self.evaluate_transitions(
                            state_to_evaluate,
                            if self.action_mutated_triggers {
                                event
                            } else {
                                event
                            },
                        )
                    };

                    // We've consumed the event, set it to None
                    self.action_fired_event = None;
                    self.action_mutated_triggers = false;

                    if let Some(state) = target_state {
                        let success = self.set_current_state(&state);
                        match success {
                            Ok(_) => {
                                if self.action_mutated_triggers {
                                    println!("🚨 Ticking");
                                    tick = true;
                                }
                            }
                            Err(_) => {
                                println!("🚨 Error setting current state");
                                break;
                            }
                        }
                    }
                }
            }

            // Now we evaluate the transitions of the current state
            if let Some(current_state_to_evaluate) = &self.current_state {
                // If there was an action fired event, we need to call evaluate_transitions with it
                // Othwerwise we call it with the event that was passed in if there was one
                let target_state: Option<String> = if self.action_fired_event.is_some() {
                    self.evaluate_transitions(
                        current_state_to_evaluate,
                        self.action_fired_event.as_ref(),
                    )
                } else {
                    // If we're not propagating events, set to else { None }
                    self.evaluate_transitions(
                        current_state_to_evaluate,
                        if self.action_mutated_triggers {
                            // event
                            None
                        } else {
                            event
                        },
                    )
                };

                // We've consumed the event, but the event might be valid for the global state
                // If there is a global state, it will consume and clear it for us
                // If there isn't, we need to clear it here
                if self.global_state.is_none() {
                    self.action_fired_event = None;
                }
                if let Some(state) = target_state {
                    println!("Target State: {}", state);

                    // Rest this boolean so that it reflects correctly if the actions mutated triggers
                    self.action_mutated_triggers = false;

                    let success = self.set_current_state(&state);

                    match success {
                        Ok(_) => {
                            // Since setting the current state modified triggers, we need to
                            // Complete another loop with the Global state included
                            //
                            // If we didn't mutate triggers, we re-evalaute the current state's transitions
                            if self.action_mutated_triggers {
                                tick = true;
                                ignore_global = false;
                            } else {
                                tick = true;
                                ignore_global = true;
                            }
                        }
                        Err(_) => {
                            println!("🚨 Error setting current state");
                            break;
                        }
                    }
                }
            }

            // One of the states fired an event, we need to re-evaluate the pipeline
            if self.action_fired_event.is_some() {
                tick = true;
            }
        }

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

    fn get_correct_pointer_actions_from_listener(
        &self,
        event: &Event,
        layer_name: Option<String>,
        actions: &Vec<Action>,
        x: f32,
        y: f32,
    ) -> Vec<Action> {
        let mut actions_to_execute = Vec::new();

        // User defined a specific layer to check if hit
        if let Some(layer) = layer_name {
            // Check if the layer was hit, otherwise we ignore this listener
            if let Some(rc_player) = &self.player {
                let try_read_lock = rc_player.try_read();

                match try_read_lock {
                    Ok(player_container) => {
                        // If we have a pointer down event, we need to check if the pointer is outside of the layer
                        if let Event::PointerExit { x, y } = event {
                            if !player_container.hit_check(&layer, *x, *y) {
                                for action in actions {
                                    actions_to_execute.push(action.clone());
                                }
                            }
                        } else {
                            // Hit check will return true if the layer was hit
                            if player_container.hit_check(&layer, x, y) {
                                for action in actions {
                                    actions_to_execute.push(action.clone());
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        } else {
            // No layer was specified, add all actions
            for action in actions {
                actions_to_execute.push(action.clone());
            }
        }

        actions_to_execute
    }

    fn manage_pointer_event(&mut self, event: &Event, x: f32, y: f32) {
        let listeners = self.listeners(Some(event.type_name()));

        if listeners.is_empty() {
            return;
        }

        let mut actions_to_execute = Vec::new();

        for listener in listeners {
            let action_vec = self.get_correct_pointer_actions_from_listener(
                event,
                listener.get_layer_name(),
                listener.get_actions(),
                x,
                y,
            );

            // Action vec was moved in to action_to_execute, it can't be used again
            actions_to_execute.extend(action_vec);
        }

        for action in actions_to_execute {
            // Run the pipeline because listeners are outside of the evaluation pipeline loop
            if let Some(player_ref) = &self.player {
                let _ = action.execute(self, player_ref.clone(), true);
            }
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
                        for action in actions {
                            // Clones the reference to action
                            actions_to_execute.push(action.clone());
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

    pub fn get_state_machine(&self) -> &StateMachine {
        &self.state_machine
    }

    pub fn get_current_state_name(&self) -> String {
        if let Some(state) = &self.current_state {
            return state.name();
        }

        "".to_string()
    }
}

unsafe impl Send for StateMachineEngine {}
unsafe impl Sync for StateMachineEngine {}
