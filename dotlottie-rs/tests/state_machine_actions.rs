#[cfg(test)]
mod tests {
    use dotlottie_rs::{
        Config, DotLottiePlayer, StateMachineEvent, actions::open_url_policy::OpenUrlPolicy
    };

    #[test]
    fn increment() {
        let global_state = include_str!("fixtures/statemachines/action_tests/inc_rating.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        // Tests default increment without a value
        sm.set_numeric_input("rating", 1.0, true, false).expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");

        // Tests adding with value
        sm.set_numeric_input("rating", 3.0, true, false).expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_5");

        // Tests add from a input
        sm.set_numeric_input("rating", 6.0, true, false).expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_12");

        // Tests add from a inexistant input, increments by 1.0 instead
        sm.set_numeric_input("rating", 13.0, true, false).expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_14");
    }

    #[test]
    fn decrement() {
        let global_state = include_str!("fixtures/statemachines/action_tests/decr_rating.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        // Tests decrement from an inexistant input, decrements by 1.0 instead
        sm.set_numeric_input("rating", 13.0, true, false).expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_12");

        // Tests decrement from a input
        sm.set_numeric_input("rating", 6.0, true, false).expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_6");

        // Tests decrementing with value
        sm.set_numeric_input("rating", 3.0, true, false).expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_1");

        // Tests default increment without a value
        sm.set_numeric_input("rating", 5.0, true, false).expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_4");
    }

    #[test]
    fn toggle() {
        let global_state = include_str!("fixtures/statemachines/action_tests/toggle.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        sm.fire("Step", true).expect("event to fire successfully");

        // C state should of toggled the switch to true, landing us in state a
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "a");

        sm.fire("Step", true).expect("event to fire successfully");

        // C state should of toggled the switch to false, landing us in state b
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");
    }

    #[test]
    fn set_boolean() {
        let global_state = include_str!("fixtures/statemachines/action_tests/set_inputs.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        sm.fire("Step", true).expect("event to fire successfully");

        // C state should of set the switch to true, landing us in state a
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "a");
    }

    #[test]
    fn set_numeric() {
        let global_state = include_str!("fixtures/statemachines/action_tests/set_inputs.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        sm.set_numeric_input("NumericInput", 10.0, true, false).expect("input to set successfully");

        // C state should of set the switch to true, landing us in state a
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "d");
    }

    #[test]
    fn set_string() {
        let global_state = include_str!("fixtures/statemachines/action_tests/set_inputs.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        sm.set_string_input("StringInput", "second", true, false).expect("input to set successfully");

        // C state should of set the switch to true, landing us in state a
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "f");
    }

    #[test]
    fn fire() {
        let global_state = include_str!("fixtures/statemachines/action_tests/fire.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        sm.set_boolean_input("OnOffSwitch", true, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "c");
    }

    #[test]
    fn set_frame() {
        let global_state = include_str!("fixtures/statemachines/action_tests/set_frame.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        assert_eq!(player.current_frame(), 0.0);

        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        // B Should of set the frame to input value "frame_holder", meaning 35
        assert_eq!(sm.player.current_frame(), 35.0);

        sm.set_boolean_input("OnOffSwitch", true, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "a");

        // A Should of set the frame to 10
        assert_eq!(player.current_frame(), 10.0);
    }

    #[test]
    fn set_progress() {
        let global_state = include_str!("fixtures/statemachines/action_tests/set_progress.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        assert_eq!(player.current_frame(), 0.0);

        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        // B Should of set the frame to input value "frame_holder", 75% of the animation
        assert_eq!(sm.player.current_frame(), 66.75);

        sm.set_boolean_input("OnOffSwitch", true, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "a");

        // A Should of set the progress to 10%
        assert_eq!(player.current_frame(), 8.900001);
    }

    #[test]
    fn reset() {
        let reset_sm = include_str!("fixtures/statemachines/action_tests/reset.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        assert_eq!(player.current_frame(), 0.0);

        let mut sm = player.state_machine_load_data(reset_sm).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        sm.set_numeric_input("rating", 3.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");

        sm.set_numeric_input("rating", 6.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_0");
    }

    #[test]
    fn fire_custom_event() {
        let reset_sm = include_str!("fixtures/statemachines/action_tests/fire_custom_event.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        assert_eq!(player.current_frame(), 0.0);

        let mut sm = player.state_machine_load_data(reset_sm).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        sm.set_numeric_input("rating", 3.0, true, false).expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");

        let expected_events = [
            // on start events
            "on_transition:  -> global".to_string(),
            "on_state_exit: ".to_string(),
            "on_state_entered: global".to_string(),
            "on_transition: global -> star_0".to_string(),
            "on_state_exit: global".to_string(),
            "on_state_entered: star_0".to_string(),
            "custom_event: WOOHOO STAR 0".to_string(),

            // our interactions related
            "on_transition: star_0 -> star_3".to_string(),
            "on_state_exit: star_0".to_string(),
            "on_state_entered: star_3".to_string(),
            "custom_event: WOOHOO STAR 3".to_string(),
        ];

        let mut events = vec![];

        while let Some(event) = sm.poll_event() {
            let event_str: Option<String> = match event {
                StateMachineEvent::Transition { previous_state, new_state } => {
                    Some(format!("on_transition: {} -> {}", previous_state, new_state))
                }
                StateMachineEvent::StateEntered { state } => {
                    Some(format!("on_state_entered: {}", state))
                }
                StateMachineEvent::StateExit { state } => {
                    Some(format!("on_state_exit: {}", state))
                }
                StateMachineEvent::CustomEvent { message } => {
                    Some(format!("custom_event: {}", message))
                }
                _ => None            
            };

            if let Some(event_str) = event_str {
                events.push(event_str.to_string());
            }
        }



        for (i, event) in events.iter().enumerate() {
            assert_eq!(
                event, &expected_events[i],
                "Mismatch at event index {}: expected '{}', found '{}'",
                i, expected_events[i], event
            );
        }
    }

    #[test]
    fn open_url() {
        // todo!()
    }

    #[test]
    fn set_slot() {
        // todo!()
    }

    #[test]
    fn set_theme() {
        // todo!()
    }

    #[test]
    fn set_expression() {
        // todo!()
    }
}
