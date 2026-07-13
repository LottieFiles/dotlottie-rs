#![cfg(feature = "state-machines")]
#[cfg(test)]
mod tests {
    use core::assert_eq;
    use std::ffi::CString;
    use std::fs::{self, File};

    use dotlottie_rs::{
        actions::open_url_policy::OpenUrlPolicy, ColorSpace, Event, Player, StateMachineEngine,
        StateMachineEngineStatus, StateMachineEvent, Status,
    };
    use std::io::Read;

    #[test]
    fn get_state_machine() {
        let mut player = Player::new();
        player.set_autoplay(true);

        let mut buffer: Vec<u32> = vec![0; (500 * 500) as usize];

        assert!(player
            .set_sw_target(&mut buffer, 500, 500, ColorSpace::ABGR8888,)
            .is_ok());

        let mut markers =
            File::open("assets/statemachines/normal_usecases/sm_exploding_pigeon.lottie")
                .expect("no file found");
        let metadatamarkers =
            fs::metadata("assets/statemachines/normal_usecases/sm_exploding_pigeon.lottie")
                .expect("unable to read metadata");
        let mut markers_buffer = vec![0; metadatamarkers.len() as usize];
        markers
            .read_exact(&mut markers_buffer)
            .expect("buffer overflow");

        assert_eq!(player.load_dotlottie_data(&markers_buffer), Ok(()));

        assert_eq!(player.status(), Status::Playing);

        let sm_id = CString::new("Exploding Pigeon").unwrap();
        let mut sm = player
            .state_machine_load(&sm_id)
            .expect("state machine to load successfully");

        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        // Tests with a state machine loaded
        let global_state =
            include_str!("../assets/statemachines/normal_usecases/exploding_pigeon.json");

        let l = player
            .get_state_machine(&sm_id)
            .expect("to return a state machine json");

        assert_eq!(l, global_state);
    }

    #[test]
    fn state_machine_start() {
        let mut player = Player::new();

        let sm = player.state_machine_load_data("bad_data");

        assert!(sm.is_err());

        let global_state = include_str!("../assets/statemachines/action_tests/inc_rating.json");
        let mut sm2 = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");

        let r = sm2.start(&OpenUrlPolicy::default());

        assert_eq!(r, Ok(()));
    }

    #[test]
    fn state_machine_stop() {
        let mut player = Player::new();

        let sm = player.state_machine_load_data("bad_data");

        // Should not load
        assert!(sm.is_err());

        let global_state = include_str!("../assets/statemachines/action_tests/inc_rating.json");
        let mut sm2 = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");

        let r = sm2.start(&OpenUrlPolicy::default());
        sm2.stop();

        assert_eq!(r, Ok(()));
        assert!(sm2.status == StateMachineEngineStatus::Stopped);
    }

    #[test]
    fn state_machine_framework_setup() {
        let mut player = Player::new();
        let pointer_down =
            include_str!("../assets/statemachines/interaction_tests/interaction_array.json");

        let mut sm = player
            .state_machine_load_data(pointer_down)
            .expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert_eq!(r, Ok(()));

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
        let mut player = Player::new();
        let pointer_down =
            include_str!("../assets/statemachines/interaction_tests/all_interaction_events.json");

        let mut sm = player
            .state_machine_load_data(pointer_down)
            .expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert_eq!(r, Ok(()));

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
        let mut player = Player::new();
        let rating = include_str!("../assets/statemachines/normal_usecases/rating.json");

        let mut sm = player
            .state_machine_load_data(rating)
            .expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert_eq!(r, Ok(()));

        // Setting the inputs
        sm.set_numeric_input("rating", 1.0, true, false);
        assert_eq!(sm.get_current_state_name(), "star_1".to_string());

        assert_eq!(
            sm.get_numeric_input("rating")
                .expect("to get numeric input"),
            1.0
        );

        sm.set_numeric_input("rating", 5.0, true, false);
        assert_eq!(sm.get_current_state_name(), "star_5".to_string());

        assert_eq!(
            sm.get_numeric_input("rating")
                .expect("to get numeric input"),
            5.0
        );
    }

