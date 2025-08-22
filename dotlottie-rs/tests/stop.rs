mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

use dotlottie_rs::{ColorSpace, Config, DotLottiePlayer, Mode};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_stop() {
        let configs: Vec<Config> = vec![
            Config {
                autoplay: true,
                ..Config::default()
            },
            Config {
                mode: Mode::Reverse,
                autoplay: true,
                ..Config::default()
            },
            Config {
                mode: Mode::Bounce,
                autoplay: true,
                ..Config::default()
            },
            Config {
                mode: Mode::ReverseBounce,
                autoplay: true,
                ..Config::default()
            },
            // test with different segments
            Config {
                autoplay: true,
                segment: vec![10.0, 30.0],
                ..Config::default()
            },
            Config {
                mode: Mode::Reverse,
                autoplay: true,
                segment: vec![10.0, 30.0],
                ..Config::default()
            },
            Config {
                mode: Mode::Bounce,
                autoplay: true,
                segment: vec![10.0, 30.0],
                ..Config::default()
            },
            Config {
                mode: Mode::ReverseBounce,
                autoplay: true,
                segment: vec![10.0, 30.0],
                ..Config::default()
            },
        ];

        for config in configs {
            let player = DotLottiePlayer::new(config);

            let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];
            player.set_sw_target(
                buffer.as_ptr() as u64,
                WIDTH as u32,
                WIDTH as u32,
                HEIGHT as u32,
                ColorSpace::ARGB8888,
            );

            assert!(
                player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
                "Animation should load"
            );

            assert!(player.is_playing(), "Animation should be playing");

            let (start_frame, end_frame) = if player.config().segment.is_empty() {
                (0.0, player.total_frames() - 1.0)
            } else {
                (player.config().segment[0], player.config().segment[1])
            };

            // wait until we're half way to the end
            let mid_frame = (start_frame + end_frame) / 2.0;

            assert!(player.set_frame(mid_frame), "Frame should be set");
            assert!(player.render(), "Frame should render");
            assert!(player.is_playing(), "Animation should be playing");

            assert!(player.stop(), "Animation should stop");

            assert!(!player.is_playing(), "Animation should not be playing");
            assert!(player.is_stopped(), "Animation should be stopped");
            assert!(!player.is_paused(), "Animation should not be paused");

            // based on the mode the current frame should be at the start or end
            match player.config().mode {
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

            assert!(!player.stop(), "Animation should not stop again");
        }
    }
}
