mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

use std::ffi::CString;

use dotlottie_rs::{DotLottiePlayer, Mode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_segment_rejected() {
        let mut player = DotLottiePlayer::new(0);
        player.set_autoplay(true);
        let _ = player.set_segment(Some([50.0, 30.0]));

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
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
        let mut player = DotLottiePlayer::new(0);
        player.set_autoplay(true);
        let _ = player.set_segment(Some([0.0, 0.0]));

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
            "Animation should load"
        );

        let total_frames = player.total_frames();

        for i in 0..20 {
            let frame = player.request_frame();

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
            let mut player = DotLottiePlayer::new(0);
            player.set_mode(mode);
            player.set_autoplay(true);
            let _ = player.set_segment(Some([50.0, 30.0]));

            assert!(
                player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
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
        let mut player = DotLottiePlayer::new(0);
        player.set_autoplay(true);
        let _ = player.set_segment(Some([30.0, 50.0]));

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
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
    fn test_set_segment_rejects_invalid() {
        let mut player = DotLottiePlayer::new(0);
        player.set_autoplay(false);
        let _ = player.set_segment(Some([10.0, 20.0]));

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
            "Animation should load"
        );

        let initial_segment = player.segment();
        assert_eq!(initial_segment, Some([10.0, 20.0]));

        // Try to set invalid segment
        let result = player.set_segment(Some([50.0, 30.0]));
        assert!(result.is_err(), "Invalid segment should be rejected");

        let updated_segment = player.segment();
        assert_eq!(
            updated_segment,
            Some([10.0, 20.0]),
            "Invalid segment should be rejected, keeping previous valid segment"
        );

        // Try to set same start/end segment
        let result2 = player.set_segment(Some([25.0, 25.0]));
        assert!(
            result2.is_err(),
            "Same start/end segment should be rejected"
        );

        let final_segment = player.segment();
        assert_eq!(
            final_segment,
            Some([10.0, 20.0]),
            "Invalid segment [25, 25] should be rejected"
        );
    }
}
