use dotlottie_rs::{ColorSpace, Player, PlayerError};
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

        assert!(player.is_playing());
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

        assert!(player.is_playing());
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

        assert!(player.is_playing());
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

        assert!(player.is_playing());
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

        assert!(player.is_playing());
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

        assert!(player.is_playing());
    }

    /// Regression: `reset_theme()` must restore slots to their initial
    /// (default) values, NOT clear them. After resetting the theme the
    /// animation's own slots should still be present, and any overridden
    /// values should be back to the loaded defaults.
    #[test]
    fn test_reset_theme_restores_slots_to_defaults() {
        use dotlottie_rs::ColorSlot;

        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .is_ok());

        // bouncy_ball.json defines slots (ball_color, ball_opacity, ...) whose
        // initial values are captured at load time.
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

        // Override a slot, then reset the theme.
        player
            .set_color_slot("ball_color", ColorSlot::new([1.0, 0.0, 0.0]))
            .unwrap();
        assert_ne!(
            player.get_slot_str("ball_color"),
            default_color,
            "override should change the slot value"
        );

        assert_eq!(player.reset_theme(), Ok(()));

        // Slots must NOT be cleared — they remain, restored to initial values.
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
}
