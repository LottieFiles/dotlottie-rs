use std::ffi::CString;

use dotlottie_rs::{ColorSpace, Player};

mod test_utils;

use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use dotlottie_rs::PlayerEvent;

    use super::*;

    #[test]
    fn test_subscribe_unsubscribe() {
        let mut events: Vec<String> = vec![];

        let mut player = Player::new();
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
            player.load_animation_path(&invalid_path).is_err(),
            "Invalid path should not load"
        );

        assert!(
            player.load_animation_path(&valid_path).is_ok(),
            "Valid path should load"
        );

        let mut expected_events = vec![
            "on_load_error".to_string(),
            "on_load".to_string(),
            "on_play".to_string(),
        ];

        let drain = |player: &mut Player, events: &mut Vec<String>| {
            while let Some(event) = player.poll_event() {
                events.push(match event {
                    PlayerEvent::Load => "on_load".to_string(),
                    PlayerEvent::LoadError => "on_load_error".to_string(),
                    PlayerEvent::Play => "on_play".to_string(),
                    PlayerEvent::Pause => "on_pause".to_string(),
                    PlayerEvent::Stop => "on_stop".to_string(),
                    PlayerEvent::Frame { frame_no } => format!("on_frame: {frame_no}"),
                    PlayerEvent::Render { frame_no } => format!("on_render: {frame_no}"),
                    PlayerEvent::Loop { loop_count } => format!("on_loop: {loop_count}"),
                    PlayerEvent::Complete => "on_complete".to_string(),
                });
            }
        };

        // animation loop
        loop {
            let rendered = player.tick(1000.0 / 60.0).unwrap_or(false);
            if rendered {
                expected_events.push(format!("on_frame: {}", player.current_frame()));
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
            drain(&mut player, &mut events);
        }

        let _ = player.stop();

        expected_events.push("on_pause".to_string());
        // Stop set_frame to 0.0 before seding stop event
        expected_events.push("on_frame: 0".to_string());
        expected_events.push("on_stop".to_string());

        drain(&mut player, &mut events);

        for (i, event) in events.iter().enumerate() {
            assert_eq!(
                event, &expected_events[i],
                "Mismatch at event index {}: expected '{}', found '{}'",
                i, expected_events[i], event
            );
        }

        assert_eq!(
            player.load_animation_path(&valid_path),
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
