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
        assert!(player.load_animation_path(&path).is_ok());
        assert!(player.is_playing());
        assert!(!player.is_paused());
        assert!(!player.is_stopped());
        assert!(!player.is_complete());
        assert_eq!(player.current_frame(), 0.0);

        let mut rendered_frames: Vec<f32> = vec![];
        let dt = 1.0 / 60.0;

        while !player.is_complete() {
            if player.tick(dt).unwrap_or(false) {
                rendered_frames.push(player.current_frame());
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
        let loaded = player.load_animation_path(&path);

        assert!(loaded.is_ok());

        assert!(!player.is_playing());
        assert!(!player.is_paused());
        assert!(player.is_stopped());
        assert!(!player.is_complete());
        assert!(player.current_frame() == 0.0);

        let times: usize = 10;

        for _ in 0..times {
            let _ = player.tick(1.0 / 60.0);
            assert_eq!(player.current_frame(), 0.0);
        }
    }
}
