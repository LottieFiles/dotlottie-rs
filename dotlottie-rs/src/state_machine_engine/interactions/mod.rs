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
// The DragAndDrop variant is much larger than the pointer variants, but
// interactions are parsed once into a small list — not worth boxing.
#[allow(clippy::large_enum_variant)]
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
        /// Scope to a state (OnComplete-style): actions only run while the
        /// named state is current. Omitted = active in every state.
        state_name: Option<DotString>,
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
    /// Stateful drag gesture. The constraint fields select the mode:
    ///
    /// - **Free** (default): the object follows the pointer; `slot_id`
    ///   (required in this mode) is the Position slot written while held.
    ///   Releasing over a drop zone snaps to it and runs its actions;
    ///   releasing elsewhere returns to the pre-drag rest position.
    /// - **Bounded** (`boundary_layer_name`): free, but the object is
    ///   clamped inside that layer's rendered bounds.
    /// - **Path** (`path_layer_name`): the pointer is projected onto that
    ///   layer's bezier path and the gesture becomes a pure progress
    ///   sensor writing `progress_input` — no slot writes, and drop zones
    ///   and boundary are ignored (the path is the constraint; visuals
    ///   come from whatever consumes the progress).
    DragAndDrop {
        layer_name: DotString,
        /// Position slot written while dragging (free/bounded modes).
        slot_id: Option<DotString>,
        /// Layer whose first shape path is the constraint curve; selects
        /// path mode.
        path_layer_name: Option<DotString>,
        /// Numeric input receiving arc-length progress in [0, 1]
        /// (path mode).
        progress_input: Option<DotString>,
        /// Path mode: where an UNCAPTURED release docks. "previous" =
        /// nearest zone at or behind the release progress (ratchet);
        /// "nearest" = nearest zone in either direction. Omitted = no
        /// fallback (zone output stays 0 and the machine decides).
        dock_fallback: Option<DotString>,
        /// Scope the gesture to a state (like OnComplete): grabbing only
        /// works while this state is current, and leaving it mid-drag
        /// cancels the gesture back to the rest position. Omitted = active
        /// in every state.
        state_name: Option<DotString>,
        /// Constrain the drag to a layer's rendered bounds: while held, the
        /// object's center is clamped into this layer's current oriented
        /// bounding box, inset by the object's own half-extents so the
        /// whole object stays inside. Read from the scene every move, so
        /// scaled/animated boundaries are honored.
        boundary_layer_name: Option<DotString>,
        /// Free mode: drag a frozen GHOST duplicate of the layer instead of
        /// the object itself. The original stays parked; on release the
        /// ghost glides to the dock (or back home on a miss) and the slot
        /// is written once at landing. Purely visual — state still flows
        /// only through the slot.
        ghost: Option<bool>,
        /// Snap/return glide; omitted = instant.
        tween: Option<DragTween>,
        /// Actions executed when the object is grabbed (e.g. Fire an event
        /// so states can react to the gesture starting).
        on_grab: Option<Vec<Action>>,
        /// Actions executed on every held pointer move, AFTER the gesture's
        /// sensor publishes — so a `$progressInput` ref reads the fresh
        /// value (e.g. converting path progress to a timeline scrub via
        /// SetProgress).
        on_drag: Option<Vec<Action>>,
        /// Actions executed when the object is released, before drop-zone
        /// resolution and regardless of its outcome. Not executed when a
        /// state-bound gesture is cancelled by leaving its owning state.
        on_drop: Option<Vec<Action>>,
        /// Drop zones to resolve against on release. EMPTY (or omitted)
        /// means the gesture is lifecycle-only: no snap, no return — the
        /// object stays wherever the release (or any live slot binding)
        /// leaves it. Ignored in path mode.
        #[serde(default)]
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
            // DragAndDrop actions live in gesture hooks and are executed
            // by the gesture runtimes, never by the generic path.
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
            // Never match pointer event type names, so the generic
            // explicit-event path ignores gesture interactions entirely.
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
            Interaction::PointerMove {
                state_name,
                actions,
            } => {
                if let Some(name) = state_name {
                    *name = interner.intern(name.as_str());
                }
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
                path_layer_name,
                progress_input,
                dock_fallback,
                state_name,
                boundary_layer_name,
                on_grab,
                on_drag,
                on_drop,
                drop_zones,
                ..
            } => {
                *layer_name = interner.intern(layer_name.as_str());
                for name in [
                    slot_id,
                    path_layer_name,
                    progress_input,
                    dock_fallback,
                    state_name,
                    boundary_layer_name,
                ]
                .into_iter()
                .flatten()
                {
                    *name = interner.intern(name.as_str());
                }
                for actions in [on_grab, on_drag, on_drop].into_iter().flatten() {
                    for a in actions {
                        a.intern_identifiers(interner);
                    }
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
