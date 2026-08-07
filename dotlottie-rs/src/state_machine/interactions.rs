use crate::json::{array_of, opt, Value};
use crate::state_machine::definition::dot_string;
use crate::string::{DotString, DotStringInterner};

use super::actions::Action;
use super::events::EventKind;

#[derive(Debug)]
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

pub(crate) fn interaction_from_json(v: &Value) -> Option<Interaction> {
    let actions = || -> Option<Vec<Action>> {
        array_of(
            v.get("actions")?,
            crate::state_machine::actions::action_from_json,
        )
    };
    let layer_name = || opt(v.get("layerName"), dot_string);
    Some(match v.str_field("type")? {
        "PointerUp" => Interaction::PointerUp {
            layer_name: layer_name()?,
            actions: actions()?,
        },
        "PointerDown" => Interaction::PointerDown {
            layer_name: layer_name()?,
            actions: actions()?,
        },
        "PointerEnter" => Interaction::PointerEnter {
            layer_name: layer_name()?,
            actions: actions()?,
        },
        "PointerMove" => Interaction::PointerMove {
            actions: actions()?,
        },
        "PointerExit" => Interaction::PointerExit {
            layer_name: layer_name()?,
            actions: actions()?,
        },
        "Click" => Interaction::Click {
            layer_name: layer_name()?,
            actions: actions()?,
        },
        "OnComplete" => Interaction::OnComplete {
            state_name: dot_string(v.get("stateName")?)?,
            actions: actions()?,
        },
        "OnLoopComplete" => Interaction::OnLoopComplete {
            state_name: dot_string(v.get("stateName")?)?,
            actions: actions()?,
        },
        _ => return None,
    })
}

impl Interaction {
    pub fn get_layer_name(&self) -> Option<&DotString> {
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

    pub fn actions(&self) -> &[Action] {
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

    pub fn kind(&self) -> EventKind {
        match self {
            Interaction::PointerUp { .. } => EventKind::PointerUp,
            Interaction::PointerDown { .. } => EventKind::PointerDown,
            Interaction::PointerEnter { .. } => EventKind::PointerEnter,
            Interaction::PointerMove { .. } => EventKind::PointerMove,
            Interaction::PointerExit { .. } => EventKind::PointerExit,
            Interaction::OnComplete { .. } => EventKind::OnComplete,
            Interaction::OnLoopComplete { .. } => EventKind::OnLoopComplete,
            Interaction::Click { .. } => EventKind::Click,
        }
    }
}

impl Interaction {
    /// Canonicalize identifier fields through a shared interner so runtime
    /// comparisons hit the `Arc::ptr_eq` fast path.
    pub fn intern_identifiers(&mut self, interner: &mut DotStringInterner) {
        match self {
            Interaction::PointerUp {
                layer_name,
                actions,
            }
            | Interaction::PointerDown {
                layer_name,
                actions,
            }
            | Interaction::PointerEnter {
                layer_name,
                actions,
            }
            | Interaction::PointerExit {
                layer_name,
                actions,
            }
            | Interaction::Click {
                layer_name,
                actions,
            } => {
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
            Interaction::OnComplete {
                state_name,
                actions,
            }
            | Interaction::OnLoopComplete {
                state_name,
                actions,
            } => {
                *state_name = interner.intern(state_name.as_str());
                for a in actions {
                    a.intern_identifiers(interner);
                }
            }
        }
    }
}
