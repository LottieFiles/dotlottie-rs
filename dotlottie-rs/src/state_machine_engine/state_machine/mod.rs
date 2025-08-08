use serde::Deserialize;

use crate::errors::StateMachineError;

use super::{
    inputs::Input,
    interactions::Interaction,
    states::{State, StateTrait},
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
pub enum StringBool {
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StringNumber {
    String(String),
    F32(f32),
}

#[derive(Deserialize, Debug)]
pub struct StateMachine {
    pub initial: String,
    pub states: Vec<State>,
    pub interactions: Option<Vec<Interaction>>,
    pub inputs: Option<Vec<Input>>,
}

impl StateMachine {
    pub fn new(
        initial: String,
        states: Vec<State>,
        interactions: Option<Vec<Interaction>>,
        inputs: Option<Vec<Input>>,
    ) -> Self {
        StateMachine {
            initial,
            states,
            interactions,
            inputs,
        }
    }

    pub fn states(&self) -> &Vec<State> {
        &self.states
    }

    pub fn interactions(&self) -> Option<&Vec<Interaction>> {
        self.interactions.as_ref()
    }

    pub fn inputs(&self) -> Option<&Vec<Input>> {
        self.inputs.as_ref()
    }

    pub fn get_state_by_name(&self, name: &str) -> Option<&State> {
        self.states.iter().find(|state| state.name() == name)
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        StateMachine {
            initial: "".to_string(),
            states: Vec::new(),
            interactions: None,
            inputs: None,
        }
    }
}

pub fn state_machine_parse(json: &str) -> Result<StateMachine, StateMachineError> {
    let result: Result<StateMachine, _> = serde_json::from_str(json);

    match result {
        Ok(k) => Ok(k),
        Err(err) => Err(StateMachineError::ParsingError(err.to_string())),
    }
}
