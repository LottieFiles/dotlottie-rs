#![cfg(feature = "state-machines")]

// Spiral drag, progress-only (assets/statemachines/spiral_drag.json +
// spiral_scrub.json): the knob's journey along the spiral is AUTHORED
// KEYFRAMES (arc-length-proportional timing, spatial tangents), scrubbed
// through the tm/driver bridge by a single bound input. The PathDrag
// gesture emits ONLY progress — one state, one binding, one input.
//
// The spiral's turns pass within ~57px of each other — the case that
// breaks global nearest-point projection. The windowed branch-local
// search must hold the knob on its current turn even when the pointer
// sits exactly on an outer turn, while following a legitimate drag
// around the whole spiral.

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::events::Event;
    use dotlottie_rs::{ColorSpace, Player};

    #[test]
    fn spiral_branch_locality_progress_only() {
        let definition = include_str!("../assets/statemachines/spiral_drag.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/spiral_scrub.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();

        // Knob parked at the spiral's inner start (keyframe 0).
        assert!(sm.player.hit_check("knob", 298.0, 256.0), "knob at start");
        sm.post_event(&Event::PointerDown { x: 298.0, y: 256.0 });

        // ── Branch protection ────────────────────────────────────────────
        // [355, 256] lies on the SECOND turn — 57px away spatially, more
        // than a full turn away in arc length. The windowed search must
        // hold the knob on the inner turn.
        sm.post_event(&Event::PointerMove { x: 355.0, y: 256.0 });
        let t_blocked = sm.get_numeric_input("path_t").unwrap();
        assert!(
            t_blocked < 0.1,
            "progress must stay on the first turn, got {t_blocked}"
        );
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();
        assert!(
            !sm.player.hit_check("knob", 355.0, 256.0),
            "knob must NOT appear on the outer turn"
        );

        // ── Legitimate travel: walk the pointer around the first turn ────
        let route = [
            [290.0, 291.0],
            [256.0, 312.0],
            [211.0, 301.0],
            [185.0, 256.0],
            [201.0, 201.0],
            [256.0, 171.0],
            [321.0, 191.0],
            [355.0, 256.0],
        ];
        for p in route {
            sm.post_event(&Event::PointerMove { x: p[0], y: p[1] });
        }
        let t_walked = sm.get_numeric_input("path_t").unwrap();
        assert!(
            t_walked > t_blocked + 0.1,
            "progress should advance along the walked arc (blocked {t_blocked}, walked {t_walked})"
        );

        // The AUTHORED keyframes place the knob on the outer point when
        // scrubbed to that progress (arc-proportional timing).
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("knob", 355.0, 256.0),
            "knob should ride its authored keyframes to the outer point"
        );

        // Release: nothing snaps — progress input holds, knob stays.
        sm.post_event(&Event::PointerUp { x: 355.0, y: 256.0 });
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("knob", 355.0, 256.0),
            "knob rests where released"
        );

        // Re-grab there (re-seed picks the branch from the grab point) and
        // continue outward one more vertex.
        sm.post_event(&Event::PointerDown { x: 355.0, y: 256.0 });
        sm.post_event(&Event::PointerMove { x: 331.0, y: 331.0 });
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("knob", 331.0, 331.0),
            "drag continues on the correct branch after re-grab"
        );
    }
}