    #[test]
    fn state_machine_set_get_boolean_input() {
        let mut player = Player::new();
        let sm = include_str!("../assets/statemachines/toggle.json");

        let mut sm = player
            .state_machine_load_data(sm)
            .expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert_eq!(r, Ok(()));

        assert!(!sm
            .get_boolean_input("OnOffSwitch")
            .expect("to get boolean input"));

        // Setting the inputs
        sm.set_boolean_input("OnOffSwitch", true, true, false);
        assert_eq!(sm.get_current_state_name(), "a".to_string());
        assert!(sm
            .get_boolean_input("OnOffSwitch")
            .expect("to get boolean input"));

        sm.set_boolean_input("OnOffSwitch", false, true, false);
        assert_eq!(sm.get_current_state_name(), "b".to_string());
        assert!(!sm
            .get_boolean_input("OnOffSwitch")
            .expect("to get boolean input"));
    }

    #[test]
    fn state_machine_set_get_string_input() {
        let mut player = Player::new();
        let sm = include_str!("../assets/statemachines/normal_usecases/password.json");

        let mut sm = player
            .state_machine_load_data(sm)
            .expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert_eq!(r, Ok(()));

        assert_eq!(
            sm.get_string_input("password")
                .expect("to get string input"),
            "incorrect".to_string()
        );

        // Setting the inputs
        sm.set_string_input("password", "welcome", true, false);
        assert_eq!(sm.get_current_state_name(), "a".to_string());
        assert_eq!(
            sm.get_string_input("password")
                .expect("to get string input"),
            "welcome".to_string()
        );

        sm.set_string_input("password", "goodbye", true, false);
        assert_eq!(sm.get_current_state_name(), "b".to_string());
        assert_eq!(
            sm.get_string_input("password")
                .expect("to get string input"),
            "goodbye".to_string()
        );
    }

    #[test]
    fn state_machine_fire_event() {
        let mut player = Player::new();
        let sm = include_str!("../assets/statemachines/normal_usecases/password_with_events.json");

        let mut sm = player
            .state_machine_load_data(sm)
            .expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert_eq!(r, Ok(()));

        sm.fire("Step", true).expect("event to fire successfully");
        assert_eq!(sm.get_current_state_name(), "a".to_string());

        sm.fire("Step", true).expect("event to fire successfully");
        assert_eq!(sm.get_current_state_name(), "b".to_string());
    }

