use dotlottie_rs::{Config, DotLottiePlayer};

mod test_utils;

use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use dotlottie_rs::DotLottieEvent;

    use super::*;

    #[test]
    fn test_subscribe_unsubscribe() {
        let mut events: Vec<String> = vec![];

        let mut player = DotLottiePlayer::new(
            Config {
                autoplay: true,
                loop_animation: true,
                use_frame_interpolation: false,
                ..Config::default()
            },
            0,
        );

        assert!(
            !player.load_animation_path("invalid/path", WIDTH, HEIGHT),
            "Invalid path should not load"
        );

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
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
            if player.set_frame(next_frame) {
                expected_events.push(format!("on_frame: {}", player.current_frame()));
                if player.render() {
                    expected_events.push(format!("on_render: {}", player.current_frame()));
                    if player.is_complete() {
                        if player.config().loop_animation {
                            let loop_count = player.loop_count();
                            expected_events.push(format!("on_loop: {loop_count}"));

                            if loop_count == 1 {
                                player.pause();
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

        player.stop();

        expected_events.push("on_pause".to_string());
        // Stop set_frame to 0.0 before seding stop event
        expected_events.push("on_frame: 0".to_string());
        expected_events.push("on_stop".to_string());

        while let Some(event) = player.poll_event() {
            let event_str = match event {
                DotLottieEvent::Load => {
                    format!("on_load")
                }
                DotLottieEvent::LoadError => {
                    format!("on_load_error")
                }
                DotLottieEvent::Play => {
                    format!("on_play")
                }
                DotLottieEvent::Pause => {
                    format!("on_pause")
                }
                DotLottieEvent::Stop => {
                    format!("on_stop")
                }
                DotLottieEvent::Frame { frame_no } => {
                    format!("on_frame: {}", frame_no)
                }
                DotLottieEvent::Render { frame_no } => {
                    format!("on_render: {}", frame_no)
                }
                DotLottieEvent::Loop { loop_count } => {
                    format!("on_loop: {}", loop_count)
                }
                DotLottieEvent::Complete => {
                    format!("on_complete")
                }
            };

            events.push(event_str.to_string());
        }

        println!("Events: {:?}", events);

        for (i, event) in events.iter().enumerate() {
            assert_eq!(
                event, &expected_events[i],
                "Mismatch at event index {}: expected '{}', found '{}'",
                i, expected_events[i], event
            );
        }

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Valid path should load"
        );

        assert_eq!(
            events.len(),
            expected_events.len(),
            "Events should not change after unsubscribing"
        );
    }
}
