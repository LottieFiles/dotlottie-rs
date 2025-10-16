#[cfg(test)]
mod tests {
    use dotlottie_rs::Config;
    use dotlottie_rs::DotLottiePlayer;

    #[test]
    pub fn scalar_global_input_test() {
        let animation_data = include_str!("fixtures/global_inputs/test_ball_scalar.lottie");
        let data = std::fs::read(animation_data).unwrap();

        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(&data, 100, 100);

        player.set_theme("test_ball_scalar_theme");
        player.global_inputs_load_data("binding_tests_ball_scalar");
    }
    pub fn color_global_input_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_down_up.json");

        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
    }
    pub fn vector_global_input_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_down_up.json");

        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
    }
    pub fn boolean_global_input_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_down_up.json");

        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
    }
    pub fn gradient_global_input_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_down_up.json");

        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
    }
    pub fn image_global_input_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_down_up.json");

        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
    }
    pub fn text_global_input_test() {
        let global_state =
            include_str!("fixtures/statemachines/interaction_tests/pointer_down_up.json");

        let player = DotLottiePlayer::new(Config::default());
        player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
    }
}
