mod poll_events;
mod player;
mod fms;
mod layout;
mod lottie_renderer;
mod markers;
mod state_machine_engine;
mod theme;
pub(crate) mod time;

#[cfg(feature = "c_api")]
pub mod c_api;

pub use poll_events::*;
pub use player::*;
pub use fms::*;
pub use layout::*;
pub use lottie_renderer::*;
pub use markers::*;
pub use state_machine_engine::events::*;
pub use state_machine_engine::security::*;
pub use state_machine_engine::*;
pub use theme::*;

#[cfg(feature = "tvg")]
pub fn register_font(font_name: &str, font_data: &[u8]) -> bool {
    use lottie_renderer::Renderer;
    crate::TvgRenderer::register_font(font_name, font_data).is_ok()
}
