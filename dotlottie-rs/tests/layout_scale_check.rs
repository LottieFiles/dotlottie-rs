#![cfg(feature = "state-machines")]

// Regression tests for coordinate-space correctness: gestures on a canvas
// whose size does NOT match the composition (256x256 canvas, 512-unit
// comps -> layout scale 0.5, Contain fit, no offset).
//
// The contract: pointer events arrive in CANVAS pixels; slots, layer
// transforms, and paths live in COMPOSITION units; the engine converts
// exactly once at the gesture boundary. Before that conversion existed,
// every test below failed — objects dragged at half pointer speed and
// detached from the cursor (found by the exploration in
// docs/explainers/thorvg-paint-apis-for-dnd.md §7).
//
// The strongest assertions here: COMP-space outcomes are identical to the
// 1:1 suites' — canvas size must not leak into gesture results.

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::events::Event;
    use dotlottie_rs::{ColorSpace, Player};

    const S: f32 = 0.5; // canvas px per comp unit at 256/512 Contain

    fn setup(anim: &str) -> (Vec<u32>, Player) {
        let mut buffer: Vec<u32> = vec![0; 256 * 256];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 256, 256, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new(format!("assets/animations/lottie/{anim}.json")).unwrap();
        player.load_animation_path(&path).unwrap();
        (buffer, player)
    }

    fn slot_pos(player: &Player, slot: &str) -> [f32; 2] {
        let json = player.get_slot_str(slot);
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
    fn canvas_to_comp_mapping() {
        let (_buffer, player) = setup("star_drop_static");
        assert_eq!(player.canvas_to_comp(128.0, 128.0), [256.0, 256.0]);
        assert_eq!(player.canvas_to_comp(0.0, 0.0), [0.0, 0.0]);
    }

    #[test]
    fn dnd_drag_and_dock_at_half_scale() {
        let definition = include_str!("../assets/statemachines/star_drop_dnd.json");
        let (_buffer, mut player) = setup("star_drop_static");
        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();

        // Star slot rests at comp [138.5, 392.5]; rendered at canvas x0.5.
        assert_eq!(slot_pos(sm.player, "star_pos"), [138.5, 392.5]);
        assert!(sm.player.hit_check("Star 2", 138.5 * S, 392.5 * S));

        // Grab at the star's rendered center; +40 CANVAS px = +80 comp
        // units — the object must stay under the pointer, not at half
        // speed.
        sm.post_event(&Event::PointerDown {
            x: 138.5 * S,
            y: 392.5 * S,
        });
        sm.post_event(&Event::PointerMove {
            x: 138.5 * S + 40.0,
            y: 392.5 * S,
        });
        assert_near(
            slot_pos(sm.player, "star_pos"),
            [138.5 + 40.0 / S, 392.5],
            "canvas pointer delta converts to comp units",
        );
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("Star 2", 138.5 * S + 40.0, 392.5 * S),
            "star should render under the pointer"
        );

        // Drag onto the zone (rendered at canvas [79.75, 55.25]) and
        // release: docks at the zone's COMP transform position — the same
        // value the full-size suite asserts.
        sm.post_event(&Event::PointerMove {
            x: 159.5 * S,
            y: 110.5 * S,
        });
        sm.post_event(&Event::PointerUp {
            x: 159.5 * S,
            y: 110.5 * S,
        });
        sm.tick(400.0).unwrap();
        assert_near(
            slot_pos(sm.player, "star_pos"),
            [159.5, 110.5],
            "dock target is canvas-size independent",
        );
        assert_eq!(sm.get_numeric_input("docked_count"), Some(1.0));
    }

    #[test]
    fn boundary_clamp_at_half_scale() {
        let definition = include_str!("../assets/statemachines/boundary_drag.json");
        let (_buffer, mut player) = setup("boundary_drag");
        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();

        sm.post_event(&Event::PointerDown {
            x: 183.0 * S,
            y: 239.5 * S,
        });

        // Inside: follows exactly (comp units).
        sm.post_event(&Event::PointerMove {
            x: 300.0 * S,
            y: 300.0 * S,
        });
        assert_near(
            slot_pos(sm.player, "circle_pos"),
            [300.0, 300.0],
            "inside: follows in comp units",
        );

        // Escape top-right: clamps to the SAME comp values as the
        // full-size suite ([427, 115.5]).
        sm.post_event(&Event::PointerMove { x: 250.0, y: 50.0 });
        assert_near(
            slot_pos(sm.player, "circle_pos"),
            [427.0, 115.5],
            "boundary clamp is canvas-size independent",
        );
    }

    #[test]
    fn path_drag_scrub_at_half_scale() {
        let definition = include_str!("../assets/statemachines/spiral_unlock.json");
        let (_buffer, mut player) = setup("spiral_unlock");
        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();

        // Path verts are comp units; the pointer walks their canvas
        // projections.
        let wp = sm.player.layer_path("Spiral Path").expect("path").verts;
        assert!(sm.player.hit_check("Ellipse 1", wp[0][0] * S, wp[0][1] * S));

        sm.post_event(&Event::PointerDown {
            x: wp[0][0] * S,
            y: wp[0][1] * S,
        });
        assert_eq!(sm.get_current_state_name(), "dragging");

        for w in 1..=4 {
            let (a, b) = (wp[w - 1], wp[w]);
            for step in 1..=4 {
                let t = step as f32 / 4.0;
                sm.post_event(&Event::PointerMove {
                    x: (a[0] + (b[0] - a[0]) * t) * S,
                    y: (a[1] + (b[1] - a[1]) * t) * S,
                });
            }
        }
        let t_mid = sm.get_numeric_input("path_t").unwrap();
        assert!(
            t_mid > 0.2 && t_mid < 0.9,
            "projection should advance along the path at half scale, got {t_mid}"
        );
        let frame = sm.player.current_frame();
        assert!(
            (frame - t_mid * 119.0).abs() < 1.5,
            "scrub should track progress: t={t_mid}, frame={frame}"
        );
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("Ellipse 1", wp[4][0] * S, wp[4][1] * S),
            "knob should ride to the waypoint-4 pose at half scale"
        );
    }
}
