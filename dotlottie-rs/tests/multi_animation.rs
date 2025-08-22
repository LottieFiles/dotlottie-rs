use dotlottie_rs::{Config, DotLottiePlayer};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use dotlottie_rs::ColorSpace;

    use super::*;

    #[test]
    pub fn test_load_animation_with_animation_id() {
        let animation_id = "crying".to_string();

        let player = DotLottiePlayer::new(Config {
            animation_id: animation_id.clone(),
            ..Config::default()
        });

        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];

        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );

        assert!(player.load_dotlottie_data(include_bytes!("fixtures/emoji.lottie"), WIDTH, HEIGHT));

        assert_eq!(player.active_animation_id(), animation_id);
    }

    #[test]
    pub fn test_load_animation() {
        let player = DotLottiePlayer::new(Config::default());

        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];

        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );

        assert!(player.load_dotlottie_data(include_bytes!("fixtures/emoji.lottie"), WIDTH, HEIGHT));

        let manifest = player.manifest();

        assert!(manifest.is_some(), "Manifest is not loaded");

        let manifest = manifest.unwrap();

        let animations = manifest.animations;

        assert!(
            animations[0].id == player.active_animation_id(),
            "Active animation id is not the first animation id"
        );

        for animation in animations {
            assert!(
                player.load_animation(&animation.id, WIDTH, HEIGHT),
                "Failed to load animation with id {}",
                animation.id
            );

            let active_animation_id = player.active_animation_id();

            assert_eq!(
                active_animation_id, animation.id,
                "Active animation id is not equal to the loaded animation id"
            );
        }

        assert!(
            !player.load_animation("invalid_id", WIDTH, HEIGHT),
            "Loaded animation with invalid id"
        );

        let active_action_id = player.active_animation_id();

        assert!(
            active_action_id.is_empty(),
            "Active animation id is not empty"
        );
    }
}
