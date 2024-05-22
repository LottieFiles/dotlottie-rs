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
    pub fn as_str(&self) -> String {
        match self {
            Event::Bool(value) => value.to_string(),
            Event::String(value) => value.clone(),
            Event::Numeric(value) => value.to_string(),
            Event::OnPointerDown(x, y) => x.to_string() + ", " + &y.to_string(),
            Event::OnPointerUp(x, y) => x.to_string() + ", " + &y.to_string(),
            Event::OnPointerMove(x, y) => x.to_string() + ", " + &y.to_string(),
            Event::OnPointerEnter(x, y) => x.to_string() + ", " + &y.to_string(),
            Event::OnPointerExit => "OnPointerExitEvent".to_string(),
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
