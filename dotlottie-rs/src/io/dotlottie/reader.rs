use super::base64;
use super::error::ReaderError;
use super::manifest::Manifest;
use super::zip::{DotLottieArchive, ZipError};
#[cfg(feature = "theming")]
use crate::theme::Theme;
use serde_json::Value;

const DATA_IMAGE_PREFIX: &str = "data:image/";
const DATA_FONT_PREFIX: &str = "data:font/";
const BASE64_PREFIX: &str = ";base64,";
const DEFAULT_EXT: &str = "png";
const DEFAULT_FONT_EXT: &str = "ttf";

pub struct Reader {
    initial_animation_id: Box<str>,
    manifest: Manifest,
    version: u8,
    archive: DotLottieArchive,
}

impl Reader {
    pub fn new(dotlottie: &[u8]) -> Result<Self, ReaderError> {
        let archive =
            DotLottieArchive::new(dotlottie.to_vec()).map_err(ReaderError::Zip)?;

        let manifest_bytes = archive.read("manifest.json").map_err(|e| match e {
            ZipError::FileNotFound => ReaderError::ManifestNotFound,
            other => ReaderError::Zip(other),
        })?;
        let manifest_str = std::str::from_utf8(&manifest_bytes)?;
        let manifest: Manifest = serde_json::from_str(manifest_str)?;

        let id = manifest
            .initial
            .as_ref()
            .and_then(|initial| initial.animation.as_ref())
            .or_else(|| manifest.animations.first().map(|a| &a.id))
            .ok_or(ReaderError::NoAnimations)?
            .clone()
            .into_boxed_str();

        let version = manifest
            .version
            .as_deref()
            .map(|v| if v == "2" { 2 } else { 1 })
            .unwrap_or(1);

        Ok(Reader {
            initial_animation_id: id,
            manifest,
            version,
            archive,
        })
    }

    #[inline]
    pub fn initial_animation(&self) -> Result<String, ReaderError> {
        self.animation(&self.initial_animation_id)
    }

    pub fn animation(&self, animation_id: &str) -> Result<String, ReaderError> {
        let (json_path, lot_path) = if self.version == 2 {
            (
                format!("a/{animation_id}.json"),
                format!("a/{animation_id}.lot"),
            )
        } else {
            (
                format!("animations/{animation_id}.json"),
                format!("animations/{animation_id}.lot"),
            )
        };

        let file_data = self
            .archive
            .read(&json_path)
            .or_else(|e| match e {
                ZipError::FileNotFound => self.archive.read(&lot_path),
                other => Err(other),
            })
            .map_err(|e| match e {
                ZipError::FileNotFound => ReaderError::AnimationNotFound,
                other => ReaderError::Zip(other),
            })?;

        let animation_data = std::str::from_utf8(&file_data)?;

        let mut lottie_animation: Value = serde_json::from_str(animation_data)?;

        if let Some(assets) = lottie_animation
            .get_mut("assets")
            .and_then(|v| v.as_array_mut())
        {
            self.embed_image_assets(assets)?;
        }

        if self.version == 2 {
            if let Some(fonts) = lottie_animation
                .get_mut("fonts")
                .and_then(|v| v.as_object_mut())
            {
                if let Some(font_list) = fonts.get_mut("list").and_then(|v| v.as_array_mut()) {
                    self.embed_font_assets(font_list)?;
                }
            }
        }

        Ok(serde_json::to_string(&lottie_animation)?)
    }

    #[inline]
    #[cfg(feature = "state-machines")]
    pub fn state_machine(&self, state_machine_id: &str) -> Result<String, ReaderError> {
        let path = format!("s/{state_machine_id}.json");
        let content = self.archive.read(&path).map_err(|e| match e {
            ZipError::FileNotFound => ReaderError::FileNotFound,
            other => ReaderError::Zip(other),
        })?;
        String::from_utf8(content.into_owned())
            .map_err(|e| ReaderError::InvalidUtf8(e.utf8_error()))
    }

    #[inline]
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    #[inline]
    pub fn initial_animation_id(&self) -> String {
        self.initial_animation_id.to_string()
    }

