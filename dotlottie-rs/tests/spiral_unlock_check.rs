#![cfg(feature = "state-machines")]

// Spiral slide-to-unlock (assets/statemachines/spiral_unlock.json +
// spiral_unlock.json): the drag scrubs the MAIN timeline. The knob's
// position slot default is keyframed along the spiral over the "locked"
// segment (0..119), so segment-relative SetProgress moves the knob; the
// "unlocked" segment (120..150) is the celebration.
//
// Composition under test: path-mode DragAndDrop (progress sensor +
// grab/release events) + a PointerMove interaction SCOPED to the
// "dragging" state (SetNumeric/Multiply/SetProgress scrub) + guarded
// Tweened transitions (threshold unlock, glide-home on a miss).
//
// All geometry is DERIVED from the loaded animation (path verts double as
// the knob's keyframe waypoints), so re-authoring the spiral — including
// via layer scale — does not break this test.

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::events::Event;
    use dotlottie_rs::{ColorSpace, Player};

    /// The spiral's vertices in comp space — by construction also the
    /// knob's authored keyframe positions.
    fn waypoints(player: &Player) -> Vec<[f32; 2]> {
        player
            .layer_path("Spiral Path")
            .expect("spiral path should extract")
            .verts
    }

    /// Walk the pointer along the spiral through waypoint index `upto`,
    /// subdividing each hop so the windowed projection follows.
    fn drag_route(sm: &mut dotlottie_rs::StateMachineEngine, wp: &[[f32; 2]], upto: usize) {
        for w in 1..=upto {
            let (a, b) = (wp[w - 1], wp[w]);
            for step in 1..=4 {
                let t = step as f32 / 4.0;
                sm.post_event(&Event::PointerMove {
                    x: a[0] + (b[0] - a[0]) * t,
                    y: a[1] + (b[1] - a[1]) * t,
                });
            }
        }
    }

    #[test]
    fn unlock_cycle_scrub_snapback_and_locked_state2() {
        let definition = include_str!("../assets/statemachines/spiral_unlock.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/spiral_unlock.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();

        let wp = waypoints(sm.player);
        assert!(wp.len() >= 3, "spiral should have several vertices");
        let rest = wp[0];
        let end = *wp.last().unwrap();

        // Initial: paused at the locked segment start, knob at rest.
        assert_eq!(sm.get_current_state_name(), "idle");
        assert_eq!(sm.player.current_frame(), 0.0);
        assert!(sm.player.hit_check("Ellipse 1", rest[0], rest[1]));

        // ── Grab enters "dragging"; moves scrub the timeline ─────────────
        sm.post_event(&Event::PointerDown {
            x: rest[0],
            y: rest[1],
        });
        assert_eq!(sm.get_current_state_name(), "dragging");

        drag_route(&mut sm, &wp, 4); // partway around the spiral
        let t_mid = sm.get_numeric_input("path_t").unwrap();
        assert!(
            t_mid > 0.2 && t_mid < 0.9,
            "partial drag should yield mid progress, got {t_mid}"
        );
        let frame_mid = sm.player.current_frame();
        assert!(
            (frame_mid - t_mid * 119.0).abs() < 1.5,
            "frame should track progress: t={t_mid}, frame={frame_mid}"
        );
        // The knob rides its authored keyframes to the scrubbed pose.
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("Ellipse 1", wp[4][0], wp[4][1]),
            "knob should sit at the waypoint-4 pose"
        );

        // ── Early release: tween glides back to the locked rest ─────────
        sm.post_event(&Event::PointerUp {
            x: wp[4][0],
            y: wp[4][1],
        });
        sm.tick(600.0).unwrap();
        sm.tick(16.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "idle");
        assert_eq!(
            sm.player.current_frame(),
            0.0,
            "missed unlock should return to the locked rest frame"
        );

        // Idle mouse movement must not scrub (stale path_t, scoped scrub).
        sm.post_event(&Event::PointerMove { x: 100.0, y: 100.0 });
        assert_eq!(sm.player.current_frame(), 0.0);

        // ── Full drag to the spiral end unlocks ─────────────────────────
        sm.post_event(&Event::PointerDown {
            x: rest[0],
            y: rest[1],
        });
        drag_route(&mut sm, &wp, wp.len() - 1);
        let t_end = sm.get_numeric_input("path_t").unwrap();
        assert!(t_end >= 0.95, "full route should reach the end, got {t_end}");

        sm.post_event(&Event::PointerUp { x: end[0], y: end[1] });
        sm.tick(600.0).unwrap(); // unlock tween completes
        assert_eq!(sm.get_current_state_name(), "unlocked");

        // The celebration segment plays (frames 120..150).
        sm.tick(500.0).unwrap();
        let celebrating = sm.player.current_frame();
        assert!(
            celebrating >= 120.0,
            "unlocked state should play its segment, at frame {celebrating}"
        );

        // ── State 2: dragging is inert ───────────────────────────────────
        sm.tick(1500.0).unwrap(); // let the celebration finish
        let settled = sm.player.current_frame();
        assert!(settled >= 120.0);

        sm.post_event(&Event::PointerDown { x: end[0], y: end[1] });
        drag_route(&mut sm, &wp, 4); // scrub actions are out of scope here
        sm.post_event(&Event::PointerUp {
            x: wp[4][0],
            y: wp[4][1],
        });
        sm.tick(16.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "unlocked");
        assert!(
            sm.player.current_frame() >= 120.0,
            "dragging in unlocked must not scrub the timeline, at frame {}",
            sm.player.current_frame()
        );
    }
}
