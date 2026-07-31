#![cfg(feature = "state-machines")]

// PathDrag interaction (assets/statemachines/slide_path_drag.json +
// slide_path_progress.json): the pointer is PROJECTED onto the authored
// bezier ("Path 1"), normalized arc-length progress lands in the `path_t`
// input, and the `sliding` state binds it straight into the time-remapped
// precomp clock (tm scales progress -> seconds). The entire state machine
// has 3 inputs and zero action math; the host only posts pointer events.

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::events::Event;
    use dotlottie_rs::{ColorSpace, Player};

    #[test]
    fn path_drag_projects_and_scrubs() {
        let definition = include_str!("../assets/statemachines/slide_path_drag.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/slide_path_progress.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();

        // Grab the thumb at the curve's start.
        assert!(
            sm.player.hit_check("thumb", 72.0, 70.0),
            "thumb at curve start"
        );
        sm.post_event(&Event::PointerDown { x: 72.0, y: 70.0 });
        assert_eq!(sm.get_current_state_name(), "sliding");

        // Point the cursor NEAR (but off) the curve around vertex
        // [297.2, 148.9]: projection should land the thumb ON the curve.
        sm.post_event(&Event::PointerMove { x: 300.0, y: 150.0 });
        let t_mid = sm.get_numeric_input("path_t").unwrap();
        assert!(t_mid > 0.2 && t_mid < 0.8, "mid progress, got {t_mid}");
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("thumb", 297.0, 149.0),
            "thumb should sit ON the curve near the projected vertex"
        );

        // The x->time mapping killer: pointer far right at (420, 250).
        // Projection picks the nearest curve point (~[343, 259]) instead of
        // clamping to the end like a linear x mapping would.
        sm.post_event(&Event::PointerMove { x: 420.0, y: 250.0 });
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("thumb", 343.0, 259.0),
            "thumb should project to the nearest curve point, not the end"
        );
        let t_proj = sm.get_numeric_input("path_t").unwrap();
        assert!(
            t_proj > t_mid && t_proj < 0.9,
            "projection progress, got {t_proj}"
        );

        // Drag to the end and release: progress ~1 -> unlocked, thumb green
        // at the curve's end, clock pinned to 1.0.
        sm.post_event(&Event::PointerMove { x: 400.0, y: 340.0 });
        sm.post_event(&Event::PointerUp { x: 400.0, y: 340.0 });
        sm.tick(350.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "unlocked");
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("thumb", 396.0, 338.0),
            "thumb should rest at the curve's end"
        );

        // Grab again, release early: glides home via the state tween.
        sm.post_event(&Event::PointerDown { x: 396.0, y: 338.0 });
        assert_eq!(sm.get_current_state_name(), "sliding");
        sm.post_event(&Event::PointerMove { x: 100.0, y: 100.0 });
        sm.post_event(&Event::PointerUp { x: 100.0, y: 100.0 });
        sm.tick(350.0).unwrap();
        assert_eq!(sm.get_current_state_name(), "idle");
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("thumb", 72.0, 70.0),
            "thumb should glide home to the curve start"
        );
    }
}
