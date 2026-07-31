#![cfg(feature = "state-machines")]

//! Integration tests for the state-declared slots prototype
//! (docs/spec-updates/state-slots.md).
//!
//! Uses bouncy_ball.json, whose authored slots are:
//! - ball_color:   static Color [0.71, 0.192, 0.278]
//! - ball_opacity: static Scalar 100
//! - ball_position, ball_scale: keyframed (non-static) values

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::{
        actions::open_url_policy::OpenUrlPolicy, ColorSpace, Player, StateMachineEngineStatus,
    };

    const AUTHORED_COLOR: [f32; 3] = [0.71, 0.192, 0.278];
    const AUTHORED_OPACITY: f32 = 100.0;

    fn setup_player(buffer: &mut Vec<u32>) -> Player {
        let mut player = Player::new();
        player
            .set_sw_target(buffer, 100, 100, ColorSpace::ABGR8888)
            .expect("set_sw_target should succeed");

        let path = CString::new("assets/animations/lottie/bouncy_ball.json").unwrap();
        player
            .load_animation_path(&path)
            .expect("animation should load");
        player
    }

    fn slot_k(player: &Player, slot_id: &str) -> serde_json::Value {
        let json = player.get_slot_str(slot_id);
        assert!(!json.is_empty(), "slot '{slot_id}' should be tracked");
        serde_json::from_str::<serde_json::Value>(&json).expect("slot json should parse")["k"]
            .clone()
    }

    fn assert_color(player: &Player, slot_id: &str, expected: [f32; 3]) {
        let k = slot_k(player, slot_id);
        let arr = k.as_array().expect("color k should be an array");
        for (i, e) in expected.iter().enumerate() {
            let v = arr[i].as_f64().unwrap() as f32;
            assert!((v - e).abs() < 1e-3, "{slot_id}[{i}] expected {e}, got {v}");
        }
    }

    fn assert_scalar(player: &Player, slot_id: &str, expected: f32) {
        let k = slot_k(player, slot_id);
        let v = k.as_f64().expect("scalar k should be a number") as f32;
        assert!(
            (v - expected).abs() < 1e-3,
            "{slot_id} expected {expected}, got {v}"
        );
    }

    #[test]
    fn instant_apply_on_entry() {
        let definition = include_str!("../assets/statemachines/state_slot_tests/state_slots.json");
        let mut buffer: Vec<u32> = vec![0; 100 * 100];
        let mut player = setup_player(&mut buffer);
        let mut sm = player
            .state_machine_load_data(definition)
            .expect("state machine should load");
        sm.start(&OpenUrlPolicy::default()).unwrap();

        // Initial state declares nothing: authored values are tracked.
        assert_eq!(sm.get_current_state_name(), "a");
        assert_color(sm.player, "ball_color", AUTHORED_COLOR);
        assert_scalar(sm.player, "ball_opacity", AUTHORED_OPACITY);

        // Entering b applies its declared slots; "$opacity" resolves to the
        // opacity input (42).
        sm.set_numeric_input("trigger", 1.0, true, false).unwrap();
        assert_eq!(sm.get_current_state_name(), "b");
        assert_color(sm.player, "ball_color", [1.0, 0.0, 0.0]);
        assert_scalar(sm.player, "ball_opacity", 42.0);
    }

    #[test]
    fn redeclared_slot_keeps_base_and_partial_release_restores() {
        let definition = include_str!("../assets/statemachines/state_slot_tests/state_slots.json");
        let mut buffer: Vec<u32> = vec![0; 100 * 100];
        let mut player = setup_player(&mut buffer);
        let mut sm = player
            .state_machine_load_data(definition)
            .expect("state machine should load");
        sm.start(&OpenUrlPolicy::default()).unwrap();

        sm.set_numeric_input("trigger", 1.0, true, false).unwrap();
        assert_eq!(sm.get_current_state_name(), "b");

        // b -> c: c redeclares ball_color (green) but not ball_opacity, so
        // opacity releases back to its authored value.
        sm.set_numeric_input("trigger", 2.0, true, false).unwrap();
        assert_eq!(sm.get_current_state_name(), "c");
        assert_color(sm.player, "ball_color", [0.0, 1.0, 0.0]);
        assert_scalar(sm.player, "ball_opacity", AUTHORED_OPACITY);

        // c -> d: nothing declared, ball_color releases to the ORIGINAL
        // authored base (saved when b first covered it, preserved through
        // c's redeclaration).
        sm.set_numeric_input("trigger", 3.0, true, false).unwrap();
        assert_eq!(sm.get_current_state_name(), "d");
        assert_color(sm.player, "ball_color", AUTHORED_COLOR);
        assert_scalar(sm.player, "ball_opacity", AUTHORED_OPACITY);
    }

    #[test]
    fn live_binding_reapplies_on_input_change() {
        let definition = include_str!("../assets/statemachines/state_slot_tests/binding.json");
        let mut buffer: Vec<u32> = vec![0; 100 * 100];
        let mut player = setup_player(&mut buffer);
        let mut sm = player
            .state_machine_load_data(definition)
            .expect("state machine should load");
        sm.start(&OpenUrlPolicy::default()).unwrap();

        sm.set_numeric_input("trigger", 1.0, true, false).unwrap();
        assert_eq!(sm.get_current_state_name(), "bound");

        // Refs sampled at entry: r=0, level=30.
        assert_color(sm.player, "ball_color", [0.0, 0.0, 0.0]);
        assert_scalar(sm.player, "ball_opacity", 30.0);

        // Standing binding: changing referenced inputs re-applies the slots
        // without any state change.
        sm.set_numeric_input("r", 0.75, true, false).unwrap();
        assert_eq!(sm.get_current_state_name(), "bound");
        assert_color(sm.player, "ball_color", [0.75, 0.0, 0.0]);

        sm.set_numeric_input("level", 80.0, true, false).unwrap();
        assert_scalar(sm.player, "ball_opacity", 80.0);

        // Leaving the state releases the slots AND kills the binding.
        sm.set_numeric_input("trigger", 0.0, true, false).unwrap();
        assert_eq!(sm.get_current_state_name(), "a");
        assert_color(sm.player, "ball_color", AUTHORED_COLOR);
        assert_scalar(sm.player, "ball_opacity", AUTHORED_OPACITY);

        sm.set_numeric_input("r", 0.2, true, false).unwrap();
        assert_color(sm.player, "ball_color", AUTHORED_COLOR);
    }

    #[test]
    fn tweened_transition_interpolates_slots() {
        let definition = include_str!("../assets/statemachines/state_slot_tests/tween.json");
        let mut buffer: Vec<u32> = vec![0; 100 * 100];
        let mut player = setup_player(&mut buffer);
        let mut sm = player
            .state_machine_load_data(definition)
            .expect("state machine should load");
        sm.start(&OpenUrlPolicy::default()).unwrap();
        assert_eq!(sm.get_current_state_name(), "a");

        // Trigger the Tweened transition (1s, linear easing) into b.
        sm.set_numeric_input("trigger", 1.0, true, false).unwrap();
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);
        // Nothing applied yet at progress 0.
        assert_color(sm.player, "ball_color", AUTHORED_COLOR);
        assert_scalar(sm.player, "ball_opacity", AUTHORED_OPACITY);

        // Halfway: values are midway between authored and declared.
        sm.tick(500.0).unwrap();
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);
        assert_color(
            sm.player,
            "ball_color",
            [
                (AUTHORED_COLOR[0] + 1.0) / 2.0,
                AUTHORED_COLOR[1] / 2.0,
                AUTHORED_COLOR[2] / 2.0,
            ],
        );
        assert_scalar(sm.player, "ball_opacity", (AUTHORED_OPACITY + 10.0) / 2.0);

        // Completion: exact declared values, state entered.
        sm.tick(600.0).unwrap();
        assert_eq!(sm.status, StateMachineEngineStatus::Running);
        assert_eq!(sm.get_current_state_name(), "b");
        assert_color(sm.player, "ball_color", [1.0, 0.0, 0.0]);
        assert_scalar(sm.player, "ball_opacity", 10.0);
    }

    #[test]
    fn tweened_exit_tweens_back_to_base() {
        let definition = include_str!("../assets/statemachines/state_slot_tests/tween.json");
        let mut buffer: Vec<u32> = vec![0; 100 * 100];
        let mut player = setup_player(&mut buffer);
        let mut sm = player
            .state_machine_load_data(definition)
            .expect("state machine should load");
        sm.start(&OpenUrlPolicy::default()).unwrap();

        // Get fully into b first.
        sm.set_numeric_input("trigger", 1.0, true, false).unwrap();
        sm.tick(1100.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "b");
        assert_color(sm.player, "ball_color", [1.0, 0.0, 0.0]);

        // b -> c is Tweened and c declares nothing: released slots tween
        // BACK to their authored base values.
        sm.set_numeric_input("trigger", 2.0, true, false).unwrap();
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        sm.tick(500.0).unwrap();
        assert_color(
            sm.player,
            "ball_color",
            [
                (1.0 + AUTHORED_COLOR[0]) / 2.0,
                AUTHORED_COLOR[1] / 2.0,
                AUTHORED_COLOR[2] / 2.0,
            ],
        );
        assert_scalar(sm.player, "ball_opacity", (10.0 + AUTHORED_OPACITY) / 2.0);

        sm.tick(600.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "c");
        assert_color(sm.player, "ball_color", AUTHORED_COLOR);
        assert_scalar(sm.player, "ball_opacity", AUTHORED_OPACITY);
    }

    #[test]
    fn invalid_entries_are_silent_no_ops() {
        let definition = include_str!("../assets/statemachines/state_slot_tests/invalid.json");
        let mut buffer: Vec<u32> = vec![0; 100 * 100];
        let mut player = setup_player(&mut buffer);
        let mut sm = player
            .state_machine_load_data(definition)
            .expect("state machine should load");
        sm.start(&OpenUrlPolicy::default()).unwrap();

        // Unknown slot id, type mismatch, unresolvable ref, wrong arity, and
        // a global-input ref: all no-ops, nothing crashes, values untouched.
        sm.set_numeric_input("trigger", 1.0, true, false).unwrap();
        assert_eq!(sm.get_current_state_name(), "broken");
        assert_color(sm.player, "ball_color", AUTHORED_COLOR);
        assert_scalar(sm.player, "ball_opacity", AUTHORED_OPACITY);
    }
}
