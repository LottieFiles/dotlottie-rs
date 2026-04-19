pub use crate::event_queue::EventQueue;

/// Events emitted by the DotLottie player.
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerEvent {
    Load,
    LoadError,
    Play,
    Pause,
    Stop,
    Frame { frame_no: f32 },
    Render { frame_no: f32 },
    Loop { loop_count: u32 },
    Complete,
}
