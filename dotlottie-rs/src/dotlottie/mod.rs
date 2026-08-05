mod archive;
mod errors;
mod manifest;

pub use errors::*;
pub use manifest::*;

use archive::{Archive, ArchiveError};

#[cfg(feature = "theming")]
use crate::theme::Theme;
use std::borrow::Cow;
use std::sync::Arc;

pub struct DotLottieManager {
    active_animation_id: Box<str>,
    manifest: Manifest,
    version: u8,
    archive: Arc<Archive>,
}

impl DotLottieManager {
    pub fn new(dotlottie: &[u8]) -> Result<Self, Error> {
        let archive =
            Arc::new(Archive::new(dotlottie.to_vec()).map_err(|_| Error::ArchiveOpenError)?);

        let manifest = Self::read_file(&archive, "manifest.json")?;
        let manifest_str = std::str::from_utf8(&manifest).map_err(|_| Error::ReadContentError)?;
        let manifest = Manifest::from_json(manifest_str).ok_or(Error::ReadContentError)?;

        let id = manifest
            .initial
            .as_ref()
            .and_then(|initial| initial.animation.as_ref())
            .or_else(|| manifest.animations.first().map(|a| &a.id))
            .ok_or(Error::AnimationsNotFound)?
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
            archive,
        })
    }

    #[inline]
    pub fn get_active_animation(&self) -> Result<String, Error> {
        self.get_animation(&self.active_animation_id)
    }

    pub fn get_animation(&self, animation_id: &str) -> Result<String, Error> {
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

        let file_data = Self::read_file(&self.archive, &json_path)
            .or_else(|_| Self::read_file(&self.archive, &lot_path))?;

        String::from_utf8(file_data.into_owned()).map_err(|_| Error::ReadContentError)
    }

    #[inline]
    #[cfg(feature = "state-machines")]
    pub fn get_state_machine(&self, state_machine_id: &str) -> Result<String, Error> {
        let path = format!("s/{state_machine_id}.json");
        let content = Self::read_file(&self.archive, &path)?;
        String::from_utf8(content.into_owned()).map_err(|_| Error::InvalidUtf8Error)
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
    pub fn assets(&self) -> AssetStore {
        AssetStore {
            archive: Arc::clone(&self.archive),
            version: self.version,
        }
    }

    #[inline]
    #[cfg(feature = "theming")]
    pub fn get_theme(&self, theme_id: &str) -> Result<Theme, Error> {
        let path = format!("t/{theme_id}.json");
        let content = Self::read_file(&self.archive, &path)?;
        let theme_str = std::str::from_utf8(&content).map_err(|_| Error::InvalidUtf8Error)?;
        theme_str
            .parse::<Theme>()
            .map_err(|_| Error::ReadContentError)
    }

    #[inline]
    fn read_file<'a>(archive: &'a Archive, path: &str) -> Result<Cow<'a, [u8]>, Error> {
        archive.by_name(path).map_err(|e| match e {
            ArchiveError::NotFound => Error::FileFindError,
            _ => Error::ReadContentError,
        })
    }
}

/// Cheap cloneable handle for resolving container assets from resolver closures.
#[derive(Clone)]
pub struct AssetStore {
    archive: Arc<Archive>,
    version: u8,
}

