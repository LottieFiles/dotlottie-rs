#[cfg(test)]
#[test]
fn get_animation_test() {
    use std::{fs::File, io::Read};

    use crate::get_animation;

    let file_path = format!(
        "{}{}",
        env!("CARGO_MANIFEST_DIR"),
        "/src/tests/resources/emoji-collection.lottie"
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
        "/src/tests/resources/emoji-collection.lottie"
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
        "/src/tests/resources/emoji-collection.lottie"
    );

    let mut animation_file = File::open(file_path).unwrap();
    let mut buffer = Vec::new();

    animation_file.read_to_end(&mut buffer).unwrap();

    let manifest = crate::get_manifest(&buffer).unwrap();

    // First and last animations
    let first_animation_lock = manifest.animations.read().unwrap();

    let first_animation = first_animation_lock.first().unwrap();

    assert_eq!(first_animation.id == "anger", true);

    let last_animation = first_animation_lock.last().unwrap();

    assert_eq!(last_animation.id == "yummy", true);
}
