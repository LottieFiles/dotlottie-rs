use json::{self, object};

use serde::{Deserialize, Serialize};

use std::fmt::Display;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestTheme {
    pub id: String,
    pub values: Vec<String>,
}

impl ManifestTheme {
    pub fn to_json(&self) -> json::JsonValue {
        object! {
            "id" => self.id.clone(),
            "values" => self.values.clone(),
        }
    }
}

impl Display for ManifestTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let json_value = object! {
            "id" => self.id.clone(),
            "values" => self.values.clone(),
        };
        write!(f, "{}", json_value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestThemes {
    pub value: Option<Vec<ManifestTheme>>,
}

impl ManifestThemes {
    pub fn to_json(&self) -> json::JsonValue {
        if self.value.is_none() {
            return json::JsonValue::Array(vec![]);
        }

        let animations_json: Vec<json::JsonValue> = self
            .value
            .as_ref()
            .unwrap()
            .iter()
            .map(|themes| themes.to_json())
            .collect();

        json::JsonValue::Array(animations_json)
    }
}

impl std::fmt::Display for ManifestThemes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.value {
            Some(themes) => {
                let themes_json = themes
                    .iter()
                    .map(|theme| theme.to_json()) // Assuming ManifestTheme implements Display
                    .collect::<Vec<json::JsonValue>>();

                write!(
                    f,
                    "[{}]",
                    json::stringify(json::JsonValue::Array(themes_json))
                )
            }
            None => write!(f, "[]"),
        }
    }
}
