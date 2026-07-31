#![cfg(feature = "state-machines")]

// Validation of the dedicated DragAndDrop interaction prototype
// (assets/statemachines/star_drop_dnd.json): the entire gesture — pickup
// hit-test, grab offset, drag, zone hit-test, snap tween, lock, actions —
// is owned by the interaction. The host posts pointer events only; there
// are no cursor inputs at all.

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::events::Event;
    use dotlottie_rs::{ColorSpace, Player, StateMachineEngineStatus};

    fn pos_of(player: &Player) -> [f32; 2] {
        let json = player.get_slot_str("star_pos");
        let v: serde_json::Value = serde_json::from_str(&json).expect("slot json");
        let k = v["k"].as_array().expect("k array");
        [k[0].as_f64().unwrap() as f32, k[1].as_f64().unwrap() as f32]
    }

    /// A layer's current transform position (matrix translation) — the
    /// quantity the engine derives snap targets from.
    fn position_of(player: &Player, layer: &str) -> [f32; 2] {
        let m = player.layer_transform(layer).expect("layer transform");
        [m[2], m[5]]
    }

    fn assert_near(actual: [f32; 2], expected: [f32; 2], msg: &str) {
        assert!(
            (actual[0] - expected[0]).abs() < 0.5 && (actual[1] - expected[1]).abs() < 0.5,
            "{msg}: expected ~{expected:?}, got {actual:?}"
        );
    }

    #[test]
    fn drag_and_drop_interaction_full_cycle() {
        let definition = include_str!("../assets/statemachines/star_drop_dnd.json");
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
        assert_eq!(pos_of(sm.player), [138.5, 392.5]);
        sm.tick(16.0).unwrap(); // bounds lag the scene by one canvas update

        // The dock target is the zone's TRANSFORM position, read from the
        // rendered scene at drop time (matrix translation — the exact
        // authored semantic, not a bounding-box approximation).
        let dock = position_of(sm.player, "drop_zone");
        assert_eq!(dock, [159.5, 110.5], "zone's authored transform position");

        // ── Grab off-center: the pointer-to-object offset is always
        // preserved — grabbing at (150, 400) keeps the (-11.5, -7.5)
        // offset for the whole drag, so the star never jumps to center
        // itself on the pointer.
        sm.post_event(&Event::PointerDown { x: 150.0, y: 400.0 });
        sm.post_event(&Event::PointerMove { x: 300.0, y: 300.0 });
        assert_eq!(pos_of(sm.player), [288.5, 292.5]);

        // ── Miss: release in empty space → snap-back tween ──────────────
        sm.post_event(&Event::PointerUp { x: 300.0, y: 300.0 });
        // The snap tween is NON-blocking: the engine stays Running.
        assert_eq!(sm.status, StateMachineEngineStatus::Running);

        sm.tick(125.0).unwrap();
        let mid = pos_of(sm.player);
        assert!(
            mid != [288.5, 292.5] && mid != [138.5, 392.5],
            "mid-snap position should be between release and rest, got {mid:?}"
        );

        sm.tick(200.0).unwrap();
        assert_eq!(
            pos_of(sm.player),
            [138.5, 392.5],
            "missed drop returns to rest"
        );
        assert_eq!(sm.get_numeric_input("docked_count"), Some(0.0));
        // Settle: the SHAPE-accurate pickup test reads the scene one
        // canvas update behind, so re-grabbing needs the glide's final
        // frame to be visible.
        sm.tick(16.0).unwrap();

        // ── Dock: release over the drop_zone layer ───────────────────────
        sm.post_event(&Event::PointerDown { x: 138.5, y: 392.5 });
        sm.post_event(&Event::PointerMove { x: 170.0, y: 120.0 });
        assert_eq!(pos_of(sm.player), [170.0, 120.0]);

        sm.post_event(&Event::PointerUp { x: 170.0, y: 120.0 });
        sm.tick(300.0).unwrap();

        // Snap target derived from the drop_zone layer's RENDERED bounds at
        // drop time (its center, plus the star's anchor offset so the star
        // lands visually centered) — no coordinates anywhere in the state
        // machine JSON, and nothing extracted from the animation at load.
        assert_near(pos_of(sm.player), dock, "dock lands on zone center");
        assert_eq!(
            sm.get_numeric_input("docked_count"),
            Some(1.0),
            "drop zone actions should run on dock"
        );

        // ── Locked: docked object can no longer be grabbed ───────────────
        let docked_at = pos_of(sm.player);
        sm.post_event(&Event::PointerDown {
            x: docked_at[0],
            y: docked_at[1],
        });
        sm.post_event(&Event::PointerMove { x: 300.0, y: 300.0 });
        assert_eq!(
            pos_of(sm.player),
            docked_at,
            "locked object should ignore further drags"
        );
    }

    // The payoff of drop-time bounds reads: a zone that is ANIMATING snaps
    // the object to wherever it currently is — no coordinates, no tracking
    // expression. star_drop_moving.json glides drop_zone from [100,100]
    // toward [400,100] over frames 0..150 @30fps.
    #[test]
    fn snap_target_follows_animated_zone() {
        let definition = include_str!("../assets/statemachines/star_drop_dnd.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/star_drop_moving.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap(); // bounds lag the scene by one canvas update

        // Let the zone travel roughly half its journey.
        sm.tick(2500.0).unwrap();
        sm.tick(16.0).unwrap();
        let zone_pos = position_of(sm.player, "drop_zone");
        assert!(
            zone_pos[0] > 200.0,
            "zone should be well past its authored start, got {zone_pos:?}"
        );

        // Grab the star and release it over the zone's CURRENT position.
        // The engine reads the same rendered scene the test just did, so
        // the derived snap target matches this zone_pos exactly.
        sm.post_event(&Event::PointerDown { x: 138.5, y: 392.5 });
        sm.post_event(&Event::PointerMove {
            x: zone_pos[0],
            y: zone_pos[1],
        });
        sm.post_event(&Event::PointerUp {
            x: zone_pos[0],
            y: zone_pos[1],
        });
        sm.tick(400.0).unwrap();

        assert_near(
            pos_of(sm.player),
            zone_pos,
            "star snaps to where the zone WAS at drop time",
        );
        assert_eq!(
            sm.get_numeric_input("docked_count"),
            Some(1.0),
            "animated zone should still run its dock actions"
        );
    }

    // stateName scoping (assets/statemachines/star_drop_dnd_scoped.json):
    // the gesture is bound to the "board" state, OnComplete-style.
    #[test]
    fn state_bound_gesture_gates_and_cancels() {
        let definition = include_str!("../assets/statemachines/star_drop_dnd_scoped.json");
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

        // In "paused", the gesture is inert: no grab, no movement.
        sm.set_numeric_input("paused", 1.0, true, false).unwrap();
        assert_eq!(sm.get_current_state_name(), "paused");
        sm.post_event(&Event::PointerDown { x: 138.5, y: 392.5 });
        sm.post_event(&Event::PointerMove { x: 300.0, y: 300.0 });
        assert_eq!(
            pos_of(sm.player),
            [138.5, 392.5],
            "state-bound gesture must not grab outside its state"
        );
        sm.post_event(&Event::PointerUp { x: 300.0, y: 300.0 });

        // Back in "board", the gesture works normally — grabbed off-center
        // at (150, 400), the (-11.5, -7.5) pointer-to-object offset is
        // preserved through the drag.
        sm.set_numeric_input("paused", 0.0, true, false).unwrap();
        assert_eq!(sm.get_current_state_name(), "board");
        sm.post_event(&Event::PointerDown { x: 150.0, y: 400.0 });
        sm.post_event(&Event::PointerMove { x: 300.0, y: 300.0 });
        assert_eq!(pos_of(sm.player), [288.5, 292.5]);

        // Leaving the owning state MID-DRAG cancels the gesture: the star
        // glides back to rest and the eventual PointerUp does not dock.
        sm.set_numeric_input("paused", 1.0, true, false).unwrap();
        assert_eq!(sm.get_current_state_name(), "paused");
        sm.tick(16.0).unwrap(); // cancel detected, return snap starts
        sm.tick(300.0).unwrap(); // snap completes
        assert_eq!(
            pos_of(sm.player),
            [138.5, 392.5],
            "mid-drag state exit should cancel back to rest"
        );
        sm.post_event(&Event::PointerUp { x: 159.5, y: 110.5 });
        sm.tick(300.0).unwrap();
        assert_eq!(
            sm.get_numeric_input("docked_count"),
            Some(0.0),
            "release after cancel must not dock"
        );
    }

}