    #[test]
    fn final_state() {
        let mut player = Player::new();
        let sm = include_str!("../assets/statemachines/normal_usecases/final_state.json");

        let mut sm = player
            .state_machine_load_data(sm)
            .expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert_eq!(r, Ok(()));

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
        let mut player = Player::new();
        let pointer_down =
            include_str!("../assets/statemachines/interaction_tests/all_interaction_events.json");

        let mut sm = player
            .state_machine_load_data(pointer_down)
            .expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert_eq!(r, Ok(()));

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
    fn tweened_transition_retargets_when_player_already_tweening() {
        let sm_json = include_str!("../assets/statemachines/smileys.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok());

        let mut sm = player
            .state_machine_load_data(sm_json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");

        assert_eq!(sm.get_current_state_name(), "star_1");

        // Pre-start a tween; the state machine transition must retarget it
        sm.player
            .tween(5.0, 2000.0, [0.0, 0.0, 1.0, 1.0])
            .expect("initial tween should succeed");
        assert_eq!(sm.player.status(), Status::Tweening);

        sm.set_numeric_input("rating", 2.0, true, false)
            .expect("input should be set");

        assert_eq!(
            sm.status,
            StateMachineEngineStatus::Tweening,
            "the manual tween is retargeted by the tweened transition"
        );

        let _ = sm.tick(2500.0);
        assert_eq!(sm.get_current_state_name(), "star_2");
        assert_eq!(sm.status, StateMachineEngineStatus::Running);
    }

    #[test]
    fn tweened_transition_to_state_without_segment_should_tween() {
        let sm_json = include_str!("../assets/statemachines/tween_no_segment.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok());

        let mut sm = player
            .state_machine_load_data(sm_json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");

        assert_eq!(sm.get_current_state_name(), "segment_state");

        sm.set_numeric_input("trigger", 1.0, true, false)
            .expect("input should be set");

        assert_eq!(
            sm.status,
            StateMachineEngineStatus::Tweening,
            "state machine should be in Tweening status when tweened transition targets a state without a segment"
        );
        assert_eq!(
            sm.player.status(),
            Status::Tweening,
            "player should be tweening after a tweened transition to a state without a segment"
        );

        assert_eq!(
            sm.get_current_state_name(),
            "segment_state",
            "current state should still be the source state while tweening"
        );
    }

    #[test]
    fn stop_during_tweened_transition_aborts_transition() {
        let sm_json = include_str!("../assets/statemachines/tween_no_segment.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok());

        let mut sm = player
            .state_machine_load_data(sm_json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");

        assert_eq!(sm.get_current_state_name(), "segment_state");

        sm.set_numeric_input("trigger", 1.0, true, false)
            .expect("input should be set");

        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);
        assert_eq!(sm.player.status(), Status::Tweening);

        sm.player.stop().expect("stop during tween should succeed");
        assert_eq!(sm.player.status(), Status::Stopped);
        let frame_after_stop = sm.player.current_frame();

        let _ = sm.tick(16.0);

        assert_eq!(
            sm.status,
            StateMachineEngineStatus::Running,
            "engine should leave Tweening after the tween is cancelled"
        );
        assert_eq!(
            sm.get_current_state_name(),
            "segment_state",
            "cancelled tweened transition must not enter the target state"
        );
        assert_eq!(
            sm.player.status(),
            Status::Stopped,
            "stop must stick: no autoplay restart from the aborted transition"
        );
        assert_eq!(
            sm.player.current_frame(),
            frame_after_stop,
            "frame must not teleport to the tween target"
        );
    }

    #[test]
    fn tweened_transition_to_reverse_state_with_segment_should_tween() {
        let sm_json = include_str!("../assets/statemachines/tween_reverse_segment.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok());

        let mut sm = player
            .state_machine_load_data(sm_json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");

        assert_eq!(sm.get_current_state_name(), "forward_state");

        sm.set_numeric_input("trigger", 1.0, true, false)
            .expect("input should be set");

        assert_eq!(
            sm.status,
            StateMachineEngineStatus::Tweening,
            "state machine should be in Tweening status for a reverse mode state with segment"
        );
        assert_eq!(
            sm.player.status(),
            Status::Tweening,
            "player should be tweening"
        );

        assert_eq!(sm.get_current_state_name(), "forward_state");

        // Pass enough dt to complete the 1ms tween
        let _ = sm.tick(10.0);

        assert_eq!(
            sm.get_current_state_name(),
            "reverse_state",
            "should have transitioned to reverse_state after tween completed"
        );
        assert_eq!(
            sm.status,
            StateMachineEngineStatus::Running,
            "status should be Running after tween completes"
        );
    }

    #[test]
    fn tweened_transition_to_reverse_state_without_segment_should_tween() {
        let sm_json = include_str!("../assets/statemachines/tween_reverse_segment.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok());

        let mut sm = player
            .state_machine_load_data(sm_json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");

        assert_eq!(sm.get_current_state_name(), "forward_state");

        sm.set_numeric_input("trigger", 2.0, true, false)
            .expect("input should be set");

        assert_eq!(
            sm.status,
            StateMachineEngineStatus::Tweening,
            "state machine should be in Tweening status for a reverse mode state without segment"
        );
        assert_eq!(
            sm.player.status(),
            Status::Tweening,
            "player should be tweening"
        );

        assert_eq!(sm.get_current_state_name(), "forward_state");

        // Pass enough dt to complete the 1ms tween
        let _ = sm.tick(10.0);

        assert_eq!(
            sm.get_current_state_name(),
            "reverse_no_segment_state",
            "should have transitioned to reverse_no_segment_state after tween completed"
        );
        assert_eq!(
            sm.status,
            StateMachineEngineStatus::Running,
            "status should be Running after tween completes"
        );

        let expected_target = sm.player.total_frames() - 1.0;
        assert_eq!(
            sm.player.current_frame(),
            expected_target,
            "current_frame should be at tween target (last frame) after tween to reverse state, not the pre-tween frame"
        );
    }

    #[test]
    fn state_machine_get_inputs() {
        let mut player = Player::new();
        let pointer_down =
            include_str!("../assets/statemachines/sanity_tests/test_get_all_inputs.json");

        let mut sm = player
            .state_machine_load_data(pointer_down)
            .expect("state machine to load successfully");

        let r = sm.start(&OpenUrlPolicy::default());
        assert_eq!(r, Ok(()));

        let predefined_inputs = [
            "a_exited",
            "Boolean",
            "Step",
            "Event",
            "rating",
            "Numeric",
            "b_exited",
            "String",
            // Built-in: every state machine exposes the @elapsedTime input.
            "@elapsedTime",
            "Numeric",
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
        use std::collections::HashSet;

        let expected_set: HashSet<(&str, &str)> = predefined_inputs
            .chunks(2)
            .map(|chunk| (chunk[0], chunk[1]))
            .collect();

        let actual_set: HashSet<(&str, &str)> = inputs
            .chunks(2)
            .map(|chunk| (chunk[0].as_str(), chunk[1].as_str()))
            .collect();

        assert_eq!(
            actual_set, expected_set,
            "Input key-value pairs do not match"
        );
    }

    fn setup_tween_redirection_sm(player: &mut Player) -> StateMachineEngine<'_> {
        let sm_json = include_str!("../assets/statemachines/tween_redirection.json");

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        std::mem::forget(buffer);

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok());

        let mut sm = player
            .state_machine_load_data(sm_json)
            .expect("state machine to load successfully");
        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");
        assert_eq!(sm.get_current_state_name(), "star_1");

        // Drain startup events so tests only see what they trigger.
        while sm.poll_event().is_some() {}
        sm
    }

    fn drain_entered_states(sm: &mut StateMachineEngine<'_>) -> Vec<String> {
        let mut entered = Vec::new();
        while let Some(event) = sm.poll_event() {
            if let StateMachineEvent::StateEntered { state } = event {
                entered.push(state.as_str().to_owned());
            }
        }
        entered
    }

    #[test]
    fn inputs_are_processed_during_tween_and_retarget() {
        let mut player = Player::new();
        let mut sm = setup_tween_redirection_sm(&mut player);

        sm.set_numeric_input("rating", 2.0, true, false)
            .expect("input should be set");
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);
        assert_eq!(sm.get_current_state_name(), "star_1");

        let _ = sm.tick(500.0);
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        let result = sm.set_numeric_input("rating", 3.0, true, false);
        assert!(
            result.is_some(),
            "inputs must be processed during a tween, got None"
        );
        assert_eq!(
            sm.status,
            StateMachineEngineStatus::Tweening,
            "a tweened interrupt keeps the machine tweening"
        );
        assert_eq!(sm.get_current_state_name(), "star_1");

        let _ = sm.tick(2500.0);
        assert_eq!(sm.get_current_state_name(), "star_3");
        assert_eq!(sm.status, StateMachineEngineStatus::Running);

        let entered = drain_entered_states(&mut sm);
        assert!(
            entered.contains(&"star_3".to_string()),
            "final target must be entered, got {entered:?}"
        );
        assert!(
            !entered.contains(&"star_2".to_string()),
            "interrupted target must be skipped entirely, got {entered:?}"
        );
    }

    #[test]
    fn instant_transition_interrupts_tween() {
        let mut player = Player::new();
        let mut sm = setup_tween_redirection_sm(&mut player);

        sm.set_numeric_input("rating", 2.0, true, false)
            .expect("input should be set");
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);
        let _ = sm.tick(500.0);

        sm.set_numeric_input("rating", 5.0, true, false)
            .expect("input should be set");
        assert_eq!(sm.get_current_state_name(), "star_5");
        assert_eq!(sm.status, StateMachineEngineStatus::Running);
        assert!(!sm.player.is_tweening(), "tween must be cancelled");

        let entered = drain_entered_states(&mut sm);
        assert!(entered.contains(&"star_5".to_string()));
        assert!(!entered.contains(&"star_2".to_string()));
    }

    #[test]
    fn chained_interrupts_land_on_last_target() {
        let mut player = Player::new();
        let mut sm = setup_tween_redirection_sm(&mut player);

        sm.set_numeric_input("rating", 2.0, true, false)
            .expect("input should be set");
        let _ = sm.tick(400.0);
        sm.set_numeric_input("rating", 3.0, true, false)
            .expect("input should be set");
        let _ = sm.tick(400.0);
        sm.set_numeric_input("rating", 4.0, true, false)
            .expect("input should be set");
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        let _ = sm.tick(2500.0);
        assert_eq!(sm.get_current_state_name(), "star_4");
        assert_eq!(sm.status, StateMachineEngineStatus::Running);

        let entered = drain_entered_states(&mut sm);
        assert_eq!(
            entered,
            vec!["star_4".to_string()],
            "only the final target is ever entered"
        );
    }

    /// Center of mass of the rendered blob, in pixels. The demo animation draws a
    /// bright shape on a dark background, so thresholding on luminance isolates it.
    fn blob_centroid(buffer: &[u32], size: usize) -> (f32, f32) {
        let (mut sx, mut sy, mut n) = (0.0f32, 0.0f32, 0.0f32);
        for y in 0..size {
            for x in 0..size {
                let px = buffer[y * size + x];
                let (r, g, b) = (px & 0xFF, (px >> 8) & 0xFF, (px >> 16) & 0xFF);
                if r + g + b > 300 {
                    sx += x as f32;
                    sy += y as f32;
                    n += 1.0;
                }
            }
        }
        assert!(n > 0.0, "nothing bright rendered");
        (sx / n, sy / n)
    }

    /// The demo bundle animates entirely through tweening: every state is a single
    /// static pose (autoplay off), so all motion on screen comes from the tween.
    #[test]
    fn tween_poses_demo_moves_and_redirects_mid_tween() {
        const SIZE: usize = 300;

        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];
        assert!(player
            .set_sw_target(&mut buffer, SIZE as u32, SIZE as u32, ColorSpace::ABGR8888)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/statemachines/normal_usecases/tween_poses.lottie"
            ))
            .is_ok());

        let sm_id = CString::new("tween_poses").unwrap();
        let mut sm = player
            .state_machine_load(&sm_id)
            .expect("embedded state machine should load");
        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");
        assert_eq!(sm.get_current_state_name(), "pose_center");

        let _ = sm.tick(0.0);
        let start = blob_centroid(&buffer, SIZE);

        sm.set_numeric_input("pose", 1.0, true, false)
            .expect("input should be set");
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);
        let _ = sm.tick(500.0);

        let mid_up = blob_centroid(&buffer, SIZE);
        assert!(
            mid_up.1 < start.1 - 20.0,
            "blob should have moved up mid-tween: start y={}, mid y={}",
            start.1,
            mid_up.1
        );

        sm.set_numeric_input("pose", 2.0, true, false)
            .expect("input must be accepted during a tween");
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        let _ = sm.tick(500.0);
        let mid_right = blob_centroid(&buffer, SIZE);
        assert!(
            mid_right.0 > mid_up.0 + 20.0,
            "blob should be heading right after the interrupt: was x={}, now x={}",
            mid_up.0,
            mid_right.0
        );

        let _ = sm.tick(1500.0);
        assert_eq!(sm.get_current_state_name(), "pose_right");
        assert_eq!(sm.status, StateMachineEngineStatus::Running);

        let landed = blob_centroid(&buffer, SIZE);
        assert!(
            landed.0 > mid_right.0,
            "blob should finish further right than mid-tween: {} -> {}",
            mid_right.0,
            landed.0
        );
        assert!(
            (landed.1 - start.1).abs() < 25.0,
            "pose_right sits at the vertical center, got y={}",
            landed.1
        );

        let entered = drain_entered_states(&mut sm);
        assert!(
            entered.contains(&"pose_right".to_string()),
            "got {entered:?}"
        );
        assert!(!entered.contains(&"pose_up".to_string()), "got {entered:?}");
    }

    /// The source state stays current for the duration of a tween, so this looks like
    /// a self-transition unless the tween target is taken into account.
    #[test]
    fn transition_back_to_the_source_state_interrupts_the_tween() {
        const SIZE: usize = 300;
        const UP: (f32, f32) = (150.0, 55.0);
        const CENTER: (f32, f32) = (150.0, 150.0);

        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];
        assert!(player
            .set_sw_target(&mut buffer, SIZE as u32, SIZE as u32, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/statemachines/normal_usecases/tween_poses.lottie"
            ))
            .is_ok());

        let sm_id = CString::new("tween_poses").unwrap();
        let mut sm = player
            .state_machine_load(&sm_id)
            .expect("embedded state machine should load");
        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");
        while sm.poll_event().is_some() {}

        sm.post_event(&Event::PointerDown { x: UP.0, y: UP.1 });
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);
        let _ = sm.tick(500.0);
        let mid = blob_centroid(&buffer, SIZE);
        assert!(mid.1 < 130.0, "ball should be on its way up, y={}", mid.1);

        // The centre is the state we are tweening away from.
        sm.post_event(&Event::PointerDown {
            x: CENTER.0,
            y: CENTER.1,
        });
        assert_eq!(sm.get_numeric_input("pose"), Some(0.0));
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        let _ = sm.tick(500.0);
        let coming_back = blob_centroid(&buffer, SIZE);
        assert!(
            coming_back.1 > mid.1 + 10.0,
            "ball should be heading back down to the centre: y {} -> {}",
            mid.1,
            coming_back.1
        );

        for _ in 0..120 {
            let _ = sm.tick(1000.0 / 60.0);
        }
        assert_eq!(sm.get_current_state_name(), "pose_center");
        assert_eq!(sm.status, StateMachineEngineStatus::Running);

        let entered = drain_entered_states(&mut sm);
        assert!(
            !entered.contains(&"pose_up".to_string()),
            "pose_up was abandoned mid-tween and must never be entered, got {entered:?}"
        );
    }

