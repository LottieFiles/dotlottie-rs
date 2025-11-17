mod errors;
mod manifest;

pub use errors::*;
pub use manifest::*;

use serde_json::Value;
use std::cell::RefCell;
use std::io::{self, Read};
use zip::ZipArchive;

const DATA_IMAGE_PREFIX: &str = "data:image/";

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
            let embedded_flag = Value::Number(1.into());

            for asset in assets.iter_mut() {
                if let Some(asset_obj) = asset.as_object_mut() {
                    if let Some(p_str) = asset_obj.get("p").and_then(|v| v.as_str()) {
                        if p_str.starts_with(DATA_IMAGE_PREFIX) {
                            asset_obj.insert("e".to_string(), embedded_flag.clone());
                        }
                    }
                }
            }
        }

        serde_json::to_string(&lottie_animation).map_err(|_| DotLottieError::ReadContentError)
    }

    pub fn resolve_asset(&self, asset_path: &str) -> Result<Vec<u8>, DotLottieError> {
        let mut archive = self.archive.borrow_mut();

        let mut asset_path = asset_path.to_string();
        if asset_path.starts_with("/f/") {
            // font path handling
            asset_path = format!("f/{asset_path}");
        } else {
            // image path handling
            let image_prefix = if self.version == 2 { "i/" } else { "images/" };
            let asset_name = asset_path.split('/').next_back().unwrap_or("");
            asset_path = format!("{image_prefix}{asset_name}");
        }

        if let Ok(mut result) = archive.by_name(&asset_path) {
            let mut content = Vec::with_capacity(result.size() as usize);
            if result.read_to_end(&mut content).is_ok() {
                return Ok(content);
            }
        }

        Err(DotLottieError::FileFindError)
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
}
