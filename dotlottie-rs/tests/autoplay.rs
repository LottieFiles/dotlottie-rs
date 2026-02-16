use std::ffi::CString;

use dotlottie_rs::{ColorSpace, DotLottiePlayer};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default_autoplay() {
        let player = DotLottiePlayer::new();

        assert!(!player.autoplay());
    }

    #[test]
    fn test_set_autoplay() {
        let mut player = DotLottiePlayer::new();

        player.set_autoplay(true);

        assert!(player.autoplay());
    }

    #[test]
    fn test_autoplay() {
        let mut player = DotLottiePlayer::new();
        player.set_autoplay(true);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path, WIDTH, HEIGHT).is_ok());
        assert!(player.is_playing());
        assert!(!player.is_paused());
        assert!(!player.is_stopped());
        assert!(!player.is_complete());
        assert_eq!(player.current_frame(), 0.0);

        let mut rendered_frames: Vec<f32> = vec![];

        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame).is_ok() && player.render().is_ok() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(!rendered_frames.is_empty());
    }

    #[test]
    fn test_no_autoplay() {
        let mut player = DotLottiePlayer::new();
        player.set_autoplay(false);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .is_ok());

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        let loaded = player.load_animation_path(&path, WIDTH, HEIGHT);

        assert!(loaded.is_ok());

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
