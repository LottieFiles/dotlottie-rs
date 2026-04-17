mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

use std::ffi::CString;

use dotlottie_rs::{ColorSpace, DotLottiePlayer, Mode, Segment, Status};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_segment_rejected() {
        let mut player = DotLottiePlayer::new();
        player.set_autoplay(true);

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(
            player.load_animation_path(&path).is_ok(),
            "Animation should load"
        );

        let result = player.set_segment(Some(Segment {
            start: 50.0,
            end: 30.0,
        }));
        assert!(result.is_err(), "Invalid segment should be rejected");

        assert_eq!(
            player.status(),
            Status::Playing,
            "Animation should be playing"
        );

        let total_frames = player.total_frames();

        for i in 0..50 {
            let _ = player.tick(1000.0 / 60.0);
            let frame = player.current_frame();

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
        let mut player = DotLottiePlayer::new();
        player.set_autoplay(true);

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(
            player.load_animation_path(&path).is_ok(),
            "Animation should load"
        );

        let result = player.set_segment(Some(Segment {
            start: 0.0,
            end: 0.0,
        }));
        assert!(result.is_err(), "Same start/end segment should be rejected");

        let total_frames = player.total_frames();

        for i in 0..20 {
            let _ = player.tick(1000.0 / 60.0);
            let frame = player.current_frame();

            assert!(frame.is_finite(), "Frame should be finite at iteration {i}");

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

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        for mode in modes {
            let mut player = DotLottiePlayer::new();
            player.set_mode(mode);
            player.set_autoplay(true);

            let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

            assert!(player
                .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
                .is_ok());

            assert!(
                player.load_animation_path(&path).is_ok(),
                "Animation should load for mode {mode:?}"
            );

            let result = player.set_segment(Some(Segment {
                start: 50.0,
                end: 30.0,
            }));
            assert!(
                result.is_err(),
                "Invalid segment should be rejected for mode {mode:?}"
            );

            let total_frames = player.total_frames();

            for i in 0..20 {
                let _ = player.tick(1000.0 / 60.0);
                let frame = player.current_frame();

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
        let mut player = DotLottiePlayer::new();
        player.set_autoplay(true);

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(
            player.load_animation_path(&path).is_ok(),
            "Animation should load with valid segment"
        );

        assert!(player
            .set_segment(Some(Segment {
                start: 30.0,
                end: 50.0
            }))
            .is_ok());

        let total_frames = player.total_frames();

        for i in 0..20 {
            let _ = player.tick(1000.0 / 60.0);
            let frame = player.current_frame();

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
    fn test_set_segment_rejects_invalid() {
        let mut player = DotLottiePlayer::new();
        player.set_autoplay(false);

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(
            player.load_animation_path(&path).is_ok(),
            "Animation should load"
        );

        assert!(player
            .set_segment(Some(Segment {
                start: 10.0,
                end: 20.0
            }))
            .is_ok());

        let initial_segment = player.segment().unwrap();
        assert_eq!(
            initial_segment,
            Segment {
                start: 10.0,
                end: 20.0
            }
        );

        // Try to set invalid segment
        let result = player.set_segment(Some(Segment {
            start: 50.0,
            end: 30.0,
        }));
        assert!(result.is_err(), "Invalid segment should be rejected");

        let updated_segment = player.segment().unwrap();
        assert_eq!(
            updated_segment,
            Segment {
                start: 10.0,
                end: 20.0
            },
            "Invalid segment should be rejected, keeping previous valid segment"
        );

        // Try to set same start/end segment
        let result2 = player.set_segment(Some(Segment {
            start: 25.0,
            end: 25.0,
        }));
        assert!(
            result2.is_err(),
            "Same start/end segment should be rejected"
        );

        let final_segment = player.segment().unwrap();
        assert_eq!(
            final_segment,
            Segment {
                start: 10.0,
                end: 20.0
            },
            "Invalid segment [25, 25] should be rejected"
        );
    }
}
