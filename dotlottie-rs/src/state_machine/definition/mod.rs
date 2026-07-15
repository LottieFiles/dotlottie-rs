use serde::{Deserialize, Deserializer};

use crate::string::{DotString, DotStringInterner};

use super::{
    inputs::Input,
    interactions::Interaction,
    states::{State, StateTrait},
    GLOBAL_INPUT_PREFIX,
};

// Drop @-prefixed user declarations at deserialization. The @ namespace is reserved for built-ins
fn deserialize_user_inputs<'de, D>(deserializer: D) -> Result<Option<Vec<Input>>, D::Error>
where
    D: Deserializer<'de>,
{
    let inputs: Option<Vec<Input>> = Option::deserialize(deserializer)?;
    Ok(inputs.map(|mut v| {
        v.retain(|input| {
            let name = match input {
                Input::Numeric { name, .. }
                | Input::String { name, .. }
                | Input::Boolean { name, .. }
                | Input::Event { name } => name,
            };
            !name.starts_with(GLOBAL_INPUT_PREFIX)
        });
        v
    }))
}

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
    #[serde(default, deserialize_with = "deserialize_user_inputs")]
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

pub fn state_machine_parse(json: &str) -> Result<StateMachine, super::Error> {
    serde_json::from_str(json).map_err(|err| super::Error::ParsingError(err.to_string()))
}
