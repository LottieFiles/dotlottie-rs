use dotlottie_player_core::{Config, DotLottiePlayer};

mod test_utils;

use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod play_mode_tests {
    use dotlottie_player_core::Mode;

    use super::*;

    #[test]
    fn test_default_play_mode() {
        let player = DotLottiePlayer::new(Config::default());

        assert_eq!(player.config().mode, Mode::Forward);
    }

    #[test]
    fn test_set_play_mode() {
        let play_modes = vec![
            Mode::Forward,
            Mode::Reverse,
            Mode::Bounce,
            Mode::ReverseBounce,
        ];

        for mode in play_modes {
            let player = DotLottiePlayer::new(Config::default());

            let mut config = player.config();
            config.mode = mode;
            player.set_config(config);

            assert_eq!(
                player.config().mode,
                mode,
                "Expected play mode to be {:?}, found {:?}",
                mode,
                player.config().mode
            );

            assert!(
                player.load_animation_path("tests/assets/test.json", WIDTH, HEIGHT),
                "Animation should load"
            );

            match mode {
                Mode::Forward => {
                    assert_eq!(
                        player.current_frame(),
                        0.0,
                        "Expected current frame to be 0"
                    );
                }
                Mode::Reverse => {
                    assert_eq!(
                        player.current_frame(),
                        player.total_frames(),
                        "Expected current frame to be total frames"
                    );
                }
                Mode::Bounce => {
                    assert_eq!(
                        player.current_frame(),
                        0.0,
                        "Expected current frame to be 0"
                    );
                }
                Mode::ReverseBounce => {
                    assert_eq!(
                        player.current_frame(),
                        player.total_frames(),
                        "Expected current frame to be total frames"
                    );
                }
            }
        }
    }

    #[test]
    fn test_forward_play_mode() {
        let player =
            DotLottiePlayer::new(Config {
                mode: Mode::Forward,
                autoplay: true,
                use_frame_interpolation: false,
                ..Config::default()
            });

        assert!(
            player.load_animation_path("tests/assets/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        let mut rendered_frames: Vec<f32> = vec![];

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) && player.render() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(
            rendered_frames.len() >= player.total_frames() as usize,
            "Expected rendered frames to be greater than or equal to total frames"
        );

        let mut prev_frame = 0.0;
        for frame in rendered_frames {
            assert!(
                frame >= prev_frame,
                "Expected frame to be greater than or equal to previous frame"
            );
            prev_frame = frame;
        }

        assert_eq!(
            player.total_frames(),
            prev_frame,
            "Expected last frame to be total frames"
        );
    }

    #[test]
    fn test_reverse_play_mode() {
        let player =
            DotLottiePlayer::new(Config {
                mode: Mode::Reverse,
                autoplay: true,
                use_frame_interpolation: false,
                ..Config::default()
            });

        assert!(
            player.load_animation_path("tests/assets/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );

        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        let mut rendered_frames: Vec<f32> = vec![];

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) && player.render() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(
            rendered_frames.len() >= player.total_frames() as usize,
            "Expected rendered frames to be greater than or equal to total frames"
        );

        let mut prev_frame = player.total_frames();
        for frame in rendered_frames {
            assert!(
                frame <= prev_frame,
                "Expected frame to be less than or equal to previous frame"
            );
            prev_frame = frame;
        }

        // check if the last frame is 0
        assert_eq!(0.0, prev_frame, "Expected last frame to be 0");
    }

    #[test]
    #[ignore = "fail cause of a bug in is_complete()"]
    fn test_bounce_play_mode() {
        let player =
            DotLottiePlayer::new(Config {
                mode: Mode::Bounce,
                autoplay: true,
                use_frame_interpolation: false,
                ..Config::default()
            });

        assert!(
            player.load_animation_path("tests/assets/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );

        let mut rendered_frames: Vec<f32> = vec![];

        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) && player.render() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(
            rendered_frames.len() >= player.total_frames() as usize,
            "Expected rendered frames to be greater than or equal to total frames"
        );

        // check if the rendered frames are increasing and decreasing
        let mut prev_frame = 0.0;
        for frame in &rendered_frames {
            assert!(
                frame >= &prev_frame,
                "Expected frame to be greater than or equal to previous frame"
            );
            prev_frame = *frame;
        }

        for frame in rendered_frames.iter().rev() {
            assert!(
                frame <= &prev_frame,
                "Expected frame to be less than or equal to previous frame"
            );
            prev_frame = *frame;
        }

        // check if the last frame is 0
        assert_eq!(0.0, prev_frame, "Expected last frame to be 0");
    }

    #[test]
    #[ignore = "fail cause of a bug in is_complete()"]
    fn test_reverse_bounce_play_mode() {
        let player =
            DotLottiePlayer::new(Config {
                mode: Mode::ReverseBounce,
                autoplay: true,
                use_frame_interpolation: false,
                ..Config::default()
            });

        assert!(
            player.load_animation_path("tests/assets/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );

        assert!(player.is_playing(), "Animation should be playing");

        let mut rendered_frames: Vec<f32> = vec![];

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) && player.render() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(
            rendered_frames.len() >= player.total_frames() as usize,
            "Expected rendered frames to be greater than or equal to total frames"
        );

        // check if the rendered frames are decreasing and increasing
        let mut prev_frame = player.total_frames();
        for frame in &rendered_frames {
            assert!(
                frame <= &prev_frame,
                "Expected frame to be less than or equal to previous frame"
            );
            prev_frame = *frame;
        }

        for frame in rendered_frames.iter().rev() {
            assert!(
                frame >= &prev_frame,
                "Expected frame to be greater than or equal to previous frame"
            );
            prev_frame = *frame;
        }

        // check if the last frame is 0
        assert_eq!(0.0, prev_frame, "Expected last frame to be 0");
    }
}
