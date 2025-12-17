mod test_utils;

#[cfg(test)]
mod tests {

    use dotlottie_rs::actions::open_url_policy::OpenUrlPolicy;
    use dotlottie_rs::Config;
    use dotlottie_rs::DotLottiePlayer;
    use dotlottie_rs::GradientStop;

    use crate::test_utils::compare_with_snapshot;
    // use crate::test_utils::snapshot_to_png;
    // use crate::test_utils::write_buffer_snapshot;
    use crate::test_utils::HEIGHT;
    use crate::test_utils::WIDTH;

    #[test]
    pub fn test_getters() {
        // Description:
        // Test fetching the current value of the global inputs

        let animation_data = include_bytes!("fixtures/global_inputs/all_global_inputs.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let set_theme = player.set_theme("theme");
        let inputs_loaded = player.global_inputs_load("big_inputs_file");
        let inputs_apply = player.global_inputs_start();

        assert!(set_theme);
        assert!(inputs_apply);
        assert!(load);
        assert!(inputs_loaded);

        assert_eq!(
            player.global_inputs_get_numeric("numeric_static"),
            Some(50.0)
        );
        assert_eq!(
            player.global_inputs_get_numeric("numeric_animated"),
            Some(100.0)
        );

        assert_eq!(
            player.global_inputs_get_color("color_static"),
            (vec![0.9, 0.9, 0.9, 1.0])
        );

        assert_eq!(
            player.global_inputs_get_color("color_animated"),
            (vec![0.1, 0.1, 0.1, 1.0])
        );
        assert_eq!(
            player.global_inputs_get_gradient("gradient_static"),
            (vec![
                GradientStop {
                    color: vec![0.0, 0.0, 0.0, 1.0],
                    offset: 0.0
                },
                GradientStop {
                    color: vec![1.0, 1.0, 1.0, 1.0],
                    offset: 1.0
                }
            ])
        );

        assert_eq!(
            player.global_inputs_get_gradient("gradient_animated"),
            ([GradientStop {
                color: vec![0.1, 0.1, 0.1, 1.0],
                offset: 0.1
            }])
        );
        assert_eq!(
            player.global_inputs_get_string("string_static"),
            Some("First Try!".to_string())
        );
        assert_eq!(
            player.global_inputs_get_string("string_animated"),
            Some("START REPLACED WITH BINDING".to_string())
        );
        assert_eq!(
            player.global_inputs_get_boolean("boolean_static"),
            Some(false)
        );
        assert_eq!(
            player.global_inputs_get_vector("vector_static"),
            (vec![50.0, 50.0])
        );
    }

    #[test]
    pub fn test_setters() {
        // Description:
        // Test setting values for global inputs and verifying they update correctly

        let animation_data = include_bytes!("fixtures/global_inputs/all_global_inputs.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let set_theme = player.set_theme("theme");
        let inputs_loaded = player.global_inputs_load("big_inputs_file");

        // We don't start the engine, because this is just a test file with placeholder paths, changing binding values that don't have valid paths will fail

        assert!(load);
        assert!(set_theme);
        assert!(inputs_loaded);

        assert!(player.global_inputs_set_numeric("numeric_static", 123.0));
        assert_eq!(
            player.global_inputs_get_numeric("numeric_static"),
            Some(123.0)
        );

        assert!(player.global_inputs_set_color("color_static", &vec![0.5, 0.6, 0.7, 0.8]));
        assert_eq!(
            player.global_inputs_get_color("color_static"),
            vec![0.5, 0.6, 0.7, 0.8]
        );

        let new_gradient = vec![
            GradientStop {
                color: vec![1.0, 0.0, 0.0, 1.0],
                offset: 0.0,
            },
            GradientStop {
                color: vec![0.0, 0.0, 1.0, 1.0],
                offset: 0.5,
            },
            GradientStop {
                color: vec![0.0, 1.0, 0.0, 1.0],
                offset: 1.0,
            },
        ];
        assert!(player.global_inputs_set_gradient("gradient_static", &new_gradient));
        assert_eq!(
            player.global_inputs_get_gradient("gradient_static"),
            new_gradient
        );

        // Test string setter
        assert!(player.global_inputs_set_string("string_static", "Updated String!"));
        assert_eq!(
            player.global_inputs_get_string("string_static"),
            Some("Updated String!".to_string())
        );

        // Test boolean setter
        assert!(player.global_inputs_set_boolean("boolean_static", true));
        assert_eq!(
            player.global_inputs_get_boolean("boolean_static"),
            Some(true)
        );

        // Test vector setter
        assert!(player.global_inputs_set_vector("vector_static", &[100.0, 200.0]));
        assert_eq!(
            player.global_inputs_get_vector("vector_static"),
            vec![100.0, 200.0]
        );

        // Test setting non-existent inputs returns false
        assert!(!player.global_inputs_set_numeric("non_existent", 999.0));
        assert!(!player.global_inputs_set_color("non_existent", &vec![1.0, 1.0, 1.0, 1.0]));
        assert!(!player.global_inputs_set_string("non_existent", "test"));
        assert!(!player.global_inputs_set_boolean("non_existent", true));
        assert!(!player.global_inputs_set_vector("non_existent", &[0.0, 0.0]));
        assert!(!player.global_inputs_set_gradient("non_existent", &vec![]));
    }

    #[test]
    pub fn numeric_static_global_input_test() {
        // Description:
        // SID is on the opacity of the ball's fill
        // The data binding affects the transparency of the fill

        let animation_data =
            include_bytes!("fixtures/global_inputs/test_inputs_ball_numeric_static.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let set_theme = player.set_theme("theme");
        let inputs_loaded = player.global_inputs_load("inputs");
        let inputs_apply = player.global_inputs_start();

        assert!(inputs_apply);
        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);

        assert_eq!(player.global_inputs_get_numeric("ball"), Some(50.0));

        player.set_frame(30.0);
        player.render();

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/numeric_global_input_static_snapshot.bin";

        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
            "Buffer at frame 30 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/numeric_global_input_static_snapshot.png",
        // )
        // .unwrap();
    }

    #[test]
    pub fn numeric_animated_global_input_test() {
        // Description:
        // SID is on the opacity of the ball's fill
        // The data binding affects the transparency of the fill

        let animation_data =
            include_bytes!("fixtures/global_inputs/test_inputs_ball_numeric_animated.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let set_theme = player.set_theme("theme");
        let inputs_loaded = player.global_inputs_load("inputs");
        let inputs_apply = player.global_inputs_start();

        assert!(inputs_apply);
        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);

        assert_eq!(player.global_inputs_get_numeric("ball"), Some(100.0));
        assert_eq!(player.global_inputs_get_numeric("ball_end"), Some(10.0));

        player.set_frame(20.0);
        player.render();

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/numeric_global_input_animated_snapshot.bin";

        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
            "Buffer at frame 20 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/numeric_global_input_animated_snapshot.png",
        // )
        // .unwrap();
    }

    #[test]
    pub fn gradient_static_global_input_test() {
        // Description:
        // SID is on the content of the gradient
        // The data binding affects the colors of the gradient
        let animation_data =
            include_bytes!("fixtures/global_inputs/test_inputs_sheet_gradient_static.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let set_theme = player.set_theme("theme");
        let inputs_loaded = player.global_inputs_load("inputs");
        let inputs_apply = player.global_inputs_start();

        assert!(inputs_apply);
        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);

        player.set_frame(30.0);
        player.render();

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/gradient_global_input_static_snapshot.bin";

        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
            "Buffer at frame 30 doesn't match snapshot"
        );

        //⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/gradient_global_input_static_snapshot.png",
        // )
        // .unwrap();
    }

    #[test]
    pub fn gradient_animated_global_input_test() {
        // Description:
        // SID is on the content of the gradient
        // The data binding affects the colors of the gradient
        let animation_data =
            include_bytes!("fixtures/global_inputs/test_inputs_sheet_gradient_animated.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let set_theme = player.set_theme("theme");
        let inputs_loaded = player.global_inputs_load("inputs");
        let inputs_apply = player.global_inputs_start();

        assert!(load);
        assert!(inputs_apply);
        assert!(inputs_loaded);
        assert!(set_theme);

        assert!(player.set_frame(50.0));
        assert!(player.render());

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/gradient_global_input_animated_snapshot.bin";

        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
            "Buffer at frame 50 doesn't match snapshot"
        );

        //⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/gradient_global_input_animated_snapshot.png",
        // )
        // .unwrap();
    }

    #[test]
    pub fn color_static_global_input_test() {
        // Description:
        // SID is on the gradient color
        // The data binding is a Color that replaces the color of a Gradient
        // todo: Test Color -> Color
        let animation_data =
            include_bytes!("fixtures/global_inputs/test_inputs_sheet_color_static.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let set_theme = player.set_theme("theme");
        let inputs_loaded = player.global_inputs_load("inputs");
        let inputs_apply = player.global_inputs_start();

        assert!(inputs_apply);
        assert_eq!(
            player.global_inputs_get_color("start_0"),
            ([0.9, 0.9, 0.9, 1.0])
        );
        assert_eq!(
            player.global_inputs_get_color("end_0"),
            ([0.1, 0.1, 0.1, 1.0])
        );

        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);

        player.set_frame(30.0);
        player.render();

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/color_global_input_static_snapshot.bin";

        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
            "Buffer at frame 30 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/color_global_input_static_snapshot.png",
        // )
        // .unwrap();
    }

    #[test]
    pub fn color_animated_global_input_test() {
        // Description:
        // SID is on the gradient color
        // The data binding is a Color that replaces the color of a Gradient
        // todo: Test Color -> Color
        let animation_data =
            include_bytes!("fixtures/global_inputs/test_inputs_sheet_color_animated.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let set_theme = player.set_theme("theme");
        let inputs_loaded = player.global_inputs_load("inputs");
        let inputs_apply = player.global_inputs_start();

        assert!(inputs_apply);
        assert_eq!(
            player.global_inputs_get_color("start_0"),
            ([0.1, 0.1, 0.1, 1.0].to_vec())
        );
        assert_eq!(
            player.global_inputs_get_color("end_0"),
            ([0.2, 0.2, 0.2, 1.0].to_vec())
        );
        assert_eq!(
            player.global_inputs_get_color("start_1"),
            ([0.3, 0.3, 0.3, 1.0].to_vec())
        );
        assert_eq!(
            player.global_inputs_get_color("end_1"),
            ([0.4, 0.4, 0.4, 1.0].to_vec())
        );
        assert_eq!(
            player.global_inputs_get_color("start_2"),
            ([0.4, 0.4, 0.4, 1.0].to_vec())
        );
        assert_eq!(
            player.global_inputs_get_color("end_2"),
            ([0.5, 0.5, 0.5, 1.0].to_vec())
        );

        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);

        assert!(player.set_frame(30.0));
        assert!(player.render());

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/color_global_input_animated_snapshot.bin";

        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
            "Buffer at frame 30 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/color_global_input_animated_snapshot.png",
        // )
        // .unwrap();
    }

    #[test]
    pub fn vector_global_input_test() {
        // Description:
        // SID is on the position of the wand
        // The data binding affects it's position, taking it from the center to the top left corner
        let animation_data =
            include_bytes!("fixtures/global_inputs/test_vector_global_input.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let set_theme = player.set_theme("theme");
        let inputs_loaded = player.global_inputs_load("inputs");
        let inputs_apply = player.global_inputs_start();

        assert!(inputs_apply);
        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);

        assert_eq!(
            player.global_inputs_get_vector("wand_pos"),
            ([50.0, 50.0].to_vec())
        );

        player.set_frame(30.0);
        player.render();

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/vector_global_input_snapshot.bin";

        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
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
        let inputs_apply = player.global_inputs_start();

        assert!(inputs_apply);
        assert!(load);
        assert!(inputs_loaded);
        assert!(sm);

        assert_eq!(
            player.global_inputs_get_boolean("toggle_global_input"),
            Some(false)
        );

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
            unsafe {
                compare_with_snapshot(
                    buffer,
                    WIDTH,
                    HEIGHT,
                    "./tests/snapshots/boolean_global_input_snapshot_before.bin",
                )
                .unwrap()
            },
            "Buffer at frame 0 doesn't match snapshot"
        );

        player.global_inputs_set_boolean("toggle_global_input", true);
        assert_eq!(
            player.global_inputs_get_boolean("toggle_global_input"),
            Some(true)
        );

        player.set_frame(50.0);
        player.render();

        assert!(
            unsafe {
                compare_with_snapshot(
                    buffer,
                    WIDTH,
                    HEIGHT,
                    "./tests/snapshots/boolean_global_input_snapshot_after.bin",
                )
                .unwrap()
            },
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
    pub fn string_global_input_test_static() {
        // Description:
        let animation_data =
            include_bytes!("fixtures/global_inputs/test_inputs_text_static.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let inputs_loaded = player.global_inputs_load("inputs");
        let set_theme = player.set_theme("theme");
        let apply = player.global_inputs_start();

        assert!(apply);
        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);
        assert_eq!(
            player.global_inputs_get_string("text"),
            Some("First Try!".to_string())
        );

        player.set_frame(30.0);
        player.render();

        let snapshot_path = "./tests/snapshots/text_global_input_static_snapshot_before.bin";
        let buffer = player.buffer();

        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
            "Buffer at frame 30 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/text_global_input_static_snapshot_before.png",
        // )
        // .unwrap();

        player.global_inputs_set_string("text", "New Value");
        assert_eq!(
            player.global_inputs_get_string("text"),
            Some("New Value".to_string())
        );

        player.set_frame(10.0);
        player.render();

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/text_global_input_static_snapshot_after.bin";

        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
            "Buffer at frame 30 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/text_global_input_static_snapshot_after.png",
        // )
        // .unwrap();
    }

    #[test]
    pub fn string_global_input_test_animated() {
        // Description:
        let animation_data =
            include_bytes!("fixtures/global_inputs/test_inputs_text_animated.lottie");

        let player = DotLottiePlayer::new(Config::default());
        let load = player.load_dotlottie_data(animation_data, WIDTH, HEIGHT);

        let inputs_loaded = player.global_inputs_load("inputs");
        let set_theme = player.set_theme("theme");
        let apply = player.global_inputs_start();

        assert!(apply);
        assert!(load);
        assert!(inputs_loaded);
        assert!(set_theme);
        assert_eq!(
            player.global_inputs_get_string("text"),
            Some("START REPLACED WITH BINDING".to_string())
        );
        assert_eq!(
            player.global_inputs_get_string("second_text"),
            Some("END REPLACED WITH BINDING".to_string())
        );

        player.set_frame(0.0);
        player.render();

        let snapshot_path =
            "./tests/snapshots/text_global_input_animated_snapshot_start_before.bin";
        let buffer = player.buffer();

        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
            "Buffer at frame 0 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/text_global_input_animated_snapshot_start_before.png",
        // )
        // .unwrap();

        player.set_frame(40.0);
        player.render();

        let snapshot_path = "./tests/snapshots/text_global_input_animated_snapshot_end_before.bin";
        let buffer = player.buffer();
        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
            "Buffer at frame 0 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/text_global_input_animated_snapshot_end_before.png",
        // )
        // .unwrap();

        player.global_inputs_set_string("text", "New Start");
        assert_eq!(
            player.global_inputs_get_string("text"),
            Some("New Start".to_string())
        );
        player.global_inputs_set_string("second_text", "New End");
        assert_eq!(
            player.global_inputs_get_string("second_text"),
            Some("New End".to_string())
        );

        player.set_frame(10.0);
        player.render();

        let buffer = player.buffer();
        let snapshot_path = "./tests/snapshots/text_global_input_animated_snapshot_after.bin";

        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
            "Buffer at frame 20 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/text_global_input_animated_snapshot_after.png",
        // )
        // .unwrap();

        player.set_frame(40.0);
        player.render();

        let snapshot_path = "./tests/snapshots/text_global_input_animated_snapshot_end_after.bin";
        let buffer = player.buffer();

        assert!(
            unsafe { compare_with_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap() },
            "Buffer at frame 20 doesn't match snapshot"
        );

        // ⚠️ Uncomment block to generate initial snapshot
        // let buffer = player.buffer();
        // write_buffer_snapshot(buffer, WIDTH, HEIGHT, snapshot_path).unwrap();
        // snapshot_to_png(
        //     snapshot_path,
        //     "./tests/snapshots/text_global_input_animated_snapshot_end_after.png",
        // )
        // .unwrap();
    }
}
