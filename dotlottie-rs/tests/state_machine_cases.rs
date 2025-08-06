#[cfg(test)]
mod tests {
    use dotlottie_rs::{actions::open_url_policy::OpenUrlPolicy, Config, DotLottiePlayer};

    fn get_current_state_name(player: &DotLottiePlayer) -> String {
        player.state_machine_current_state()
    }

    #[test]
    pub fn global_and_guardless() {
        let global_state =
            include_str!("fixtures/statemachines/sanity_tests/test_global_and_guardless.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/smileys.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);

        let s = player.state_machine_start(OpenUrlPolicy::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        player.state_machine_set_numeric_input("Rating", 2.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        player.state_machine_set_numeric_input("Rating", 1.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "d");
    }

    #[test]
    pub fn guarded_and_guardless() {
        let global_state =
            include_str!("fixtures/statemachines/sanity_tests/test_guarded_and_guardless.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/smileys.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);

        let s = player.state_machine_start(OpenUrlPolicy::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");

        player.state_machine_fire_event("Step");

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "d");

        player.state_machine_set_numeric_input("r", 1.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");

        player.state_machine_fire_event("Step");
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "g");
    }

    #[test]
    pub fn guardless_and_event() {
        let global_state =
            include_str!("fixtures/statemachines/sanity_tests/test_guardless_and_event.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/smileys.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);

        let s = player.state_machine_start(OpenUrlPolicy::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        player.state_machine_set_numeric_input("Rating", 1.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "c");
    }

    /**
     * Feature tested:
     * If there is an exit action that takes place when leaving a state, the global
     * state is re-evaluated after the exit action is executed. If there is a valid transition
     * from the global state, it is taken. Otherwise the state machine remains in the current state.
     */
    #[test]
    pub fn exit_action_causes_global_to_transition() {
        let global_state = include_str!(
            "fixtures/statemachines/sanity_tests/test_exit_action_causes_global_to_transition.json"
        );
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/smileys.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);

        let s = player.state_machine_start(OpenUrlPolicy::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "Initial");

        player.state_machine_set_numeric_input("rating", 1.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");

        // Even though the transition to C is valid, the exit action causes a re-evaluation of the global state
        // This re-evaluation causes the state machine to transition to state 'a' because rating is still 1.0
        player.state_machine_fire_event("Step");
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");

        // So that the global state doesn't set us back in 'a'
        player.state_machine_set_numeric_input("rating", 4.0);
        // A's exit action will toggle a_exited to true, allowing us to go to d
        player.state_machine_set_boolean_input("a_exited", false);

        player.state_machine_fire_event("Step");
        let curr_state_name = get_current_state_name(&player);

        // Why is this e and not d
        assert_eq!(curr_state_name, "d");
    }

    /**
     * Feature tested:
     * If there is an exit action that takes place when leaving a state, but non o the global state
     * transitions are valid, we maintain position on the current state / decided transition.
     */
    #[test]
    pub fn exit_action_ignored_if_non_valid() {
        let global_state = include_str!(
            "fixtures/statemachines/sanity_tests/test_exit_action_global_ignored_if_non_valid.json"
        );
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/smileys.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);

        let s = player.state_machine_start(OpenUrlPolicy::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "Initial");

        player.state_machine_set_numeric_input("rating", 1.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");

        // Leaving a, an exit action is triggered that sets rating to 3.0
        // Since non of the global state transitions are valid, we continue with the transition
        // To c
        player.state_machine_fire_event("Step");
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "c");

        // Once on c, we set rating to 3.0
        // Since still none of the global state transitions are valid, we stay on c
        player.state_machine_set_numeric_input("rating", 3.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "c");
    }

    /**
     * Feature tested:
     * If there is an entry action that takes place when entering a state, the global
     * state is re-evaluated after the entry action is executed. If there is a valid transition
     * from the global state, it is taken. Otherwise the state machine remains in the current state.
     */
    #[test]
    pub fn entry_action_causes_global_to_transition() {
        let global_state = include_str!(
            "fixtures/statemachines/sanity_tests/test_entry_action_causes_global_to_transition.json"
        );
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/smileys.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);

        let s = player.state_machine_start(OpenUrlPolicy::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "Initial");

        // State a has a transition with no guards, meaning that it should be taken by default
        // However it also has an entry action that validates a global state transition to b
        // This global state transition is taken, and the state machine transitions to b
        player.state_machine_set_numeric_input("rating", 1.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        // Modifying the rating to which has no valid global transitions to check that
        // the state machine remains in the current state
        player.state_machine_set_numeric_input("rating", 3.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        player.state_machine_fire_event("Step");

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "d");
    }

    /**
     * Feature tested:
     * If there is an entry action that takes place when entering a state, but non of the global state
     * transitions are valid, we maintain position on the current state / decided transition.
     */
    #[test]
    pub fn entry_action_ignored_if_non_valid() {
        let global_state = include_str!(
            "fixtures/statemachines/sanity_tests/test_entry_action_global_ignored_if_non_valid.json"
        );
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/smileys.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);

        let s = player.state_machine_start(OpenUrlPolicy::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "Initial");

        player.state_machine_set_numeric_input("rating", 1.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");

        // Leaving a, an exit action is triggered that sets rating to 3.0
        // Since non of the global state transitions are valid, we continue with the transition
        // To c
        player.state_machine_fire_event("Step");
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "c");

        // Once on c, we set rating to 3.0
        // Since still none of the global state transitions are valid, we stay on c
        player.state_machine_set_boolean_input("a_exited", true);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "d");
    }
}
