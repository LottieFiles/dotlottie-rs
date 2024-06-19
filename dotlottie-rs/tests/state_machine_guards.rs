#[cfg(test)]
mod tests {
    use dotlottie_player_core::states::StateTrait;
    use dotlottie_player_core::{
        parser::TransitionGuardConditionType,
        transitions::{guard::Guard, TransitionTrait},
    };

    use dotlottie_player_core::DotLottiePlayer;

    #[test]
    pub fn guards_loaded_correctly() {
        use dotlottie_player_core::transitions::TransitionTrait;

        use dotlottie_player_core::{states::State, Config, DotLottiePlayer};

        // let file_path = format!(
        //     "{}{}",
        //     env!("CARGO_MANIFEST_DIR"),
        //     "/tests/fixtures/pigeon_fsm_gt_gte_guard.lottie"
        // );
        // let mut loaded_file = File::open(file_path.clone()).expect("no file found");
        // let meta_data = fs::metadata(file_path.clone()).expect("unable to read metadata");

        // let mut buffer = vec![0; meta_data.len() as usize];
        // loaded_file.read(&mut buffer).expect("buffer overflow");

        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(
            include_bytes!("fixtures/pigeon_fsm_gt_gte_guard.lottie"),
            100,
            100,
        );

        // player.load_dotlottie_data(&buffer, 100, 100);

        // let mut sm_definition = File::open(file_path).unwrap();
        // let mut buffer_to_string = String::new();

        // sm_definition.read_to_string(&mut buffer_to_string).unwrap();

        player.load_state_machine("gt_gte_guard");

        player.start_state_machine();

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        let tmp_unwrap = sm.read().unwrap();
        let unwrapped_sm = tmp_unwrap.as_ref().unwrap();

        let guard_0 = Guard {
            context_key: "counter_0".to_string(),
            condition_type: TransitionGuardConditionType::GreaterThan,
            compare_to: dotlottie_player_core::parser::StringNumberBool::F32(5.0),
        };
        let guard_1 = Guard {
            context_key: "counter_0".to_string(),
            condition_type: TransitionGuardConditionType::GreaterThanOrEqual,
            compare_to: dotlottie_player_core::parser::StringNumberBool::F32(60.0),
        };
        let guard_2 = Guard {
            context_key: "counter_0".to_string(),
            condition_type: TransitionGuardConditionType::GreaterThanOrEqual,
            compare_to: dotlottie_player_core::parser::StringNumberBool::F32(65.0),
        };

        let guards = [guard_0, guard_1, guard_2];

        for (i, state) in unwrapped_sm.states.iter().enumerate() {
            let unwrapped_state = &*state.read().unwrap();

            // match unwrapped_state {
            if let State::Playback {
                name: _,
                config: _,
                reset_context: _,
                animation_id: _,
                transitions,
            } = unwrapped_state
            {
                let first_transition = &*transitions[0].read().unwrap();

                assert_eq!(
                    first_transition.get_guards()[0].compare_to,
                    guards[i].compare_to
                );

                assert_eq!(
                    first_transition.get_guards()[0].condition_type,
                    guards[i].condition_type
                );

                assert_eq!(
                    first_transition.get_guards()[0].context_key,
                    guards[i].context_key
                );
            }
        }
    }

    // Helper function to get the current transition event as a string
    pub fn get_current_transition_event(player: &DotLottiePlayer) -> String {
        let players_first_state = player
            .get_state_machine()
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .get_current_state()
            .unwrap();

        let players_first_transition =
            &*players_first_state.read().unwrap().get_transitions()[0].clone();

        let complete_event = &*players_first_transition.read().unwrap();

        let complete_event_string = complete_event.get_event().read().unwrap().as_str().clone();

        complete_event_string.clone()
    }

