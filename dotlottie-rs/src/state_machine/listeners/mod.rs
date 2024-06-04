use std::fmt::{Debug, Pointer};

use crate::parser::StringNumberBool;

pub trait ListenerTrait {
    // fn set_type(&mut self, r#type: ListenerType);
    // fn set_target(&mut self, target: &str);
    // fn set_action(&mut self, action: &str);
    // fn set_value(&mut self, value: ListenerValueType);
    // fn set_context_key(&mut self, context_key: &str);

    fn get_type(&self) -> &ListenerType;
    // fn get_target(&self) -> String;
    // fn get_action(&self) -> String;
    // fn get_value(&self) -> &ListenerValueType;
    // fn get_context_key(&self) -> String;
}

// pub enum ListenerValueType {
//     Bool { value: bool },
//     String { value: String },
//     Numeric { value: f32 },
// }

#[derive(Debug)]
pub enum ListenerType {
    PointerUp,
    PointerDown,
    PointerEnter,
    PointerExit,
    PointerMove,
}

impl ListenerType {
    pub fn to_string(&self) -> String {
        match self {
            ListenerType::PointerUp => "PointerUp".to_string(),
            ListenerType::PointerDown => "PointerDown".to_string(),
            ListenerType::PointerEnter => "PointerEnter".to_string(),
            ListenerType::PointerExit => "PointerExit".to_string(),
            ListenerType::PointerMove => "PointerMove".to_string(),
        }
    }
}

pub enum ListenerAction {
    Increment,
    Decrement,
    Set,
    None,
}

pub enum Listener {
    PointerUp {
        r#type: ListenerType,
        target: Option<String>,
        action: Option<String>,
        value: Option<StringNumberBool>,
        context_key: Option<String>,
    },
    PointerDown {
        r#type: ListenerType,
        target: Option<String>,
        action: Option<String>,
        value: Option<StringNumberBool>,
        context_key: Option<String>,
    },
    PointerEnter {
        r#type: ListenerType,
        target: Option<String>,
        action: Option<String>,
        value: Option<StringNumberBool>,
        context_key: Option<String>,
    },
    PointerMove {
        r#type: ListenerType,
        target: Option<String>,
        action: Option<String>,
        value: Option<StringNumberBool>,
        context_key: Option<String>,
    },
    PointerExit {
        r#type: ListenerType,
    },
}

impl Debug for Listener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PointerUp {
                r#type,
                target,
                action,
                value,
                context_key,
            } => f
                .debug_struct("PointerUp")
                .field("r#type", r#type)
                .field("target", target)
                .field("action", action)
                .field("value", value)
                .field("context_key", context_key)
                .finish(),
            Self::PointerDown {
                r#type,
                target,
                action,
                value,
                context_key,
            } => f
                .debug_struct("PointerDown")
                .field("r#type", r#type)
                .field("target", target)
                .field("action", action)
                .field("value", value)
                .field("context_key", context_key)
                .finish(),
            Self::PointerEnter {
                r#type,
                target,
                action,
                value,
                context_key,
            } => f
                .debug_struct("PointerEnter")
                .field("r#type", r#type)
                .field("target", target)
                .field("action", action)
                .field("value", value)
                .field("context_key", context_key)
                .finish(),
            Self::PointerMove {
                r#type,
                target,
                action,
                value,
                context_key,
            } => f
                .debug_struct("PointerMove")
                .field("r#type", r#type)
                .field("target", target)
                .field("action", action)
                .field("value", value)
                .field("context_key", context_key)
                .finish(),
            Self::PointerExit { r#type } => f
                .debug_struct("PointerExit")
                .field("r#type", r#type)
                .finish(),
        }
    }
}

impl ListenerTrait for Listener {
    // fn set_type(&mut self, r#listener_type: ListenerType) {
    //     match self {
    //         Listener::PointerUp {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => {
    //             *r#type = listener_type;
    //         }
    //         Listener::PointerDown {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerEnter {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerExit {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerMove {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //     }
    // }

    // fn set_target(&mut self, target: &str) {
    //     match self {
    //         Listener::PointerUp {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => {
    //             *target = target.to_string();
    //         }
    //         Listener::PointerDown {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerEnter {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerExit {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerMove {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //     }
    // }

    // fn set_action(&mut self, action: &str) {
    //     match self {
    //         Listener::PointerUp {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => {
    //             *action = action.to_string();
    //         }
    //         Listener::PointerDown {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerEnter {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerExit {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerMove {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //     }
    // }

    // fn set_value(&mut self, value: ListenerValueType) {
    //     match self {
    //         Listener::PointerUp {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => {
    //             *value = *value;
    //         }
    //         Listener::PointerDown {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerEnter {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerExit {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerMove {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //     }
    // }

    // fn set_context_key(&mut self, context_key: &str) {
    //     match self {
    //         Listener::PointerUp {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => {
    //             *context_key = context_key.to_string();
    //         }
    //         Listener::PointerDown {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerEnter {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerExit {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerMove {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //     }
    // }

    fn get_type(&self) -> &ListenerType {
        match self {
            Listener::PointerUp {
                r#type,
                target,
                action,
                value,
                context_key,
            } => r#type,
            Listener::PointerDown {
                r#type,
                target,
                action,
                value,
                context_key,
            } => r#type,
            Listener::PointerEnter {
                r#type,
                target,
                action,
                value,
                context_key,
            } => r#type,
            Listener::PointerExit { r#type } => r#type,
            Listener::PointerMove {
                r#type,
                target,
                action,
                value,
                context_key,
            } => r#type,
        }
    }

    // fn get_target(&self) -> String {
    //     match self {
    //         Listener::PointerUp {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => target.clone(),
    //         Listener::PointerDown {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerEnter {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerExit {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerMove {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //     }
    // }

    // fn get_action(&self) -> String {
    //     match self {
    //         Listener::PointerUp {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => action.clone(),
    //         Listener::PointerDown {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerEnter {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerExit {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerMove {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //     }
    // }

    // fn get_value(&self) -> &ListenerValueType {
    //     match self {
    //         Listener::PointerUp {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => value,
    //         Listener::PointerDown {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerEnter {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerExit {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerMove {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //     }
    // }

    // fn get_context_key(&self) -> String {
    //     match self {
    //         Listener::PointerUp {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => context_key.clone(),
    //         Listener::PointerDown {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerEnter {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerExit {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //         Listener::PointerMove {
    //             r#type,
    //             target,
    //             action,
    //             value,
    //             context_key,
    //         } => todo!(),
    //     }
    // }
}
