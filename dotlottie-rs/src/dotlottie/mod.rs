mod manifest;
mod zip;
use std::fmt;
use std::rc::Rc;

pub use manifest::*;
pub use zip::ZipError;

use self::zip::DotLottieArchive;
use crate::asset_resolver::ResolvedAsset;
#[cfg(feature = "theming")]
use crate::theme::Theme;

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

pub struct DotLottieAssetResolver {
    archive: Rc<DotLottieArchive>,
    version: u8,
}

impl DotLottieAssetResolver {
    fn resolve_image(&self, src: &str) -> Option<ResolvedAsset> {
        let image_prefix = if self.version == 2 { "i/" } else { "images/" };
        let asset_path = format!("{image_prefix}{}", src.trim_matches('"'));
        let data = self.archive.read(&asset_path).ok()?.into_owned();
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

        let font_name = src.strip_prefix("name:").unwrap_or(src);

        let file_name: Option<String> = self
            .archive
            .file_names()
            .filter(|name| name.starts_with("f/"))
            .find(|name| {
                let base_name = name
                    .strip_prefix("f/")
                    .unwrap_or(name)
                    .rsplit('.')
                    .last()
                    .unwrap_or("");
                base_name.eq_ignore_ascii_case(font_name) || name.ends_with(font_name)
            })
            .map(|s| s.to_string());

        let file_name = file_name?;
        let data = self.archive.read(&file_name).ok()?.into_owned();
        let mimetype = file_name
            .rfind('.')
            .map(|i| file_name[i + 1..].to_string())
            .unwrap_or_else(|| DEFAULT_FONT_EXT.to_string());

        Some(ResolvedAsset { data, mimetype })
    }

    pub fn resolve(&self, src: &str) -> Option<ResolvedAsset> {
        if src.starts_with("name:") {
            self.resolve_font(src)
        } else {
            self.resolve_image(src).or_else(|| self.resolve_font(src))
        }
    }
}

pub struct DotLottieReader {
    initial_animation_id: Box<str>,
    manifest: Manifest,
    version: u8,
    archive: Rc<DotLottieArchive>,
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
            archive: Rc::new(archive),
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

        // Return the raw JSON without base64-inlining assets.
        // The asset resolver callback will handle loading assets on-demand.
        let animation_data = std::str::from_utf8(&file_data)?;
        Ok(animation_data.to_string())
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

    pub fn create_resolver(&self) -> DotLottieAssetResolver {
        DotLottieAssetResolver {
            archive: Rc::clone(&self.archive),
            version: self.version,
        }
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

}
