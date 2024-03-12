use json::{self, object};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ManifestTheme {
    pub id: String,
    pub animations: Vec<String>,
}

impl ManifestTheme {
    pub fn new(id: String, animations: Vec<String>) -> Self {
        Self { id, animations }
    }

    pub fn to_json(&self) -> json::JsonValue {
        object! {
            "id" => self.id.clone(),
            "animations" => self.animations.clone(),
        }
    }
}
