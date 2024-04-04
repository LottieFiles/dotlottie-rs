use dotlottie_player_core::{Config, DotLottiePlayer};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_load_animation() {
        let player = DotLottiePlayer::new(Config::default());
        assert!(player.load_dotlottie_data(include_bytes!("assets/emoji.lottie"), WIDTH, HEIGHT));

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

            /*
               TODO: assert if the currently loaded animation is the same as the one we loaded
               require the player to have a method to get the currently loaded animation

                let current_animation = player.current_animation();
                assert_eq!(current_animation.id, animation.id);
            */
        }
    }
}
