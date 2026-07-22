use std::ffi::CString;

use dotlottie_rs::{ColorSpace, Player, PlayerEvent, Status, TweenStatus};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

fn setup_player() -> (Player, Vec<u32>) {
    let mut player = Player::new();
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
        assert_eq!(player.status(), Status::Tweening);

        // Pass enough dt to complete the 1ms tween
        let result = player.tween_advance(10.0);
        assert_eq!(
            result,
            Ok(TweenStatus::Completed),
            "tween_advance should return Ok(Completed) when tween completes"
        );
        assert_ne!(
            player.status(),
            Status::Tweening,
            "should not be tweening after tween completes"
        );
    }

    #[test]
    fn tween_starts_a_tween_when_stopped() {
        let (mut player, _buf) = setup_player();

        let result = player.tween(20.0, 1.0, [0.0, 0.0, 1.0, 1.0]);
        assert!(result.is_ok(), "tween should start, got {result:?}");
        assert!(player.is_tweening());

        let result = player.tween_advance(10.0);
        assert_eq!(result, Ok(TweenStatus::Completed));
        assert!(!player.is_tweening());
    }

    #[test]
    fn current_frame_holds_the_origin_until_a_tween_lands() {
        let (mut player, _buf) = setup_player();
        let start = player.current_frame();

        player
            .tween(20.0, 1000.0, [0.0, 0.0, 1.0, 1.0])
            .expect("tween should start");

        let _ = player.tick(500.0);
        assert_eq!(
            player.current_frame(),
            start,
            "a blended pose is not a frame, so current_frame stays at the origin"
        );

        let _ = player.tick(600.0);
        assert!(!player.is_tweening());
        assert_eq!(player.current_frame(), 20.0);
    }

    #[test]
    fn tween_retargets_active_tween() {
        let (mut player, _buf) = setup_player();

        player
            .tween(20.0, 2000.0, [0.0, 0.0, 1.0, 1.0])
            .expect("tween should start");
        let _ = player.tick(500.0);
        assert!(player.is_tweening());

        let result = player.tween(5.0, 1000.0, [0.0, 0.0, 1.0, 1.0]);
        assert!(
            result.is_ok(),
            "tween should retarget an active tween, got {result:?}"
        );
        assert!(player.is_tweening());

        let _ = player.tick(800.0);
        assert!(
            player.is_tweening(),
            "800ms into a 1000ms retarget: still tweening"
        );
        let _ = player.tick(300.0);
        assert!(
            !player.is_tweening(),
            "1100ms into a 1000ms retarget: completed"
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
        assert_eq!(player.status(), Status::Tweening);

        // Tick with enough dt to complete the 1ms tween
        let result = player.tick(10.0);
        assert!(result.is_ok(), "tick() should succeed, got {result:?}");
        assert_eq!(
            player.status(),
            Status::Playing,
            "should resume Playing after tween completes"
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

    #[test]
    fn tween_from_stopped_resumes_to_stopped() {
        let (mut player, _buf) = setup_player();

        assert_eq!(player.status(), Status::Stopped);

        player
            .tween(10.0, 1.0, [0.0, 0.0, 1.0, 1.0])
            .expect("tween from stopped should succeed");
        assert_eq!(player.status(), Status::Tweening);

        let result = player.tween_advance(10.0);
        assert_eq!(result, Ok(TweenStatus::Completed));
        assert_eq!(
            player.status(),
            Status::Stopped,
            "should resume Stopped after tween completes"
        );
    }

    #[test]
    fn tween_from_paused_resumes_to_paused() {
        let (mut player, _buf) = setup_player();

        player.play().expect("play should succeed");
        player.pause().expect("pause should succeed");
        assert_eq!(player.status(), Status::Paused);

        player
            .tween(10.0, 1.0, [0.0, 0.0, 1.0, 1.0])
            .expect("tween from paused should succeed");
        assert_eq!(player.status(), Status::Tweening);

        let result = player.tween_advance(10.0);
        assert_eq!(result, Ok(TweenStatus::Completed));
        assert_eq!(
            player.status(),
            Status::Paused,
            "should resume Paused after tween completes"
        );
    }

    #[test]
    fn play_during_tweening_retargets_resume() {
        let (mut player, _buf) = setup_player();

        player
            .tween(10.0, 1.0, [0.0, 0.0, 1.0, 1.0])
            .expect("tween from stopped should succeed");
        assert_eq!(player.status(), Status::Tweening);

        while player.poll_event().is_some() {}

        assert_eq!(player.play(), Ok(()), "play() during tween should succeed");
        assert_eq!(
            player.status(),
            Status::Tweening,
            "player should keep tweening after retarget"
        );
        assert!(
            matches!(player.poll_event(), Some(PlayerEvent::Play)),
            "retarget should emit PlayerEvent::Play"
        );

        assert_eq!(player.tween_advance(10.0), Ok(TweenStatus::Completed));
        assert_eq!(
            player.status(),
            Status::Playing,
            "should resume Playing after retargeted tween completes"
        );
    }

    #[test]
    fn play_during_tweening_from_playing_returns_error() {
        let (mut player, _buf) = setup_player();

        player.play().expect("play should succeed");
        player
            .tween(10.0, 1.0, [0.0, 0.0, 1.0, 1.0])
            .expect("tween should start");

        assert!(
            player.play().is_err(),
            "play() should fail when the tween already resumes to Playing"
        );
        assert_eq!(player.status(), Status::Tweening);
    }

    #[test]
    fn pause_during_tweening_retargets_resume() {
        let (mut player, _buf) = setup_player();

        player.play().expect("play should succeed");
        player
            .tween(10.0, 1.0, [0.0, 0.0, 1.0, 1.0])
            .expect("tween should start");
        assert_eq!(player.status(), Status::Tweening);

        while player.poll_event().is_some() {}

        assert_eq!(
            player.pause(),
            Ok(()),
            "pause() during tween should succeed"
        );
        assert_eq!(
            player.status(),
            Status::Tweening,
            "player should keep tweening after retarget"
        );
        assert!(
            matches!(player.poll_event(), Some(PlayerEvent::Pause)),
            "retarget should emit PlayerEvent::Pause"
        );

        assert_eq!(player.tween_advance(10.0), Ok(TweenStatus::Completed));
        assert_eq!(
            player.status(),
            Status::Paused,
            "should resume Paused after retargeted tween completes"
        );
    }

    #[test]
    fn pause_during_tweening_from_stopped_returns_error() {
        let (mut player, _buf) = setup_player();

        player
            .tween(10.0, 1.0, [0.0, 0.0, 1.0, 1.0])
            .expect("tween from stopped should succeed");

        assert!(
            player.pause().is_err(),
            "pause() should fail when the tween does not resume to Playing"
        );
        assert_eq!(player.status(), Status::Tweening);
    }

    #[test]
    fn stop_during_tweening_cancels_tween() {
        let (mut player, _buf) = setup_player();

        player.play().expect("play should succeed");

        player
            .tween(10.0, 0.5, [0.0, 0.0, 1.0, 1.0])
            .expect("tween should start");
        assert_eq!(player.status(), Status::Tweening);

        let result = player.stop();
        assert!(result.is_ok(), "stop() during tweening should succeed");
        assert_eq!(
            player.status(),
            Status::Stopped,
            "should be Stopped after cancelling tween"
        );
    }
}
