mod manifest;
mod zip;
use std::fmt;

pub use manifest::*;
pub use zip::ZipError;

use self::zip::DotLottieArchive;
#[cfg(feature = "theming")]
use crate::theme::Theme;
use serde_json::Value;

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const DATA_IMAGE_PREFIX: &str = "data:image/";
const DATA_FONT_PREFIX: &str = "data:font/";
const BASE64_PREFIX: &str = ";base64,";
const DEFAULT_EXT: &str = "png";
const DEFAULT_FONT_EXT: &str = "ttf";

#[derive(Debug)]
pub enum DotLottieReaderError {
    /// ZIP archive could not be opened or is malformed.
    Zip(ZipError),
    /// manifest.json not found in the archive.
    ManifestNotFound,
    /// No animations listed in the manifest.
    NoAnimations,
    /// Requested animation not found in the archive.
    AnimationNotFound,
    /// Requested file not found in the archive.
    FileNotFound,
    /// Content is not valid UTF-8.
    InvalidUtf8(std::str::Utf8Error),
    /// JSON parsing failed.
    InvalidJson(serde_json::Error),
}

impl fmt::Display for DotLottieReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for DotLottieReaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Zip(e) => Some(e),
            Self::InvalidUtf8(e) => Some(e),
            Self::InvalidJson(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::str::Utf8Error> for DotLottieReaderError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::InvalidUtf8(e)
    }
}

impl From<serde_json::Error> for DotLottieReaderError {
    fn from(e: serde_json::Error) -> Self {
        Self::InvalidJson(e)
    }
}

pub struct DotLottieReader {
    initial_animation_id: Box<str>,
    manifest: Manifest,
    version: u8,
    archive: DotLottieArchive,
}

impl DotLottieReader {
    pub fn new(dotlottie: &[u8]) -> Result<Self, DotLottieReaderError> {
        let archive =
            DotLottieArchive::new(dotlottie.to_vec()).map_err(DotLottieReaderError::Zip)?;

        let manifest_bytes = archive.read("manifest.json").map_err(|e| match e {
            ZipError::FileNotFound => DotLottieReaderError::ManifestNotFound,
            other => DotLottieReaderError::Zip(other),
        })?;
        let manifest_str = std::str::from_utf8(&manifest_bytes)?;
        let manifest: Manifest = serde_json::from_str(manifest_str)?;

        let id = manifest
            .initial
            .as_ref()
            .and_then(|initial| initial.animation.as_ref())
            .or_else(|| manifest.animations.first().map(|a| &a.id))
            .ok_or(DotLottieReaderError::NoAnimations)?
            .clone()
            .into_boxed_str();

        let version = manifest
            .version
            .as_deref()
            .map(|v| if v == "2" { 2 } else { 1 })
            .unwrap_or(1);

        Ok(DotLottieReader {
            initial_animation_id: id,
            manifest,
            version,
            archive,
        })
    }

    #[inline]
    pub fn initial_animation(&self) -> Result<String, DotLottieReaderError> {
        self.animation(&self.initial_animation_id)
    }

    pub fn animation(&self, animation_id: &str) -> Result<String, DotLottieReaderError> {
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
                ZipError::FileNotFound => DotLottieReaderError::AnimationNotFound,
                other => DotLottieReaderError::Zip(other),
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
    pub fn state_machine(&self, state_machine_id: &str) -> Result<String, DotLottieReaderError> {
        let path = format!("s/{state_machine_id}.json");
        let content = self.archive.read(&path).map_err(|e| match e {
            ZipError::FileNotFound => DotLottieReaderError::FileNotFound,
            other => DotLottieReaderError::Zip(other),
        })?;
        String::from_utf8(content.into_owned())
            .map_err(|e| DotLottieReaderError::InvalidUtf8(e.utf8_error()))
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
    pub fn theme(&self, theme_id: &str) -> Result<Theme, DotLottieReaderError> {
        let path = format!("t/{theme_id}.json");
        let content = self.archive.read(&path).map_err(|e| match e {
            ZipError::FileNotFound => DotLottieReaderError::FileNotFound,
            other => DotLottieReaderError::Zip(other),
        })?;
        let theme_str = std::str::from_utf8(&content)?;
        Ok(theme_str.parse::<Theme>()?)
    }

    fn embed_image_assets(&self, assets: &mut Vec<Value>) -> Result<(), DotLottieReaderError> {
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
                            Self::encode_base64_into(&content, &mut data_url_buf);

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

    fn embed_font_assets(&self, font_list: &mut Vec<Value>) -> Result<(), DotLottieReaderError> {
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
                            Self::encode_base64_into(&content, &mut data_url_buf);

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

    fn encode_base64_into(input: &[u8], out: &mut String) {
        let output_len = input.len().div_ceil(3) * 4;
        out.reserve(output_len);

        let mut i = 0;
        while i + 2 < input.len() {
            let b0 = input[i] as u32;
            let b1 = input[i + 1] as u32;
            let b2 = input[i + 2] as u32;
            let n = (b0 << 16) | (b1 << 8) | b2;

            out.push(BASE64_CHARS[((n >> 18) & 63) as usize] as char);
            out.push(BASE64_CHARS[((n >> 12) & 63) as usize] as char);
            out.push(BASE64_CHARS[((n >> 6) & 63) as usize] as char);
            out.push(BASE64_CHARS[(n & 63) as usize] as char);
            i += 3;
        }

        if i < input.len() {
            let b0 = input[i] as u32;
            let b1 = input.get(i + 1).copied().unwrap_or(0) as u32;
            let n = (b0 << 16) | (b1 << 8);

            out.push(BASE64_CHARS[((n >> 18) & 63) as usize] as char);
            out.push(BASE64_CHARS[((n >> 12) & 63) as usize] as char);
            out.push(if i + 1 < input.len() {
                BASE64_CHARS[((n >> 6) & 63) as usize] as char
            } else {
                '='
            });
            out.push('=');
        }
    }

    #[cfg(test)]
    fn encode_base64(input: &[u8]) -> String {
        let mut out = String::new();
        Self::encode_base64_into(input, &mut out);
        out
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
                let reader = DotLottieReader::new(&buffer);
                assert!(reader.is_ok());

                if let Ok(mgr) = reader {
                    assert!(!mgr.initial_animation_id().is_empty());
                    assert!(!mgr.manifest().animations.is_empty());
                }
            }
        }
    }

    #[test]
    fn test_base64_encoding() {
        let input = b"Hello, World!";
        let result = DotLottieReader::encode_base64(input);
        assert_eq!(result, "SGVsbG8sIFdvcmxkIQ==");

        let empty_input = b"";
        let empty_result = DotLottieReader::encode_base64(empty_input);
        assert_eq!(empty_result, "");
    }
}
