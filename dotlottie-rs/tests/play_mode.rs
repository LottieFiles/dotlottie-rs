use std::sync::{Arc, Mutex};

use dotlottie_rs::{Config, DotLottiePlayer, Observer};

mod test_utils;

use crate::test_utils::{HEIGHT, WIDTH};

struct LoopObserver {
    loop_count: Arc<Mutex<u32>>,
    completed: Arc<Mutex<bool>>,
}

impl Observer for LoopObserver {
    fn on_loop(&self, _count: u32) {
        let mut count = self.loop_count.lock().unwrap();
        *count += 1;
    }

    fn on_load(&self) {}

    fn on_load_error(&self) {}

    fn on_play(&self) {}

    fn on_pause(&self) {}

    fn on_stop(&self) {}

    fn on_frame(&self, _: f32) {}

    fn on_render(&self, _: f32) {}

    fn on_complete(&self) {
        let mut completed = self.completed.lock().unwrap();
        *completed = true;
    }
}

#[cfg(test)]
mod play_mode_tests {
    use dotlottie_rs::Mode;

    use super::*;

    #[test]
    fn test_default_play_mode() {
        let player = DotLottiePlayer::new(Config::default());

        assert_eq!(player.config().mode, Mode::Forward);
    }

    #[test]
    fn test_loop_count_with_loop_animation_false() {
        let player = DotLottiePlayer::new(Config {
            mode: Mode::Forward,
            autoplay: true,
            loop_animation: false,
            loop_count: 3,
            ..Config::default()
        });

        let observed_loops = Arc::new(Mutex::new(0));
        let observed_completed = Arc::new(Mutex::new(false));
        let observer = Arc::new(LoopObserver {
            loop_count: Arc::clone(&observed_loops),
            completed: Arc::clone(&observed_completed),
        });

        player.subscribe(observer);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        loop {
            if player.is_paused() || player.is_stopped() || player.loop_count() > 5 {
                break;
            }
            player.tick();
        }

        let loops = *observed_loops.lock().unwrap();
        assert_eq!(loops, 0, "Should not have looped");

        assert!(player.is_complete());
    }

    #[test]
    fn test_zero_loop_count() {
        let player = DotLottiePlayer::new(Config {
            mode: Mode::Forward,
            autoplay: true,
            loop_animation: true,
            loop_count: 0,
            ..Config::default()
        });

        let observed_loops = Arc::new(Mutex::new(0));
        let observed_completed = Arc::new(Mutex::new(false));
        let observer = Arc::new(LoopObserver {
            loop_count: Arc::clone(&observed_loops),
            completed: Arc::clone(&observed_completed),
        });

        player.subscribe(observer);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        loop {
            if player.is_paused() || player.is_stopped() || player.loop_count() > 5 {
                break;
            }
            player.tick();
        }

        let loops = *observed_loops.lock().unwrap();
        assert_eq!(loops, 6, "Will loop and ignore loop count");

        assert!(player.is_complete());
    }

    #[test]
    fn test_playing_after_loop_has_completed() {
        let player = DotLottiePlayer::new(Config {
            mode: Mode::Forward,
            autoplay: true,
            loop_animation: true,
            loop_count: 3,
            ..Config::default()
        });

        let observed_loops = Arc::new(Mutex::new(0));
        let observed_completed = Arc::new(Mutex::new(false));
        let observer = Arc::new(LoopObserver {
            loop_count: Arc::clone(&observed_loops),
            completed: Arc::clone(&observed_completed),
        });

        player.subscribe(observer);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        loop {
            if player.is_paused() || player.is_stopped() || player.loop_count() > 5 {
                break;
            }
            player.tick();
        }

        let loops = *observed_loops.lock().unwrap();
        assert_eq!(loops, 3, "Should have looped 3 times, got {loops}");

        let completed = *observed_completed.lock().unwrap();
        assert!(completed);

        // Restart the player
        player.play();

        loop {
            if player.is_paused() || player.is_stopped() || player.loop_count() > 5 {
                break;
            }
            player.tick();
        }

        let loops = *observed_loops.lock().unwrap();
        assert_eq!(loops, 6, "Should have looped 6 times, got {loops}");
    }

    #[test]
    fn test_loop_count_paused_mid_play() {
        let player = DotLottiePlayer::new(Config {
            mode: Mode::Forward,
            autoplay: true,
            loop_animation: true,
            loop_count: 5,
            ..Config::default()
        });

        let observed_loops = Arc::new(Mutex::new(0));
        let observed_completed = Arc::new(Mutex::new(false));
        let observer = Arc::new(LoopObserver {
            loop_count: Arc::clone(&observed_loops),
            completed: Arc::clone(&observed_completed),
        });

        player.subscribe(observer);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        loop {
            if player.is_paused() || player.is_stopped() || player.loop_count() >= 3 {
                break;
            }
            player.tick();
        }

        let loops = *observed_loops.lock().unwrap();
        assert_eq!(loops, 3, "Should have looped 3 times, got {loops}");

        // Restart the player
        player.play();

        loop {
            if player.is_paused() || player.is_stopped() || player.loop_count() > 10 {
                break;
            }
            player.tick();
        }

        let loops = *observed_loops.lock().unwrap();
        assert_eq!(loops, 5, "Should have looped 5 times, got {loops}");

        let completed = *observed_completed.lock().unwrap();
        assert!(completed);
    }

