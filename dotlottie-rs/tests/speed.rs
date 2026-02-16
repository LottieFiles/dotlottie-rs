use std::ffi::CString;

use dotlottie_rs::{ColorSpace, Config, DotLottiePlayer};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    struct TestConfig {
        speed: f32,
        autoplay: bool,
        segment: Option<[f32; 2]>,
    }

    #[test]
    fn test_default_speed() {
        let player = DotLottiePlayer::new(0);

        assert_eq!(player.speed(), 1.0);
    }

    #[test]
    fn test_set_speed() {
        let mut player = DotLottiePlayer::new(0);

        player.set_speed(2.0);

        assert_eq!(player.speed(), 2.0);
    }

    #[test]
    fn test_playback_speed_accuracy() {
        let configs: Vec<(TestConfig, f32)> = vec![
            // test with default config
            (
                TestConfig {
                    speed: 1.0,
                    autoplay: true,
                    segment: None,
                },
                1.0,
            ),
            // test with different speeds
            (
                TestConfig {
                    speed: 2.0,
                    autoplay: true,
                    segment: None,
                },
                2.0,
            ),
            (
                TestConfig {
                    speed: 0.5,
                    autoplay: true,
                    segment: None,
                },
                0.5,
            ),
            // test with a segment
            (
                TestConfig {
                    speed: 2.0,
                    segment: Some([10.0, 30.0]),
                    autoplay: true,
                },
                2.0,
            ),
            (
                TestConfig {
                    speed: 0.4,
                    autoplay: true,
                    segment: Some([10.0, 30.0]),
                },
                0.4,
            ),
        ];

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        for (config, expected_speed) in configs {
            let mut player = DotLottiePlayer::new(0);
            player.set_speed(config.speed);
            player.set_autoplay(config.autoplay);
            if let Some(seg) = config.segment {
                let _ = player.set_segment(Some(seg));
            }

            let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

            assert!(player.set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,).is_ok());

            assert!(
                player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
                "Animation should load"
            );
            assert!(player.is_playing(), "Animation should be playing");

            let expected_duration = player.segment_duration();

            let start_time = std::time::Instant::now();

            // animation loop
            while !player.is_complete() {
                let next_frame = player.request_frame();
                if player.set_frame(next_frame).is_ok() {
                    let _ = player.render();
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
        let mut player = DotLottiePlayer::new(0);

        player.set_speed(0.0);

        assert_eq!(player.speed(), 1.0);
    }

    #[test]
    fn test_negative_speed() {
        let mut player = DotLottiePlayer::new(0);

        player.set_speed(-1.0);

        assert_eq!(player.speed(), 1.0);
    }
}
