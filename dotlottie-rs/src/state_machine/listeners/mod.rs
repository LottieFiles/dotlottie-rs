use std::{fmt::Debug, fmt::Display};

use crate::parser::StringNumberBool;

pub trait ListenerTrait {
    fn set_type(&mut self, r#type: ListenerType);
    fn set_target(&mut self, target: &str);
    fn set_action(&mut self, action: &str);
    fn set_value(&mut self, value: StringNumberBool);
    fn set_context_key(&mut self, context_key: &str);

    fn get_type(&self) -> &ListenerType;
    fn get_target(&self) -> Option<String>;
    fn get_action(&self) -> Option<String>;
    fn get_value(&self) -> Option<&StringNumberBool>;
    fn get_context_key(&self) -> Option<String>;
}

#[derive(Debug, PartialEq)]
pub enum ListenerType {
    PointerUp,
    PointerDown,
    PointerEnter,
    PointerExit,
    PointerMove,
}

impl Display for ListenerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListenerType::PointerUp => write!(f, "PointerUp"),
            ListenerType::PointerDown => write!(f, "PointerDown"),
            ListenerType::PointerEnter => write!(f, "PointerEnter"),
            ListenerType::PointerExit => write!(f, "PointerExit"),
            ListenerType::PointerMove => write!(f, "PointerMove"),
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
        target: Option<String>,
        action: Option<String>,
        value: Option<StringNumberBool>,
        context_key: Option<String>,
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
            Self::PointerExit {
                r#type,
                target,
                action,
                value,
                context_key,
            } => f
                .debug_struct("PointerExit")
                .field("r#type", r#type)
                .field("target", target)
                .field("action", action)
                .field("value", value)
                .field("context_key", context_key)
                .finish(),
        }
    }
}

impl ListenerTrait for Listener {
    fn set_type(&mut self, r#listener_type: ListenerType) {
        match self {
            Listener::PointerUp {
                r#type,
                target: _,
                action: _,
                value: _,
                context_key: _,
            } => {
                *r#type = listener_type;
            }
            Listener::PointerDown {
                r#type,
                target: _,
                action: _,
                value: _,
                context_key: _,
            } => {
                *r#type = listener_type;
            }
            Listener::PointerEnter {
                r#type,
                target: _,
                action: _,
                value: _,
                context_key: _,
            } => {
                *r#type = listener_type;
            }
            Listener::PointerExit {
                r#type,
                target: _,
                action: _,
                value: _,
                context_key: _,
            } => {
                *r#type = listener_type;
            }
            Listener::PointerMove {
                r#type,
                target: _,
                action: _,
                value: _,
                context_key: _,
            } => {
                *r#type = listener_type;
            }
        }
    }

    fn set_target(&mut self, new_target: &str) {
        match self {
            Listener::PointerUp {
                r#type: _,
                target,
                action: _,
                value: _,
                context_key: _,
            } => {
                *target = Some(new_target.to_string());
            }
            Listener::PointerDown {
                r#type: _,
                target,
                action: _,
                value: _,
                context_key: _,
            } => {
                *target = Some(new_target.to_string());
            }
            Listener::PointerEnter {
                r#type: _,
                target,
                action: _,
                value: _,
                context_key: _,
            } => {
                *target = Some(new_target.to_string());
            }
            Listener::PointerExit {
                r#type: _,
                target,
                action: _,
                value: _,
                context_key: _,
            } => {
                *target = Some(new_target.to_string());
            }
            Listener::PointerMove {
                r#type: _,
                target,
                action: _,
                value: _,
                context_key: _,
            } => {
                *target = Some(new_target.to_string());
            }
        }
    }

    fn set_action(&mut self, new_action: &str) {
        match self {
            Listener::PointerUp {
                r#type: _,
                target: _,
                action,
                value: _,
                context_key: _,
            } => {
                *action = Some(new_action.to_string());
            }
            Listener::PointerDown {
                r#type: _,
                target: _,
                action,
                value: _,
                context_key: _,
            } => {
                *action = Some(new_action.to_string());
            }
            Listener::PointerEnter {
                r#type: _,
                target: _,
                action,
                value: _,
                context_key: _,
            } => {
                *action = Some(new_action.to_string());
            }
            Listener::PointerExit {
                r#type: _,
                target: _,
                action,
                value: _,
                context_key: _,
            } => {
                *action = Some(new_action.to_string());
            }
            Listener::PointerMove {
                r#type: _,
                target: _,
                action,
                value: _,
                context_key: _,
            } => {
                *action = Some(new_action.to_string());
            }
        }
    }

    fn set_value(&mut self, new_value: StringNumberBool) {
        match self {
            Listener::PointerUp {
                r#type: _,
                target: _,
                action: _,
                value,
                context_key: _,
            } => {
                *value = Some(new_value);
            }
            Listener::PointerDown {
                r#type: _,
                target: _,
                action: _,
                value,
                context_key: _,
            } => {
                *value = Some(new_value);
            }
            Listener::PointerEnter {
                r#type: _,
                target: _,
                action: _,
                value,
                context_key: _,
            } => {
                *value = Some(new_value);
            }
            Listener::PointerExit {
                r#type: _,
                target: _,
                action: _,
                value,
                context_key: _,
            } => {
                *value = Some(new_value);
            }
            Listener::PointerMove {
                r#type: _,
                target: _,
                action: _,
                value,
                context_key: _,
            } => {
                *value = Some(new_value);
            }
        }
    }

