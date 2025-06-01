// hint: this is a workaround as the generated code from uniffi has empty lines after doc comments
#![allow(clippy::empty_line_after_doc_comments)]

pub use dotlottie_rs::*;

pub use dotlottie_rs::actions::open_url::{OpenUrl, OpenUrlMode};

use dotlottie_rs::actions::open_url::OpenUrl as OpenUrlType;

pub fn create_default_layout() -> Layout {
    Layout::default()
}

pub fn create_default_open_url() -> OpenUrlType {
    OpenUrlType::default()
}

pub fn create_default_config() -> Config {
    Config::default()
}

pub fn transform_theme_to_lottie_slots(theme_data: &str, animation_id: &str) -> String {
    dotlottie_rs::transform_theme_to_lottie_slots(theme_data, animation_id).unwrap_or_default()
}
