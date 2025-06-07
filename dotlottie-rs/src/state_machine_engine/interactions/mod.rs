use serde::Deserialize;

use super::actions::Action;

pub trait InteractionTrait {
    fn get_layer_name(&self) -> Option<String>;
    fn get_state_name(&self) -> Option<String>;
    fn get_actions(&self) -> &Vec<Action>;
    fn type_name(&self) -> String;
}

pub enum InteractionAction {
    Increment,
    Decrement,
    Set,
    None,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "type")]
pub enum Interaction {
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
        actions: Vec<Action>,
    },
    PointerExit {
        layer_name: Option<String>,
        actions: Vec<Action>,
    },
    Click {
        layer_name: Option<String>,
        actions: Vec<Action>,
    },
    OnComplete {
        state_name: String,
        actions: Vec<Action>,
    },
    OnLoopComplete {
        state_name: String,
        actions: Vec<Action>,
    },
}

impl InteractionTrait for Interaction {
    fn get_layer_name(&self) -> Option<String> {
        match self {
            Interaction::PointerUp { layer_name, .. } => layer_name.clone(),
            Interaction::PointerDown { layer_name, .. } => layer_name.clone(),
            Interaction::PointerEnter { layer_name, .. } => layer_name.clone(),
            Interaction::PointerMove { .. } => None,
            Interaction::PointerExit { layer_name, .. } => layer_name.clone(),
            Interaction::OnComplete { .. } => None,
            Interaction::OnLoopComplete { .. } => None,
            Interaction::Click { layer_name, .. } => layer_name.clone(),
        }
    }

    fn get_actions(&self) -> &Vec<Action> {
        match self {
            Interaction::PointerUp { actions, .. } => actions,
            Interaction::PointerDown { actions, .. } => actions,
            Interaction::PointerEnter { actions, .. } => actions,
            Interaction::PointerMove { actions, .. } => actions,
            Interaction::PointerExit { actions, .. } => actions,
            Interaction::OnComplete { actions, .. } => actions,
            Interaction::OnLoopComplete { actions, .. } => actions,
            Interaction::Click { actions, .. } => actions,
        }
    }

    fn get_state_name(&self) -> Option<String> {
        match self {
            Interaction::PointerUp { .. } => None,
            Interaction::PointerDown { .. } => None,
            Interaction::PointerEnter { .. } => None,
            Interaction::PointerMove { .. } => None,
            Interaction::PointerExit { .. } => None,
            Interaction::Click { .. } => None,
            Interaction::OnComplete { state_name, .. } => Some(state_name.clone()),
            Interaction::OnLoopComplete { state_name, .. } => Some(state_name.clone()),
        }
    }

    fn type_name(&self) -> String {
        match self {
            Interaction::PointerUp { .. } => "PointerUp".to_string(),
            Interaction::PointerDown { .. } => "PointerDown".to_string(),
            Interaction::PointerEnter { .. } => "PointerEnter".to_string(),
            Interaction::PointerMove { .. } => "PointerMove".to_string(),
            Interaction::PointerExit { .. } => "PointerExit".to_string(),
            Interaction::OnComplete { .. } => "OnComplete".to_string(),
            Interaction::OnLoopComplete { .. } => "OnLoopComplete".to_string(),
            Interaction::Click { .. } => "Click".to_string(),
        }
    }
}
