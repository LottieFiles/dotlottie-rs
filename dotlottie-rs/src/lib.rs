#[cfg(feature = "audio")]
mod audio;
#[cfg(feature = "dotlottie")]
mod fms;
mod layout;
mod lottie_renderer;
mod player;
mod poll_events;
mod result;
#[cfg(feature = "state-machines")]
mod state_machine_engine;
#[cfg(feature = "theming")]
mod theme;
pub(crate) mod time;
mod tween;

#[cfg(feature = "c_api")]
pub mod c_api;

pub mod tools;

// wasm32-unknown-unknown: auto-enabled when targeting wasm32 (excluding emscripten) with ThorVG
#[cfg(all(feature = "tvg", target_arch = "wasm32", not(target_os = "emscripten")))]
mod wasm;

#[cfg(all(
    feature = "tvg",
    target_arch = "wasm32",
    not(target_os = "emscripten"),
    feature = "wasm-bindgen-api"
))]
pub use wasm::wasm_bindgen_api;

#[cfg(feature = "audio")]
pub use audio::*;
#[cfg(feature = "dotlottie")]
pub use fms::*;
pub use layout::*;
pub use lottie_renderer::*;
pub use player::*;
pub use poll_events::*;
pub use result::*;
#[cfg(feature = "state-machines")]
pub use state_machine_engine::events::*;
#[cfg(feature = "state-machines")]
pub use state_machine_engine::security::*;
#[cfg(feature = "state-machines")]
pub use state_machine_engine::*;
#[cfg(feature = "theming")]
pub use theme::*;
pub use tween::TweenStatus;
