use json::{self, array, object};

use crate::{ManifestAnimation, ManifestTheme};
use serde::{Deserialize, Serialize};

use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub active_animation_id: Option<String>,
    pub animations: Vec<ManifestAnimation>,
    pub author: Option<String>,
    // pub custom: Option<record(string(), an>))>
    pub description: Option<String>,
    pub generator: Option<String>,
    pub keywords: Option<String>,
    pub revision: Option<u32>,
    pub themes: Option<Vec<ManifestTheme>>,
    pub states: Option<Vec<String>>,
    pub version: Option<String>,
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            active_animation_id: None,
            animations: vec![],
            author: Some("LottieFiles".to_string()),
            // custom,
            description: None,
            generator: Some("dotLottie-fms".to_string()),
            keywords: Some("dotLottie".to_string()),
            revision: Some(1),
            themes: None,
            states: None,
            version: Some("1.0.0".to_string()),
        }
    }

    pub fn as_json(&self) -> Result<String, std::fmt::Error> {
        let json_str = format!("{}", self);

        Ok(json_str)
    }

    pub fn to_json(&self) -> json::JsonValue {
        let mut json = object! {
            "activeAnimationId" => self.active_animation_id.clone(),
            "animations" => self.animations.iter().map(|animation| animation.to_json()).collect::<Vec<json::JsonValue>>(),
            "author" => self.author.clone(),
        };

        if self.description.is_some() {
            json["description"] = self.description.clone().into();
        }
        if let Some(themes) = &self.themes {
            json["themes"] = themes
                .iter()
                .map(|t| t.to_json())
                .collect::<Vec<json::JsonValue>>()
                .into();
        }
        if let Some(states) = &self.states {
            json["states"] = states
                .iter()
                .map(|t| t.clone().into())
                .collect::<Vec<json::JsonValue>>()
                .into();
        }

        json["generator"] = self.generator.clone().into();
        json["keywords"] = self.keywords.clone().into();
        json["revision"] = self.revision.into();
        json["version"] = self.version.clone().into();

        json
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_json())
    }
}
