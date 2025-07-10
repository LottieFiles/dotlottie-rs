use dotlottie_rs::{Config, DotLottiePlayer};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default_speed() {
        let player = DotLottiePlayer::new(Config::default());

        assert_eq!(player.config().speed, 1.0);
    }

    #[test]
    fn test_set_speed() {
        let player = DotLottiePlayer::new(Config::default());

        let mut config = player.config();
        config.speed = 2.0;
        player.set_config(config);

        assert_eq!(player.config().speed, 2.0);
    }

    #[test]
    fn test_playback_speed_accuracy() {
        let configs: Vec<(Config, f32)> = vec![
            // test with default config
            (
                Config {
                    autoplay: true,
                    ..Config::default()
                },
                1.0,
            ),
            // test with different speeds
            (
                Config {
                    speed: 2.0,
                    autoplay: true,
                    ..Config::default()
                },
                2.0,
            ),
            (
                Config {
                    speed: 0.5,
                    autoplay: true,
                    ..Config::default()
                },
                0.5,
            ),
            // test with a segment
            (
                Config {
                    speed: 2.0,
                    segment: vec![10.0, 30.0],
                    autoplay: true,
                    ..Config::default()
                },
                2.0,
            ),
            (
                Config {
                    speed: 0.4,
                    autoplay: true,
                    segment: vec![10.0, 30.0],
                    ..Config::default()
                },
                0.4,
            ),
        ];

        for (config, expected_speed) in configs {
            let player = DotLottiePlayer::new(config);

            assert!(
                player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
                "Animation should load"
            );
            assert!(player.is_playing(), "Animation should be playing");

            let expected_duration = player.segment_duration();

            let start_time = std::time::Instant::now();

            // animation loop
            while !player.is_complete() {
                let next_frame = player.request_frame();
                if player.set_frame(next_frame) {
                    player.render();
                }
            }

            let end_time = std::time::Instant::now();

            let actual_duration = end_time.duration_since(start_time).as_secs_f32();

            let playback_speed = expected_duration / actual_duration;

            // assert if actual playback speed is close to the expected speed +/- 0.1
            assert!(
                (playback_speed - expected_speed).abs() <= 0.1,
                "Expected playback speed to be close to {expected_speed}, found {playback_speed}"
            );
        }
    }

    #[test]
    fn test_zero_speed() {
        let player = DotLottiePlayer::new(Config::default());

        let mut config = player.config();
        config.speed = 0.0;
        player.set_config(config);

        assert_eq!(player.config().speed, 1.0);
    }

    #[test]
    fn test_negative_speed() {
        let player = DotLottiePlayer::new(Config::default());

        let mut config = player.config();
        config.speed = -1.0;
        player.set_config(config);

        assert_eq!(player.config().speed, 1.0);
    }
}
