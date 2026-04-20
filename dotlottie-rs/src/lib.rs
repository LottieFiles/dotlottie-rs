#[cfg(feature = "audio")]
mod audio;
mod event_queue;
#[cfg(feature = "dotlottie")]
mod fms;
mod layout;
mod lottie_renderer;
mod player;
mod result;
#[cfg(feature = "state-machines")]
mod state_machine_engine;
mod string;
#[cfg(feature = "theming")]
mod theme;
mod tween;

#[cfg(feature = "c_api")]
pub mod c_api;

pub mod tools;

/// cbindgen:ignore
#[cfg(all(feature = "tvg", target_arch = "wasm32", not(target_os = "emscripten")))]
mod wasm;

/// cbindgen:ignore
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
pub use event_queue::*;
pub use player::*;
pub use result::*;
#[cfg(feature = "state-machines")]
pub use state_machine_engine::events::*;
#[cfg(feature = "state-machines")]
pub use state_machine_engine::security::*;
#[cfg(feature = "state-machines")]
pub use state_machine_engine::*;
pub use string::*;
#[cfg(feature = "theming")]
pub use theme::*;
pub use tween::TweenStatus;
