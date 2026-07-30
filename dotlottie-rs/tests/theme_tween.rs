use std::collections::BTreeMap;
use std::ffi::CString;

use dotlottie_rs::{ColorSpace, Player, SlotType, VectorSlot};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

const LINEAR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

fn setup_multi_themes() -> (Player, Vec<u32>) {
    let mut player = Player::new();
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .expect("set_sw_target should succeed");

    player
        .load_dotlottie_data(include_bytes!(
            "../assets/animations/dotlottie/v2/multi_themes.lottie"
        ))
        .expect("multi_themes.lottie should load");

    (player, buffer)
}

fn setup_joystick() -> (Player, Vec<u32>) {
    let mut player = Player::new();
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .expect("set_sw_target should succeed");

    let path = CString::new("assets/animations/lottie/joystick.json").unwrap();
    player
        .load_animation_path(&path)
        .expect("joystick.json should load");

    (player, buffer)
}

/// Extracts the `[r, g, b]` floats out of a static color slot's JSON
/// (`{"a":0,"k":[r,g,b]}`). Only valid for non-animated (static) properties.
fn extract_rgb(json: &str) -> [f32; 3] {
    let start = json.find("\"k\":[").expect("static \"k\" array") + 5;
    let end = start + json[start..].find(']').expect("closing bracket");
    let nums: Vec<f32> = json[start..end]
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();
    [nums[0], nums[1], nums[2]]
}

fn extract_xy(json: &str) -> [f32; 2] {
    let start = json.find("\"k\":[").expect("static \"k\" array") + 5;
    let end = start + json[start..].find(']').expect("closing bracket");
    let nums: Vec<f32> = json[start..end]
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();
    [nums[0], nums[1]]
}

