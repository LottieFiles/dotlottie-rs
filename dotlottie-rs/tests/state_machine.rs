mod test_utils;

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::Read,
        sync::{Arc, RwLock},
    };

    use dotlottie_player_core::transitions::{Transition::Transition, TransitionTrait};

    use dotlottie_player_core::{events::Event, states::State, Config, DotLottiePlayer, Mode};

    use crate::test_utils::{HEIGHT, WIDTH};

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
}