impl AssetStore {
    /// Resolve a ThorVG resolver `src` to container bytes.
    /// Probes: exact key (slashes trimmed), conventional prefix + key,
    /// conventional prefix + basename.
    pub fn get(&self, src: &str) -> Option<Vec<u8>> {
        let key = src.trim_start_matches('/');
        if key.is_empty() {
            return None;
        }
        if let Ok(content) = self.archive.by_name(key) {
            return Some(content.into_owned());
        }
        // v2 containers spec images, fonts, and audio; v1 specs images only.
        let prefixes: &[&str] = if self.version == 2 {
            &["i/", "f/", "u/"]
        } else {
            &["images/"]
        };
        for prefix in prefixes {
            if let Ok(content) = self.archive.by_name(&format!("{prefix}{key}")) {
                return Some(content.into_owned());
            }
        }
        let base = key.rsplit('/').next().unwrap_or(key);
        if base != key {
            for prefix in prefixes {
                if let Ok(content) = self.archive.by_name(&format!("{prefix}{base}")) {
                    return Some(content.into_owned());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    const IMAGE_LOTTIE: &[u8] = include_bytes!("../../assets/animations/dotlottie/v2/image.lottie");
    const FONT_LOTTIE: &[u8] =
        include_bytes!("../../assets/animations/dotlottie/v2/elapsed_time.lottie");

    #[test]
    fn asset_store_resolves_exact_key_after_slash_trim() {
        let manager = DotLottieManager::new(IMAGE_LOTTIE).expect("create manager");
        let store = manager.assets();
        let bytes = store.get("//images/image_0.png").expect("exact probe");
        assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']), "PNG magic");
    }

    #[test]
    fn asset_store_resolves_prefix_and_basename_probes() {
        let manager = DotLottieManager::new(IMAGE_LOTTIE).expect("create manager");
        let store = manager.assets();
        assert!(store.get("image_0.png").is_some(), "prefix+key probe");
        assert!(
            store.get("/wrong/dir/image_0.png").is_some(),
            "prefix+basename probe"
        );
        assert!(store.get("missing.png").is_none());
    }

    #[test]
    fn asset_store_resolves_fonts_by_verbatim_fpath() {
        let manager = DotLottieManager::new(FONT_LOTTIE).expect("create manager");
        let store = manager.assets();
        assert!(store.get("f/Bebas-Regular.ttf").is_some());
        assert!(
            store.get("/f/Bebas-Regular.ttf").is_some(),
            "leading slash trimmed"
        );
    }

    #[test]
    fn asset_store_is_send_and_sync() {
        fn assert_sync<T: Sync>() {}
        fn assert_send<T: Send>() {}
        assert_sync::<AssetStore>();
        assert_send::<AssetStore>();
    }

    #[test]
    fn test_dotlottie_manager_creation() {
        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/assets/animations/dotlottie/v1/emojis.lottie"
        );

        let mut buffer = Vec::new();
        File::open(&file_path)
            .expect("fixture must exist")
            .read_to_end(&mut buffer)
            .expect("fixture must be readable");

        let manager = DotLottieManager::new(&buffer).expect("manager creation");
        assert!(!manager.active_animation_id().is_empty());
        assert!(!manager.manifest().animations.is_empty());
    }

    #[test]
    fn manager_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<DotLottieManager>();
    }

    // happy_birthday_audio.lottie ships 3 mp3s (223190, 95712, 9613 bytes).
    #[cfg(feature = "audio")]
    const AUDIO_LOTTIE: &[u8] =
        include_bytes!("../../assets/animations/dotlottie/v2/happy_birthday_audio.lottie");

    #[test]
    #[cfg(feature = "audio")]
    fn asset_store_resolves_audio_by_packaged_path() {
        let manager = DotLottieManager::new(AUDIO_LOTTIE).expect("create manager");
        let store = manager.assets();
        let json = manager.get_active_animation().expect("get animation");
        let parsed = crate::json::Value::parse(&json).expect("valid JSON");
        let sample = match parsed.get("assets") {
            Some(crate::json::Value::Array(items)) => items
                .iter()
                .filter_map(|i| i.str_field("p"))
                .find(|p| p.ends_with(".mp3"))
                .map(str::to_owned),
            _ => None,
        }
        .expect("audio asset present");
        // src shape ThorVG produces for u:"/u/" p:"<name>.mp3"
        let bytes = store.get(&format!("//u/{sample}")).expect("audio resolves");
        assert!(!bytes.is_empty());
    }

    #[test]
    #[cfg(feature = "audio")]
    fn test_dotlottie_json_excludes_audio_payload() {
        let manager = DotLottieManager::new(AUDIO_LOTTIE).expect("create manager");
        let json = manager.get_active_animation().expect("get animation");

        assert!(
            !json.contains("data:audio/"),
            "rendered JSON should not embed base64 audio"
        );
    }

    #[test]
    fn animation_json_returned_verbatim() {
        let manager = DotLottieManager::new(IMAGE_LOTTIE).expect("create manager");
        let json = manager.get_active_animation().expect("get animation");
        assert!(!json.contains("data:image/"), "no inlined images");
        assert!(
            json.contains("\"p\":\"image_0.png\""),
            "original asset path intact"
        );
    }
}
