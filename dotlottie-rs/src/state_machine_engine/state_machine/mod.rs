use serde::Deserialize;

use crate::errors::StateMachineError;

use super::{listeners::Listener, states::State, triggers::Trigger};

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StringNumberBool {
    String(String),
    F32(f32),
    Bool(bool),
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
