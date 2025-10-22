mod test_utils;

#[cfg(test)]
mod tests {

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::Config;
    use dotlottie_rs::DotLottiePlayer;

    use crate::test_utils::compare_with_snapshot;
    use crate::test_utils::HEIGHT;
    use crate::test_utils::WIDTH;

    // #[test]
    // fn test_writing_snapshot_jpg() {
    //     let player = DotLottiePlayer::new(Config::default());
    //     player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), WIDTH, HEIGHT);
    //     player.set_frame(42.0);
    //     player.render();
    //     let buffer = player.buffer(); // Returns *const u32
    //     let snapshot_path = "./tests/snapshots/frame_42.bin";
    //     assert!(
    //         compare_or_create_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap(),
    //         "Buffer at frame 42 doesn't match snapshot"
    //     );
    //     snapshot_to_png(snapshot_path, "./tests/snapshots/frame_42.jpg").unwrap();
    // }

    // #[test]
    // fn test_frame_rendering_with_diff() {
    //     let player = DotLottiePlayer::new(Config::default());
    //     player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), WIDTH, HEIGHT);

    //     player.set_frame(42.0);
    //     player.render();

    //     let buffer = player.buffer();
    //     let snapshot_path = "./tests/snapshots/frame_42.bin";

    //     match get_buffer_diff(buffer, WIDTH, HEIGHT, snapshot_path) {
    //         Ok(diff) if diff.is_empty() => {
    //             // Test passed
    //         }
    //         Ok(diff) => {
    //             panic!(
    //                 "Found {} pixel differences. First few: {:?}",
    //                 diff.len(),
    //                 &diff[..diff.len().min(5)]
    //             );
    //         }
    //         Err(e) => panic!("Failed to compare: {}", e),
    //     }
    // }

    #[test]
    pub fn scalar_global_input_test() {
        let animation_data =
            include_bytes!("fixtures/global_inputs/test_inputs_ball_scalar.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let inputs_loaded = player.global_inputs_load("inputs");
        let set_theme = player.set_theme("theme");

        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);

        assert_eq!(player.global_inputs_get_scalar("ball"), Some(50.0));

        player.set_frame(30.0);
        player.render();

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/scalar_global_input_snapshot.bin";

        assert!(
            compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap(),
            "Buffer at frame 30 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/scalar_global_input_snapshot.png",
        // )
        // .unwrap();
    }

    #[test]
    pub fn gradient_global_input_test() {
        let animation_data =
            include_bytes!("fixtures/global_inputs/test_inputs_ball_gradient.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let inputs_loaded = player.global_inputs_load("inputs");
        let set_theme = player.set_theme("theme");

        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);

        player.set_frame(30.0);
        player.render();

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/gradient_global_input_snapshot.bin";

        assert!(
            compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap(),
            "Buffer at frame 30 doesn't match snapshot"
        );

        //⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/gradient_global_input_snapshot.png",
        // )
        // .unwrap();
    }

    #[test]
    pub fn color_global_input_test() {
        let animation_data = include_bytes!("fixtures/global_inputs/test_inputs_ball_color.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let inputs_loaded = player.global_inputs_load("inputs");
        let set_theme = player.set_theme("theme");

        assert_eq!(
            player.global_inputs_get_color("ball"),
            Some([0.5, 0.9, 0.2])
        );

        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);

        player.set_frame(30.0);
        player.render();

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/color_global_input_snapshot.bin";

        assert!(
            compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap(),
            "Buffer at frame 30 doesn't match snapshot"
        );

        //⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/color_global_input_snapshot.png",
        // )
        // .unwrap();
    }

    #[test]
    pub fn vector_global_input_test() {
        let animation_data =
            include_bytes!("fixtures/global_inputs/test_inputs_ball_vector.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let inputs_loaded = player.global_inputs_load("inputs");
        let set_theme = player.set_theme("theme");

        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);

        assert_eq!(player.global_inputs_get_vector("ball"), Some([50.0, 50.0]));

        player.set_frame(30.0);
        player.render();

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/vector_global_input_snapshot.bin";

        assert!(
            compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap(),
            "Buffer at frame 30 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/vector_global_input_snapshot.png",
        // )
        // .unwrap();
    }

    #[test]
    pub fn boolean_global_input_test() {
        let animation_data = include_bytes!("fixtures/global_inputs/test_inputs_toggle_sm.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let inputs_loaded = player.global_inputs_load("inputs");
        let sm = player.state_machine_load("toggleButton");
        player.state_machine_start(OpenUrlPolicy::default());

        assert!(load);
        assert!(inputs_loaded);
        assert!(sm);

        assert_eq!(player.global_inputs_get_boolean("OnOffSwitch"), Some(false));

        player.set_frame(0.0);
        player.render();

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(
        //     buffer,
        //     WIDTH,
        //     HEIGHT,
        //     "./tests/snapshots/boolean_global_input_snapshot_before.bin",
        // )
        // .unwrap();
        // snapshot_to_png(
        //     "./tests/snapshots/boolean_global_input_snapshot_before.bin",
        //     "./tests/snapshots/boolean_global_input_snapshot_before.png",
        // )
        // .unwrap();
        let buffer = player.buffer();

        assert!(
            compare_with_snapshot(
                buffer,
                WIDTH,
                HEIGHT,
                "./tests/snapshots/boolean_global_input_snapshot_before.bin"
            )
            .unwrap(),
            "Buffer at frame 0 doesn't match snapshot"
        );

        player.global_inputs_set_boolean("OnOffSwitch", true);
        assert_eq!(player.global_inputs_get_boolean("OnOffSwitch"), Some(true));

        player.set_frame(50.0);
        player.render();

        assert!(
            compare_with_snapshot(
                buffer,
                WIDTH,
                HEIGHT,
                "./tests/snapshots/boolean_global_input_snapshot_after.bin"
            )
            .unwrap(),
            "Buffer at frame 50 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(
        //     buffer,
        //     WIDTH,
        //     HEIGHT,
        //     "./tests/snapshots/boolean_global_input_snapshot_after.bin",
        // )
        // .unwrap();
        // snapshot_to_png(
        //     "./tests/snapshots/boolean_global_input_snapshot_after.bin",
        //     "./tests/snapshots/boolean_global_input_snapshot_after.png",
        // )
        // .unwrap();
    }

    // pub fn boolean_global_input_test() {
    //     let global_state =
    //         include_str!("fixtures/statemachines/interaction_tests/pointer_down_up.json");

    //     let player = DotLottiePlayer::new(Config::default());
    //     player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
    // }
    // pub fn image_global_input_test() {
    //     let global_state =
    //         include_str!("fixtures/statemachines/interaction_tests/pointer_down_up.json");

    //     let player = DotLottiePlayer::new(Config::default());
    //     player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
    // }
    // pub fn text_global_input_test() {
    //     let global_state =
    //         include_str!("fixtures/statemachines/interaction_tests/pointer_down_up.json");

    //     let player = DotLottiePlayer::new(Config::default());
    //     player.load_dotlottie_data(include_bytes!("fixtures/star_marked.lottie"), 100, 100);
    // }
}