    #[test]
    pub fn not_equal_test() {
        use dotlottie_player_core::transitions::TransitionTrait;

        use dotlottie_player_core::{events::Event, Config, DotLottiePlayer};

        // let file_path = format!(
        //     "{}{}",
        //     env!("CARGO_MANIFEST_DIR"),
        //     "/tests/fixtures/pigeon_fsm_ne_guard.lottie"
        // );
        // let mut loaded_file = File::open(file_path.clone()).expect("no file found");
        // let meta_data = fs::metadata(file_path.clone()).expect("unable to read metadata");

        // let mut buffer = vec![0; meta_data.len() as usize];
        // loaded_file.read(&mut buffer).expect("buffer overflow");

        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(
            include_bytes!("fixtures/pigeon_fsm_ne_guard.lottie"),
            100,
            100,
        );

        // player.load_dotlottie_data(&buffer, 100, 100);

        player.load_state_machine("ne_guard");

        player.start_state_machine();

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        let players_first_state = sm
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .get_current_state()
            .unwrap();

        let players_first_transition =
            &*players_first_state.read().unwrap().get_transitions()[0].clone();

        let first_player_transition_unwrapped = players_first_transition.read().unwrap();

        assert_eq!(
            &*first_player_transition_unwrapped
                .get_event()
                .read()
                .unwrap()
                .as_str(),
            "explosion".to_string()
        );

        // Test != with a numeric value
        player.tmp_set_state_machine_context("counter_0", 5.0);
        player.post_event(&Event::String {
            value: "explosion".to_string(),
        });

        // Should stay in stage 0 since 5 = 5
        assert_eq!(get_current_transition_event(&player), "explosion");

        player.tmp_set_state_machine_context("counter_0", 18.0);
        player.post_event(&Event::String {
            value: "explosion".to_string(),
        });

        // Should go to stage 2
        assert_eq!(get_current_transition_event(&player), "complete");

        // Test != with a string value
        player.tmp_set_state_machine_string_context(
            "counter_1",
            "to_be_the_same".to_string().as_str(),
        );
        player.post_event(&Event::String {
            value: "complete".to_string(),
        });

        // Should stay in stage 2 since diff_value = diff_value
        assert_eq!(get_current_transition_event(&player), "complete");

        player
            .tmp_set_state_machine_string_context("counter_1", "not_the_same".to_string().as_str());
        player.post_event(&Event::String {
            value: "complete".to_string(),
        });

        // Should go to stage 3
        assert_eq!(get_current_transition_event(&player), "done");

        // Test != with a bool value
        player.tmp_set_state_machine_bool_context("counter_2", true);
        player.post_event(&Event::String {
            value: "done".to_string(),
        });

        // Should stay in stage 3 since diff_value = diff_value
        assert_eq!(get_current_transition_event(&player), "done");

        player.tmp_set_state_machine_bool_context("counter_2", false);
        player.post_event(&Event::String {
            value: "done".to_string(),
        });

        // Should go to stage 0
        assert_eq!(get_current_transition_event(&player), "explosion");
    }

    #[test]
    pub fn equal_test() {
        use dotlottie_player_core::transitions::TransitionTrait;

        use dotlottie_player_core::{events::Event, Config, DotLottiePlayer};

        // let file_path = format!(
        //     "{}{}",
        //     env!("CARGO_MANIFEST_DIR"),
        //     "/tests/fixtures/pigeon_fsm_eq_guard.lottie"
        // );
        // let mut loaded_file = File::open(file_path.clone()).expect("no file found");
        // let meta_data = fs::metadata(file_path.clone()).expect("unable to read metadata");

        // let mut buffer = vec![0; meta_data.len() as usize];
        // loaded_file.read(&mut buffer).expect("buffer overflow");

        let player = DotLottiePlayer::new(Config::default());
        // player.load_dotlottie_data(&buffer, 100, 100);
        player.load_dotlottie_data(
            include_bytes!("fixtures/pigeon_fsm_eq_guard.lottie"),
            100,
            100,
        );

        player.load_state_machine("fsm_eq_guard");

        player.start_state_machine();

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        let players_first_state = sm
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .get_current_state()
            .unwrap();

        let players_first_transition =
            &*players_first_state.read().unwrap().get_transitions()[0].clone();

        let first_player_transition_unwrapped = players_first_transition.read().unwrap();

        assert_eq!(
            &*first_player_transition_unwrapped
                .get_event()
                .read()
                .unwrap()
                .as_str(),
            "explosion".to_string()
        );

        // Test with a numeric value
        player.tmp_set_state_machine_context("counter_0", 4.0);
        player.post_event(&Event::String {
            value: "explosion".to_string(),
        });

        // Should stay on stage 0 since 4 != 5
        assert_eq!(get_current_transition_event(&player), "explosion");

        player.tmp_set_state_machine_context("counter_0", 5.0);
        player.post_event(&Event::String {
            value: "explosion".to_string(),
        });

        // Should go to stage 1
        assert_eq!(get_current_transition_event(&player), "complete");

        // Test with a string value
        player.tmp_set_state_machine_string_context("counter_1", "diff");
        player.post_event(&Event::String {
            value: "complete".to_string(),
        });

        // Should stay on stage 1 since to_be_the_same != diff
        assert_eq!(get_current_transition_event(&player), "complete");

        player.tmp_set_state_machine_string_context("counter_1", "to_be_the_same");
        player.post_event(&Event::String {
            value: "complete".to_string(),
        });

        // Should go to stage 2
        assert_eq!(get_current_transition_event(&player), "done");

        // Test with a bool value
        player.tmp_set_state_machine_bool_context("counter_2", false);
        player.post_event(&Event::String {
            value: "done".to_string(),
        });

        // Should stay on stage 3 since false != true
        assert_eq!(get_current_transition_event(&player), "done");

        player.tmp_set_state_machine_bool_context("counter_2", true);
        player.post_event(&Event::String {
            value: "done".to_string(),
        });

        // Should go to stage 0
        assert_eq!(get_current_transition_event(&player), "explosion");
    }

