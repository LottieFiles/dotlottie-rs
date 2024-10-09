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
    OnComplete,
}

// #[derive(Debug, Clone)]
// pub enum InternalEvent {
//     OnPointerDown { target: Option<String> },
//     OnPointerUp { target: Option<String> },
//     OnPointerMove { target: Option<String> },
//     OnPointerEnter { target: Option<String> },
//     OnPointerExit { target: Option<String> },
//     OnComplete,
// }

// Display for InternalEvent
// impl std::fmt::Display for InternalEvent {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match self {
//             InternalEvent::OnPointerDown { target } => {
//                 if let Some(target) = target {
//                     write!(f, "{}", target)
//                 } else {
//                     write!(f, "OnPointerDownEvent")
//                 }
//             }
//             InternalEvent::OnPointerUp { target } => {
//                 if let Some(target) = target {
//                     write!(f, "{}", target)
//                 } else {
//                     write!(f, "OnPointerUpEvent")
//                 }
//             }
//             InternalEvent::OnPointerMove { target } => {
//                 if let Some(target) = target {
//                     write!(f, "{}", target)
//                 } else {
//                     write!(f, "OnPointerMoveEvent")
//                 }
//             }
//             InternalEvent::OnPointerEnter { target } => {
//                 if let Some(target) = target {
//                     write!(f, "{}", target)
//                 } else {
//                     write!(f, "OnPointerEnterEvent")
//                 }
//             }
//             InternalEvent::OnPointerExit { target } => {
//                 if let Some(target) = target {
//                     write!(f, "{}", target)
//                 } else {
//                     write!(f, "OnPointerExitEvent")
//                 }
//             }
//             InternalEvent::OnComplete => write!(f, "OnCompleteEvent"),
//         }
//     }
// }

impl PointerEvent for Event {
    fn x(&self) -> f32 {
        match self {
            Event::PointerDown { x, .. }
            | Event::PointerUp { x, .. }
            | Event::PointerMove { x, .. }
            | Event::PointerEnter { x, .. } => *x,
            _ => 0.0,
        }
    }

    fn y(&self) -> f32 {
        match self {
            Event::PointerDown { y, .. }
            | Event::PointerUp { y, .. }
            | Event::PointerMove { y, .. }
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
            Event::OnComplete => "OnComplete".to_string(),
        }
    }
}