    #[inline]
    #[cfg(feature = "theming")]
    pub fn theme(&self, theme_id: &str) -> Result<Theme, ReaderError> {
        let path = format!("t/{theme_id}.json");
        let content = self.archive.read(&path).map_err(|e| match e {
            ZipError::FileNotFound => ReaderError::FileNotFound,
            other => ReaderError::Zip(other),
        })?;
        let theme_str = std::str::from_utf8(&content)?;
        Ok(theme_str.parse::<Theme>()?)
    }

    fn embed_image_assets(&self, assets: &mut [Value]) -> Result<(), ReaderError> {
        let image_prefix = if self.version == 2 { "i/" } else { "images/" };
        let mut asset_path = String::with_capacity(128);
        let mut data_url_buf = String::with_capacity(1024);

        let embedded_flag = Value::Number(1.into());
        let empty_u = Value::String(String::new());
        let key_e = "e".to_owned();
        let key_u = "u".to_owned();
        let key_p = "p".to_owned();

        for asset in assets.iter_mut() {
            if let Some(asset_obj) = asset.as_object_mut() {
                if let Some(p_str) = asset_obj.get("p").and_then(|v| v.as_str()) {
                    if p_str.starts_with(DATA_IMAGE_PREFIX) {
                        asset_obj.insert(key_e.clone(), embedded_flag.clone());
                    } else {
                        asset_path.clear();
                        asset_path.push_str(image_prefix);
                        asset_path.push_str(p_str.trim_matches('"'));

                        if let Ok(content) = self.archive.read(&asset_path) {
                            let image_ext = p_str
                                .rfind('.')
                                .map(|i| &p_str[i + 1..])
                                .unwrap_or(DEFAULT_EXT);

                            data_url_buf.clear();
                            data_url_buf.push_str(DATA_IMAGE_PREFIX);
                            data_url_buf.push_str(image_ext);
                            data_url_buf.push_str(BASE64_PREFIX);
                            base64::encode_into(&content, &mut data_url_buf);

                            asset_obj.insert(key_u.clone(), empty_u.clone());
                            asset_obj.insert(
                                key_p.clone(),
                                Value::String(std::mem::take(&mut data_url_buf)),
                            );
                            asset_obj.insert(key_e.clone(), embedded_flag.clone());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn embed_font_assets(&self, font_list: &mut [Value]) -> Result<(), ReaderError> {
        let mut font_path = String::with_capacity(128);
        let mut data_url_buf = String::with_capacity(1024);
        let key_fpath = "fPath".to_owned();

        for font in font_list.iter_mut() {
            if let Some(font_obj) = font.as_object_mut() {
                if let Some(f_path_str) = font_obj.get("fPath").and_then(|v| v.as_str()) {
                    if f_path_str.starts_with("/f/") {
                        font_path.clear();
                        font_path.push_str("f/");
                        let path_without_prefix =
                            f_path_str.strip_prefix("/f/").unwrap_or(f_path_str);
                        font_path.push_str(path_without_prefix);

                        if let Ok(content) = self.archive.read(&font_path) {
                            let font_ext = path_without_prefix
                                .rfind('.')
                                .map(|i| &path_without_prefix[i + 1..])
                                .unwrap_or(DEFAULT_FONT_EXT);

                            data_url_buf.clear();
                            data_url_buf.push_str(DATA_FONT_PREFIX);
                            data_url_buf.push_str(font_ext);
                            data_url_buf.push_str(BASE64_PREFIX);
                            base64::encode_into(&content, &mut data_url_buf);

                            font_obj.insert(
                                key_fpath.clone(),
                                Value::String(std::mem::take(&mut data_url_buf)),
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_dotlottie_reader_creation() {
        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/src/dotlottie/tests/resources/emoji-collection.lottie"
        );

        if let Ok(mut file) = File::open(&file_path) {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                let reader = Reader::new(&buffer);
                assert!(reader.is_ok());

                if let Ok(mgr) = reader {
                    assert!(!mgr.initial_animation_id().is_empty());
                    assert!(!mgr.manifest().animations.is_empty());
                }
            }
        }
    }
}
