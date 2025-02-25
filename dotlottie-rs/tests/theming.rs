use dotlottie_rs::{Config, DotLottiePlayer};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[ignore]
    fn test_load_valid_theme() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let valid_theme_id = "test_theme";

        assert!(
            !player.set_theme(valid_theme_id),
            "Expected theme to not load"
        );

        assert!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT));
        assert!(player.active_theme_id().is_empty());

        assert!(player.set_theme(valid_theme_id), "Expected theme to load");
        assert_eq!(player.active_theme_id(), valid_theme_id);

        assert!(player.is_playing());
    }

    #[test]
    #[ignore]
    fn test_load_invalid_theme() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let invalid_theme_id = "invalid_theme";

        assert!(
            !player.set_theme(invalid_theme_id),
            "Expected theme to not load"
        );

        assert!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT));

        assert!(
            !player.set_theme(invalid_theme_id),
            "Expected theme to not load"
        );

        assert!(player.is_playing());
    }

    #[test]
    #[ignore = "malloc: Double free detected when unloading theme"]
    fn test_unset_theme() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let theme_id = "test_theme";

        assert!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT));

        assert!(player.set_theme(theme_id), "Expected theme to load");
        assert!(player.set_theme(""), "Expected theme to unload");
    }

    #[test]
    #[ignore]
    fn test_unset_theme_before_load() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        assert!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT));

        assert!(player.set_theme(""), "Expected theme to unload");
    }

    #[test]
    #[ignore]
    fn test_clear_active_theme_id_after_new_animation_data_is_loaded() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let valid_theme_id = "test_theme";

        assert!(
            !player.set_theme(valid_theme_id),
            "Expected theme to not load"
        );

        assert!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT));

        assert!(player.set_theme(valid_theme_id), "Expected theme to load");
        assert_eq!(player.active_theme_id(), valid_theme_id);

        let data =
            std::str::from_utf8(include_bytes!("fixtures/test.json")).expect("Invalid data.");
        assert!(player.load_animation_data(data, WIDTH, HEIGHT));
        assert!(player.active_theme_id().is_empty());

        assert!(player.is_playing());
    }

    #[test]
    #[ignore]
    fn test_clear_active_theme_id_after_new_animation_path_is_loaded() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let valid_theme_id = "test_theme";

        assert!(
            !player.set_theme(valid_theme_id),
            "Expected theme to not load"
        );

        assert!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT));

        assert!(player.set_theme(valid_theme_id), "Expected theme to load");
        assert_eq!(player.active_theme_id(), valid_theme_id);

        assert!(player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT));
        assert!(player.active_theme_id().is_empty());

        assert!(player.is_playing());
    }

    #[test]
    #[ignore]
    fn test_clear_active_theme_id_after_new_dotlottie_is_loaded() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let valid_theme_id = "test_theme";

        assert!(player.load_dotlottie_data(include_bytes!("fixtures/test.lottie"), WIDTH, HEIGHT));
        assert!(player.active_theme_id().is_empty());

        assert!(player.set_theme(valid_theme_id), "Expected theme to load");
        assert_eq!(player.active_theme_id(), valid_theme_id);

        assert!(player.load_dotlottie_data(include_bytes!("fixtures/emoji.lottie"), WIDTH, HEIGHT));
        assert!(player.active_theme_id().is_empty());

        assert!(player.is_playing());
    }
}
