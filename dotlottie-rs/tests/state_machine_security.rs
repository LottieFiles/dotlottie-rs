#[cfg(test)]
mod tests {
    use dotlottie_rs::{Config, DotLottiePlayer};

    #[test]
    fn check_guards_for_existing_triggers() {
        let global_state = include_str!("fixtures/statemachines/security_tests/compare_to.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        let l = player.state_machine_load(global_state);
        let s = player.state_machine_start();

        assert!(!l);
        assert!(!s);
    }

    #[test]
    fn check_states_for_guardless_transitions() {
        let global_state =
            include_str!("fixtures/statemachines/security_tests/guardless_transitions.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        let l = player.state_machine_load(global_state);
        let s = player.state_machine_start();

        assert!(!l);
        assert!(!s);
    }

    #[test]
    fn check_states_for_existing_events() {
        let global_state = include_str!("fixtures/statemachines/security_tests/event_guards.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        let l = player.state_machine_load(global_state);
        let s = player.state_machine_start();

        assert!(!l);
        assert!(!s);
    }
}
