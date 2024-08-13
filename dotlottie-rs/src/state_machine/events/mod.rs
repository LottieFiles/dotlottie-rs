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
            Event::OnPointerExit { x, y } => format!("{}, {}", x, y),
            Event::OnComplete => "OnCompleteEvent".to_string(),
            Event::SetNumericContext { key, value } => format!("{}, {}", key, value),
        }
    }
}

impl InternalEvent {
    pub fn as_str(&self) -> String {
        match self {
            InternalEvent::Bool { value } => value.to_string(),
            InternalEvent::String { value } => value.clone(),
            InternalEvent::Numeric { value } => value.to_string(),
            InternalEvent::OnPointerDown { target } => {
                if let Some(target) = target {
                    target.clone()
                } else {
                    "OnPointerDownEvent".to_string()
                }
            }
            InternalEvent::OnPointerUp { target } => {
                if let Some(target) = target {
                    target.clone()
                } else {
                    "OnPointerUpEvent".to_string()
                }
            }
            InternalEvent::OnPointerMove { target } => {
                if let Some(target) = target {
                    target.clone()
                } else {
                    "OnPointerMoveEvent".to_string()
                }
            }
            InternalEvent::OnPointerEnter { target } => {
                if let Some(target) = target {
                    target.clone()
                } else {
                    "OnPointerEnterEvent".to_string()
                }
            }
            InternalEvent::OnPointerExit { target } => {
                if let Some(target) = target {
                    target.clone()
                } else {
                    "OnPointerExitEvent".to_string()
                }
            }
            InternalEvent::OnComplete => "OnCompleteEvent".to_string(),
            InternalEvent::SetNumericContext { key, value } => format!("{}, {}", key, value),
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