    #[test]
    pub fn greater_than_greater_than_or_equal_test() {
        use dotlottie_player_core::transitions::TransitionTrait;

        use dotlottie_player_core::{events::Event, Config, DotLottiePlayer};

        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(
            include_bytes!("fixtures/pigeon_fsm_gt_gte_guard.lottie"),
            100,
            100,
        );

        player.load_state_machine("gt_gte_guard");

        player.start_state_machine();

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        let players_first_state = sm
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .get_current_state()
            .unwrap();

        let players_first_transition =
            &*players_first_state.read().unwrap().get_transitions()[0].clone();

        let first_player_transition_unwrapped = players_first_transition.read().unwrap();

        assert_eq!(
            &*first_player_transition_unwrapped
                .get_event()
                .read()
                .unwrap()
                .as_str(),
            "explosion".to_string()
        );

        drop(sm);

        player.tmp_set_state_machine_context("counter_0", 5.0);
        player.post_event(&Event::String {
            value: "explosion".to_string(),
        });

        // Not greater than 5.0, should stay on first stage
        assert_eq!(get_current_transition_event(&player), "explosion");

        // Greater than, should go to stage 2
        player.tmp_set_state_machine_context("counter_0", 6.0);

        player.post_event(&Event::String {
            value: "explosion".to_string(),
        });

        // Assert stage 2
        assert_eq!(get_current_transition_event(&player), "complete");

        // Greater than or equal, since its equal should go to stage 3
        player.tmp_set_state_machine_context("counter_0", 60.0);

        player.post_event(&Event::String {
            value: "complete".to_string(),
        });

        // Assert stage 3
        assert_eq!(get_current_transition_event(&player), "done");

        // Greater than or equal, since its not >= should stay on same stage
        player.tmp_set_state_machine_context("counter_0", 64.0);

        player.post_event(&Event::String {
            value: "done".to_string(),
        });

        // Assert stage 3
        assert_eq!(get_current_transition_event(&player), "done");

        // Greater than or equal, greater than should go to stage 0
        player.tmp_set_state_machine_context("counter_0", 66.0);

        player.post_event(&Event::String {
            value: "done".to_string(),
        });

        // Assert stage 0
        assert_eq!(get_current_transition_event(&player), "explosion");
    }

    #[test]
    pub fn less_than_less_than_equal_test() {
        use dotlottie_player_core::transitions::TransitionTrait;

        use dotlottie_player_core::{events::Event, Config, DotLottiePlayer};

        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(
            include_bytes!("fixtures/pigeon_fsm_lt_lte_guard.lottie"),
            100,
            100,
        );

        player.load_state_machine("lt_lte_guard");

        player.start_state_machine();

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        let players_first_state = sm
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .get_current_state()
            .unwrap();

        let players_first_transition =
            &*players_first_state.read().unwrap().get_transitions()[0].clone();

        let first_player_transition_unwrapped = players_first_transition.read().unwrap();

        assert_eq!(
            &*first_player_transition_unwrapped
                .get_event()
                .read()
                .unwrap()
                .as_str(),
            "explosion".to_string()
        );

        // Test less than
        player.tmp_set_state_machine_context("counter_0", 5.0);
        player.post_event(&Event::String {
            value: "explosion".to_string(),
        });

        // Should stay in first stage since 5 == 5
        assert_eq!(get_current_transition_event(&player), "explosion");

        // Test less than
        player.tmp_set_state_machine_context("counter_0", 1.0);

        let string_event = Event::String {
            value: "explosion".to_string(),
        };

        player.post_event(&string_event);

        // Should go to stage 2
        assert_eq!(get_current_transition_event(&player), "complete");

        // Start testing less than equal
        player.tmp_set_state_machine_context("counter_0", 11.0);
        player.post_event(&Event::String {
            value: "complete".to_string(),
        });

        // Should stay in second stage since 11 > 10
        assert_eq!(get_current_transition_event(&player), "complete");

        // Test equal
        player.tmp_set_state_machine_context("counter_0", 10.0);
        player.post_event(&Event::String {
            value: "complete".to_string(),
        });

        // Should go to third stage
        assert_eq!(get_current_transition_event(&player), "done");

        // Test less than
        player.tmp_set_state_machine_context("counter_0", 14.0);
        player.post_event(&Event::String {
            value: "done".to_string(),
        });

        // Should go back to first stage
        assert_eq!(get_current_transition_event(&player), "explosion");
    }
}
