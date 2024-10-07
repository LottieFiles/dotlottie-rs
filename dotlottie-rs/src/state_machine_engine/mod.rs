use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::rc::Rc;
use std::sync::RwLock;

pub mod actions;
pub mod errors;
pub mod events;
pub mod listeners;
pub mod state_machine;
pub mod states;
pub mod transitions;
pub mod triggers;

use state_machine::StateMachine;
use states::StateTrait;
use transitions::guard::GuardTrait;
use transitions::TransitionTrait;
use triggers::Trigger;

use crate::state_machine_engine::listeners::Listener;
use crate::DotLottiePlayerContainer;

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
}

pub struct StateMachineEngine {
    pub listeners: Vec<Listener>,

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

    observers: RwLock<Vec<Rc<dyn StateMachineObserver>>>,

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
            // states: HashMap::new(),
            state_machine: StateMachine::default(),
            listeners: Vec::new(),
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
        }
    }
}

impl Display for StateMachineEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMachine")
            .field("global_state", &self.global_state)
            // .field("states", &self.states)
            .field("listeners", &self.listeners)
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
            listeners: Vec::new(),
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

    pub fn get_boolean_trigger(&self, key: &str) -> Option<bool> {
        self.boolean_trigger.get(key).cloned()
    }

    pub fn set_numeric_trigger(
        &mut self,
        key: &str,
        value: f32,
        run_pipeline: bool,
    ) -> Option<f32> {
        let ret = self.numeric_trigger.insert(key.to_string(), value);

        self.action_mutated_triggers = true;

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
    ) -> Option<String> {
        let ret = self
            .string_trigger
            .insert(key.to_string(), value.to_string());

        self.action_mutated_triggers = true;

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
    ) -> Option<bool> {
        let ret = self.boolean_trigger.insert(key.to_string(), value);

        self.action_mutated_triggers = true;

        if run_pipeline {
            let _ = self.run_current_state_pipeline(None);
        }

        ret
    }

    pub fn fire(&mut self, event: &str) -> Result<(), StateMachineEngineError> {
        if let Some(_event) = self.event_trigger.get(event) {
            let _ = self.run_current_state_pipeline(Some(&event.to_string()));

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
                                new_state_machine.set_numeric_trigger(&name, *value, false);
                            }
                            Trigger::String { name, value } => {
                                new_state_machine.set_string_trigger(&name, &value, false);
                            }
                            Trigger::Boolean { name, value } => {
                                new_state_machine.set_boolean_trigger(&name, *value, false);
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

                let err = new_state_machine.set_current_state(&initial_state_index);
                match err {
                    Ok(_) => {}
                    Err(error) => {
                        println!("🚨 Error setting initial state: {:?}", error);
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

    pub fn get_listeners(&self) -> &Vec<Listener> {
        &self.listeners
    }

    fn get_state(&self, state_name: &str) -> Option<Rc<State>> {
        if let Some(global_state) = &self.global_state {
            if global_state.get_name() == state_name {
                return Some(global_state.clone());
            }
        }

        for state in self.state_machine.states.iter() {
            if state.get_name() == state_name {
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
            // Perofrm exit actions on the current state
            if let Some(state) = &self.current_state {
                if let Some(player) = &self.player {
                    // Perform exit actions
                    state.exit(
                        &player,
                        &self.string_trigger,
                        &self.boolean_trigger,
                        &self.numeric_trigger,
                        &self.event_trigger,
                    );
                }
            }

            // Assign the new state to the current_state
            self.current_state = new_state;

            // Perform entry actions
            // Execute its type of state
            let state = self.current_state.take();
            let player = self.player.take();

            // Now use the extracted information
            if let (Some(state), Some(player)) = (state, player) {
                let _ = state.enter(self, &player);

                state.execute(
                    &player,
                    &mut self.string_trigger,
                    &mut self.boolean_trigger,
                    &mut self.numeric_trigger,
                    &mut self.event_trigger,
                );

                // Don't forget to put things back
                self.current_state = Some(state);
                self.player = Some(player);
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
        let transitions = state_to_evaluate.get_transitions();

        for transition in transitions {
            /* If in the transitions we need an event, and there wasn't one fired, don't run the checks */
            if (transition.transitions_contain_event() && event.is_some())
                || (!transition.transitions_contain_event() && event.is_none())
            {
                if let Some(guards) = transition.get_guards() {
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
                        let target_state = transition.get_target_state();

                        return Some(target_state.to_string());
                    }
                }
            }
        }

        None
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
        let mut loop_count = 0;

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
            self.action_mutated_triggers = false;

            // Infinite loop detection
            // Todo: Infinite loop on same state
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
                self.state_history.push(state.get_name().to_string());
            }

            // Check if there is a global state
            // If there is, evaluate the transitions of the global state first
            if let Some(state_to_evaluate) = &self.global_state {
                let target_state = self.evaluate_transitions(state_to_evaluate, event);

                if let Some(state) = target_state {
                    let success = self.set_current_state(&state);
                    match success {
                        Ok(_) => {
                            if self.action_mutated_triggers {
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

            // Now we evaluate the transitions of the current state
            if let Some(current_state_to_evaluate) = &self.current_state {
                let target_state = self.evaluate_transitions(
                    current_state_to_evaluate,
                    if loop_count > 0 { None } else { event },
                );

                if let Some(state) = target_state {
                    let success = self.set_current_state(&state);

                    match success {
                        Ok(_) => {
                            if self.action_mutated_triggers {
                                tick = true;
                            }
                        }
                        Err(_) => {
                            println!("🚨 Error setting current state");
                            break;
                        }
                    }

                    loop_count += 1;
                }
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

    // Return codes
    // 0: Success
    // 1: Failure
    // 2: Play animation
    // 3: Pause animation
    // 4: Request and draw a new single frame of the animation (needed for sync state)
    pub fn post_event(&mut self, event: &Event) -> i32 {
        0
    }

    pub fn get_state_machine(&self) -> &StateMachine {
        &self.state_machine
    }

    pub fn get_current_state_name(&self) -> String {
        if let Some(state) = &self.current_state {
            return state.get_name();
        }

        "".to_string()
    }
}

unsafe impl Send for StateMachineEngine {}
unsafe impl Sync for StateMachineEngine {}
