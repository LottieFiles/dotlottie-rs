use dotlottie_player_core::{Config, DotLottiePlayer};

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
        let configs: Vec<(Config, f32)> =
            vec![
                // test with default config
                (
                    Config {
                        autoplay: true,
                        use_frame_interpolation: false,
                        ..Config::default()
                    },
                    1.0,
                ),
                // test with different speeds
                (
                    Config {
                        speed: 2.0,
                        autoplay: true,
                        use_frame_interpolation: false,
                        ..Config::default()
                    },
                    2.0,
                ),
                (
                    Config {
                        speed: 0.5,
                        autoplay: true,
                        use_frame_interpolation: false,
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
                        use_frame_interpolation: false,
                        ..Config::default()
                    },
                    2.0,
                ),
                (
                    Config {
                        speed: 0.4,
                        autoplay: true,
                        segment: vec![10.0, 30.0],
                        use_frame_interpolation: false,
                        ..Config::default()
                    },
                    0.4,
                ),
            ];

        for (config, expected_speed) in configs {
            let player = DotLottiePlayer::new(config);

            assert!(
                player.load_animation_path("tests/assets/test.json", WIDTH, HEIGHT),
                "Animation should load"
            );
            assert!(player.is_playing(), "Animation should be playing");

            let expected_duration = if player.config().segment.is_empty() {
                player.duration()
            } else {
                let segment_total_frames = player.config().segment[1] - player.config().segment[0];

                segment_total_frames / player.total_frames() * player.duration()
            };

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
                "Expected playback speed to be close to {}, found {}",
                expected_speed,
                playback_speed
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
