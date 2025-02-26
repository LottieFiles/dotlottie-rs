use dotlottie_rs::TvgEngine;
use dotlottie_rs::{Config, DotLottiePlayer};

#[allow(dead_code)]
pub const WIDTH: u32 = 100;
#[allow(dead_code)]
pub const HEIGHT: u32 = 100;

pub const DEFAULT_THREADS: u32 = 1;
pub const DEFAULT_HTML_CANVAS_SELECTOR: &str = "";

pub fn create_test_player(config: Config) -> DotLottiePlayer {
    DotLottiePlayer::new(
        config,
        TvgEngine::TvgEngineSw,
        DEFAULT_THREADS,
        DEFAULT_HTML_CANVAS_SELECTOR.to_string(),
    )
}
