#[cfg(test)]
mod tests {
    #[test]
    fn get_animation_test() {
        use std::{fs::File, io::Read};

        use crate::get_animation;

        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/src/fms/tests/resources/emoji-collection.lottie"
        );

        let mut animation_file = File::open(file_path).unwrap();
        let mut buffer = Vec::new();

        animation_file.read_to_end(&mut buffer).unwrap();

        let animation = get_animation(&buffer, "anger").unwrap();

        assert_eq!(animation.contains("ADBE Vector Graphic - Stroke"), true);
    }

    #[test]
    fn get_animations_test() {
        use std::{fs::File, io::Read};

        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/src/fms/tests/resources/emoji-collection.lottie"
        );

        let mut animation_file = File::open(file_path).unwrap();
        let mut buffer = Vec::new();

        animation_file.read_to_end(&mut buffer).unwrap();

        let animation = crate::get_animations(&buffer).unwrap();

        assert_eq!(animation.len(), 62);

        assert_eq!(animation[0].id, "anger");
        assert_eq!(animation[5].id, "confused");
    }

    #[test]
    fn get_manifest_test() {
        use std::{fs::File, io::Read};

        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/src/fms/tests/resources/emoji-collection.lottie"
        );

        let mut animation_file = File::open(file_path).unwrap();
        let mut buffer = Vec::new();

        animation_file.read_to_end(&mut buffer).unwrap();

        let manifest = crate::get_manifest(&buffer).unwrap();

        // First and last animations
        let first_animation_lock = manifest.animations;

        let first_animation = first_animation_lock.first().unwrap();

        assert_eq!(first_animation.id == "anger", true);

        let last_animation = first_animation_lock.last().unwrap();

        assert_eq!(last_animation.id == "yummy", true);
    }

    #[test]
    fn get_animation_with_image_assets_test() {
        let dotlottie_bytes = &include_bytes!("../resources/bull.lottie").to_vec();
        let animation_name = "animation_1";

        let lottie_string = crate::get_animation(&dotlottie_bytes, animation_name)
            .expect("Failed to get animation from lottie bytes");

        let lottie_json =
            jzon::parse(&lottie_string).expect("Failed to parse lottie string to JSON");

        let assets = lottie_json["assets"]
            .as_array()
            .expect("Expected assets to be an array");

        for asset in assets {
            assert_eq!(
                asset["e"]
                    .as_i64()
                    .expect("Expected embed property to be an integer"),
                1,
                "Expected asset to be embedded with 'e' set to 1"
            );

            assert!(
                asset["u"]
                    .as_str()
                    .expect("Expected asset URL ('u') to be a string")
                    .is_empty(),
                "Expected asset URL ('u') to be empty"
            );

            assert!(
                asset["p"]
                    .as_str()
                    .expect("Expected asset path ('p') to be a string")
                    .starts_with("data:image/"),
                "Expected asset path ('p') to be a data URL starting with 'data:image/'"
            );
        }
    }
}
