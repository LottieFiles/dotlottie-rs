use json::{self, object};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestTheme {
    pub id: String,
    pub animations: Vec<String>,
}

impl ManifestTheme {
    pub fn to_json(&self) -> json::JsonValue {
        object! {
            "id" => self.id.clone(),
            "animations" => self.animations.clone(),
        }
    }
}
