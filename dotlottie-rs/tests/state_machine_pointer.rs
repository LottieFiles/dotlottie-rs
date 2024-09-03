#[cfg(test)]
mod tests {
    use dotlottie_player_core::{states::StateTrait, Config, DotLottiePlayer, Event};

    #[test]
    pub fn pointer_down_up_test() {
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star-rating.lottie"), 100, 100);

        // Load the state machine as a string
        let sm_config = include_str!("fixtures/star_pointer_up_down_sm.json");
        let load_result = player.load_state_machine_data(sm_config);
        let start_result = player.start_state_machine();

        assert!(load_result);
        assert!(start_result);

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        assert_eq!(sm.read().unwrap().as_ref().unwrap().states.len(), 6);

        // Send pointer coordinates to activate first state (first star)
        let event = Event::OnPointerDown { x: 15.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 2);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "star1"
        );

        // Send pointer coordinates to de-activate first state (first star)
        let event = Event::OnPointerUp { x: 15.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 0);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "global"
        );

        // Send pointer coordinates to activate second state (second star)
        let event = Event::OnPointerDown { x: 30.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 2);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "star2"
        );

        // Send pointer coordinates to de-activate second state (second star)
        let event = Event::OnPointerUp { x: 30.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 0);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "global"
        );

        // Send pointer coordinates to activate third state (third star)
        let event = Event::OnPointerDown { x: 45.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 2);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "star3"
        );

        // Send pointer coordinates to de-activate third state (third star)
        let event = Event::OnPointerUp { x: 45.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 0);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "global"
        );

        // Send pointer coordinates to activate fourth state (fourth star)
        let event = Event::OnPointerDown { x: 60.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 2);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "star4"
        );

        // Send pointer coordinates to de-activate fourth state (fourth star)
        let event = Event::OnPointerUp { x: 60.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 0);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "global"
        );

        // Send pointer coordinates to activate fifth state (fifth star)
        let event = Event::OnPointerDown { x: 74.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 2);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "star5"
        );

        // Send pointer coordinates to de-activate fifth state (fifth star)
        let event = Event::OnPointerUp { x: 74.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 0);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "global"
        );
    }

    // Equivalent to hovering
    #[test]
    pub fn pointer_enter_exit_test() {
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star-rating.lottie"), 100, 100);

        // Load the state machine as a string
        let sm_config = include_str!("fixtures/star_pointer_enter_exit_sm.json");
        let load_result = player.load_state_machine_data(sm_config);
        let start_result = player.start_state_machine();

        assert!(load_result);
        assert!(start_result);

        let sm = player.get_state_machine();

        assert!(
            sm.read().unwrap().as_ref().is_some(),
            "State machine is not loaded"
        );

        assert_eq!(sm.read().unwrap().as_ref().unwrap().states.len(), 6);

        // Send pointer coordinates to activate first state (first star)
        let event = Event::OnPointerMove { x: 15.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 2);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "star1"
        );

        // Send pointer coordinates to de-activate first state (first star)
        let event = Event::OnPointerMove { x: 0.0, y: 0.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 0);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "global"
        );

        // Send pointer coordinates to activate second state (second star)
        let event = Event::OnPointerMove { x: 30.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 2);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "star2"
        );

        // Send pointer coordinates to de-activate second state (second star)
        let event = Event::OnPointerMove { x: 0.0, y: 0.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 0);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "global"
        );

        // Send pointer coordinates to activate third state (third star)
        let event = Event::OnPointerMove { x: 45.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 2);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "star3"
        );

        // Send pointer coordinates to de-activate third state (third star)
        let event = Event::OnPointerMove { x: 0.0, y: 0.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 0);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "global"
        );

        // Send pointer coordinates to activate fourth state (fourth star)
        let event = Event::OnPointerMove { x: 60.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 2);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "star4"
        );

        // Send pointer coordinates to de-activate fourth state (fourth star)
        let event = Event::OnPointerMove { x: 0.0, y: 0.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 0);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "global"
        );

        // Send pointer coordinates to activate fifth state (fifth star)
        let event = Event::OnPointerMove { x: 74.0, y: 45.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 2);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "star5"
        );

        // Send pointer coordinates to de-activate fifth state (fifth star)
        let event = Event::OnPointerMove { x: 0.0, y: 0.0 };
        let pe = player.post_event(&event);
        assert_eq!(pe, 0);
        assert_eq!(
            sm.read()
                .unwrap()
                .as_ref()
                .unwrap()
                .current_state
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .get_name(),
            "global"
        );
    }
}
