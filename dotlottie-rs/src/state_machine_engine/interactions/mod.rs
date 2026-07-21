use serde::Deserialize;

use crate::string::{DotString, DotStringInterner};

use super::actions::Action;

pub trait InteractionTrait {
    fn get_layer_name(&self) -> Option<&DotString>;
    fn get_state_name(&self) -> Option<String>;
    fn get_actions(&self) -> &Vec<Action>;
    fn type_name(&self) -> &'static str;
}

/// Snap/return tween configuration for DragAndDrop (duration in seconds,
/// cubic-bézier easing — same convention as Tweened transitions).
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DragTween {
    pub duration: f32,
    pub easing: [f32; 4],
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DropZone {
    pub layer_name: DotString,
    /// Snap target override; when omitted, the zone layer's authored
    /// position is used (extracted from the animation).
    #[serde(default)]
    pub snap: Option<[f32; 2]>,
    /// When true, the object can no longer be grabbed after docking here.
    #[serde(default)]
    pub lock: Option<bool>,
    /// When true, docking binds the slot to the zone layer's position via a
    /// Lottie expression, so the object follows a MOVING zone. Requires an
    /// expressions-capable player (the written static value is the
    /// fallback). Tracking docks are always locked.
    #[serde(default)]
    pub track: Option<bool>,
    /// Actions executed when the object docks in this zone.
    #[serde(default)]
    pub actions: Option<Vec<Action>>,
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
    /// Stateful drag & drop gesture: `layer_name` is the draggable layer
    /// (engine hit-tests pickup), `slot_id` the Position slot written while
    /// held. Releasing over a drop zone snaps to it and runs its actions;
    /// releasing elsewhere returns to the pre-drag rest position.
    DragAndDrop {
        layer_name: DotString,
        slot_id: DotString,
        /// Scope the gesture to a state (like OnComplete): grabbing only
        /// works while this state is current, and leaving it mid-drag
        /// cancels the gesture back to the rest position. Omitted = active
        /// in every state.
        state_name: Option<DotString>,
        /// Preserve the pointer-to-object offset from grab (default true).
        grab_offset: Option<bool>,
        /// Snap/return glide; omitted = instant.
        tween: Option<DragTween>,
        drop_zones: Vec<DropZone>,
    },
}

static EMPTY_ACTIONS: Vec<Action> = Vec::new();

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
            Interaction::DragAndDrop { layer_name, .. } => Some(layer_name),
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
            // DragAndDrop actions live per drop zone and are executed by the
            // gesture runtime, never by the generic interaction path.
            Interaction::DragAndDrop { .. } => &EMPTY_ACTIONS,
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
            Interaction::DragAndDrop { state_name, .. } => {
                state_name.as_ref().map(|s| s.as_str().to_owned())
            }
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
            // Never matches a pointer event type name, so the generic
            // explicit-event path ignores DragAndDrop entirely.
            Interaction::DragAndDrop { .. } => "DragAndDrop",
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
            Interaction::DragAndDrop {
                layer_name,
                slot_id,
                state_name,
                drop_zones,
                ..
            } => {
                *layer_name = interner.intern(layer_name.as_str());
                *slot_id = interner.intern(slot_id.as_str());
                if let Some(state) = state_name {
                    *state = interner.intern(state.as_str());
                }
                for zone in drop_zones {
                    zone.layer_name = interner.intern(zone.layer_name.as_str());
                    if let Some(actions) = &mut zone.actions {
                        for a in actions {
                            a.intern_identifiers(interner);
                        }
                    }
                }
            }
        }
    }
}
