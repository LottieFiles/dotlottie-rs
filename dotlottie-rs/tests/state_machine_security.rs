#[cfg(test)]
mod tests {
    use dotlottie_rs::{states::StateTrait, Config, DotLottiePlayer};

    #[test]
    fn check_guards_for_existing_triggers() {
        let global_state = include_str!("fixtures/statemachines/security_tests/compare_to.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        let l = player.load_state_machine_data(global_state);
        let s = player.start_state_machine();

        assert_eq!(l, false);
        assert_eq!(s, false);
    }

    #[test]
    fn check_states_for_guardless_transitions() {
        let global_state =
            include_str!("fixtures/statemachines/security_tests/guardless_transitions.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        let l = player.load_state_machine_data(global_state);
        let s = player.start_state_machine();

        assert_eq!(l, false);
        assert_eq!(s, false);
    }

    #[test]
    fn check_states_for_existing_events() {
        let global_state = include_str!("fixtures/statemachines/security_tests/event_guards.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        let l = player.load_state_machine_data(global_state);
        let s = player.start_state_machine();

        assert_eq!(l, false);
        assert_eq!(s, false);
    }
}
