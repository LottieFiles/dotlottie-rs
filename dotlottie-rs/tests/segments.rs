mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

use dotlottie_rs::{Config, DotLottiePlayer, Mode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_segment_rejected() {
        let config = Config {
            autoplay: true,
            segment: vec![50.0, 30.0],
            ..Config::default()
        };

        let mut player = DotLottiePlayer::new(config, 0);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );

        assert!(player.is_playing(), "Animation should be playing");

        let total_frames = player.total_frames();

        for i in 0..50 {
            let frame = player.request_frame();

            assert!(
                frame.is_finite(),
                "Frame should be finite at iteration {i}, got: {frame}"
            );

            assert!(
                frame >= 0.0 && frame < total_frames,
                "Frame should be within full animation range [0, {total_frames}), iteration {i}, got: {frame}"
            );
        }
    }

    #[test]
    fn test_same_start_end_rejected() {
        let config = Config {
            autoplay: true,
            segment: vec![0.0, 0.0],
            ..Config::default()
        };

        let mut player = DotLottiePlayer::new(config, 0);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );

        let total_frames = player.total_frames();

        for i in 0..20 {
            let frame = player.request_frame();

            assert!(
                frame.is_finite(),
                "Frame should be finite at iteration {i}"
            );

            assert!(
                frame >= 0.0 && frame < total_frames,
                "Frame should be within full range, iteration {i}, got: {frame}"
            );
        }
    }

    #[test]
    fn test_invalid_segment_all_modes() {
        let modes = vec![
            Mode::Forward,
            Mode::Reverse,
            Mode::Bounce,
            Mode::ReverseBounce,
        ];

        for mode in modes {
            let config = Config {
                mode,
                autoplay: true,
                segment: vec![50.0, 30.0],
                ..Config::default()
            };

            let mut player = DotLottiePlayer::new(config, 0);

            assert!(
                player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
                "Animation should load for mode {mode:?}"
            );

            let total_frames = player.total_frames();

            for i in 0..20 {
                let frame = player.request_frame();

                assert!(
                    frame.is_finite(),
                    "Frame should be finite for mode {mode:?}, iteration {i}, got: {frame}"
                );

                assert!(
                    frame >= 0.0 && frame < total_frames,
                    "Frame should be in valid range for mode {mode:?}, iteration {i}, got: {frame}"
                );
            }
        }
    }

    #[test]
    fn test_valid_segments_unchanged() {
        let config = Config {
            autoplay: true,
            segment: vec![30.0, 50.0],
            ..Config::default()
        };

        let mut player = DotLottiePlayer::new(config, 0);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load with valid segment"
        );

        let total_frames = player.total_frames();

        for i in 0..20 {
            let frame = player.request_frame();

            assert!(
                frame.is_finite(),
                "Frame should be finite, iteration {i}, got: {frame}"
            );

            assert!(
                frame >= 0.0 && frame < total_frames,
                "Frame should be in valid range, iteration {i}, got: {frame}"
            );
        }
    }

    #[test]
    fn test_set_config_rejects_invalid_segment() {
        let config = Config {
            autoplay: false,
            segment: vec![10.0, 20.0],
            ..Config::default()
        };

        let mut player = DotLottiePlayer::new(config, 0);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );

        let initial_config = player.config();
        assert_eq!(initial_config.segment, vec![10.0, 20.0]);

        let invalid_config = Config {
            autoplay: false,
            segment: vec![50.0, 30.0],
            ..initial_config
        };

        player.set_config(invalid_config);

        let updated_config = player.config();
        assert_eq!(
            updated_config.segment,
            vec![10.0, 20.0],
            "Invalid segment should be rejected, keeping previous valid segment"
        );

        let invalid_config2 = Config {
            autoplay: false,
            segment: vec![25.0, 25.0],
            ..updated_config
        };

        player.set_config(invalid_config2);

        let final_config = player.config();
        assert_eq!(
            final_config.segment,
            vec![10.0, 20.0],
            "Invalid segment [25, 25] should be rejected"
        );
    }
}
