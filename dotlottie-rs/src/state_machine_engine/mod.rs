use std::collections::HashMap;
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
    #[error("Failed to parse JSON state machine definition")]
    ParsingError { reason: String },

    #[error("Failed to create StateMachineEngine")]
    CreationError { reason: String },
}

pub struct StateMachineEngine {
    pub listeners: Vec<Listener>,

    /* We keep references to the StateMachine's States. */
    /* This prevents duplicating the data inside the engine. */
    // pub states: HashMap<String, Rc<State>>,
    pub global_state: Option<Rc<State>>,
    pub current_state: Option<Rc<State>>,

    pub player: Option<Rc<RwLock<DotLottiePlayerContainer>>>,
    pub status: StateMachineEngineStatus,

    numeric_trigger: HashMap<String, f32>,
    string_trigger: HashMap<String, String>,
    bool_trigger: HashMap<String, bool>,
    event_trigger: HashMap<String, String>,

    observers: RwLock<Vec<Rc<dyn StateMachineObserver>>>,

    state_machine: StateMachine,
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
            bool_trigger: HashMap::new(),
            event_trigger: HashMap::new(),
            status: StateMachineEngineStatus::Stopped,
            observers: RwLock::new(Vec::new()),
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
            .field("bool_trigger", &self.bool_trigger)
            .field("event_trigger", &self.event_trigger)
            .field("status", &self.status)
            .finish()
    }
}

impl StateMachineEngine {
    pub fn new(
        state_machine_definition: &str,
        player: Rc<RwLock<DotLottiePlayerContainer>>,
    ) -> Result<StateMachineEngine, StateMachineEngineError> {
        let mut state_machine = StateMachineEngine {
            global_state: None,
            // states: HashMap::new(),
            state_machine: StateMachine::default(),
            listeners: Vec::new(),
            current_state: None,
            player: Some(player.clone()),
            numeric_trigger: HashMap::new(),
            string_trigger: HashMap::new(),
            bool_trigger: HashMap::new(),
            event_trigger: HashMap::new(),
            status: StateMachineEngineStatus::Stopped,
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
                /* Build all trigger variables into hashmaps for easier use */
                if let Some(triggers) = parsed_state_machine.triggers {
                    for trigger in triggers {
                        match trigger {
                            Trigger::Numeric { name, value } => {
                                new_state_machine.set_numeric_trigger(&name, value);
                            }
                            Trigger::String { name, value } => {
                                new_state_machine.set_string_trigger(&name, &value);
                            }
                            Trigger::Boolean { name, value } => {
                                new_state_machine.set_bool_trigger(&name, value);
                            }
                            Trigger::Event { name } => {
                                new_state_machine.event_trigger.insert(name, "".to_string());
                            }
                        }
                    }
                }

                /* Setup the global & initial state */
                let initial_state_index = parsed_state_machine.descriptor.initial;

                for state in &parsed_state_machine.states {
                    match state {
                        State::GlobalState { name, .. } => {
                            if name == &initial_state_index {
                                new_state_machine.current_state = Some(Rc::new(state.clone()));
                            }

                            new_state_machine.global_state = Some(Rc::new(state.clone()));
                        }
                        State::PlaybackState { name, .. } => {
                            if name == &initial_state_index {
                                new_state_machine.current_state = Some(Rc::new(state.clone()));
                            }
                        }
                    }
                }

                Ok(new_state_machine)
            }
            Err(_) => todo!(),
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
        self.status = StateMachineEngineStatus::Running;
        // self.execute_current_state();
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

    // Return codes
    // 0: Success
    // 1: Failure
    // 2: Play animation
    // 3: Pause animation
    // 4: Request and draw a new single frame of the animation (needed for sync state)
    pub fn execute_current_state(&mut self) -> i32 {
        0
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
}

unsafe impl Send for StateMachineEngine {}
unsafe impl Sync for StateMachineEngine {}
