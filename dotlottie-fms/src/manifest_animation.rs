use json::object;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestAnimation {
    pub autoplay: Option<bool>,
    pub default_theme: Option<String>,
    pub direction: Option<i8>,
    pub hover: Option<bool>,
    pub id: String,
    pub intermission: Option<u32>,
    pub _loop: Option<bool>,
    pub loop_count: Option<u32>,
    pub play_mode: Option<String>,
    pub speed: Option<u32>,
    pub theme_color: Option<String>,
}

impl ManifestAnimation {
    pub fn new(
        autoplay: Option<bool>,
        default_theme: Option<String>,
        direction: Option<i8>,
        hover: Option<bool>,
        id: String,
        intermission: Option<u32>,
        _loop: Option<bool>,
        loop_count: Option<u32>,
        play_mode: Option<String>,
        speed: Option<u32>,
        theme_color: Option<String>,
    ) -> Self {
        Self {
            autoplay: if autoplay.is_none() {
                Some(false)
            } else {
                autoplay
            },
            default_theme: if default_theme.is_none() {
                Some("".to_string())
            } else {
                default_theme
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
            _loop: if _loop.is_none() { Some(false) } else { _loop },
            loop_count: if loop_count.is_none() {
                Some(0)
            } else {
                loop_count
            },
            play_mode: if play_mode.is_none() {
                Some("Normal".to_string())
            } else {
                play_mode
            },
            speed: if speed.is_none() { Some(1) } else { speed },
            theme_color: if theme_color.is_none() {
                Some("".to_string())
            } else {
                theme_color
            },
        }
    }

    pub fn new_with_id(id: String) -> Self {
        Self {
            autoplay: Some(false),
            default_theme: Some("".to_string()),
            direction: Some(1),
            hover: Some(false),
            id,
            intermission: Some(0),
            _loop: Some(false),
            loop_count: Some(0),
            play_mode: Some("normal".to_string()),
            speed: Some(1),
            theme_color: Some("".to_string()),
        }
    }

    pub fn to_json(&self) -> json::JsonValue {
        object! {
            "autoplay" => self.autoplay,
            "defaultTheme" => self.default_theme.clone(),
            "direction" => self.direction,
            "hover" => self.hover,
            "id" => self.id.clone(),
            "intermission" => self.intermission,
            "loop" => self._loop,
            "loopCount" => self.loop_count,
            "playMode" => self.play_mode.clone(),
            "speed" => self.speed,
            "themeColor" => self.theme_color.clone(),
        }
    }
}

impl Display for ManifestAnimation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_json())
    }
}