    fn set_context_key(&mut self, new_context_key: &str) {
        match self {
            Listener::PointerUp {
                r#type: _,
                target: _,
                action: _,
                value: _,
                context_key,
            } => {
                *context_key = Some(new_context_key.to_string());
            }
            Listener::PointerDown {
                r#type: _,
                target: _,
                action: _,
                value: _,
                context_key,
            } => {
                *context_key = Some(new_context_key.to_string());
            }
            Listener::PointerEnter {
                r#type: _,
                target: _,
                action: _,
                value: _,
                context_key,
            } => {
                *context_key = Some(new_context_key.to_string());
            }
            Listener::PointerExit {
                r#type: _,
                target: _,
                action: _,
                value: _,
                context_key,
            } => {
                *context_key = Some(new_context_key.to_string());
            }
            Listener::PointerMove {
                r#type: _,
                target: _,
                action: _,
                value: _,
                context_key,
            } => {
                *context_key = Some(new_context_key.to_string());
            }
        }
    }

    fn get_type(&self) -> &ListenerType {
        match self {
            Listener::PointerUp {
                r#type,
                target: _,
                action: _,
                value: _,
                context_key: _,
            } => r#type,
            Listener::PointerDown {
                r#type,
                target: _,
                action: _,
                value: _,
                context_key: _,
            } => r#type,
            Listener::PointerEnter {
                r#type,
                target: _,
                action: _,
                value: _,
                context_key: _,
            } => r#type,
            Listener::PointerExit {
                r#type,
                target: _,
                action: _,
                value: _,
                context_key: _,
            } => r#type,
            Listener::PointerMove {
                r#type,
                target: _,
                action: _,
                value: _,
                context_key: _,
            } => r#type,
        }
    }

    fn get_target(&self) -> Option<String> {
        match self {
            Listener::PointerUp {
                r#type: _,
                target,
                action: _,
                value: _,
                context_key: _,
            } => target.clone(),
            Listener::PointerDown {
                r#type: _,
                target,
                action: _,
                value: _,
                context_key: _,
            } => target.clone(),
            Listener::PointerEnter {
                r#type: _,
                target,
                action: _,
                value: _,
                context_key: _,
            } => target.clone(),
            Listener::PointerExit {
                r#type: _,
                target,
                action: _,
                value: _,
                context_key: _,
            } => target.clone(),
            Listener::PointerMove {
                r#type: _,
                target,
                action: _,
                value: _,
                context_key: _,
            } => target.clone(),
        }
    }

    fn get_action(&self) -> Option<String> {
        match self {
            Listener::PointerUp {
                r#type: _,
                target: _,
                action,
                value: _,
                context_key: _,
            } => action.clone(),
            Listener::PointerDown {
                r#type: _,
                target: _,
                action,
                value: _,
                context_key: _,
            } => action.clone(),
            Listener::PointerEnter {
                r#type: _,
                target: _,
                action,
                value: _,
                context_key: _,
            } => action.clone(),
            Listener::PointerExit {
                r#type: _,
                target: _,
                action,
                value: _,
                context_key: _,
            } => action.clone(),
            Listener::PointerMove {
                r#type: _,
                target: _,
                action,
                value: _,
                context_key: _,
            } => action.clone(),
        }
    }

    fn get_value(&self) -> Option<&StringNumberBool> {
        match self {
            Listener::PointerUp {
                r#type: _,
                target: _,
                action: _,
                value,
                context_key: _,
            } => value.as_ref(),
            Listener::PointerDown {
                r#type: _,
                target: _,
                action: _,
                value,
                context_key: _,
            } => value.as_ref(),
            Listener::PointerEnter {
                r#type: _,
                target: _,
                action: _,
                value,
                context_key: _,
            } => value.as_ref(),
            Listener::PointerExit {
                r#type: _,
                target: _,
                action: _,
                value,
                context_key: _,
            } => value.as_ref(),
            Listener::PointerMove {
                r#type: _,
                target: _,
                action: _,
                value,
                context_key: _,
            } => value.as_ref(),
        }
    }

    fn get_context_key(&self) -> Option<String> {
        match self {
            Listener::PointerUp {
                r#type: _,
                target: _,
                action: _,
                value: _,
                context_key,
            } => context_key.clone(),
            Listener::PointerDown {
                r#type: _,
                target: _,
                action: _,
                value: _,
                context_key,
            } => context_key.clone(),
            Listener::PointerEnter {
                r#type: _,
                target: _,
                action: _,
                value: _,
                context_key,
            } => context_key.clone(),
            Listener::PointerExit {
                r#type: _,
                target: _,
                action: _,
                value: _,
                context_key,
            } => context_key.clone(),
            Listener::PointerMove {
                r#type: _,
                target: _,
                action: _,
                value: _,
                context_key,
            } => context_key.clone(),
        }
    }
}
