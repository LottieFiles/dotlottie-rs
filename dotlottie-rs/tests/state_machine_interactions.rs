#[cfg(test)]
mod tests {
    use dotlottie_rs::{
        actions::open_url_policy::OpenUrlPolicy, Config, DotLottiePlayer, Event, Observer,
    };
    use std::sync::{Arc, Mutex};

    fn get_current_state_name(player: &DotLottiePlayer) -> String {
        player.state_machine_current_state()
    }

    struct MockObserver {
        events: Arc<Mutex<Vec<String>>>,
    }

    impl MockObserver {
        fn new(events: Arc<Mutex<Vec<String>>>) -> Self {
            MockObserver { events }
        }
    }

    impl Observer for MockObserver {
        fn on_load(&self) {}

        fn on_load_error(&self) {}

        fn on_play(&self) {}

        fn on_pause(&self) {}

        fn on_stop(&self) {}

        fn on_frame(&self, _frame_no: f32) {}

        fn on_render(&self, _frame_no: f32) {}

        fn on_loop(&self, loop_count: u32) {
            let mut events = self.events.lock().unwrap();

            events.push(format!("on_loop {loop_count}"));
        }

        fn on_complete(&self) {}
    }

    #[test]
    pub fn pointer_down_up_test() {
        #[cfg(feature = "tvg-v1")]
        {
            let global_state =
                include_str!("fixtures/statemachines/interaction_tests/pointer_down_up.json");
            let player = DotLottiePlayer::new(Config::default());
            player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
            let l = player.state_machine_load_data(global_state);

            let s = player.state_machine_start(OpenUrlPolicy::default());

            assert!(l);
            assert!(s);

            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "global");

            let star_1_box = player.get_layer_bounds("star_1");

            player.state_machine_post_event(&Event::PointerDown {
                x: star_1_box[0],
                y: star_1_box[1],
            });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_1");

            let star_2_box = player.get_layer_bounds("star_2");

            player.state_machine_post_event(&Event::PointerDown {
                x: star_2_box[0],
                y: star_2_box[1],
            });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_2");

            let star_3_box = player.get_layer_bounds("star_3");
            player.state_machine_post_event(&Event::PointerDown {
                x: star_3_box[0],
                y: star_3_box[1],
            });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_3");

            let star_4_box = player.get_layer_bounds("star_4");
            player.state_machine_post_event(&Event::PointerDown {
                x: star_4_box[0],
                y: star_4_box[1],
            });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_4");

            let star_5_box = player.get_layer_bounds("star_5");
            player.state_machine_post_event(&Event::PointerDown {
                x: star_5_box[0],
                y: star_5_box[1],
            });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_5");

            let star_6_box = player.get_layer_bounds("star_6");
            // Test that pointerUp anywhere on the canvas sets us back to global
            player.state_machine_post_event(&Event::PointerUp {
                x: star_6_box[0],
                y: star_6_box[1],
            });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_6");
        }
    }

    #[test]
    pub fn pointer_down_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_down.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start(OpenUrlPolicy::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        player.state_machine_post_event(&Event::PointerDown { x: 0.0, y: 0.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_4");
    }

    #[test]
    pub fn pointer_enter_test() {
        #[cfg(feature = "tvg-v1")]
        {
            let global_state =
                include_str!("fixtures/statemachines/interaction_tests/pointer_enter.json");
            let player = DotLottiePlayer::new(Config::default());
            player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
            let l = player.state_machine_load_data(global_state);
            let s = player.state_machine_start(OpenUrlPolicy::default());

            assert!(l);
            assert!(s);

            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_0");

            player.state_machine_post_event(&Event::PointerEnter { x: 15.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_1");

            player.state_machine_post_event(&Event::PointerEnter { x: 30.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_2");

            player.state_machine_post_event(&Event::PointerEnter { x: 45.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_3");

            player.state_machine_post_event(&Event::PointerEnter { x: 60.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_4");

            player.state_machine_post_event(&Event::PointerEnter { x: 75.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_5");
        }
    }

    #[test]
    pub fn pointer_enter_via_move_test() {
        #[cfg(feature = "tvg-v1")]
        {
            let global_state =
                include_str!("fixtures/statemachines/interaction_tests/pointer_enter.json");
            let player = DotLottiePlayer::new(Config::default());
            player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
            let l = player.state_machine_load_data(global_state);
            let s = player.state_machine_start(OpenUrlPolicy::default());

            assert!(l);
            assert!(s);

            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_0");

            player.state_machine_post_event(&Event::PointerMove { x: 15.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_1");

            player.state_machine_post_event(&Event::PointerMove { x: 30.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_2");

            player.state_machine_post_event(&Event::PointerMove { x: 45.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_3");

            player.state_machine_post_event(&Event::PointerMove { x: 60.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_4");

            player.state_machine_post_event(&Event::PointerMove { x: 75.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_5");
        }
    }

    #[test]
    pub fn pointer_exit_test() {
        #[cfg(feature = "tvg-v1")]
        {
            let global_state =
                include_str!("fixtures/statemachines/interaction_tests/pointer_exit.json");
            let player = DotLottiePlayer::new(Config::default());
            player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
            let l = player.state_machine_load_data(global_state);
            let s = player.state_machine_start(OpenUrlPolicy::default());

            assert!(l);
            assert!(s);

            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_0");

            player.state_machine_post_event(&Event::PointerEnter { x: 15.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_1");
            player.state_machine_post_event(&Event::PointerExit { x: 0.0, y: 0.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_0");
        }
    }

    #[test]
    pub fn pointer_exit_via_move_test() {
        #[cfg(feature = "tvg-v1")]
        {
            let global_state =
                include_str!("fixtures/statemachines/interaction_tests/pointer_exit.json");
            let player = DotLottiePlayer::new(Config::default());
            player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
            let l = player.state_machine_load_data(global_state);
            let s = player.state_machine_start(OpenUrlPolicy::default());

            assert!(l);
            assert!(s);

            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_0");

            player.state_machine_post_event(&Event::PointerMove { x: 15.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_1");
            player.state_machine_post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_0");

            player.state_machine_post_event(&Event::PointerMove { x: 30.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_2");
            player.state_machine_post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_0");

            player.state_machine_post_event(&Event::PointerMove { x: 45.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_3");
            player.state_machine_post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_0");

            player.state_machine_post_event(&Event::PointerMove { x: 60.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_4");
            player.state_machine_post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_0");

            player.state_machine_post_event(&Event::PointerMove { x: 75.0, y: 45.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_5");
            player.state_machine_post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
            let curr_state_name = get_current_state_name(&player);
            assert_eq!(curr_state_name, "star_0");
        }
    }

    #[test]
    pub fn pointer_move_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_move.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start(OpenUrlPolicy::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        // Posting PointerMove should increase the rating by one
        player.state_machine_post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_1");

        player.state_machine_post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_2");

        player.state_machine_post_event(&Event::PointerMove { x: 0.0, y: 0.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_3");
    }

    #[test]
    pub fn on_complete_manual_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/on_complete.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start(OpenUrlPolicy::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        // Posting OnComplete should increase the rating by one
        player.state_machine_post_event(&Event::OnComplete {});
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_1");

        player.state_machine_post_event(&Event::OnComplete {});
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_2");

        player.state_machine_post_event(&Event::OnComplete {});
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_3");
    }

    #[test]
    pub fn on_complete_player_test() {
        let global_state = include_str!("fixtures/statemachines/interaction_tests/pigeon_fsm.json");
        let player = DotLottiePlayer::new(Config::default());

        player.load_dotlottie_data(include_bytes!("fixtures/pigeon.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start(OpenUrlPolicy::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "PigeonRunning");

        player.state_machine_post_event(&Event::PointerDown { x: 0.0, y: 0.0 });

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "Explosion");

        while !player.is_complete() {
            let next_frame = player.request_frame();
            if player.set_frame(next_frame) {
                player.render();
            }
        }

        // let curr_state_name = get_current_state_name(&player);
        // assert_eq!(curr_state_name, "Feathers falling");
    }

    #[test]
    pub fn on_loop_complete_player_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/on_loop_complete.json");
        let player = DotLottiePlayer::new(Config::default());

        player.load_dotlottie_data(include_bytes!("fixtures/pigeon.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start(OpenUrlPolicy::default());

        let events = Arc::new(Mutex::new(vec![]));
        let observer_events = Arc::clone(&events);
        let observer = MockObserver::new(observer_events);
        let observer_arc: Arc<dyn Observer> = Arc::new(observer);
        player.subscribe(Arc::clone(&observer_arc));

        assert!(l);
        assert!(s);

        let mut count = 0;
        while count < 4 {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) {
                player.render();
            }

            if player.is_complete() {
                count += 1;
            }
        }

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "explosion");
    }
}
