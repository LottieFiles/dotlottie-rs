pub use dotlottie_fms::*;
pub use dotlottie_player_core::*;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        uniffi::include_scaffolding!("dotlottie_player_cpp");
    } else {
        uniffi::include_scaffolding!("dotlottie_player");
    }
}
