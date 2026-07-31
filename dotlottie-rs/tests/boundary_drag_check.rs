#![cfg(feature = "state-machines")]

// Boundary-constrained free drag (assets/statemachines/boundary_drag.json
// + boundary_drag.json): a DragAndDrop with NO drop zones (lifecycle-only,
// object stays where released) and `boundaryLayerName: "Rectangle 1"`.
//
// The boundary is the rectangle layer's RENDERED bounds, read every move —
// which is what makes this asset work at all: the layer carries a
// ~134%/132% scale, so its authored path (±146.5, ±141 around [262,268.5])
// renders as ±197.5, ±186 -> [64.5..459.5] x [82.5..454.5]. The clamp is
// inset by the circle's half-extents (32.5, 33), keeping the WHOLE circle
// inside: centers clamp to [97..427] x [115.5..421.5].

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::events::Event;
    use dotlottie_rs::{ColorSpace, Player};

    fn pos_of(player: &Player) -> [f32; 2] {
        let json = player.get_slot_str("circle_pos");
        let v: serde_json::Value = serde_json::from_str(&json).expect("slot json");
        let k = v["k"].as_array().expect("k array");
        [k[0].as_f64().unwrap() as f32, k[1].as_f64().unwrap() as f32]
    }

    fn assert_near(actual: [f32; 2], expected: [f32; 2], msg: &str) {
        assert!(
            (actual[0] - expected[0]).abs() < 0.5 && (actual[1] - expected[1]).abs() < 0.5,
            "{msg}: expected ~{expected:?}, got {actual:?}"
        );
    }

    #[test]
    fn circle_drags_freely_but_never_leaves_the_rectangle() {
        let definition = include_str!("../assets/statemachines/boundary_drag.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/boundary_drag.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap(); // bounds lag the scene by one canvas update

        assert_eq!(pos_of(sm.player), [183.0, 239.5]);

        // Grab at the circle's center (zero grab offset).
        sm.post_event(&Event::PointerDown { x: 183.0, y: 239.5 });

        // ── Free movement inside the rectangle ──────────────────────────
        sm.post_event(&Event::PointerMove { x: 300.0, y: 300.0 });
        assert_near(pos_of(sm.player), [300.0, 300.0], "inside: follows exactly");

        // ── Pointer escapes top-right: clamped per-axis ──────────────────
        // x pins at 459.5 - 32.5 = 427, y pins at 82.5 + 33 = 115.5. The
        // circle SLIDES along the edge rather than sticking.
        sm.post_event(&Event::PointerMove { x: 500.0, y: 100.0 });
        assert_near(pos_of(sm.player), [427.0, 115.5], "clamped to top-right");

        // ── Pointer far bottom-left: opposite corner ─────────────────────
        sm.post_event(&Event::PointerMove { x: 0.0, y: 500.0 });
        assert_near(pos_of(sm.player), [97.0, 421.5], "clamped to bottom-left");

        // Sliding along the bottom edge: x free, y stays pinned.
        sm.post_event(&Event::PointerMove { x: 250.0, y: 500.0 });
        assert_near(pos_of(sm.player), [250.0, 421.5], "slides along the edge");

        // ── Release: lifecycle-only gesture, the circle stays put ────────
        sm.post_event(&Event::PointerUp { x: 250.0, y: 500.0 });
        sm.tick(300.0).unwrap();
        assert_near(pos_of(sm.player), [250.0, 421.5], "stays where released");

        // The rendered circle is there too.
        sm.tick(16.0).unwrap();
        assert!(sm.player.hit_check("Ellipse 1", 250.0, 421.5));

        // ── Re-grab from the new rest works ──────────────────────────────
        sm.post_event(&Event::PointerDown { x: 250.0, y: 421.5 });
        sm.post_event(&Event::PointerMove { x: 183.0, y: 239.5 });
        assert_near(pos_of(sm.player), [183.0, 239.5], "re-drag back home");
    }
}
