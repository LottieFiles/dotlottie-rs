#[cfg(test)]
mod tests {
    use dotlottie_rs::{states::StateTrait, Config, DotLottiePlayer, Event};

    fn get_current_state_name(player: &DotLottiePlayer) -> String {
        let sm = player.get_state_machine();
        let read_lock = sm.try_read();

        match read_lock {
            Ok(sm) => {
                let engine = &*sm;

                if let Some(engine) = engine {
                    let curr_state = &engine.current_state;

                    if let Some(curr_state) = curr_state {
                        let name = curr_state.name();
                        return name;
                    }
                }
            }
            Err(_) => return "".to_string(),
        }

        "".to_string()
    }

    #[test]
    pub fn pointer_down_up_test() {
        let global_state =
            include_str!("fixtures/statemachines/listener_tests/pointer_down_up.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);

        let s = player.state_machine_start();

        assert_eq!(l, true);
        assert_eq!(s, true);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        player.state_machine_post_event(&Event::PointerDown { x: 15.0, y: 45.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_1");

        player.state_machine_post_event(&Event::PointerDown { x: 30.0, y: 45.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_2");

        player.state_machine_post_event(&Event::PointerDown { x: 45.0, y: 45.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_3");

        player.state_machine_post_event(&Event::PointerDown { x: 60.0, y: 45.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_4");

        player.state_machine_post_event(&Event::PointerDown { x: 75.0, y: 45.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_5");

        // Test that pointerUp anywhere on the canvas sets us back to global
        player.state_machine_post_event(&Event::PointerUp { x: 0.0, y: 0.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_6");
    }

    #[test]
    pub fn pointer_down_test() {
        let global_state = include_str!("fixtures/statemachines/listener_tests/pointer_down.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert_eq!(l, true);
        assert_eq!(s, true);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        player.state_machine_post_event(&Event::PointerDown { x: 0.0, y: 0.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_4");
    }

    #[test]
    pub fn pointer_enter_exit_test() {
        let global_state =
            include_str!("fixtures/statemachines/listener_tests/pointer_enter_exit.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert_eq!(l, true);
        assert_eq!(s, true);

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

        // This should keep rating at 5 since we're still in the last star
        player.state_machine_post_event(&Event::PointerExit { x: 75.0, y: 45.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_5");

        // This should no keep rating at 5 since we're not in the last star
        player.state_machine_post_event(&Event::PointerExit { x: 0.0, y: 0.0 });
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_0");
    }

    #[test]
    pub fn pointer_move_test() {
        let global_state = include_str!("fixtures/statemachines/listener_tests/pointer_move.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert_eq!(l, true);
        assert_eq!(s, true);

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
        let global_state = include_str!("fixtures/statemachines/listener_tests/on_complete.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert_eq!(l, true);
        assert_eq!(s, true);

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
        let global_state = include_str!("fixtures/statemachines/listener_tests/pigeon_fsm.json");
        let player = DotLottiePlayer::new(Config::default());

        player.load_dotlottie_data(include_bytes!("fixtures/pigeon.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert_eq!(l, true);
        assert_eq!(s, true);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "pigeonRunning");

        player.state_machine_post_event(&Event::PointerDown { x: 0.0, y: 0.0 });

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "explosion");

        while !player.is_complete() {
            let next_frame = player.request_frame();
            if player.set_frame(next_frame) {
                player.render();
            }
        }

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "feathersFalling");
    }
}
