mod test_utils;

use crate::test_utils::{HEIGHT, WIDTH};

use core::assert_eq;
use std::fs::{self, File};

use dotlottie_rs::{
    actions::open_url_policy::OpenUrlPolicy, ColorSpace, Config, DotLottiePlayer, Event,
};
use std::io::Read;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn get_state_machine() {
        let config = Config {
            autoplay: true,
            ..Config::default()
        };
        let player = DotLottiePlayer::new(config);

        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];
        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );

        let mut markers =
            File::open("tests/fixtures/statemachines/normal_usecases/sm_exploding_pigeon.lottie")
                .expect("no file found");
        let metadatamarkers =
            fs::metadata("tests/fixtures/statemachines/normal_usecases/sm_exploding_pigeon.lottie")
                .expect("unable to read metadata");
        let mut markers_buffer = vec![0; metadatamarkers.len() as usize];
        markers
            .read_exact(&mut markers_buffer)
            .expect("buffer overflow");

        player.load_dotlottie_data(&markers_buffer, 500, 500);

        assert!(player.is_playing());

        let load = player.state_machine_load("Exploding Pigeon");
        let start = player.state_machine_start(OpenUrlPolicy::default());

        assert!(load);
        assert!(start);

        // Tests with a state machine loaded
        let global_state =
            include_str!("fixtures/statemachines/normal_usecases/exploding_pigeon.json");

        let l = player.get_state_machine("Exploding Pigeon");

        assert_eq!(l, global_state);
    }

    #[test]
    fn state_machine_start() {
        let player = DotLottiePlayer::new(Config::default());

        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];
        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );

        player.state_machine_load_data("bad_data");

        let r = player.state_machine_start(OpenUrlPolicy::default());

        assert!(!r);

        let global_state = include_str!("fixtures/statemachines/action_tests/inc_rating.json");
        player.state_machine_load_data(global_state);

        let r = player.state_machine_start(OpenUrlPolicy::default());

        assert!(r);
    }

    #[test]
    fn state_machine_stop() {
        let player = DotLottiePlayer::new(Config::default());

        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];
        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );

        player.state_machine_load_data("bad_data");

        // Not started
        let r = player.state_machine_stop();

        assert!(!r);

        let global_state = include_str!("fixtures/statemachines/action_tests/inc_rating.json");
        player.state_machine_load_data(global_state);

        let r = player.state_machine_start(OpenUrlPolicy::default());
        let s = player.state_machine_stop();

        assert!(r);
        assert!(s);
    }

    #[test]
    fn state_machine_framework_setup() {
        let player = DotLottiePlayer::new(Config::default());
        let pointer_down =
            include_str!("fixtures/statemachines/interaction_tests/interaction_array.json");

        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];
        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );

        player.state_machine_load_data(pointer_down);

        let r = player.state_machine_start(OpenUrlPolicy::default());
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
            include_str!("fixtures/statemachines/interaction_tests/all_interaction_events.json");

        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];
        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );

        player.state_machine_load_data(pointer_down);

        let r = player.state_machine_start(OpenUrlPolicy::default());
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

    #[test]
    fn state_machine_set_get_numeric_input() {
        let player = DotLottiePlayer::new(Config::default());

        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];
        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );

        let rating = include_str!("fixtures/statemachines/normal_usecases/rating.json");
        player.state_machine_load_data(rating);

        let r = player.state_machine_start(OpenUrlPolicy::default());
        assert!(r);

        // Setting the inputs
        player.state_machine_set_numeric_input("rating", 1.0);
        assert_eq!(player.state_machine_current_state(), "star_1".to_string());

        assert_eq!(player.state_machine_get_numeric_input("rating"), 1.0);

        player.state_machine_set_numeric_input("rating", 5.0);
        assert_eq!(player.state_machine_current_state(), "star_5".to_string());

        assert_eq!(player.state_machine_get_numeric_input("rating"), 5.0);
    }

    #[test]
    fn state_machine_set_get_boolean_input() {
        let player = DotLottiePlayer::new(Config::default());
        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];
        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );

        let sm = include_str!("fixtures/statemachines/normal_usecases/toggle.json");

        player.state_machine_load_data(sm);

        let r = player.state_machine_start(OpenUrlPolicy::default());
        assert!(r);

        assert!(!player.state_machine_get_boolean_input("OnOffSwitch"));

        // Setting the inputs
        player.state_machine_set_boolean_input("OnOffSwitch", true);
        assert_eq!(player.state_machine_current_state(), "a".to_string());
        assert!(player.state_machine_get_boolean_input("OnOffSwitch"));

        player.state_machine_set_boolean_input("OnOffSwitch", false);
        assert_eq!(player.state_machine_current_state(), "b".to_string());
        assert!(!player.state_machine_get_boolean_input("OnOffSwitch"));
    }

    #[test]
    fn state_machine_set_get_string_input() {
        let player = DotLottiePlayer::new(Config::default());

        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];
        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );

        let sm = include_str!("fixtures/statemachines/normal_usecases/password.json");

        player.state_machine_load_data(sm);

        let r = player.state_machine_start(OpenUrlPolicy::default());
        assert!(r);

        assert_eq!(
            player.state_machine_get_string_input("password"),
            "incorrect".to_string()
        );

        // Setting the inputs
        player.state_machine_set_string_input("password", "welcome");
        assert_eq!(player.state_machine_current_state(), "a".to_string());
        assert_eq!(
            player.state_machine_get_string_input("password"),
            "welcome".to_string()
        );

        player.state_machine_set_string_input("password", "goodbye");
        assert_eq!(player.state_machine_current_state(), "b".to_string());
        assert_eq!(
            player.state_machine_get_string_input("password"),
            "goodbye".to_string()
        );
    }

    #[test]
    fn state_machine_fire_event() {
        let player = DotLottiePlayer::new(Config::default());
        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];
        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );
        let sm = include_str!("fixtures/statemachines/normal_usecases/password_with_events.json");

        player.state_machine_load_data(sm);

        let r = player.state_machine_start(OpenUrlPolicy::default());
        assert!(r);

        player.state_machine_fire_event("Step");
        assert_eq!(player.state_machine_current_state(), "a".to_string());

        player.state_machine_fire_event("Step");
        assert_eq!(player.state_machine_current_state(), "b".to_string());
    }

    #[test]
    fn final_state() {
        let player = DotLottiePlayer::new(Config::default());
        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];
        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );
        let sm = include_str!("fixtures/statemachines/normal_usecases/final_state.json");

        player.state_machine_load_data(sm);

        let r = player.state_machine_start(OpenUrlPolicy::default());
        assert!(r);

        assert_eq!(player.state_machine_current_state(), "star_0".to_string());

        player.state_machine_set_numeric_input("rating", 3.0);
        assert_eq!(player.state_machine_current_state(), "star_3".to_string());

        player.state_machine_set_numeric_input("rating", 5.0);
        assert_eq!(player.state_machine_current_state(), "star_1".to_string());

        player.state_machine_set_numeric_input("rating", 3.0);
        assert_eq!(player.state_machine_current_state(), "star_1".to_string());
    }

    #[test]
    fn state_machine_current_state() {
        let player = DotLottiePlayer::new(Config::default());
        let buffer = vec![0u32; (WIDTH * HEIGHT) as usize];
        player.set_sw_target(
            buffer.as_ptr() as u64,
            WIDTH as u32,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ARGB8888,
        );
        let pointer_down =
            include_str!("fixtures/statemachines/interaction_tests/all_interaction_events.json");

        player.state_machine_load_data(pointer_down);

        let r = player.state_machine_start(OpenUrlPolicy::default());
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
