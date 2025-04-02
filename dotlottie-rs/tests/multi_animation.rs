use dotlottie_rs::Config;

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_load_animation() {
        let player = crate::test_utils::create_test_player(Config::default());
        assert!(player.load_dotlottie_data(include_bytes!("fixtures/emoji.lottie"), WIDTH, HEIGHT));

        let manifest = player.manifest();

        assert!(manifest.is_some(), "Manifest is not loaded");

        let manifest = manifest.unwrap();

        let animations = manifest.animations;

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
