mod errors;
mod manifest;

pub use errors::*;
pub use manifest::*;

#[cfg(feature = "theming")]
use crate::theme::Theme;
#[cfg(feature = "audio")]
use rustc_hash::FxHashMap;
use serde_json::Value;
use std::cell::RefCell;
use std::io::{self, Read};
#[cfg(feature = "audio")]
use std::sync::Arc;
use zip::ZipArchive;

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const DATA_IMAGE_PREFIX: &str = "data:image/";
const DATA_FONT_PREFIX: &str = "data:font/";
#[cfg(feature = "audio")]
const DATA_AUDIO_PREFIX: &str = "data:audio/";
const BASE64_PREFIX: &str = ";base64,";
const DEFAULT_EXT: &str = "png";
const DEFAULT_FONT_EXT: &str = "ttf";

pub struct DotLottieManager {
    active_animation_id: Box<str>,
    manifest: Manifest,
    version: u8,
    archive: RefCell<ZipArchive<io::Cursor<Vec<u8>>>>,
    /// Audio bytes keyed by packaged path (e.g. `u/clip.mp3`).
    #[cfg(feature = "audio")]
    audio_sources: RefCell<FxHashMap<String, Arc<[u8]>>>,
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
            #[cfg(feature = "audio")]
            audio_sources: RefCell::new(FxHashMap::default()),
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

        #[cfg(feature = "audio")]
        let mut audio_sources: FxHashMap<String, Arc<[u8]>> = FxHashMap::default();

        if let Some(assets) = lottie_animation
            .get_mut("assets")
            .and_then(|v| v.as_array_mut())
        {
            Self::embed_images(&mut archive, assets, self.version);
            #[cfg(feature = "audio")]
            {
                audio_sources = Self::collect_audio(&mut archive, assets, self.version);
            }
        }

        if self.version == 2 {
            if let Some(font_list) = lottie_animation
                .get_mut("fonts")
                .and_then(|v| v.as_object_mut())
                .and_then(|fonts| fonts.get_mut("list"))
                .and_then(|v| v.as_array_mut())
            {
                Self::embed_fonts(&mut archive, font_list);
            }
        }

        #[cfg(feature = "audio")]
        {
            *self.audio_sources.borrow_mut() = audio_sources;
        }

