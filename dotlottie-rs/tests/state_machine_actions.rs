use std::sync::{Arc, Mutex};

use dotlottie_rs::StateMachineObserver;

struct MockObserver {
    events: Arc<Mutex<Vec<String>>>,
}

impl MockObserver {
    fn new(events: Arc<Mutex<Vec<String>>>) -> Self {
        MockObserver { events }
    }
}

impl StateMachineObserver for MockObserver {
    fn on_transition(&self, previous_state: String, new_state: String) {
        let mut events = self.events.lock().unwrap();
        events.push(format!(
            "on_transition: {} -> {}",
            previous_state, new_state
        ));
    }

    fn on_state_entered(&self, entering_state: String) {
        let mut events = self.events.lock().unwrap();
        events.push(format!("on_state_entered: {}", entering_state));
    }

    fn on_state_exit(&self, leaving_state: String) {
        let mut events = self.events.lock().unwrap();
        events.push(format!("on_state_exit: {}", leaving_state));
    }

    fn on_custom_event(&self, message: String) {
        let mut events = self.events.lock().unwrap();
        events.push(format!("custom_event: {}", message));
    }

    fn on_start(&self) {
        // todo!()
    }

    fn on_stop(&self) {
        // todo!()
    }

    fn on_string_input_value_change(
        &self,
        _input_name: String,
        _old_value: String,
        _new_value: String,
    ) {
        // todo!()
    }

    fn on_numeric_input_value_change(&self, _input_name: String, _old_value: f32, _new_value: f32) {
        // todo!()
    }

    fn on_boolean_input_value_change(
        &self,
        _input_name: String,
        _old_value: bool,
        _new_value: bool,
    ) {
        // todo!()
    }

    fn on_input_fired(&self, _input_name: String) {
        // todo!()
    }

    fn on_error(&self, _error: String) {
        // todo!()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use dotlottie_rs::{actions::open_url::OpenUrl, Config, DotLottiePlayer, StateMachineObserver};

    fn get_current_state_name(player: &DotLottiePlayer) -> String {
        player.state_machine_current_state()
    }

    #[test]
    fn increment() {
        let global_state = include_str!("fixtures/statemachines/action_tests/inc_rating.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start(OpenUrl::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        // Tests default increment without a value
        player.state_machine_set_numeric_input("rating", 1.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_2");

        // Tests adding with value
        player.state_machine_set_numeric_input("rating", 3.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_5");

        // Tests add from a input
        player.state_machine_set_numeric_input("rating", 6.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_12");

        // Tests add from a inexistant input, increments by 1.0 instead
        player.state_machine_set_numeric_input("rating", 13.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_14");
    }

    #[test]
    fn decrement() {
        let global_state = include_str!("fixtures/statemachines/action_tests/decr_rating.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start(OpenUrl::default());

        assert!(l);
        assert!(s);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "global");

        // Tests decrement from an inexistant input, decrements by 1.0 instead
        player.state_machine_set_numeric_input("rating", 13.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_12");

        // Tests decrement from a input
        player.state_machine_set_numeric_input("rating", 6.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_6");

        // Tests decrementing with value
        player.state_machine_set_numeric_input("rating", 3.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_1");

        // Tests default increment without a value
        player.state_machine_set_numeric_input("rating", 5.0);
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_4");
    }

    #[test]
    fn toggle() {
        let global_state = include_str!("fixtures/statemachines/action_tests/toggle.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start(OpenUrl::default());

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
        let global_state = include_str!("fixtures/statemachines/action_tests/set_inputs.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start(OpenUrl::default());

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
        let global_state = include_str!("fixtures/statemachines/action_tests/set_inputs.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start(OpenUrl::default());

        assert!(l);
        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        player.state_machine_set_numeric_input("NumericInput", 10.0);

        // C state should of set the switch to true, landing us in state a
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "d");
    }

    #[test]
    fn set_string() {
        let global_state = include_str!("fixtures/statemachines/action_tests/set_inputs.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
        let l = player.state_machine_load_data(global_state);
        let s = player.state_machine_start(OpenUrl::default());

        assert!(l);
        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        player.state_machine_set_string_input("StringInput", "second");

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
        let s = player.state_machine_start(OpenUrl::default());

        assert!(l);
        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        player.state_machine_set_boolean_input("OnOffSwitch", true);

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
        let s = player.state_machine_start(OpenUrl::default());

        assert!(l);
        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        // B Should of set the frame to input value "frame_holder", meaning 35
        assert_eq!(player.current_frame(), 35.0);

        player.state_machine_set_boolean_input("OnOffSwitch", true);

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
        let s = player.state_machine_start(OpenUrl::default());

        assert!(l);
        assert!(s);

        // Since switch is false by default, on load we land in the b state
        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "b");

        // B Should of set the frame to input value "frame_holder", 75% of the animation
        assert_eq!(player.current_frame(), 66.75);

        player.state_machine_set_boolean_input("OnOffSwitch", true);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "a");

        // A Should of set the progress to 10%
        assert_eq!(player.current_frame(), 8.900001);
    }

    #[test]
    fn reset() {
        let reset_sm = include_str!("fixtures/statemachines/action_tests/reset.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        assert_eq!(player.current_frame(), 0.0);

        let l = player.state_machine_load_data(reset_sm);
        let s = player.state_machine_start(OpenUrl::default());

        assert!(l);
        assert!(s);

        player.state_machine_set_numeric_input("rating", 3.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_3");

        player.state_machine_set_numeric_input("rating", 6.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_0");
    }

    #[test]
    fn fire_custom_event() {
        let reset_sm = include_str!("fixtures/statemachines/action_tests/fire_custom_event.json");
        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);

        assert_eq!(player.current_frame(), 0.0);

        let l = player.state_machine_load_data(reset_sm);
        let s = player.state_machine_start(OpenUrl::default());

        let events = Arc::new(Mutex::new(vec![]));
        let observer_events = Arc::clone(&events);
        let observer = MockObserver::new(observer_events);
        let observer_arc: Arc<dyn StateMachineObserver> = Arc::new(observer);
        player.state_machine_subscribe(Arc::clone(&observer_arc));

        assert!(l);
        assert!(s);

        player.state_machine_set_numeric_input("rating", 3.0);

        let curr_state_name = get_current_state_name(&player);
        assert_eq!(curr_state_name, "star_3");

        let expected_events = vec![
            "on_transition: star_0 -> star_3".to_string(),
            "on_state_exit: star_0".to_string(),
            "on_state_entered: star_3".to_string(),
            "custom_event: WOOHOO STAR 3".to_string(),
        ];

        let recorded_events = events.lock().unwrap();

        for (i, event) in recorded_events.iter().enumerate() {
            assert_eq!(
                event, &expected_events[i],
                "Mismatch at event index {}: expected '{}', found '{}'",
                i, expected_events[i], event
            );
        }
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
}
