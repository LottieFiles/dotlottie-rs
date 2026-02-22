mod errors;
mod manifest;

pub use errors::*;
pub use manifest::*;

use crate::asset_resolver::ResolvedAsset;
#[cfg(feature = "theming")]
use crate::theme::Theme;
use std::cell::RefCell;
use std::io::{self, Read};
use std::rc::Rc;
use zip::ZipArchive;

const DEFAULT_EXT: &str = "png";
const DEFAULT_FONT_EXT: &str = "ttf";

type SharedArchive = Rc<RefCell<ZipArchive<io::Cursor<Vec<u8>>>>>;

pub struct DotLottieManager {
    active_animation_id: Box<str>,
    manifest: Manifest,
    version: u8,
    archive: SharedArchive,
}

pub struct FmsAssetResolver {
    archive: SharedArchive,
    version: u8,
}

impl FmsAssetResolver {
    fn resolve_image(&self, src: &str) -> Option<ResolvedAsset> {
        let mut archive = self.archive.borrow_mut();

        let image_prefix = if self.version == 2 { "i/" } else { "images/" };
        let asset_path = format!("{image_prefix}{}", src.trim_matches('"'));

        let mut file = archive.by_name(&asset_path).ok()?;
        let mut data = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut data).ok()?;

        let mimetype = src
            .rfind('.')
            .map(|i| &src[i + 1..])
            .unwrap_or(DEFAULT_EXT)
            .to_string();

        Some(ResolvedAsset { data, mimetype })
    }

    fn resolve_font(&self, src: &str) -> Option<ResolvedAsset> {
        if self.version != 2 {
            return None;
        }

        let mut archive = self.archive.borrow_mut();

        // ThorVG passes font src as either a path or "name:<fontname>"
        let font_name = src.strip_prefix("name:").unwrap_or(src);

        // Search font files in f/ directory
        let file_names: Vec<String> = (0..archive.len())
            .filter_map(|i| {
                let file = archive.by_index(i).ok()?;
                let name = file.name().to_string();
                if name.starts_with("f/") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();

        for file_name in file_names {
            // Check if the font file name contains the font name
            let base_name = file_name
                .strip_prefix("f/")
                .unwrap_or(&file_name)
                .rsplit('.')
                .last()
                .unwrap_or("");

            if base_name.eq_ignore_ascii_case(font_name) || file_name.ends_with(font_name) {
                let mut file = archive.by_name(&file_name).ok()?;
                let mut data = Vec::with_capacity(file.size() as usize);
                file.read_to_end(&mut data).ok()?;

                let mimetype = file_name
                    .rfind('.')
                    .map(|i| &file_name[i + 1..])
                    .unwrap_or(DEFAULT_FONT_EXT)
                    .to_string();

                return Some(ResolvedAsset { data, mimetype });
            }
        }

        None
    }

    pub fn resolve(&self, src: &str) -> Option<ResolvedAsset> {
        if src.starts_with("name:") {
            self.resolve_font(src)
        } else {
            // Try image first, fall back to font
            self.resolve_image(src).or_else(|| self.resolve_font(src))
        }
    }
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
            archive: Rc::new(RefCell::new(archive)),
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

        // Return the raw JSON without base64-inlining assets.
        // The asset resolver callback will handle loading assets on-demand.
        String::from_utf8(file_data).map_err(|_| DotLottieError::ReadContentError)
    }

    pub fn create_fms_resolver(&self) -> FmsAssetResolver {
        FmsAssetResolver {
            archive: Rc::clone(&self.archive),
            version: self.version,
        }
    }

    #[inline]
    #[cfg(feature = "state-machines")]
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
    #[cfg(feature = "theming")]
    pub fn get_theme(&self, theme_id: &str) -> Result<Theme, DotLottieError> {
        let mut archive = self.archive.borrow_mut();
        let path = format!("t/{theme_id}.json");
        let content = Self::read_zip_file(&mut archive, &path)?;
        let theme_str =
            std::str::from_utf8(&content).map_err(|_| DotLottieError::InvalidUtf8Error)?;
        theme_str
            .parse::<Theme>()
            .map_err(|_| DotLottieError::ReadContentError)
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
