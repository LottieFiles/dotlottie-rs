use super::{DotLottieError, Manifest};
use std::io::{self, Read};

use base64::{engine::general_purpose, Engine};
use serde_json::Value;
use zip::ZipArchive;

/// Extract a single animation with its image assets inlined.
///
/// bytes: The bytes of the dotLottie file
/// animation_id: The id of the animation to extract
/// Result<String, DotLottieError>: The extracted animation, or an error
/// Notes: This function uses jzon rather than serde as serde was exporting invalid JSON
pub fn get_animation(
    bytes: &Vec<u8>,
    animation_id: &str,
    version: u8,
) -> Result<String, DotLottieError> {
    let mut archive =
        ZipArchive::new(io::Cursor::new(bytes)).map_err(|_| DotLottieError::ArchiveOpenError)?;

    let json_path = if version == 2 {
        format!("a/{}.json", animation_id)
    } else {
        format!("animations/{}.json", animation_id)
    };

    let lot_path = if version == 2 {
        format!("a/{}.lot", animation_id)
    } else {
        format!("animations/{}.lot", animation_id)
    };

    let file_data = if let Ok(data) = read_zip_file(&mut archive, &json_path) {
        data
    } else if let Ok(data) = read_zip_file(&mut archive, &lot_path) {
        data
    } else {
        return Err(DotLottieError::FileFindError);
    };

    let animation_data =
        String::from_utf8(file_data).map_err(|_| DotLottieError::ReadContentError)?;

    let mut lottie_animation =
        jzon::parse(&animation_data).map_err(|_| DotLottieError::ReadContentError)?;

    if let Some(assets) = lottie_animation["assets"].as_array_mut() {
        for asset in assets {
            if let Some(p) = asset["p"].as_str() {
                if p.starts_with("data:image/") {
                    // if the asset is already inlined, force the embed flag to 1
                    asset["e"] = 1.into();
                } else {
                    let image_asset_filename: String = if version == 2 {
                        format!("i/{}", asset["p"].to_string().replace('"', ""))
                    } else {
                        format!("images/{}", asset["p"].to_string().replace('"', ""))
                    };

                    let image_ext = asset["p"]
                        .to_string()
                        .split('.')
                        .next_back()
                        .unwrap()
                        .to_string()
                        .replace('"', "");

                    let mut result = archive
                        .by_name(&image_asset_filename)
                        .map_err(|_| DotLottieError::FileFindError)?;

                    let mut content = Vec::new();

                    result
                        .read_to_end(&mut content)
                        .map_err(|_| DotLottieError::ReadContentError)?;

                    // Write the image data to the lottie
                    let image_data_base64 = general_purpose::STANDARD.encode(&content);

                    asset["u"] = "".into();
                    asset["p"] =
                        format!("data:image/{};base64,{}", image_ext, image_data_base64).into();
                    // explicitly indicate that the image asset is inlined
                    asset["e"] = 1.into();
                }
            }
        }
    }

    Ok(jzon::stringify(lottie_animation))
}

/// Get the manifest of a dotLottie file.
///
/// bytes: The bytes of the dotLottie file
/// Result<Manifest, DotLottieError>: The extracted manifest, or an error
pub fn get_manifest(bytes: &[u8]) -> Result<Manifest, DotLottieError> {
    let mut archive =
        ZipArchive::new(io::Cursor::new(bytes)).map_err(|_| DotLottieError::ArchiveOpenError)?;

    let mut result = archive
        .by_name("manifest.json")
        .map_err(|_| DotLottieError::FileFindError)?;

    let mut content = Vec::new();
    result
        .read_to_end(&mut content)
        .map_err(|_| DotLottieError::ReadContentError)?;

    let manifest_string = String::from_utf8_lossy(&content).to_string();
    let manifest: Manifest = serde_json::from_str(&manifest_string).unwrap();

    Ok(manifest)
}

/// Get the width and height of a dotLottie file.
pub fn get_width_height(animation_data: &str) -> (u32, u32) {
    let lottie_animation: Value = serde_json::from_str(animation_data).unwrap();

    let width = lottie_animation["w"].as_u64().unwrap() as u32;
    let height = lottie_animation["h"].as_u64().unwrap() as u32;

    (width, height)
}

pub fn get_theme(bytes: &[u8], theme_id: &str) -> Result<String, DotLottieError> {
    let mut archive =
        ZipArchive::new(io::Cursor::new(bytes)).map_err(|_| DotLottieError::ArchiveOpenError)?;
    let search_file_name = format!("t/{}.json", theme_id);

    let mut content = Vec::new();
    archive
        .by_name(&search_file_name)
        .map_err(|_| DotLottieError::FileFindError)?
        .read_to_end(&mut content)
        .map_err(|_| DotLottieError::ReadContentError)?;

    String::from_utf8(content).map_err(|_| DotLottieError::InvalidUtf8Error)
}

pub fn get_state_machine(bytes: &[u8], state_machine_id: &str) -> Result<String, DotLottieError> {
    let mut archive =
        ZipArchive::new(io::Cursor::new(bytes)).map_err(|_| DotLottieError::ArchiveOpenError)?;
    let search_file_name = format!("s/{}.json", state_machine_id);

    let mut content = Vec::new();
    archive
        .by_name(&search_file_name)
        .map_err(|_| DotLottieError::FileFindError)?
        .read_to_end(&mut content)
        .map_err(|_| DotLottieError::ReadContentError)?;

    String::from_utf8(content).map_err(|_| DotLottieError::InvalidUtf8Error)
}

// Helper function to read a file from a ZIP archive
fn read_zip_file(
    archive: &mut ZipArchive<io::Cursor<&Vec<u8>>>,
    path: &str,
) -> Result<Vec<u8>, DotLottieError> {
    let mut file = archive
        .by_name(path)
        .map_err(|_| DotLottieError::FileFindError)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|_| DotLottieError::ReadContentError)?;

    Ok(buf)
}
