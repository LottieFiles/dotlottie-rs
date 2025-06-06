use dotlottie_rs::{Config, DotLottiePlayer};
use std::slice;

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buflen() {
        let player = DotLottiePlayer::new(Config::default());
        assert!(player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT));
        let frame = unsafe {
            slice::from_raw_parts(
                player.buffer_ptr() as *const u32,
                player.buffer_len() as usize,
            )
        };
        assert_eq!(frame.len(), (WIDTH * HEIGHT) as usize);
    }

    #[test]
    fn test_buffer_len_with_animation_data() {
        let player = DotLottiePlayer::new(Config::default());

        let test_data = r#"{"v":"5.5.7","fr":60,"ip":0,"op":60,"w":100,"h":100,"nm":"Test","ddd":0,"assets":[],"layers":[],"markers":[]}"#;
        assert!(player.load_animation_data(test_data, WIDTH, HEIGHT));

        assert_eq!(player.buffer_len() as usize, (WIDTH * HEIGHT) as usize);

        let frame = unsafe {
            slice::from_raw_parts(
                player.buffer_ptr() as *const u32,
                player.buffer_len() as usize,
            )
        };
        assert_eq!(frame.len(), (WIDTH * HEIGHT) as usize);
    }
}
