mod test_utils;

#[cfg(test)]
mod tests {

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::Config;
    use dotlottie_rs::DotLottiePlayer;

    use crate::test_utils::compare_with_snapshot;
    use crate::test_utils::snapshot_to_png;
    use crate::test_utils::write_buffer_snapshot;
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
        // Description:
        // SID is on the opacity of the ball's fill
        // The data binding affects the transparency of the fill

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
        // Description:
        // SID is on the content of the gradient
        // The data binding affects the colors of the gradient
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
        // Description:
        // SID is on the fill color
        // The data binding affects which color is used
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
        // Description:
        // SID is on the position of the wand
        // The data binding affects it's position, taking it from the center to the top left corner
        let animation_data =
            include_bytes!("fixtures/global_inputs/test_inputs_wand_vector.lottie");

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
        // Description:
        // The boolean data binding is used inside the toggle state machine
        // Changing the value of the data binding makes the toggle state machine go from day to night
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

    #[test]
    pub fn image_global_input_test() {
        // Description:
        // The image data binding is used to replace an image with another inside the dotLottie
        let animation_data = include_bytes!("fixtures/global_inputs/test_inputs_bull_image.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let inputs_loaded = player.global_inputs_load("inputs");

        let set_theme = player.set_theme("theme");

        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);

        player.set_frame(30.0);
        player.render();

        assert_eq!(
            player.global_inputs_get_image("image").unwrap().id,
            Some("test".to_string())
        );

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/image_global_input_snapshot_after.bin";

        assert!(
            compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap(),
            "Buffer at frame 30 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/image_global_input_snapshot_after.png",
        // )
        // .unwrap();
    }

    #[test]
    pub fn text_global_input_test() {
        // Description:
        // The image data binding is used to replace an image with another inside the dotLottie
        let animation_data = include_bytes!("fixtures/global_inputs/test_inputs_text.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let inputs_loaded = player.global_inputs_load("inputs");

        let set_theme = player.set_theme("theme");

        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);

        player.render();

        assert_eq!(
            player.global_inputs_get_text("text_input"),
            Some("First Try!".to_string())
        );

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/text_global_input_snapshot_before.bin";

        assert!(
            compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap(),
            "Buffer at frame 30 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/text_global_input_snapshot_before.png",
        // )
        // .unwrap();

        player.global_inputs_set_text("text_input", "New Value");
        assert_eq!(
            player.global_inputs_get_text("text_input"),
            Some("New Value".to_string())
        );

        player.render();

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/text_global_input_snapshot_after.bin";

        assert!(
            compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap(),
            "Buffer at frame 30 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/text_global_input_snapshot_after.png",
        // )
        // .unwrap();
    }
}
