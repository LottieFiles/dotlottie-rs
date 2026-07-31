#![cfg(feature = "state-machines")]

// Headless validation of the PURE state-machine drag & drop
// (assets/statemachines/drag_drop_pure.json): the host supplies only
// pointer events and cursor position inputs — pickup hit-testing, drop-zone
// matching, docking persistence, and glide tweens are all authored in the
// state machine.

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::events::Event;
    use dotlottie_rs::{ColorSpace, Player, StateMachineEngineStatus};

    fn pos_of(player: &Player, slot_id: &str) -> [f32; 2] {
        let json = player.get_slot_str(slot_id);
        let v: serde_json::Value = serde_json::from_str(&json).expect("slot json");
        let k = v["k"].as_array().expect("k array");
        [k[0].as_f64().unwrap() as f32, k[1].as_f64().unwrap() as f32]
    }

    #[test]
    fn full_drag_and_drop_cycle_pure_state_machine() {
        let definition = include_str!("../assets/statemachines/drag_drop_pure.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/drag_drop.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        // Render once so hit-testing has a rendered scene.
        sm.tick(16.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "idle");

        // ── Pick up the circle (authored rest [64.5, 299.5]) ────────────
        // Engine-side hit test on the FILLED draggable layer.
        sm.set_numeric_input("cursor_x", 64.5, true, false).unwrap();
        sm.set_numeric_input("cursor_y", 299.5, true, false)
            .unwrap();
        sm.post_event(&Event::PointerDown { x: 64.5, y: 299.5 });
        assert_eq!(
            sm.get_current_state_name(),
            "dragging_circle",
            "PointerDown on the circle layer should grab it"
        );

        // ── Drag toward the circle_drop zone [57.5, 77.5] ───────────────
        sm.set_numeric_input("cursor_x", 60.0, true, false).unwrap();
        sm.set_numeric_input("cursor_y", 150.0, true, false)
            .unwrap();
        assert_eq!(pos_of(sm.player, "circle_pos"), [60.0, 150.0]);

        sm.set_numeric_input("cursor_x", 57.5, true, false).unwrap();
        sm.set_numeric_input("cursor_y", 77.5, true, false).unwrap();

        // ── Release over the STROKE-ONLY drop zone layer ─────────────────
        // This is the hit-test probe: does picking register the interior
        // of a fill-less outline shape?
        sm.post_event(&Event::PointerUp { x: 57.5, y: 77.5 });

        let state = sm.get_current_state_name();
        assert!(
            sm.status == StateMachineEngineStatus::Tweening,
            "release should start a docking/return tween (state: {state})"
        );

        // Finish the 0.25s glide; cursor keeps feeding (host behavior) so
        // the pipeline keeps ticking after the tween resumes.
        sm.tick(300.0).unwrap();
        sm.set_numeric_input("cursor_x", 57.5, true, false).unwrap();

        assert_eq!(
            sm.get_current_state_name(),
            "idle",
            "docking is transient and should fall through to idle"
        );
        assert_eq!(
            pos_of(sm.player, "circle_pos"),
            [57.5, 77.5],
            "circle should PERSISTENTLY rest at its dock"
        );
        assert_eq!(
            sm.get_numeric_input("circle_x"),
            Some(57.5),
            "dock coords should be baked into the rest inputs"
        );

        // ── Failed drop: grab the square, release in empty space ────────
        sm.set_numeric_input("cursor_x", 381.5, true, false)
            .unwrap();
        sm.set_numeric_input("cursor_y", 402.5, true, false)
            .unwrap();
        sm.post_event(&Event::PointerDown { x: 381.5, y: 402.5 });
        assert_eq!(sm.get_current_state_name(), "dragging_square");

        sm.set_numeric_input("cursor_x", 250.0, true, false)
            .unwrap();
        sm.set_numeric_input("cursor_y", 250.0, true, false)
            .unwrap();
        sm.post_event(&Event::PointerUp { x: 250.0, y: 250.0 });
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        sm.tick(300.0).unwrap();
        sm.set_numeric_input("cursor_x", 250.0, true, false)
            .unwrap();

        assert_eq!(sm.get_current_state_name(), "idle");
        assert_eq!(
            pos_of(sm.player, "square_pos"),
            [381.5, 402.5],
            "failed drop should return the square to its authored rest"
        );

        // The docked circle must be untouched by the square's drag cycle.
        assert_eq!(pos_of(sm.player, "circle_pos"), [57.5, 77.5]);
    }

    #[test]
    fn single_object_star_drop_cycle() {
        let definition = include_str!("../assets/statemachines/star_drop.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/star_drop_static.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "idle");
        assert_eq!(pos_of(sm.player, "star_pos"), [138.5, 392.5]);

        // Failed drop first: grab the star, release in empty space.
        sm.set_numeric_input("cursor_x", 138.5, true, false)
            .unwrap();
        sm.set_numeric_input("cursor_y", 392.5, true, false)
            .unwrap();
        sm.post_event(&Event::PointerDown { x: 138.5, y: 392.5 });
        assert_eq!(sm.get_current_state_name(), "dragging");

        sm.set_numeric_input("cursor_x", 400.0, true, false)
            .unwrap();
        sm.set_numeric_input("cursor_y", 400.0, true, false)
            .unwrap();
        sm.post_event(&Event::PointerUp { x: 400.0, y: 400.0 });
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);
        sm.tick(300.0).unwrap();
        sm.set_numeric_input("cursor_x", 400.0, true, false)
            .unwrap();
        assert_eq!(sm.get_current_state_name(), "idle");
        assert_eq!(pos_of(sm.player, "star_pos"), [138.5, 392.5]);

        // Successful drop: release over the drop_zone layer [159.5, 110.5].
        sm.post_event(&Event::PointerDown { x: 138.5, y: 392.5 });
        assert_eq!(sm.get_current_state_name(), "dragging");
        sm.set_numeric_input("cursor_x", 159.5, true, false)
            .unwrap();
        sm.set_numeric_input("cursor_y", 110.5, true, false)
            .unwrap();
        sm.post_event(&Event::PointerUp { x: 159.5, y: 110.5 });
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);
        sm.tick(300.0).unwrap();
        sm.set_numeric_input("cursor_x", 159.5, true, false)
            .unwrap();

        assert_eq!(sm.get_current_state_name(), "idle");
        assert_eq!(pos_of(sm.player, "star_pos"), [159.5, 110.5]);
        assert_eq!(sm.get_numeric_input("star_x"), Some(159.5));
    }
}
