use std::collections::{HashMap, HashSet};

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
    Boolean(bool),
    String(String),
    Event(String),
}

pub struct InputManager {
    pub(super) numeric: HashMap<String, (f32, f32)>,
    pub(super) boolean: HashMap<String, (bool, bool)>,
    pub(super) string: HashMap<String, (String, String)>,
    pub(super) event: HashSet<String>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            numeric: HashMap::new(),
            boolean: HashMap::new(),
            string: HashMap::new(),
            event: HashSet::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.numeric.len() + self.boolean.len() + self.string.len() + self.event.len()
    }

    pub fn is_empty(&self) -> bool {
        self.numeric.is_empty()
            && self.boolean.is_empty()
            && self.string.is_empty()
            && self.event.is_empty()
    }

    pub fn set_initial_numeric(&mut self, key: &str, value: f32) {
        self.numeric.insert(key.to_string(), (value, value));
    }

    pub fn set_initial_string(&mut self, key: &str, value: &str) {
        self.string
            .insert(key.to_string(), (value.to_string(), value.to_string()));
    }

    pub fn set_initial_boolean(&mut self, key: &str, value: bool) {
        self.boolean.insert(key.to_string(), (value, value));
    }

    pub fn set_initial_event(&mut self, key: &str) {
        self.event.insert(key.to_string());
    }

    pub fn set_numeric(&mut self, key: &str, value: f32) -> Option<f32> {
        let (current, _) = self.numeric.get_mut(key)?;
        let old = *current;
        *current = value;
        Some(old)
    }

    pub fn set_boolean(&mut self, key: &str, value: bool) -> Option<bool> {
        let (current, _) = self.boolean.get_mut(key)?;
        let old = *current;
        *current = value;
        Some(old)
    }

    pub fn set_string(&mut self, key: &str, value: &str) -> Option<String> {
        let (current, _) = self.string.get_mut(key)?;
        Some(std::mem::replace(current, value.to_string()))
    }

    pub fn get_numeric(&self, key: &str) -> Option<f32> {
        self.numeric.get(key).map(|(v, _)| *v)
    }

    pub fn get_boolean(&self, key: &str) -> Option<bool> {
        self.boolean.get(key).map(|(v, _)| *v)
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.string.get(key).map(|(v, _)| v.as_str())
    }

    pub fn get_event(&self, key: &str) -> Option<&str> {
        self.event.get(key).map(|s| s.as_str())
    }

    pub fn reset(&mut self, key: &str) -> Option<(InputValue, InputValue)> {
        if let Some((current, default)) = self.numeric.get_mut(key) {
            let old = InputValue::Numeric(*current);
            *current = *default;
            return Some((old, InputValue::Numeric(*default)));
        }
        if let Some((current, default)) = self.boolean.get_mut(key) {
            let old = InputValue::Boolean(*current);
            *current = *default;
            return Some((old, InputValue::Boolean(*default)));
        }
        if let Some((current, default)) = self.string.get_mut(key) {
            let new_val = default.clone();
            let old_val = std::mem::replace(current, new_val.clone());
            return Some((InputValue::String(old_val), InputValue::String(new_val)));
        }
        None
    }
}
