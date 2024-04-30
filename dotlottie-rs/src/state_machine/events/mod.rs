pub trait PointerEvent {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

#[derive(Debug)]
pub enum Event {
    Bool(bool),
    String(String),
    Numeric(f32),
    OnPointerDown(f32, f32),
    OnPointerUp(f32, f32),
    OnPointerMove(f32, f32),
    OnPointerEnter(f32, f32),
    OnPointerExit,
}

impl Event {
    pub fn as_str(&self) -> &str {
        match self {
            Event::Bool(_) => "BoolEvent",
            Event::String(_) => "StringEvent",
            Event::Numeric(_) => "NumericEvent: {value}",
            Event::OnPointerDown(_, _) => "OnPointerDownEvent",
            Event::OnPointerUp(_, _) => "OnPointerUpEvent",
            Event::OnPointerMove(_, _) => "OnPointerMoveEvent",
            Event::OnPointerEnter(_, _) => "OnPointerEnterEvent",
            Event::OnPointerExit => "OnPointerExitEvent",
        }
    }
}

impl PointerEvent for Event {
    fn x(&self) -> f32 {
        match self {
            Event::OnPointerDown(x, _)
            | Event::OnPointerUp(x, _)
            | Event::OnPointerMove(x, _)
            | Event::OnPointerEnter(x, _) => *x,
            _ => 0.0,
        }
    }

    fn y(&self) -> f32 {
        match self {
            Event::OnPointerDown(_, y)
            | Event::OnPointerUp(_, y)
            | Event::OnPointerMove(_, y)
            | Event::OnPointerEnter(_, y) => *y,
            _ => 0.0,
        }
    }
}
