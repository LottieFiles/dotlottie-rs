mod test_utils;

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::Read,
        sync::{Arc, RwLock},
    };

    use dotlottie_player_core::{
        states::StateTrait,
        transitions::{Transition::Transition, TransitionTrait},
        StateMachineObserver,
    };

    use crate::test_utils::{HEIGHT, WIDTH};
    use dotlottie_player_core::{events::Event, states::State, Config, DotLottiePlayer, Mode};

    #[test]
    pub fn load_multiple_states() {
        let player = DotLottiePlayer::new(Config::default());
        let file_path = format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/pigeon_fsm.json"
        );

        let mut sm_definition = File::open(file_path).unwrap();
        let mut buffer_to_string = String::new();

        sm_definition.read_to_string(&mut buffer_to_string).unwrap();

        player.load_state_machine(&buffer_to_string);

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        let tmp_unwrap = sm.read().unwrap();
        let unwrapped_sm = tmp_unwrap.as_ref().unwrap();

        assert!(
            unwrapped_sm.states.len() == 3,
            "State machine states are not loaded"
        );

        let pigeon_transition_0 = Transition {
            target_state: 1,
            event: Arc::new(RwLock::new(Event::String {
                value: "explosion".to_string(),
            })),
            guards: Vec::new(),
        };

        let pigeon_transition_1 = Transition {
            target_state: 2,
            event: Arc::new(RwLock::new(Event::String {
                value: "complete".to_string(),
            })),
            guards: Vec::new(),
        };

        let pigeon_transition_2 = Transition {
            target_state: 0,
            event: Arc::new(RwLock::new(Event::String {
                value: "complete".to_string(),
            })),
            guards: Vec::new(),
        };

        let pigeon_state_0 = State::Playback {
            config: Config {
                mode: Mode::Forward,
                loop_animation: true,
                speed: 1.0,
                use_frame_interpolation: true,
                autoplay: true,
                segment: [].to_vec(),
                background_color: Config::default().background_color,
                layout: Config::default().layout,
                marker: "bird".to_string(),
            },
            reset_context: "".to_string(),
            animation_id: "".to_string(),
            width: WIDTH,
            height: HEIGHT,
            transitions: vec![Arc::new(RwLock::new(pigeon_transition_0))],
        };

        let pigeon_state_1 = State::Playback {
            config: Config {
                mode: Mode::Forward,
                loop_animation: false,
                speed: 0.5,
                use_frame_interpolation: true,
                autoplay: true,
                segment: [].to_vec(),
                background_color: Config::default().background_color,
                layout: Config::default().layout,
                marker: "explosion".to_string(),
            },
            reset_context: "".to_string(),
            animation_id: "".to_string(),
            width: WIDTH,
            height: HEIGHT,
            transitions: vec![Arc::new(RwLock::new(pigeon_transition_1))],
        };

        let pigeon_state_2 = State::Playback {
            config: Config {
                mode: Mode::Forward,
                loop_animation: false,
                speed: 1.0,
                use_frame_interpolation: true,
                autoplay: true,
                segment: [].to_vec(),
                background_color: Config::default().background_color,
                layout: Config::default().layout,
                marker: "feather".to_string(),
            },
            reset_context: "".to_string(),
            animation_id: "".to_string(),
            width: WIDTH,
            height: HEIGHT,
            transitions: vec![Arc::new(RwLock::new(pigeon_transition_2))],
        };

        let pigeon_states = vec![pigeon_state_0, pigeon_state_1, pigeon_state_2];

        let mut i = 0;

        for state in unwrapped_sm.states.iter() {
            let unwrapped_state = &*state.read().unwrap();
            let ps = pigeon_states[i].clone();

            match unwrapped_state {
                State::Playback {
                    config: state_config,
                    reset_context: _,
                    animation_id: _,
                    width: _,
                    height: _,
                    transitions: state_transitions,
                } => match ps {
                    State::Playback {
                        config,
                        reset_context: _,
                        animation_id: _,
                        width: _,
                        height: _,
                        transitions,
                    } => {
                        let first_transition = &*state_transitions[0].read().unwrap();
                        let second_transition = &*transitions[0].read().unwrap();

                        assert!(*state_config == config, "State config is not equal");
                        assert!(
                            first_transition.get_target_state()
                                == second_transition.get_target_state(),
                            "Transition target state is not equal"
                        );
                    }
                    _ => {
                        assert!(false, "State is not Playback");
                    }
                },
                _ => {
                    assert!(false, "State is not Playback");
                }
            }

            i += 1;
        }

        assert_eq!(i, 3)
    }

    #[test]
    fn state_machine_observer_test() {
        // We create 3 separate observers to test the different methods
        // Otherwise if we use the same observer all three events will modify the same data
        pub struct SMObserver1 {
            pub custom_data: RwLock<String>,
        }

        impl StateMachineObserver for SMObserver1 {
            fn transition_occured(&self, previous_state: &State, new_state: &State) {
                let transition_event = previous_state
                    .get_transitions()
                    .get(0)
                    .unwrap()
                    .read()
                    .unwrap()
                    .get_event();

                let previous_state_transition_event = &*transition_event.read().unwrap();

                let next_transition_event = new_state
                    .get_transitions()
                    .get(0)
                    .unwrap()
                    .read()
                    .unwrap()
                    .get_event();

                let next_state_transition_event = &*next_transition_event.read().unwrap();

                *self.custom_data.write().unwrap() = format!(
                    "{:?} -> {:?}",
                    previous_state_transition_event.as_str(),
                    next_state_transition_event.as_str()
                );
            }

            fn on_state_entered(&self, _entering_state: &State) {}

            fn on_state_exit(&self, _leaving_state: &State) {}
        }

        pub struct SMObserver2 {
            pub custom_data: RwLock<String>,
        }

        impl StateMachineObserver for SMObserver2 {
            fn transition_occured(&self, previous_state: &State, new_state: &State) {}

            fn on_state_entered(&self, entering_state: &State) {
                let transition_event = entering_state
                    .get_transitions()
                    .get(0)
                    .unwrap()
                    .read()
                    .unwrap()
                    .get_event();

                let state_transition_event = &*transition_event.read().unwrap();

                *self.custom_data.write().unwrap() =
                    format!("{:?}", state_transition_event.as_str(),);
            }

            fn on_state_exit(&self, _leaving_state: &State) {}
        }

        pub struct SMObserver3 {
            pub custom_data: RwLock<String>,
        }

        impl StateMachineObserver for SMObserver3 {
            fn transition_occured(&self, _previous_state: &State, _new_state: &State) {}

            fn on_state_entered(&self, _entering_state: &State) {}

            fn on_state_exit(&self, leaving_state: &State) {
                let transition_event = leaving_state
                    .get_transitions()
                    .get(0)
                    .unwrap()
                    .read()
                    .unwrap()
                    .get_event();

                let state_transition_event = &*transition_event.read().unwrap();

                *self.custom_data.write().unwrap() =
                    format!("{:?}", state_transition_event.as_str(),);
            }
        }

        let observer = Arc::new(SMObserver1 {
            custom_data: RwLock::new("No event so far".to_string()),
        });
        let observer2 = Arc::new(SMObserver2 {
            custom_data: RwLock::new("No event so far".to_string()),
        });
        let observer3 = Arc::new(SMObserver3 {
            custom_data: RwLock::new("No event so far".to_string()),
        });

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

        player.state_machine_subscribe(observer.clone());

        assert_eq!(*observer.custom_data.read().unwrap(), "No event so far");

        // First test that the event doesn't fire if the guard is not met
        player.tmp_set_state_machine_context("counter_0", 5.0);
        player.post_event(&Event::String {
            value: "explosion".to_string(),
        });

        // Should stay the same value we initialized it at
        assert_eq!(*observer.custom_data.read().unwrap(), "No event so far");

        player.tmp_set_state_machine_context("counter_0", 18.0);
        player.post_event(&Event::String {
            value: "explosion".to_string(),
        });

        // Should go to stage 2
        assert_eq!(
            *observer.custom_data.read().unwrap(),
            "\"explosion\" -> \"complete\""
        );

        // Start second observer to test on_state_enter
        player.state_machine_subscribe(observer2.clone());

        // Should stay the same value we initialized it at
        assert_eq!(*observer2.custom_data.read().unwrap(), "No event so far");

        player.tmp_set_state_machine_string_context("counter_1", "not_the_same");
        player.post_event(&Event::String {
            value: "complete".to_string(),
        });

        // Should go to stage 3
        assert_eq!(*observer2.custom_data.read().unwrap(), "\"done\"");

        // Start third observer to test on_state_exit
        player.state_machine_subscribe(observer3.clone());

        // Should stay the same value we initialized it at
        assert_eq!(*observer3.custom_data.read().unwrap(), "No event so far");

        player.tmp_set_state_machine_bool_context("counter_2", false);
        player.post_event(&Event::String {
            value: "done".to_string(),
        });

        // Should go to stage 0 and use previous state so it should be "done"
        assert_eq!(*observer3.custom_data.read().unwrap(), "\"done\"");
    }
}
