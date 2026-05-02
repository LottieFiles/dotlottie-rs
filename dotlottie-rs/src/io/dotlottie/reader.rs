use super::error::ReaderError;
use super::manifest::Manifest;
use super::zip::{Archive, ZipError};
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

/// dotLottie archive layout version, derived from `manifest.json`'s `version`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DotLottieVersion {
    V1,
    V2,
}

impl DotLottieVersion {
    fn from_manifest(version: Option<&str>) -> Self {
        match version {
            Some("2") => Self::V2,
            _ => Self::V1,
        }
    }

    fn animation_dir(self) -> &'static str {
        match self {
            Self::V1 => "animations",
            Self::V2 => "a",
        }
    }

    /// Animation directory prefix as bytes, including the trailing `/`
    /// separator — for prefix-matching against archive entry names.
    fn animation_prefix_bytes(self) -> &'static [u8] {
        match self {
            Self::V1 => b"animations/",
            Self::V2 => b"a/",
        }
    }
}

pub struct Reader {
    initial_animation_id: Box<str>,
    manifest: Manifest,
    version: DotLottieVersion,
    archive: Arc<Archive>,
}

impl Reader {
    pub fn new(dotlottie: Vec<u8>) -> Result<Self, ReaderError> {
        let archive = Archive::new(dotlottie).map_err(ReaderError::Zip)?;

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

        let version = DotLottieVersion::from_manifest(manifest.version.as_deref());

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
        Box::new(ArchiveAssetResolver {
            archive: Arc::clone(&self.archive),
            version: self.version,
        })
    }

    #[inline]
    pub fn version(&self) -> DotLottieVersion {
        self.version
    }

    /// Returns `true` when this archive contains a single animation and no
    /// themes or state machines. In that case the archive can be released
    /// after the renderer's `load_data` has eagerly resolved any external
    /// assets — none of the reader-backed APIs (theme, state machine, or
    /// switching to a different animation) could ever succeed afterwards.
    #[inline]
    pub fn is_single_shot(&self) -> bool {
        self.archive
            .is_single_shot(self.version.animation_prefix_bytes())
    }

    #[inline]
    pub fn initial_animation(&self) -> Result<Cow<'_, [u8]>, ReaderError> {
        self.animation(&self.initial_animation_id)
    }

    /// Returns the raw, unmodified animation JSON bytes for `animation_id`.
    ///
    /// External asset references (images, fonts, audio) are intentionally not
    /// embedded; pair this with [`Reader::asset_bytes`] (or ThorVG's asset
    /// resolver via `Player`) to materialise assets on demand.
    ///
    /// The returned `Cow` borrows from the archive when the entry was stored
    /// uncompressed and owns a fresh `Vec` when it was deflated.
    pub fn animation(&self, animation_id: &str) -> Result<Cow<'_, [u8]>, ReaderError> {
        let dir = self.version.animation_dir();
        let json_path = format!("{dir}/{animation_id}.json");

        self.archive
            .read(&json_path)
            .or_else(|e| match e {
                ZipError::FileNotFound => {
                    let lot_path = format!("{dir}/{animation_id}.lot");
                    self.archive.read(&lot_path)
                }
                other => Err(other),
            })
            .map_err(|e| match e {
                ZipError::FileNotFound => ReaderError::AnimationNotFound,
                other => ReaderError::Zip(other),
            })
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
    pub fn state_machine(&self, state_machine_id: &str) -> Result<Cow<'_, str>, ReaderError> {
        let path = format!("s/{state_machine_id}.json");
        let content = self.archive.read(&path).map_err(|e| match e {
            ZipError::FileNotFound => ReaderError::FileNotFound,
            other => ReaderError::Zip(other),
        })?;
        match content {
            Cow::Borrowed(bytes) => Ok(Cow::Borrowed(std::str::from_utf8(bytes)?)),
            Cow::Owned(vec) => String::from_utf8(vec)
                .map(Cow::Owned)
                .map_err(|e| ReaderError::InvalidUtf8(e.utf8_error())),
        }
    }

    #[inline]
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    #[inline]
    pub fn initial_animation_id(&self) -> &str {
        &self.initial_animation_id
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

/// On-disk prefix for an `(version, kind)` pair, or `None` if the combination
/// is not representable (e.g. v1 archives have no fonts).
fn asset_prefix(version: DotLottieVersion, kind: AssetKind) -> Option<&'static str> {
    match (version, kind) {
        (DotLottieVersion::V2, AssetKind::Image) => Some("i/"),
        (DotLottieVersion::V1, AssetKind::Image) => Some("images/"),
        (DotLottieVersion::V2, AssetKind::Audio) => Some("u/"),
        (DotLottieVersion::V1, AssetKind::Audio) => Some("audio/"),
        (DotLottieVersion::V2, AssetKind::Font) => Some("f/"),
        (DotLottieVersion::V1, AssetKind::Font) => None,
    }
}

