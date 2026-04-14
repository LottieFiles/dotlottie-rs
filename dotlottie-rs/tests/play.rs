mod test_utils;

use std::ffi::CString;

use crate::test_utils::{HEIGHT, WIDTH};
use dotlottie_rs::{ColorSpace, DotLottiePlayer, DotLottiePlayerError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_play_fail_when_animation_is_not_loaded() {
        let mut player = DotLottiePlayer::new();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert_eq!(
            player.play(),
            Err(DotLottiePlayerError::AnimationNotLoaded),
            "Expected play to fail when animation is not loaded"
        );

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());

        assert_eq!(
            player.play(),
            Ok(()),
            "Expected play to succeed when animation is loaded"
        );
    }

    #[test]
    fn test_play_while_playing() {
        let mut player = DotLottiePlayer::new();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());

        assert_eq!(player.play(), Ok(()));

        assert!(player.is_playing(), "Expected player to be playing");

        assert_eq!(
            player.play(),
            Err(DotLottiePlayerError::InsufficientCondition),
            "Expected play to fail when already playing"
        );
    }

    #[test]
    fn test_play_after_pause() {
        let mut player = DotLottiePlayer::new();
        player.set_use_frame_interpolation(false);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());

        assert_eq!(player.play(), Ok(()));

        let mid_frame = player.total_frames() / 2.0;

        while player.current_frame() < mid_frame {
            let _ = player.tick(1.0 / 60.0);
        }

        assert_eq!(player.pause(), Ok(()), "Expected pause to succeed");

        let paused_at = player.current_frame();

        assert_eq!(
            player.play(),
            Ok(()),
            "Expected play to succeed after pause"
        );

        let mut rendered_frames = vec![];

        while !player.is_complete() {
            if player.tick(1.0 / 60.0).unwrap_or(false) {
                rendered_frames.push(player.current_frame());
            }
        }

        assert!(
            (rendered_frames[0] - paused_at).abs() <= 1.0,
            "Expected first rendered frame to be the same as the frame we paused at"
        );
    }

    #[test]
    fn test_play_after_complete() {
        let mut player = DotLottiePlayer::new();
        player.set_use_frame_interpolation(false);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());

        assert_eq!(player.play(), Ok(()));

        while !player.is_complete() {
            let _ = player.tick(1.0 / 60.0);
        }

        assert!(player.is_complete(), "Expected player to be complete");

        assert!(!player.is_playing(), "Expected player to not be playing");

        assert!(
            player.current_frame() == player.total_frames() - 1.0,
            "Expected current frame to be total frames"
        );

        assert_eq!(
            player.play(),
            Ok(()),
            "Expected play to succeed after complete"
        );

        assert_eq!(
            player.current_frame(),
            0.0,
            "Expected current frame to be 0 after play"
        );
    }

    #[test]
    fn test_play_after_setting_frame() {
        let mut player = DotLottiePlayer::new();
        player.set_use_frame_interpolation(false);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert_eq!(player.load_animation_path(&path), Ok(()));

        let mid_frame = player.total_frames() / 2.0;

        assert_eq!(player.set_frame(mid_frame), Ok(()));

        assert_eq!(
            player.current_frame(),
            mid_frame,
            "Expected current frame to be mid frame"
        );

        assert_eq!(player.play(), Ok(()));

        assert_eq!(
            player.current_frame(),
            mid_frame,
            "Expected current frame to be mid frame"
        );

        let mut rendered_frames = vec![];

        while !player.is_complete() {
            if player.tick(1.0 / 60.0).unwrap_or(false) {
                rendered_frames.push(player.current_frame());
            }
        }

        assert!((rendered_frames[0] - mid_frame).abs() <= 1.0);
    }

    #[test]
    fn test_tick_zero_dt_does_not_advance() {
        let mut player = DotLottiePlayer::new();
        player.set_autoplay(true);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .is_ok());

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());

        // First tick with real dt to establish a rendered frame
        let _ = player.tick(1.0 / 60.0);
        let frame_after_first = player.current_frame();

        // Subsequent ticks with dt=0 should not advance the frame
        for _ in 0..10 {
            let result = player.tick(0.0);
            assert_eq!(result, Ok(false), "Zero dt should skip rendering");
        }
        assert_eq!(
            player.current_frame(),
            frame_after_first,
            "Frame should not change with zero dt"
        );
    }

    #[test]
    fn test_tick_large_dt_completes_without_panic() {
        let mut player = DotLottiePlayer::new();
        player.set_autoplay(true);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .is_ok());

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());

        // A huge dt should jump to the end frame in one tick
        let result = player.tick(100.0);
        assert!(result.is_ok(), "Large dt should not panic");
        assert!(
            player.is_complete(),
            "Animation should be complete after a massive dt"
        );
    }

    #[test]
    fn test_tick_negative_dt_is_clamped_to_zero() {
        let mut player = DotLottiePlayer::new();
        player.set_autoplay(true);
        player.set_use_frame_interpolation(false);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .is_ok());

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());

        // Advance a few frames normally
        for _ in 0..5 {
            let _ = player.tick(1.0 / 60.0);
        }
        let frame_before = player.current_frame();

        // Negative dt should be clamped to 0 and not reverse or panic
        let result = player.tick(-1.0);
        assert!(result.is_ok(), "Negative dt should not panic");
        assert_eq!(
            player.current_frame(),
            frame_before,
            "Frame should not change with negative dt"
        );
    }
}