        serde_json::to_string(&lottie_animation).map_err(|_| DotLottieError::ReadContentError)
    }

    fn embed_images<R: Read + io::Seek>(
        archive: &mut ZipArchive<R>,
        assets: &mut [Value],
        version: u8,
    ) {
        let image_prefix = if version == 2 { "i/" } else { "images/" };
        let mut asset_path = String::with_capacity(128);

        for asset in assets.iter_mut() {
            let Some(asset_obj) = asset.as_object_mut() else {
                continue;
            };
            let Some(p_str) = asset_obj.get("p").and_then(|v| v.as_str()) else {
                continue;
            };

            if p_str.starts_with(DATA_IMAGE_PREFIX) {
                asset_obj.insert("e".to_string(), Value::Number(1.into()));
                continue;
            }

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
                    let data_url = format!(
                        "{DATA_IMAGE_PREFIX}{image_ext}{BASE64_PREFIX}{}",
                        Self::encode_base64(&content)
                    );
                    asset_obj.insert("u".to_string(), Value::String(String::new()));
                    asset_obj.insert("p".to_string(), Value::String(data_url));
                    asset_obj.insert("e".to_string(), Value::Number(1.into()));
                }
            }
        }
    }

    /// Read audio assets from the archive as raw bytes, keyed by packaged path
    /// (e.g. `u/clip.mp3`). Audio is not re-embedded in the JSON since ThorVG
    /// never renders it.
    #[cfg(feature = "audio")]
    fn collect_audio<R: Read + io::Seek>(
        archive: &mut ZipArchive<R>,
        assets: &[Value],
        version: u8,
    ) -> FxHashMap<String, Arc<[u8]>> {
        let audio_prefix = if version == 2 { "u/" } else { "audio/" };
        const SUPPORTED_AUDIO_EXTENSION: &str = "mp3";

        let mut out: FxHashMap<String, Arc<[u8]>> = FxHashMap::default();
        let mut asset_path = String::with_capacity(128);

        for asset in assets {
            let Some(asset_obj) = asset.as_object() else {
                continue;
            };
            let Some(p_str) = asset_obj.get("p").and_then(|v| v.as_str()) else {
                continue;
            };

            // Embedded data URL: ThorVG delivers the bytes via the resolver.
            if p_str.starts_with(DATA_AUDIO_PREFIX) {
                continue;
            }

            if !p_str.to_lowercase().ends_with(SUPPORTED_AUDIO_EXTENSION) {
                continue;
            }

            asset_path.clear();
            asset_path.push_str(audio_prefix);
            asset_path.push_str(p_str);

            if let Ok(mut result) = archive.by_name(&asset_path) {
                let mut content = Vec::with_capacity(result.size() as usize);
                if result.read_to_end(&mut content).is_ok() {
                    out.insert(asset_path.clone(), Arc::from(content));
                }
            }
        }

        out
    }

    fn embed_fonts<R: Read + io::Seek>(archive: &mut ZipArchive<R>, font_list: &mut [Value]) {
        let mut font_path = String::with_capacity(128);

        for font in font_list.iter_mut() {
            let Some(font_obj) = font.as_object_mut() else {
                continue;
            };
            // Clone to release the immutable borrow before mutating font_obj below.
            let Some(f_path_str) = font_obj
                .get("fPath")
                .and_then(|v| v.as_str())
                .map(str::to_string)
            else {
                continue;
            };

            // Only process package-internal fonts with the /f/ prefix.
            let Some(path_without_prefix) = f_path_str.strip_prefix("/f/") else {
                continue;
            };

            font_path.clear();
            font_path.push_str("f/");
            font_path.push_str(path_without_prefix);

            if let Ok(mut result) = archive.by_name(&font_path) {
                let mut content = Vec::with_capacity(result.size() as usize);
                if result.read_to_end(&mut content).is_ok() {
                    let font_ext = path_without_prefix
                        .rfind('.')
                        .map(|i| &path_without_prefix[i + 1..])
                        .unwrap_or(DEFAULT_FONT_EXT);
                    let data_url = format!(
                        "{DATA_FONT_PREFIX}{font_ext}{BASE64_PREFIX}{}",
                        Self::encode_base64(&content)
                    );
                    font_obj.insert("fPath".to_string(), Value::String(data_url));
                }
            }
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

    /// Read a packaged image by file name and return it as a `data:` URI. Any
    /// leading directory is ignored.
    pub fn get_image_data_url(&self, file_name: &str) -> Option<String> {
        let name = file_name.rsplit('/').next().unwrap_or(file_name);

        if name.is_empty() {
            return None;
        }

        let prefix = if self.version == 2 { "i/" } else { "images/" };
        let mut archive = self.archive.borrow_mut();
        let mut entry = archive.by_name(&format!("{prefix}{name}")).ok()?;

        let mut content = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut content).ok()?;

        let ext = name.rfind('.').map(|i| &name[i + 1..]).unwrap_or(DEFAULT_EXT);

        Some(format!(
            "{DATA_IMAGE_PREFIX}{ext}{BASE64_PREFIX}{}",
            Self::encode_base64(&content)
        ))
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

    #[cfg(feature = "audio")]
    pub fn get_audio_sources(&self) -> FxHashMap<String, Arc<[u8]>> {
        self.audio_sources.borrow().clone()
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

    // happy_birthday_audio.lottie ships 3 mp3s (223190, 95712, 9613 bytes).
    #[cfg(feature = "audio")]
    const AUDIO_LOTTIE: &[u8] =
        include_bytes!("../../assets/animations/dotlottie/v2/happy_birthday_audio.lottie");

    #[test]
    #[cfg(feature = "audio")]
    fn test_audio_assets_extracted_from_dotlottie() {
        let manager = DotLottieManager::new(AUDIO_LOTTIE).expect("create manager");
        let _json = manager.get_active_animation().expect("get animation");

        let sources = manager.get_audio_sources();

        assert_eq!(sources.len(), 3, "three decoded audio assets");
        let mut sizes: Vec<usize> = sources.values().map(|a| a.len()).collect();
        sizes.sort_unstable();
        assert_eq!(sizes, vec![9613, 95712, 223190], "audio bytes match zip");

        assert!(
            sources.keys().all(|k| k.starts_with("u/")),
            "sources keyed by packaged path"
        );
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
}
