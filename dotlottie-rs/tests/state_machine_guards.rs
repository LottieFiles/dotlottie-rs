#[cfg(test)]
mod tests {
    use dotlottie_rs::{actions::open_url_policy::OpenUrlPolicy, Config, DotLottiePlayer};

    #[test]
    pub fn not_equal_test() {
        let global_state = include_str!("fixtures/statemachines/guard_tests/equal_not_equal.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        sm.set_numeric_input("rating", 3.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");
    }

    #[test]
    pub fn equal_test() {
        let global_state = include_str!("fixtures/statemachines/guard_tests/equal_not_equal.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        sm.set_numeric_input("rating", 5.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_5");
    }

    #[test]
    pub fn greater_than() {
        let global_state = include_str!("fixtures/statemachines/guard_tests/greater_than.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        sm.set_numeric_input("rating", 5.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");

        sm.set_numeric_input("rating", 4.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");

        sm.set_numeric_input("rating", 3.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");

        sm.set_numeric_input("rating", 1.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_1");

        sm.set_numeric_input("rating", 2.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_1");
    }

    #[test]
    pub fn greater_than_or_equal() {
        let global_state =
            include_str!("fixtures/statemachines/guard_tests/greater_than_equal.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        sm.set_numeric_input("rating", 5.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");

        sm.set_numeric_input("rating", 1.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_1");

        sm.set_numeric_input("rating", 2.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");
    }

    #[test]
    pub fn less_than_equal() {
        let global_state = include_str!("fixtures/statemachines/guard_tests/less_than_equal.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");

        sm.set_numeric_input("rating", 5.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_5");

        sm.set_numeric_input("rating", 3.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");

        sm.set_numeric_input("rating", 5.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_5");

        sm.set_numeric_input("rating", 1.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");
    }

    #[test]
    pub fn less_than() {
        let global_state = include_str!("fixtures/statemachines/guard_tests/less_than.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");

        sm.set_numeric_input("rating", 5.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_5");

        sm.set_numeric_input("rating", 3.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_5");

        sm.set_numeric_input("rating", 1.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");
    }

    #[test]
    pub fn guardless_transition() {}

    // Todo cases
}
