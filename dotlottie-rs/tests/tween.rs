use std::ffi::CString;

use dotlottie_rs::{ColorSpace, DotLottiePlayer};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

fn setup_player() -> (DotLottiePlayer, Vec<u32>) {
    let mut player = DotLottiePlayer::new();
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .expect("set_sw_target should succeed");

    let path = CString::new("assets/animations/lottie/test.json").unwrap();
    player
        .load_animation_path(&path, WIDTH, HEIGHT)
        .expect("animation should load");

    (player, buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tween_accepts_easing_with_y_values_outside_unit_range() {
        let (mut player, _buf) = setup_player();

        // ease-in-back: y2 = -0.56 is outside [0,1] but valid for CSS cubic-bezier
        let result = player.tween(10.0, Some(0.5), Some([0.36, 0.0, 0.66, -0.56]));
        assert!(
            result.is_ok(),
            "Easing with y2 = -0.56 should be accepted, got {result:?}"
        );
    }

    #[test]
    fn tween_accepts_easing_with_y_values_above_one() {
        let (mut player, _buf) = setup_player();

        // overshoot: y1 = 1.56 is outside [0,1] but valid
        let result = player.tween(10.0, Some(0.5), Some([0.34, 1.56, 0.64, 1.0]));
        assert!(
            result.is_ok(),
            "Easing with y1 = 1.56 should be accepted, got {result:?}"
        );
    }

    #[test]
    fn tween_rejects_easing_with_x_values_outside_unit_range() {
        let (mut player, _buf) = setup_player();

        let result = player.tween(10.0, Some(0.5), Some([-0.1, 0.0, 0.5, 1.0]));
        assert!(
            result.is_err(),
            "Easing with x1 = -0.1 should be rejected"
        );
    }

    #[test]
    fn tween_rejects_easing_with_x2_above_one() {
        let (mut player, _buf) = setup_player();

        let result = player.tween(10.0, Some(0.5), Some([0.0, 0.0, 1.5, 1.0]));
        assert!(result.is_err(), "Easing with x2 = 1.5 should be rejected");
    }

    #[test]
    fn manual_progress_tween_completes_at_full_progress() {
        let (mut player, _buf) = setup_player();

        player
            .tween(20.0, None, None)
            .expect("tween should start");
        assert!(player.is_tweening(), "should be tweening after tween()");

        let result = player.tween_update(Some(0.5));
        assert!(result.is_ok(), "tween_update at 50% should succeed");
        assert!(
            player.is_tweening(),
            "should still be tweening at 50% progress"
        );

        // Drive to 100% — should complete
        let result = player.tween_update(Some(1.0));
        assert!(result.is_ok(), "tween_update at 100% should succeed");
        assert!(
            !player.is_tweening(),
            "should NOT be tweening after progress reaches 1.0"
        );
    }

    #[test]
    fn tween_to_marker_propagates_error_when_already_tweening() {
        let (mut player, _buf) = setup_player();

        player
            .tween(10.0, Some(1.0), None)
            .expect("first tween should start");
        assert!(player.is_tweening());

        let marker = CString::new("Marker_1").unwrap();
        let result = player.tween_to_marker(&marker, Some(1.0), None);
        assert!(
            result.is_err(),
            "tween_to_marker should propagate error when already tweening, got {result:?}"
        );
    }

    #[test]
    fn tick_clears_manual_progress_tween_on_error() {
        let (mut player, _buf) = setup_player();

        player
            .tween(20.0, None, None)
            .expect("tween should start");
        assert!(player.is_tweening());

        // tick() should fail because no progress is provided for manual-progress tween
        let result = player.tick();
        assert!(
            result.is_err(),
            "tick() should error on manual-progress tween without progress"
        );

        assert!(
            !player.is_tweening(),
            "tween state should be cleared after tick() error"
        );

        let result = player.tick();
        assert!(
            result.is_ok(),
            "tick() should succeed after tween state is cleared, got {result:?}"
        );
    }

    #[test]
    fn timed_tween_update_returns_false_on_completion() {
        let (mut player, _buf) = setup_player();

        // Very short timed tween (0.001s) so it completes immediately
        player
            .tween(20.0, Some(0.001), None)
            .expect("tween should start");
        assert!(player.is_tweening());

        std::thread::sleep(std::time::Duration::from_millis(10));

        let result = player.tween_update(None);
        assert!(
            result.is_ok(),
            "tween_update should succeed when timed tween completes"
        );
        assert!(
            !player.is_tweening(),
            "should not be tweening after timed tween completes"
        );
    }

    #[test]
    fn tick_continues_normally_after_tween_completion() {
        let (mut player, _buf) = setup_player();

        player.play().expect("play should succeed");

        // Advance time significantly before starting the tween
        std::thread::sleep(std::time::Duration::from_millis(100));

        player
            .tween(20.0, Some(0.001), None)
            .expect("tween should start");

        std::thread::sleep(std::time::Duration::from_millis(10));

        let result = player.tick();
        assert!(result.is_ok(), "tick() should succeed, got {result:?}");
        assert!(
            !player.is_tweening(),
            "should not be tweening after tick completes the tween"
        );

        let result = player.tick();
        assert!(
            result.is_ok(),
            "subsequent tick() should succeed, got {result:?}"
        );

        // Frame should be near tween target (20.0), not jumped far ahead
        // due to stale start_time
        let frame = player.current_frame();
        let total_frames = player.total_frames();
        assert!(
            (frame - 20.0).abs() < total_frames * 0.5,
            "frame after tween should be near target (20.0), not jumped far ahead; got {frame}"
        );
    }
}
