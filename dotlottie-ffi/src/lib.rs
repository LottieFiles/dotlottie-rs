pub use dotlottie_rs::uniffi::prelude::*;

mod ffi;

pub fn create_default_layout() -> Layout {
    Layout::default()
}

pub fn create_default_config() -> Config {
    Config::default()
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub fn transform_theme_to_lottie_slots(theme_data: &str, animation_id: &str) -> String {
            theming::transform_theme_to_lottie_slots(theme_data, animation_id)
                .unwrap_or_default()
        }

        uniffi::include_scaffolding!("dotlottie_player_cpp");
    } else {
        uniffi::include_scaffolding!("dotlottie_player");
    }
}
