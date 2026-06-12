mod errors;
mod manifest;

pub use errors::*;
pub use manifest::*;

#[cfg(feature = "audio")]
use crate::audio::{extract_audio, AudioLayer};
#[cfg(feature = "theming")]
use crate::theme::Theme;
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
    #[cfg(feature = "encryption")]
    password: Option<zeroize::Zeroizing<Vec<u8>>>,
    #[cfg(feature = "audio")]
    audio_data: RefCell<Option<(Vec<Arc<[u8]>>, Vec<AudioLayer>)>>,
}

impl DotLottieManager {
    pub fn new(dotlottie: &[u8]) -> Result<Self, DotLottieError> {
        Self::with_password(dotlottie, None)
    }

    pub fn with_password(dotlottie: &[u8], password: Option<&str>) -> Result<Self, DotLottieError> {
        let mut archive = ZipArchive::new(io::Cursor::new(dotlottie.to_vec()))
            .map_err(|_| DotLottieError::ArchiveOpenError)?;

        // Without the `encryption` feature a supplied password cannot be used;
        // encrypted archives then fail at the manifest read below.
        #[cfg(feature = "encryption")]
        let password_bytes = password.map(str::as_bytes);
        #[cfg(not(feature = "encryption"))]
        let password_bytes: Option<&[u8]> = {
            let _ = password;
            None
        };

        let manifest = Self::read_zip_file(&mut archive, "manifest.json", password_bytes)?;
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
            #[cfg(feature = "encryption")]
            password: password.map(|p| zeroize::Zeroizing::new(p.as_bytes().to_vec())),
            #[cfg(feature = "audio")]
            audio_data: RefCell::new(None),
        })
    }

    #[cfg(feature = "encryption")]
    #[inline]
    fn password_bytes(&self) -> Option<&[u8]> {
        self.password.as_ref().map(|p| p.as_slice())
    }

    #[cfg(not(feature = "encryption"))]
    #[inline]
    fn password_bytes(&self) -> Option<&[u8]> {
        None
    }

    #[inline]
    pub fn get_active_animation(&self) -> Result<String, DotLottieError> {
        self.get_animation(&self.active_animation_id)
    }

    pub fn get_animation(&self, animation_id: &str) -> Result<String, DotLottieError> {
        let mut archive = self.archive.borrow_mut();
        let password = self.password_bytes();

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

        let file_data = Self::read_zip_file(&mut archive, &json_path, password)
            .or_else(|_| Self::read_zip_file(&mut archive, &lot_path, password))?;

        let animation_data =
            std::str::from_utf8(&file_data).map_err(|_| DotLottieError::ReadContentError)?;

        let mut lottie_animation: Value =
            serde_json::from_str(animation_data).map_err(|_| DotLottieError::ReadContentError)?;

        if let Some(assets) = lottie_animation
            .get_mut("assets")
            .and_then(|v| v.as_array_mut())
        {
            Self::embed_images(&mut archive, assets, self.version, password);
            #[cfg(feature = "audio")]
            Self::embed_audio(&mut archive, assets, self.version, password);
        }

        if self.version == 2 {
            if let Some(font_list) = lottie_animation
                .get_mut("fonts")
                .and_then(|v| v.as_object_mut())
                .and_then(|fonts| fonts.get_mut("list"))
                .and_then(|v| v.as_array_mut())
            {
                Self::embed_fonts(&mut archive, font_list, password);
            }
        }

        #[cfg(feature = "audio")]
        {
            *self.audio_data.borrow_mut() = Some(extract_audio(&lottie_animation));
        }

        serde_json::to_string(&lottie_animation).map_err(|_| DotLottieError::ReadContentError)
    }

    fn embed_images<R: Read + io::Seek>(
        archive: &mut ZipArchive<R>,
        assets: &mut [Value],
        version: u8,
        password: Option<&[u8]>,
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

            if let Ok(content) = Self::read_zip_file(archive, &asset_path, password) {
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

    #[cfg(feature = "audio")]
    fn embed_audio<R: Read + io::Seek>(
        archive: &mut ZipArchive<R>,
        assets: &mut Vec<Value>,
        version: u8,
        password: Option<&[u8]>,
    ) {
        let audio_prefix = if version == 2 { "u/" } else { "audio/" };
        const SUPPORTED_AUDIO_EXTENSION: &'static str = "mp3";

        for asset in assets.iter_mut() {
            let Some(asset_obj) = asset.as_object_mut() else {
                continue;
            };
            let Some(p_str) = asset_obj
                .get("p")
                .and_then(|v| v.as_str())
                .map(str::to_string)
            else {
                continue;
            };

            if p_str.starts_with(DATA_AUDIO_PREFIX) {
                asset_obj.insert("e".to_string(), Value::Number(1.into()));
                continue;
            }

            let p_lower = p_str.to_lowercase();
            if !p_lower.ends_with(SUPPORTED_AUDIO_EXTENSION) {
                continue;
            }

            let asset_path = String::from(format!("{}{}", audio_prefix, p_str));
            if let Ok(content) = Self::read_zip_file(archive, &asset_path, password) {
                let data_url = format!(
                    "{DATA_AUDIO_PREFIX}{SUPPORTED_AUDIO_EXTENSION}{BASE64_PREFIX}{}",
                    Self::encode_base64(&content)
                );
                asset_obj.insert("u".to_string(), Value::String(String::new()));
                asset_obj.insert("p".to_string(), Value::String(data_url));
                asset_obj.insert("e".to_string(), Value::Number(1.into()));
            }
        }
    }

    fn embed_fonts<R: Read + io::Seek>(
        archive: &mut ZipArchive<R>,
        font_list: &mut [Value],
        password: Option<&[u8]>,
    ) {
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

            if let Ok(content) = Self::read_zip_file(archive, &font_path, password) {
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

    #[inline]
    #[cfg(feature = "state-machines")]
    pub fn get_state_machine(&self, state_machine_id: &str) -> Result<String, DotLottieError> {
        let mut archive = self.archive.borrow_mut();
        let path = format!("s/{state_machine_id}.json");
        let content = Self::read_zip_file(&mut archive, &path, self.password_bytes())?;
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
        let content = Self::read_zip_file(&mut archive, &path, self.password_bytes())?;
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

    fn map_zip_error(err: zip::result::ZipError) -> DotLottieError {
        use zip::result::ZipError;
        match err {
            ZipError::InvalidPassword => DotLottieError::InvalidPassword,
            ZipError::UnsupportedArchive(msg) if msg == ZipError::PASSWORD_REQUIRED => {
                DotLottieError::EncryptedArchive
            }
            // Without `encryption`, AES entries surface a different
            // UnsupportedArchive message; on the dotLottie read path the only
            // other reachable UnsupportedArchive is encryption-related.
            #[cfg(not(feature = "encryption"))]
            ZipError::UnsupportedArchive(_) => DotLottieError::EncryptedArchive,
            ZipError::FileNotFound => DotLottieError::FileFindError,
            _ => DotLottieError::ReadContentError,
        }
    }

    #[inline]
    fn read_zip_file<R: Read + io::Seek>(
        archive: &mut ZipArchive<R>,
        path: &str,
        password: Option<&[u8]>,
    ) -> Result<Vec<u8>, DotLottieError> {
        let mut file = match password {
            Some(pw) => archive.by_name_decrypt(path, pw),
            None => archive.by_name(path),
        }
        .map_err(Self::map_zip_error)?;

        let mut buf = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buf)
            .map_err(|_| DotLottieError::ReadContentError)?;

        Ok(buf)
    }

    #[cfg(feature = "audio")]
    pub fn get_audio_assets(&self) -> Option<(Vec<Arc<[u8]>>, Vec<AudioLayer>)> {
        self.audio_data.borrow().clone()
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

    #[cfg(feature = "encryption")]
    mod encryption {
        use super::super::*;
        use std::io;

        const PASSWORD: &str = "s3cr3t-pa55";

        fn read_fixture(rel: &str) -> Vec<u8> {
            let path = format!(
                "{}/assets/animations/dotlottie/v2/{}",
                env!("CARGO_MANIFEST_DIR"),
                rel
            );
            std::fs::read(&path).unwrap_or_else(|_| panic!("fixture {rel} should exist"))
        }

        /// Re-zip every entry of a plaintext .lottie with WinZip AES-256.
        fn encrypt_dotlottie(plaintext: &[u8], password: &str) -> Vec<u8> {
            use zip::write::SimpleFileOptions;
            use zip::{AesMode, CompressionMethod, ZipArchive, ZipWriter};

            let mut src =
                ZipArchive::new(io::Cursor::new(plaintext.to_vec())).expect("open source");
            let mut out = ZipWriter::new(io::Cursor::new(Vec::new()));
            for i in 0..src.len() {
                let mut entry = src.by_index(i).expect("entry by index");
                let name = entry.name().to_string();
                let options = SimpleFileOptions::default()
                    .compression_method(CompressionMethod::Deflated)
                    .with_aes_encryption(AesMode::Aes256, password);
                out.start_file(name, options).expect("start_file");
                io::copy(&mut entry, &mut out).expect("copy entry");
            }
            out.finish().expect("finish").into_inner()
        }

        // v1 (image.lottie: `images/` embedding) and v2 (elapsed_time.lottie:
        // `a/` animation JSON) must both decrypt to the exact same bytes as a
        // plaintext load. (`s/` state-machine and `t/` theme decrypt paths are
        // covered by the dedicated tests below.)
        #[test]
        fn correct_password_matches_plaintext() {
            for fixture in ["image.lottie", "elapsed_time.lottie"] {
                let plaintext = read_fixture(fixture);
                let encrypted = encrypt_dotlottie(&plaintext, PASSWORD);

                let plain = DotLottieManager::new(&plaintext).expect("plaintext loads");
                let enc = DotLottieManager::with_password(&encrypted, Some(PASSWORD))
                    .unwrap_or_else(|e| panic!("{fixture} should load with password: {e:?}"));

                assert_eq!(
                    enc.get_active_animation().unwrap(),
                    plain.get_active_animation().unwrap(),
                    "{fixture}: decrypted animation must match plaintext byte-for-byte"
                );
            }
        }

        // Exercise the remaining decrypt read paths: `s/` state machines and
        // `t/` themes (both go through the same password-aware reader).
        #[cfg(feature = "state-machines")]
        #[test]
        fn decrypts_state_machine_path() {
            let encrypted = encrypt_dotlottie(&read_fixture("elapsed_time.lottie"), PASSWORD);
            let mgr = DotLottieManager::with_password(&encrypted, Some(PASSWORD)).unwrap();
            assert!(
                mgr.get_state_machine("long_press").is_ok(),
                "encrypted s/ path should decrypt"
            );
        }

        #[cfg(feature = "theming")]
        #[test]
        fn decrypts_theme_path() {
            let encrypted = encrypt_dotlottie(&read_fixture("multi_themes.lottie"), PASSWORD);
            let mgr = DotLottieManager::with_password(&encrypted, Some(PASSWORD)).unwrap();
            assert!(
                mgr.get_theme("dark").is_ok(),
                "encrypted t/ path should decrypt"
            );
        }

        #[test]
        fn wrong_password_is_invalid_password() {
            let encrypted = encrypt_dotlottie(&read_fixture("image.lottie"), PASSWORD);
            // Match on the Result directly: `unwrap_err` would require the Ok
            // type (DotLottieManager) to be Debug, which it is not.
            assert!(
                matches!(
                    DotLottieManager::with_password(&encrypted, Some("nope")),
                    Err(DotLottieError::InvalidPassword)
                ),
                "wrong password should yield InvalidPassword"
            );
        }

        #[test]
        fn missing_password_is_encrypted_archive() {
            let encrypted = encrypt_dotlottie(&read_fixture("image.lottie"), PASSWORD);
            assert!(
                matches!(
                    DotLottieManager::new(&encrypted),
                    Err(DotLottieError::EncryptedArchive)
                ),
                "missing password should report EncryptedArchive"
            );
        }

        #[test]
        fn password_on_plaintext_still_loads() {
            let plaintext = read_fixture("image.lottie");
            let mgr = DotLottieManager::with_password(&plaintext, Some(PASSWORD))
                .expect("plaintext loads even with a password supplied");
            assert!(!mgr.active_animation_id().is_empty());
        }

        // Real-world smoke test against the committed AES-256 fixture, which was
        // produced externally (password "test").
        #[test]
        fn shipped_password_lottie_fixture() {
            let bytes = read_fixture("password.lottie");

            let mgr = DotLottieManager::with_password(&bytes, Some("test"))
                .expect("password.lottie should load with password \"test\"");
            assert!(mgr.get_active_animation().is_ok());

            assert!(
                matches!(
                    DotLottieManager::with_password(&bytes, Some("wrong")),
                    Err(DotLottieError::InvalidPassword)
                ),
                "wrong password should be rejected"
            );
            assert!(
                matches!(
                    DotLottieManager::new(&bytes),
                    Err(DotLottieError::EncryptedArchive)
                ),
                "missing password should report EncryptedArchive"
            );
        }
    }
}
