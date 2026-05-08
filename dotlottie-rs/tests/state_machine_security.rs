#![cfg(feature = "state-machines")]
#[cfg(test)]
mod tests {
    use dotlottie_rs::{
        actions::open_url_policy::OpenUrlPolicy, ColorSpace, Player, StateMachineEngineStatus,
    };

    /// State-machine tests don't render, but `load_dotlottie_data` wires
    /// drawables into ThorVG eagerly and that step needs a canvas. A leaked
    /// 1×1 software target is the cheapest valid setup.
    fn make_test_player() -> Player {
        let buf: &'static mut [u32] = Box::leak(vec![0u32; 1].into_boxed_slice());
        let mut p = Player::new();
        p.set_sw_target(buf, 1, 1, ColorSpace::ABGR8888)
            .expect("set_sw_target");
        p
    }

    #[test]
    fn check_guards_for_existing_inputs() {
        let global_state = include_str!("../assets/statemachines/security_tests/compare_to.json");
        let mut player = make_test_player();
        assert!(player
            .load_dotlottie_data(
                include_bytes!("../assets/animations/dotlottie/v1/star_rating.lottie").to_vec()
            )
            .is_ok(),);

        let global_state_cstring =
            std::ffi::CString::new(global_state).expect("Invalid JSON for CString");

        let sm = player.state_machine_load(&global_state_cstring);

        assert!(sm.is_err());
    }

    #[test]
    fn check_states_for_guardless_transitions() {
        let global_state =
            include_str!("../assets/statemachines/security_tests/guardless_transitions.json");
        let mut player = make_test_player();
        assert!(player
            .load_dotlottie_data(
                include_bytes!("../assets/animations/dotlottie/v1/star_rating.lottie").to_vec()
            )
            .is_ok(),);
        let global_state_cstring =
            std::ffi::CString::new(global_state).expect("Invalid JSON for CString");

        let sm = player.state_machine_load(&global_state_cstring);

        assert!(sm.is_err());
    }

    #[test]
    fn check_states_for_existing_events() {
        let global_state = include_str!("../assets/statemachines/security_tests/event_guards.json");
        let mut player = make_test_player();
        assert!(player
            .load_dotlottie_data(
                include_bytes!("../assets/animations/dotlottie/v1/star_rating.lottie").to_vec()
            )
            .is_ok(),);

        let global_state_cstring =
            std::ffi::CString::new(global_state).expect("Invalid JSON for CString");
        let sm = player.state_machine_load(&global_state_cstring);

        assert!(sm.is_err());
    }

    #[test]
    fn check_state_for_multiple_global() {
        let global_state = include_str!("../assets/statemachines/security_tests/multi_global.json");
        let mut player = make_test_player();
        assert!(player
            .load_dotlottie_data(
                include_bytes!("../assets/animations/dotlottie/v1/star_rating.lottie").to_vec()
            )
            .is_ok(),);

        let sm = player.state_machine_load_data(global_state);

        assert!(sm.is_err());
    }

    #[test]
    fn check_infinite_loop() {
        let global_state =
            include_str!("../assets/statemachines/security_tests/infinite_loop.json");
        let mut player = make_test_player();
        assert!(player
            .load_dotlottie_data(
                include_bytes!("../assets/animations/dotlottie/v1/smiley-slider.lottie").to_vec()
            )
            .is_ok(),);

        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert_eq!(r, Ok(()));

        sm.set_numeric_input("rating", 3.0, true, false);
        // rating==3 takes the `Tweened` transition (2s duration) into
        // star_3. The previously-asserted `Stopped` outcome only happened
        // when the player wasn't fully loaded (a separate, since-fixed bug
        // where load_dotlottie_data silently ignored renderer failures);
        // tween actions then failed and the SM eventually hit the cycle
        // limit. With a properly set-up player the tween starts cleanly.
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);
    }
}
