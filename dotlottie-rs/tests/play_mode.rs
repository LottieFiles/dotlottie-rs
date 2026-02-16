use std::ffi::CString;

use dotlottie_rs::{ColorSpace, DotLottiePlayer};

mod test_utils;

use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod play_mode_tests {
    use dotlottie_rs::{DotLottieEvent, Mode};

    use super::*;

    #[test]
    fn test_default_play_mode() {
        let player = DotLottiePlayer::new(0);

        assert_eq!(player.mode(), Mode::Forward);
    }

    #[test]
    fn test_loop_count_with_loop_animation_false() {
        let mut player = DotLottiePlayer::new(0);
        player.set_mode(Mode::Forward);
        player.set_autoplay(true);
        player.set_loop(false);
        player.set_loop_count(3);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let mut observed_loops = 0;
        let mut observed_completed = false;

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        loop {
            if player.is_paused() || player.is_stopped() || player.current_loop_count() > 5 {
                break;
            }
            let _ = player.tick();
        }

        while let Some(event) = player.poll_event() {
            match event {
                DotLottieEvent::Loop { loop_count } => {
                    observed_loops = loop_count;
                }
                DotLottieEvent::Complete => {
                    observed_completed = true;
                }
                _ => {}
            }
        }

        assert_eq!(observed_loops, 0, "Should not have looped");

        assert!(observed_completed);
        assert!(player.is_complete());
    }

    #[test]
    fn test_zero_loop_count() {
        let mut player = DotLottiePlayer::new(0);
        player.set_mode(Mode::Forward);
        player.set_autoplay(true);
        player.set_loop(true);
        player.set_loop_count(0);
        player.set_use_frame_interpolation(false);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let mut observed_loops = 0;

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        loop {
            if player.is_paused() || player.is_stopped() || player.current_loop_count() > 5 {
                break;
            }
            let _ = player.tick();
        }

        while let Some(event) = player.poll_event() {
            if let DotLottieEvent::Loop { loop_count } = event {
                observed_loops = loop_count;
            }
        }

        assert_eq!(observed_loops, 6, "Will loop and ignore loop count");

        // Same behaviour before refactor. I think is_complete should be false but its true
        assert!(player.is_complete());
    }

    #[test]
    fn test_playing_after_loop_has_completed() {
        let mut player = DotLottiePlayer::new(0);
        player.set_mode(Mode::Forward);
        player.set_autoplay(true);
        player.set_loop(true);
        player.set_loop_count(3);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let mut observed_loops = 0;
        let mut observed_completed = false;

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        loop {
            if player.is_paused() || player.is_stopped() || player.current_loop_count() > 5 {
                break;
            }
            let _ = player.tick();
        }

        while let Some(event) = player.poll_event() {
            match event {
                DotLottieEvent::Loop { loop_count } => {
                    observed_loops = loop_count;
                }
                DotLottieEvent::Complete => {
                    observed_completed = true;
                }
                _ => {}
            }
        }

        assert_eq!(
            observed_loops, 3,
            "Should have looped 3 times, got {observed_loops}"
        );
        assert!(observed_completed);

        // Restart the player
        let _ = player.play();
        observed_completed = false;

        loop {
            if player.is_paused() || player.is_stopped() || player.current_loop_count() > 5 {
                break;
            }
            let _ = player.tick();
        }

        while let Some(event) = player.poll_event() {
            match event {
                DotLottieEvent::Loop { loop_count } => {
                    observed_loops = loop_count;
                }
                DotLottieEvent::Complete => {
                    observed_completed = true;
                }
                _ => {}
            }
        }

        // loop count resets on complete
        assert_eq!(
            observed_loops, 3,
            "Should have looped 3 times, got {observed_loops}"
        );
        assert!(observed_completed);
    }

    #[test]
    fn test_loop_count_paused_mid_play() {
        let mut player = DotLottiePlayer::new(0);
        player.set_mode(Mode::Forward);
        player.set_autoplay(true);
        player.set_loop(true);
        player.set_loop_count(5);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let mut observed_loops = 0;
        let mut observed_completed = false;

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        loop {
            if player.is_paused() || player.is_stopped() || player.current_loop_count() >= 3 {
                break;
            }
            let _ = player.tick();
        }
        while let Some(event) = player.poll_event() {
            match event {
                DotLottieEvent::Loop { loop_count } => {
                    observed_loops = loop_count;
                }
                DotLottieEvent::Complete => {
                    observed_completed = true;
                }
                _ => {}
            }
        }

        assert_eq!(
            observed_loops, 3,
            "Should have looped 3 times, got {observed_loops}"
        );
        assert!(!observed_completed);

        // Restart the player
        let _ = player.play();

        loop {
            if player.is_paused() || player.is_stopped() || player.current_loop_count() > 10 {
                break;
            }
            let _ = player.tick();
        }

        while let Some(event) = player.poll_event() {
            match event {
                DotLottieEvent::Loop { loop_count } => {
                    observed_loops = loop_count;
                }
                DotLottieEvent::Complete => {
                    observed_completed = true;
                }
                _ => {}
            }
        }

        assert_eq!(
            observed_loops, 5,
            "Should have looped 5 times, got {observed_loops}"
        );

        assert!(observed_completed);
    }

    #[test]
    fn test_set_play_mode() {
        let play_modes = vec![
            Mode::Forward,
            Mode::Reverse,
            Mode::Bounce,
            Mode::ReverseBounce,
        ];

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        for mode in play_modes {
            let mut player = DotLottiePlayer::new(0);
            player.set_mode(mode);

            let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

            // Set software rendering target
            assert!(player
                .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
                .is_ok());

            assert_eq!(
                player.mode(),
                mode,
                "Expected play mode to be {:?}, found {:?}",
                mode,
                player.mode()
            );

            assert!(
                player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
                "Animation should load"
            );

            match mode {
                Mode::Forward => {
                    assert_eq!(
                        player.current_frame(),
                        0.0,
                        "Expected current frame to be 0"
                    );
                }
                Mode::Reverse => {
                    assert_eq!(
                        player.current_frame(),
                        player.total_frames() - 1.0,
                        "Expected current frame to be end frame"
                    );
                }
                Mode::Bounce => {
                    assert_eq!(
                        player.current_frame(),
                        0.0,
                        "Expected current frame to be 0"
                    );
                }
                Mode::ReverseBounce => {
                    assert_eq!(
                        player.current_frame(),
                        player.total_frames() - 1.0,
                        "Expected current frame to be end frame"
                    );
                }
            }
        }
    }

    #[test]
    fn test_forward_play_mode() {
        let mut player = DotLottiePlayer::new(0);
        player.set_mode(Mode::Forward);
        player.set_autoplay(true);

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        let mut rendered_frames: Vec<f32> = vec![];

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame).is_ok() && player.render().is_ok() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(
            rendered_frames.len() >= (player.total_frames() - 1.0) as usize,
            "Expected rendered frames to be greater than or equal to total frames"
        );

        let mut prev_frame = 0.0;
        for frame in rendered_frames {
            assert!(
                frame >= prev_frame,
                "Expected frame to be greater than or equal to previous frame"
            );
            prev_frame = frame;
        }

        assert_eq!(
            player.total_frames() - 1.0,
            prev_frame,
            "Expected last frame to be total frames"
        );
    }

    #[test]
    fn test_forward_play_mode_with_loop_count() {
        let mut player = DotLottiePlayer::new(0);
        player.set_mode(Mode::Forward);
        player.set_autoplay(true);
        player.set_loop(true);
        player.set_loop_count(3);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let mut observed_loops = 0;
        let mut observed_completed = false;

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        loop {
            if player.is_paused() || player.is_stopped() || player.current_loop_count() > 5 {
                break;
            }
            let _ = player.tick();
        }

        while let Some(event) = player.poll_event() {
            match event {
                DotLottieEvent::Loop { loop_count } => {
                    observed_loops = loop_count;
                }
                DotLottieEvent::Complete => {
                    observed_completed = true;
                }
                _ => {}
            }
        }

        assert_eq!(
            observed_loops, 3,
            "Should have looped 3 times, got {observed_loops}"
        );

        assert!(observed_completed);
    }

    #[test]
    fn test_reverse_play_mode() {
        let mut player = DotLottiePlayer::new(0);
        player.set_mode(Mode::Reverse);
        player.set_autoplay(true);

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
            "Animation should load"
        );

        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        let mut rendered_frames: Vec<f32> = vec![];

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame).is_ok() && player.render().is_ok() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(
            rendered_frames.len() >= (player.total_frames() - 1.0) as usize,
            "Expected rendered frames to be greater than or equal to total frames"
        );

        let mut prev_frame = player.total_frames() - 1.0;
        for frame in rendered_frames {
            assert!(
                frame <= prev_frame,
                "Expected frame to be less than or equal to previous frame"
            );
            prev_frame = frame;
        }

        // check if the last frame is 0
        assert_eq!(0.0, prev_frame, "Expected last frame to be 0");
    }

    #[test]
    fn test_reverse_play_mode_with_loop_count() {
        let mut player = DotLottiePlayer::new(0);
        player.set_mode(Mode::Reverse);
        player.set_autoplay(true);
        player.set_loop(true);
        player.set_loop_count(3);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let mut observed_loops = 0;
        let mut observed_completed = false;

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        loop {
            if player.is_paused() || player.is_stopped() || player.current_loop_count() > 5 {
                break;
            }
            let _ = player.tick();
        }

        while let Some(event) = player.poll_event() {
            match event {
                DotLottieEvent::Loop { loop_count } => {
                    observed_loops = loop_count;
                }
                DotLottieEvent::Complete => {
                    observed_completed = true;
                }
                _ => {}
            }
        }

        assert_eq!(
            observed_loops, 3,
            "Should have looped 3 times, got {observed_loops}"
        );

        assert!(observed_completed);
    }

    #[test]
    fn test_bounce_play_mode() {
        let mut player = DotLottiePlayer::new(0);
        player.set_mode(Mode::Bounce);
        player.set_autoplay(true);

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert_eq!(
            player.load_animation_path(&path, WIDTH, HEIGHT),
            Ok(()),
            "Animation should load"
        );

        let mut rendered_frames: Vec<f32> = vec![];

        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame).is_ok() && player.render().is_ok() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(
            rendered_frames.len() >= (player.total_frames() - 1.0) as usize,
            "Expected rendered frames to be greater than or equal to total frames"
        );

        let mut frame_idx = 0;

        while frame_idx < rendered_frames.len()
            && rendered_frames[frame_idx] < player.total_frames() - 1.0
        {
            assert!(
                rendered_frames[frame_idx] <= rendered_frames[frame_idx + 1],
                "Expected frame to be less than or equal to next frame"
            );
            frame_idx += 1;
        }

        assert!(
            rendered_frames[frame_idx] == player.total_frames() - 1.0,
            "Expected frame to be total frames at index {frame_idx}"
        );

        while frame_idx < rendered_frames.len() && rendered_frames[frame_idx] > 0.0 {
            assert!(
                rendered_frames[frame_idx] >= rendered_frames[frame_idx + 1],
                "Expected frame to be greater than or equal to next frame"
            );
            frame_idx += 1;
        }

        assert!(
            rendered_frames[frame_idx] == 0.0,
            "Expected frame to be 0 at index {frame_idx}"
        );
    }

    #[test]
    fn test_bounce_play_mode_with_loop_count() {
        let mut player = DotLottiePlayer::new(0);
        player.set_mode(Mode::Bounce);
        player.set_autoplay(true);
        player.set_loop(true);
        player.set_loop_count(3);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let mut observed_loops = 0;
        let mut observed_completed = false;

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert_eq!(
            player.load_animation_path(&path, WIDTH, HEIGHT),
            Ok(()),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");

        loop {
            if player.is_paused() || player.is_stopped() || player.current_loop_count() > 5 {
                break;
            }
            let _ = player.tick();
        }
        while let Some(event) = player.poll_event() {
            match event {
                DotLottieEvent::Loop { loop_count } => {
                    observed_loops = loop_count;
                }
                DotLottieEvent::Complete => {
                    observed_completed = true;
                }
                _ => {}
            }
        }

        assert_eq!(
            observed_loops, 3,
            "Should have looped 3 times, got {observed_loops}"
        );

        assert!(observed_completed);
    }

    #[test]
    fn test_reverse_bounce_play_mode() {
        let mut player = DotLottiePlayer::new(0);
        player.set_mode(Mode::ReverseBounce);
        player.set_autoplay(true);

        let path = CString::new("assets/animations/lottie/test.json").unwrap();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
            "Animation should load"
        );

        assert!(player.is_playing(), "Animation should be playing");

        let mut rendered_frames: Vec<f32> = vec![];

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame).is_ok() && player.render().is_ok() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(
            rendered_frames.len() >= (player.total_frames() - 1.0) as usize,
            "Expected rendered frames to be greater than or equal to total frames"
        );

        let mut frame_idx = 0;

        while frame_idx < rendered_frames.len() && rendered_frames[frame_idx] > 0.0 {
            assert!(
                rendered_frames[frame_idx] >= rendered_frames[frame_idx + 1],
                "Expected frame to be greater than or equal to next frame"
            );
            frame_idx += 1;
        }

        assert!(
            rendered_frames[frame_idx] == 0.0,
            "Expected frame to be 0 at index {frame_idx}"
        );

        while frame_idx < rendered_frames.len()
            && rendered_frames[frame_idx] < player.total_frames() - 1.0
        {
            assert!(
                rendered_frames[frame_idx] <= rendered_frames[frame_idx + 1],
                "Expected frame to be less than or equal to next frame"
            );
            frame_idx += 1;
        }

        assert!(
            rendered_frames[frame_idx] == player.total_frames() - 1.0,
            "Expected frame to be total frames at index {frame_idx}"
        );
    }

    #[test]
    fn test_reverse_bounce_play_mode_with_loop_count() {
        let mut player = DotLottiePlayer::new(0);
        player.set_mode(Mode::ReverseBounce);
        player.set_autoplay(true);
        player.set_loop(true);
        player.set_loop_count(3);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let mut observed_loops = 0;
        let mut observed_completed = false;

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path, WIDTH, HEIGHT).is_ok(),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");

        loop {
            if player.is_paused() || player.is_stopped() || player.current_loop_count() > 5 {
                break;
            }
            let _ = player.tick();
        }
        while let Some(event) = player.poll_event() {
            match event {
                DotLottieEvent::Loop { loop_count } => {
                    observed_loops = loop_count;
                }
                DotLottieEvent::Complete => {
                    observed_completed = true;
                }
                _ => {}
            }
        }

        assert_eq!(
            observed_loops, 3,
            "Should have looped 3 times, got {observed_loops}"
        );

        assert!(observed_completed);
    }
}
