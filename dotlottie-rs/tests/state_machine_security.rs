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
    fn rejects_user_declared_elapsed_time_input() {
        let json =
            include_str!("../assets/statemachines/security_tests/reserved_input_declaration.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok());

        let sm = player.state_machine_load_data(json);
        assert!(sm.is_err());
    }

    #[test]
    fn rejects_set_numeric_targeting_elapsed_time() {
        let json = include_str!("../assets/statemachines/security_tests/reserved_set_numeric.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok());

        let sm = player.state_machine_load_data(json);
        assert!(sm.is_err());
    }

    #[test]
    fn rejects_increment_targeting_elapsed_time() {
        let json = include_str!("../assets/statemachines/security_tests/reserved_increment.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok());

        let sm = player.state_machine_load_data(json);
        assert!(sm.is_err());
    }

    #[test]
    fn allows_reset_action_targeting_elapsed_time() {
        // reset_action.json uses Action::Reset { elapsedTime } in entry actions.
        let json = include_str!("../assets/statemachines/elapsed_time_tests/reset_action.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok());

        let sm = player.state_machine_load_data(json);
        assert!(sm.is_ok());
    }

    #[test]
    fn allows_guard_referencing_elapsed_time() {
        // timeout_transition.json uses elapsedTime as a guard input_name.
        let json =
            include_str!("../assets/statemachines/elapsed_time_tests/timeout_transition.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok());

        let sm = player.state_machine_load_data(json);
        assert!(sm.is_ok());
    }

    #[test]
    fn allows_guard_using_dollar_elapsed_time_compare_to() {
        // compare_to_elapsed.json uses "$elapsedTime" in compareTo.
        let json =
            include_str!("../assets/statemachines/elapsed_time_tests/compare_to_elapsed.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok());

        let sm = player.state_machine_load_data(json);
        assert!(sm.is_ok());
    }
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
