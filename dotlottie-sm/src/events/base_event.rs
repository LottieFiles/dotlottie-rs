use std::any::Any;

use crate::events::event::Event;

// Trait for String, Bool, Int events
pub trait BaseEvent {
    type EventType;

    fn value(&self) -> &Self::EventType;
}

pub struct BoolEvent {
    pub value: bool,
}

pub struct StringEvent {
    pub value: String,
}

pub struct NumericEvent {
    pub value: f32,
}

impl BoolEvent {
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

impl Event for BoolEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BaseEvent for BoolEvent {
    type EventType = bool;

    fn value(&self) -> &Self::EventType {
        &self.value
    }
}

impl StringEvent {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl Event for StringEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BaseEvent for StringEvent {
    type EventType = String;

    fn value(&self) -> &Self::EventType {
        &self.value
    }
}

impl NumericEvent {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

impl Event for NumericEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BaseEvent for NumericEvent {
    type EventType = f32;

    fn value(&self) -> &Self::EventType {
        &self.value
    }
}
