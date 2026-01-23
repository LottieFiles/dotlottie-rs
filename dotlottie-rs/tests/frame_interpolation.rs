use dotlottie_rs::{Config, DotLottiePlayer};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default_use_frame_interpolation() {
        let player = DotLottiePlayer::new(Config::default(), 0);

        assert!(player.config().use_frame_interpolation);
    }

    #[test]
    fn test_set_use_frame_interpolation() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);

        let mut config = player.config();
        config.use_frame_interpolation = false;
        player.set_config(config);

        assert!(!player.config().use_frame_interpolation);
    }

    #[test]
    fn test_disable_frame_interpolation() {
        let mut player = DotLottiePlayer::new(
            Config {
                autoplay: true,
                use_frame_interpolation: false,
                ..Config::default()
            },
            0,
        );

        assert!(player.load_dotlottie_data(include_bytes!("fixtures/emoji.lottie"), WIDTH, HEIGHT));

        let mut rendered_frames: Vec<f32> = vec![];

        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) && player.render() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(!rendered_frames.is_empty());
        assert_eq!(
            rendered_frames.len(),
            (player.total_frames() - 1.0) as usize
        );

        assert_eq!(
            rendered_frames[rendered_frames.len() - 1],
            player.total_frames() - 1.0
        );

        for (i, frame) in rendered_frames.iter().enumerate() {
            assert!(*frame == frame.floor(), "Frame {i} is interpolated.");
        }
    }
}
