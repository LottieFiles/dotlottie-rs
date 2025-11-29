#[cfg(test)]
mod tests {
    use dotlottie_rs::{
        Config, DotLottiePlayer, Event, actions::open_url_policy::OpenUrlPolicy
    };

    #[test]
    pub fn pointer_down_up_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_down_up.json");

        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");

        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        let star_1_box = sm.player.get_layer_bounds("star1");

        sm.post_event(&Event::PointerDown {
            x: star_1_box[0],
            y: star_1_box[1],
        });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_1");

        let star_2_box = sm.player.get_layer_bounds("star2");

        sm.post_event(&Event::PointerDown {
            x: star_2_box[0],
            y: star_2_box[1],
        });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");

        let star_3_box = sm.player.get_layer_bounds("star3");
        sm.post_event(&Event::PointerDown {
            x: star_3_box[0],
            y: star_3_box[1],
        });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");

        let star_4_box = sm.player.get_layer_bounds("star4");
        sm.post_event(&Event::PointerDown {
            x: star_4_box[0],
            y: star_4_box[1],
        });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_4");

        let star_5_box = sm.player.get_layer_bounds("star5");
        sm.post_event(&Event::PointerDown {
            x: star_5_box[0],
            y: star_5_box[1],
        });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_5");

        let star_6_box = sm.player.get_layer_bounds("star 6");
        // Test that pointerUp anywhere on the canvas sets us back to global
        sm.post_event(&Event::PointerUp {
            x: star_6_box[0],
            y: star_6_box[1],
        });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_6");
    }

    #[test]
    pub fn pointer_down_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_down.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        sm.post_event(&Event::PointerDown { x: 0.0, y: 0.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_4");
    }

    #[test]
    pub fn pointer_enter_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_enter.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_0");

        sm.post_event(&Event::PointerEnter { x: 15.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_1");

        sm.post_event(&Event::PointerEnter { x: 30.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");

        sm.post_event(&Event::PointerEnter { x: 45.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");

        sm.post_event(&Event::PointerEnter { x: 60.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_4");

        sm.post_event(&Event::PointerEnter { x: 75.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_5");
    }

    #[test]
    pub fn pointer_enter_via_move_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_enter.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_0");

        sm.post_event(&Event::PointerMove { x: 15.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_1");

        sm.post_event(&Event::PointerMove { x: 30.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");

        sm.post_event(&Event::PointerMove { x: 45.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");

        sm.post_event(&Event::PointerMove { x: 60.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_4");

        sm.post_event(&Event::PointerMove { x: 75.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_5");
    }

    #[test]
    pub fn pointer_exit_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_exit.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_0");

        sm.post_event(&Event::PointerEnter { x: 15.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_1");
        sm.post_event(&Event::PointerExit { x: 0.0, y: 0.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_0");
    }

    #[test]
    pub fn pointer_exit_via_move_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_exit.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_0");

        sm.post_event(&Event::PointerMove { x: 15.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_1");
        sm.post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_0");

        sm.post_event(&Event::PointerMove { x: 30.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");
        sm.post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_0");

        sm.post_event(&Event::PointerMove { x: 45.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");
        sm.post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_0");

        sm.post_event(&Event::PointerMove { x: 60.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_4");
        sm.post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_0");

        sm.post_event(&Event::PointerMove { x: 75.0, y: 45.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_5");
        sm.post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_0");
    }

    #[test]
    pub fn pointer_move_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_move.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        // Posting PointerMove should increase the rating by one
        sm.post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_1");

        sm.post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");

        sm.post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");
    }

    #[test]
    pub fn on_complete_manual_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/on_complete.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "global");

        // Posting OnComplete should increase the rating by one
        sm.post_event(&Event::OnComplete {});
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_1");

        sm.post_event(&Event::OnComplete {});
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_2");

        sm.post_event(&Event::OnComplete {});
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "star_3");
    }

    #[test]
    pub fn on_complete_player_test() {
        let global_state = include_str!("fixtures/statemachines/interaction_tests/pigeon_fsm.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);

        player.load_dotlottie_data(include_bytes!("fixtures/pigeon.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "PigeonRunning");

        sm.post_event(&Event::PointerDown { x: 10.0, y: 10.0 });
        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "Explosion");
        loop {
            sm.tick();
            if sm.player.is_complete() { 
                break;
            }
        }

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "Feathers falling");
    }

    #[test]
    pub fn on_loop_complete_player_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/on_loop_complete.json");
        let mut player = DotLottiePlayer::new(Config::default(), 0);

        player.load_dotlottie_data(include_bytes!("fixtures/pigeon.lottie"), 100, 100);
        let mut sm = player.state_machine_load_data(global_state).expect("state machine to load successfully");
        let s = sm.start(&OpenUrlPolicy::default());

        assert!(s);

        loop {
            sm.tick();
            if sm.status() == "Stopped".to_string() {
                break;
            }
        }

        let curr_state_name = sm.get_current_state_name();
        assert_eq!(curr_state_name, "explosion");
    }
}
