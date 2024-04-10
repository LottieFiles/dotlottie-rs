use dotlottie_player_core::{Config, DotLottiePlayer};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Read,
        path::Path,
    };

    use super::*;

    #[test]
    fn test_load_valid_theme() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let valid_theme_id = "test_theme";

        assert!(
            !player.load_theme(valid_theme_id),
            "Expected theme to not load"
        );

        assert!(player.load_dotlottie_data(include_bytes!("assets/test.lottie"), WIDTH, HEIGHT));
        assert!(player.active_theme_id().is_empty());

        assert!(player.load_theme(valid_theme_id), "Expected theme to load");
        assert_eq!(player.active_theme_id(), valid_theme_id);

        assert!(player.is_playing());
    }

    #[test]
    fn test_load_invalid_theme() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let invalid_theme_id = "invalid_theme";

        assert!(
            !player.load_theme(invalid_theme_id),
            "Expected theme to not load"
        );

        assert!(player.load_dotlottie_data(include_bytes!("assets/test.lottie"), WIDTH, HEIGHT));

        assert!(
            !player.load_theme(invalid_theme_id),
            "Expected theme to not load"
        );

        assert!(player.is_playing());
    }

    #[test]
    #[ignore = "malloc: Double free detected when unloading theme"]
    fn test_unload_theme() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let theme_id = "test_theme";

        assert!(player.load_dotlottie_data(include_bytes!("assets/test.lottie"), WIDTH, HEIGHT));

        assert!(player.load_theme(theme_id), "Expected theme to load");
        assert!(player.load_theme(""), "Expected theme to unload");
    }

    #[test]
    fn test_unload_theme_before_load() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        assert!(player.load_dotlottie_data(include_bytes!("assets/test.lottie"), WIDTH, HEIGHT));

        assert!(player.load_theme(""), "Expected theme to unload");
    }

    #[test]
    fn test_clear_active_theme_id_after_new_animation_data_is_loaded() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let valid_theme_id = "test_theme";

        assert!(
            !player.load_theme(valid_theme_id),
            "Expected theme to not load"
        );

        assert!(player.load_dotlottie_data(include_bytes!("assets/test.lottie"), WIDTH, HEIGHT));

        assert!(player.load_theme(valid_theme_id), "Expected theme to load");
        assert_eq!(player.active_theme_id(), valid_theme_id);

        let data = std::str::from_utf8(include_bytes!("assets/test.json")).expect("Invalid data.");
        assert!(player.load_animation_data(data, WIDTH, HEIGHT));
        assert!(player.active_theme_id().is_empty());

        assert!(player.is_playing());
    }

    #[test]
    fn test_clear_active_theme_id_after_new_animation_path_is_loaded() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let valid_theme_id = "test_theme";

        assert!(
            !player.load_theme(valid_theme_id),
            "Expected theme to not load"
        );

        assert!(player.load_dotlottie_data(include_bytes!("assets/test.lottie"), WIDTH, HEIGHT));

        assert!(player.load_theme(valid_theme_id), "Expected theme to load");
        assert_eq!(player.active_theme_id(), valid_theme_id);

        assert!(player.load_animation_path("tests/assets/test.json", WIDTH, HEIGHT));
        assert!(player.active_theme_id().is_empty());

        assert!(player.is_playing());
    }

    #[test]
    fn test_clear_active_theme_id_after_new_dotlottie_is_loaded() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let valid_theme_id = "test_theme";

        assert!(player.load_dotlottie_data(include_bytes!("assets/test.lottie"), WIDTH, HEIGHT));
        assert!(player.active_theme_id().is_empty());

        assert!(player.load_theme(valid_theme_id), "Expected theme to load");
        assert_eq!(player.active_theme_id(), valid_theme_id);

        assert!(player.load_dotlottie_data(include_bytes!("assets/emoji.lottie"), WIDTH, HEIGHT));
        assert!(player.active_theme_id().is_empty());

        assert!(player.is_playing());
    }
}
