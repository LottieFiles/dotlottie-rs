use std::fmt::Display;

use serde::Deserialize;

use super::actions::Action;

pub trait ListenerTrait {
    fn get_layer_name(&self) -> Option<String>;
    fn get_state_name(&self) -> Option<String>;
    fn get_actions(&self) -> &Vec<Action>;
    fn type_name(&self) -> String;
}

pub enum ListenerAction {
    Increment,
    Decrement,
    Set,
    None,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "type")]
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
        state_name: String,
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
        match self {
            Listener::PointerUp { layer_name, .. } => layer_name.clone(),
            Listener::PointerDown { layer_name, .. } => layer_name.clone(),
            Listener::PointerEnter { layer_name, .. } => layer_name.clone(),
            Listener::PointerMove { layer_name, .. } => layer_name.clone(),
            Listener::PointerExit { layer_name, .. } => layer_name.clone(),
            Listener::OnComplete { .. } => None,
        }
    }

    fn get_actions(&self) -> &Vec<Action> {
        match self {
            Listener::PointerUp { actions, .. } => actions,
            Listener::PointerDown { actions, .. } => actions,
            Listener::PointerEnter { actions, .. } => actions,
            Listener::PointerMove { actions, .. } => actions,
            Listener::PointerExit { actions, .. } => actions,
            Listener::OnComplete { actions, .. } => actions,
        }
    }

    fn get_state_name(&self) -> Option<String> {
        match self {
            Listener::PointerUp { .. } => None,
            Listener::PointerDown { .. } => None,
            Listener::PointerEnter { .. } => None,
            Listener::PointerMove { .. } => None,
            Listener::PointerExit { .. } => None,
            Listener::OnComplete { state_name, .. } => Some(state_name.clone()),
        }
    }

    fn type_name(&self) -> String {
        match self {
            Listener::PointerUp { .. } => "PointerUp".to_string(),
            Listener::PointerDown { .. } => "PointerDown".to_string(),
            Listener::PointerEnter { .. } => "PointerEnter".to_string(),
            Listener::PointerMove { .. } => "PointerMove".to_string(),
            Listener::PointerExit { .. } => "PointerExit".to_string(),
            Listener::OnComplete { .. } => "OnComplete".to_string(),
        }
    }
}