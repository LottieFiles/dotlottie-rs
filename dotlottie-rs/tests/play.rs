mod test_utils;

use crate::test_utils::{HEIGHT, WIDTH};
use dotlottie_rs::{Config, DotLottiePlayer, DotLottieResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_play_fail_when_animation_is_not_loaded() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);

        assert_eq!(
            player.play(),
            DotLottieResult::AnimationNotLoaded,
            "Expected play to fail when animation is not loaded"
        );

        assert_eq!(player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT), DotLottieResult::Success);

        assert_eq!(
            player.play(),
            DotLottieResult::Success,
            "Expected play to succeed when animation is loaded"
        );
    }

    #[test]
    fn test_play_while_playing() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);

        assert_eq!(player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT), DotLottieResult::Success);

        assert_eq!(player.play(), DotLottieResult::Success);

        assert!(player.is_playing(), "Expected player to be playing");

        assert_eq!(player.play(), DotLottieResult::InsufficientCondition, "Expected play to fail when already playing");
    }

    #[test]
    fn test_play_after_pause() {
        let mut player = DotLottiePlayer::new(Config {
            use_frame_interpolation: false,
            ..Config::default()
        }, 0);

        assert_eq!(player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT), DotLottieResult::Success);

        assert_eq!(player.play(), DotLottieResult::Success);

        let mid_frame = player.total_frames() / 2.0;

        while player.current_frame() < mid_frame {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) == DotLottieResult::Success {
                player.render();
            }
        }

        assert_eq!(player.pause(), DotLottieResult::Success, "Expected pause to succeed");

        let paused_at = player.current_frame();

        assert_eq!(player.play(), DotLottieResult::Success, "Expected play to succeed after pause");

        let mut rendered_frames = vec![];

        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) == DotLottieResult::Success {
                player.render();

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
        let mut player = DotLottiePlayer::new(Config {
            use_frame_interpolation: false,
            ..Config::default()
        }, 0);

        assert_eq!(player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT), DotLottieResult::Success);

        assert_eq!(player.play(), DotLottieResult::Success);

        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) == DotLottieResult::Success {
                player.render();
            }
        }

        assert!(player.is_complete(), "Expected player to be complete");

        assert!(!player.is_playing(), "Expected player to not be playing");

        assert!(
            player.current_frame() == player.total_frames() - 1.0,
            "Expected current frame to be total frames"
        );

        assert_eq!(player.play(), DotLottieResult::Success, "Expected play to succeed after complete");

        assert_eq!(
            player.current_frame(),
            0.0,
            "Expected current frame to be 0 after play"
        );
    }

    #[test]
    fn test_play_after_setting_frame() {
        let mut player = DotLottiePlayer::new(Config {
            use_frame_interpolation: false,
            ..Config::default()
        }, 0);

        assert_eq!(player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT), DotLottieResult::Success);

        let mid_frame = player.total_frames() / 2.0;

        assert_eq!(player.set_frame(mid_frame), DotLottieResult::Success);

        assert_eq!(
            player.current_frame(),
            mid_frame,
            "Expected current frame to be mid frame"
        );

        assert_eq!(player.play(), DotLottieResult::Success);

        assert_eq!(
            player.current_frame(),
            mid_frame,
            "Expected current frame to be mid frame"
        );

        let mut rendered_frames = vec![];

        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) == DotLottieResult::Success && player.render() == DotLottieResult::Success {
                rendered_frames.push(player.current_frame());
            }
        }

        assert!((rendered_frames[0] - mid_frame).abs() <= 1.0);
    }
}
