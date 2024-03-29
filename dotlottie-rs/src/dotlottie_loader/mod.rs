use libdeflater::Decompressor;
use std::str::from_utf8;

use crate::{utils::base64_encode, DotLottiePlayerError, Manifest};

fn inflate(data: &[u8], uncompressed_len: usize) -> Result<Vec<u8>, DotLottiePlayerError> {
    let mut decompressor = Decompressor::new();
    let mut output = vec![0; uncompressed_len];
    decompressor.deflate_decompress(data, &mut output)?;
    Ok(output)
}

struct LazyFile {
    name: String,

    compressed_data: Option<Vec<u8>>,

    decompressed_data: Option<String>,
    decompressed_data_len: usize,
}

impl LazyFile {
    pub fn get_or_decompress_data(&mut self) -> Result<&str, DotLottiePlayerError> {
        if self.decompressed_data.is_none() {
            if let Some(compressed) = self.compressed_data.take() {
                // Optionally take to clear memory
                let decompressed_bytes = inflate(&compressed, self.decompressed_data_len)?;
                let decompressed_str = from_utf8(&decompressed_bytes)
                    .map_err(|_| DotLottiePlayerError::InvalidUtf8Error)?
                    .to_owned();
                self.decompressed_data = Some(decompressed_str);
            } else {
                // Handle the case where decompression isn't possible due to missing data.
                return Err(DotLottiePlayerError::DataUnavailable);
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
}

impl DotLottieLoader {
    fn new() -> Self {
        Self {
            active_animation_id: String::new(),
            manifest: None,
            animations: Vec::new(),
            themes: Vec::new(),
            images: Vec::new(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<DotLottieLoader, DotLottiePlayerError> {
        let mut file = Self::new();
        file.read(bytes)?;

        Ok(file)
    }

    pub fn set_active_animation_id(&mut self, active_animation_id: &str) {
        self.active_animation_id = active_animation_id.to_string();
    }

    #[inline]
    fn read(&mut self, bytes: &[u8]) -> Result<(), DotLottiePlayerError> {
        let eocd_offset = bytes
            .len()
            .checked_sub(22)
            .ok_or(DotLottiePlayerError::InvalidDotLottieFile)?;
        let eocd = &bytes[eocd_offset..];
        if eocd[0..4] != [0x50, 0x4B, 0x05, 0x06] {
            return Err(DotLottiePlayerError::InvalidDotLottieFile);
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
    ) -> Result<(), DotLottiePlayerError> {
        let mut central_dir_offset = offset;
        for _ in 0..num_central_dir {
            let central_dir_entry = &bytes[central_dir_offset..];

            // Check if the central directory entry signature is correct
            if central_dir_entry[0..4] != [0x50, 0x4B, 0x01, 0x02] {
                return Err(DotLottiePlayerError::InvalidDotLottieFile);
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
    ) -> Result<(), DotLottiePlayerError> {
        let header = &bytes[offset..];

        if header[0..4] != [0x50, 0x4B, 0x03, 0x04] {
            return Err(DotLottiePlayerError::InvalidDotLottieFile);
        }

        let name_len = u16::from_le_bytes(header[26..28].try_into()?) as usize;

        let compression_method = u16::from_le_bytes(header[8..10].try_into()?);
        let compressed = compression_method != 0;
        let data_offset = offset + 30 + name_len;
        let data_len = if compressed {
            u32::from_le_bytes(header[18..22].try_into()?)
        } else {
            u32::from_le_bytes(header[22..26].try_into()?)
        } as usize;

        let uncompressed_len = u32::from_le_bytes(header[14..18].try_into()?) as usize;

        // Correctly handle the Result from String::from_utf8 before comparing
        let file_name_result = String::from_utf8(header[30..30 + name_len].to_vec())
            .map_err(|_| DotLottiePlayerError::InvalidUtf8Error);
        let file_data = bytes[data_offset..data_offset + data_len].to_vec();

        // Now use a match or if let to work with the Result
        if let Ok(file_name) = file_name_result {
            if file_name == "manifest.json" {
                let manifest_data = if compressed {
                    inflate(&file_data, uncompressed_len)?
                } else {
                    file_data
                };

                let manifest_str =
                    from_utf8(&manifest_data).map_err(|_| DotLottiePlayerError::InvalidUtf8Error)?;

                // let manifest_json = jzon::parse(manifest_str)
                //     .map_err(|_| DotLottiePlayerError::InvalidDotLottieFile)?;

                // self.manifest = Some(Manifest::from_json(&manifest_json));

                let manifest: Result<(Manifest, _), serde_json_core::de::Error> =
                    serde_json_core::de::from_str(manifest_str);
                self.manifest = Some(
                    manifest
                        .map_err(|_| DotLottiePlayerError::InvalidDotLottieFile)?
                        .0,
                );
            } else if file_name.starts_with("animations/") && file_name.ends_with(".json") {
                if compressed {
                    self.animations.push(LazyFile {
                        name: file_name.replace("animations/", "").replace(".json", ""),
                        compressed_data: Some(file_data),
                        decompressed_data: None,
                        decompressed_data_len: 0,
                    });
                } else {
                    let animation_str = from_utf8(&file_data)
                        .map_err(|_| DotLottiePlayerError::InvalidUtf8Error)?
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
                        decompressed_data_len: 0,
                    });
                } else {
                    let theme_str = from_utf8(&file_data)
                        .map_err(|_| DotLottiePlayerError::InvalidUtf8Error)?
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
                        decompressed_data_len: 0,
                    });
                } else {
                    let image_str = from_utf8(&file_data)
                        .map_err(|_| DotLottiePlayerError::InvalidUtf8Error)?
                        .to_string();
                    self.images.push(LazyFile {
                        name: file_name.replace("images/", ""),
                        compressed_data: None,
                        // base64 encoded image data
                        decompressed_data: Some(base64_encode(&file_data)),
                        decompressed_data_len: 0,
                    });
                }
            }
        } else {
            // Handle or propagate the error as appropriate
            return Err(DotLottiePlayerError::InvalidUtf8Error);
        }

        Ok(())
    }

    pub fn manifest(&self) -> Option<&Manifest> {
        self.manifest.as_ref()
    }

    pub fn active_animation_id(&self) -> &str {
        &self.active_animation_id
    }

    pub fn get_animation(&mut self, animation_id: &str) -> Result<&str, DotLottiePlayerError> {
        let animation = self
            .animations
            .iter_mut()
            .find(|animation| animation.name == animation_id)
            .ok_or(DotLottiePlayerError::AnimationNotFound {
                animation_id: animation_id.to_string(),
            })?;

        animation.get_or_decompress_data()
    }

    pub fn get_theme(&mut self, theme_id: &str) -> Result<&str, DotLottiePlayerError> {
        let theme = self
            .themes
            .iter_mut()
            .find(|theme| theme.name == theme_id)
            .ok_or(DotLottiePlayerError::AnimationNotFound {
                animation_id: theme_id.to_string(),
            })?;

        theme.get_or_decompress_data()
    }
}
