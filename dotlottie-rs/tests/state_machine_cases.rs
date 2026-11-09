#![cfg(feature = "state-machines")]
#[cfg(test)]
mod tests {
    use dotlottie_rs::{state_machine::OpenUrlPolicy, Player};

    #[test]
    pub fn global_and_guardless() {
        let global_state =
            include_str!("../assets/statemachines/sanity_tests/test_global_and_guardless.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok(),);
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");

        let l = sm.start(&OpenUrlPolicy::default());

        assert_eq!(l, Ok(()));

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        sm.set_numeric_input("Rating", 2.0, true, false)
            .expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        sm.set_numeric_input("Rating", 1.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "d");
    }

    #[test]
    pub fn guarded_and_guardless() {
        let global_state =
            include_str!("../assets/statemachines/sanity_tests/test_guarded_and_guardless.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok(),);
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");

        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "a");

        sm.fire("Step", true).expect("event to fire successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "d");

        sm.set_numeric_input("r", 1.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "a");

        sm.fire("Step", true).expect("event to fire successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "g");
    }

    #[test]
    pub fn guardless_and_event() {
        let global_state =
            include_str!("../assets/statemachines/sanity_tests/test_guardless_and_event.json");
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok(),);
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");

        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        sm.set_numeric_input("Rating", 1.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
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
            "../assets/statemachines/sanity_tests/test_exit_action_causes_global_to_transition.json"
        );
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok(),);
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");

        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "Initial");

        sm.set_numeric_input("rating", 1.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "a");

        // Even though the transition to C is valid, the exit action causes a re-evaluation of the global state
        // This re-evaluation causes the state machine to transition to state 'a' because rating is still 1.0
        sm.fire("Step", true).expect("event to fire successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "a");

        // So that the global state doesn't set us back in 'a'
        sm.set_numeric_input("rating", 4.0, true, false)
            .expect("input to set successfully");
        // A's exit action will toggle a_exited to true, allowing us to go to d
        sm.set_boolean_input("a_exited", false, true, false)
            .expect("input to set successfully");

        sm.fire("Step", true).expect("event to fire successfully");
        let curr_state_name = sm.get_current_state_name();

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
            "../assets/statemachines/sanity_tests/test_exit_action_global_ignored_if_non_valid.json"
        );
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok(),);
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");

        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "Initial");

        sm.set_numeric_input("rating", 1.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "a");

        // Leaving a, an exit action is triggered that sets rating to 3.0
        // Since non of the global state transitions are valid, we continue with the transition
        // To c
        sm.fire("Step", true).expect("event to fire successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "c");

        // Once on c, we set rating to 3.0
        // Since still none of the global state transitions are valid, we stay on c
        sm.set_numeric_input("rating", 3.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
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
            "../assets/statemachines/sanity_tests/test_entry_action_causes_global_to_transition.json"
        );
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok(),);
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");

        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "Initial");

        // State a has a transition with no guards, meaning that it should be taken by default
        // However it also has an entry action that validates a global state transition to b
        // This global state transition is taken, and the state machine transitions to b
        sm.set_numeric_input("rating", 1.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        // Modifying the rating to which has no valid global transitions to check that
        // the state machine remains in the current state
        sm.set_numeric_input("rating", 3.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        sm.fire("Step", true).expect("event to fire successfully");

        let curr_state_name = sm.get_current_state_name();
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
            "../assets/statemachines/sanity_tests/test_entry_action_global_ignored_if_non_valid.json"
        );
        let mut player = Player::new();
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok(),);
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");

        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "Initial");

        sm.set_numeric_input("rating", 1.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "a");

        // Leaving a, an exit action is triggered that sets rating to 3.0
        // Since non of the global state transitions are valid, we continue with the transition
        // To c
        sm.fire("Step", true).expect("event to fire successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "c");

        // Once on c, we set rating to 3.0
        // Since still none of the global state transitions are valid, we stay on c
        sm.set_boolean_input("a_exited", true, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "d");
    }

    const PENDING_EVENT_SM: &str = r#"{
        "initial": "waiting",
        "states": [
            {"type": "PlaybackState", "name": "waiting", "animation": "", "transitions": [
                {"type": "Transition", "toState": "aborted", "guards": [
                    {"type": "Boolean", "inputName": "abort", "conditionType": "Equal", "compareTo": true}]},
                {"type": "Transition", "toState": "done", "guards": [
                    {"type": "Event", "inputName": "loaded"},
                    {"type": "Boolean", "inputName": "boundary", "conditionType": "Equal", "compareTo": true}]},
                {"type": "Transition", "toState": "failed", "guards": [
                    {"type": "Event", "inputName": "error"},
                    {"type": "Boolean", "inputName": "boundary", "conditionType": "Equal", "compareTo": true}]}
            ]},
            {"type": "PlaybackState", "name": "aborted", "animation": "", "transitions": [
                {"type": "Transition", "toState": "done", "guards": [
                    {"type": "Event", "inputName": "loaded"}]}
            ]},
            {"type": "PlaybackState", "name": "done", "animation": "", "transitions": []},
            {"type": "PlaybackState", "name": "failed", "animation": "", "transitions": []}
        ],
        "interactions": [],
        "inputs": [
            {"type": "Boolean", "name": "boundary", "value": false},
            {"type": "Boolean", "name": "abort", "value": false},
            {"type": "Event", "name": "loaded"},
            {"type": "Event", "name": "error"}
        ]
    }"#;

    fn start_pending_event_sm(player: &mut Player) -> dotlottie_rs::StateMachineEngine<'_> {
        let mut sm = player
            .state_machine_load_data(PENDING_EVENT_SM)
            .expect("state machine to load successfully");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));
        assert_eq!(sm.get_current_state_name(), "waiting");
        sm
    }

    #[test]
    pub fn event_stays_pending_until_consumed() {
        let mut player = Player::new();
        let mut sm = start_pending_event_sm(&mut player);

        sm.fire("loaded", true).expect("event to fire successfully");
        assert_eq!(sm.get_current_state_name(), "waiting");

        sm.set_boolean_input("boundary", true, true, false)
            .expect("input to set successfully");
        assert_eq!(sm.get_current_state_name(), "done");
    }

    #[test]
    pub fn newer_event_replaces_pending_event() {
        let mut player = Player::new();
        let mut sm = start_pending_event_sm(&mut player);

        sm.fire("loaded", true).expect("event to fire successfully");
        sm.fire("error", true).expect("event to fire successfully");
        assert_eq!(sm.get_current_state_name(), "waiting");

        sm.set_boolean_input("boundary", true, true, false)
            .expect("input to set successfully");
        assert_eq!(sm.get_current_state_name(), "failed");
    }

    #[test]
    pub fn pending_event_is_dropped_when_state_exits() {
        let mut player = Player::new();
        let mut sm = start_pending_event_sm(&mut player);

        sm.fire("loaded", true).expect("event to fire successfully");
        sm.set_boolean_input("abort", true, true, false)
            .expect("input to set successfully");
        assert_eq!(sm.get_current_state_name(), "aborted");

        sm.set_boolean_input("boundary", true, true, false)
            .expect("input to set successfully");
        assert_eq!(sm.get_current_state_name(), "aborted");

        sm.fire("loaded", true).expect("event to fire successfully");
        assert_eq!(sm.get_current_state_name(), "done");
    }

    #[test]
    pub fn pending_event_is_served_before_input_routing() {
        let sm_json = r#"{
            "initial": "a",
            "states": [
                {"type": "PlaybackState", "name": "a", "animation": "", "transitions": [
                    {"type": "Transition", "toState": "by_input", "guards": [
                        {"type": "Boolean", "inputName": "flag", "conditionType": "Equal", "compareTo": true}]},
                    {"type": "Transition", "toState": "by_event", "guards": [
                        {"type": "Event", "inputName": "go"},
                        {"type": "Boolean", "inputName": "flag", "conditionType": "Equal", "compareTo": true}]}
                ]},
                {"type": "PlaybackState", "name": "by_input", "animation": "", "transitions": []},
                {"type": "PlaybackState", "name": "by_event", "animation": "", "transitions": []}
            ],
            "inputs": [
                {"type": "Boolean", "name": "flag", "value": false},
                {"type": "Event", "name": "go"}
            ]
        }"#;
        let mut player = Player::new();
        let mut sm = player
            .state_machine_load_data(sm_json)
            .expect("state machine to load successfully");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        sm.fire("go", true).expect("event to fire successfully");
        sm.set_boolean_input("flag", true, true, false)
            .expect("input to set successfully");
        assert_eq!(sm.get_current_state_name(), "by_event");
    }

    #[test]
    pub fn pending_event_is_dropped_on_stop() {
        let mut player = Player::new();
        let mut sm = start_pending_event_sm(&mut player);

        sm.fire("loaded", true).expect("event to fire successfully");
        sm.stop();
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        sm.set_boolean_input("boundary", true, true, false)
            .expect("input to set successfully");
        assert_eq!(sm.get_current_state_name(), "waiting");
    }
}
