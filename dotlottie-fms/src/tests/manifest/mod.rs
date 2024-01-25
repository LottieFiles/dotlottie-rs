#[cfg(test)]
use crate::{Manifest, ManifestAnimation};

#[test]
fn manifest_has_correct_default_values() {
    let manifest = Manifest::new();

    // Test your code here
    assert_eq!(manifest.author, Some("LottieFiles".to_string()));
    assert_eq!(manifest.generator, Some("dotLottie-utils".to_string()));
    assert_eq!(manifest.keywords, Some("dotLottie".to_string()));
    assert_eq!(manifest.revision, Some(1));
    assert_eq!(manifest.version, Some("1.0.0".to_string()));
}

#[test]
fn display() {
    use json::object;
    use std::sync::RwLock;

    let mut animations: Vec<ManifestAnimation> = Vec::new();
    let mut multi_animations: Vec<ManifestAnimation> = Vec::new();

    let animation_01 = ManifestAnimation::new(
        Some(true),
        Some("blue_theme".to_string()),
        Some(1),
        None,
        "animation_01".to_string(),
        None,
        Some(true),
        None,
        Some("normal".to_string()),
        None,
        None,
    );

    let animation_02 = ManifestAnimation::new(
        Some(false),
        Some("red_theme".to_string()),
        Some(-1),
        None,
        "animation_02".to_string(),
        None,
        Some(false),
        Some(12),
        Some("bounce".to_string()),
        Some(2),
        None,
    );

    let animation_03 = ManifestAnimation::new(
        Some(true),
        Some("orange_theme".to_string()),
        Some(1),
        None,
        "animation_02".to_string(),
        None,
        Some(true),
        Some(12),
        None,
        None,
        None,
    );

    animations.push(animation_01);

    let mut manifest = Manifest::new();
    manifest.active_animation_id = Some("default_animation_id".to_string());
    manifest.author = Some("test_author".to_string());
    manifest.animations = animations;

    let dis = manifest.to_json();

    let dis_expected = object! {
          "activeAnimationId": "default_animation_id",
          "animations": [
            {
              "autoplay": true,
              "defaultTheme": "blue_theme",
              "direction": 1,
              "hover": false,
              "id": "animation_01",
              "intermission": 0,
              "loop": true,
              "loopCount": 0,
              "playMode": "normal",
              "speed": 1,
              "themeColor": ""
            }
          ],
          "author": "test_author",
          "generator": "dotLottie-utils",
          "keywords": "dotLottie",
          "revision": 1,
          "version": "1.0.0",
    };
    assert_eq!(dis.dump(), dis_expected.dump());

    multi_animations.push(animation_02);
    multi_animations.push(animation_03);

    let mut manifest_with_two_animations = Manifest::new();
    manifest_with_two_animations.active_animation_id = Some("default_animation_id".to_string());
    manifest_with_two_animations.animations = multi_animations;
    manifest_with_two_animations.author = Some("test_author".to_string());
    manifest_with_two_animations.description = Some("Multi animation".to_string());
    manifest_with_two_animations.generator = Some("dotLottie-utils".to_string());
    manifest_with_two_animations.revision = Some(2);

    let dis_02 = manifest_with_two_animations.to_json();

    let dis_02_expected = object! {
      "activeAnimationId": "default_animation_id",
      "animations": [
        {
          "autoplay": false,
          "defaultTheme": "red_theme",
          "direction": -1,
          "hover": false,
          "id": "animation_02",
          "intermission": 0,
          "loop": false,
          "loopCount": 12,
          "playMode": "bounce",
          "speed": 2,
          "themeColor": ""
        },
        {
          "autoplay": true,
          "defaultTheme": "orange_theme",
          "direction": 1,
          "hover": false,
          "id": "animation_02",
          "intermission": 0,
          "loop": true,
          "loopCount": 12,
          "playMode": "Normal",
          "speed": 1,
          "themeColor": ""
        }
      ],
      "author": "test_author",
      "description": "Multi animation",
      "generator": "dotLottie-utils",
      "keywords": "dotLottie",
      "revision": 2,
      "version": "1.0.0"
    };

    assert_eq!(dis_02.dump(), dis_02_expected.dump());
}
