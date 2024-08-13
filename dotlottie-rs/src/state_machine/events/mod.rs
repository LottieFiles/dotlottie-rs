pub trait PointerEvent {
    fn target(&self) -> Option<String>;
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

#[derive(Debug, Clone)]
pub enum Event {
    Bool {
        value: bool,
    },
    String {
        value: String,
    },
    Numeric {
        value: f32,
    },
    OnPointerDown {
        target: Option<String>,
        x: f32,
        y: f32,
    },
    OnPointerUp {
        target: Option<String>,
        x: f32,
        y: f32,
    },
    OnPointerMove {
        target: Option<String>,
        x: f32,
        y: f32,
    },
    OnPointerEnter {
        target: Option<String>,
        x: f32,
        y: f32,
    },
    OnPointerExit,
    OnComplete,
    SetNumericContext {
        key: String,
        value: f32,
    },
}

impl Event {
    pub fn as_str(&self) -> String {
        match self {
            Event::Bool { value } => value.to_string(),
            Event::String { value } => value.clone(),
            Event::Numeric { value } => value.to_string(),
            Event::OnPointerDown { target, x, y } => format!("{}, {}", x, y),
            Event::OnPointerUp { target, x, y } => format!("{}, {}", x, y),
            Event::OnPointerMove { target, x, y } => format!("{}, {}", x, y),
            Event::OnPointerEnter { target, x, y } => format!("{}, {}", x, y),
            Event::OnPointerExit => "OnPointerExitEvent".to_string(),
            Event::OnComplete => "OnCompleteEvent".to_string(),
            Event::SetNumericContext { key, value } => format!("{}, {}", key, value),
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

    fn target(&self) -> Option<String> {
        match self {
            Event::OnPointerDown { target, .. }
            | Event::OnPointerUp { target, .. }
            | Event::OnPointerMove { target, .. }
            | Event::OnPointerEnter { target, .. } => target.clone(),
            _ => None,
        }
    }
}
