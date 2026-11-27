use dotlottie_rs::{ColorSpace, Player};
use std::ffi::CString;

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buflen() {
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .is_ok());

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());

        let _ = player.render();

        assert_eq!(buffer.len(), (WIDTH * HEIGHT) as usize);
    }

    #[test]
    fn test_buffer_len_with_animation_data() {
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .is_ok());

        let test_data_str = r#"{"v":"5.5.7","fr":60,"ip":0,"op":60,"w":100,"h":100,"nm":"Test","ddd":0,"assets":[],"layers":[],"markers":[]}"#;
        let test_data = CString::new(test_data_str).expect("Failed to create CString");
        assert_eq!(player.load_animation_data(&test_data), Ok(()));

        let _ = player.render();

        assert_eq!(buffer.len(), (WIDTH * HEIGHT) as usize);
    }

    #[test]
    fn set_sw_target_rejects_overflowing_size() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert_eq!(
            player.set_sw_target(&mut buffer, 65536, 65536, 65536, ColorSpace::ABGR8888),
            Err(dotlottie_rs::player::Error::InvalidParameter)
        );
    }

    #[test]
    fn players_share_one_buffer_with_stride() {
        const CANVAS_WIDTH: usize = 200;
        const REGION_WIDTH: usize = 80;
        let height = HEIGHT as usize;

        let region = |canvas: &[u32], x: usize| -> Vec<u32> {
            (0..height)
                .flat_map(|y| canvas[y * CANVAS_WIDTH + x..][..REGION_WIDTH].to_vec())
                .collect()
        };

        let mut canvas: Vec<u32> = vec![0; CANVAS_WIDTH * height];
        let region_len = (height - 1) * CANVAS_WIDTH + REGION_WIDTH;
        let data = CString::new(include_str!("../assets/animations/lottie/test.json")).unwrap();

        let mut players = Vec::new();
        for x in [10, 110] {
            let mut player = Player::new();
            player
                .set_sw_target(
                    &mut canvas[x..x + region_len],
                    CANVAS_WIDTH as u32,
                    REGION_WIDTH as u32,
                    HEIGHT,
                    ColorSpace::ABGR8888,
                )
                .unwrap();
            player
                .set_background(dotlottie_rs::Rgba::new(255, 0, 0, 255))
                .unwrap();
            player.load_animation_data(&data).unwrap();
            players.push(player);
        }

        for frame in [1.0, 10.0, 20.0] {
            players[0].set_frame(frame).unwrap();
            players[0].render().unwrap();
            let left = region(&canvas, 10);

            players[1].set_frame(frame + 5.0).unwrap();
            players[1].render().unwrap();
            assert_eq!(left, region(&canvas, 10));
        }

        for (i, pixel) in canvas.iter().enumerate() {
            let x = i % CANVAS_WIDTH;
            let inside = (10..90).contains(&x) || (110..190).contains(&x);
            assert_eq!(*pixel != 0, inside, "pixel at column {x}");
        }
    }

    #[test]
    fn set_sw_target_rejects_invalid_region() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; 8 * 3 + 4];

        assert_eq!(
            player.set_sw_target(&mut buffer, 4, 8, 4, ColorSpace::ABGR8888),
            Err(dotlottie_rs::player::Error::InvalidParameter)
        );
        assert_eq!(
            player.set_sw_target(&mut buffer, 8, 4, 5, ColorSpace::ABGR8888),
            Err(dotlottie_rs::player::Error::InvalidParameter)
        );
        assert_eq!(
            player.set_sw_target(&mut buffer, 8, 4, 4, ColorSpace::ABGR8888),
            Ok(())
        );
    }
}
