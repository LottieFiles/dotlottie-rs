use serde::{Deserialize, Serialize};
use std::string::String;
use std::vec::Vec;

#[derive(Clone, Deserialize)]
pub struct ManifestAnimation {
    pub id: String,
    pub autoplay: Option<bool>,
    pub r#loop: Option<bool>,
    pub direction: Option<i8>,
    pub play_mode: Option<String>,
    pub speed: Option<u32>,
    pub default_theme: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ManifestTheme {
    pub id: String,
    pub animations: Vec<String>,
}

#[derive(Clone, Deserialize)]
pub struct Manifest {
    pub active_animation_id: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub generator: Option<String>,
    pub keywords: Option<String>,
    pub revision: Option<u32>,
    pub version: Option<String>,
    pub animations: Vec<ManifestAnimation>,
    pub themes: Vec<ManifestTheme>,
    pub states: Vec<String>,
}
