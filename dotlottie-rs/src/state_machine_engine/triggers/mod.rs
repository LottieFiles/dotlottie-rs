use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "type")]
pub enum Trigger {
    Numeric { name: String, value: f32 },
    String { name: String, value: String },
    Boolean { name: String, value: bool },
    Event { name: String },
}

#[derive(Clone, Debug)]
pub enum TriggerValue {
    Numeric(f32),
    String(String),
    Boolean(bool),
}

pub trait TriggerTrait {
    fn set_initial_boolean(&mut self, key: &str, value: bool);
    fn set_initial_string(&mut self, key: &str, value: String);
    fn set_initial_numeric(&mut self, key: &str, value: f32);
    fn new() -> Self;
    fn set_boolean(&mut self, key: &str, value: bool) -> Option<TriggerValue>;
    fn set_string(&mut self, key: &str, value: String) -> Option<TriggerValue>;
    fn set_numeric(&mut self, key: &str, value: f32) -> Option<TriggerValue>;
    fn get_numeric(&self, key: &str) -> Option<f32>;
    fn get_string(&self, key: &str) -> Option<String>;
    fn get_boolean(&self, key: &str) -> Option<bool>;
    fn get(&self, key: &str) -> Result<&TriggerValue, &'static str>;
    fn reset_all(&mut self);
    fn reset(&mut self, key: &str) -> Option<(TriggerValue, TriggerValue)>;
}

pub struct TriggerManager {
    triggers: HashMap<String, TriggerValue>,
    default_values: HashMap<String, TriggerValue>,
}

impl TriggerTrait for TriggerManager {
    fn new() -> Self {
        let triggers = HashMap::new();

        // Store defaults
        let default_values = triggers.clone();

        TriggerManager {
            triggers,
            default_values,
        }
    }

    fn reset(&mut self, key: &str) -> Option<(TriggerValue, TriggerValue)> {
        if let Some(default_value) = self.default_values.get(key) {
            return Some((
                self.triggers.insert(key.to_string(), default_value.clone()).unwrap_or(default_value.clone()),
                default_value.clone(),
            ));
        }

        None
    }

    fn reset_all(&mut self) {
        self.triggers = self.default_values.clone();
    }

    fn set_numeric(&mut self, key: &str, value: f32) -> Option<TriggerValue> {
        self.triggers
            .insert(key.to_string(), TriggerValue::Numeric(value))
    }

    // Get methods for each type
    fn get_numeric(&self, key: &str) -> Option<f32> {
        match self.triggers.get(key) {
            Some(TriggerValue::Numeric(value)) => Some(*value),
            _ => None,
        }
    }

    fn set_string(&mut self, key: &str, value: String) -> Option<TriggerValue> {
        self.triggers
            .insert(key.to_string(), TriggerValue::String(value))
    }

    fn get_string(&self, key: &str) -> Option<String> {
        match self.triggers.get(key) {
            Some(TriggerValue::String(value)) => Some(value.clone()),
            _ => None,
        }
    }

    fn set_boolean(&mut self, key: &str, value: bool) -> Option<TriggerValue> {
        self.triggers
            .insert(key.to_string(), TriggerValue::Boolean(value))
    }

    fn get_boolean(&self, key: &str) -> Option<bool> {
        match self.triggers.get(key) {
            Some(TriggerValue::Boolean(value)) => Some(*value),
            _ => None,
        }
    }

    fn set_initial_numeric(&mut self, key: &str, value: f32) {
        self.triggers
            .insert(key.to_string(), TriggerValue::Numeric(value));

        self.default_values
            .insert(key.to_string(), TriggerValue::Numeric(value));
    }

    fn set_initial_string(&mut self, key: &str, value: String) {
        self.triggers
            .insert(key.to_string(), TriggerValue::String(value.clone()));

        self.default_values
            .insert(key.to_string(), TriggerValue::String(value.clone()));
    }

    fn set_initial_boolean(&mut self, key: &str, value: bool) {
        self.triggers
            .insert(key.to_string(), TriggerValue::Boolean(value));

        self.default_values
            .insert(key.to_string(), TriggerValue::Boolean(value));
    }

    // Generic get method that returns a Result
    fn get(&self, key: &str) -> Result<&TriggerValue, &'static str> {
        self.triggers.get(key).ok_or("Trigger key not found")
    }
}
