mod errors;
mod manifest;

pub use errors::*;
pub use manifest::*;

use serde_json::Value;
use std::cell::RefCell;
use std::io::{self, Read};
use zip::ZipArchive;

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const DATA_IMAGE_PREFIX: &str = "data:image/";
const BASE64_PREFIX: &str = ";base64,";
const DEFAULT_EXT: &str = "png";

pub struct DotLottieManager {
    active_animation_id: Box<str>,
    manifest: Manifest,
    version: u8,
    archive: RefCell<ZipArchive<io::Cursor<Vec<u8>>>>,
}

impl DotLottieManager {
    pub fn new(dotlottie: &[u8]) -> Result<Self, DotLottieError> {
        let mut archive = ZipArchive::new(io::Cursor::new(dotlottie.to_vec()))
            .map_err(|_| DotLottieError::ArchiveOpenError)?;

        let manifest = Self::read_zip_file(&mut archive, "manifest.json")?;
        let manifest_str =
            std::str::from_utf8(&manifest).map_err(|_| DotLottieError::ReadContentError)?;
        let manifest: Manifest =
            serde_json::from_str(manifest_str).map_err(|_| DotLottieError::ReadContentError)?;

        let id = manifest
            .initial
            .as_ref()
            .and_then(|initial| initial.animation.as_ref())
            .or_else(|| manifest.animations.first().map(|a| &a.id))
            .ok_or(DotLottieError::AnimationsNotFound)?
            .clone()
            .into_boxed_str();

        let version = manifest
            .version
            .as_deref()
            .map(|v| if v == "2" { 2 } else { 1 })
            .unwrap_or(1);

        Ok(DotLottieManager {
            active_animation_id: id,
            manifest,
            version,
            archive: RefCell::new(archive),
        })
    }

    #[inline]
    pub fn get_active_animation(&self) -> Result<String, DotLottieError> {
        self.get_animation(&self.active_animation_id)
    }

