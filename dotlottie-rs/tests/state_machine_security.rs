#[cfg(test)]
mod tests {
    use dotlottie_rs::DotLottiePlayer;

    #[test]
    fn check_guards_for_existing_inputs() {
        let global_state = include_str!("fixtures/statemachines/security_tests/compare_to.json");
        let mut player = DotLottiePlayer::new(0);
        assert_eq!(
            player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100),
            Ok(())
        );

        let global_state_cstring =
            std::ffi::CString::new(global_state).expect("Invalid JSON for CString");

        let sm = player.state_machine_load(&global_state_cstring);

        assert!(sm.is_err());
    }

    #[test]
    fn check_states_for_guardless_transitions() {
        let global_state =
            include_str!("fixtures/statemachines/security_tests/guardless_transitions.json");
        let mut player = DotLottiePlayer::new(0);
        assert_eq!(
            player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100),
            Ok(())
        );
        let global_state_cstring =
            std::ffi::CString::new(global_state).expect("Invalid JSON for CString");

        let sm = player.state_machine_load(&global_state_cstring);

        assert!(sm.is_err());
    }

    #[test]
    fn check_states_for_existing_events() {
        let global_state = include_str!("fixtures/statemachines/security_tests/event_guards.json");
        let mut player = DotLottiePlayer::new(0);
        assert_eq!(
            player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100),
            Ok(())
        );

        let global_state_cstring =
            std::ffi::CString::new(global_state).expect("Invalid JSON for CString");
        let sm = player.state_machine_load(&global_state_cstring);

        assert!(sm.is_err());
    }

    #[test]
    fn check_state_for_multiple_global() {
        let global_state = include_str!("fixtures/statemachines/security_tests/multi_global.json");
        let mut player = DotLottiePlayer::new(0);
        assert_eq!(
            player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100),
            Ok(())
        );

        let sm = player.state_machine_load_data(global_state);

        assert!(sm.is_err());
    }
}
