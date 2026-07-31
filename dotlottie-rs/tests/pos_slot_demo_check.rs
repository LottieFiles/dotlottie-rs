#![cfg(feature = "state-machines")]

// Throwaway verification for assets/statemachines/pos_slot_tween.json
// against assets/animations/lottie/tween.json (the state machine's segment
// names, "three"/"circle", are markers of tween.json — with an animation
// lacking those markers the Tweened transitions degrade to instant).

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::{
        actions::open_url_policy::OpenUrlPolicy, ColorSpace, Player, StateMachineEngineStatus,
    };

    fn pos_of(player: &Player) -> [f32; 2] {
        let json = player.get_slot_str("pos");
        let v: serde_json::Value = serde_json::from_str(&json).expect("slot json");
        let k = v["k"].as_array().expect("k array");
        [k[0].as_f64().unwrap() as f32, k[1].as_f64().unwrap() as f32]
    }

    #[test]
    fn pos_slot_demo_tweens_between_states() {
        let definition = include_str!("../assets/statemachines/pos_slot_tween.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/tween.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();

        // Initial state applies instantly (entered without a transition).
        assert_eq!(sm.get_current_state_name(), "left");
        assert_eq!(pos_of(sm.player), [80.0, 256.0]);

        // left -> right: 1s ease-in-out tween.
        sm.set_numeric_input("side", 1.0, true, false).unwrap();
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        // Halfway through an [0.42, 0, 0.58, 1] ease the eased progress is
        // exactly 0.5 (symmetric curve): midpoint of 80..432 is 256.
        sm.tick(500.0).unwrap();
        let mid = pos_of(sm.player);
        assert!(
            (mid[0] - 256.0).abs() < 1.0 && (mid[1] - 256.0).abs() < 0.01,
            "midpoint should be ~[256, 256], got {mid:?}"
        );

        sm.tick(600.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "right");
        assert_eq!(pos_of(sm.player), [432.0, 256.0]);

        // And back.
        sm.set_numeric_input("side", 0.0, true, false).unwrap();
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);
        sm.tick(1100.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "left");
        assert_eq!(pos_of(sm.player), [80.0, 256.0]);
    }
}
