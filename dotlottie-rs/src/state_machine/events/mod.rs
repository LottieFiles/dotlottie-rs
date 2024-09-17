pub trait PointerEvent {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

#[derive(Debug, Clone)]
pub enum Event {
    Bool { value: bool },
    String { value: String },
    Numeric { value: f32 },
    OnPointerDown { x: f32, y: f32 },
    OnPointerUp { x: f32, y: f32 },
    OnPointerMove { x: f32, y: f32 },
    OnPointerEnter { x: f32, y: f32 },
    OnPointerExit { x: f32, y: f32 },
    OnComplete,
    SetNumericContext { key: String, value: f32 },
}

#[derive(Debug, Clone)]
pub enum InternalEvent {
    Bool { value: bool },
    String { value: String },
    Numeric { value: f32 },
    OnPointerDown { target: Option<String> },
    OnPointerUp { target: Option<String> },
    OnPointerMove { target: Option<String> },
    OnPointerEnter { target: Option<String> },
    OnPointerExit { target: Option<String> },
    OnComplete,
    SetNumericContext { key: String, value: f32 },
}

// Display for InternalEvent
impl std::fmt::Display for InternalEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InternalEvent::Bool { value } => write!(f, "{}", value),
            InternalEvent::String { value } => write!(f, "{}", value),
            InternalEvent::Numeric { value } => write!(f, "{}", value),
            InternalEvent::OnPointerDown { target } => {
                if let Some(target) = target {
                    write!(f, "{}", target)
                } else {
                    write!(f, "OnPointerDownEvent")
                }
            }
            InternalEvent::OnPointerUp { target } => {
                if let Some(target) = target {
                    write!(f, "{}", target)
                } else {
                    write!(f, "OnPointerUpEvent")
                }
            }
            InternalEvent::OnPointerMove { target } => {
                if let Some(target) = target {
                    write!(f, "{}", target)
                } else {
                    write!(f, "OnPointerMoveEvent")
                }
            }
            InternalEvent::OnPointerEnter { target } => {
                if let Some(target) = target {
                    write!(f, "{}", target)
                } else {
                    write!(f, "OnPointerEnterEvent")
                }
            }
            InternalEvent::OnPointerExit { target } => {
                if let Some(target) = target {
                    write!(f, "{}", target)
                } else {
                    write!(f, "OnPointerExitEvent")
                }
            }
            InternalEvent::OnComplete => write!(f, "OnCompleteEvent"),
            InternalEvent::SetNumericContext { key, value } => write!(f, "{}, {}", key, value),
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
