mod test_utils;

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::Read,
        sync::{Arc, RwLock},
    };

    use dotlottie_player_core::states::StateTrait;
    use dotlottie_player_core::{
        parser::TransitionGuardConditionType,
        transitions::{guard::Guard, TransitionTrait},
    };

    use dotlottie_player_core::DotLottiePlayer;

    #[test]
    pub fn guards_loaded_correctly() {
        use dotlottie_player_core::transitions::{Transition::Transition, TransitionTrait};

        use dotlottie_player_core::{events::Event, states::State, Config, DotLottiePlayer};

        let player = DotLottiePlayer::new(Config::default());
        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/pigeon_fsm_gt_guard.json"
        );

        let mut sm_definition = File::open(file_path).unwrap();
        let mut buffer_to_string = String::new();

        sm_definition.read_to_string(&mut buffer_to_string).unwrap();

        player.load_state_machine(&buffer_to_string);

        player.start_state_machine();

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        let tmp_unwrap = sm.read().unwrap();
        let unwrapped_sm = tmp_unwrap.as_ref().unwrap();

        let guard_clone = Guard {
            context_key: "counter_0".to_string(),
            condition_type: TransitionGuardConditionType::GreaterThan,
            compare_to: dotlottie_player_core::parser::StringNumberBool::F32(5.0),
        };

        let pigeon_transition_0_clone = Transition {
            target_state: 1,
            event: Arc::new(RwLock::new(Event::String("explosion".to_string()))),
            guards: vec![guard_clone],
        };

        let mut i = 0;

        for state in unwrapped_sm.states.iter() {
            let unwrapped_state = &*state.read().unwrap();

            match unwrapped_state {
                State::Playback {
                    config: _,
                    reset_context: _,
                    animation_id: _,
                    width: _,
                    height: _,
                    transitions,
                } => {
                    if i == 0 {
                        let first_transition = &*transitions[0].read().unwrap();

                        assert_eq!(
                            first_transition.get_guards()[0].compare_to,
                            pigeon_transition_0_clone.get_guards()[0].compare_to
                        );

                        assert_eq!(
                            first_transition.get_guards()[0].condition_type,
                            pigeon_transition_0_clone.get_guards()[0].condition_type
                        );

                        assert_eq!(
                            first_transition.get_guards()[0].context_key,
                            pigeon_transition_0_clone.get_guards()[0].context_key
                        );
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

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

        return complete_event_string.clone();
    }

    #[test]
    pub fn diff_test() {
        use dotlottie_player_core::transitions::TransitionTrait;

        use dotlottie_player_core::{events::Event, Config, DotLottiePlayer};

        let player = DotLottiePlayer::new(Config::default());
        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/pigeon_fsm_ne_guard.json"
        );

        let mut sm_definition = File::open(file_path).unwrap();
        let mut buffer_to_string = String::new();

        sm_definition.read_to_string(&mut buffer_to_string).unwrap();

        player.load_state_machine(&buffer_to_string);

        player.start_state_machine();

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        // Test if the state machine transitions after sending 5 events to it

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
        player.post_event(&Event::String("explosion".to_string()));

        // Should stay in first state
        assert_eq!(get_current_transition_event(&player), "explosion");

        player.tmp_set_state_machine_context("counter_0", 18.0);
        player.post_event(&Event::String("explosion".to_string()));

        // Should stay in first state
        assert_eq!(get_current_transition_event(&player), "complete");
    }

    #[test]
    pub fn equal_test() {
        use dotlottie_player_core::transitions::TransitionTrait;

        use dotlottie_player_core::{events::Event, Config, DotLottiePlayer};

        let player = DotLottiePlayer::new(Config::default());
        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/pigeon_fsm_eq_guard.json"
        );

        let mut sm_definition = File::open(file_path).unwrap();
        let mut buffer_to_string = String::new();

        sm_definition.read_to_string(&mut buffer_to_string).unwrap();

        player.load_state_machine(&buffer_to_string);

        player.start_state_machine();

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        // Test if the state machine transitions after sending 5 events to it

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
        player.post_event(&Event::String("explosion".to_string()));

        assert_eq!(get_current_transition_event(&player), "complete");
    }

    #[test]
    pub fn greater_than_test() {
        use dotlottie_player_core::transitions::TransitionTrait;

        use dotlottie_player_core::{events::Event, Config, DotLottiePlayer};

        let player = DotLottiePlayer::new(Config::default());
        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/pigeon_fsm_gt_guard.json"
        );

        let mut sm_definition = File::open(file_path).unwrap();
        let mut buffer_to_string = String::new();

        sm_definition.read_to_string(&mut buffer_to_string).unwrap();

        player.load_state_machine(&buffer_to_string);

        player.start_state_machine();

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        // Test if the state machine transitions after sending 5 events to it

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
        player.post_event(&Event::String("explosion".to_string()));

        assert_eq!(get_current_transition_event(&player), "explosion");

        player.tmp_set_state_machine_context("counter_0", 6.0);

        let string_event = Event::String("explosion".to_string());

        player.post_event(&string_event);

        assert_eq!(get_current_transition_event(&player), "complete");
    }

    #[test]
    pub fn greater_than_or_equal_test() {
        use dotlottie_player_core::transitions::TransitionTrait;

        use dotlottie_player_core::{events::Event, Config, DotLottiePlayer};

        let player = DotLottiePlayer::new(Config::default());
        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/pigeon_fsm_gte_guard.json"
        );

        let mut sm_definition = File::open(file_path).unwrap();
        let mut buffer_to_string = String::new();

        sm_definition.read_to_string(&mut buffer_to_string).unwrap();

        player.load_state_machine(&buffer_to_string);

        player.start_state_machine();

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        // Test if the state machine transitions after sending 5 events to it

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
        player.post_event(&Event::String("explosion".to_string()));

        assert_eq!(get_current_transition_event(&player), "complete");

        player.end_state_machine();

        // Test greater than too -------------------- //
        player.load_state_machine(&buffer_to_string);

        player.start_state_machine();

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        // Test if the state machine transitions after sending 5 events to it

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

        player.tmp_set_state_machine_context("counter_0", 19.0);
        player.post_event(&Event::String("explosion".to_string()));

        assert_eq!(get_current_transition_event(&player), "complete");
    }

    #[test]
    pub fn less_than_test() {
        use dotlottie_player_core::transitions::TransitionTrait;

        use dotlottie_player_core::{events::Event, Config, DotLottiePlayer};

        let player = DotLottiePlayer::new(Config::default());
        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/pigeon_fsm_lt_guard.json"
        );

        let mut sm_definition = File::open(file_path).unwrap();
        let mut buffer_to_string = String::new();

        sm_definition.read_to_string(&mut buffer_to_string).unwrap();

        player.load_state_machine(&buffer_to_string);

        player.start_state_machine();

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        // Test if the state machine transitions after sending 5 events to it

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

        player.tmp_set_state_machine_context("counter_0", 5.0);
        player.post_event(&Event::String("explosion".to_string()));

        assert_eq!(get_current_transition_event(&player), "explosion");

        player.tmp_set_state_machine_context("counter_0", 1.0);

        let string_event = Event::String("explosion".to_string());

        player.post_event(&string_event);

        assert_eq!(get_current_transition_event(&player), "complete");
    }

    #[test]
    pub fn less_than_or_equal_test() {
        use dotlottie_player_core::transitions::TransitionTrait;

        use dotlottie_player_core::{events::Event, Config, DotLottiePlayer};

        let player = DotLottiePlayer::new(Config::default());
        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/pigeon_fsm_lte_guard.json"
        );

        let mut sm_definition = File::open(file_path).unwrap();
        let mut buffer_to_string = String::new();

        sm_definition.read_to_string(&mut buffer_to_string).unwrap();

        player.load_state_machine(&buffer_to_string);

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

        // test equal
        player.tmp_set_state_machine_context("counter_0", 5.0);
        player.post_event(&Event::String("explosion".to_string()));

        assert_eq!(get_current_transition_event(&player), "complete");

        player.end_state_machine();

        player.load_state_machine(&buffer_to_string);

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

        // test less than
        player.tmp_set_state_machine_context("counter_0", 1.0);
        player.post_event(&Event::String("explosion".to_string()));

        assert_eq!(get_current_transition_event(&player), "complete");
    }
}
