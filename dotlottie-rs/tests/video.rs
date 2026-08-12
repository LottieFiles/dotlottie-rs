use dotlottie_rs::{ColorSpace, Player};

mod test_utils;
use crate::test_utils::{HEIGHT, WIDTH};

/// `video_test.lottie` is a v2 package holding a 2s 160x120 clip in `v/`,
/// referenced by an ordinary image layer running the animation's full 0..60
/// frame range at 30fps.
const VIDEO_TEST: &[u8] = include_bytes!("../assets/animations/dotlottie/v2/video_test.lottie");

fn render_at(player: &mut Player, frame: f32) {
    let _ = player.set_frame(frame);
    let _ = player.render();
}

#[test]
fn video_package_loads_and_renders() {
    let mut player = Player::new();
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    assert!(player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .is_ok());

    assert_eq!(player.load_dotlottie_data(VIDEO_TEST), Ok(()));
    assert_eq!(player.total_frames(), 60.0);

    for frame in [0.0, 15.0, 30.0, 45.0] {
        render_at(&mut player, frame);
    }
}

/// The clip covers the whole canvas, so a decoded frame shows up as opaque
/// pixels. Decoding is asynchronous on every real backend, so allow a bounded
/// number of ticks rather than demanding the first one.
///
/// Frame *advance* is deliberately not asserted: the media backend is chosen at
/// link time, and the Apple one drives playback from an AVQueuePlayer, which
/// does not progress in a headless test process.
#[test]
fn video_frames_reach_the_canvas() {
    let mut player = Player::new();
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    assert!(player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .is_ok());
    assert_eq!(player.load_dotlottie_data(VIDEO_TEST), Ok(()));

    let mut opaque = 0;
    for tick in 0..120 {
        render_at(&mut player, (tick % 60) as f32);
        opaque = buffer.iter().filter(|px| **px != 0).count();
        if opaque > 0 {
            break;
        }
    }

    assert!(
        opaque > 0,
        "the video layer never produced pixels over 120 ticks"
    );
}

/// Smoke test only: play/pause/stop must stay well-behaved with a video loaded.
/// Whether the pause actually reaches a player is covered by the registry unit
/// test in `renderer::media`, since the backend here is ThorVG's own.
#[test]
fn player_lifecycle_runs_with_video() {
    let mut player = Player::new();
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    assert!(player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .is_ok());
    assert_eq!(player.load_dotlottie_data(VIDEO_TEST), Ok(()));

    assert_eq!(player.play(), Ok(()));
    render_at(&mut player, 10.0);
    assert_eq!(player.pause(), Ok(()));
    render_at(&mut player, 10.0);
    assert_eq!(player.stop(), Ok(()));
}
