mod poll_events;
mod player;
mod fms;
mod layout;
mod lottie_renderer;
mod markers;
#[cfg(feature = "state-machines")]
mod state_machine_engine;
#[cfg(feature = "theming")]
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
#[cfg(feature = "state-machines")]
pub use state_machine_engine::events::*;
#[cfg(feature = "state-machines")]
pub use state_machine_engine::security::*;
#[cfg(feature = "state-machines")]
pub use state_machine_engine::*;
#[cfg(feature = "theming")]
pub use theme::*;

#[cfg(feature = "tvg")]
pub fn register_font(font_name: &str, font_data: &[u8]) -> bool {
    use lottie_renderer::Renderer;
    crate::TvgRenderer::register_font(font_name, font_data).is_ok()
}
