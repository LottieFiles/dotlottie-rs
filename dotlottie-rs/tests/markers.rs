use dotlottie_player_core::{Config, DotLottiePlayer, Marker};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default_marker() {
        let player = DotLottiePlayer::new(Config::default());

        assert!(
            player.config().marker.is_empty(),
            "Expected no marker by default"
        );
    }

    #[test]
    fn test_markers() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        assert!(
            player.markers().is_empty(),
            "Expected no markers before loading animation"
        );

        assert!(
            player.load_animation_path("tests/assets/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );

        let actual_markers = player.markers();

        assert_eq!(actual_markers.len(), 4);

        let expected_markers = [Marker {
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
            }];

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
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            ..Config::default()
        });

        let marker_name = "Marker_3".to_string();

        player.set_config(Config {
            marker: marker_name.clone(),
            ..player.config()
        });

        assert_eq!(player.config().marker, marker_name.clone());

        assert!(
            player.load_animation_path("tests/assets/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");

        // assert current frame is the marker time
        assert_eq!(player.current_frame(), 20.0);

        let mut rendered_frames: Vec<f32> = vec![];

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) && player.render() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        // assert if all rendered frames are within the marker time and time+duration and in increasing order, as the mode is forward
        let marker = player
            .markers()
            .into_iter()
            .find(|m| m.name == marker_name.clone())
            .unwrap();

        for frame in rendered_frames {
            assert!(
                frame >= marker.time && frame <= marker.time + marker.duration,
                "Expected frame to be within marker time and time+duration"
            );
        }
    }
}
