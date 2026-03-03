use dotlottie_rs::{ColorSpace, DotLottiePlayer};
use std::ffi::CString;

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use super::*;

    fn load_bouncy_ball() -> DotLottiePlayer {
        let mut player = DotLottiePlayer::new();
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .unwrap();

        let data = include_str!("../assets/animations/lottie/bouncy_ball.json");
        let c_data = CString::new(data).unwrap();
        player.load_animation_data(&c_data, WIDTH, HEIGHT).unwrap();
        player
    }

    #[test]
    fn test_get_slot_ids_after_load() {
        let player = load_bouncy_ball();

        // bouncy_ball.json has 4 slots: ball_opacity, ball_color, ball_position, ball_scale
        let ids = player.get_slot_ids();
        assert!(!ids.is_empty(), "get_slot_ids should return slot IDs from animation");

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
        assert!(!color_json.is_empty(), "should return JSON for existing slot");

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
        assert_eq!(original, restored, "slot should return to default after reset");
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
        assert!(!all_slots.is_empty(), "get_slots_str should return non-empty JSON");
        assert!(all_slots.starts_with('{'), "should be a JSON object");
        assert!(all_slots.contains("ball_color"), "should contain ball_color slot");
    }
}
