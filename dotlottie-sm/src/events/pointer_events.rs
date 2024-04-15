use std::any::Any;

use crate::events::event::Event;

pub trait PointerEvent {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

pub struct OnPointerDownEvent {
    x: f32,
    y: f32,
}

pub struct OnPointerUpEvent {
    x: f32,
    y: f32,
}

pub struct OnPointerMoveEvent {
    x: f32,
    y: f32,
}

pub struct OnPointerEnterEvent {
    x: f32,
    y: f32,
}

pub struct OnPointerExitEvent {}

impl OnPointerDownEvent {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Event for OnPointerDownEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PointerEvent for OnPointerDownEvent {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

impl OnPointerUpEvent {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Event for OnPointerUpEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PointerEvent for OnPointerUpEvent {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

impl OnPointerEnterEvent {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Event for OnPointerEnterEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PointerEvent for OnPointerEnterEvent {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

impl OnPointerExitEvent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Event for OnPointerExitEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PointerEvent for OnPointerExitEvent {
    fn x(&self) -> f32 {
        0.0
    }

    fn y(&self) -> f32 {
        0.0
    }
}

impl OnPointerMoveEvent {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Event for OnPointerMoveEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PointerEvent for OnPointerMoveEvent {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}
// pub impl Event for OnPointerUpEvent {
//     pub fn new(x: f32, y: f32) -> Self {
//         Self { x, y }
//     }
// }

// pub impl Event for OnPointerMoveEvent {
//     pub fn new(x: f32, y: f32) -> Self {
//         Self { x, y }
//     }
// }

// pub impl Event for OnPointerEnterEvent {
//     pub fn new(x: f32, y: f32) -> Self {
//         Self { x, y }
//     }
// }

// pub impl Event for OnPointerExitEvent {}
