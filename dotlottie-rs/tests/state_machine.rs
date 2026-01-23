#[cfg(test)]
mod tests {
    use core::assert_eq;
    use std::fs::{self, File};

    use dotlottie_rs::{Config, DotLottiePlayer, Event, StateMachineEngineStatus, actions::open_url_policy::OpenUrlPolicy};
    use std::io::Read;

    #[test]
    fn get_state_machine() {
        let config = Config {
            autoplay: true,
            ..Config::default()
        };
        let mut player = DotLottiePlayer::new(config, 0);

        let mut markers =
            File::open("tests/fixtures/statemachines/normal_usecases/sm_exploding_pigeon.lottie")
                .expect("no file found");
        let metadatamarkers =
            fs::metadata("tests/fixtures/statemachines/normal_usecases/sm_exploding_pigeon.lottie")
                .expect("unable to read metadata");
        let mut markers_buffer = vec![0; metadatamarkers.len() as usize];
        markers
            .read_exact(&mut markers_buffer)
            .expect("buffer overflow");

        player.load_dotlottie_data(&markers_buffer, 500, 500);

        assert!(player.is_playing());

        let mut sm = player.state_machine_load("Exploding Pigeon").expect("state machine to load successfully");

        assert!(sm.start(&OpenUrlPolicy::default()));

        // Tests with a state machine loaded
        let global_state =
            include_str!("fixtures/statemachines/normal_usecases/exploding_pigeon.json");

        let l = player.get_state_machine("Exploding Pigeon").expect("to return a state machine json");

        assert_eq!(l, global_state);
    }

