use serde::Deserialize;

use crate::string::{DotString, DotStringInterner};

use super::actions::Action;

pub trait InteractionTrait {
    fn get_layer_name(&self) -> Option<&DotString>;
    fn get_state_name(&self) -> Option<String>;
    fn get_actions(&self) -> &Vec<Action>;
    fn type_name(&self) -> &'static str;
}

#[derive(Deserialize, Debug)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "type")]
pub enum Interaction {
    PointerUp {
        layer_name: Option<DotString>,
        actions: Vec<Action>,
    },
    PointerDown {
        layer_name: Option<DotString>,
        actions: Vec<Action>,
    },
    PointerEnter {
        layer_name: Option<DotString>,
        actions: Vec<Action>,
    },
    PointerMove {
        actions: Vec<Action>,
    },
    PointerExit {
        layer_name: Option<DotString>,
        actions: Vec<Action>,
    },
    Click {
        layer_name: Option<DotString>,
        actions: Vec<Action>,
    },
    OnComplete {
        state_name: DotString,
        actions: Vec<Action>,
    },
    OnLoopComplete {
        state_name: DotString,
        actions: Vec<Action>,
    },
}

impl InteractionTrait for Interaction {
    fn get_layer_name(&self) -> Option<&DotString> {
        match self {
            Interaction::PointerUp { layer_name, .. } => layer_name.as_ref(),
            Interaction::PointerDown { layer_name, .. } => layer_name.as_ref(),
            Interaction::PointerEnter { layer_name, .. } => layer_name.as_ref(),
            Interaction::PointerMove { .. } => None,
            Interaction::PointerExit { layer_name, .. } => layer_name.as_ref(),
            Interaction::OnComplete { .. } => None,
            Interaction::OnLoopComplete { .. } => None,
            Interaction::Click { layer_name, .. } => layer_name.as_ref(),
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
            Interaction::OnComplete { state_name, .. } => Some(state_name.as_str().to_owned()),
            Interaction::OnLoopComplete { state_name, .. } => Some(state_name.as_str().to_owned()),
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Interaction::PointerUp { .. } => "PointerUp",
            Interaction::PointerDown { .. } => "PointerDown",
            Interaction::PointerEnter { .. } => "PointerEnter",
            Interaction::PointerMove { .. } => "PointerMove",
            Interaction::PointerExit { .. } => "PointerExit",
            Interaction::OnComplete { .. } => "OnComplete",
            Interaction::OnLoopComplete { .. } => "OnLoopComplete",
            Interaction::Click { .. } => "Click",
        }
    }
}

impl Interaction {
    /// Canonicalize identifier fields through a shared interner so runtime
    /// comparisons hit the `Arc::ptr_eq` fast path.
    pub fn intern_identifiers(&mut self, interner: &mut DotStringInterner) {
        match self {
            Interaction::PointerUp { layer_name, actions }
            | Interaction::PointerDown { layer_name, actions }
            | Interaction::PointerEnter { layer_name, actions }
            | Interaction::PointerExit { layer_name, actions }
            | Interaction::Click { layer_name, actions } => {
                if let Some(name) = layer_name {
                    *name = interner.intern(name.as_str());
                }
                for a in actions {
                    a.intern_identifiers(interner);
                }
            }
            Interaction::PointerMove { actions } => {
                for a in actions {
                    a.intern_identifiers(interner);
                }
            }
            Interaction::OnComplete { state_name, actions }
            | Interaction::OnLoopComplete { state_name, actions } => {
                *state_name = interner.intern(state_name.as_str());
                for a in actions {
                    a.intern_identifiers(interner);
                }
            }
        }
    }
}
