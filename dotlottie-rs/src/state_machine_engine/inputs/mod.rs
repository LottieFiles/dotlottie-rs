use std::collections::HashMap;

use serde::Deserialize;

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
    fn get(&self, key: &str) -> Result<&InputValue, &'static str>;
    fn reset_all(&mut self);
    fn reset(&mut self, key: &str) -> Option<(InputValue, InputValue)>;
}

pub struct InputManager {
    pub inputs: HashMap<String, InputValue>,
    default_values: HashMap<String, InputValue>,
}

impl InputManager {
    pub fn new() -> Self {
        let inputs = HashMap::new();

        // Store defaults
        let default_values = inputs.clone();

        InputManager {
            inputs,
            default_values,
        }
    }

    pub fn reset(&mut self, key: &str) -> Option<(InputValue, InputValue)> {
        if let Some(default_value) = self.default_values.get(key) {
            return Some((
                self.inputs
                    .insert(key.to_string(), default_value.clone())
                    .unwrap_or(default_value.clone()),
                default_value.clone(),
            ));
        }

        None
    }

    pub fn reset_all(&mut self) {
        self.inputs = self.default_values.clone();
    }

    pub fn set_numeric(&mut self, key: &str, value: f32) -> Option<InputValue> {
        self.inputs
            .insert(key.to_string(), InputValue::Numeric(value))
    }

    // Get methods for each type
    pub fn get_numeric(&self, key: &str) -> Option<f32> {
        match self.inputs.get(key) {
            Some(InputValue::Numeric(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn set_string(&mut self, key: &str, value: String) -> Option<InputValue> {
        self.inputs
            .insert(key.to_string(), InputValue::String(value))
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.inputs.get(key) {
            Some(InputValue::String(value)) => Some(value.clone()),
            _ => None,
        }
    }

    pub fn set_boolean(&mut self, key: &str, value: bool) -> Option<InputValue> {
        self.inputs
            .insert(key.to_string(), InputValue::Boolean(value))
    }

    pub fn get_boolean(&self, key: &str) -> Option<bool> {
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
        self.inputs
            .insert(key.to_string(), InputValue::Numeric(value));

        self.default_values
            .insert(key.to_string(), InputValue::Numeric(value));
    }

    pub fn set_initial_string(&mut self, key: &str, value: String) {
        self.inputs
            .insert(key.to_string(), InputValue::String(value.clone()));

        self.default_values
            .insert(key.to_string(), InputValue::String(value.clone()));
    }

    pub fn set_initial_boolean(&mut self, key: &str, value: bool) {
        self.inputs
            .insert(key.to_string(), InputValue::Boolean(value));

        self.default_values
            .insert(key.to_string(), InputValue::Boolean(value));
    }

    fn set_initial_event(&mut self, key: &str, value: &str) {
        self.inputs
            .insert(key.to_string(), InputValue::Event(value.to_string()));
    }

    // Generic get method that returns a Result
    pub fn get(&self, key: &str) -> Result<&InputValue, &'static str> {
        self.inputs.get(key).ok_or("Input key not found")
    }

    pub fn get_all_input_names(&self) -> Vec<String> {
        self.inputs.keys().cloned().collect()
    }
}
