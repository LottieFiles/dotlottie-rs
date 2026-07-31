#![cfg(feature = "state-machines")]

// Slide-to-unlock, fully authored in the state machine
// (assets/statemachines/slide_unlock.json + slide_unlock.json animation).
//
// The continuous position -> color mapping is computed by state machine
// ACTIONS on PointerMove (SetNumeric/Clamp/Increment/Multiply):
//   thumb_x  = clamp(cursor_x, 137, 387)
//   progress = (thumb_x - 137) * 0.004        -> 0..1 across the track
//   inv      = 1 - progress
// and rendered through live-bound state slots:
//   thumb_pos = [$thumb_x, 433]
//   slide_bg  = [$inv, $progress, 0.25]        -> red..green
//
// The host feeds cursor_x and pointer events. No host math at all.

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::events::Event;
    use dotlottie_rs::{ColorSpace, Player};

    fn slot_k(player: &Player, slot_id: &str) -> Vec<f32> {
        let json = player.get_slot_str(slot_id);
        let v: serde_json::Value = serde_json::from_str(&json).expect("slot json");
        match &v["k"] {
            serde_json::Value::Number(n) => vec![n.as_f64().unwrap() as f32],
            serde_json::Value::Array(arr) => {
                arr.iter().map(|x| x.as_f64().unwrap() as f32).collect()
            }
            other => panic!("unexpected k shape: {other:?}"),
        }
    }

    fn assert_near(actual: &[f32], expected: &[f32], what: &str) {
        assert_eq!(actual.len(), expected.len(), "{what}: arity");
        for (a, e) in actual.iter().zip(expected) {
            assert!((a - e).abs() < 1e-3, "{what}: {actual:?} vs {expected:?}");
        }
    }

    #[test]
    fn slide_to_unlock_full_cycle() {
        let definition = include_str!("../assets/statemachines/slide_unlock.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/slide_unlock.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();

        assert_eq!(sm.get_current_state_name(), "idle");
        assert_near(
            &slot_k(sm.player, "thumb_pos"),
            &[137.0, 433.0],
            "rest thumb",
        );
        assert_near(
            &slot_k(sm.player, "slide_bg"),
            &[1.0, 0.0, 0.25],
            "rest color",
        );

        // Grab the thumb (engine hit-test on the Slider Thumb layer).
        sm.post_event(&Event::PointerDown { x: 137.0, y: 433.0 });
        assert_eq!(sm.get_current_state_name(), "sliding");

        // Drag to the middle of the track: progress 0.5 -> 50/50 color mix.
        sm.set_numeric_input("cursor_x", 262.0, true, false)
            .unwrap();
        sm.post_event(&Event::PointerMove { x: 262.0, y: 433.0 });
        assert_near(
            &slot_k(sm.player, "thumb_pos"),
            &[262.0, 433.0],
            "mid thumb",
        );
        assert_near(
            &slot_k(sm.player, "slide_bg"),
            &[0.5, 0.5, 0.25],
            "mid color",
        );

        // Overshoot far right: clamp holds the thumb at 387, progress at 1.
        sm.set_numeric_input("cursor_x", 500.0, true, false)
            .unwrap();
        sm.post_event(&Event::PointerMove { x: 500.0, y: 433.0 });
        assert_near(
            &slot_k(sm.player, "thumb_pos"),
            &[387.0, 433.0],
            "clamped thumb",
        );
        assert_near(
            &slot_k(sm.player, "slide_bg"),
            &[0.0, 1.0, 0.25],
            "full color",
        );

        // Release at the end: progress > 0.9 -> tweened into unlocked.
        sm.post_event(&Event::PointerUp { x: 500.0, y: 433.0 });
        sm.tick(350.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "unlocked");
        assert_near(
            &slot_k(sm.player, "main_bg"),
            &[0.15, 0.6, 0.35],
            "unlock bg",
        );

        // Grab again from the unlocked position and abandon mid-track:
        // release below the threshold glides everything home.
        sm.tick(16.0).unwrap();
        sm.post_event(&Event::PointerDown { x: 387.0, y: 433.0 });
        assert_eq!(sm.get_current_state_name(), "sliding");
        sm.set_numeric_input("cursor_x", 200.0, true, false)
            .unwrap();
        sm.post_event(&Event::PointerMove { x: 200.0, y: 433.0 });
        sm.post_event(&Event::PointerUp { x: 200.0, y: 433.0 });
        sm.tick(350.0).unwrap();

        assert_eq!(sm.get_current_state_name(), "idle");
        assert_near(
            &slot_k(sm.player, "thumb_pos"),
            &[137.0, 433.0],
            "thumb home",
        );
        assert_near(
            &slot_k(sm.player, "slide_bg"),
            &[1.0, 0.0, 0.25],
            "color home",
        );
        // main_bg was only declared by `unlocked`: released back to authored.
        assert_near(
            &slot_k(sm.player, "main_bg"),
            &[0.8863, 0.2902, 0.2588],
            "main bg released to authored",
        );
    }

    // Same slider built on the DragAndDrop interaction with onGrab/onDrop
    // hooks (slide_unlock_dnd.json): DnD provides the gesture lifecycle
    // (hit-tested grab, release), empty dropZones make it lifecycle-only,
    // and the state slot binding [$thumb_x, 433] constrains the position —
    // its write lands AFTER the gesture's raw 2D write within the same
    // pointer event, which is the composition this test pins down.
    #[test]
    fn slide_to_unlock_via_dnd_hooks() {
        let definition = include_str!("../assets/statemachines/slide_unlock_dnd.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/slide_unlock.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "idle");

        // onGrab hook fires `grab` -> sliding.
        sm.post_event(&Event::PointerDown { x: 137.0, y: 433.0 });
        assert_eq!(sm.get_current_state_name(), "sliding");

        // Drag with a VERTICAL stray (y=420): the gesture writes [262, 420],
        // then the clamped binding writes [262, 433] last. Y stays on the
        // track — the constraint holds by write ordering.
        sm.set_numeric_input("cursor_x", 262.0, true, false)
            .unwrap();
        sm.post_event(&Event::PointerMove { x: 262.0, y: 420.0 });
        assert_near(
            &slot_k(sm.player, "thumb_pos"),
            &[262.0, 433.0],
            "constrained thumb",
        );
        assert_near(
            &slot_k(sm.player, "slide_bg"),
            &[0.5, 0.5, 0.25],
            "mid color",
        );

        // Release past the threshold: onDrop fires `release`, guards pick
        // unlocked; empty dropZones -> no snap/return fighting the tween.
        sm.set_numeric_input("cursor_x", 500.0, true, false)
            .unwrap();
        sm.post_event(&Event::PointerMove { x: 500.0, y: 433.0 });
        sm.post_event(&Event::PointerUp { x: 500.0, y: 433.0 });
        sm.tick(350.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "unlocked");
        assert_near(
            &slot_k(sm.player, "main_bg"),
            &[0.15, 0.6, 0.35],
            "unlock bg",
        );
        assert_near(
            &slot_k(sm.player, "thumb_pos"),
            &[387.0, 433.0],
            "thumb at end",
        );

        // Grab from unlocked, abandon early: glide home, releases main_bg.
        sm.tick(16.0).unwrap();
        sm.post_event(&Event::PointerDown { x: 387.0, y: 433.0 });
        assert_eq!(sm.get_current_state_name(), "sliding");
        sm.set_numeric_input("cursor_x", 200.0, true, false)
            .unwrap();
        sm.post_event(&Event::PointerMove { x: 200.0, y: 433.0 });
        sm.post_event(&Event::PointerUp { x: 200.0, y: 433.0 });
        sm.tick(350.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "idle");
        assert_near(
            &slot_k(sm.player, "thumb_pos"),
            &[137.0, 433.0],
            "thumb home",
        );
        assert_near(
            &slot_k(sm.player, "main_bg"),
            &[0.8863, 0.2902, 0.2588],
            "main bg released",
        );
    }

    // Slider drag scrubbing a PRECOMP's timeline
    // (slide_unlock_precomp.json + slider_precomp_scrub.json): thumb glide
    // AND track color journey are AUTHORED KEYFRAMES inside the precomp;
    // the state machine only maps the drag to seconds
    // (inner_t = clamp(cursor_x, 61..461) - 61) * 0.01 -> 0..4s) and binds
    // the "inner_time" scalar slot, which drives a hidden layer's rotation
    // read by the precomp's tm expression. No color math, no thumb binding.
    // Pixel samples verify the authored color journey follows the scrub.
    #[test]
    fn slider_scrubs_precomp_timeline() {
        let definition = include_str!("../assets/statemachines/slide_unlock_precomp.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/slider_precomp_scrub.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();
        assert_near(
            &slot_k(sm.player, "inner_time"),
            &[0.0],
            "precomp clock at 0",
        );

        // Grab the thumb INSIDE the precomp (hit-testing reaches nested
        // layers) at its authored rest position.
        sm.post_event(&Event::PointerDown { x: 61.0, y: 214.0 });
        assert_eq!(sm.get_current_state_name(), "sliding");

        // Mid-track: (261 - 61) * 0.01 = 2.0s -> precomp frame 60, where
        // the authored slide_bg journey is BLUE [0.23, 0.51, 0.96].
        sm.set_numeric_input("cursor_x", 261.0, true, false)
            .unwrap();
        sm.post_event(&Event::PointerMove { x: 261.0, y: 214.0 });
        assert_near(
            &slot_k(sm.player, "inner_time"),
            &[2.0],
            "precomp clock mid",
        );
        sm.tick(16.0).unwrap();
        // Sample the track away from the thumb (thumb is ~x=261 at 2s).
        let px = buffer[215 * 512 + 420];
        let (r, g, b) = (px & 0xFF, (px >> 8) & 0xFF, (px >> 16) & 0xFF);
        assert!(
            b > 180 && r < 100,
            "track should be authored-blue at 2s, got r={r} g={g} b={b}"
        );

        // Release at the end: unlocked pins the clock to 4.0s (frame 120),
        // where the authored journey is GREEN [0.13, 0.77, 0.37].
        sm.set_numeric_input("cursor_x", 500.0, true, false)
            .unwrap();
        sm.post_event(&Event::PointerMove { x: 500.0, y: 214.0 });
        sm.post_event(&Event::PointerUp { x: 500.0, y: 214.0 });
        sm.tick(350.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "unlocked");
        assert_near(
            &slot_k(sm.player, "inner_time"),
            &[4.0],
            "precomp clock at end",
        );
        sm.tick(16.0).unwrap();
        // Thumb is at the right end now; sample the track's left side.
        let px = buffer[215 * 512 + 100];
        let (r, g, b) = (px & 0xFF, (px >> 8) & 0xFF, (px >> 16) & 0xFF);
        assert!(
            g > 150 && r < 100,
            "track should be authored-green when unlocked, got r={r} g={g} b={b}"
        );
    }

    // Curved-path slider (slide_path.json SM + slide_path_scrub.json):
    // the thumb's slot default is a 37-keyframe MOTION PATH with spatial
    // tangents. The state machine still only maps cursor x -> seconds
    // (inner_t = clamp((cursor_x - 72) * 0.01524, 0, 4.95)); the curve
    // following comes entirely from scrubbing the authored keyframes.
    #[test]
    fn slider_scrubs_curved_path() {
        let definition = include_str!("../assets/statemachines/slide_path.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/slide_path_scrub.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();

        // Grab the thumb at the curve's START (authored keyframe 0).
        assert!(
            sm.player.hit_check("thumb", 72.0, 70.0),
            "thumb at curve start"
        );
        sm.post_event(&Event::PointerDown { x: 72.0, y: 70.0 });
        assert_eq!(sm.get_current_state_name(), "sliding");

        // Drag to cursor x=235: inner_t = (235 - 72) * 0.01524 = 2.48412s
        // -> precomp frame ~74.5. The authored curve there is ~[278, 140] —
        // nowhere near the cursor's y, proving the thumb rides the CURVE,
        // not the pointer.
        sm.set_numeric_input("cursor_x", 235.0, true, false)
            .unwrap();
        sm.post_event(&Event::PointerMove { x: 235.0, y: 300.0 });
        assert_near(
            &slot_k(sm.player, "inner_time"),
            &[2.48412],
            "curve clock mid",
        );
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("thumb", 278.0, 140.0),
            "thumb should be ON the authored curve at 2.48s"
        );

        // Release far right: clamped to 4.95s > 4.4 threshold -> unlocked;
        // thumb at the curve's end, unlock recolors the thumb green.
        sm.set_numeric_input("cursor_x", 500.0, true, false)
            .unwrap();
        sm.post_event(&Event::PointerMove { x: 500.0, y: 300.0 });
        sm.post_event(&Event::PointerUp { x: 500.0, y: 300.0 });
        sm.tick(350.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "unlocked");
        assert_near(
            &slot_k(sm.player, "inner_time"),
            &[4.95],
            "curve clock at end",
        );
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("thumb", 396.0, 338.0),
            "thumb should rest at the curve's end"
        );
        // Unlock feedback: thumb recolored green. Sample within the thumb
        // but past the curve's endpoint — the black path stroke draws ON TOP
        // of the thumb, so sampling the exact center hits the stroke.
        let px = buffer[350 * 512 + 405];
        let (r, g, b) = (px & 0xFF, (px >> 8) & 0xFF, (px >> 16) & 0xFF);
        assert!(
            g > 150 && r < 100,
            "thumb should be green when unlocked, got r={r} g={g} b={b}"
        );
    }
}
