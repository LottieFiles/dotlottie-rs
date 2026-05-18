#![cfg(feature = "state-machines")]
#[cfg(test)]
mod tests {
    use dotlottie_rs::{actions::open_url_policy::OpenUrlPolicy, ColorSpace, Player};

    const STAR_RATING_LOTTIE: &[u8] =
        include_bytes!("../assets/animations/dotlottie/v1/star_rating.lottie");

    // ---------- Behavior ----------

    #[test]
    fn tick_accumulates_in_seconds() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player.load_dotlottie_data(STAR_RATING_LOTTIE).is_ok());

        let json = include_str!("../assets/statemachines/elapsed_time_tests/no_elapsed_time.json");
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default()).unwrap();

        assert_eq!(sm.get_numeric_input("elapsedTime"), Some(0.0));

        let _ = sm.tick(500.0);
        let _ = sm.tick(500.0);

        let elapsed = sm.get_numeric_input("elapsedTime").unwrap();
        assert!(
            (elapsed - 1.0).abs() < 1e-4,
            "expected ~1.0s, got {elapsed}"
        );
    }

    #[test]
    fn no_increment_when_stopped() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player.load_dotlottie_data(STAR_RATING_LOTTIE).is_ok());

        let json = include_str!("../assets/statemachines/elapsed_time_tests/no_elapsed_time.json");
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");

        // Don't start — engine status is Stopped.
        let _ = sm.tick(1000.0);
        let _ = sm.tick(1000.0);

        assert_eq!(sm.get_numeric_input("elapsedTime"), Some(0.0));
    }

    #[test]
    fn start_re_zeroes_elapsed_time() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player.load_dotlottie_data(STAR_RATING_LOTTIE).is_ok());

        let json = include_str!("../assets/statemachines/elapsed_time_tests/no_elapsed_time.json");
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default()).unwrap();
        let _ = sm.tick(750.0);
        assert!(sm.get_numeric_input("elapsedTime").unwrap() > 0.7);

        sm.stop();
        sm.start(&OpenUrlPolicy::default()).unwrap();
        assert_eq!(sm.get_numeric_input("elapsedTime"), Some(0.0));
    }

    // ---------- Guards ----------

    #[test]
    fn input_name_elapsed_time_guard_fires() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player.load_dotlottie_data(STAR_RATING_LOTTIE).is_ok());

        let json =
            include_str!("../assets/statemachines/elapsed_time_tests/timeout_transition.json");
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default()).unwrap();
        assert_eq!(sm.get_current_state_name(), "running");

        // Tick under the threshold — no transition.
        let _ = sm.tick(500.0);
        assert_eq!(sm.get_current_state_name(), "running");

        // Cross the > 1.0s threshold — pipeline should re-evaluate on tick.
        let _ = sm.tick(700.0);
        assert_eq!(sm.get_current_state_name(), "done");
    }

    #[test]
    fn at_elapsed_time_resolves_in_action_value() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player.load_dotlottie_data(STAR_RATING_LOTTIE).is_ok());

        let json = r#"{
          "initial": "s",
          "states": [
            { "type": "PlaybackState", "name": "s", "animation": "", "transitions": [],
              "entryActions": [
                { "type": "Increment", "inputName": "stamp", "value": "@elapsedTime" }
              ]
            }
          ],
          "inputs": [{ "type": "Numeric", "name": "stamp", "value": 10 }]
        }"#;
        let mut sm = player.state_machine_load_data(json).expect("load");

        let _ = sm.tick(750.0);
        sm.start(&OpenUrlPolicy::default()).unwrap();

        let stamp = sm.get_numeric_input("stamp").unwrap();
        assert!(
            (stamp - 10.0).abs() < 1e-4,
            "Increment by @elapsedTime at start (elapsedTime=0): expected ~10.0, got {stamp}"
        );

        let _ = sm.tick(500.0);
        sm.set_numeric_input("stamp", 10.0, false, false);
        sm.fire("evt", true).ok();
        sm.override_current_state("s", false).unwrap();

        let stamp = sm.get_numeric_input("stamp").unwrap();
        assert!(
            (stamp - 10.5).abs() < 1e-3,
            "Increment by @elapsedTime at elapsedTime=0.5: expected ~10.5, got {stamp}"
        );
    }

    #[test]
    fn compare_to_at_elapsed_time_works() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player.load_dotlottie_data(STAR_RATING_LOTTIE).is_ok());

        let json = include_str!(
            "../assets/statemachines/elapsed_time_tests/compare_to_at_elapsed_time.json"
        );
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default()).unwrap();
        assert_eq!(sm.get_current_state_name(), "waiting");

        let _ = sm.tick(300.0);
        assert_eq!(sm.get_current_state_name(), "waiting");

        let _ = sm.tick(400.0);
        assert_eq!(sm.get_current_state_name(), "after");
    }

    // ---------- Pipeline gating ----------

    #[test]
    fn no_pipeline_re_eval_when_no_elapsed_time_guard() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player.load_dotlottie_data(STAR_RATING_LOTTIE).is_ok());

        // This SM has a single state with no transitions and an entry action
        // that increments `counter`. Entry runs on start (counter=1). If tick
        // were re-evaluating the pipeline, set_current_state would re-fire the
        // entry action. It must not.
        let json = include_str!("../assets/statemachines/elapsed_time_tests/no_elapsed_time.json");
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default()).unwrap();
        assert_eq!(sm.get_numeric_input("counter"), Some(1.0));

        for _ in 0..50 {
            let _ = sm.tick(16.0);
        }

        assert_eq!(
            sm.get_numeric_input("counter"),
            Some(1.0),
            "tick must not re-run entry actions for a state with no elapsedTime guard"
        );
    }

    // ---------- Observability ----------

    #[test]
    fn get_inputs_includes_elapsed_time_even_with_no_declared_inputs() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player.load_dotlottie_data(STAR_RATING_LOTTIE).is_ok());

        // timeout_transition.json has empty inputs — only elapsedTime exists.
        let json =
            include_str!("../assets/statemachines/elapsed_time_tests/timeout_transition.json");
        let sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");

        let inputs = sm.get_inputs();
        // get_inputs returns flat (name, type) pairs.
        let mut found = false;
        let mut iter = inputs.iter();
        while let (Some(name), Some(type_)) = (iter.next(), iter.next()) {
            if name == "elapsedTime" && type_ == "Numeric" {
                found = true;
            }
        }
        assert!(found, "elapsedTime should be listed in get_inputs()");
    }
}
