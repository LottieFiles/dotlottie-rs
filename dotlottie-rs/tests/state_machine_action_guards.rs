#![cfg(feature = "state-machines")]
#[cfg(test)]
mod tests {
    use dotlottie_rs::{actions::open_url_policy::OpenUrlPolicy, Player};

    const LOTTIE: &[u8] =
        include_bytes!("../assets/animations/dotlottie/v1/star_rating.lottie");

    fn load() -> Player {
        let mut player = Player::new();
        assert!(player.load_dotlottie_data(LOTTIE).is_ok());
        player
    }

    #[test]
    fn entry_actions_pass_fail_and_live_read() {
        let json =
            include_str!("../assets/statemachines/action_guard_tests/gated_entry_actions.json");
        let mut player = load();
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");
        sm.start(&OpenUrlPolicy::default()).expect("start");

        // Sanity: initial state is `a`, no entry actions have run.
        assert_eq!(sm.get_current_state_name(), "a");
        assert_eq!(sm.get_numeric_input("result_a"), Some(0.0));

        // Trigger the transition into `b`, which runs the gated entry actions.
        sm.set_boolean_input("proceed", true, true, false);
        assert_eq!(sm.get_current_state_name(), "b");

        // A and C ran (unguarded).
        assert_eq!(sm.get_numeric_input("result_a"), Some(100.0));
        assert_eq!(sm.get_numeric_input("result_c"), Some(100.0));

        // B was skipped because `gate` is false. Skip-and-continue means C still ran.
        assert_eq!(sm.get_numeric_input("result_b"), Some(0.0));

        // Live read: the Increment ran before the guarded SetBoolean, so its guard
        // sees live_counter=1 and fires.
        assert_eq!(sm.get_numeric_input("live_counter"), Some(1.0));
        assert_eq!(sm.get_boolean_input("done"), Some(true));

        // AND semantics: two guards, second fails, action is skipped.
        assert_eq!(sm.get_boolean_input("skipped"), Some(false));
    }

    #[test]
    fn exit_action_respects_guard() {
        let json =
            include_str!("../assets/statemachines/action_guard_tests/gated_exit_action.json");
        let mut player = load();
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");
        sm.start(&OpenUrlPolicy::default()).expect("start");

        // Cause the transition from `a` to `b`, executing `a`'s exit actions.
        sm.set_boolean_input("go", true, true, false);
        assert_eq!(sm.get_current_state_name(), "b");

        // Guarded exit action skipped because `exit_gate` is false.
        assert_eq!(sm.get_numeric_input("exit_skipped"), Some(0.0));
        // Subsequent unguarded exit action still ran (skip-and-continue).
        assert_eq!(sm.get_numeric_input("exit_ran"), Some(1.0));
    }

    #[test]
    fn interaction_action_guard_with_live_read() {
        use dotlottie_rs::Event;
        let json =
            include_str!("../assets/statemachines/action_guard_tests/gated_interaction_action.json");
        let mut player = load();
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");
        sm.start(&OpenUrlPolicy::default()).expect("start");

        assert_eq!(sm.get_current_state_name(), "a");
        assert_eq!(sm.get_boolean_input("armed"), Some(false));

        // PointerDown runs:
        //   1. SetBoolean armed=true (unguarded)
        //   2. Fire clicked, guarded by armed==true  -> passes due to live read
        // Firing `clicked` then drives the Event-guarded transition into `b`.
        sm.post_event(&Event::PointerDown { x: 0.0, y: 0.0 });

        assert_eq!(sm.get_boolean_input("armed"), Some(true));
        assert_eq!(sm.get_current_state_name(), "b");
    }

    #[test]
    fn action_guard_compare_to_input_reference() {
        let json = include_str!(
            "../assets/statemachines/action_guard_tests/input_reference_guard.json"
        );
        let mut player = load();
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");
        sm.start(&OpenUrlPolicy::default()).expect("start");

        // lhs == rhs (both 7) → guarded action fires.
        sm.set_boolean_input("go", true, true, false);
        assert_eq!(sm.get_current_state_name(), "b");
        assert_eq!(sm.get_numeric_input("ran"), Some(1.0));
    }

    #[test]
    fn action_guard_compare_to_input_reference_mismatch() {
        let json = include_str!(
            "../assets/statemachines/action_guard_tests/input_reference_guard.json"
        );
        let mut player = load();
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");
        sm.start(&OpenUrlPolicy::default()).expect("start");

        // Make lhs != rhs before the transition fires; the guarded entry action
        // should be skipped on entering `b`.
        sm.set_numeric_input("rhs", 99.0, false, false);
        sm.set_boolean_input("go", true, true, false);
        assert_eq!(sm.get_current_state_name(), "b");
        assert_eq!(sm.get_numeric_input("ran"), Some(0.0));
    }

    #[test]
    fn event_guard_on_action_is_rejected_at_load() {
        let json =
            include_str!("../assets/statemachines/action_guard_tests/event_guard_on_action.json");
        let mut player = Player::new();
        assert!(player.load_dotlottie_data(LOTTIE).is_ok());

        let sm = player.state_machine_load_data(json);
        assert!(
            sm.is_err(),
            "expected load failure when an Event guard is used on an action"
        );
    }

    #[test]
    fn undeclared_input_in_action_guard_is_rejected_at_load() {
        let json = include_str!(
            "../assets/statemachines/action_guard_tests/undeclared_input_in_action_guard.json"
        );
        let mut player = Player::new();
        assert!(player.load_dotlottie_data(LOTTIE).is_ok());

        let sm = player.state_machine_load_data(json);
        assert!(
            sm.is_err(),
            "expected load failure when an action guard references an undeclared input"
        );
    }
}
