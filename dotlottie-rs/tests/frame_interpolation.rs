use dotlottie_rs::{ColorSpace, Player};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default_use_frame_interpolation() {
        let player = Player::new();

        assert!(player.use_frame_interpolation());
    }

    #[test]
    fn test_set_use_frame_interpolation() {
        let mut player = Player::new();

        player.set_use_frame_interpolation(false);

        assert!(!player.use_frame_interpolation());
    }

    #[test]
    fn test_disable_frame_interpolation() {
        let mut player = Player::new();
        player.set_autoplay(true);
        player.set_use_frame_interpolation(false);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/emojis.lottie"
            ))
            .is_ok());

        let mut rendered_frames: Vec<f32> = vec![];

        while !player.is_complete() {
            if player.tick(1000.0 / 60.0).unwrap_or(false) {
                rendered_frames.push(player.current_frame());
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
