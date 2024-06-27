use libdeflater::{DecompressionError, Decompressor};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::from_utf8;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ManifestTheme {
    pub id: String,
    pub animations: Vec<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManifestAnimation {
    pub autoplay: Option<bool>,
    pub defaultTheme: Option<String>,
    pub direction: Option<i8>,
    pub hover: Option<bool>,
    pub id: String,
    pub intermission: Option<u32>,
    pub r#loop: Option<bool>,
    pub loop_count: Option<u32>,
    pub playMode: Option<String>,
    pub speed: Option<f32>,
    pub themeColor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub active_animation_id: Option<String>,
    pub animations: Vec<ManifestAnimation>,
    pub author: Option<String>,
    // pub custom: Option<record(string(), an>))>
    pub description: Option<String>,
    pub generator: Option<String>,
    pub keywords: Option<String>,
    pub revision: Option<u32>,
    pub themes: Option<Vec<ManifestTheme>>,
    pub states: Option<Vec<String>>,
    pub version: Option<String>,
}

#[derive(Debug)]
pub enum DotLottieLoaderError {
    ArchiveOpenError,
    FileFindError { file_name: String },
    AnimationNotFound { animation_id: String },
    AnimationsNotFound,
    ManifestNotFound,
    InvalidUtf8Error,
    InvalidDotLottieFile,
    DecompressionError(DecompressionError),
    Utf8Error(std::str::Utf8Error),
    FromBytesError(std::array::TryFromSliceError),
    DataUnavailable,
}

impl fmt::Display for DotLottieLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DotLottieLoaderError::ArchiveOpenError => write!(f, "Failed to open archive"),
            DotLottieLoaderError::FileFindError { file_name } => {
                write!(f, "Unable to find the file: {}", file_name)
            }
            DotLottieLoaderError::AnimationNotFound { animation_id } => {
                write!(f, "Animation not found: {}", animation_id)
            }
            DotLottieLoaderError::AnimationsNotFound => {
                write!(f, "No animations found in dotLottie file")
            }
            DotLottieLoaderError::ManifestNotFound => write!(f, "No manifest found"),
            DotLottieLoaderError::InvalidUtf8Error => write!(f, "Invalid UTF-8"),
            DotLottieLoaderError::InvalidDotLottieFile => write!(f, "Invalid dotLottie file"),
            DotLottieLoaderError::DecompressionError(err) => {
                write!(f, "Decompression error: {}", err)
            }
            DotLottieLoaderError::Utf8Error(err) => write!(f, "UTF-8 error: {}", err),
            // DotLottieLoaderError::IOError(err) => write!(f, "IO error: {}", err),
            DotLottieLoaderError::FromBytesError(err) => write!(f, "From bytes error: {}", err),
            DotLottieLoaderError::DataUnavailable => write!(f, "Data unavailable"),
        }
    }
}

impl std::error::Error for DotLottieLoaderError {}

impl From<DecompressionError> for DotLottieLoaderError {
    fn from(err: DecompressionError) -> Self {
        DotLottieLoaderError::DecompressionError(err)
    }
}

impl From<std::str::Utf8Error> for DotLottieLoaderError {
    fn from(err: std::str::Utf8Error) -> Self {
        DotLottieLoaderError::Utf8Error(err)
    }
}

impl From<std::array::TryFromSliceError> for DotLottieLoaderError {
    fn from(err: std::array::TryFromSliceError) -> Self {
        DotLottieLoaderError::FromBytesError(err)
    }
}

fn base64_encode(plain: &[u8]) -> String {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    plain
        .chunks(3)
        .flat_map(|chunk| {
            let (b1, b2, b3) = match *chunk {
                [b1, b2, b3] => (b1, b2, b3),
                [b1, b2] => (b1, b2, 0),
                [b1] => (b1, 0, 0),
                _ => (0, 0, 0),
            };
            [
                BASE64_CHARS[(b1 >> 2) as usize],
                BASE64_CHARS[((b1 & 0x03) << 4 | (b2 >> 4)) as usize],
                if chunk.len() > 1 {
                    BASE64_CHARS[((b2 & 0x0f) << 2 | (b3 >> 6)) as usize]
                } else {
                    b'='
                },
                if chunk.len() == 3 {
                    BASE64_CHARS[(b3 & 0x3f) as usize]
                } else {
                    b'='
                },
            ]
        })
        .map(|b| b as char)
        .collect()
}

fn inflate(data: &[u8], uncompressed_len: usize) -> Result<Vec<u8>, DotLottieLoaderError> {
    let mut decompressor = Decompressor::new();
    let mut output = vec![0; uncompressed_len];

    decompressor.deflate_decompress(data, &mut output)?;

    Ok(output)
}

#[derive(Debug)]
struct LazyFile {
    name: String,

    compressed_data: Option<Vec<u8>>,

    decompressed_data: Option<String>,
    decompressed_data_len: usize,
}

