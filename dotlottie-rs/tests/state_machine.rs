#[cfg(test)]
mod tests {
    use core::{assert_eq, option::Option::Some};

    use dotlottie_rs::{Config, DotLottiePlayer, Event};

    #[test]
    fn get_state_machine() {
        // Tests with no state machine loaded
        let player = DotLottiePlayer::new(Config::default());
        let l = player.get_state_machine();
        let m = &*l;
        let r = m.try_read();

        match r {
            Ok(read_lock) => {
                let state_machine = read_lock;
                assert!(state_machine.is_none());
            }
            Err(_) => {
                assert!(false);
            }
        }

        // Tests with a state machine loaded
        let global_state = include_str!("fixtures/statemachines/action_tests/inc_rating.json");
        player.state_machine_load_data(global_state);

        let l = player.get_state_machine();
        let m = &*l;
        let r = m.try_read();

        match r {
            Ok(read_lock) => {
                let state_machine = read_lock;
                assert!(state_machine.is_some());
                if let Some(sm) = &*state_machine {
                    assert_eq!(sm.get_current_state_name(), "global");
                }
            }
            Err(_) => {
                assert!(false);
            }
        }
    }

    #[test]
    fn state_machine_start() {
        let player = DotLottiePlayer::new(Config::default());

        player.state_machine_load_data("bad_data");

        let r = player.state_machine_start();

        assert_eq!(r, false);

        let global_state = include_str!("fixtures/statemachines/action_tests/inc_rating.json");
        player.state_machine_load_data(global_state);

        let r = player.state_machine_start();

        assert!(r);
    }

    #[test]
    fn state_machine_stop() {
        let player = DotLottiePlayer::new(Config::default());

        player.state_machine_load_data("bad_data");

        // Not started
        let r = player.state_machine_stop();

        assert_eq!(r, false);

        let global_state = include_str!("fixtures/statemachines/action_tests/inc_rating.json");
        player.state_machine_load_data(global_state);

        let r = player.state_machine_start();
        let s = player.state_machine_stop();

        assert!(r);
        assert!(s);
    }

    #[test]
    fn state_machine_framework_setup() {
        let player = DotLottiePlayer::new(Config::default());
        let pointer_down =
            include_str!("fixtures/statemachines/listener_tests/listener_array.json");

        player.state_machine_load_data(pointer_down);

        let r = player.state_machine_start();
        assert!(r);

        let r = player.state_machine_framework_setup();

        assert!(r.contains(&"PointerDown".to_string()));
        assert!(r.contains(&"PointerUp".to_string()));
        assert!(r.contains(&"PointerMove".to_string()));
        assert!(r.contains(&"PointerEnter".to_string()));
        assert!(r.contains(&"PointerExit".to_string()));
        assert!(r.contains(&"OnComplete".to_string()));
    }

    #[test]
    fn state_machine_post_event() {
        let player = DotLottiePlayer::new(Config::default());
        let pointer_down =
            include_str!("fixtures/statemachines/listener_tests/all_listener_events.json");

        player.state_machine_load_data(pointer_down);

        let r = player.state_machine_start();
        assert!(r);

        let event = Event::PointerDown { x: 0.0, y: 0.0 };
        let r = player.state_machine_post_event(&event);
        assert_eq!(r, 0);
        assert_eq!(player.state_machine_current_state(), "a".to_string());

        let event = Event::PointerUp { x: 0.0, y: 0.0 };
        let r = player.state_machine_post_event(&event);
        assert_eq!(r, 0);
        assert_eq!(player.state_machine_current_state(), "b".to_string());

        let event = Event::PointerMove { x: 0.0, y: 0.0 };
        let r = player.state_machine_post_event(&event);
        assert_eq!(r, 0);
        assert_eq!(player.state_machine_current_state(), "c".to_string());

        let event = Event::PointerEnter { x: 0.0, y: 0.0 };
        let r = player.state_machine_post_event(&event);
        assert_eq!(r, 0);
        assert_eq!(player.state_machine_current_state(), "d".to_string());

        let event = Event::PointerExit { x: 0.0, y: 0.0 };
        let r = player.state_machine_post_event(&event);
        assert_eq!(r, 0);
        assert_eq!(player.state_machine_current_state(), "e".to_string());

        let event = Event::OnComplete;
        let r = player.state_machine_post_event(&event);
        assert_eq!(r, 0);
        assert_eq!(player.state_machine_current_state(), "f".to_string());
    }

    #[test]
    fn state_machine_set_get_numeric_trigger() {
        let player = DotLottiePlayer::new(Config::default());
        let rating = include_str!("fixtures/statemachines/normal_usecases/rating.json");

        player.state_machine_load_data(rating);

        let r = player.state_machine_start();
        assert!(r);

        // Setting the triggers
        player.state_machine_set_numeric_trigger("rating", 1.0);
        assert_eq!(player.state_machine_current_state(), "star_1".to_string());

        assert_eq!(player.state_machine_get_numeric_trigger("rating"), 1.0);

        player.state_machine_set_numeric_trigger("rating", 5.0);
        assert_eq!(player.state_machine_current_state(), "star_5".to_string());

        assert_eq!(player.state_machine_get_numeric_trigger("rating"), 5.0);
    }

