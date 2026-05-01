use super::error::ReaderError;
use super::manifest::Manifest;
use super::zip::{DotLottieArchive, ZipError};
use crate::lottie_renderer::AssetResolver;
#[cfg(feature = "theming")]
use crate::theme::Theme;
use std::borrow::Cow;
use std::sync::Arc;

/// Categorises an asset reference looked up from a `.lottie` archive.
///
/// The on-disk prefix differs by archive version:
/// * v1 — `images/`, `audio/`, fonts not supported
/// * v2 — `i/`, `u/`, `f/`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Image,
    Audio,
    Font,
}

pub struct Reader {
    initial_animation_id: Box<str>,
    manifest: Manifest,
    version: u8,
    archive: Arc<DotLottieArchive>,
}

impl Reader {
    pub fn new(dotlottie: &[u8]) -> Result<Self, ReaderError> {
        let archive = DotLottieArchive::new(dotlottie.to_vec()).map_err(ReaderError::Zip)?;

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
            archive: Arc::new(archive),
        })
    }

    /// Build a boxed [`AssetResolver`] backed by this reader's archive.
    ///
    /// The resolver clones the archive `Arc`, so it stays valid even if the
    /// owning [`Reader`] is dropped while the resolver is still registered
    /// with the renderer.
    pub fn asset_resolver(&self) -> Box<dyn AssetResolver> {
        Box::new(DotLottieAssetResolver {
            archive: Arc::clone(&self.archive),
            version: self.version,
        })
    }

    #[inline]
    pub fn version(&self) -> u8 {
        self.version
    }

    #[inline]
    pub fn initial_animation(&self) -> Result<Vec<u8>, ReaderError> {
        self.animation(&self.initial_animation_id)
    }

    /// Returns the raw, unmodified animation JSON bytes for `animation_id`.
    ///
    /// External asset references (images, fonts, audio) are intentionally not
    /// embedded; pair this with [`Reader::asset_bytes`] (or ThorVG's asset
    /// resolver via `Player`) to materialise assets on demand.
    pub fn animation(&self, animation_id: &str) -> Result<Vec<u8>, ReaderError> {
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

        Ok(file_data.into_owned())
    }

    /// Resolve an external asset by source name and kind.
    ///
    /// `src` is the value taken from a Lottie field such as `assets[*].p`
    /// (image / audio) or `fonts.list[*].fPath` (font, with the leading
    /// `/f/` already stripped by the caller). Returns `None` when the asset
    /// is not present in the archive.
    pub fn asset_bytes(&self, kind: AssetKind, src: &str) -> Option<Cow<'_, [u8]>> {
        archive_asset_bytes(&self.archive, self.version, kind, src)
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
}

/// Look up an asset by `(version, kind, src)` in a `.lottie` archive.
fn archive_asset_bytes<'a>(
    archive: &'a DotLottieArchive,
    version: u8,
    kind: AssetKind,
    src: &str,
) -> Option<Cow<'a, [u8]>> {
    let prefix = match (version, kind) {
        (2, AssetKind::Image) => "i/",
        (1, AssetKind::Image) => "images/",
        (2, AssetKind::Audio) => "u/",
        (1, AssetKind::Audio) => "audio/",
        (2, AssetKind::Font) => "f/",
        _ => return None,
    };
    let mut path = String::with_capacity(prefix.len() + src.len());
    path.push_str(prefix);
    path.push_str(src);
    archive.read(&path).ok()
}

/// Concrete [`AssetResolver`] used to feed ThorVG external assets stored in
/// the dotLottie archive. ThorVG passes the unprefixed asset name (the value
/// from the JSON's `p` / `fPath` field, with any `/f/` already stripped); we
/// try image first, then font.
struct DotLottieAssetResolver {
    archive: Arc<DotLottieArchive>,
    version: u8,
}

impl AssetResolver for DotLottieAssetResolver {
    fn resolve(&self, src: &str) -> Option<Cow<'_, [u8]>> {
        let src = src.strip_prefix("/f/").unwrap_or(src);
        if let Some(bytes) = archive_asset_bytes(&self.archive, self.version, AssetKind::Image, src)
        {
            return Some(bytes);
        }
        archive_asset_bytes(&self.archive, self.version, AssetKind::Font, src)
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
