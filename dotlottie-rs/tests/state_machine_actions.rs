#[cfg(test)]
mod tests {
    use dotlottie_rs::{states::StateTrait, Config, DotLottiePlayer};

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
    fn increment() {
        let global_state = include_str!("fixtures/statemachines/action_tests/inc_rating.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        // Tests default increment without a value
        player.state_machine_set_numeric_trigger("rating", 1.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_2");

        // Tests adding with value
        player.state_machine_set_numeric_trigger("rating", 3.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_5");

        // Tests add from a trigger
        player.state_machine_set_numeric_trigger("rating", 6.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_12");

        // Tests add from a inexistant trigger, increments by 1.0 instead
        player.state_machine_set_numeric_trigger("rating", 13.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_14");
    }

    #[test]
    fn decrement() {
        let global_state = include_str!("fixtures/statemachines/action_tests/decr_rating.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        // Tests decrement from an inexistant trigger, decrements by 1.0 instead
        player.state_machine_set_numeric_trigger("rating", 13.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_12");

        // Tests decrement from a trigger
        player.state_machine_set_numeric_trigger("rating", 6.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_6");

        // Tests decrementing with value
        player.state_machine_set_numeric_trigger("rating", 3.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_1");

        // Tests default increment without a value
        player.state_machine_set_numeric_trigger("rating", 5.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_4");
    }

    #[test]
    fn toggle() {
        let global_state = include_str!("fixtures/statemachines/action_tests/toggle.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        player.state_machine_fire_event("Step");

        // C state should of toggled the switch to true, landing us in state a
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");

        player.state_machine_fire_event("Step");

        // C state should of toggled the switch to false, landing us in state b
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");
    }

    #[test]
    fn set_boolean() {
        let global_state = include_str!("fixtures/statemachines/action_tests/set_triggers.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        player.state_machine_fire_event("Step");

        // C state should of set the switch to true, landing us in state a
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");
    }

    #[test]
    fn set_numeric() {
        let global_state = include_str!("fixtures/statemachines/action_tests/set_triggers.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        player.state_machine_set_numeric_trigger("NumericTrigger", 10.0);

        // C state should of set the switch to true, landing us in state a
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "d");
    }

    #[test]
    fn set_string() {
        let global_state = include_str!("fixtures/statemachines/action_tests/set_triggers.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        player.state_machine_set_string_trigger("StringTrigger", "second");

        // C state should of set the switch to true, landing us in state a
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "f");
    }

    #[test]
    fn fire() {
        let global_state = include_str!("fixtures/statemachines/action_tests/fire.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        player.state_machine_set_boolean_trigger("OnOffSwitch", true);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "c");
    }

    #[test]
    fn set_frame() {
        let global_state = include_str!("fixtures/statemachines/action_tests/set_frame.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        assert_eq!(player.current_frame(), 0.0);

        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        // B Should of set the frame to trigger value "frame_holder", meaning 35
        assert_eq!(player.current_frame(), 35.0);

        player.state_machine_set_boolean_trigger("OnOffSwitch", true);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");

        // A Should of set the frame to 10
        assert_eq!(player.current_frame(), 10.0);
    }

    #[test]
    fn set_progress() {
        let global_state = include_str!("fixtures/statemachines/action_tests/set_progress.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        assert_eq!(player.current_frame(), 0.0);

        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start();

        assert!(l);
        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        // B Should of set the frame to trigger value "frame_holder", 75% of the animation
        assert_eq!(player.current_frame(), 66.75);

        player.state_machine_set_boolean_trigger("OnOffSwitch", true);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");

        // A Should of set the progress to 10%
        assert_eq!(player.current_frame(), 8.900001);
    }

    // TODO
    #[test]
    fn theme_action() { // todo!()
    }

    #[test]
    fn fire_custom_event() {
        // todo!()
    }

    #[test]
    fn open_url() {
        // todo!()
    }

    #[test]
    fn set_slot() {
        // todo!()
    }

    #[test]
    fn set_theme() {
        // todo!()
    }

    #[test]
    fn set_expression() {
        // todo!()
    }

    #[test]
    fn reset() {
        // todo!()
    }
}