    #[test]
    fn state_machine_set_get_boolean_trigger() {
        let player = DotLottiePlayer::new(Config::default());
        let sm = include_str!("fixtures/statemachines/normal_usecases/toggle.json");

        player.state_machine_load_data(sm);

        let r = player.state_machine_start();
        assert!(r);

        assert_eq!(
            player.state_machine_get_boolean_trigger("OnOffSwitch"),
            false
        );

        // Setting the triggers
        player.state_machine_set_boolean_trigger("OnOffSwitch", true);
        assert_eq!(player.state_machine_current_state(), "a".to_string());
        assert_eq!(
            player.state_machine_get_boolean_trigger("OnOffSwitch"),
            true
        );

        player.state_machine_set_boolean_trigger("OnOffSwitch", false);
        assert_eq!(player.state_machine_current_state(), "b".to_string());
        assert_eq!(
            player.state_machine_get_boolean_trigger("OnOffSwitch"),
            false
        );
    }

    #[test]
    fn state_machine_set_get_string_trigger() {
        let player = DotLottiePlayer::new(Config::default());
        let sm = include_str!("fixtures/statemachines/normal_usecases/password.json");

        player.state_machine_load_data(sm);

        let r = player.state_machine_start();
        assert!(r);

        assert_eq!(
            player.state_machine_get_string_trigger("password"),
            "incorrect".to_string()
        );

        // Setting the triggers
        player.state_machine_set_string_trigger("password", "welcome");
        assert_eq!(player.state_machine_current_state(), "a".to_string());
        assert_eq!(
            player.state_machine_get_string_trigger("password"),
            "welcome".to_string()
        );

        player.state_machine_set_string_trigger("password", "goodbye");
        assert_eq!(player.state_machine_current_state(), "b".to_string());
        assert_eq!(
            player.state_machine_get_string_trigger("password"),
            "goodbye".to_string()
        );
    }

    #[test]
    fn state_machine_fire_event() {
        let player = DotLottiePlayer::new(Config::default());
        let sm = include_str!("fixtures/statemachines/normal_usecases/password_with_events.json");

        player.state_machine_load_data(sm);

        let r = player.state_machine_start();
        assert!(r);

        player.state_machine_fire_event("Step");
        assert_eq!(player.state_machine_current_state(), "a".to_string());

        player.state_machine_fire_event("Step");
        assert_eq!(player.state_machine_current_state(), "b".to_string());
    }

    #[test]
    fn final_state() {
        let player = DotLottiePlayer::new(Config::default());
        let sm = include_str!("fixtures/statemachines/normal_usecases/final_state.json");

        player.state_machine_load_data(sm);

        let r = player.state_machine_start();
        assert!(r);

        assert_eq!(player.state_machine_current_state(), "star_0".to_string());

        player.state_machine_set_numeric_trigger("rating", 3.0);
        assert_eq!(player.state_machine_current_state(), "star_3".to_string());

        player.state_machine_set_numeric_trigger("rating", 5.0);
        assert_eq!(player.state_machine_current_state(), "star_1".to_string());

        player.state_machine_set_numeric_trigger("rating", 3.0);
        assert_eq!(player.state_machine_current_state(), "star_1".to_string());
    }

    #[test]
    fn state_machine_current_state() {
        let player = DotLottiePlayer::new(Config::default());
        let pointer_down =
            include_str!("fixtures/statemachines/listener_tests/all_listener_events.json");

        player.state_machine_load_data(pointer_down);

        let r = player.state_machine_start();
        assert!(r);

        let event = Event::PointerDown { x: 0.0, y: 0.0 };
        player.state_machine_post_event(&event);
        assert_eq!(player.state_machine_current_state(), "a".to_string());

        let event = Event::PointerUp { x: 0.0, y: 0.0 };
        player.state_machine_post_event(&event);
        assert_eq!(player.state_machine_current_state(), "b".to_string());

        let event = Event::PointerMove { x: 0.0, y: 0.0 };
        player.state_machine_post_event(&event);
        assert_eq!(player.state_machine_current_state(), "c".to_string());

        let event = Event::PointerEnter { x: 0.0, y: 0.0 };
        player.state_machine_post_event(&event);
        assert_eq!(player.state_machine_current_state(), "d".to_string());

        let event = Event::PointerExit { x: 0.0, y: 0.0 };
        player.state_machine_post_event(&event);
        assert_eq!(player.state_machine_current_state(), "e".to_string());

        let event = Event::OnComplete;
        player.state_machine_post_event(&event);
        assert_eq!(player.state_machine_current_state(), "f".to_string());
    }
}
