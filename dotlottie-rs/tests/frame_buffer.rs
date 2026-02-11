use dotlottie_rs::{Config, DotLottiePlayer};
use std::ffi::CString;
use std::slice;

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buflen() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        assert!(player
            .load_animation_path("assets/animations/lottie/test.json", WIDTH, HEIGHT)
            .is_ok());
        let frame =
            unsafe { slice::from_raw_parts(player.buffer().as_ptr(), player.buffer().len()) };
        assert_eq!(frame.len(), (WIDTH * HEIGHT) as usize);
    }

    #[test]
    fn test_buffer_len_with_animation_data() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);

        let test_data_str = r#"{"v":"5.5.7","fr":60,"ip":0,"op":60,"w":100,"h":100,"nm":"Test","ddd":0,"assets":[],"layers":[],"markers":[]}"#;
        let test_data = CString::new(test_data_str).expect("Failed to create CString");
        assert_eq!(
            player.load_animation_data(&test_data, WIDTH, HEIGHT),
            Ok(())
        );

        assert_eq!(player.buffer().len(), (WIDTH * HEIGHT) as usize);

        let frame =
            unsafe { slice::from_raw_parts(player.buffer().as_ptr(), player.buffer().len()) };
        assert_eq!(frame.len(), (WIDTH * HEIGHT) as usize);
    }
}
