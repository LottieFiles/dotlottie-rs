use dotlottie_player_core::{Config, DotLottiePlayer};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_theme() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let valid_theme_id = "test_theme";

        assert!(
            player.load_theme(valid_theme_id) == false,
            "Expected theme to not load"
        );

        assert!(player.load_dotlottie_data(include_bytes!("assets/test.lottie"), WIDTH, HEIGHT));

        assert!(player.load_theme(valid_theme_id), "Expected theme to load");

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
            player.load_theme(invalid_theme_id) == false,
            "Expected theme to not load"
        );

        assert!(player.load_dotlottie_data(include_bytes!("assets/test.lottie"), WIDTH, HEIGHT));

        assert!(
            player.load_theme(invalid_theme_id) == false,
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
}
