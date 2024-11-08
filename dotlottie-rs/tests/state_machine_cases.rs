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
    pub fn global_and_guardless() {
        let global_state =
            include_str!("fixtures/statemachines/sanity_tests/test_global_and_guardless.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/smileys.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);

        let s = player.state_machine_start();

        assert_eq!(l, true);
        assert_eq!(s, true);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        player.state_machine_set_numeric_trigger("Rating", 2.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        player.state_machine_set_numeric_trigger("Rating", 1.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "d");
    }

    #[test]
    pub fn guarded_and_guardless() {
        let global_state =
            include_str!("fixtures/statemachines/sanity_tests/test_guarded_and_guardless.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/smileys.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);

        let s = player.state_machine_start();

        assert_eq!(l, true);
        assert_eq!(s, true);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");

        player.state_machine_fire_event("Step");

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "d");

        player.state_machine_set_numeric_trigger("r", 1.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");

        player.state_machine_fire_event("Step");
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "g");
    }

    #[test]
    pub fn guardless_and_event_propagation() {
        let global_state = include_str!(
            "fixtures/statemachines/sanity_tests/test_guardless_and_event_propagation.json"
        );
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/smileys.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);

        let s = player.state_machine_start();

        assert_eq!(l, true);
        assert_eq!(s, true);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        player.state_machine_set_numeric_trigger("Rating", 1.0);
        player.state_machine_fire_event("Step");
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "e");
    }

    #[test]
    pub fn guardless_and_event() {
        let global_state =
            include_str!("fixtures/statemachines/sanity_tests/test_guardless_and_event.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/smileys.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);

        let s = player.state_machine_start();

        assert_eq!(l, true);
        assert_eq!(s, true);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        player.state_machine_set_numeric_trigger("Rating", 1.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "c");
    }
}
