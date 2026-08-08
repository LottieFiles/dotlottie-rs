use dotlottie_rs::{player::Error as PlayerError, ColorSpace, Player, Status};
use std::ffi::CString;

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_load_valid_theme() {
        let mut player = Player::new();
        player.set_autoplay(true);

        let valid_theme_id = CString::new("test_theme").expect("Failed to create CString");
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert_eq!(
            player.set_theme(&valid_theme_id),
            Err(PlayerError::InsufficientCondition),
            "Expected theme to not load"
        );

        assert_eq!(
            player.load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v2/test.lottie"
            )),
            Ok(())
        );
        assert!(player.theme_id().is_none());

        assert_eq!(
            player.set_theme(&valid_theme_id),
            Ok(()),
            "Expected theme to load"
        );
        assert_eq!(player.theme_id(), Some(valid_theme_id.as_c_str()));

        assert_eq!(player.status(), Status::Playing);
    }

    #[test]
    fn test_load_invalid_theme() {
        let mut player = Player::new();
        player.set_autoplay(true);

        let invalid_theme_id = CString::new("invalid_theme").expect("Failed to create CString");
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert_eq!(
            player.set_theme(&invalid_theme_id),
            Err(PlayerError::InsufficientCondition),
            "Expected theme to not load"
        );

        assert_eq!(
            player.load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v2/test.lottie"
            )),
            Ok(())
        );

        assert_ne!(
            player.set_theme(&invalid_theme_id),
            Ok(()),
            "Expected theme to not load"
        );

        assert_eq!(player.status(), Status::Playing);
    }

    #[test]
    fn test_unset_theme() {
        let mut player = Player::new();
        player.set_autoplay(true);

        let theme_id = CString::new("test_theme").expect("Failed to create CString");

        assert_eq!(
            player.load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v2/test.lottie"
            )),
            Ok(())
        );

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v2/test.lottie"
            ))
            .is_ok());

        assert_eq!(
            player.set_theme(&theme_id),
            Ok(()),
            "Expected theme to load"
        );
        assert_eq!(player.reset_theme(), Ok(()), "Expected theme to unload");
    }

    #[test]
    fn test_unset_theme_before_load() {
        let mut player = Player::new();
        player.set_autoplay(true);

        assert_eq!(
            player.load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v2/test.lottie"
            )),
            Ok(())
        );

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v2/test.lottie"
            ))
            .is_ok());

        assert_eq!(player.reset_theme(), Ok(()), "Expected theme to unload");
    }

    #[test]
    fn test_clear_active_theme_id_after_new_animation_data_is_loaded() {
        let mut player = Player::new();
        player.set_autoplay(true);

        let valid_theme_id = CString::new("test_theme").expect("Failed to create CString");
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert_eq!(
            player.set_theme(&valid_theme_id),
            Err(PlayerError::InsufficientCondition),
            "Expected theme to not load"
        );

        assert_eq!(
            player.load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v2/test.lottie"
            )),
            Ok(())
        );

        assert_eq!(
            player.set_theme(&valid_theme_id),
            Ok(()),
            "Expected theme to load"
        );
        assert_eq!(player.theme_id(), Some(valid_theme_id.as_c_str()));

        let data_str = std::str::from_utf8(include_bytes!("../assets/animations/lottie/test.json"))
            .expect("Invalid data.");
        let data = CString::new(data_str).expect("Failed to create CString");
        assert_eq!(player.load_animation_data(&data), Ok(()));
        assert!(player.theme_id().is_none());

        assert_eq!(player.status(), Status::Playing);
    }

    #[test]
    fn test_clear_active_theme_id_after_new_animation_path_is_loaded() {
        let mut player = Player::new();
        player.set_autoplay(true);

        let valid_theme_id = CString::new("test_theme").expect("Failed to create CString");
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert_eq!(
            player.set_theme(&valid_theme_id),
            Err(PlayerError::InsufficientCondition),
            "Expected theme to not load"
        );

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v2/test.lottie"
            ))
            .is_ok(),);

        assert!(
            player.set_theme(&valid_theme_id).is_ok(),
            "Expected theme to load"
        );
        assert_eq!(player.theme_id(), Some(valid_theme_id.as_c_str()));

        let path =
            CString::new("assets/animations/lottie/test.json").expect("Failed to create CString");
        assert_eq!(player.load_animation_path(&path), Ok(()));
        assert!(player.theme_id().is_none());

        assert_eq!(player.status(), Status::Playing);
    }

    #[test]
    fn test_clear_active_theme_id_after_new_dotlottie_is_loaded() {
        let mut player = Player::new();
        player.set_autoplay(true);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let valid_theme_id = CString::new("test_theme").expect("Failed to create CString");

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v2/test.lottie"
            ))
            .is_ok());
        assert!(player.theme_id().is_none());

        assert!(
            player.set_theme(&valid_theme_id).is_ok(),
            "Expected theme to load"
        );
        assert_eq!(player.theme_id(), Some(valid_theme_id.as_c_str()));

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/emojis.lottie"
            ))
            .is_ok());
        assert!(player.theme_id().is_none());

        assert_eq!(player.status(), Status::Playing);
    }

    #[test]
    fn test_theme_persists_after_load_animation() {
        let mut player = Player::new();
        player.set_autoplay(true);

        let theme_id = CString::new("red").expect("Failed to create CString");
        let second_anim = CString::new("rect").expect("Failed to create CString");
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        // Load a .lottie with two animations (circle, rect) and two themes (red, yellow)
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v2/multi_anim_theme.lottie"
            ))
            .is_ok());

        assert_eq!(
            player.set_theme(&theme_id),
            Ok(()),
            "Expected theme to load"
        );
        assert_eq!(player.theme_id(), Some(theme_id.as_c_str()));

        // Switch to a different animation within the same .lottie — theme should persist
        assert_eq!(player.load_animation(&second_anim), Ok(()));
        assert_eq!(
            player.theme_id(),
            Some(theme_id.as_c_str()),
            "Theme should persist after load_animation within the same .lottie container"
        );

        assert_eq!(player.status(), Status::Playing);
    }

    #[test]
    fn test_reset_theme_restores_slots_to_defaults() {
        use dotlottie_rs::ColorSlot;

        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .is_ok());

        let data = CString::new(include_str!("../assets/animations/lottie/bouncy_ball.json"))
            .expect("Failed to create CString");
        assert_eq!(player.load_animation_data(&data), Ok(()));

        let mut original_ids = player.get_slot_ids();
        original_ids.sort();
        assert!(
            !original_ids.is_empty(),
            "animation should expose its own slots after load"
        );
        let default_color = player.get_slot_str("ball_color");

        player
            .set_color_slot("ball_color", ColorSlot::new([1.0, 0.0, 0.0]))
            .unwrap();
        assert_ne!(
            player.get_slot_str("ball_color"),
            default_color,
            "override should change the slot value"
        );

        assert_eq!(player.reset_theme(), Ok(()));

        let mut ids_after_reset = player.get_slot_ids();
        ids_after_reset.sort();
        assert_eq!(
            ids_after_reset, original_ids,
            "reset_theme must keep the animation's slots, not clear them"
        );
        assert_eq!(
            player.get_slot_str("ball_color"),
            default_color,
            "reset_theme must restore the slot to its initial value"
        );
    }

    // ── BezierPath rule (experimental) ────────────────────────────

    const BEZIER_PATH_THEME: &str = r#"{
        "rules": [
            {
                "id": "shape_path",
                "type": "BezierPath",
                "value": {
                    "vertices": [[-80, -80], [80, -80], [80, 80], [-80, 80]],
                    "closed": true
                }
            }
        ]
    }"#;

    fn load_bezier_path_player() -> Player {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .unwrap();

        let data = CString::new(include_str!("../assets/animations/lottie/bezier_path.json"))
            .expect("Failed to create CString");
        assert_eq!(player.load_animation_data(&data), Ok(()));
        player
    }

    #[test]
    fn test_bezier_path_rule_transforms_to_slot() {
        let slots = dotlottie_rs::transform_theme_to_lottie_slots(BEZIER_PATH_THEME, "");
        assert!(
            slots.contains(r#""shape_path":{"p":{"a":0,"k":{"#),
            "theme should emit a static path slot, got: {slots}"
        );
        assert!(
            slots.contains(r#""i":[[0,0],[0,0],[0,0],[0,0]]"#),
            "omitted tangents should default to zero, got: {slots}"
        );
        assert!(slots.contains(r#""v":[[-80,-80],[80,-80],[80,80],[-80,80]]"#));
        assert!(slots.contains(r#""c":true"#));
    }

    #[test]
    fn test_bezier_path_rule_applies_to_animation() {
        let mut player = load_bezier_path_player();
        let default_path = player.get_slot_str("shape_path");

        let theme = CString::new(BEZIER_PATH_THEME).unwrap();
        assert_eq!(player.set_theme_data(&theme), Ok(()));

        let themed = player.get_slot_str("shape_path");
        assert_ne!(default_path, themed, "theme should override the path slot");
        assert_eq!(player.get_slot_type("shape_path"), "bezier_path");

        assert_eq!(player.reset_theme(), Ok(()));
        assert_eq!(player.get_slot_str("shape_path"), default_path);
    }

    #[test]
    fn test_bezier_path_rule_animated() {
        let theme = r#"{
            "rules": [
                {
                    "id": "shape_path",
                    "type": "BezierPath",
                    "keyframes": [
                        {
                            "frame": 0,
                            "value": {"vertices": [[-50, -50], [50, -50], [0, 50]], "closed": true},
                            "inTangent": {"x": 0.833, "y": 0.833},
                            "outTangent": {"x": 0.167, "y": 0.167}
                        },
                        {
                            "frame": 60,
                            "value": {"vertices": [[-90, -90], [90, -90], [0, 90]], "closed": true}
                        }
                    ]
                }
            ]
        }"#;

        let mut player = load_bezier_path_player();
        assert_eq!(
            player.set_theme_data(&CString::new(theme).unwrap()),
            Ok(()),
            "an animated path rule should apply"
        );

        let json = player.get_slot_str("shape_path");
        assert!(
            json.starts_with(r#"{"a":1,"k":[{"#),
            "keyframed rule should emit an animated property, got: {json}"
        );
        assert!(json.contains(r#""t":60"#));
    }

    #[test]
    fn test_bezier_path_rule_empty_keyframes_falls_back_to_value() {
        let theme = r#"{
            "rules": [
                {
                    "id": "shape_path",
                    "type": "BezierPath",
                    "keyframes": [],
                    "value": {"vertices": [[0, 0], [10, 0], [0, 10]], "closed": true}
                }
            ]
        }"#;
        let slots = dotlottie_rs::transform_theme_to_lottie_slots(theme, "");
        assert!(
            slots.contains(r#""shape_path":{"p":{"a":0,"k":{"#),
            "empty keyframes must fall back to the static value, got: {slots}"
        );
    }

    #[test]
    fn test_bezier_path_rule_without_value_is_dropped() {
        let theme = r#"{"rules":[{"id":"shape_path","type":"BezierPath"}]}"#;
        let slots = dotlottie_rs::transform_theme_to_lottie_slots(theme, "");
        assert_eq!(slots, "{}");
    }

    #[test]
    fn test_bezier_path_rule_rejects_mismatched_tangents() {
        let theme = r#"{
            "rules": [
                {
                    "id": "shape_path",
                    "type": "BezierPath",
                    "value": {
                        "vertices": [[0, 0], [10, 0], [0, 10]],
                        "inTangents": [[0, 0]]
                    }
                }
            ]
        }"#;
        assert_eq!(
            dotlottie_rs::transform_theme_to_lottie_slots(theme, ""),
            "",
            "a tangent count that doesn't match the vertices is an invalid theme"
        );
    }
}
