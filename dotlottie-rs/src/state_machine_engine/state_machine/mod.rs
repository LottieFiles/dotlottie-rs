use serde::Deserialize;

use crate::errors::StateMachineError;

use super::{
    listeners::Listener,
    states::{State, StateTrait},
    triggers::Trigger,
};

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StringNumberBool {
    String(String),
    F32(f32),
    Bool(bool),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StringNumber {
    String(String),
    F32(f32),
}

#[derive(Deserialize, Debug)]
pub struct Descriptor {
    pub id: String,
    pub initial: String,
}

#[derive(Deserialize, Debug)]
pub struct StateMachine {
    pub descriptor: Descriptor,
    pub states: Vec<State>,
    pub listeners: Option<Vec<Listener>>,
    pub triggers: Option<Vec<Trigger>>,
}

impl StateMachine {
    pub fn default() -> Self {
        StateMachine {
            descriptor: Descriptor {
                id: "".to_string(),
                initial: "".to_string(),
            },
            states: Vec::new(),
            listeners: None,
            triggers: None,
        }
    }

    pub fn new(
        descriptor: Descriptor,
        states: Vec<State>,
        listeners: Option<Vec<Listener>>,
        triggers: Option<Vec<Trigger>>,
    ) -> Self {
        StateMachine {
            descriptor,
            states,
            listeners,
            triggers,
        }
    }

    pub fn states(&self) -> &Vec<State> {
        &self.states
    }

    pub fn listeners(&self) -> Option<&Vec<Listener>> {
        self.listeners.as_ref()
    }

    pub fn triggers(&self) -> Option<&Vec<Trigger>> {
        self.triggers.as_ref()
    }

    pub fn get_state_by_name(&self, name: &str) -> Option<&State> {
        self.states.iter().find(|state| state.name() == name)
    }
}

pub fn state_machine_parse(json: &str) -> Result<StateMachine, StateMachineError> {
    let result: Result<StateMachine, _> = serde_json::from_str(json);

    match result {
        Ok(k) => Ok(k),
        Err(err) => Err(StateMachineError::ParsingError {
            reason: err.to_string(),
        }),
    }
}
