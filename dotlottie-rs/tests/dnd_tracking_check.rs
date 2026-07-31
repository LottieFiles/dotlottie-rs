#![cfg(feature = "state-machines")]

// Engine-driven tracking docks (assets/statemachines/star_drop_track.json
// against star_drop_moving.json, whose drop_zone glides [100,100] ->
// [400,100] over frames 0..150 at 30fps).
//
// Docking on a `track: true` zone is instant (a glide cannot chase a
// moving endpoint), then the engine FOLLOWS the zone: each tick it reads
// the zone's rendered center and rewrites the position slot. No Lottie
// expressions, no jerryscript — and because the engine always knows the
// object's position, a tracked object can be re-grabbed (grab un-docks).
// The follow runs one canvas update behind the zone, which is invisible
// at interactive tick rates but means tests settle with an extra tick.

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::events::Event;
    use dotlottie_rs::{ColorSpace, Player};

    #[test]
    fn tracking_dock_follows_moving_zone_and_regrabs() {
        let definition = include_str!("../assets/statemachines/star_drop_track.json");
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

        // Sanity: zone starts near [100,100]; star rests at [138.5, 392.5].
        assert!(sm.player.hit_check("drop_zone", 100.0, 100.0));
        assert!(sm.player.hit_check("Star 2", 138.5, 392.5));

        // Grab the star and release it over the zone's CURRENT position.
        // Track docks land instantly: zone actions run at release.
        sm.post_event(&Event::PointerDown { x: 138.5, y: 392.5 });
        sm.post_event(&Event::PointerMove { x: 105.0, y: 102.0 });
        sm.post_event(&Event::PointerUp { x: 105.0, y: 102.0 });
        assert_eq!(
            sm.get_numeric_input("docked_count"),
            Some(1.0),
            "tracking dock should run zone actions"
        );

        // Play ~2.5s: the zone glides to roughly frame 76 -> x ~ 252.
        sm.tick(2500.0).unwrap();
        // The follow reads the zone one canvas update behind: settle.
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();

        // The zone really is there...
        assert!(
            sm.player.hit_check("drop_zone", 252.0, 100.0),
            "zone should have animated to ~[252, 100]"
        );
        // ...and the STAR is there too: the engine followed the zone.
        assert!(
            sm.player.hit_check("Star 2", 252.0, 100.0),
            "star should follow the moving zone via per-tick bounds reads"
        );
        // And it is NOT where it was dropped anymore.
        assert!(
            !sm.player.hit_check("Star 2", 105.0, 300.0),
            "sanity: star is not somewhere unrelated"
        );
        assert!(
            !sm.player.hit_check("Star 2", 138.5, 392.5),
            "star should have left its authored rest position"
        );

        // Re-grab WORKS now: the engine knows where the tracked object is,
        // so grabbing un-docks it and the drag continues normally.
        sm.post_event(&Event::PointerDown { x: 252.0, y: 100.0 });
        sm.post_event(&Event::PointerMove { x: 400.0, y: 400.0 });
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("Star 2", 400.0, 400.0),
            "re-grabbed star should follow the pointer again"
        );
    }

    // Same behavior against the creator-authored animation (star_drop.json),
    // whose zone ping-pongs [96.5,109.5] -> [413.5,113.5] @ frame 73 -> back.
    // Keyframe endpoints are easing-independent, so frame ~73 is a
    // deterministic assertion point.
    #[test]
    fn tracking_dock_follows_creator_authored_zone() {
        let definition = include_str!("../assets/statemachines/star_drop_track.json");
        let mut buffer: Vec<u32> = vec![0; 512 * 512];
        let mut player = Player::new();
        player
            .set_sw_target(&mut buffer, 512, 512, ColorSpace::ABGR8888)
            .unwrap();
        let path = CString::new("assets/animations/lottie/star_drop.json").unwrap();
        player.load_animation_path(&path).unwrap();

        let mut sm = player.state_machine_load_data(definition).unwrap();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        sm.tick(16.0).unwrap();

        // Dock onto the zone near its frame-0 position.
        sm.post_event(&Event::PointerDown { x: 138.5, y: 392.5 });
        sm.post_event(&Event::PointerMove { x: 100.0, y: 110.0 });
        sm.post_event(&Event::PointerUp { x: 100.0, y: 110.0 });
        sm.tick(16.0).unwrap();
        assert_eq!(sm.get_numeric_input("docked_count"), Some(1.0));

        // Advance to ~frame 73, the zone's right apex (the ping-pong is
        // slow near the apex, so the settle ticks stay in its vicinity).
        sm.tick(2385.0).unwrap();
        sm.tick(16.0).unwrap();
        sm.tick(16.0).unwrap();
        assert!(
            sm.player.hit_check("drop_zone", 413.5, 113.5),
            "zone should be at its right apex"
        );
        assert!(
            sm.player.hit_check("Star 2", 413.5, 113.5),
            "star should ride the creator-authored zone"
        );
    }
}
