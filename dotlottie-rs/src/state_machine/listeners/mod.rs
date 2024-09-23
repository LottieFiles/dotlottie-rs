use std::fmt::Display;

use super::actions::Action;

pub trait ListenerTrait {
    fn get_layer_name(&self) -> Option<String>;
    fn get_action(&self) -> Option<String>;
}

pub enum ListenerAction {
    Increment,
    Decrement,
    Set,
    None,
}

#[derive(Debug)]
pub enum Listener {
    PointerUp {
        layer_name: Option<String>,
        actions: Vec<Action>,
    },
    PointerDown {
        layer_name: Option<String>,
        actions: Vec<Action>,
    },
    PointerEnter {
        layer_name: Option<String>,
        actions: Vec<Action>,
    },
    PointerMove {
        layer_name: Option<String>,
        actions: Vec<Action>,
    },
    PointerExit {
        layer_name: Option<String>,
        actions: Vec<Action>,
    },
    OnComplete {
        state_name: Option<String>,
        actions: Vec<Action>,
    },
}

impl Display for Listener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PointerUp {
                layer_name,
                actions,
            } => f
                .debug_struct("PointerUp")
                .field("layer_name", layer_name)
                .field("action", actions)
                .finish(),
            Self::PointerDown {
                layer_name,
                actions,
            } => f
                .debug_struct("PointerUp")
                .field("layer_name", layer_name)
                .field("action", actions)
                .finish(),
            Self::PointerEnter {
                layer_name,
                actions,
            } => f
                .debug_struct("PointerUp")
                .field("layer_name", layer_name)
                .field("action", actions)
                .finish(),
            Self::PointerMove {
                layer_name,
                actions,
            } => f
                .debug_struct("PointerUp")
                .field("layer_name", layer_name)
                .field("action", actions)
                .finish(),
            Self::PointerExit {
                layer_name,
                actions,
            } => f
                .debug_struct("PointerUp")
                .field("layer_name", layer_name)
                .field("action", actions)
                .finish(),
            Self::OnComplete {
                state_name,
                actions,
            } => f
                .debug_struct("PointerUp")
                .field("state_name", state_name)
                .field("action", actions)
                .finish(),
        }
    }
}

impl ListenerTrait for Listener {
    fn get_layer_name(&self) -> Option<String> {
        todo!()
    }

    fn get_action(&self) -> Option<String> {
        todo!()
    }
}
