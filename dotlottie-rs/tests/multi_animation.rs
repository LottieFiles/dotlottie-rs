use std::ffi::CString;

use dotlottie_rs::DotLottiePlayer;

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_load_animation_with_animation_id() {
        let animation_id = CString::new("crying").unwrap();

        let mut player = DotLottiePlayer::new(0);

        // First load the dotlottie, then load the specific animation
        assert!(player
            .load_dotlottie_data(
                include_bytes!("../assets/animations/dotlottie/v1/emojis.lottie"),
                WIDTH,
                HEIGHT
            )
            .is_ok(),);

        assert_eq!(player.load_animation(&animation_id, WIDTH, HEIGHT), Ok(()));

        assert_eq!(player.active_animation_id(), Some(animation_id.as_c_str()));
    }

    #[test]
    pub fn test_load_animation() {
        let mut player = DotLottiePlayer::new(0);
        assert!(player
            .load_dotlottie_data(
                include_bytes!("../assets/animations/dotlottie/v1/emojis.lottie"),
                WIDTH,
                HEIGHT
            )
            .is_ok());

        let manifest = player.manifest();

        assert!(manifest.is_some(), "Manifest is not loaded");

        let manifest = manifest.unwrap();

        let animations = manifest.animations.clone();

        let first_id = CString::new(animations[0].id.clone()).unwrap();
        assert_eq!(
            player.active_animation_id(),
            Some(first_id.as_c_str()),
            "Active animation id is not the first animation id"
        );

        for animation in &animations {
            let anim_id = CString::new(animation.id.clone()).unwrap();
            assert_eq!(
                player.load_animation(&anim_id, WIDTH, HEIGHT),
                Ok(()),
                "Failed to load animation with id {}",
                animation.id
            );

            let active_animation_id = player.active_animation_id();

            assert_eq!(
                active_animation_id,
                Some(anim_id.as_c_str()),
                "Active animation id is not equal to the loaded animation id"
            );
        }

        let invalid_id = CString::new("invalid_id").unwrap();
        assert_ne!(
            player.load_animation(&invalid_id, WIDTH, HEIGHT),
            Ok(()),
            "Loaded animation with invalid id"
        );

        let active_animation_id = player.active_animation_id();

        assert!(
            active_animation_id.is_none(),
            "Active animation id should be None after invalid load"
        );
    }
}
