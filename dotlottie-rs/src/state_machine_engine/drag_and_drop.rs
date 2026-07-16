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
    pub use_grab_offset: bool,
    /// (duration ms, easing); None = snap instantly.
    pub tween: Option<(f32, [f32; 4])>,
    pub zones: Vec<DndZone>,
    pub phase: DndPhase,
    /// Where the object returns on a missed drop. Captured at first grab,
    /// updated to the snap target on every dock.
    pub rest: Option<[f32; 2]>,
    pub locked: bool,
}

impl DndRuntime {
    pub(crate) fn from_interaction(interaction: &Interaction) -> Option<Self> {
        let Interaction::DragAndDrop {
            layer_name,
            slot_id,
            grab_offset,
            tween,
            drop_zones,
        } = interaction
        else {
            return None;
        };

        Some(DndRuntime {
            layer_name: layer_name.as_str().to_owned(),
            slot_id: slot_id.as_str().to_owned(),
            use_grab_offset: grab_offset.unwrap_or(true),
            // Seconds -> milliseconds, same convention as Tweened transitions.
            tween: tween.as_ref().map(|t| (t.duration * 1000.0, t.easing)),
            zones: drop_zones
                .iter()
                .map(|z| DndZone {
                    layer_name: z.layer_name.as_str().to_owned(),
                    snap: z.snap,
                    lock: z.lock.unwrap_or(false),
                    actions: z.actions.clone().unwrap_or_default(),
                })
                .collect(),
            phase: DndPhase::Idle,
            rest: None,
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
