use std::{collections::HashMap, rc::Rc, sync::RwLock};

use serde::Deserialize;

use crate::DotLottiePlayerContainer;

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
}

pub trait InputTrait {
    fn resolve_numeric(&self, key: &str) -> Option<f32>;
    fn resolve_boolean(&self, key: &str) -> Option<bool>;
    fn resolve_string(&self, key: &str) -> Option<String>;
    fn set_initial_boolean(&mut self, key: &str, value: bool);
    fn set_initial_string(&mut self, key: &str, value: String);
    fn set_initial_numeric(&mut self, key: &str, value: f32);
    fn new(player: Option<Rc<RwLock<DotLottiePlayerContainer>>>) -> Self;
    fn set_boolean(&mut self, key: &str, value: bool) -> Option<InputValue>;
    fn set_string(&mut self, key: &str, value: String) -> Option<InputValue>;
    fn set_numeric(&mut self, key: &str, value: f32) -> Option<InputValue>;
    fn get_numeric(&self, key: &str) -> Option<f32>;
    fn get_string(&self, key: &str) -> Option<String>;
    fn get_boolean(&self, key: &str) -> Option<bool>;
    fn get(&self, key: &str) -> Result<&InputValue, &'static str>;
    fn reset_all(&mut self);
    fn reset(&mut self, key: &str) -> Option<(InputValue, InputValue)>;
}

pub struct InputManager {
    inputs: HashMap<String, InputValue>,
    default_values: HashMap<String, InputValue>,
    pub player: Option<Rc<RwLock<DotLottiePlayerContainer>>>,
}

impl InputTrait for InputManager {
    fn new(player: Option<Rc<RwLock<DotLottiePlayerContainer>>>) -> Self {
        let inputs = HashMap::new();

        // Store defaults
        let default_values = inputs.clone();

        InputManager {
            inputs,
            default_values,
            player,
        }
    }

    fn reset(&mut self, key: &str) -> Option<(InputValue, InputValue)> {
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

    // Resolve a string input either from the State machine inputs or the data bindings
    // Return value: If neither the input or data bind was found, returns None
    fn resolve_string(&self, key: &str) -> Option<String> {
        match key.strip_prefix('@') {
            Some(binding_name) => self
                .player
                .as_ref()
                .and_then(|p| p.try_read().ok())
                .and_then(|p| p.global_inputs_get_text(binding_name)),
            None => key
                .strip_prefix('$')
                .and_then(|binding_name| self.get_string(binding_name)),
        }
    }

    // Resolve a boolean input either from the State machine inputs or the data bindings
    // Return value: If neither the input or data bind was found, returns None
    fn resolve_boolean(&self, key: &str) -> Option<bool> {
        match key.strip_prefix('@') {
            Some(binding_name) => self
                .player
                .as_ref()
                .and_then(|p| p.try_read().ok())
                .and_then(|p| p.global_inputs_get_boolean(binding_name)),
            None => key
                .strip_prefix('$')
                .and_then(|binding_name| self.get_boolean(binding_name)),
        }
    }

    // Resolve a numeric input either from the State machine inputs or the data bindings
    // Return value: If neither the input or data bind was found, returns None
    fn resolve_numeric(&self, key: &str) -> Option<f32> {
        match key.strip_prefix('@') {
            Some(binding_name) => self
                .player
                .as_ref()
                .and_then(|p| p.try_read().ok())
                .and_then(|p| p.global_inputs_get_scalar(binding_name))
                .map(|v| v as f32),
            None => key
                .strip_prefix('$')
                .and_then(|binding_name| self.get_numeric(binding_name))
                .map(|v| v as f32),
        }
    }

    fn reset_all(&mut self) {
        self.inputs = self.default_values.clone();
    }

    fn set_numeric(&mut self, key: &str, value: f32) -> Option<InputValue> {
        self.inputs
            .insert(key.to_string(), InputValue::Numeric(value))
    }

    // Get methods for each type
    fn get_numeric(&self, key: &str) -> Option<f32> {
        match self.inputs.get(key) {
            Some(InputValue::Numeric(value)) => Some(*value),
            _ => None,
        }
    }

    fn set_string(&mut self, key: &str, value: String) -> Option<InputValue> {
        self.inputs
            .insert(key.to_string(), InputValue::String(value))
    }

    fn get_string(&self, key: &str) -> Option<String> {
        match self.inputs.get(key) {
            Some(InputValue::String(value)) => Some(value.clone()),
            _ => None,
        }
    }

    fn set_boolean(&mut self, key: &str, value: bool) -> Option<InputValue> {
        self.inputs
            .insert(key.to_string(), InputValue::Boolean(value))
    }

    fn get_boolean(&self, key: &str) -> Option<bool> {
        match self.inputs.get(key) {
            Some(InputValue::Boolean(value)) => Some(*value),
            _ => None,
        }
    }

    fn set_initial_numeric(&mut self, key: &str, value: f32) {
        self.inputs
            .insert(key.to_string(), InputValue::Numeric(value));

        self.default_values
            .insert(key.to_string(), InputValue::Numeric(value));
    }

    fn set_initial_string(&mut self, key: &str, value: String) {
        self.inputs
            .insert(key.to_string(), InputValue::String(value.clone()));

        self.default_values
            .insert(key.to_string(), InputValue::String(value.clone()));
    }

    fn set_initial_boolean(&mut self, key: &str, value: bool) {
        self.inputs
            .insert(key.to_string(), InputValue::Boolean(value));

        self.default_values
            .insert(key.to_string(), InputValue::Boolean(value));
    }

    // Generic get method that returns a Result
    fn get(&self, key: &str) -> Result<&InputValue, &'static str> {
        self.inputs.get(key).ok_or("Input key not found")
    }
}