    pub fn get_animation(&self, animation_id: &str) -> Result<String, DotLottieError> {
        let mut archive = self.archive.borrow_mut();

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

        let file_data = Self::read_zip_file(&mut archive, &json_path)
            .or_else(|_| Self::read_zip_file(&mut archive, &lot_path))?;

        let animation_data =
            std::str::from_utf8(&file_data).map_err(|_| DotLottieError::ReadContentError)?;

        let mut lottie_animation: Value =
            serde_json::from_str(animation_data).map_err(|_| DotLottieError::ReadContentError)?;

        if let Some(assets) = lottie_animation
            .get_mut("assets")
            .and_then(|v| v.as_array_mut())
        {
            let image_prefix = if self.version == 2 { "i/" } else { "images/" };
            let mut asset_path = String::with_capacity(128); // Larger initial capacity

            let embedded_flag = Value::Number(1.into());
            let empty_u = Value::String(String::new());

            for asset in assets.iter_mut() {
                if let Some(asset_obj) = asset.as_object_mut() {
                    if let Some(p_str) = asset_obj.get("p").and_then(|v| v.as_str()) {
                        if p_str.starts_with(DATA_IMAGE_PREFIX) {
                            asset_obj.insert("e".to_string(), embedded_flag.clone());
                        } else {
                            asset_path.clear();
                            asset_path.push_str(image_prefix);
                            asset_path.push_str(p_str.trim_matches('"'));

                            if let Ok(mut result) = archive.by_name(&asset_path) {
                                let mut content = Vec::with_capacity(result.size() as usize);
                                if result.read_to_end(&mut content).is_ok() {
                                    let image_ext = p_str
                                        .rfind('.')
                                        .map(|i| &p_str[i + 1..])
                                        .unwrap_or(DEFAULT_EXT);
                                    let image_data_base64 = Self::encode_base64(&content);

                                    let data_url = format!(
                                        "{DATA_IMAGE_PREFIX}{image_ext}{BASE64_PREFIX}{image_data_base64}"
                                    );

                                    asset_obj.insert("u".to_string(), empty_u.clone());
                                    asset_obj.insert("p".to_string(), Value::String(data_url));
                                    asset_obj.insert("e".to_string(), embedded_flag.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        serde_json::to_string(&lottie_animation).map_err(|_| DotLottieError::ReadContentError)
    }

    #[inline]
    pub fn get_state_machine(&self, state_machine_id: &str) -> Result<String, DotLottieError> {
        let mut archive = self.archive.borrow_mut();
        let path = format!("s/{state_machine_id}.json");
        let content = Self::read_zip_file(&mut archive, &path)?;
        String::from_utf8(content).map_err(|_| DotLottieError::InvalidUtf8Error)
    }

    #[inline]
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    #[inline]
    pub fn active_animation_id(&self) -> String {
        self.active_animation_id.to_string()
    }

    #[inline]
    pub fn get_theme(&self, theme_id: &str) -> Result<String, DotLottieError> {
        let mut archive = self.archive.borrow_mut();
        let path = format!("t/{theme_id}.json");
        let content = Self::read_zip_file(&mut archive, &path)?;
        String::from_utf8(content).map_err(|_| DotLottieError::InvalidUtf8Error)
    }

    #[inline]
    fn encode_base64(input: &[u8]) -> String {
        if input.is_empty() {
            return String::new();
        }

        let output_len = input.len().div_ceil(3) * 4;
        let mut result = Vec::with_capacity(output_len);

        let mut i = 0;
        while i + 2 < input.len() {
            let b0 = input[i] as u32;
            let b1 = input[i + 1] as u32;
            let b2 = input[i + 2] as u32;
            let n = (b0 << 16) | (b1 << 8) | b2;

            result.push(BASE64_CHARS[((n >> 18) & 63) as usize]);
            result.push(BASE64_CHARS[((n >> 12) & 63) as usize]);
            result.push(BASE64_CHARS[((n >> 6) & 63) as usize]);
            result.push(BASE64_CHARS[(n & 63) as usize]);
            i += 3;
        }

        if i < input.len() {
            let b0 = input[i] as u32;
            let b1 = input.get(i + 1).copied().unwrap_or(0) as u32;
            let n = (b0 << 16) | (b1 << 8);

            result.push(BASE64_CHARS[((n >> 18) & 63) as usize]);
            result.push(BASE64_CHARS[((n >> 12) & 63) as usize]);
            result.push(if i + 1 < input.len() {
                BASE64_CHARS[((n >> 6) & 63) as usize]
            } else {
                b'='
            });
            result.push(b'=');
        }

        // safe conversion from Vec<u8> to String since we only used valid ASCII
        unsafe { String::from_utf8_unchecked(result) }
    }

    #[inline]
    fn read_zip_file<R: Read + io::Seek>(
        archive: &mut ZipArchive<R>,
        path: &str,
    ) -> Result<Vec<u8>, DotLottieError> {
        let mut file = archive
            .by_name(path)
            .map_err(|_| DotLottieError::FileFindError)?;

        let mut buf = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buf)
            .map_err(|_| DotLottieError::ReadContentError)?;

        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_dotlottie_manager_creation() {
        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/src/fms/tests/resources/emoji-collection.lottie"
        );

        if let Ok(mut file) = File::open(&file_path) {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                let manager = DotLottieManager::new(&buffer);
                assert!(manager.is_ok());

                if let Ok(mgr) = manager {
                    assert!(!mgr.active_animation_id().is_empty());
                    assert!(!mgr.manifest().animations.is_empty());
                }
            }
        }
    }

    #[test]
    fn test_base64_encoding() {
        let input = b"Hello, World!";
        let result = DotLottieManager::encode_base64(input);
        assert_eq!(result, "SGVsbG8sIFdvcmxkIQ==");

        let empty_input = b"";
        let empty_result = DotLottieManager::encode_base64(empty_input);
        assert_eq!(empty_result, "");
    }
}