fn assert_close(a: [f32; 3], b: [f32; 3], tol: f32) {
    for i in 0..3 {
        assert!(
            (a[i] - b[i]).abs() < tol,
            "component {i}: {} not within {tol} of {}",
            a[i],
            b[i]
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tween_lerps_from_dark_to_light_and_completes_exactly() {
        let (mut player, _buf) = setup_multi_themes();

        let dark = CString::new("dark").unwrap();
        let light = CString::new("light").unwrap();

        player.set_theme(&dark).expect("dark theme should apply");
        assert_close(
            extract_rgb(&player.get_slot_str("bg_color")),
            [0.0, 0.0, 0.0],
            1e-4,
        );

        player
            .set_theme_tweened(&light, 1000.0, LINEAR)
            .expect("tween to light should start");

        // Halfway through a linear 1000ms tween from black to light's bg_color.
        player.tick(500.0).expect("tick should succeed");
        let mid = extract_rgb(&player.get_slot_str("bg_color"));
        assert_close(mid, [0.5, 0.4098, 0.11375], 1e-3);

        // Finish the tween.
        player.tick(600.0).expect("tick should succeed");
        let end = extract_rgb(&player.get_slot_str("bg_color"));
        assert_close(end, [1.0, 0.8196, 0.2275], 1e-3);
    }

    #[test]
    fn tween_to_animated_theme_snaps_instantly() {
        let (mut player, _buf) = setup_multi_themes();

        let dark = CString::new("dark").unwrap();
        let animated_dark = CString::new("animated_dark").unwrap();

        player.set_theme(&dark).expect("dark theme should apply");

        player
            .set_theme_tweened(&animated_dark, 1000.0, LINEAR)
            .expect("tween to animated_dark should start");

        // The target's bg_color is keyframed (Animated), so it can't be smoothly
        // interpolated and must be applied immediately, before any tick.
        let snapped = player.get_slot_str("bg_color");
        assert!(
            snapped.contains("\"a\":1"),
            "expected an animated (keyframed) slot to be applied instantly, got {snapped}"
        );
    }

    #[test]
    fn retargeting_mid_tween_continues_smoothly_without_a_jump() {
        let (mut player, _buf) = setup_multi_themes();

        let dark = CString::new("dark").unwrap();
        let light = CString::new("light").unwrap();

        player.set_theme(&dark).expect("dark theme should apply");
        player
            .set_theme_tweened(&light, 1000.0, LINEAR)
            .expect("tween to light should start");

        player.tick(400.0).expect("tick should succeed");
        let mid = extract_rgb(&player.get_slot_str("bg_color"));

        // Retarget back toward dark before the light tween completes.
        player
            .set_theme_tweened(&dark, 1000.0, LINEAR)
            .expect("retargeted tween should start");

        // At progress 0 of the new tween, the value must still be exactly where the
        // previous tween left off — no jump back to the original dark value.
        let just_after_retarget = extract_rgb(&player.get_slot_str("bg_color"));
        assert_close(just_after_retarget, mid, 1e-4);
    }

    #[test]
    fn duration_zero_behaves_like_the_instant_method() {
        let (mut instant_player, _buf1) = setup_multi_themes();
        let (mut tweened_player, _buf2) = setup_multi_themes();

        let light = CString::new("light").unwrap();

        instant_player
            .set_theme(&light)
            .expect("instant set_theme should succeed");
        tweened_player
            .set_theme_tweened(&light, 0.0, LINEAR)
            .expect("duration <= 0 should behave like set_theme");

        assert_eq!(
            instant_player.get_slot_str("bg_color"),
            tweened_player.get_slot_str("bg_color")
        );

        // No tween should be left running: a subsequent tick must not change the value.
        let before = tweened_player.get_slot_str("bg_color");
        tweened_player.tick(16.0).expect("tick should succeed");
        assert_eq!(tweened_player.get_slot_str("bg_color"), before);
    }

    #[test]
    fn set_slots_tweened_lerps_vector_slots() {
        let (mut player, _buf) = setup_joystick();

        // These slots come from the animation's own declared `slots` block with plain
        // 2-element arrays, which the parser treats as `Vector` (not `Position`) absent an
        // explicit type hint — matching type is required for the lerp to kick in.
        let start = extract_xy(&player.get_slot_str("triangle_pos"));

        let mut slots = BTreeMap::new();
        slots.insert(
            "triangle_pos".to_string(),
            SlotType::Vector(VectorSlot::static_value([
                start[0] + 100.0,
                start[1] + 200.0,
            ])),
        );

        player
            .set_slots_tweened(slots, 1000.0, LINEAR)
            .expect("position tween should start");

        player.tick(500.0).expect("tick should succeed");
        let mid = extract_xy(&player.get_slot_str("triangle_pos"));
        assert!(
            (mid[0] - (start[0] + 50.0)).abs() < 1e-2,
            "x should be halfway, got {mid:?}"
        );
        assert!(
            (mid[1] - (start[1] + 100.0)).abs() < 1e-2,
            "y should be halfway, got {mid:?}"
        );

        player.tick(600.0).expect("tick should succeed");
        let end = extract_xy(&player.get_slot_str("triangle_pos"));
        assert!((end[0] - (start[0] + 100.0)).abs() < 1e-2);
        assert!((end[1] - (start[1] + 200.0)).abs() < 1e-2);
    }

    #[test]
    fn tween_progresses_while_animation_is_paused() {
        let (mut player, _buf) = setup_multi_themes();

        let dark = CString::new("dark").unwrap();
        let light = CString::new("light").unwrap();

        // The player is never played, so `current_frame` never changes across ticks.
        player.set_theme(&dark).expect("dark theme should apply");
        let start = extract_rgb(&player.get_slot_str("bg_color"));

        player
            .set_theme_tweened(&light, 1000.0, LINEAR)
            .expect("tween to light should start");

        let rendered = player.tick(500.0).expect("tick should succeed");
        assert!(
            rendered,
            "tick should report a render even though the frame is unchanged"
        );

        let mid = extract_rgb(&player.get_slot_str("bg_color"));
        assert_ne!(
            mid, start,
            "slot value should have progressed even while paused"
        );
    }
}
