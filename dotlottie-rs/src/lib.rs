#[cfg(feature = "audio")]
mod audio;
#[cfg(feature = "dotlottie")]
mod fms;
mod layout;
mod lottie_renderer;
mod markers;
mod player;
mod poll_events;
mod result;
#[cfg(feature = "state-machines")]
mod state_machine_engine;
#[cfg(feature = "theming")]
mod theme;
pub(crate) mod time;

#[cfg(feature = "c_api")]
pub mod c_api;

// wasm32-unknown-unknown modules
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
mod stubs;

#[cfg(all(target_arch = "wasm32", feature = "webgl"))]
pub(crate) mod webgl_stubs;

#[cfg(all(target_arch = "wasm32", feature = "webgpu"))]
pub(crate) mod webgpu_stubs;

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-api"))]
pub mod wasm_bindgen_api;

#[cfg(feature = "audio")]
pub use audio::*;
#[cfg(feature = "dotlottie")]
pub use fms::*;
pub use layout::*;
pub use lottie_renderer::*;
pub use markers::*;
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