    #[test]
    fn state_machine_start() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);

        let sm = player.state_machine_load_data("bad_data");

        assert!(sm.is_err());

        let global_state = include_str!("fixtures/statemachines/action_tests/inc_rating.json");
        let mut sm2 = player.state_machine_load_data(global_state).expect("state machine to load successfully");

        let r = sm2.start(&OpenUrlPolicy::default());

        assert!(r);
    }

    #[test]
    fn state_machine_stop() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);

        let sm = player.state_machine_load_data("bad_data");

        // Should not load
        assert!(sm.is_err());

        let global_state = include_str!("fixtures/statemachines/action_tests/inc_rating.json");
        let mut sm2 = player.state_machine_load_data(global_state).expect("state machine to load successfully");

        let r = sm2.start(&OpenUrlPolicy::default());
        sm2.stop();

        assert!(r);
        assert!(sm2.status == StateMachineEngineStatus::Stopped);
    }

    #[test]
    fn state_machine_framework_setup() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        let pointer_down =
            include_str!("fixtures/statemachines/interaction_tests/interaction_array.json");

        let mut sm = player.state_machine_load_data(pointer_down).expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert!(r);

        let r = sm.framework_setup();

        assert!(r.contains(&"PointerDown".to_string()));
        assert!(r.contains(&"PointerUp".to_string()));
        assert!(r.contains(&"PointerMove".to_string()));
        assert!(r.contains(&"PointerEnter".to_string()));
        assert!(r.contains(&"PointerExit".to_string()));
        assert!(r.contains(&"OnComplete".to_string()));
    }

    #[test]
    fn state_machine_post_event() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        let pointer_down =
            include_str!("fixtures/statemachines/interaction_tests/all_interaction_events.json");

        let mut sm = player.state_machine_load_data(pointer_down).expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert!(r);

        let event = Event::PointerDown { x: 0.0, y: 0.0 };
        sm.post_event(&event);
        assert_eq!(sm.get_current_state_name(), "a".to_string());

        let event = Event::PointerUp { x: 0.0, y: 0.0 };
        sm.post_event(&event);
        assert_eq!(sm.get_current_state_name(), "b".to_string());

        let event = Event::PointerMove { x: 0.0, y: 0.0 };
        sm.post_event(&event);
        assert_eq!(sm.get_current_state_name(), "c".to_string());

        let event = Event::PointerEnter { x: 0.0, y: 0.0 };
        sm.post_event(&event);
        assert_eq!(sm.get_current_state_name(), "d".to_string());

        let event = Event::PointerExit { x: 0.0, y: 0.0 };
        sm.post_event(&event);
        assert_eq!(sm.get_current_state_name(), "e".to_string());

        let event = Event::OnComplete;
        sm.post_event(&event);
        assert_eq!(sm.get_current_state_name(), "f".to_string());
    }

    #[test]
    fn state_machine_set_get_numeric_input() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        let rating = include_str!("fixtures/statemachines/normal_usecases/rating.json");

        let mut sm = player.state_machine_load_data(rating).expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert!(r);

        // Setting the inputs
        sm.set_numeric_input("rating", 1.0, true, false);
        assert_eq!(sm.get_current_state_name(), "star_1".to_string());

        assert_eq!(sm.get_numeric_input("rating").expect("to get numeric input"), 1.0);

        sm.set_numeric_input("rating", 5.0, true, false);
        assert_eq!(sm.get_current_state_name(), "star_5".to_string());

        assert_eq!(sm.get_numeric_input("rating").expect("to get numeric input"), 5.0);
    }

    #[test]
    fn state_machine_set_get_boolean_input() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        let sm = include_str!("fixtures/statemachines/normal_usecases/toggle.json");

        let mut sm = player.state_machine_load_data(sm).expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert!(r);

        assert!(!sm.get_boolean_input("OnOffSwitch").expect("to get boolean input"));

        // Setting the inputs
        sm.set_boolean_input("OnOffSwitch", true, true, false);
        assert_eq!(sm.get_current_state_name(), "a".to_string());
        assert!(sm.get_boolean_input("OnOffSwitch").expect("to get boolean input"));

        sm.set_boolean_input("OnOffSwitch", false, true, false);
        assert_eq!(sm.get_current_state_name(), "b".to_string());
        assert!(!sm.get_boolean_input("OnOffSwitch").expect("to get boolean input"));
    }

    #[test]
    fn state_machine_set_get_string_input() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        let sm = include_str!("fixtures/statemachines/normal_usecases/password.json");

        let mut sm = player.state_machine_load_data(sm).expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert!(r);

        assert_eq!(
            sm.get_string_input("password").expect("to get string input"),
            "incorrect".to_string()
        );

        // Setting the inputs
        sm.set_string_input("password", "welcome", true, false);
        assert_eq!(sm.get_current_state_name(), "a".to_string());
        assert_eq!(
            sm.get_string_input("password").expect("to get string input"),
            "welcome".to_string()
        );

        sm.set_string_input("password", "goodbye", true, false);
        assert_eq!(sm.get_current_state_name(), "b".to_string());
        assert_eq!(
            sm.get_string_input("password").expect("to get string input"),
            "goodbye".to_string()
        );
    }

    #[test]
    fn state_machine_fire_event() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        let sm = include_str!("fixtures/statemachines/normal_usecases/password_with_events.json");

        let mut sm = player.state_machine_load_data(sm).expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert!(r);

        sm.fire("Step", true).expect("event to fire successfully");
        assert_eq!(sm.get_current_state_name(), "a".to_string());

        sm.fire("Step", true).expect("event to fire successfully");
        assert_eq!(sm.get_current_state_name(), "b".to_string());
    }

    #[test]
    fn final_state() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        let sm = include_str!("fixtures/statemachines/normal_usecases/final_state.json");

        let mut sm = player.state_machine_load_data(sm).expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert!(r);

        assert_eq!(sm.get_current_state_name(), "star_0".to_string());

        sm.set_numeric_input("rating", 3.0, true, false);
        assert_eq!(sm.get_current_state_name(), "star_3".to_string());

        sm.set_numeric_input("rating", 5.0, true, false);
        assert_eq!(sm.get_current_state_name(), "star_1".to_string());

        sm.set_numeric_input("rating", 3.0, true, false);
        assert_eq!(sm.get_current_state_name(), "star_1".to_string());
    }

    #[test]
    fn state_machine_current_state() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        let pointer_down =
            include_str!("fixtures/statemachines/interaction_tests/all_interaction_events.json");

        let mut sm = player.state_machine_load_data(pointer_down).expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert!(r);

        let event = Event::PointerDown { x: 0.0, y: 0.0 };
        sm.post_event(&event);
        assert_eq!(sm.get_current_state_name(), "a".to_string());

        let event = Event::PointerUp { x: 0.0, y: 0.0 };
        sm.post_event(&event);
        assert_eq!(sm.get_current_state_name(), "b".to_string());

        let event = Event::PointerMove { x: 0.0, y: 0.0 };
        sm.post_event(&event);
        assert_eq!(sm.get_current_state_name(), "c".to_string());

        let event = Event::PointerEnter { x: 0.0, y: 0.0 };
        sm.post_event(&event);
        assert_eq!(sm.get_current_state_name(), "d".to_string());

        let event = Event::PointerExit { x: 0.0, y: 0.0 };
        sm.post_event(&event);
        assert_eq!(sm.get_current_state_name(), "e".to_string());

        let event = Event::OnComplete;
        sm.post_event(&event);
        assert_eq!(sm.get_current_state_name(), "f".to_string());
    }

    #[test]
    fn state_machine_get_inputs() {
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        let pointer_down =
            include_str!("fixtures/statemachines/sanity_tests/test_get_all_inputs.json");

        let mut sm = player.state_machine_load_data(pointer_down).expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert!(r);

        let predefined_inputs = [
            "a_exited", "Boolean", "Step", "Event", "rating", "Numeric", "b_exited", "String",
        ];

        let inputs = sm.get_inputs();

        // Check that the lengths match
        assert_eq!(
            inputs.len(),
            predefined_inputs.len(),
            "Length mismatch: got {} elements, expected {}",
            inputs.len(),
            predefined_inputs.len()
        );

        assert_eq!(
            inputs.len() % 2,
            0,
            "Input array must have even length (key-value pairs)"
        );

        // Convert both arrays into sets of (key, type) pairs
        let mut input_pairs: Vec<(&str, &str)> = inputs
            .chunks_exact(2)
            .map(|chunk| (chunk[0].as_str(), chunk[1].as_str()))
            .collect();

        let mut predefined_pairs: Vec<(&str, &str)> = predefined_inputs
            .chunks_exact(2)
            .map(|chunk| (chunk[0], chunk[1]))
            .collect();

        // Sort both for comparison
        input_pairs.sort();
        predefined_pairs.sort();

        assert_eq!(input_pairs, predefined_pairs);
    }
}
