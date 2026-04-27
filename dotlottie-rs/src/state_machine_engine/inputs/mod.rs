use {rustc_hash::FxHashMap, rustc_hash::FxHashSet};

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
    Boolean(bool),
    String(String),
    Event(String),
}

pub struct InputManager {
    pub(super) numeric: FxHashMap<DotString, (f32, f32)>,
    pub(super) boolean: FxHashMap<DotString, (bool, bool)>,
    pub(super) string: FxHashMap<DotString, (String, String)>,
    pub(super) event: FxHashSet<DotString>,
}

/// Replace an existing entry's value in-place, or insert a new `DotString`
/// key if missing. Avoids allocating a fresh `DotString` on every update
/// (the hot path — reads/writes against keys that were declared at load
/// time).
fn insert_or_update<V>(map: &mut FxHashMap<DotString, V>, key: &str, value: V) -> Option<V> {
    if let Some(slot) = map.get_mut(key) {
        return Some(std::mem::replace(slot, value));
    }
    map.insert(DotString::new(key), value);
    None
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            numeric: FxHashMap::default(),
            boolean: FxHashMap::default(),
            string: FxHashMap::default(),
            event: FxHashSet::default(),
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
        insert_or_update(&mut self.numeric, key, (value, value));
    }

    pub fn set_initial_string(&mut self, key: &str, value: &str) {
        insert_or_update(
            &mut self.string,
            key,
            (value.to_string(), value.to_string()),
        );
    }

    pub fn set_initial_boolean(&mut self, key: &str, value: bool) {
        insert_or_update(&mut self.boolean, key, (value, value));
    }

    pub fn set_initial_event(&mut self, key: &str) {
        self.event.insert(DotString::new(key));
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

    pub fn set_string(&mut self, key: &str, value: String) -> Option<String> {
        let (current, _) = self.string.get_mut(key)?;
        Some(std::mem::replace(current, value))
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
