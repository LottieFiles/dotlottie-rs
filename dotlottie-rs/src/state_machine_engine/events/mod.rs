pub trait PointerEvent {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

pub trait EventName {
    fn type_name(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum Event {
    PointerDown { x: f32, y: f32 },
    PointerUp { x: f32, y: f32 },
    PointerMove { x: f32, y: f32 },
    PointerEnter { x: f32, y: f32 },
    PointerExit { x: f32, y: f32 },
    Click { x: f32, y: f32 },
    OnComplete,
    OnLoopComplete,
}

impl PointerEvent for Event {
    fn x(&self) -> f32 {
        match self {
            Event::PointerDown { x, .. }
            | Event::PointerUp { x, .. }
            | Event::PointerMove { x, .. }
            | Event::Click { x, .. }
            | Event::PointerEnter { x, .. } => *x,
            _ => 0.0,
        }
    }

    fn y(&self) -> f32 {
        match self {
            Event::PointerDown { y, .. }
            | Event::PointerUp { y, .. }
            | Event::PointerMove { y, .. }
            | Event::Click { y, .. }
            | Event::PointerEnter { y, .. } => *y,
            _ => 0.0,
        }
    }
}

impl EventName for Event {
    fn type_name(&self) -> String {
        match self {
            Event::PointerDown { .. } => "PointerDown".to_string(),
            Event::PointerUp { .. } => "PointerUp".to_string(),
            Event::PointerMove { .. } => "PointerMove".to_string(),
            Event::PointerEnter { .. } => "PointerEnter".to_string(),
            Event::PointerExit { .. } => "PointerExit".to_string(),
            Event::Click { .. } => "Click".to_string(),
            Event::OnComplete => "OnComplete".to_string(),
            Event::OnLoopComplete => "OnLoopComplete".to_string(),
        }
    }
}

#[macro_export]
macro_rules! event_type_name {
    (PointerDown) => {
        "PointerDown"
    };
    (PointerUp) => {
        "PointerUp"
    };
    (PointerMove) => {
        "PointerMove"
    };
    (PointerEnter) => {
        "PointerEnter"
    };
    (PointerExit) => {
        "PointerExit"
    };
    (Click) => {
        "Click"
    };
    (OnComplete) => {
        "OnComplete"
    };
    (OnLoopComplete) => {
        "OnLoopComplete"
    };
}
