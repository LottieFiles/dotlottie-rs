#![cfg(feature = "state-machines")]
#[cfg(test)]
mod tests {
    use dotlottie_rs::{
        actions::open_url_policy::OpenUrlPolicy, ColorSpace, Player, StateMachineEvent,
    };

    #[test]
    fn increment() {
        let global_state = include_str!("../assets/statemachines/action_tests/inc_rating.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];

        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok());
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        // Tests default increment without a value
        sm.set_numeric_input("rating", 1.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");

        // Tests adding with value
        sm.set_numeric_input("rating", 3.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_5");

        // Tests add from a input
        sm.set_numeric_input("rating", 6.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_12");

        // Tests add from a inexistant input, increments by 1.0 instead
        sm.set_numeric_input("rating", 13.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_14");
    }

    #[test]
    fn decrement() {
        let global_state = include_str!("../assets/statemachines/action_tests/decr_rating.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];

        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok());
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        // Tests decrement from an inexistant input, decrements by 1.0 instead
        sm.set_numeric_input("rating", 13.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_12");

        // Tests decrement from a input
        sm.set_numeric_input("rating", 6.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_6");

        // Tests decrementing with value
        sm.set_numeric_input("rating", 3.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_1");

        // Tests default increment without a value
        sm.set_numeric_input("rating", 5.0, true, false)
            .expect("input to set successfully");
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_4");
    }

    #[test]
    fn toggle() {
        let global_state = include_str!("../assets/statemachines/action_tests/toggle.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];

        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok());
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

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
        let global_state = include_str!("../assets/statemachines/action_tests/set_inputs.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];

        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888,)
            .is_ok());

        assert_eq!(
            player.load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            )),
            Ok(())
        );
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

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
        let global_state = include_str!("../assets/statemachines/action_tests/set_inputs.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];

        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok(),);
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        sm.set_numeric_input("NumericInput", 10.0, true, false)
            .expect("input to set successfully");

        // C state should of set the switch to true, landing us in state a
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "d");
    }

    #[test]
    fn set_string() {
        let global_state = include_str!("../assets/statemachines/action_tests/set_inputs.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];

        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok(),);
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        sm.set_string_input("StringInput", "second", true, false)
            .expect("input to set successfully");

        // C state should of set the switch to true, landing us in state a
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "f");
    }

    #[test]
    fn fire() {
        let global_state = include_str!("../assets/statemachines/action_tests/fire.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];

        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok(),);
        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        sm.set_boolean_input("OnOffSwitch", true, true, false)
            .expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "c");
    }

    #[test]
    fn set_frame() {
        let global_state = include_str!("../assets/statemachines/action_tests/set_frame.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];

        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok(),);

        assert_eq!(player.current_frame(), 0.0);

        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        // B Should of set the frame to input value "frame_holder", meaning 35
        assert_eq!(sm.player.current_frame(), 35.0);

        sm.set_boolean_input("OnOffSwitch", true, true, false)
            .expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "a");

        // A Should of set the frame to 10
        assert_eq!(player.current_frame(), 10.0);
    }

    #[test]
    fn set_progress() {
        let global_state = include_str!("../assets/statemachines/action_tests/set_progress.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];

        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok(),);

        assert_eq!(player.current_frame(), 0.0);

        let mut sm = player
            .state_machine_load_data(global_state)
            .expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "b");

        // B Should of set the frame to input value "frame_holder", 75% of the animation
        assert_eq!(sm.player.current_frame(), 66.75);

