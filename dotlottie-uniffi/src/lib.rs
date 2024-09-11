pub use dotlottie_rs::*;

pub fn create_default_layout() -> Layout {
    Layout::default()
}

pub fn create_default_config() -> Config {
    Config::default()
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        uniffi::include_scaffolding!("dotlottie_uniffi_cpp");
    } else {
        uniffi::include_scaffolding!("dotlottie_uniffi");
    }
}
