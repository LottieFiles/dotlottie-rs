use std::ffi::CString;

use dotlottie_rs::{ColorSpace, DotLottiePlayer, Marker};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default_marker() {
        let player = DotLottiePlayer::new();

        assert!(player.marker().is_none(), "Expected no marker by default");
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

        let expected_markers = [
            Marker {
                name: "Marker_1".to_string(),
                time: 0.0,
                duration: 10.0,
            },
            Marker {
                name: "Marker_2".to_string(),
                time: 10.0,
                duration: 10.0,
            },
            Marker {
                name: "Marker_3".to_string(),
                time: 20.0,
                duration: 10.0,
            },
            Marker {
                name: "Marker_4".to_string(),
                time: 30.0,
                duration: 12.0,
            },
        ];

        for marker in actual_markers {
            let expected = expected_markers
                .iter()
                .find(|m| m.name == marker.name)
                .unwrap();

            assert_eq!(marker.name, expected.name, "Expected marker name to match");
            assert_eq!(marker.time, expected.time, "Expected marker time to match");
            assert_eq!(
                marker.duration, expected.duration,
                "Expected marker duration to match"
            );
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

        let marker_name = CString::new("Marker_3").unwrap();

        let path = CString::new("assets/animations/lottie/test.json").unwrap();
        assert!(
            player.load_animation_path(&path).is_ok(),
            "Animation should load"
        );

        player.set_marker(Some(&marker_name));

        assert_eq!(player.marker(), Some(marker_name.as_c_str()));

        assert!(player.is_playing(), "Animation should be playing");

        // assert current frame is the marker time
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

        // assert if all rendered frames are within the marker time and time+duration and in increasing order, as the mode is forward
        let marker = player
            .markers()
            .into_iter()
            .find(|m| m.name == "Marker_3")
            .unwrap();

        for frame in rendered_frames {
            assert!(
                frame >= marker.time && frame <= marker.time + marker.duration,
                "Expected frame to be within marker time and time+duration"
            );
        }
    }
}