    /// Time keeps running during a tween, so an elapsedTime guard has to be able to
    /// interrupt one just like an input change can.
    #[test]
    fn elapsed_time_guard_interrupts_a_tween() {
        let sm_json = r#"{
            "initial": "pose_center",
            "states": [
                {
                    "name": "global",
                    "type": "GlobalState",
                    "animation": "",
                    "transitions": [
                        {
                            "type": "Transition",
                            "toState": "pose_right",
                            "guards": [
                                { "type": "Numeric", "conditionType": "GreaterThan",
                                  "inputName": "@elapsedTime", "compareTo": 1.0 }
                            ]
                        },
                        {
                            "type": "Tweened",
                            "toState": "pose_up",
                            "duration": 2.0,
                            "easing": [0.42, 0.0, 0.58, 1.0],
                            "guards": [
                                { "type": "Numeric", "conditionType": "Equal",
                                  "inputName": "pose", "compareTo": 1 }
                            ]
                        }
                    ]
                },
                { "type": "PlaybackState", "name": "pose_center", "animation": "",
                  "autoplay": false, "loop": false, "segment": "pose_center",
                  "transitions": [] },
                { "type": "PlaybackState", "name": "pose_up", "animation": "",
                  "autoplay": false, "loop": false, "segment": "pose_up",
                  "transitions": [] },
                { "type": "PlaybackState", "name": "pose_right", "animation": "",
                  "autoplay": false, "loop": false, "segment": "pose_right",
                  "transitions": [] }
            ],
            "interactions": [],
            "inputs": [{ "type": "Numeric", "name": "pose", "value": 0 }]
        }"#;

        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; 300 * 300];
        assert!(player
            .set_sw_target(&mut buffer, 300, 300, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/statemachines/normal_usecases/tween_poses.lottie"
            ))
            .is_ok());

        let mut sm = player
            .state_machine_load_data(sm_json)
            .expect("state machine to load successfully");
        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");

        sm.set_numeric_input("pose", 1.0, true, false)
            .expect("input should be set");
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        for _ in 0..3 {
            let _ = sm.tick(500.0);
        }

        assert_eq!(
            sm.get_current_state_name(),
            "pose_right",
            "the elapsedTime guard must interrupt the tween, not wait it out"
        );
        assert_eq!(sm.status, StateMachineEngineStatus::Running);
        assert!(!sm.player.is_tweening());
    }

    /// Cancelling a tween must drop the renderer-side tween too, or the scene stays
    /// frozen at the interrupted pose while the engine reports the new state.
    #[test]
    fn instant_transition_cancels_the_rendered_tween() {
        const SIZE: usize = 300;

        let sm_json = r#"{
            "initial": "pose_center",
            "states": [
                {
                    "name": "global",
                    "type": "GlobalState",
                    "animation": "",
                    "transitions": [
                        {
                            "type": "Tweened",
                            "toState": "pose_up",
                            "duration": 2.0,
                            "easing": [0.42, 0.0, 0.58, 1.0],
                            "guards": [
                                { "type": "Numeric", "conditionType": "Equal",
                                  "inputName": "pose", "compareTo": 1 }
                            ]
                        },
                        {
                            "type": "Transition",
                            "toState": "pose_center",
                            "guards": [
                                { "type": "Numeric", "conditionType": "Equal",
                                  "inputName": "pose", "compareTo": 0 }
                            ]
                        }
                    ]
                },
                { "type": "PlaybackState", "name": "pose_center", "animation": "",
                  "autoplay": false, "loop": false, "segment": "pose_center",
                  "transitions": [] },
                { "type": "PlaybackState", "name": "pose_up", "animation": "",
                  "autoplay": false, "loop": false, "segment": "pose_up",
                  "transitions": [] }
            ],
            "interactions": [],
            "inputs": [{ "type": "Numeric", "name": "pose", "value": 0 }]
        }"#;

        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];
        assert!(player
            .set_sw_target(&mut buffer, SIZE as u32, SIZE as u32, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/statemachines/normal_usecases/tween_poses.lottie"
            ))
            .is_ok());

        let mut sm = player
            .state_machine_load_data(sm_json)
            .expect("state machine to load successfully");
        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");

        let _ = sm.tick(0.0);
        let center = blob_centroid(&buffer, SIZE);

        sm.set_numeric_input("pose", 1.0, true, false)
            .expect("input should be set");
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);
        let _ = sm.tick(1000.0);
        let mid = blob_centroid(&buffer, SIZE);
        assert!(mid.1 < center.1 - 20.0, "ball should be on its way up");

        // pose_center's segment is already the active marker, so cancelling the tween
        // is the only thing that can put the ball back at the centre.
        sm.set_numeric_input("pose", 0.0, true, false)
            .expect("input should be set");
        assert_eq!(sm.get_current_state_name(), "pose_center");
        assert_eq!(sm.status, StateMachineEngineStatus::Running);
        assert!(!sm.player.is_tweening());

        let _ = sm.tick(1000.0 / 60.0);
        let after = blob_centroid(&buffer, SIZE);
        assert!(
            (after.1 - center.1).abs() < 2.0,
            "cancelling the tween must snap back to the centre pose (y={}), \
             not leave the scene frozen mid-tween (y={})",
            center.1,
            after.1
        );
    }

    /// Hover-driven: entering a target sends the ball there, with no click involved.
    #[test]
    fn hovering_a_target_sends_the_ball_there() {
        const SIZE: usize = 300;
        const UP: (f32, f32) = (150.0, 55.0);
        const RIGHT: (f32, f32) = (245.0, 150.0);
        const EMPTY: (f32, f32) = (10.0, 10.0);

        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];
        assert!(player
            .set_sw_target(&mut buffer, SIZE as u32, SIZE as u32, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/statemachines/normal_usecases/tween_poses.lottie"
            ))
            .is_ok());

        let sm_id = CString::new("tween_poses").unwrap();
        let mut sm = player
            .state_machine_load(&sm_id)
            .expect("embedded state machine should load");
        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");
        assert_eq!(sm.get_current_state_name(), "pose_center");

        sm.post_event(&Event::PointerMove {
            x: EMPTY.0,
            y: EMPTY.1,
        });
        assert_eq!(sm.get_numeric_input("pose"), Some(0.0));
        assert_eq!(sm.status, StateMachineEngineStatus::Running);

        sm.post_event(&Event::PointerMove { x: UP.0, y: UP.1 });
        assert_eq!(
            sm.get_numeric_input("pose"),
            Some(1.0),
            "entering the top target selects pose_up"
        );
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        let _ = sm.tick(500.0);
        let mid = blob_centroid(&buffer, SIZE);
        assert!(mid.1 < 130.0, "ball should be on its way up, y={}", mid.1);

        sm.post_event(&Event::PointerMove {
            x: RIGHT.0,
            y: RIGHT.1,
        });
        assert_eq!(
            sm.get_numeric_input("pose"),
            Some(2.0),
            "hovering another target during a tween must retarget it"
        );
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        let _ = sm.tick(500.0);
        let redirected = blob_centroid(&buffer, SIZE);
        assert!(
            redirected.0 > mid.0 + 20.0,
            "ball should now be heading right: x {} -> {}",
            mid.0,
            redirected.0
        );

        for _ in 0..120 {
            let _ = sm.tick(1000.0 / 60.0);
        }
        assert_eq!(sm.get_current_state_name(), "pose_right");
        assert_eq!(sm.status, StateMachineEngineStatus::Running);

        let entered = drain_entered_states(&mut sm);
        assert!(
            entered.contains(&"pose_right".to_string()),
            "got {entered:?}"
        );
        assert!(!entered.contains(&"pose_up".to_string()), "got {entered:?}");
    }

    /// Targets sit 95px from the centre of a 300x300 canvas. A press inside a target
    /// counts as entering it, so this holds on touch too.
    #[test]
    fn clicking_targets_redirects_the_ball_mid_tween() {
        const SIZE: usize = 300;
        const UP: (f32, f32) = (150.0, 55.0);
        const LEFT: (f32, f32) = (55.0, 150.0);

        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; SIZE * SIZE];
        assert!(player
            .set_sw_target(&mut buffer, SIZE as u32, SIZE as u32, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/statemachines/normal_usecases/tween_poses.lottie"
            ))
            .is_ok());

        let sm_id = CString::new("tween_poses").unwrap();
        let mut sm = player
            .state_machine_load(&sm_id)
            .expect("embedded state machine should load");
        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");
        assert_eq!(sm.get_current_state_name(), "pose_center");

        sm.post_event(&Event::PointerDown { x: UP.0, y: UP.1 });
        assert_eq!(
            sm.get_numeric_input("pose"),
            Some(1.0),
            "clicking the top target selects pose_up"
        );
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        let _ = sm.tick(500.0);
        let mid = blob_centroid(&buffer, SIZE);
        assert!(mid.1 < 130.0, "ball should be on its way up, y={}", mid.1);

        sm.post_event(&Event::PointerDown {
            x: LEFT.0,
            y: LEFT.1,
        });
        assert_eq!(
            sm.get_numeric_input("pose"),
            Some(4.0),
            "clicks must register during a tween"
        );
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        let _ = sm.tick(500.0);
        let redirected = blob_centroid(&buffer, SIZE);
        assert!(
            redirected.0 < mid.0 - 20.0,
            "ball should now be heading left: x {} -> {}",
            mid.0,
            redirected.0
        );

        for _ in 0..120 {
            let _ = sm.tick(1000.0 / 60.0);
        }
        assert_eq!(sm.get_current_state_name(), "pose_left");
        assert_eq!(sm.status, StateMachineEngineStatus::Running);

        let entered = drain_entered_states(&mut sm);
        assert!(
            entered.contains(&"pose_left".to_string()),
            "got {entered:?}"
        );
        assert!(!entered.contains(&"pose_up".to_string()), "got {entered:?}");
    }

    /// A tween target is not current yet, so its own transitions go unevaluated. They
    /// must be evaluated the moment it lands, or a guard satisfied during the tween is
    /// stranded until the next unrelated input.
    #[test]
    fn target_state_transitions_are_evaluated_once_the_tween_lands() {
        let sm_json = r#"{
            "initial": "star_1",
            "states": [
                {
                    "name": "global",
                    "type": "GlobalState",
                    "animation": "",
                    "transitions": [
                        {
                            "type": "Tweened",
                            "toState": "star_2",
                            "duration": 2.0,
                            "easing": [0.42, 0.0, 0.58, 1.0],
                            "guards": [
                                { "type": "Numeric", "conditionType": "Equal",
                                  "inputName": "rating", "compareTo": 2 }
                            ]
                        }
                    ]
                },
                { "type": "PlaybackState", "name": "star_1", "animation": "",
                  "autoplay": false, "loop": false, "segment": "angry",
                  "transitions": [] },
                { "type": "PlaybackState", "name": "star_2", "animation": "",
                  "autoplay": false, "loop": false, "segment": "sad",
                  "transitions": [
                      {
                          "type": "Transition",
                          "toState": "star_3",
                          "guards": [
                              { "type": "Boolean", "conditionType": "Equal",
                                "inputName": "ready", "compareTo": true }
                          ]
                      }
                  ] },
                { "type": "PlaybackState", "name": "star_3", "animation": "",
                  "autoplay": false, "loop": false, "segment": "mourn",
                  "transitions": [] }
            ],
            "interactions": [],
            "inputs": [
                { "type": "Numeric", "name": "rating", "value": 0 },
                { "type": "Boolean", "name": "ready", "value": false }
            ]
        }"#;

        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/smiley-slider.lottie"
            ))
            .is_ok());

        let mut sm = player
            .state_machine_load_data(sm_json)
            .expect("state machine to load successfully");
        sm.start(&OpenUrlPolicy::default())
            .expect("state machine should start");
        assert_eq!(sm.get_current_state_name(), "star_1");

        sm.set_numeric_input("rating", 2.0, true, false)
            .expect("input should be set");
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        let _ = sm.tick(500.0);
        sm.set_boolean_input("ready", true, true, false)
            .expect("input must be accepted during a tween");
        assert_eq!(
            sm.get_current_state_name(),
            "star_1",
            "star_2's transition cannot fire before star_2 is entered"
        );

        let _ = sm.tick(2500.0);
        assert_eq!(
            sm.get_current_state_name(),
            "star_3",
            "the guard set during the tween must not be dropped on arrival"
        );
    }

    #[test]
    fn retriggering_same_target_does_not_restart_tween() {
        let mut player = Player::new();
        let mut sm = setup_tween_redirection_sm(&mut player);

        sm.set_numeric_input("rating", 2.0, true, false)
            .expect("input should be set");
        let _ = sm.tick(500.0);

        // Same guard value again: the in-flight tween toward star_2 must not reset
        sm.set_numeric_input("rating", 2.0, true, false);
        assert_eq!(sm.status, StateMachineEngineStatus::Tweening);

        // 500ms already elapsed; 1600ms more crosses the original 2000ms deadline.
        // If the retrigger had reset elapsed time, the tween would still be running.
        let _ = sm.tick(1600.0);
        assert_eq!(
            sm.get_current_state_name(),
            "star_2",
            "tween must complete on the original schedule"
        );
        assert_eq!(sm.status, StateMachineEngineStatus::Running);
    }
}