fn archive_asset_bytes<'a>(
    archive: &'a Archive,
    version: DotLottieVersion,
    kind: AssetKind,
    src: &str,
) -> Option<Cow<'a, [u8]>> {
    let prefix = asset_prefix(version, kind)?;
    archive.read_concat(prefix, src).ok()
}

/// Concrete [`AssetResolver`] used to feed ThorVG external assets stored in
/// the dotLottie archive. ThorVG passes the unprefixed asset name (the value
/// from the JSON's `p` / `fPath` field, with any `/f/` already stripped); we
/// try image first, then font.
struct ArchiveAssetResolver {
    archive: Arc<Archive>,
    version: DotLottieVersion,
}

impl AssetResolver for ArchiveAssetResolver {
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
                let reader = Reader::new(buffer);
                assert!(reader.is_ok());

                if let Ok(mgr) = reader {
                    assert!(!mgr.initial_animation_id().is_empty());
                    assert!(!mgr.manifest().animations.is_empty());
                }
            }
        }
    }

    // ── is_single_shot ──────────────────────────────────────────────────

    #[test]
    fn is_single_shot_with_only_one_animation_and_image_asset_v2() {
        // image.lottie: 1 animation + 1 image, no themes, no state machines.
        // External assets don't disqualify — they're resolved during load_data
        // and the archive isn't needed afterwards.
        let bytes = include_bytes!("../../../assets/animations/dotlottie/v2/image.lottie").to_vec();
        let reader = Reader::new(bytes).expect("Reader::new failed");
        assert!(
            reader.is_single_shot(),
            "image.lottie should be single-shot"
        );
    }

    #[test]
    fn is_single_shot_with_themes_v2() {
        // themed.lottie: 1 animation + 3 themes — themes mean the user might
        // call apply_theme later, so the archive must stay alive.
        let bytes =
            include_bytes!("../../../assets/animations/dotlottie/v2/themed.lottie").to_vec();
        let reader = Reader::new(bytes).expect("Reader::new failed");
        assert!(
            !reader.is_single_shot(),
            "themed.lottie has themes; archive must be retained"
        );
    }

    #[test]
    fn is_single_shot_with_state_machine_v2() {
        // pigeon.lottie: 1 animation + 1 state machine.
        let bytes =
            include_bytes!("../../../assets/animations/dotlottie/v2/pigeon.lottie").to_vec();
        let reader = Reader::new(bytes).expect("Reader::new failed");
        assert!(
            !reader.is_single_shot(),
            "pigeon.lottie has a state machine; archive must be retained"
        );
    }

    #[test]
    fn is_single_shot_with_multiple_animations_v2() {
        // multi_anim_theme.lottie: 2 animations + 2 themes.
        let bytes =
            include_bytes!("../../../assets/animations/dotlottie/v2/multi_anim_theme.lottie")
                .to_vec();
        let reader = Reader::new(bytes).expect("Reader::new failed");
        assert!(
            !reader.is_single_shot(),
            "multi_anim_theme.lottie has 2 animations; archive must be retained"
        );
    }

    #[test]
    fn is_single_shot_with_many_animations_v1() {
        // emojis.lottie: many animations under v1 layout (`animations/`).
        let bytes =
            include_bytes!("../../../assets/animations/dotlottie/v1/emojis.lottie").to_vec();
        let reader = Reader::new(bytes).expect("Reader::new failed");
        assert_eq!(reader.version(), DotLottieVersion::V1);
        assert!(
            !reader.is_single_shot(),
            "emojis.lottie has many animations; archive must be retained"
        );
    }
}