        sm.set_boolean_input("OnOffSwitch", true, true, false)
            .expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "a");

        // A Should of set the progress to 10%
        assert_eq!(player.current_frame(), 8.900001);
    }

    #[test]
    fn reset() {
        let reset_sm = include_str!("../assets/statemachines/action_tests/reset.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];

        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok(),);

        assert_eq!(player.current_frame(), 0.0);

        let mut sm = player
            .state_machine_load_data(reset_sm)
            .expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        sm.set_numeric_input("rating", 3.0, true, false)
            .expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");

        sm.set_numeric_input("rating", 6.0, true, false)
            .expect("input to set successfully");

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_0");
    }

    #[test]
    fn fire_custom_event() {
        let reset_sm = include_str!("../assets/statemachines/normal_usecases/rating.json");
        let mut player = Player::new();

        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];

        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok(),);

        assert_eq!(player.current_frame(), 0.0);

        let mut sm = player
            .state_machine_load_data(reset_sm)
            .expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert_eq!(s, Ok(()));

        sm.set_numeric_input("rating", 3.0, true, false)
            .expect("input to set successfully");

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
                StateMachineEvent::Transition {
                    previous_state,
                    new_state,
                } => Some(format!("on_transition: {previous_state} -> {new_state}")),
                StateMachineEvent::StateEntered { state } => {
                    Some(format!("on_state_entered: {state}"))
                }
                StateMachineEvent::StateExit { state } => Some(format!("on_state_exit: {state}")),
                StateMachineEvent::CustomEvent { message } => {
                    Some(format!("custom_event: {message}"))
                }
                _ => None,
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

    #[allow(clippy::ptr_arg)] // set_sw_target requires &mut Vec<u32>
    fn load_action_sm<'a>(
        player: &'a mut Player,
        buffer: &'a mut Vec<u32>,
        json: &str,
    ) -> dotlottie_rs::StateMachineEngine<'a> {
        assert!(player
            .set_sw_target(buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player
            .load_dotlottie_data(include_bytes!(
                "../assets/animations/dotlottie/v1/star_rating.lottie"
            ))
            .is_ok());
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));
        assert_eq!(sm.get_current_state_name(), "global");
        sm
    }

    #[test]
    fn multiply() {
        let json = include_str!("../assets/statemachines/action_tests/multiply.json");
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        let mut sm = load_action_sm(&mut player, &mut buffer, json);

        // literal operand: 2 * 3 = 6
        sm.set_numeric_input("x", 2.0, true, false);
        sm.set_numeric_input("trigger", 1.0, true, false);
        assert_eq!(sm.get_numeric_input("x"), Some(6.0));

        // input reference: 4 * factor(5) = 20
        sm.set_numeric_input("x", 4.0, true, false);
        sm.set_numeric_input("trigger", 2.0, true, false);
        assert_eq!(sm.get_numeric_input("x"), Some(20.0));

        // unresolvable reference: no-op, stays 7
        sm.set_numeric_input("x", 7.0, true, false);
        sm.set_numeric_input("trigger", 3.0, true, false);
        assert_eq!(sm.get_numeric_input("x"), Some(7.0));
    }

    #[test]
    fn floor() {
        let json = include_str!("../assets/statemachines/action_tests/floor.json");
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        let mut sm = load_action_sm(&mut player, &mut buffer, json);

        // 5.7 -> 5
        sm.set_numeric_input("x", 5.7, true, false);
        sm.set_numeric_input("trigger", 1.0, true, false);
        assert_eq!(sm.get_numeric_input("x"), Some(5.0));

        // -1.2 -> -2 (floor rounds toward negative infinity)
        sm.set_numeric_input("x", -1.2, true, false);
        sm.set_numeric_input("trigger", 2.0, true, false);
        assert_eq!(sm.get_numeric_input("x"), Some(-2.0));
    }

    #[test]
    fn clamp() {
        let json = include_str!("../assets/statemachines/action_tests/clamp.json");
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        let mut sm = load_action_sm(&mut player, &mut buffer, json);

        let cases = [
            (150.0, 1.0, 100.0), // [0,100] caps high
            (-5.0, 2.0, 0.0),    // [0,100] caps low
            (50.0, 3.0, 50.0),   // [0,100] in range, unchanged
            (-5.0, 4.0, 0.0),    // min-only floors at 0
            (150.0, 5.0, 100.0), // max-only caps at 100
            (25.0, 6.0, 20.0),   // ref bounds [lo=10, hi=20]
            (50.0, 7.0, 50.0),   // inverted bounds (min>max) -> no-op
            (50.0, 8.0, 50.0),   // both bounds absent -> no-op
        ];
        for (input, trigger, expected) in cases {
            sm.set_numeric_input("x", input, true, false);
            sm.set_numeric_input("trigger", trigger, true, false);
            assert_eq!(
                sm.get_numeric_input("x"),
                Some(expected),
                "clamp case trigger={trigger} input={input}"
            );
        }
    }

    #[test]
    fn set_random() {
        let json = include_str!("../assets/statemachines/action_tests/set_random.json");
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        let mut sm = load_action_sm(&mut player, &mut buffer, json);

        sm.set_seed(99);

        // Raw draw lands in [0, 1).
        sm.set_numeric_input("trigger", 1.0, true, false);
        let raw = sm.get_numeric_input("x").expect("x exists");
        assert!(
            (0.0..1.0).contains(&raw),
            "raw random out of [0,1): {raw}"
        );

        // Dice via SetRandom -> Multiply(6) -> Floor -> Increment(4) lands in {4..9}.
        sm.set_numeric_input("trigger", 2.0, true, false);
        let dice = sm.get_numeric_input("dice").expect("dice exists");
        assert!((4.0..=9.0).contains(&dice), "dice out of range: {dice}");
        assert_eq!(dice.fract(), 0.0, "dice not an integer: {dice}");

        // Determinism: re-seeding to the same value reproduces the first draw.
        sm.set_seed(99);
        sm.set_numeric_input("trigger", 3.0, true, false);
        assert_eq!(
            sm.get_numeric_input("x2"),
            sm.get_numeric_input("x"),
            "same seed should reproduce the first draw"
        );
    }

    // Drives the random_timer machine (the `set_random` example's SM) by
    // ticking, seeded with `seed`, until it has either completed a full
    // dice -> countdown -> timer -> dice loop or exhausted the tick budget.
    // Returns (ordered states visited, timerWait rolled in the dice state).
    fn run_random_timer(seed: u64) -> (Vec<String>, Option<f32>) {
        let sm_json = include_str!("../assets/statemachines/random_timer.json");
        let anim = include_str!("../assets/animations/lottie/random_timer.json");
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        let c_anim = std::ffi::CString::new(anim).expect("CString");
        assert!(player.load_animation_data(&c_anim).is_ok());
        let mut sm = player
            .state_machine_load_data(sm_json)
            .expect("state machine to load successfully");
        sm.set_seed(seed);
        assert_eq!(sm.start(&OpenUrlPolicy::default()), Ok(()));

        let mut visited = vec![sm.get_current_state_name()];
        let mut timer_wait = None;
        // 16ms steps; dice plays ~1.4s, countdown waits 1-3s, timer 1-10s.
        for _ in 0..4000 {
            let _ = sm.tick(16.0);
            let s = sm.get_current_state_name();
            if visited.last() != Some(&s) {
                if s == "timer" && timer_wait.is_none() {
                    timer_wait = sm.get_numeric_input("timerWait");
                }
                visited.push(s.clone());
            }
            // Stop once we've looped back to dice after a full cycle.
            if visited.iter().any(|v| v == "timer") && s == "dice" && visited.len() > 1 {
                break;
            }
        }
        (visited, timer_wait)
    }

    #[test]
    fn random_timer_loop() {
        let (visited, timer_wait) = run_random_timer(7);

        // Starts on the dice throw, then progresses through the full loop.
        assert_eq!(visited.first().map(String::as_str), Some("dice"));
        assert!(visited.iter().any(|s| s == "countdown"), "never reached countdown: {visited:?}");
        assert!(visited.iter().any(|s| s == "timer"), "never reached timer: {visited:?}");
        assert!(
            visited.iter().skip(1).any(|s| s == "dice"),
            "did not loop back to dice: {visited:?}"
        );

        // The dice rolled an integer wait in [1, 10] for the timer state.
        let wait = timer_wait.expect("timerWait should be set during the dice state");
        assert!((1.0..=10.0).contains(&wait), "timerWait out of range: {wait}");
        assert_eq!(wait.fract(), 0.0, "timerWait should be an integer: {wait}");

        // Deterministic given the seed.
        let (visited_again, wait_again) = run_random_timer(7);
        assert_eq!(visited, visited_again, "same seed should reproduce the loop");
        assert_eq!(timer_wait, wait_again);
    }
}
