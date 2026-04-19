use std::collections::HashMap;

use serde::Deserialize;

use crate::string::DotString;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "type")]
pub enum Input {
    Numeric { name: String, value: f32 },
    String { name: String, value: String },
    Boolean { name: String, value: bool },
    Event { name: String },
}

#[derive(Clone, Debug)]
pub enum InputValue {
    Numeric(f32),
    String(String),
    Boolean(bool),
    Event(String),
}

pub trait InputTrait {
    fn set_initial_boolean(&mut self, key: &str, value: bool);
    fn set_initial_string(&mut self, key: &str, value: String);
    fn set_initial_numeric(&mut self, key: &str, value: f32);
    fn set_initial_event(&mut self, key: &str, value: &str);
    fn new() -> Self;
    fn set_boolean(&mut self, key: &str, value: bool) -> Option<InputValue>;
    fn set_string(&mut self, key: &str, value: String) -> Option<InputValue>;
    fn set_numeric(&mut self, key: &str, value: f32) -> Option<InputValue>;
    fn get_numeric(&self, key: &str) -> Option<f32>;
    fn get_string(&self, key: &str) -> Option<String>;
    fn get_boolean(&self, key: &str) -> Option<bool>;
    fn get_event(&self, key: &str) -> Option<String>;
    fn reset(&mut self, key: &str) -> Option<(InputValue, InputValue)>;
}

pub struct InputManager {
    pub inputs: HashMap<DotString, InputValue>,
    default_values: HashMap<DotString, InputValue>,
}

/// Replace an existing entry's value in-place, or insert a new `DotString`
/// key if missing. Avoids allocating a fresh `DotString` on every update
/// (the hot path — reads/writes against keys that were declared at load
/// time).
fn insert_or_update(
    map: &mut HashMap<DotString, InputValue>,
    key: &str,
    value: InputValue,
) -> Option<InputValue> {
    if let Some(slot) = map.get_mut(key) {
        return Some(std::mem::replace(slot, value));
    }
    map.insert(DotString::new(key), value);
    None
}

impl InputTrait for InputManager {
    fn new() -> Self {
        let inputs = HashMap::new();

        // Store defaults
        let default_values = inputs.clone();

        InputManager {
            inputs,
            default_values,
        }
    }

    fn reset(&mut self, key: &str) -> Option<(InputValue, InputValue)> {
        if let Some(default_value) = self.default_values.get(key) {
            let default_value = default_value.clone();
            let old = insert_or_update(&mut self.inputs, key, default_value.clone())
                .unwrap_or_else(|| default_value.clone());
            return Some((old, default_value));
        }

        None
    }

    fn set_numeric(&mut self, key: &str, value: f32) -> Option<InputValue> {
        insert_or_update(&mut self.inputs, key, InputValue::Numeric(value))
    }

    // Get methods for each type
    fn get_numeric(&self, key: &str) -> Option<f32> {
        match self.inputs.get(key) {
            Some(InputValue::Numeric(value)) => Some(*value),
            _ => None,
        }
    }

    fn set_string(&mut self, key: &str, value: String) -> Option<InputValue> {
        insert_or_update(&mut self.inputs, key, InputValue::String(value))
    }

    fn get_string(&self, key: &str) -> Option<String> {
        match self.inputs.get(key) {
            Some(InputValue::String(value)) => Some(value.clone()),
            _ => None,
        }
    }

    fn set_boolean(&mut self, key: &str, value: bool) -> Option<InputValue> {
        insert_or_update(&mut self.inputs, key, InputValue::Boolean(value))
    }

    fn get_boolean(&self, key: &str) -> Option<bool> {
        match self.inputs.get(key) {
            Some(InputValue::Boolean(value)) => Some(*value),
            _ => None,
        }
    }

    fn get_event(&self, key: &str) -> Option<String> {
        match self.inputs.get(key) {
            Some(InputValue::Event(value)) => Some(value.clone()),
            _ => None,
        }
    }

    fn set_initial_numeric(&mut self, key: &str, value: f32) {
        insert_or_update(&mut self.inputs, key, InputValue::Numeric(value));
        insert_or_update(&mut self.default_values, key, InputValue::Numeric(value));
    }

    fn set_initial_string(&mut self, key: &str, value: String) {
        insert_or_update(&mut self.inputs, key, InputValue::String(value.clone()));
        insert_or_update(&mut self.default_values, key, InputValue::String(value));
    }

    fn set_initial_boolean(&mut self, key: &str, value: bool) {
        insert_or_update(&mut self.inputs, key, InputValue::Boolean(value));
        insert_or_update(&mut self.default_values, key, InputValue::Boolean(value));
    }

    fn set_initial_event(&mut self, key: &str, value: &str) {
        insert_or_update(
            &mut self.inputs,
            key,
            InputValue::Event(value.to_string()),
        );
    }
}
