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

#[derive(Clone, Debug)]
struct InputSlot {
    name: String,
    current: InputValue,
    default: InputValue,
}

pub struct InputManager {
    slots: Vec<InputSlot>,
    sorted: bool,
}

impl InputManager {
    pub fn iter(&self) -> impl Iterator<Item = (&str, &InputValue)> {
        self.slots.iter().map(|s| (s.name.as_ref(), &s.current))
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            sorted: false,
        }
    }

    /// Call once after all `set_initial_*` to enable binary search.
    pub fn freeze(&mut self) {
        self.slots.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        self.sorted = true;
    }

    fn find(&self, key: &str) -> Option<&InputSlot> {
        if self.sorted {
            self.slots
                .binary_search_by(|s| s.name.as_str().cmp(key))
                .ok()
                .map(|i| &self.slots[i])
        } else {
            self.slots.iter().find(|s| s.name.as_str() == key)
        }
    }

    fn find_mut(&mut self, key: &str) -> Option<&mut InputSlot> {
        if self.sorted {
            self.slots
                .binary_search_by(|s| s.name.as_str().cmp(key))
                .ok()
                .map(|i| &mut self.slots[i])
        } else {
            self.slots.iter_mut().find(|s| s.name.as_str() == key)
        }
    }

    fn push_initial(&mut self, name: &str, value: InputValue) {
        self.slots.push(InputSlot {
            name: name.into(),
            current: value.clone(),
            default: value,
        });
        self.sorted = false;
    }

    pub fn set_initial_numeric(&mut self, key: &str, value: f32) {
        self.push_initial(key, InputValue::Numeric(value));
    }

    pub fn set_initial_string(&mut self, key: &str, value: &str) {
        self.push_initial(key, InputValue::String(value.into()));
    }

    pub fn set_initial_boolean(&mut self, key: &str, value: bool) {
        self.push_initial(key, InputValue::Boolean(value));
    }

    pub fn set_numeric(&mut self, key: &str, value: f32) -> Option<f32> {
        let slot = self.find_mut(key)?;
        if let InputValue::Numeric(old) = slot.current {
            slot.current = InputValue::Numeric(value);
            Some(old)
        } else {
            None
        }
    }

    pub fn set_boolean(&mut self, key: &str, value: bool) -> Option<bool> {
        let slot = self.find_mut(key)?;
        if let InputValue::Boolean(old) = slot.current {
            slot.current = InputValue::Boolean(value);
            Some(old)
        } else {
            None
        }
    }

    pub fn set_string(&mut self, key: &str, value: &str) -> Option<String> {
        let slot = self.find_mut(key)?;
        if let InputValue::String(old) = &mut slot.current {
            let old_value = old.clone();
            *old = value.to_string();
            Some(old_value)
        } else {
            None
        }
    }

    pub fn get_numeric(&self, key: &str) -> Option<f32> {
        match self.find(key)?.current {
            InputValue::Numeric(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_boolean(&self, key: &str) -> Option<bool> {
        match self.find(key)?.current {
            InputValue::Boolean(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        match &self.find(key)?.current {
            InputValue::String(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    pub fn get_event(&self, key: &str) -> Option<&str> {
        match &self.find(key)?.current {
            InputValue::Event(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    /// Resets to default. Returns (old, new) for scalars.
    pub fn reset(&mut self, key: &str) -> Option<(InputValue, InputValue)> {
        let slot = self.find_mut(key)?;
        let old = slot.current.clone();
        let new = slot.default.clone();
        slot.current = new.clone();
        Some((old, new))
    }
}