    #[test]
    fn test_set_play_mode() {
        let play_modes = vec![
            Mode::Forward,
            Mode::Reverse,
            Mode::Bounce,
            Mode::ReverseBounce,
        ];

        for mode in play_modes {
            let player = DotLottiePlayer::new(Config::default());

            let mut config = player.config();
            config.mode = mode;
            player.set_config(config);

            assert_eq!(
                player.config().mode,
                mode,
                "Expected play mode to be {:?}, found {:?}",
                mode,
                player.config().mode
            );

            assert!(
                player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
                "Animation should load"
            );

            match mode {
                Mode::Forward => {
                    assert_eq!(
                        player.current_frame(),
                        0.0,
                        "Expected current frame to be 0"
                    );
                }
                Mode::Reverse => {
                    assert_eq!(
                        player.current_frame(),
                        player.total_frames() - 1.0,
                        "Expected current frame to be end frame"
                    );
                }
                Mode::Bounce => {
                    assert_eq!(
                        player.current_frame(),
                        0.0,
                        "Expected current frame to be 0"
                    );
                }
                Mode::ReverseBounce => {
                    assert_eq!(
                        player.current_frame(),
                        player.total_frames() - 1.0,
                        "Expected current frame to be end frame"
                    );
                }
            }
        }
    }

    #[test]
    fn test_forward_play_mode() {
        let player = DotLottiePlayer::new(Config {
            mode: Mode::Forward,
            autoplay: true,
            ..Config::default()
        });

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        let mut rendered_frames: Vec<f32> = vec![];

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) && player.render() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(
            rendered_frames.len() >= (player.total_frames() - 1.0) as usize,
            "Expected rendered frames to be greater than or equal to total frames"
        );

        let mut prev_frame = 0.0;
        for frame in rendered_frames {
            assert!(
                frame >= prev_frame,
                "Expected frame to be greater than or equal to previous frame"
            );
            prev_frame = frame;
        }

