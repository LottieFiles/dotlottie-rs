use crate::json::{opt, Value};
use crate::state_machine::definition::dot_string;
use crate::string::{DotString, DotStringInterner};

use super::actions::Action;

pub trait InteractionTrait {
    fn get_layer_name(&self) -> Option<&DotString>;
    fn get_state_name(&self) -> Option<String>;
    fn get_actions(&self) -> &Vec<Action>;
    fn type_name(&self) -> &'static str;
}

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
    /// Runs actions whenever the named input's value changes (host-set or
    /// action-set). The declarative bridge for host-observed signals: scroll
    /// progress, sliders, sensors — anything written into an input.
    OnInputChange {
        input_name: DotString,
        actions: Vec<Action>,
    },
    /// Runs actions when a named motion finishes its last step (never fires for
    /// `repeat: "infinite"`).
    OnMotionComplete {
        motion_name: DotString,
        actions: Vec<Action>,
    },
}

pub(crate) fn interaction_from_json(v: &Value) -> Option<Interaction> {
    // Unknown action types are skipped, not fatal (forward compatibility).
    let actions = || -> Option<Vec<Action>> {
        Some(
            v.get("actions")?
                .as_array()?
                .iter()
                .filter_map(crate::state_machine::actions::action_from_json)
                .collect(),
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
        "OnInputChange" => Interaction::OnInputChange {
            input_name: dot_string(v.get("inputName")?)?,
            actions: actions()?,
        },
        "OnMotionComplete" => Interaction::OnMotionComplete {
            motion_name: dot_string(v.get("motionName")?)?,
            actions: actions()?,
        },
        _ => return None,
    })
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
            Interaction::OnInputChange { .. } => None,
            Interaction::OnMotionComplete { .. } => None,
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
            Interaction::OnInputChange { actions, .. } => actions,
            Interaction::OnMotionComplete { actions, .. } => actions,
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
            Interaction::OnInputChange { .. } => None,
            Interaction::OnMotionComplete { .. } => None,
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
            Interaction::OnInputChange { .. } => "OnInputChange",
            Interaction::OnMotionComplete { .. } => "OnMotionComplete",
            Interaction::Click { .. } => "Click",
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
            Interaction::OnInputChange {
                input_name: name,
                actions,
            }
            | Interaction::OnMotionComplete {
                motion_name: name,
                actions,
            } => {
                *name = interner.intern(name.as_str());
                for a in actions {
                    a.intern_identifiers(interner);
                }
            }
        }
    }
}
