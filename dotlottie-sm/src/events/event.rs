pub trait PointerEvent {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

pub trait BoolValue {
    fn value(&self) -> bool;
}

pub trait StringValue {
    fn value(&self) -> &str;
}

pub trait NumericValue {
    fn value(&self) -> f32;
}

pub enum Event {
    BoolEvent { value: bool },
    StringEvent { value: String },
    NumericEvent { value: f32 },
    OnPointerDownEvent { x: f32, y: f32 },
    OnPointerUpEvent { x: f32, y: f32 },
    OnPointerMoveEvent { x: f32, y: f32 },
    OnPointerEnterEvent { x: f32, y: f32 },
    OnPointerExitEvent,
}

impl Event {
    pub fn as_str(&self) -> &str {
        match self {
            Event::BoolEvent { value: _ } => "BoolEvent",
            Event::StringEvent { value: _ } => "StringEvent",
            Event::NumericEvent { value: _ } => "NumericEvent: {value}",
            Event::OnPointerDownEvent { x: _, y: _ } => "OnPointerDownEvent",
            Event::OnPointerUpEvent { x: _, y: _ } => "OnPointerUpEvent",
            Event::OnPointerMoveEvent { x: _, y: _ } => "OnPointerMoveEvent",
            Event::OnPointerEnterEvent { x: _, y: _ } => "OnPointerEnterEvent",
            Event::OnPointerExitEvent => "OnPointerExitEvent",
        }
    }
}

// Implement each trait for the repective enum variant
impl BoolValue for Event {
    fn value(&self) -> bool {
        match self {
            Event::BoolEvent { value } => *value,
            _ => false,
        }
    }
}

impl StringValue for Event {
    fn value(&self) -> &str {
        match self {
            Event::StringEvent { value } => value,
            _ => "",
        }
    }
}

impl NumericValue for Event {
    fn value(&self) -> f32 {
        match self {
            Event::NumericEvent { value } => *value,
            _ => 0.0,
        }
    }
}

impl PointerEvent for Event {
    fn x(&self) -> f32 {
        match self {
            Event::OnPointerDownEvent { x, .. }
            | Event::OnPointerUpEvent { x, .. }
            | Event::OnPointerMoveEvent { x, .. }
            | Event::OnPointerEnterEvent { x, .. } => *x,
            _ => 0.0,
        }
    }

    fn y(&self) -> f32 {
        match self {
            Event::OnPointerDownEvent { y, .. }
            | Event::OnPointerUpEvent { y, .. }
            | Event::OnPointerMoveEvent { y, .. }
            | Event::OnPointerEnterEvent { y, .. } => *y,
            _ => 0.0,
        }
    }
}
