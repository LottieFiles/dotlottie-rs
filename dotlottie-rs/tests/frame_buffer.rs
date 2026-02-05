use dotlottie_rs::{ColorSpace, Config, DotLottiePlayer};
use std::ffi::CString;

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buflen() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);

        // Allocate buffer for software rendering
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        // Set software rendering target
        assert!(player.set_sw_target(
            buffer.as_mut_ptr(),
            WIDTH,
            WIDTH,
            HEIGHT,
            ColorSpace::ABGR8888,
        ));

        assert!(player.load_animation_path("assets/animations/lottie/test.json", WIDTH, HEIGHT));

        // Render a frame
        player.render();

        assert_eq!(buffer.len(), (WIDTH * HEIGHT) as usize);
    }

    #[test]
    fn test_buffer_len_with_animation_data() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);

        // Allocate buffer for software rendering
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        // Set software rendering target
        assert!(player.set_sw_target(
            buffer.as_mut_ptr(),
            WIDTH,
            WIDTH,
            HEIGHT,
            ColorSpace::ABGR8888,
        ));

        let test_data_str = r#"{"v":"5.5.7","fr":60,"ip":0,"op":60,"w":100,"h":100,"nm":"Test","ddd":0,"assets":[],"layers":[],"markers":[]}"#;
        let test_data = CString::new(test_data_str).expect("Failed to create CString");
        assert!(player.load_animation_data(&test_data, WIDTH, HEIGHT));

        // Render a frame
        player.render();

        assert_eq!(buffer.len(), (WIDTH * HEIGHT) as usize);
    }
}
