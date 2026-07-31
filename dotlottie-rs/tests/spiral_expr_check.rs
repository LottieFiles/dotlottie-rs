#![cfg(feature = "state-machines")]

// Expression-driven variant of the spiral drag: the knob's position is a
// BAKED LOTTIE EXPRESSION (arc-length bezier interpolation over the track's
// own control points) reading the driver rotation — no keyframes, no
// precomp, no time remap. Uses the exact same state machine as the
// keyframe variant (spiral_drag.json): the gesture and SM are agnostic to
// how the animation turns progress into pixels.

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::events::Event;
    use dotlottie_rs::{ColorSpace, Player};

    #[test]
    fn expression_variant_same_state_machine() {
        let definition = include_str!("../assets/statemachines/spiral_drag.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/spiral_expr.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();

        assert!(sm.player.hit_check("knob", 298.0, 256.0), "knob at start");
        sm.post_event(&Event::PointerDown { x: 298.0, y: 256.0 });

        // Branch protection still holds (gesture-side, animation-agnostic).
        sm.post_event(&Event::PointerMove { x: 355.0, y: 256.0 });
        assert!(sm.get_numeric_input("path_t").unwrap() < 0.1);

        // Walk the first turn; the EXPRESSION places the knob at the exact
        // arc point for the gesture's progress.
        for p in [
            [290.0, 291.0],
            [256.0, 312.0],
            [211.0, 301.0],
            [185.0, 256.0],
            [201.0, 201.0],
            [256.0, 171.0],
            [321.0, 191.0],
            [355.0, 256.0],
        ] {
            sm.post_event(&Event::PointerMove { x: p[0], y: p[1] });
        }
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("knob", 355.0, 256.0),
            "expression places the knob at the projected arc point"
        );
    }
}
