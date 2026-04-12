mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

use std::ffi::CString;

use dotlottie_rs::{ColorSpace, DotLottiePlayer, DotLottiePlayerError, Mode, Segment};

#[cfg(test)]
mod tests {

    use super::*;

    struct TestConfig {
        mode: Mode,
        autoplay: bool,
        segment: Option<Segment>,
    }

    #[test]
    fn test_stop() {
        let configs: Vec<TestConfig> = vec![
            TestConfig {
                mode: Mode::Forward,
                autoplay: true,
                segment: None,
            },
            TestConfig {
                mode: Mode::Reverse,
                autoplay: true,
                segment: None,
            },
            TestConfig {
                mode: Mode::Bounce,
                autoplay: true,
                segment: None,
            },
            TestConfig {
                mode: Mode::ReverseBounce,
                autoplay: true,
                segment: None,
            },
            // test with different segments
            TestConfig {
                mode: Mode::Forward,
                autoplay: true,
                segment: Some(Segment { start: 10.0, end: 30.0 }),
            },
            TestConfig {
                mode: Mode::Reverse,
                autoplay: true,
                segment: Some(Segment { start: 10.0, end: 30.0 }),
            },
            TestConfig {
                mode: Mode::Bounce,
                autoplay: true,
                segment: Some(Segment { start: 10.0, end: 30.0 }),
            },
            TestConfig {
                mode: Mode::ReverseBounce,
                autoplay: true,
                segment: Some(Segment { start: 10.0, end: 30.0 }),
            },
        ];

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        for config in configs {
            let mut player = DotLottiePlayer::new();
            player.set_mode(config.mode);
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

            let Segment { start: start_frame, end: end_frame } = player.segment().unwrap();

            // wait until we're half way to the end
            let mid_frame = (start_frame + end_frame) / 2.0;

            assert_eq!(player.set_frame(mid_frame), Ok(()), "Frame should be set");
            assert_eq!(player.render(), Ok(()), "Frame should render");
            assert!(player.is_playing(), "Animation should be playing");

            assert_eq!(player.stop(), Ok(()), "Animation should stop");

            assert!(!player.is_playing(), "Animation should not be playing");
            assert!(player.is_stopped(), "Animation should be stopped");
            assert!(!player.is_paused(), "Animation should not be paused");

            // based on the mode the current frame should be at the start or end
            match player.mode() {
                Mode::Forward => {
                    assert_eq!(player.current_frame(), start_frame);
                }
                Mode::Reverse => {
                    assert_eq!(player.current_frame(), end_frame);
                }
                Mode::Bounce => {
                    assert_eq!(player.current_frame(), start_frame);
                }
                Mode::ReverseBounce => {
                    assert_eq!(player.current_frame(), end_frame);
                }
            }

            assert_eq!(
                player.stop(),
                Err(DotLottiePlayerError::InsufficientCondition),
                "Animation should not stop again"
            );
        }
    }
}
