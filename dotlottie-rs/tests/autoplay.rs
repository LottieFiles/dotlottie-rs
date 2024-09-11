use dotlottie_rs::{Config, DotLottiePlayer};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default_autoplay() {
        let player = DotLottiePlayer::new(Config::default());

        assert!(!player.config().autoplay);
    }

    #[test]
    fn test_set_autoplay() {
        let player = DotLottiePlayer::new(Config::default());

        let mut config = player.config();
        config.autoplay = true;
        player.set_config(config);

        assert!(player.config().autoplay);
    }

    #[test]
    fn test_autoplay() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        assert!(player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT));
        assert!(player.is_playing());
        assert!(!player.is_paused());
        assert!(!player.is_stopped());
        assert!(!player.is_complete());
        assert_eq!(player.current_frame(), 0.0);

        let mut rendered_frames: Vec<f32> = vec![];

        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) && player.render() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(!rendered_frames.is_empty());
    }

    #[test]
    fn test_no_autoplay() {
        let player = DotLottiePlayer::new(Config {
            autoplay: false,
            ..Config::default()
        });

        let loaded = player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT);

        assert!(loaded);

        assert!(!player.is_playing());
        assert!(!player.is_paused());
        assert!(player.is_stopped());
        assert!(!player.is_complete());
        assert!(player.current_frame() == 0.0);

        let times: usize = 10;

        for _ in 0..times {
            let next_frame = player.request_frame();
            assert_eq!(next_frame, 0.0);
        }
    }
}
