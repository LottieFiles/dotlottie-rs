use dotlottie_rs::{Config, DotLottiePlayer, DotLottiePlayerError};
use std::ffi::CString;

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_load_valid_theme() {
        let mut player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        }, 0);

        let valid_theme_id = "test_theme";

        assert_eq!(
            player.set_theme(valid_theme_id),
            Err(DotLottiePlayerError::InsufficientCondition),
            "Expected theme to not load"
        );

        assert_eq!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT), Ok(()));
        assert!(player.active_theme_id().is_empty());

        assert_eq!(player.set_theme(valid_theme_id), Ok(()), "Expected theme to load");
        assert_eq!(player.active_theme_id(), valid_theme_id);

        assert!(player.is_playing());
    }

    #[test]
    fn test_load_invalid_theme() {
        let mut player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        }, 0);

        let invalid_theme_id = "invalid_theme";

        assert_eq!(
            player.set_theme(invalid_theme_id),
            Err(DotLottiePlayerError::InsufficientCondition),
            "Expected theme to not load"
        );

        assert_eq!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT), Ok(()));

        assert_ne!(
            player.set_theme(invalid_theme_id),
            Ok(()),
            "Expected theme to not load"
        );

        assert!(player.is_playing());
    }

    #[test]
    fn test_unset_theme() {
        let mut player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        }, 0);

        let theme_id = "test_theme";

        assert_eq!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT), Ok(()));

        assert_eq!(player.set_theme(theme_id), Ok(()), "Expected theme to load");
        assert_eq!(player.set_theme(""), Ok(()), "Expected theme to unload");
    }

    #[test]
    fn test_unset_theme_before_load() {
        let mut player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        }, 0);

        assert_eq!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT), Ok(()));

        assert_eq!(player.set_theme(""), Ok(()), "Expected theme to unload");
    }

    #[test]
    fn test_clear_active_theme_id_after_new_animation_data_is_loaded() {
        let mut player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        }, 0);

        let valid_theme_id = "test_theme";

        assert_eq!(
            player.set_theme(valid_theme_id),
            Err(DotLottiePlayerError::InsufficientCondition),
            "Expected theme to not load"
        );

        assert_eq!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT), Ok(()));

        assert_eq!(player.set_theme(valid_theme_id), Ok(()), "Expected theme to load");
        assert_eq!(player.active_theme_id(), valid_theme_id);

        let data_str =
            std::str::from_utf8(include_bytes!("fixtures/test.json")).expect("Invalid data.");
        let data = CString::new(data_str).expect("Failed to create CString");
        assert_eq!(player.load_animation_data(&data, WIDTH, HEIGHT), Ok(()));
        assert!(player.active_theme_id().is_empty());

        assert!(player.is_playing());
    }

    #[test]
    fn test_clear_active_theme_id_after_new_animation_path_is_loaded() {
        let mut player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        }, 0);

        let valid_theme_id = "test_theme";

        assert_eq!(
            player.set_theme(valid_theme_id),
            Err(DotLottiePlayerError::InsufficientCondition),
            "Expected theme to not load"
        );

        assert_eq!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT), Ok(()));

        assert_eq!(player.set_theme(valid_theme_id), Ok(()), "Expected theme to load");
        assert_eq!(player.active_theme_id(), valid_theme_id);

        assert_eq!(player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT), Ok(()));
        assert!(player.active_theme_id().is_empty());

        assert!(player.is_playing());
    }

    #[test]
    fn test_clear_active_theme_id_after_new_dotlottie_is_loaded() {
        let mut player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        }, 0);

        let valid_theme_id = "test_theme";

        assert_eq!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT), Ok(()));
        assert!(player.active_theme_id().is_empty());

        assert_eq!(player.set_theme(valid_theme_id), Ok(()), "Expected theme to load");
        assert_eq!(player.active_theme_id(), valid_theme_id);

        assert_eq!(player.load_dotlottie_data(include_bytes!("fixtures/emoji.lottie"), WIDTH, HEIGHT), Ok(()));
        assert!(player.active_theme_id().is_empty());

        assert!(player.is_playing());
    }
}
