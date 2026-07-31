//! Runtime for the DragAndDrop interaction (prototype).
//!
//! Each DragAndDrop interaction gets a `DndRuntime` tracking its gesture
//! phase: Idle -> Held (slot follows pointer + grab offset) -> Snapping
//! (non-blocking property tween into a drop zone or back to the rest
//! position) -> Idle. Snap targets come from the drop-zone layer's authored
//! position unless overridden, so coordinates live in the animation.

use super::actions::Action;
use super::interactions::Interaction;

#[derive(Debug, Clone)]
pub(crate) struct DndZone {
    pub layer_name: String,
    pub snap: Option<[f32; 2]>,
    pub lock: bool,
    pub track: bool,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DndPhase {
    Idle,
    Held {
        offset: [f32; 2],
    },
    Snapping {
        from: [f32; 2],
        to: [f32; 2],
        elapsed: f32,
        duration: f32,
        easing: [f32; 4],
        /// Index into `zones` when docking; None for a miss-return.
        zone_index: Option<usize>,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct DndRuntime {
    pub layer_name: String,
    pub slot_id: String,
    /// When set, the gesture only operates while this state is current;
    /// leaving the state mid-drag cancels back to the rest position.
    pub state_name: Option<String>,
    /// Layer whose rendered bounds constrain the drag (center clamped
    /// inside, inset by the object's half-extents).
    pub boundary: Option<String>,
    /// (duration ms, easing); None = snap instantly.
    pub tween: Option<(f32, [f32; 4])>,
    pub on_grab: Vec<Action>,
    pub on_drag: Vec<Action>,
    pub on_drop: Vec<Action>,
    pub zones: Vec<DndZone>,
    pub phase: DndPhase,
    /// Where the object returns on a missed drop. Captured at first grab,
    /// updated to the snap target on every dock.
    pub rest: Option<[f32; 2]>,
    /// Transform position minus rendered visual center, captured at grab.
    /// Snapping adds it back so the object CENTERS on the zone even when
    /// its anchor point is off-center.
    pub anchor_offset: [f32; 2],
    /// Half width/height of the object's rendered bounds, captured at
    /// first grab; insets the boundary clamp so the whole object fits.
    pub half_extents: [f32; 2],
    /// Zone index being FOLLOWED while docked on a `track` zone: each tick
    /// the engine reads the zone's rendered center and rewrites the slot.
    /// No expressions involved, and the engine always knows the object's
    /// position — so re-grabbing works (grab clears this).
    pub tracking: Option<usize>,
    /// Ghost mode requested (free mode only).
    pub ghost: bool,
    /// A ghost duplicate is currently on the canvas; while true, the drag
    /// moves the ghost (canvas-pixel offsets) and the slot is untouched
    /// until landing.
    pub ghost_active: bool,
    /// Canvas-pixel pointer position at ghost grab (offsets are relative
    /// to it).
    pub ghost_origin: [f32; 2],
    /// Where (comp units) the slot lands when the ghost glide completes.
    pub ghost_land: Option<[f32; 2]>,
    pub locked: bool,
}

impl DndRuntime {
    pub(crate) fn from_interaction(interaction: &Interaction) -> Option<Self> {
        let Interaction::DragAndDrop {
            layer_name,
            slot_id,
            path_layer_name,
            state_name,
            boundary_layer_name,
            ghost,
            tween,
            on_grab,
            on_drag,
            on_drop,
            drop_zones,
            ..
        } = interaction
        else {
            return None;
        };

        // Path mode routes to PathDragRuntime instead; free/bounded mode
        // is inert without a slot to write.
        if path_layer_name.is_some() {
            return None;
        }
        let slot_id = slot_id.as_ref()?;

        Some(DndRuntime {
            layer_name: layer_name.as_str().to_owned(),
            slot_id: slot_id.as_str().to_owned(),
            state_name: state_name.as_ref().map(|s| s.as_str().to_owned()),
            boundary: boundary_layer_name.as_ref().map(|s| s.as_str().to_owned()),
            // Seconds -> milliseconds, same convention as Tweened transitions.
            tween: tween.as_ref().map(|t| (t.duration * 1000.0, t.easing)),
            on_grab: on_grab.clone().unwrap_or_default(),
            on_drag: on_drag.clone().unwrap_or_default(),
            on_drop: on_drop.clone().unwrap_or_default(),
            zones: drop_zones
                .iter()
                .map(|z| DndZone {
                    layer_name: z.layer_name.as_str().to_owned(),
                    snap: z.snap,
                    lock: z.lock.unwrap_or(false),
                    track: z.track.unwrap_or(false),
                    actions: z.actions.clone().unwrap_or_default(),
                })
                .collect(),
            phase: DndPhase::Idle,
            rest: None,
            anchor_offset: [0.0, 0.0],
            half_extents: [0.0, 0.0],
            tracking: None,
            ghost: ghost.unwrap_or(false),
            ghost_active: false,
            ghost_origin: [0.0, 0.0],
            ghost_land: None,
            locked: false,
        })
    }
}

pub(crate) fn lerp2(from: [f32; 2], to: [f32; 2], p: f32) -> [f32; 2] {
    [
        from[0] + (to[0] - from[0]) * p,
        from[1] + (to[1] - from[1]) * p,
    ]
}
