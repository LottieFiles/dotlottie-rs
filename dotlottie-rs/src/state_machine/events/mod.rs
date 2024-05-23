pub trait PointerEvent {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

#[derive(Debug)]
pub enum Event {
    Bool { value: bool },
    String { value: String },
    Numeric { value: f32 },
    OnPointerDown { x: f32, y: f32 },
    OnPointerUp { x: f32, y: f32 },
    OnPointerMove { x: f32, y: f32 },
    OnPointerEnter { x: f32, y: f32 },
    OnPointerExit,
}

impl Event {
    pub fn as_str(&self) -> String {
        match self {
            Event::Bool { value } => value.to_string(),
            Event::String { value } => value.clone(),
            Event::Numeric { value } => value.to_string(),
            Event::OnPointerDown { x, y } => format!("{}, {}", x, y),
            Event::OnPointerUp { x, y } => format!("{}, {}", x, y),
            Event::OnPointerMove { x, y } => format!("{}, {}", x, y),
            Event::OnPointerEnter { x, y } => format!("{}, {}", x, y),
            Event::OnPointerExit => "OnPointerExitEvent".to_string(),
        }
    }
}

impl PointerEvent for Event {
    fn x(&self) -> f32 {
        match self {
            Event::OnPointerDown { x, .. }
            | Event::OnPointerUp { x, .. }
            | Event::OnPointerMove { x, .. }
            | Event::OnPointerEnter { x, .. } => *x,
            _ => 0.0,
        }
    }

    fn y(&self) -> f32 {
        match self {
            Event::OnPointerDown { y, .. }
            | Event::OnPointerUp { y, .. }
            | Event::OnPointerMove { y, .. }
            | Event::OnPointerEnter { y, .. } => *y,
            _ => 0.0,
        }
    }
}
