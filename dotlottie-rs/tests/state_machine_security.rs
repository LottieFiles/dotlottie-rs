#![cfg(feature = "state-machines")]
#[cfg(test)]
mod tests {
    use dotlottie_rs::{actions::open_url_policy::OpenUrlPolicy, Player, StateMachineEngineStatus};

    #[test]
    fn check_guards_for_existing_inputs() {
        let global_state = include_str!("../assets/statemachines/security_tests/compare_to.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
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
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok(),);
        let global_state_cstring =
            std::ffi::CString::new(global_state).expect("Invalid JSON for CString");

        let sm = player.state_machine_load(&global_state_cstring);

        assert!(sm.is_err());
    }

    #[test]
    fn check_states_for_existing_events() {
        let global_state = include_str!("../assets/statemachines/security_tests/event_guards.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok(),);

        let global_state_cstring =
            std::ffi::CString::new(global_state).expect("Invalid JSON for CString");
        let sm = player.state_machine_load(&global_state_cstring);

        assert!(sm.is_err());
    }

    #[test]
    fn check_state_for_multiple_global() {
        let global_state = include_str!("../assets/statemachines/security_tests/multi_global.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok(),);

        let sm = player.state_machine_load_data(global_state);

        assert!(sm.is_err());
    }

    #[test]
    fn check_infinite_loop() {
        let global_state =
            include_str!("../assets/statemachines/security_tests/infinite_loop.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok(),);

        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert_eq!(r, Ok(()));

        sm.set_numeric_input("rating", 3.0, true, false);
        assert_eq!(sm.status, StateMachineEngineStatus::Stopped);
    }
}
