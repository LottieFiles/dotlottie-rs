#![allow(clippy::too_many_arguments)]
use json::object;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManifestAnimation {
    pub autoplay: Option<bool>,
    pub defaultTheme: Option<String>,
    pub direction: Option<i8>,
    pub hover: Option<bool>,
    pub id: String,
    pub intermission: Option<u32>,
    pub r#loop: Option<bool>,
    pub loop_count: Option<u32>,
    pub playMode: Option<String>,
    pub speed: Option<f32>,
    pub themeColor: Option<String>,
}

#[allow(non_snake_case)]
impl ManifestAnimation {
    pub fn new(
        autoplay: Option<bool>,
        defaultTheme: Option<String>,
        direction: Option<i8>,
        hover: Option<bool>,
        id: String,
        intermission: Option<u32>,
        r#loop: Option<bool>,
        loop_count: Option<u32>,
        playMode: Option<String>,
        speed: Option<f32>,
        themeColor: Option<String>,
    ) -> Self {
        Self {
            autoplay: if autoplay.is_none() {
                Some(false)
            } else {
                autoplay
            },
            defaultTheme: if defaultTheme.is_none() {
                Some("".to_string())
            } else {
                defaultTheme
            },
            direction: if direction.is_none() {
                Some(1)
            } else {
                direction
            },
            hover: if hover.is_none() { Some(false) } else { hover },
            id,
            intermission: if intermission.is_none() {
                Some(0)
            } else {
                intermission
            },
            r#loop: if r#loop.is_none() {
                Some(false)
            } else {
                r#loop
            },
            loop_count: if loop_count.is_none() {
                Some(0)
            } else {
                loop_count
            },
            playMode: if playMode.is_none() {
                Some("Normal".to_string())
            } else {
                playMode
            },
            speed: if speed.is_none() { Some(1.0) } else { speed },
            themeColor: if themeColor.is_none() {
                Some("".to_string())
            } else {
                themeColor
            },
        }
    }

    pub fn new_with_id(id: String) -> Self {
        Self {
            autoplay: Some(false),
            defaultTheme: Some("".to_string()),
            direction: Some(1),
            hover: Some(false),
            id,
            intermission: Some(0),
            r#loop: Some(false),
            loop_count: Some(0),
            playMode: Some("normal".to_string()),
            speed: Some(1.0),
            themeColor: Some("".to_string()),
        }
    }

    pub fn to_json(&self) -> json::JsonValue {
        object! {
            "autoplay" => self.autoplay,
            "defaultTheme" => self.defaultTheme.clone(),
            "direction" => self.direction,
            "hover" => self.hover,
            "id" => self.id.clone(),
            "intermission" => self.intermission,
            "loop" => self.r#loop,
            "loopCount" => self.loop_count,
            "playMode" => self.playMode.clone(),
            "speed" => self.speed,
            "themeColor" => self.themeColor.clone(),
        }
    }
}

impl Display for ManifestAnimation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_json())
    }
}
