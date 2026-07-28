use dotlottie_rs::{ColorSpace, Player};
use std::ffi::CString;

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: loading renders internally (load_animation_common), so `render()`
    // right after load correctly errors with "nothing to render". Assert on
    // buffer contents instead. Frame 21 is mid-animation, where the shape
    // layers are visible (frame 0 is a solid white fill, useless for pixel
    // comparisons).
    fn loaded_player(buffer: &mut [u32]) -> Player {
        let mut player = Player::new();
        assert!(player
            .set_sw_target(buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .is_ok());
        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());
        let distinct = buffer.iter().collect::<std::collections::HashSet<_>>();
        assert!(distinct.len() > 1, "frame 21 should have visual variety");
        player
    }

    #[test]
    fn opacity_and_effects_change_pixels() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let baseline = buffer.clone();
        assert!(
            baseline.iter().any(|&px| px != 0),
            "load's internal render drew nothing through the scene wrap"
        );

        assert!(player.set_opacity(60).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        assert!(player.set_opacity(255).is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        assert!(player.add_gaussian_blur(8.0, 0, 0, 100).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        assert!(player.clear_effects().is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        // The white solid fills the whole canvas, hiding a drop shadow cast
        // behind it — shrink the content first so the shadow lands on empty
        // canvas.
        let shrink = vec![0.6, 0.0, 20.0, 0.0, 0.6, 20.0, 0.0, 0.0, 1.0];
        assert!(player.set_transform(shrink).is_ok());
        assert!(player.render().is_ok());
        let shrunk = buffer.clone();
        assert_ne!(shrunk, baseline);

        assert!(player.add_drop_shadow(0, 0, 0, 200, 45.0, 12.0, 6.0, 100).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, shrunk);

        assert!(player.clear_effects().is_ok());
        assert!(player
            .add_tritone(20, 20, 20, 128, 128, 128, 235, 235, 235, 0)
            .is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, shrunk);

        assert!(player.set_blend_mode(1).is_ok());
        assert!(player.render().is_ok());
    }

    #[test]
    fn layer_props_compose_and_restore() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);
        let baseline = buffer.clone();
        assert!(baseline.iter().any(|&px| px != 0));

        let shift = vec![1.0, 0.0, 200.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        assert!(player.set_layer_transform("R", shift).is_ok());
        assert!(player.render().is_ok());
        assert_ne!(buffer, baseline);

        assert!(player.set_layer_opacity("E", 128).is_ok());
        assert!(player.render().is_ok());

        // Re-apply must survive a frame change (ThorVG rebuilds layer paints).
        assert!(player.set_frame(22.0).is_ok());
        assert!(player.render().is_ok());
        let shifted_frame22 = buffer.clone();
        assert!(player.set_frame(21.0).is_ok());
        assert!(player.render().is_ok());

        assert!(player.clear_layer_props("R").is_ok());
        assert!(player.clear_layer_props("E").is_ok());
        assert!(player.render().is_ok());
        assert_eq!(buffer, baseline);

        // Unknown layer name is a no-op, not an error.
        assert!(player.set_layer_opacity("no-such-layer", 0).is_ok());
        assert!(player.render().is_ok());

        let _ = shifted_frame22;
    }

    #[test]
    fn reload_cycle_survives_scene_wrap() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = loaded_player(&mut buffer);

        assert!(player.set_opacity(128).is_ok());
        assert!(player.add_gaussian_blur(4.0, 0, 0, 50).is_ok());
        assert!(player.render().is_ok());

        // Reload drops the wrapped TvgAnimation while the canvas holds the scene.
        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());

        let path2 = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path2).is_ok());
        assert!(buffer.iter().any(|&px| px != 0));
    }
}
