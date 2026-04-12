use std::ffi::CString;

use dotlottie_rs::{ColorSpace, DotLottiePlayer, Segment};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default_marker() {
        let player = DotLottiePlayer::new();

        assert!(
            player.active_marker().is_none(),
            "Expected no marker by default"
        );
    }

    #[test]
    fn test_markers() {
        let mut player = DotLottiePlayer::new();
        player.set_autoplay(true);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        assert!(
            player.markers().is_empty(),
            "Expected no markers before loading animation"
        );

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path).is_ok(),
            "Animation should load"
        );

        let actual_markers = player.markers();

        assert_eq!(actual_markers.len(), 4);

        let expected: &[(&str, f32, f32)] = &[
            ("Marker_1", 0.0, 10.0),
            ("Marker_2", 10.0, 20.0),
            ("Marker_3", 20.0, 30.0),
            ("Marker_4", 30.0, 42.0),
        ];

        for (name, start, end) in expected {
            let marker = actual_markers
                .iter()
                .find(|m| m.name.to_str() == Ok(*name))
                .unwrap_or_else(|| panic!("Marker {name} not found"));

            assert_eq!(marker.segment.start, *start, "start mismatch for {name}");
            assert_eq!(marker.segment.end, *end, "end mismatch for {name}");
        }
    }

    #[test]
    fn test_set_marker() {
        let mut player = DotLottiePlayer::new();
        player.set_autoplay(true);

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path).is_ok(),
            "Animation should load"
        );

        let marker_name = CString::new("Marker_3").unwrap();
        player.set_marker(Some(&marker_name));

        assert_eq!(player.active_marker(), Some(marker_name.as_c_str()));

        assert!(player.is_playing(), "Animation should be playing");

        // assert current frame is the marker start
        assert_eq!(player.current_frame(), 20.0);

        let mut rendered_frames: Vec<f32> = vec![];

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame).is_ok() && player.render().is_ok() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        // assert if all rendered frames are within the marker start..end
        let marker = player
            .markers()
            .iter()
            .find(|m| m.name.to_str() == Ok("Marker_3"))
            .unwrap();

        for frame in rendered_frames {
            assert!(
                frame >= marker.segment.start && frame <= marker.segment.end,
                "Expected frame to be within marker start and end"
            );
        }
    }

    #[test]
    fn test_set_frame_outside_segment_rejected() {
        let mut player = DotLottiePlayer::new();

        let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

        assert!(player
            .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888,)
            .is_ok());

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(player.load_animation_path(&path).is_ok());

        // Set a segment [10, 20]
        assert!(player
            .set_segment(Some(Segment {
                start: 10.0,
                end: 20.0
            }))
            .is_ok());

        // Frame within segment should succeed
        assert!(player.set_frame(15.0).is_ok());

        // Frame outside segment should fail
        assert!(player.set_frame(5.0).is_err());
        assert!(player.set_frame(25.0).is_err());
    }
}
