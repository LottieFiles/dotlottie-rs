use std::ffi::CString;

use dotlottie_rs::{ColorSpace, DotLottiePlayer};

mod test_utils;

use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use dotlottie_rs::DotLottieEvent;

    use super::*;

    #[test]
    fn test_subscribe_unsubscribe() {
        let mut events: Vec<String> = vec![];

        let mut player = DotLottiePlayer::new();
        player.set_autoplay(true);
        player.set_loop(true);
        player.set_use_frame_interpolation(false);

        let invalid_path = CString::new("invalid/path").unwrap();
        let valid_path = CString::new("assets/animations/lottie/test.json").unwrap();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(
            player
                .load_animation_path(&invalid_path, WIDTH, HEIGHT)
                .is_err(),
            "Invalid path should not load"
        );

        assert!(
            player
                .load_animation_path(&valid_path, WIDTH, HEIGHT)
                .is_ok(),
            "Valid path should load"
        );

        let mut expected_events = vec![
            "on_load_error".to_string(),
            "on_load".to_string(),
            "on_play".to_string(),
        ];

        // animation loop
        loop {
            let next_frame = player.request_frame();
            if player.set_frame(next_frame).is_ok() {
                expected_events.push(format!("on_frame: {}", player.current_frame()));
                if player.render().is_ok() {
                    expected_events.push(format!("on_render: {}", player.current_frame()));
                    if player.is_complete() {
                        if player.loop_animation() {
                            let loop_count = player.current_loop_count();
                            expected_events.push(format!("on_loop: {loop_count}"));

                            if loop_count == 1 {
                                let _ = player.pause();
                                break;
                            }
                        } else {
                            expected_events.push("on_complete".to_string());
                            break;
                        }
                    }
                }
            }
        }

        let _ = player.stop();

        expected_events.push("on_pause".to_string());
        // Stop set_frame to 0.0 before seding stop event
        expected_events.push("on_frame: 0".to_string());
        expected_events.push("on_stop".to_string());

        while let Some(event) = player.poll_event() {
            let event_str = match event {
                DotLottieEvent::Load => "on_load".to_string(),
                DotLottieEvent::LoadError => "on_load_error".to_string(),
                DotLottieEvent::Play => "on_play".to_string(),
                DotLottieEvent::Pause => "on_pause".to_string(),
                DotLottieEvent::Stop => "on_stop".to_string(),
                DotLottieEvent::Frame { frame_no } => format!("on_frame: {frame_no}"),
                DotLottieEvent::Render { frame_no } => format!("on_render: {frame_no}"),
                DotLottieEvent::Loop { loop_count } => format!("on_loop: {loop_count}"),
                DotLottieEvent::Complete => "on_complete".to_string(),
            };

            events.push(event_str);
        }

        for (i, event) in events.iter().enumerate() {
            assert_eq!(
                event, &expected_events[i],
                "Mismatch at event index {}: expected '{}', found '{}'",
                i, expected_events[i], event
            );
        }

        assert_eq!(
            player.load_animation_path(&valid_path, WIDTH, HEIGHT),
            Ok(()),
            "Valid path should load"
        );

        assert_eq!(
            events.len(),
            expected_events.len(),
            "Events should not change after unsubscribing"
        );
    }
}
