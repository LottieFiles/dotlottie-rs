#[cfg(feature = "audio")]
mod audio;
#[cfg(feature = "dotlottie")]
pub mod dotlottie;
mod event_queue;
mod layout;
pub mod player;
mod player_state;
mod poll_events;
pub mod renderer;
#[cfg(feature = "state-machines")]
pub mod state_machine;
pub mod string;
#[cfg(feature = "theming")]
pub mod theme;
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

#[cfg(feature = "dotlottie")]
pub use dotlottie::{
    DotLottieManager, Manifest, ManifestAnimation, ManifestInitial, ManifestStateMachine,
    ManifestTheme,
};
pub use layout::{Fit, Layout};
pub use player::{CompletionEvent, Mode, Player, Status};
pub use poll_events::PlayerEvent;
pub use renderer::{
    slots_from_json_string, Animation, Bezier, BezierValue, ColorSlot, ColorSpace, ColorValue,
    Drawable, GlContext, GlDisplay, GlSurface, GradientSlot, GradientStop, ImageSlot,
    LottieKeyframe, LottieProperty, Marker, PositionSlot, Renderer, Rgba, ScalarSlot, ScalarValue,
    Segment, Shape, SlotType, TextCaps, TextDocument, TextJustify, TextKeyframe, TextSlot,
    VectorSlot, WgpuDevice, WgpuInstance, WgpuTarget, WgpuTargetType,
};
#[cfg(feature = "tvg")]
pub use renderer::{TvgAnimation, TvgRenderer, TvgShape};
#[cfg(feature = "state-machines")]
pub use state_machine::events::{
    Event, EventName, PointerEvent, StateMachineEvent, StateMachineInternalEvent,
};
#[cfg(feature = "state-machines")]
pub use state_machine::{OpenUrlPolicy, StateMachineEngine, StateMachineEngineStatus};
#[cfg(feature = "theming")]
pub use theme::{
    transform_theme_to_lottie_slots, ColorKeyframe, ColorRule, GradientKeyframe, GradientRule,
    ImageRule, ImageValue, PositionKeyframe, PositionRule, ScalarKeyframe, ScalarRule, TextRule,
    TextRuleKeyframe, TextValue, Theme, ThemeRule, VectorKeyframe, VectorRule,
};
pub use tween::TweenStatus;
