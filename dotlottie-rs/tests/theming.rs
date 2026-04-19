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
}
