use std::ffi::CString;

use dotlottie_rs::{ColorSpace, Player, Segment};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    struct TestConfig {
        speed: f32,
        autoplay: bool,
        segment: Option<Segment>,
    }

    #[test]
    fn test_default_speed() {
        let player = Player::new();

        assert_eq!(player.speed(), 1.0);
    }

    #[test]
    fn test_set_speed() {
        let mut player = Player::new();

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
                    segment: Some(Segment {
                        start: 10.0,
                        end: 30.0,
                    }),
                    autoplay: true,
                },
                2.0,
            ),
            (
                TestConfig {
                    speed: 0.4,
                    autoplay: true,
                    segment: Some(Segment {
                        start: 10.0,
                        end: 30.0,
                    }),
                },
                0.4,
            ),
        ];

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        for (config, expected_speed) in configs {
            let mut player = Player::new();
            player.set_speed(config.speed);
            player.set_autoplay(config.autoplay);
            if let Some(seg) = config.segment {
                let _ = player.set_segment(Some(seg));
            }

            let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

            assert!(player
                .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
                .is_ok());

            assert!(
                player.load_animation_path(&path).is_ok(),
                "Animation should load"
            );
            assert!(player.is_playing(), "Animation should be playing");

            let seg = player.segment().unwrap();
            let expected_duration =
                (seg.end - seg.start) / player.total_frames() * player.duration();
            let dt = 1000.0 / 60.0;
            let mut tick_count = 0u32;

            // animation loop
            while !player.is_complete() {
                let _ = player.tick(dt);
                tick_count += 1;
            }

            let actual_duration = tick_count as f32 * dt;
            let playback_speed = expected_duration / actual_duration;

            // Discrete dt stepping introduces quantization error of ~dt/expected_duration
            // per cycle. At 60fps with short animations this can reach ~0.12.
            assert!(
                (playback_speed - expected_speed).abs() <= 0.15,
                "Expected playback speed to be close to {expected_speed}, found {playback_speed} (ticks: {tick_count}, actual_dur: {actual_duration:.3}s, expected_dur: {expected_duration:.3}s)"
            );
        }
    }

    #[test]
    fn test_zero_speed() {
        let mut player = Player::new();

        player.set_speed(0.0);

        assert_eq!(player.speed(), 1.0);
    }

    #[test]
    fn test_negative_speed() {
        let mut player = Player::new();

        player.set_speed(-1.0);

        assert_eq!(player.speed(), 1.0);
    }
}
