use std::sync::{Arc, Mutex};

use dotlottie_rs::{Config, DotLottiePlayer, Observer};

mod test_utils;

use crate::test_utils::{HEIGHT, WIDTH};

struct MockObserver {
    events: Arc<Mutex<Vec<String>>>,
}

impl MockObserver {
    fn new(events: Arc<Mutex<Vec<String>>>) -> Self {
        MockObserver { events }
    }
}

impl Observer for MockObserver {
    fn on_load_error(&self) {
        let mut events = self.events.lock().unwrap();
        events.push("on_load_error".to_string());
    }

    fn on_load(&self) {
        let mut events = self.events.lock().unwrap();
        events.push("on_load".to_string());
    }

    fn on_play(&self) {
        let mut events = self.events.lock().unwrap();
        events.push("on_play".to_string());
    }

    fn on_pause(&self) {
        let mut events = self.events.lock().unwrap();
        events.push("on_pause".to_string());
    }

    fn on_stop(&self) {
        let mut events = self.events.lock().unwrap();
        events.push("on_stop".to_string());
    }

    fn on_complete(&self) {
        let mut events = self.events.lock().unwrap();
        events.push("on_complete".to_string());
    }

    fn on_loop(&self, loop_count: u32) {
        let mut events = self.events.lock().unwrap();
        events.push(format!("on_loop: {loop_count}"));
    }

    fn on_frame(&self, frame: f32) {
        let mut events = self.events.lock().unwrap();
        events.push(format!("on_frame: {frame}"));
    }

    fn on_render(&self, frame: f32) {
        let mut events = self.events.lock().unwrap();
        events.push(format!("on_render: {frame}"));
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_subscribe_unsubscribe() {
        let player = DotLottiePlayer::new(Config {
            autoplay: true,
            loop_animation: true,
            ..Config::default()
        });

        let events = Arc::new(Mutex::new(vec![]));
        let observer_events = Arc::clone(&events);

        let observer = MockObserver::new(observer_events);
        let observer_arc: Arc<dyn Observer> = Arc::new(observer);
        player.subscribe(Arc::clone(&observer_arc));

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
        expected_events.push("on_stop".to_string());

        let recorded_events = events.lock().unwrap();

        for (i, event) in recorded_events.iter().enumerate() {
            assert_eq!(
                event, &expected_events[i],
                "Mismatch at event index {}: expected '{}', found '{}'",
                i, expected_events[i], event
            );
        }

        // unsubscribe the observer
        player.unsubscribe(&observer_arc);

        assert!(
            player.load_animation_path("tests/fixtures/test.json", WIDTH, HEIGHT),
            "Valid path should load"
        );

        assert_eq!(
            recorded_events.len(),
            expected_events.len(),
            "Events should not change after unsubscribing"
        );
    }
}
