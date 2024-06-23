use criterion::{criterion_group, criterion_main, Criterion};
use dotlottie_player_core::{Config, DotLottiePlayer};

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 1000;

fn load_animation_data_benchmark(c: &mut Criterion) {
    let player = DotLottiePlayer::new(Config::default());
    let data = std::str::from_utf8(include_bytes!("../tests/fixtures/test.json")).unwrap();

    c.bench_function("load_animation_data", |b| {
        b.iter(|| {
            assert!(player.load_animation_data(data, WIDTH, HEIGHT));
        });
    });
}

fn load_animation_path_benchmark(c: &mut Criterion) {
    let player = DotLottiePlayer::new(Config::default());

    let path = &format!(
        "{}/tests/fixtures/test.json",
        std::env!("CARGO_MANIFEST_DIR")
    );

    c.bench_function("load_animation_path", |b| {
        b.iter(|| {
            assert!(player.load_animation_path(path, WIDTH, HEIGHT));
        });
    });
}

fn load_dotlottie_data_benchmark(c: &mut Criterion) {
    let player = DotLottiePlayer::new(Config::default());

    let data = include_bytes!("../tests/fixtures/emoji.lottie");

    c.bench_function("load_dotlottie_data", |b| {
        b.iter(|| {
            assert!(player.load_dotlottie_data(data, WIDTH, HEIGHT));
        });
    });
}

fn animation_loop_benchmark(c: &mut Criterion) {
    let player = DotLottiePlayer::new(Config {
        autoplay: true,
        loop_animation: true,
        ..Config::default()
    });

    assert!(player.load_dotlottie_data(
        include_bytes!("../tests/fixtures/emoji.lottie"),
        WIDTH,
        HEIGHT
    ));

    c.bench_function("animation_loop_no_frame_interpolation", |b| {
        b.iter(|| {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) {
                player.render();
            }
        });
    });

    let player = DotLottiePlayer::new(Config {
        autoplay: true,
        loop_animation: true,
        use_frame_interpolation: true,
        ..Config::default()
    });
    assert!(player.load_dotlottie_data(
        include_bytes!("../tests/fixtures/emoji.lottie"),
        WIDTH,
        HEIGHT
    ));

    c.bench_function("animation_loop_frame_interpolation", |b| {
        b.iter(|| {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame) {
                player.render();
            }
        });
    });
}

fn load_theme_benchmark(c: &mut Criterion) {
    let player = DotLottiePlayer::new(Config::default());

    let data = include_bytes!("../tests/fixtures/test.lottie");
    assert!(player.load_dotlottie_data(data, WIDTH, HEIGHT));

    c.bench_function("load_theme", |b| {
        b.iter(|| {
            player.load_theme("test_theme");
        });
    });
}

criterion_group!(
    benches,
    load_animation_data_benchmark,
    load_animation_path_benchmark,
    load_dotlottie_data_benchmark,
    animation_loop_benchmark,
    load_theme_benchmark,
);
criterion_main!(benches);
