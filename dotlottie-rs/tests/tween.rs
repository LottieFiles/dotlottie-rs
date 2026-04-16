use std::ffi::CString;

use dotlottie_rs::{ColorSpace, DotLottiePlayer, TweenStatus};

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
        .load_animation_path(&path)
        .expect("animation should load");

    (player, buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tween_accepts_easing_with_y_values_outside_unit_range() {
        let (mut player, _buf) = setup_player();

        let result = player.tween(10.0, 500.0, [0.36, 0.0, 0.66, -0.56]);
        assert!(
            result.is_ok(),
            "Easing with y2 = -0.56 should be accepted, got {result:?}"
        );
    }

    #[test]
    fn tween_accepts_easing_with_y_values_above_one() {
        let (mut player, _buf) = setup_player();

        let result = player.tween(10.0, 500.0, [0.34, 1.56, 0.64, 1.0]);
        assert!(
            result.is_ok(),
            "Easing with y1 = 1.56 should be accepted, got {result:?}"
        );
    }

    #[test]
    fn tween_rejects_easing_with_x_values_outside_unit_range() {
        let (mut player, _buf) = setup_player();

        let result = player.tween(10.0, 500.0, [-0.1, 0.0, 0.5, 1.0]);
        assert!(result.is_err(), "Easing with x1 = -0.1 should be rejected");
    }

    #[test]
    fn tween_rejects_easing_with_x2_above_one() {
        let (mut player, _buf) = setup_player();

        let result = player.tween(10.0, 500.0, [0.0, 0.0, 1.5, 1.0]);
        assert!(result.is_err(), "Easing with x2 = 1.5 should be rejected");
    }

    #[test]
    fn tween_advance_returns_completed_on_completion() {
        let (mut player, _buf) = setup_player();

        player
            .tween(20.0, 1.0, [0.0, 0.0, 1.0, 1.0])
            .expect("tween should start");
        assert!(player.is_tweening());

        // Pass enough dt to complete the 1ms tween
        let result = player.tween_advance(10.0);
        assert_eq!(
            result,
            Ok(TweenStatus::Completed),
            "tween_advance should return Ok(Completed) when tween completes"
        );
        assert!(
            !player.is_tweening(),
            "should not be tweening after tween completes"
        );
    }

    #[test]
    fn tick_continues_normally_after_tween_completion() {
        let (mut player, _buf) = setup_player();

        player.play().expect("play should succeed");

        // Advance animation by 100ms
        let _ = player.tick(100.0);

        player
            .tween(20.0, 1.0, [0.0, 0.0, 1.0, 1.0])
            .expect("tween should start");

        // Tick with enough dt to complete the 1ms tween
        let result = player.tick(10.0);
        assert!(result.is_ok(), "tick() should succeed, got {result:?}");
        assert!(
            !player.is_tweening(),
            "should not be tweening after tick completes the tween"
        );

        let result = player.tick(1000.0 / 60.0);
        assert!(
            result.is_ok(),
            "subsequent tick() should succeed, got {result:?}"
        );

        let frame = player.current_frame();
        let total_frames = player.total_frames();
        assert!(
            (frame - 20.0).abs() < total_frames * 0.5,
            "frame after tween should be near target (20.0), not jumped far ahead; got {frame}"
        );
    }
}
