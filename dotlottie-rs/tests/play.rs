mod test_utils;

use crate::test_utils::{HEIGHT, WIDTH};
use dotlottie_rs::{Config, DotLottiePlayer, DotLottiePlayerError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_play_fail_when_animation_is_not_loaded() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);

        assert_eq!(
            player.play(),
            Err(DotLottiePlayerError::AnimationNotLoaded),
            "Expected play to fail when animation is not loaded"
        );

        assert!(player
            .load_animation_path("assets/animations/lottie/test.json", WIDTH, HEIGHT)
            .is_ok());

        assert_eq!(
            player.play(),
            Ok(()),
            "Expected play to succeed when animation is loaded"
        );
    }

    #[test]
    fn test_play_while_playing() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);

        assert!(player
            .load_animation_path("assets/animations/lottie/test.json", WIDTH, HEIGHT)
            .is_ok());

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
        let mut player = DotLottiePlayer::new(
            Config {
                use_frame_interpolation: false,
                ..Config::default()
            },
            0,
        );

        assert!(player
            .load_animation_path("assets/animations/lottie/test.json", WIDTH, HEIGHT)
            .is_ok());

        assert_eq!(player.play(), Ok(()));

        let mid_frame = player.total_frames() / 2.0;

        while player.current_frame() < mid_frame {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame).is_ok() {
                let _ = player.render();
            }
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
            let next_frame = player.request_frame();

            if player.set_frame(next_frame).is_ok() {
                let _ = player.render();

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
        let mut player = DotLottiePlayer::new(
            Config {
                use_frame_interpolation: false,
                ..Config::default()
            },
            0,
        );

        assert!(player
            .load_animation_path("assets/animations/lottie/test.json", WIDTH, HEIGHT)
            .is_ok());

        assert_eq!(player.play(), Ok(()));

        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame).is_ok() {
                let _ = player.render();
            }
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
        let mut player = DotLottiePlayer::new(
            Config {
                use_frame_interpolation: false,
                ..Config::default()
            },
            0,
        );

        assert!(player
            .load_animation_path("assets/animations/lottie/test.json", WIDTH, HEIGHT)
            .is_ok());

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
            let next_frame = player.request_frame();

            if player.set_frame(next_frame).is_ok() && player.render().is_ok() {
                rendered_frames.push(player.current_frame());
            }
        }

        assert!((rendered_frames[0] - mid_frame).abs() <= 1.0);
    }
}
