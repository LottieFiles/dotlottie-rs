#![cfg(feature = "state-machines")]

// Spiral unlock WITH on-path drop zones
// (assets/statemachines/spiral_unlock_zones.json against the same
// spiral_unlock.json animation, which carries four small drop_zone dots
// on the spiral).
//
// Path-mode drop zones are DOCK POINTS captured by arc proximity:
// releasing with the knob within reach of a dot snaps `path_t` to the
// dot's own on-path position and publishes `zone`; the machine
// transitions to "docked", whose entry actions bake the frame via
// segment-relative SetProgress. `dockFallback: "previous"` ratchets an
// uncaptured release back to the nearest dot behind. Grabbing un-docks.
// The unlock guard outranks the dock guard, and the end dot's lock:true
// retires the gesture after unlocking (the scrub lives in onDrag, which
// is gesture-scoped, not state-scoped).
//
// All geometry is DERIVED from the loaded animation (path verts, zone
// bounds), so re-authoring the spiral does not break this test.

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::events::Event;
    use dotlottie_rs::{ColorSpace, Player};

    fn waypoints(player: &Player) -> Vec<[f32; 2]> {
        player
            .layer_path("Spiral Path")
            .expect("spiral path should extract")
            .verts
    }

    fn center_of(player: &Player, layer: &str) -> [f32; 2] {
        let obb = player.layer_bounds(layer).expect("layer bounds");
        [
            (obb[0].x + obb[1].x + obb[2].x + obb[3].x) / 4.0,
            (obb[0].y + obb[1].y + obb[2].y + obb[3].y) / 4.0,
        ]
    }

    fn walk(sm: &mut dotlottie_rs::StateMachineEngine, from: [f32; 2], to: [f32; 2]) {
        for step in 1..=4 {
            let t = step as f32 / 4.0;
            sm.post_event(&Event::PointerMove {
                x: from[0] + (to[0] - from[0]) * t,
                y: from[1] + (to[1] - from[1]) * t,
            });
        }
    }

    #[test]
    fn dock_on_zone_ratchet_and_still_unlock() {
        let definition = include_str!("../assets/statemachines/spiral_unlock_zones.json");
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
        // No idle state: "idle" IS docked-at-the-start-zone (an invisible
        // drop_zone0 sits at the spiral's start).
        assert_eq!(sm.get_current_state_name(), "docked");
        assert_eq!(sm.player.current_frame(), 0.0);

        let wp = waypoints(sm.player);
        let rest = wp[0];
        let end = *wp.last().unwrap();
        let zone2 = center_of(sm.player, "drop_zone2");

        // ── Release before the first visible dot: ratchets to the START
        // zone and glides back ALONG THE PATH (not a straight pose blend).
        sm.post_event(&Event::PointerDown {
            x: rest[0],
            y: rest[1],
        });
        for w in 1..=2 {
            walk(&mut sm, wp[w - 1], wp[w]);
        }
        sm.post_event(&Event::PointerUp {
            x: wp[2][0],
            y: wp[2][1],
        });
        assert_eq!(sm.get_current_state_name(), "docked");
        sm.tick(600.0).unwrap();
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.current_frame() < 1.5,
            "glide should return to the start frame, at {}",
            sm.player.current_frame()
        );
        assert!(sm.player.hit_check("Ellipse 1", rest[0], rest[1]));

        // ── Dock: release NEAR drop_zone2, not on it ─────────────────────
        // Approach along the path and release a bit short of the dot —
        // arc-proximity capture (knob half-size + dot half-size of reach)
        // docks anyway and snaps progress to the DOT's position.
        sm.post_event(&Event::PointerDown {
            x: rest[0],
            y: rest[1],
        });
        for w in 1..=5 {
            walk(&mut sm, wp[w - 1], wp[w]);
        }
        walk(&mut sm, wp[5], zone2);
        let near = [
            zone2[0] + (wp[5][0] - zone2[0]) * 0.35,
            zone2[1] + (wp[5][1] - zone2[1]) * 0.35,
        ];
        sm.post_event(&Event::PointerMove {
            x: near[0],
            y: near[1],
        });
        sm.post_event(&Event::PointerUp {
            x: near[0],
            y: near[1],
        });

        assert_eq!(sm.get_current_state_name(), "docked");
        let frame_release = sm.player.current_frame();

        // The dock GLIDES along the path (interaction tween), it doesn't
        // teleport: the frame keeps moving after release until it bakes
        // the dot's exact progress.
        sm.tick(400.0).unwrap();
        sm.tick(16.0).unwrap();
        let t_dock = sm.get_numeric_input("path_t").unwrap();
        let frame = sm.player.current_frame();
        assert!(
            frame != frame_release,
            "dock should glide after release, not land instantly"
        );
        assert!(
            (frame - t_dock * 119.0).abs() < 1.5,
            "docked frame should bake the dot's progress: t={t_dock}, frame={frame}"
        );
        assert!(
            sm.player.hit_check("Ellipse 1", zone2[0], zone2[1]),
            "knob should be parked on drop_zone2"
        );

        // Docked holds: stray mouse movement must not scrub.
        sm.post_event(&Event::PointerMove { x: 400.0, y: 100.0 });
        assert_eq!(sm.player.current_frame(), frame);

        // ── Ratchet: wander forward, release off-zone -> back to zone2 ───
        // dockFallback "previous": an uncaptured release docks at the
        // nearest zone BEHIND the release progress instead of gliding all
        // the way home.
        sm.post_event(&Event::PointerDown {
            x: zone2[0],
            y: zone2[1],
        });
        assert_eq!(sm.get_current_state_name(), "dragging");
        walk(&mut sm, zone2, wp[6]);
        sm.post_event(&Event::PointerUp {
            x: wp[6][0],
            y: wp[6][1],
        });
        assert_eq!(
            sm.get_current_state_name(),
            "docked",
            "uncaptured release should ratchet back, not go home"
        );

        // This ratchet spans a long stretch of path — watch it glide
        // through a strictly intermediate frame on the way back.
        let frame_up = sm.player.current_frame();
        sm.tick(150.0).unwrap();
        let frame_mid = sm.player.current_frame();
        sm.tick(400.0).unwrap();
        let frame_final = sm.player.current_frame();
        assert!(
            frame_up > frame_mid && frame_mid > frame_final,
            "ratchet should glide back through intermediate frames: {frame_up} -> {frame_mid} -> {frame_final}"
        );
        assert!(
            (frame_final - frame).abs() < 1.5,
            "ratchet lands on the same zone2 frame, got {frame_final} vs {frame}"
        );

        // ── Un-dock: re-grab and finish the spiral -> unlocked ───────────
        sm.post_event(&Event::PointerDown {
            x: zone2[0],
            y: zone2[1],
        });
        assert_eq!(sm.get_current_state_name(), "dragging");

        walk(&mut sm, zone2, wp[6]);
        for w in 7..wp.len() {
            walk(&mut sm, wp[w - 1], wp[w]);
        }
        // The end dot captures this release, but its snapped progress is
        // still >= 0.95 and the unlock guard outranks the dock guard.
        sm.post_event(&Event::PointerUp { x: end[0], y: end[1] });
        sm.tick(600.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "unlocked");
        sm.tick(500.0).unwrap();
        assert!(
            sm.player.current_frame() >= 120.0,
            "celebration should play, at frame {}",
            sm.player.current_frame()
        );

        // ── Post-unlock: the end dot's lock retires the gesture ──────────
        sm.tick(2000.0).unwrap(); // let the celebration finish
        let settled = sm.player.current_frame();
        sm.post_event(&Event::PointerDown { x: end[0], y: end[1] });
        walk(&mut sm, end, wp[6]);
        sm.post_event(&Event::PointerUp {
            x: wp[6][0],
            y: wp[6][1],
        });
        sm.tick(16.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "unlocked");
        assert_eq!(
            sm.player.current_frame(),
            settled,
            "locked gesture must not scrub after unlock"
        );
    }
}
