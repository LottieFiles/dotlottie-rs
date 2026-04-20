use serde::Deserialize;

use crate::errors::StateMachineError;
use crate::string::{DotString, DotStringInterner};

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

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StringString {
    String(String),
}

#[derive(Deserialize, Debug, Default)]
pub struct StateMachine {
    pub initial: DotString,
    pub states: Vec<State>,
    pub interactions: Option<Vec<Interaction>>,
    pub inputs: Option<Vec<Input>>,
}

impl StateMachine {
    pub fn new(
        initial: DotString,
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

    /// Canonicalize every identifier through the shared interner so runtime
    /// comparisons hit the `Arc::ptr_eq` fast path.
    pub(crate) fn intern_identifiers(&mut self, interner: &mut DotStringInterner) {
        self.initial = interner.intern(self.initial.as_str());
        for state in &mut self.states {
            state.intern_identifiers(interner);
        }
        if let Some(interactions) = &mut self.interactions {
            for i in interactions {
                i.intern_identifiers(interner);
            }
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
