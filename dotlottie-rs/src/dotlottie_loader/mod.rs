use jzon::JsonValue;
use std::collections::HashMap;
use std::io::{self, Read};
use zip::ZipArchive;

use crate::errors::DotLottiePlayerError;
use crate::utils::base64_encode;

#[derive(Clone)]
pub struct ManifestAnimation {
    pub autoplay: Option<bool>,
    pub default_theme: Option<String>,
    pub direction: Option<i8>,
    pub hover: Option<bool>,
    pub id: String,
    pub intermission: Option<u32>,
    pub r#loop: Option<bool>,
    pub loop_count: Option<u32>,
    pub play_mode: Option<String>,
    pub speed: Option<u32>,
    pub theme_color: Option<String>,
}

impl ManifestAnimation {
    pub fn from_json(json: &JsonValue) -> Self {
        let autoplay = json["autoplay"].as_bool();
        let default_theme = json["default_theme"].as_str().map(|s| s.to_string());
        let direction = json["direction"].as_i8();
        let hover = json["hover"].as_bool();
        let id = json["id"].as_str().unwrap().to_string();
        let intermission = json["intermission"].as_u32();
        let r#loop = json["loop"].as_bool();
        let loop_count = json["loop_count"].as_u32();
        let play_mode = json["play_mode"].as_str().map(|s| s.to_string());
        let speed = json["speed"].as_u32();
        let theme_color = json["theme_color"].as_str().map(|s| s.to_string());

        ManifestAnimation {
            autoplay,
            default_theme,
            direction,
            hover,
            id,
            intermission,
            r#loop,
            loop_count,
            play_mode,
            speed,
            theme_color,
        }
    }
}

#[derive(Clone)]
pub struct ManifestTheme {
    pub id: String,
    pub animations: Vec<String>,
}

impl ManifestTheme {
    pub fn from_json(json: &JsonValue) -> Self {
        let id = json["id"].as_str().unwrap().to_string();
        let animations = json["animations"]
            .members()
            .map(|animation| animation.as_str().unwrap().to_string())
            .collect();

        ManifestTheme { id, animations }
    }
}

#[derive(Clone)]
pub struct Manifest {
    pub active_animation_id: Option<String>,
    pub animations: Vec<ManifestAnimation>,
    pub themes: Option<Vec<ManifestTheme>>,
    pub states: Option<Vec<String>>,
}

impl Manifest {
    pub fn from_json(json: &JsonValue) -> Self {
        let active_animation_id = json["activeAnimationId"].as_str().map(|s| s.to_string());
        let animations = json["animations"]
            .members()
            .map(|animation| ManifestAnimation::from_json(animation))
            .collect();
        let themes = json["themes"]
            .members()
            .map(|theme| ManifestTheme::from_json(theme))
            .collect();
        let states = json["states"]
            .members()
            .map(|state| state.as_str().unwrap().to_string())
            .collect();

        Manifest {
            active_animation_id,
            animations,
            themes: Some(themes),
            states: Some(states),
        }
    }
}

// required for `&Manifest` to implement `Into<JsonValue>`

pub struct DotLottieLoader {
    active_animation_id: String,
    manifest: Option<Manifest>,
    zip_data: Vec<u8>,
    animation_cache: HashMap<String, String>,
    animation_settings_cache: HashMap<String, ManifestAnimation>,
    theme_cache: HashMap<String, String>,
}

impl DotLottieLoader {
    pub fn new(bytes: &[u8]) -> Self {
        let manifest = get_manifest(bytes).unwrap();

        // get from the manifest the active animation id
        let active_animation_id = match &manifest.active_animation_id {
            Some(id) => id.to_string(),
            None => {
                // if there is no active animation id, get the first animation id
                match manifest.animations.first() {
                    Some(animation) => animation.id.to_string(),
                    None => "".to_string(),
                }
            }
        };

        Self {
            active_animation_id,
            manifest: Some(manifest),
            zip_data: bytes.to_vec(),
            animation_cache: HashMap::new(),
            animation_settings_cache: HashMap::new(),
            theme_cache: HashMap::new(),
        }
    }

    pub fn active_animation_id(&self) -> &str {
        &self.active_animation_id
    }

    pub fn manifest(&self) -> Option<&Manifest> {
        match &self.manifest {
            Some(manifest) => Some(manifest),
            None => None,
        }
    }

    pub fn get_manifest_animation(
        &mut self,
        animation_id: &str,
    ) -> Result<ManifestAnimation, DotLottiePlayerError> {
        match self.animation_settings_cache.get(animation_id) {
            Some(animation_settings) => Ok(animation_settings.clone()),
            None => {
                let animation_settings = self
                    .manifest
                    .as_ref()
                    .unwrap()
                    .animations
                    .iter()
                    .find(|animation| animation.id == animation_id)
                    .unwrap();

                self.animation_settings_cache
                    .insert(animation_id.to_string(), animation_settings.clone());

                Ok(animation_settings.clone())
            }
        }
    }

