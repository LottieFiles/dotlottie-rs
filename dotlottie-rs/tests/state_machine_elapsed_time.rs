#![cfg(feature = "state-machines")]
#[cfg(test)]
mod tests {
    use dotlottie_rs::{
        actions::open_url_policy::OpenUrlPolicy, ColorSpace, Player, StateMachineEvent,
    };

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
    fn compare_to_dollar_elapsed_time_works() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player.load_dotlottie_data(STAR_RATING_LOTTIE).is_ok());

        let json =
            include_str!("../assets/statemachines/elapsed_time_tests/compare_to_elapsed.json");
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default()).unwrap();
        assert_eq!(sm.get_current_state_name(), "waiting");

        // Below threshold (threshold=0.5, elapsed=0.3): no transition.
        let _ = sm.tick(300.0);
        assert_eq!(sm.get_current_state_name(), "waiting");

        // Above threshold: threshold (0.5) < elapsedTime → fire.
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

    #[test]
    fn no_numeric_input_change_events_for_elapsed_time() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player.load_dotlottie_data(STAR_RATING_LOTTIE).is_ok());

        let json = include_str!("../assets/statemachines/elapsed_time_tests/reset_action.json");
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default()).unwrap();

        // Tick to accumulate, then trigger Reset via the "go" event.
        for _ in 0..30 {
            let _ = sm.tick(50.0);
        }
        sm.fire("go", true).unwrap();

        // Drain events: no NumericInputChange should mention elapsedTime.
        while let Some(evt) = sm.poll_event() {
            if let StateMachineEvent::NumericInputChange { name, .. } = evt {
                assert_ne!(
                    name.as_str(),
                    "elapsedTime",
                    "elapsedTime must never emit NumericInputChange"
                );
            }
        }
    }

    // ---------- Integration ----------

    #[test]
    fn ping_pong_via_reset() {
        let mut player = Player::new();
        let mut buffer: Vec<u32> = vec![0; (100 * 100) as usize];
        assert!(player
            .set_sw_target(&mut buffer, 100, 100, ColorSpace::ABGR8888)
            .is_ok());
        assert!(player.load_dotlottie_data(STAR_RATING_LOTTIE).is_ok());

        let json = include_str!("../assets/statemachines/elapsed_time_tests/ping_pong.json");
        let mut sm = player
            .state_machine_load_data(json)
            .expect("state machine to load successfully");

        sm.start(&OpenUrlPolicy::default()).unwrap();
        assert_eq!(sm.get_current_state_name(), "a");

        // Each tick of 600ms should cross the 0.5s threshold and trigger one
        // transition. Reset-on-entry zeroes elapsedTime, ready for next cycle.
        let _ = sm.tick(600.0);
        assert_eq!(sm.get_current_state_name(), "b");

        let _ = sm.tick(600.0);
        assert_eq!(sm.get_current_state_name(), "a");

        let _ = sm.tick(600.0);
        assert_eq!(sm.get_current_state_name(), "b");

        let _ = sm.tick(600.0);
        assert_eq!(sm.get_current_state_name(), "a");
    }
}
