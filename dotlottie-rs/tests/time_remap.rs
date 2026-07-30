use dotlottie_rs::{ColorSpace, Player, ScalarSlot};
use std::ffi::CString;

const WIDTH: u32 = 300;
const HEIGHT: u32 = 300;

/// Outer frame both remap comparisons render at, so the only variable is the slot.
const OUTER_FRAME: f32 = 30.0;

#[cfg(test)]
mod tests {
    use super::*;

    fn load_player(buffer: &mut Vec<u32>) -> Player {
        let mut player = Player::new();
        player
            .set_sw_target(buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
            .unwrap();

        let data = include_str!("../assets/animations/lottie/time_remap.json");
        let c_data = CString::new(data).unwrap();
        player.load_animation_data(&c_data).unwrap();
        player
    }

    #[test]
    fn time_remap_slot_is_discovered() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let player = load_player(&mut buffer);

        let ids = player.get_slot_ids();
        assert!(
            ids.contains(&"inner_time".to_string()),
            "expected the tm slot to be discovered, got {ids:?}"
        );
    }

    #[test]
    fn time_remap_slot_is_typed_as_scalar() {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let player = load_player(&mut buffer);

        assert_eq!(player.get_slot_type("inner_time"), "scalar");
    }

    /// Renders a fixed outer frame with the remap slot pinned to `inner_time` seconds.
    ///
    /// The outer frame is deliberately non-zero: a freshly loaded player already sits on
    /// frame 0, and ThorVG reports an unchanged frame as `InsufficientCondition`.
    fn render_at_inner_time(inner_time: f32) -> Vec<u32> {
        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
        let mut player = load_player(&mut buffer);

        player.set_frame(OUTER_FRAME).unwrap();
        player
            .set_scalar_slot("inner_time", ScalarSlot::new(inner_time))
            .unwrap();
        player.render().unwrap();

        buffer.clone()
    }

    #[test]
    fn same_inner_time_renders_identically() {
        assert_eq!(
            render_at_inner_time(0.0),
            render_at_inner_time(0.0),
            "rendering the same inner time twice must be deterministic"
        );
    }

    #[test]
    fn different_inner_time_renders_differently() {
        let at_start = render_at_inner_time(0.0);
        let at_half = render_at_inner_time(1.5);

        let differing = at_start
            .iter()
            .zip(at_half.iter())
            .filter(|(a, b)| a != b)
            .count();

        assert!(
            differing > 100,
            "expected the remap slot to move the precomp content at a fixed outer frame, \
             but only {differing} pixels differed"
        );
    }
}