    pub fn get_animation(&mut self, animation_id: &str) -> Result<String, DotLottiePlayerError> {
        if let Some(animation) = self.animation_cache.get(animation_id) {
            return Ok(animation.clone());
        }

        let animation = crate::get_animation(&self.zip_data, animation_id)?;

        self.animation_cache
            .insert(animation_id.to_string(), animation.clone());

        Ok(animation)
    }

    pub fn get_theme_manifest(
        &self,
        theme_id: &str,
    ) -> Result<Vec<ManifestTheme>, DotLottiePlayerError> {
        if let Some(manifest) = &self.manifest {
            if let Some(themes) = &manifest.themes {
                return Ok(themes
                    .iter()
                    .filter(|theme| theme.id == theme_id)
                    .map(|theme| theme.clone())
                    .collect());
            }
        }

        Err(DotLottiePlayerError::ManifestNotFound)
    }

    pub fn get_theme(&mut self, theme_id: &str) -> Result<String, DotLottiePlayerError> {
        if let Some(theme) = self.theme_cache.get(theme_id) {
            return Ok(theme.clone());
        }

        let theme = crate::get_theme(&self.zip_data, theme_id)?;

        self.theme_cache.insert(theme_id.to_string(), theme.clone());

        Ok(theme)
    }
}

pub fn get_animation(bytes: &[u8], animation_id: &str) -> Result<String, DotLottiePlayerError> {
    let mut archive = ZipArchive::new(io::Cursor::new(bytes))
        .map_err(|_| DotLottiePlayerError::ArchiveOpenError)?;

    let search_file_name = format!("animations/{}.json", animation_id);

    let mut result =
        archive
            .by_name(&search_file_name)
            .map_err(|_| DotLottiePlayerError::FileFindError {
                file_name: search_file_name,
            })?;

    let mut content = Vec::new();

    result
        .read_to_end(&mut content)
        .map_err(|_| DotLottiePlayerError::ReadContentError)?;

    // We can drop result so that we can use archive later, everything has been read in to content variable
    drop(result);

    let animation_data = String::from_utf8(content).unwrap();

    // Untyped JSON value
    let mut lottie_animation = jzon::parse(&animation_data).unwrap();

    // Loop through the parsed lottie animation and check for image assets
    if let Some(assets) = lottie_animation["assets"].as_array_mut() {
        for i in 0..assets.len() {
            if !assets[i]["p"].is_null() {
                let image_asset_filename =
                    format!("images/{}", assets[i]["p"].to_string().replace("\"", ""));

                let image_ext = assets[i]["p"]
                    .to_string()
                    .split(".")
                    .last()
                    .unwrap()
                    .to_string()
                    .replace("\"", "");

                let mut result = archive.by_name(&image_asset_filename).map_err(|_| {
                    DotLottiePlayerError::FileFindError {
                        file_name: image_asset_filename,
                    }
                })?;

                let mut content = Vec::new();

                result
                    .read_to_end(&mut content)
                    .map_err(|_| DotLottiePlayerError::ReadContentError)?;

                let image_data_base64 = base64_encode(&content);

                assets[i]["u"] = "".into();
                assets[i]["p"] =
                    format!("data:image/{};base64,{}", image_ext, image_data_base64).into();
            }
        }
    }

    Ok(jzon::stringify(lottie_animation))
}

pub fn get_manifest(bytes: &[u8]) -> Result<Manifest, DotLottiePlayerError> {
    let mut archive = ZipArchive::new(io::Cursor::new(bytes))
        .map_err(|_| DotLottiePlayerError::ArchiveOpenError)?;

    let mut result =
        archive
            .by_name("manifest.json")
            .map_err(|_| DotLottiePlayerError::FileFindError {
                file_name: String::from("manifest.json"),
            })?;

    let mut content = Vec::new();
    result
        .read_to_end(&mut content)
        .map_err(|_| DotLottiePlayerError::ReadContentError)?;

    let manifest_string = String::from_utf8_lossy(&content).to_string();
    let manifest_json_value = jzon::parse(&manifest_string).unwrap();

    let manifest = Manifest::from_json(&manifest_json_value);

    Ok(manifest)
}

pub fn get_theme(bytes: &[u8], theme_id: &str) -> Result<String, DotLottiePlayerError> {
    let mut archive = ZipArchive::new(io::Cursor::new(bytes))
        .map_err(|_| DotLottiePlayerError::ArchiveOpenError)?;
    let search_file_name = format!("themes/{}.json", theme_id);

    let mut content = Vec::new();
    archive
        .by_name(&search_file_name)
        .map_err(|_| DotLottiePlayerError::FileFindError {
            file_name: search_file_name,
        })?
        .read_to_end(&mut content)
        .map_err(|_| DotLottiePlayerError::ReadContentError)?;

    String::from_utf8(content).map_err(|_| DotLottiePlayerError::InvalidUtf8Error)
}
