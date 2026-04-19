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
        state_name: String,
        actions: Vec<Action>,
    },
    OnLoopComplete {
        state_name: String,
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
            Interaction::OnComplete { state_name, .. } => Some(state_name.clone()),
            Interaction::OnLoopComplete { state_name, .. } => Some(state_name.clone()),
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
    /// Replace this interaction's `layer_name` (if any) with an interned copy
    /// from the shared interner. Repeated interns of the same name return
    /// clones of the same `DotString`, so runtime comparisons against other
    /// interned layer names hit the `Arc::ptr_eq` fast path.
    pub fn intern_layer_name(&mut self, interner: &mut DotStringInterner) {
        let slot = match self {
            Interaction::PointerUp { layer_name, .. }
            | Interaction::PointerDown { layer_name, .. }
            | Interaction::PointerEnter { layer_name, .. }
            | Interaction::PointerExit { layer_name, .. }
            | Interaction::Click { layer_name, .. } => layer_name,
            Interaction::PointerMove { .. }
            | Interaction::OnComplete { .. }
            | Interaction::OnLoopComplete { .. } => return,
        };

        if let Some(name) = slot {
            *name = interner.intern(name.as_str());
        }
    }
}
