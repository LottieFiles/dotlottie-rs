#[cfg(test)]
mod tests {
    use dotlottie_rs::{Config, DotLottiePlayer};

    fn get_current_state_name(player: &DotLottiePlayer) -> String {
        player.state_machine_current_state()
    }

    #[test]
    pub fn not_equal_test() {
        let global_state = include_str!("fixtures/statemachines/guard_tests/equal_not_equal.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        player.state_machine_set_numeric_trigger("rating", 3.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_2");
    }

    #[test]
    pub fn equal_test() {
        let global_state = include_str!("fixtures/statemachines/guard_tests/equal_not_equal.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        player.state_machine_set_numeric_trigger("rating", 5.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_5");
    }

    #[test]
    pub fn greater_than() {
        let global_state = include_str!("fixtures/statemachines/guard_tests/greater_than.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        player.state_machine_set_numeric_trigger("rating", 5.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_2");

        player.state_machine_set_numeric_trigger("rating", 4.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_2");

        player.state_machine_set_numeric_trigger("rating", 3.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_2");

        player.state_machine_set_numeric_trigger("rating", 1.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_1");

        player.state_machine_set_numeric_trigger("rating", 2.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_1");
    }

    #[test]
    pub fn greater_than_or_equal() {
        let global_state =
            include_str!("fixtures/statemachines/guard_tests/greater_than_equal.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        player.state_machine_set_numeric_trigger("rating", 5.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_2");

        player.state_machine_set_numeric_trigger("rating", 1.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_1");

        player.state_machine_set_numeric_trigger("rating", 2.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_2");
    }

    #[test]
    pub fn less_than_equal() {
        let global_state = include_str!("fixtures/statemachines/guard_tests/less_than_equal.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_3");

        player.state_machine_set_numeric_trigger("rating", 5.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_5");

        player.state_machine_set_numeric_trigger("rating", 3.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_3");

        player.state_machine_set_numeric_trigger("rating", 5.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_5");

        player.state_machine_set_numeric_trigger("rating", 1.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_3");
    }

    #[test]
    pub fn less_than() {
        let global_state = include_str!("fixtures/statemachines/guard_tests/less_than.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_3");

        player.state_machine_set_numeric_trigger("rating", 5.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_5");

        player.state_machine_set_numeric_trigger("rating", 3.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_5");

        player.state_machine_set_numeric_trigger("rating", 1.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_3");
    }

    #[test]
    pub fn guardless_transition() {}

    // Todo cases
}
