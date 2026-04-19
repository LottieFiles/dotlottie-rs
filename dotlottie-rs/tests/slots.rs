use dotlottie_rs::{ColorSpace, Player};
use std::ffi::CString;

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use super::*;

    fn load_bouncy_ball() -> Player {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .unwrap();

        let data = include_str!("../assets/animations/lottie/bouncy_ball.json");
        let c_data = CString::new(data).unwrap();
        player.load_animation_data(&c_data).unwrap();
        player
    }

    #[test]
    fn test_get_slot_ids_after_load() {
        let player = load_bouncy_ball();

        // bouncy_ball.json has 4 slots: ball_opacity, ball_color, ball_position, ball_scale
        let ids = player.get_slot_ids();
        assert!(
            !ids.is_empty(),
            "get_slot_ids should return slot IDs from animation"
        );

        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(
            sorted,
            vec!["ball_color", "ball_opacity", "ball_position", "ball_scale"]
        );
    }

    #[test]
    fn test_get_slot_type() {
        let player = load_bouncy_ball();

        assert_eq!(player.get_slot_type("ball_color"), "color");
        assert_eq!(player.get_slot_type("ball_opacity"), "scalar");
        assert_eq!(player.get_slot_type("nonexistent"), "");
    }

    #[test]
    fn test_get_slot_str() {
        let player = load_bouncy_ball();

        let color_json = player.get_slot_str("ball_color");
        assert!(
            !color_json.is_empty(),
            "should return JSON for existing slot"
        );

        let empty = player.get_slot_str("nonexistent");
        assert!(empty.is_empty(), "should return empty for nonexistent slot");
    }

    #[test]
    fn test_set_slot_str_and_get() {
        let mut player = load_bouncy_ball();

        // Set color slot via JSON
        let result = player.set_slot_str("ball_color", "{\"a\":0,\"k\":[1.0,0.0,0.0]}");
        assert!(result.is_ok(), "set_slot_str should succeed for known slot");

        // Verify type is still color
        assert_eq!(player.get_slot_type("ball_color"), "color");
    }

    #[test]
    fn test_reset_slot() {
        let mut player = load_bouncy_ball();

        // Get original value
        let original = player.get_slot_str("ball_color");

        // Set a new value
        player
            .set_color_slot("ball_color", dotlottie_rs::ColorSlot::new([1.0, 0.0, 0.0]))
            .unwrap();
        let modified = player.get_slot_str("ball_color");
        assert_ne!(original, modified, "slot value should change after set");

        // Reset to default
        let result = player.reset_slot("ball_color");
        assert!(result.is_ok(), "reset_slot should succeed");

        let restored = player.get_slot_str("ball_color");
        assert_eq!(
            original, restored,
            "slot should return to default after reset"
        );
    }

    #[test]
    fn test_reset_slots() {
        let mut player = load_bouncy_ball();

        let original_ids = player.get_slot_ids();

        // Set a color slot
        player
            .set_color_slot("ball_color", dotlottie_rs::ColorSlot::new([1.0, 0.0, 0.0]))
            .unwrap();

        // Reset all
        assert!(player.reset_slots(), "reset_slots should succeed");

        // IDs should still be present (defaults restored, not cleared)
        let ids_after_reset = player.get_slot_ids();
        assert_eq!(original_ids.len(), ids_after_reset.len());
    }

    #[test]
    fn test_get_slots_str() {
        let player = load_bouncy_ball();

        let all_slots = player.get_slots_str();
        assert!(
            !all_slots.is_empty(),
            "get_slots_str should return non-empty JSON"
        );
        assert!(all_slots.starts_with('{'), "should be a JSON object");
        assert!(
            all_slots.contains("ball_color"),
            "should contain ball_color slot"
        );
    }

    // ── New integration tests ─────────────────────────────────────

    #[test]
    fn test_get_slot_type_all_variants() {
        let player = load_bouncy_ball();
        assert_eq!(player.get_slot_type("ball_color"), "color");
        assert_eq!(player.get_slot_type("ball_opacity"), "scalar");
        assert_eq!(player.get_slot_type("ball_position"), "vector");
        assert_eq!(player.get_slot_type("ball_scale"), "vector");
    }

    #[test]
    fn test_set_and_get_scalar_slot() {
        let mut player = load_bouncy_ball();
        player
            .set_scalar_slot("ball_opacity", dotlottie_rs::ScalarSlot::new(0.5))
            .unwrap();
        let json = player.get_slot_str("ball_opacity");
        assert!(
            json.contains("0.5"),
            "scalar slot JSON should contain the set value, got: {json}"
        );
        assert_eq!(player.get_slot_type("ball_opacity"), "scalar");
    }

    #[test]
    fn test_set_and_get_vector_slot() {
        let mut player = load_bouncy_ball();
        player
            .set_vector_slot(
                "ball_position",
                dotlottie_rs::VectorSlot::static_value([50.0, 75.0]),
            )
            .unwrap();
        let json = player.get_slot_str("ball_position");
        assert!(
            json.contains("50.0"),
            "vector slot JSON should contain x value, got: {json}"
        );
        assert!(
            json.contains("75.0"),
            "vector slot JSON should contain y value, got: {json}"
        );
    }

    #[test]
    fn test_clear_slot() {
        let mut player = load_bouncy_ball();
        player.clear_slot("ball_color").unwrap();
        let ids = player.get_slot_ids();
        assert!(
            !ids.contains(&"ball_color".to_string()),
            "cleared slot should not appear in ids"
        );
        assert!(
            ids.contains(&"ball_opacity".to_string()),
            "other slots should remain"
        );
    }

    #[test]
    fn test_clear_slots() {
        let mut player = load_bouncy_ball();
        player.clear_slots().unwrap();
        let ids = player.get_slot_ids();
        assert!(ids.is_empty(), "all slots should be cleared");
    }

    #[test]
    fn test_set_slots_replaces_all() {
        let mut player = load_bouncy_ball();
        let mut new_slots = std::collections::BTreeMap::new();
        new_slots.insert(
            "custom".to_string(),
            dotlottie_rs::SlotType::Color(dotlottie_rs::ColorSlot::new([1.0, 1.0, 1.0])),
        );
        player.set_slots(new_slots).unwrap();
        let ids = player.get_slot_ids();
        assert_eq!(ids, vec!["custom"]);
    }

    #[test]
    fn test_set_slot_str_nonexistent() {
        let mut player = load_bouncy_ball();
        let result = player.set_slot_str("nonexistent", r#"{"a":0,"k":[1.0,0.0,0.0]}"#);
        assert!(result.is_err(), "setting a nonexistent slot should fail");
    }

    #[test]
    fn test_set_slot_str_malformed_json() {
        let mut player = load_bouncy_ball();
        let result = player.set_slot_str("ball_color", "not json");
        assert!(result.is_err(), "setting with malformed JSON should fail");
    }

    #[test]
    fn test_reset_nonexistent_slot() {
        let mut player = load_bouncy_ball();
        let result = player.reset_slot("nonexistent");
        assert!(result.is_err(), "resetting a nonexistent slot should fail");
    }

    #[test]
    fn test_get_slots_str_round_trip() {
        let player = load_bouncy_ball();
        let all_json = player.get_slots_str();
        let recovered = dotlottie_rs::slots_from_json_string(&all_json).unwrap();
        assert_eq!(recovered.len(), 4);
        assert!(recovered.contains_key("ball_color"));
        assert!(recovered.contains_key("ball_opacity"));
        assert!(recovered.contains_key("ball_position"));
        assert!(recovered.contains_key("ball_scale"));
        // Verify types match
        assert_eq!(
            dotlottie_rs::slots::slot_type_name(recovered.get("ball_color").unwrap()),
            "color"
        );
        assert_eq!(
            dotlottie_rs::slots::slot_type_name(recovered.get("ball_opacity").unwrap()),
            "scalar"
        );
    }

    #[test]
    fn test_slot_type_name_all_variants() {
        use dotlottie_rs::*;

        assert_eq!(
            slots::slot_type_name(&SlotType::Color(ColorSlot::new([1.0, 0.0, 0.0]))),
            "color"
        );
        assert_eq!(
            slots::slot_type_name(&SlotType::Scalar(ScalarSlot::new(1.0))),
            "scalar"
        );
        assert_eq!(
            slots::slot_type_name(&SlotType::Vector(VectorSlot::static_value([1.0, 2.0]))),
            "vector"
        );
        assert_eq!(
            slots::slot_type_name(&SlotType::Position(PositionSlot::static_value([1.0, 2.0]))),
            "position"
        );
        assert_eq!(
            slots::slot_type_name(&SlotType::Text(TextSlot::new("hello"))),
            "text"
        );
        assert_eq!(
            slots::slot_type_name(&SlotType::Image(ImageSlot::from_path(
                "img.png".to_string()
            ))),
            "image"
        );
        assert_eq!(
            slots::slot_type_name(&SlotType::Gradient(GradientSlot::new(vec![GradientStop {
                offset: 0.0,
                color: [1.0, 0.0, 0.0, 1.0]
            },]))),
            "gradient"
        );
    }
}
