#[cfg(test)]
mod tests {
    use dotlottie_rs::{actions::open_url_policy::OpenUrlPolicy, Config, DotLottiePlayer};

    #[test]
    fn check_guards_for_existing_inputs() {
        let global_state = include_str!("fixtures/statemachines/security_tests/compare_to.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        let mut sm = player.state_machine_load(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(!s);
    }

    #[test]
    fn check_states_for_guardless_transitions() {
        let global_state =
            include_str!("fixtures/statemachines/security_tests/guardless_transitions.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        let mut sm = player.state_machine_load(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(!s);
    }

    #[test]
    fn check_states_for_existing_events() {
        let global_state = include_str!("fixtures/statemachines/security_tests/event_guards.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        let mut sm = player.state_machine_load(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(!s);
    }

    #[test]
    fn check_state_for_multiple_global() {
        let global_state = include_str!("fixtures/statemachines/security_tests/multi_global.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(!s);
    }
}