        assert_eq!(
            player.total_frames() - 1.0,
            prev_frame,
            "Expected last frame to be total frames"
        );
    }

    #[test]
    fn test_forward_play_mode_with_loop_count() {
        let player = DotLottiePlayer::new(Config {
            mode: Mode::Forward,
            autoplay: true,
            loop_animation: true,
            loop_count: 3,
            ..Config::default()
        });

        let observed_loops = Arc::new(Mutex::new(0));
        let observed_completed = Arc::new(Mutex::new(false));
        let observer = Arc::new(LoopObserver {
            loop_count: Arc::clone(&observed_loops),
            completed: Arc::clone(&observed_completed),
        });

        player.subscribe(observer);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        loop {
            if player.is_paused() || player.is_stopped() || player.loop_count() > 5 {
                break;
            }
            player.tick();
        }

        let loops = *observed_loops.lock().unwrap();
        assert_eq!(loops, 3, "Should have looped 3 times, got {loops}");

        let completed = *observed_completed.lock().unwrap();
        assert!(completed);
    }

    #[test]
    fn test_reverse_play_mode() {
        let player = DotLottiePlayer::new(Config {
            mode: Mode::Reverse,
            autoplay: true,
            ..Config::default()
        });

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );

        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        let mut rendered_frames: Vec<f32> = vec![];

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) && player.render() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(
            rendered_frames.len() >= (player.total_frames() - 1.0) as usize,
            "Expected rendered frames to be greater than or equal to total frames"
        );

        let mut prev_frame = player.total_frames() - 1.0;
        for frame in rendered_frames {
            assert!(
                frame <= prev_frame,
                "Expected frame to be less than or equal to previous frame"
            );
            prev_frame = frame;
        }

        // check if the last frame is 0
        assert_eq!(0.0, prev_frame, "Expected last frame to be 0");
    }

    #[test]
    fn test_reverse_play_mode_with_loop_count() {
        use std::sync::{Arc, Mutex};

        let player = DotLottiePlayer::new(Config {
            mode: Mode::Reverse,
            autoplay: true,
            loop_animation: true,
            loop_count: 3,
            ..Config::default()
        });

        let observed_loops = Arc::new(Mutex::new(0));
        let observed_completed = Arc::new(Mutex::new(false));
        let observer = Arc::new(LoopObserver {
            loop_count: Arc::clone(&observed_loops),
            completed: Arc::clone(&observed_completed),
        });

        player.subscribe(observer);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        loop {
            if player.is_paused() || player.is_stopped() || player.loop_count() > 5 {
                break;
            }
            player.tick();
        }

        let loops = *observed_loops.lock().unwrap();
        assert_eq!(loops, 3, "Should have looped 3 times, got {loops}");

        let completed = *observed_completed.lock().unwrap();
        assert!(completed);
    }

    #[test]
    fn test_bounce_play_mode() {
        let player = DotLottiePlayer::new(Config {
            mode: Mode::Bounce,
            autoplay: true,
            ..Config::default()
        });

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );

        let mut rendered_frames: Vec<f32> = vec![];

        assert!(player.is_playing(), "Animation should be playing");
        assert!(!player.is_complete(), "Animation should not be complete");

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) && player.render() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(
            rendered_frames.len() >= (player.total_frames() - 1.0) as usize,
            "Expected rendered frames to be greater than or equal to total frames"
        );

        let mut frame_idx = 0;

        while frame_idx < rendered_frames.len()
            && rendered_frames[frame_idx] < player.total_frames() - 1.0
        {
            assert!(
                rendered_frames[frame_idx] <= rendered_frames[frame_idx + 1],
                "Expected frame to be less than or equal to next frame"
            );
            frame_idx += 1;
        }

        assert!(
            rendered_frames[frame_idx] == player.total_frames() - 1.0,
            "Expected frame to be total frames at index {frame_idx}"
        );

        while frame_idx < rendered_frames.len() && rendered_frames[frame_idx] > 0.0 {
            assert!(
                rendered_frames[frame_idx] >= rendered_frames[frame_idx + 1],
                "Expected frame to be greater than or equal to next frame"
            );
            frame_idx += 1;
        }

        assert!(
            rendered_frames[frame_idx] == 0.0,
            "Expected frame to be 0 at index {frame_idx}"
        );
    }

    #[test]
    fn test_bounce_play_mode_with_loop_count() {
        let player = DotLottiePlayer::new(Config {
            mode: Mode::Bounce,
            autoplay: true,
            loop_animation: true,
            loop_count: 3,
            ..Config::default()
        });

        let observed_loops = Arc::new(Mutex::new(0));
        let observed_completed = Arc::new(Mutex::new(false));
        let observer = Arc::new(LoopObserver {
            loop_count: Arc::clone(&observed_loops),
            completed: Arc::clone(&observed_completed),
        });

        player.subscribe(observer);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");

        loop {
            if player.is_paused() || player.is_stopped() || player.loop_count() > 5 {
                break;
            }
            player.tick();
        }

        let loops = *observed_loops.lock().unwrap();
        assert_eq!(loops, 3, "Should have looped 3 times, got {loops}");

        let completed = *observed_completed.lock().unwrap();
        assert!(completed);
    }

    #[test]
    fn test_reverse_bounce_play_mode() {
        let player = DotLottiePlayer::new(Config {
            mode: Mode::ReverseBounce,
            autoplay: true,
            ..Config::default()
        });

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );

        assert!(player.is_playing(), "Animation should be playing");

        let mut rendered_frames: Vec<f32> = vec![];

        // animation loop
        while !player.is_complete() {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) && player.render() {
                let current_frame = player.current_frame();
                rendered_frames.push(current_frame);
            }
        }

        assert!(
            rendered_frames.len() >= (player.total_frames() - 1.0) as usize,
            "Expected rendered frames to be greater than or equal to total frames"
        );

        let mut frame_idx = 0;

        while frame_idx < rendered_frames.len() && rendered_frames[frame_idx] > 0.0 {
            assert!(
                rendered_frames[frame_idx] >= rendered_frames[frame_idx + 1],
                "Expected frame to be greater than or equal to next frame"
            );
            frame_idx += 1;
        }

        assert!(
            rendered_frames[frame_idx] == 0.0,
            "Expected frame to be 0 at index {frame_idx}"
        );

        while frame_idx < rendered_frames.len()
            && rendered_frames[frame_idx] < player.total_frames() - 1.0
        {
            assert!(
                rendered_frames[frame_idx] <= rendered_frames[frame_idx + 1],
                "Expected frame to be less than or equal to next frame"
            );
            frame_idx += 1;
        }

        assert!(
            rendered_frames[frame_idx] == player.total_frames() - 1.0,
            "Expected frame to be total frames at index {frame_idx}"
        );
    }

    #[test]
    fn test_reverse_bounce_play_mode_with_loop_count() {
        let player = DotLottiePlayer::new(Config {
            mode: Mode::ReverseBounce,
            autoplay: true,
            loop_animation: true,
            loop_count: 3,
            ..Config::default()
        });

        let observed_loops = Arc::new(Mutex::new(0));
        let observed_completed = Arc::new(Mutex::new(false));
        let observer = Arc::new(LoopObserver {
            loop_count: Arc::clone(&observed_loops),
            completed: Arc::clone(&observed_completed),
        });

        player.subscribe(observer);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Animation should load"
        );
        assert!(player.is_playing(), "Animation should be playing");

        loop {
            if player.is_paused() || player.is_stopped() || player.loop_count() > 5 {
                break;
            }
            player.tick();
        }

        let loops = *observed_loops.lock().unwrap();
        assert_eq!(loops, 3, "Should have looped 3 times, got {loops}");

        let completed = *observed_completed.lock().unwrap();
        assert!(completed);
    }
}