impl LazyFile {
    pub fn get_or_decompress_data(&mut self) -> Result<&str, DotLottieLoaderError> {
        if self.decompressed_data.is_none() {
            if let Some(compressed) = self.compressed_data.take() {
                // Optionally take to clear memory
                let decompressed_bytes = inflate(&compressed, self.decompressed_data_len)?;
                let decompressed_str = from_utf8(&decompressed_bytes)
                    .map_err(|_| DotLottieLoaderError::InvalidUtf8Error)?
                    .to_owned();
                self.decompressed_data = Some(decompressed_str);
            } else {
                // Handle the case where decompression isn't possible due to missing data.
                return Err(DotLottieLoaderError::DataUnavailable);
            }
        }

        Ok(self.decompressed_data.as_deref().unwrap())
    }
}

pub struct DotLottieLoader {
    active_animation_id: String,
    manifest: Option<Manifest>,
    animations: Vec<LazyFile>,
    themes: Vec<LazyFile>,
    images: Vec<LazyFile>,
    states: Vec<LazyFile>,
}

impl DotLottieLoader {
    pub fn new() -> Self {
        Self {
            active_animation_id: String::new(),
            manifest: None,
            animations: Vec::new(),
            themes: Vec::new(),
            images: Vec::new(),
            states: Vec::new(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<DotLottieLoader, DotLottieLoaderError> {
        let mut file = Self::new();
        file.read(bytes)?;

        Ok(file)
    }

    pub fn set_active_animation_id(&mut self, active_animation_id: &str) {
        self.active_animation_id = active_animation_id.to_string();
    }

    #[inline]
    fn read(&mut self, bytes: &[u8]) -> Result<(), DotLottieLoaderError> {
        let eocd_offset = bytes
            .len()
            .checked_sub(22)
            .ok_or(DotLottieLoaderError::InvalidDotLottieFile)?;

        let eocd = &bytes[eocd_offset..];
        if eocd[0..4] != [0x50, 0x4B, 0x05, 0x06] {
            return Err(DotLottieLoaderError::InvalidDotLottieFile);
        }

        let offset = u32::from_le_bytes(eocd[16..20].try_into()?) as usize;
        let num_central_dir = u16::from_le_bytes(eocd[10..12].try_into()?);

        self.process_central_directory(bytes, offset, num_central_dir)
    }

    #[inline]
    fn process_central_directory(
        &mut self,
        bytes: &[u8],
        offset: usize,
        num_central_dir: u16,
    ) -> Result<(), DotLottieLoaderError> {
        let mut central_dir_offset = offset;
        for _ in 0..num_central_dir {
            let central_dir_entry = &bytes[central_dir_offset..];

            // Check if the central directory entry signature is correct
            if central_dir_entry[0..4] != [0x50, 0x4B, 0x01, 0x02] {
                return Err(DotLottieLoaderError::InvalidDotLottieFile);
            }

            let header_offset = u32::from_le_bytes(central_dir_entry[42..46].try_into()?) as usize;
            self.process_file_header(bytes, header_offset)?;

            let name_len = u16::from_le_bytes(central_dir_entry[28..30].try_into()?) as usize;
            let extra_len = u16::from_le_bytes(central_dir_entry[30..32].try_into()?) as usize;
            let comment_len = u16::from_le_bytes(central_dir_entry[32..34].try_into()?) as usize;
            central_dir_offset += 46 + name_len + extra_len + comment_len;
        }
        Ok(())
    }

    #[inline]
    fn process_file_header(
        &mut self,
        bytes: &[u8],
        offset: usize,
    ) -> Result<(), DotLottieLoaderError> {
        let header = &bytes[offset..];

        if header[0..4] != [0x50, 0x4B, 0x03, 0x04] {
            return Err(DotLottieLoaderError::InvalidDotLottieFile);
        }

        let name_len = u16::from_le_bytes(header[26..28].try_into()?) as usize;

        let compression_method = u16::from_le_bytes(header[8..10].try_into()?);
        let compressed = compression_method != 0;
        let data_offset = offset + 30 + name_len;
        let data_len = if compressed {
            u32::from_le_bytes(header[18..22].try_into()?) as usize
        } else {
            u32::from_le_bytes(header[22..26].try_into()?) as usize
        } as usize;

        let uncompressed_len = u32::from_le_bytes(header[22..26].try_into()?) as usize;

        // panic!("");

        // Correctly handle the Result from String::from_utf8 before comparing
        let file_name_result = String::from_utf8(header[30..30 + name_len].to_vec())
            .map_err(|_| DotLottieLoaderError::InvalidUtf8Error);
        let file_data = bytes[data_offset..data_offset + data_len].to_vec();

        // Now use a match or if let to work with the Result
        if let Ok(file_name) = file_name_result {
            if file_name == "manifest.json" {
                let manifest_data = if compressed {
                    inflate(&file_data, uncompressed_len)?
                } else {
                    file_data
                };

                let manifest_str = from_utf8(&manifest_data)
                    .map_err(|_| DotLottieLoaderError::InvalidUtf8Error)?;

                let manifest = serde_json::from_str::<Manifest>(manifest_str);

                self.manifest =
                    Some(manifest.map_err(|_| DotLottieLoaderError::InvalidDotLottieFile)?);
            } else if file_name.starts_with("animations/") && file_name.ends_with(".json") {
                if compressed {
                    self.animations.push(LazyFile {
                        name: file_name.replace("animations/", "").replace(".json", ""),
                        compressed_data: Some(file_data),
                        decompressed_data: None,
                        decompressed_data_len: uncompressed_len,
                    });
                } else {
                    let animation_str = from_utf8(&file_data)
                        .map_err(|_| DotLottieLoaderError::InvalidUtf8Error)?
                        .to_string();

                    self.animations.push(LazyFile {
                        name: file_name.replace("animations/", "").replace(".json", ""),
                        compressed_data: None,
                        decompressed_data: Some(animation_str),
                        decompressed_data_len: uncompressed_len,
                    });
                }
            } else if file_name.starts_with("themes/") && file_name.ends_with(".json") {
                if compressed {
                    self.themes.push(LazyFile {
                        name: file_name.replace("themes/", "").replace(".json", ""),
                        compressed_data: Some(file_data),
                        decompressed_data: None,
                        decompressed_data_len: uncompressed_len,
                    });
                } else {
                    let theme_str = from_utf8(&file_data)
                        .map_err(|_| DotLottieLoaderError::InvalidUtf8Error)?
                        .to_string();
                    self.themes.push(LazyFile {
                        name: file_name.replace("themes/", "").replace(".json", ""),
                        compressed_data: None,
                        decompressed_data: Some(theme_str),
                        decompressed_data_len: uncompressed_len,
                    });
                }
            } else if file_name.starts_with("images/") {
                if compressed {
                    self.images.push(LazyFile {
                        name: file_name.replace("images/", ""),
                        compressed_data: Some(file_data),
                        decompressed_data: None,
                        decompressed_data_len: uncompressed_len,
                    });
                } else {
                    self.images.push(LazyFile {
                        name: file_name.replace("images/", ""),
                        compressed_data: None,
                        // base64 encoded image data
                        decompressed_data: Some(base64_encode(&file_data)),
                        decompressed_data_len: uncompressed_len,
                    });
                }
            } else if file_name.starts_with("states/") && file_name.ends_with(".json") {
                if compressed {
                    self.states.push(LazyFile {
                        name: file_name.replace("states/", "").replace(".json", ""),
                        compressed_data: Some(file_data),
                        decompressed_data: None,
                        decompressed_data_len: uncompressed_len,
                    });
                } else {
                    let state_machine_str = from_utf8(&file_data)
                        .map_err(|_| DotLottieLoaderError::InvalidUtf8Error)?
                        .to_string();
                    self.states.push(LazyFile {
                        name: file_name.replace("states/", "").replace(".json", ""),
                        compressed_data: None,
                        decompressed_data: Some(state_machine_str),
                        decompressed_data_len: uncompressed_len,
                    });
                }
            }
        } else {
            // Handle or propagate the error as appropriate
            return Err(DotLottieLoaderError::InvalidUtf8Error);
        }

        Ok(())
    }

    pub fn manifest(&self) -> Option<Manifest> {
        self.manifest.clone()
    }

    pub fn active_animation_id(&self) -> String {
        self.active_animation_id.clone()
    }

    pub fn get_animation(&mut self, animation_id: &str) -> Result<String, DotLottieLoaderError> {
        let animation = self
            .animations
            .iter_mut()
            .find(|animation| animation.name == animation_id)
            .ok_or(DotLottieLoaderError::AnimationNotFound {
                animation_id: animation_id.to_string(),
            })?;

        Ok(animation.get_or_decompress_data()?.to_string())
    }

    pub fn get_theme(&mut self, theme_id: &str) -> Result<String, DotLottieLoaderError> {
        let theme = self
            .themes
            .iter_mut()
            .find(|theme| theme.name == theme_id)
            .ok_or(DotLottieLoaderError::AnimationNotFound {
                animation_id: theme_id.to_string(),
            })?;

        Ok(theme.get_or_decompress_data()?.to_string())
    }

    pub fn get_state_machine(
        &mut self,
        state_machine_id: &str,
    ) -> Result<String, DotLottieLoaderError> {
        let state_machine = self
            .states
            .iter_mut()
            .find(|state_machine| state_machine.name == state_machine_id)
            .ok_or(DotLottieLoaderError::AnimationNotFound {
                animation_id: state_machine_id.to_string(),
            })?;

        Ok(state_machine.get_or_decompress_data()?.to_string())
    }
}
