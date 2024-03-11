use crate::{errors::*, AnimationContainer, Manifest};
use std::io::{self, Read};
use std::path::Path;

use base64::{engine::general_purpose, Engine};
use serde_json::Value;
use zip::ZipArchive;

/// Extract a single animation with its image assets inlined.
///
/// bytes: The bytes of the dotLottie file
/// animation_id: The id of the animation to extract
/// Result<String, DotLottieError>: The extracted animation, or an error
/// Notes: This function uses jzon rather than serde as serde was exporting invalid JSON
pub fn get_animation(bytes: &Vec<u8>, animation_id: &str) -> Result<String, DotLottieError> {
    let mut archive =
        ZipArchive::new(io::Cursor::new(bytes)).map_err(|_| DotLottieError::ArchiveOpenError)?;

    let search_file_name = format!("animations/{}.json", animation_id);

    let mut result =
        archive
            .by_name(&search_file_name)
            .map_err(|_| DotLottieError::FileFindError {
                file_name: search_file_name,
            })?;

    let mut content = Vec::new();

    result
        .read_to_end(&mut content)
        .map_err(|_| DotLottieError::ReadContentError)?;

    // We can drop result so that we can use archive later, everything has been read in to content variable
    drop(result);

    let animation_data = String::from_utf8(content).unwrap();

    // Untyped JSON value
    let mut lottie_animation = jzon::parse(&animation_data).unwrap();

    // Loop through the parsed lottie animation and check for image assets
    if let Some(assets) = lottie_animation["assets"].as_array_mut() {
        for i in 0..assets.len() {
            if !assets[i]["p"].is_null() {
                let image_asset_filename =
                    format!("images/{}", assets[i]["p"].to_string().replace("\"", ""));

                let image_ext = assets[i]["p"]
                    .to_string()
                    .split(".")
                    .last()
                    .unwrap()
                    .to_string()
                    .replace("\"", "");

                let mut result = archive.by_name(&image_asset_filename).map_err(|_| {
                    DotLottieError::FileFindError {
                        file_name: image_asset_filename,
                    }
                })?;

                let mut content = Vec::new();

                result
                    .read_to_end(&mut content)
                    .map_err(|_| DotLottieError::ReadContentError)?;

                // Write the image data to the lottie
                let image_data_base64 = general_purpose::STANDARD.encode(&content);

                assets[i]["u"] = "".into();
                assets[i]["p"] =
                    format!("data:image/{};base64,{}", image_ext, image_data_base64).into();
            }
        }
    }

    Ok(jzon::stringify(lottie_animation))
}

/// Extract every animation with its image assets inlined.
///
/// bytes: The bytes of the dotLottie file
/// Result<Vec<AnimationData>, DotLottieError>: The extracted animations, or an error
pub fn get_animations(bytes: &Vec<u8>) -> Result<Vec<AnimationContainer>, DotLottieError> {
    let mut archive =
        ZipArchive::new(io::Cursor::new(bytes)).map_err(|_| DotLottieError::ArchiveOpenError)?;
    let mut file_contents = Vec::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();

        if (*file.name()).starts_with("animations/") && (*file.name()).ends_with(".json") {
            // Create a Path from the file path string
            let path = Path::new(file.name());

            // Get the file stem (file name without extension)
            if let Some(file_stem) = path.file_stem() {
                if let Some(file_stem_str) = file_stem.to_str() {
                    let animation = get_animation(bytes, file_stem_str).unwrap();

                    let item = AnimationContainer {
                        id: file_stem_str.to_string(),
                        animation_data: animation,
                    };

                    file_contents.push(item);
                }
            } else {
                // Handle the case where the path has no file stem
                return Err(DotLottieError::ReadContentError);
            }
        }
    }

    Ok(file_contents)
}

/// Get the manifest of a dotLottie file.
///
/// bytes: The bytes of the dotLottie file
/// Result<Manifest, DotLottieError>: The extracted manifest, or an error
pub fn get_manifest(bytes: &Vec<u8>) -> Result<Manifest, DotLottieError> {
    let mut archive =
        ZipArchive::new(io::Cursor::new(bytes)).map_err(|_| DotLottieError::ArchiveOpenError)?;

    let mut result =
        archive
            .by_name("manifest.json")
            .map_err(|_| DotLottieError::FileFindError {
                file_name: String::from("manifest.json"),
            })?;

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
    let search_file_name = format!("themes/{}.json", theme_id);

    let mut content = Vec::new();
    archive
        .by_name(&search_file_name)
        .map_err(|_| DotLottieError::FileFindError {
            file_name: search_file_name,
        })?
        .read_to_end(&mut content)
        .map_err(|_| DotLottieError::ReadContentError)?;

    String::from_utf8(content).map_err(|_| DotLottieError::InvalidUtf8Error)
}
