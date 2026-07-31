#![cfg(feature = "state-machines")]

// Ghost drag (assets/statemachines/star_drop_ghost.json): `ghost: true`
// drags a frozen DUPLICATE of the layer — a real ThorVG paint clone
// parked on the canvas above the picture — while the original (and its
// slot) stays parked. On release the ghost glides to the dock and the
// slot is written exactly once at landing.
//
// Verification is two-channel: the slot proves the original never moves
// during the drag, and pixel probes into the render buffer prove the
// ghost visually exists where the pointer is.

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::events::Event;
    use dotlottie_rs::{ColorSpace, Player};

    fn pos_of(player: &Player) -> [f32; 2] {
        let json = player.get_slot_str("star_pos");
        let v: serde_json::Value = serde_json::from_str(&json).expect("slot json");
        let k = v["k"].as_array().expect("k array");
        [k[0].as_f64().unwrap() as f32, k[1].as_f64().unwrap() as f32]
    }

    /// The star fill is light gray on a light-blue background: probe by
    /// checking the red channel dominance difference (bg 0xADD8E6 has
    /// R=0xAD; star gray 0xB2B6B7 is near-neutral). Simpler and robust:
    /// compare against the flat background pixel value sampled far from
    /// everything.
    fn px(buffer: &[u32], x: usize, y: usize) -> u32 {
        buffer[y * 512 + x]
    }

    #[test]
    fn ghost_drag_parks_original_and_lands_once() {
        let definition = include_str!("../assets/statemachines/star_drop_ghost.json");
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
        sm.tick(16.0).unwrap();

        let background = px(&buffer, 480, 480);
        assert_ne!(background, 0, "background should be painted");
        assert_ne!(
            px(&buffer, 138, 392),
            background,
            "original star visible at rest"
        );

        // ── Grab and drag: the ghost follows, the ORIGINAL stays put ────
        sm.post_event(&Event::PointerDown { x: 138.5, y: 392.5 });
        sm.post_event(&Event::PointerMove { x: 300.0, y: 250.0 });
        assert_eq!(
            pos_of(sm.player),
            [138.5, 392.5],
            "slot must not move during a ghost drag"
        );
        sm.tick(16.0).unwrap();
        assert_ne!(
            px(&buffer, 300, 250),
            background,
            "ghost should render at the pointer"
        );
        assert_ne!(
            px(&buffer, 138, 392),
            background,
            "original should still render at rest"
        );

        // ── Release over the zone: the ghost retires at the release
        // point and the ORIGINAL glides rest -> dock (slot tween) — the
        // visible "item travels to its destination" beat.
        sm.post_event(&Event::PointerMove { x: 159.5, y: 110.5 });
        sm.post_event(&Event::PointerUp { x: 159.5, y: 110.5 });
        assert_eq!(
            pos_of(sm.player),
            [138.5, 392.5],
            "glide starts from rest after the ghost retires"
        );
        sm.tick(125.0).unwrap();
        let mid = pos_of(sm.player);
        assert!(
            mid != [138.5, 392.5] && mid != [159.5, 110.5],
            "original should glide rest -> dock, got {mid:?}"
        );
        sm.tick(200.0).unwrap();
        assert_eq!(
            pos_of(sm.player),
            [159.5, 110.5],
            "original lands on the dock"
        );
        assert_eq!(sm.get_numeric_input("docked_count"), Some(1.0));

        // Ghost is gone and the original now renders at the dock.
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();
        assert_ne!(
            px(&buffer, 159, 110),
            background,
            "original renders at the dock"
        );
        assert_eq!(
            px(&buffer, 138, 392),
            background,
            "rest position is empty after landing"
        );

        // ── lock: true still applies to ghost docks ──────────────────────
        sm.post_event(&Event::PointerDown { x: 159.5, y: 110.5 });
        sm.post_event(&Event::PointerMove { x: 300.0, y: 300.0 });
        sm.tick(16.0).unwrap();
        assert_eq!(
            pos_of(sm.player),
            [159.5, 110.5],
            "locked object should ignore further drags"
        );
        assert_eq!(
            px(&buffer, 300, 300),
            background,
            "no ghost should appear for a locked object"
        );
    }
}
