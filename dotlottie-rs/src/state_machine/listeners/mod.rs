use std::fmt::{Debug, Display};

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

impl ListenerTrait for Listener {
    fn set_type(&mut self, r#listener_type: ListenerType) {
        match self {
            Listener::PointerUp { r#type, .. } => {
                *r#type = listener_type;
            }
            Listener::PointerDown { r#type, .. } => {
                *r#type = listener_type;
            }
            Listener::PointerEnter { r#type, .. } => {
                *r#type = listener_type;
            }
            Listener::PointerExit { r#type, .. } => {
                *r#type = listener_type;
            }
            Listener::PointerMove { r#type, .. } => {
                *r#type = listener_type;
            }
        }
    }

    fn set_target(&mut self, new_target: &str) {
        match self {
            Listener::PointerUp { target, .. } => {
                *target = Some(new_target.to_string());
            }
            Listener::PointerDown { target, .. } => {
                *target = Some(new_target.to_string());
            }
            Listener::PointerEnter { target, .. } => {
                *target = Some(new_target.to_string());
            }
            Listener::PointerExit { target, .. } => {
                *target = Some(new_target.to_string());
            }
            Listener::PointerMove { target, .. } => {
                *target = Some(new_target.to_string());
            }
        }
    }

    fn set_action(&mut self, new_action: &str) {
        match self {
            Listener::PointerUp { action, .. } => {
                *action = Some(new_action.to_string());
            }
            Listener::PointerDown { action, .. } => {
                *action = Some(new_action.to_string());
            }
            Listener::PointerEnter { action, .. } => {
                *action = Some(new_action.to_string());
            }
            Listener::PointerExit { action, .. } => {
                *action = Some(new_action.to_string());
            }
            Listener::PointerMove { action, .. } => {
                *action = Some(new_action.to_string());
            }
        }
    }

    fn set_value(&mut self, new_value: StringNumberBool) {
        match self {
            Listener::PointerUp { value, .. } => {
                *value = Some(new_value);
            }
            Listener::PointerDown { value, .. } => {
                *value = Some(new_value);
            }
            Listener::PointerEnter { value, .. } => {
                *value = Some(new_value);
            }
            Listener::PointerExit { value, .. } => {
                *value = Some(new_value);
            }
            Listener::PointerMove { value, .. } => {
                *value = Some(new_value);
            }
        }
    }

    fn set_context_key(&mut self, new_context_key: &str) {
        match self {
            Listener::PointerUp { context_key, .. } => {
                *context_key = Some(new_context_key.to_string());
            }
            Listener::PointerDown { context_key, .. } => {
                *context_key = Some(new_context_key.to_string());
            }
            Listener::PointerEnter { context_key, .. } => {
                *context_key = Some(new_context_key.to_string());
            }
            Listener::PointerExit { context_key, .. } => {
                *context_key = Some(new_context_key.to_string());
            }
            Listener::PointerMove { context_key, .. } => {
                *context_key = Some(new_context_key.to_string());
            }
        }
    }

    fn get_type(&self) -> &ListenerType {
        match self {
            Listener::PointerUp { r#type, .. } => r#type,
            Listener::PointerDown { r#type, .. } => r#type,
            Listener::PointerEnter { r#type, .. } => r#type,
            Listener::PointerExit { r#type, .. } => r#type,
            Listener::PointerMove { r#type, .. } => r#type,
        }
    }

    fn get_target(&self) -> Option<String> {
        match self {
            Listener::PointerUp { target, .. } => target.clone(),
            Listener::PointerDown { target, .. } => target.clone(),
            Listener::PointerEnter { target, .. } => target.clone(),
            Listener::PointerExit { target, .. } => target.clone(),
            Listener::PointerMove { target, .. } => target.clone(),
        }
    }

    fn get_action(&self) -> Option<String> {
        match self {
            Listener::PointerUp { action, .. } => action.clone(),
            Listener::PointerDown { action, .. } => action.clone(),
            Listener::PointerEnter { action, .. } => action.clone(),
            Listener::PointerExit { action, .. } => action.clone(),
            Listener::PointerMove { action, .. } => action.clone(),
        }
    }

    fn get_value(&self) -> Option<&StringNumberBool> {
        match self {
            Listener::PointerUp { value, .. } => value.as_ref(),
            Listener::PointerDown { value, .. } => value.as_ref(),
            Listener::PointerEnter { value, .. } => value.as_ref(),
            Listener::PointerExit { value, .. } => value.as_ref(),
            Listener::PointerMove { value, .. } => value.as_ref(),
        }
    }

    fn get_context_key(&self) -> Option<String> {
        match self {
            Listener::PointerUp { context_key, .. } => context_key.clone(),
            Listener::PointerDown { context_key, .. } => context_key.clone(),
            Listener::PointerEnter { context_key, .. } => context_key.clone(),
            Listener::PointerExit { context_key, .. } => context_key.clone(),
            Listener::PointerMove { context_key, .. } => context_key.clone(),
        }
    }
}
