use std::ffi::CString;

use criterion::{criterion_group, criterion_main, Criterion};
use dotlottie_rs::{ColorSpace, DotLottiePlayer};

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 1000;

fn load_animation_data_benchmark(c: &mut Criterion) {
    let mut player = DotLottiePlayer::new(0);
    let data_str =
        std::str::from_utf8(include_bytes!("../assets/animations/lottie/test.json")).unwrap();
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT).try_into().unwrap()];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();

    c.bench_function("load_animation_data", |b| {
        b.iter(|| {
            let data = CString::new(data_str).expect("Failed to create CString");
            assert!(player.load_animation_data(&data, WIDTH, HEIGHT).is_ok());
        });
    });
}

fn load_animation_path_benchmark(c: &mut Criterion) {
    let mut player = DotLottiePlayer::new(0);
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT).try_into().unwrap()];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();
    let path = CString::new(format!(
        "{}/assets/animations/lottie/test.json",
        std::env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    c.bench_function("load_animation_path", |b| {
        b.iter(|| {
            assert!(player.load_animation_path(&path, WIDTH, HEIGHT).is_ok());
        });
    });
}

fn load_dotlottie_data_benchmark(c: &mut Criterion) {
    let mut player = DotLottiePlayer::new(0);
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT).try_into().unwrap()];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();
    let data = include_bytes!("../assets/animations/dotlottie/v1/emojis.lottie");

    c.bench_function("load_dotlottie_data", |b| {
        b.iter(|| {
            assert!(player.load_dotlottie_data(data, WIDTH, HEIGHT).is_ok());
        });
    });
}

fn animation_loop_benchmark(c: &mut Criterion) {
    let mut player = DotLottiePlayer::new(0);
    player.set_autoplay(true);
    player.set_loop(true);

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT).try_into().unwrap()];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();
    assert!(player
        .load_dotlottie_data(
            include_bytes!("../assets/animations/dotlottie/v1/emojis.lottie"),
            WIDTH,
            HEIGHT
        )
        .is_ok());

    c.bench_function("animation_loop_no_frame_interpolation", |b| {
        b.iter(|| {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame).is_ok() {
                let _ = player.render();
            }
        });
    });

    let mut player = DotLottiePlayer::new(0);
    player.set_autoplay(true);
    player.set_loop(true);
    player.set_use_frame_interpolation(true);
    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();
    assert!(player
        .load_dotlottie_data(
            include_bytes!("../assets/animations/dotlottie/v1/emojis.lottie"),
            WIDTH,
            HEIGHT
        )
        .is_ok());
    assert!(player
        .load_dotlottie_data(
            include_bytes!("../assets/animations/dotlottie/v1/emojis.lottie"),
            WIDTH,
            HEIGHT
        )
        .is_ok());

    c.bench_function("animation_loop_frame_interpolation", |b| {
        b.iter(|| {
            let next_frame = player.request_frame();

            if player.set_frame(next_frame).is_ok() {
                let _ = player.render();
            }
        });
    });
}

fn set_theme_benchmark(c: &mut Criterion) {
    let mut player = DotLottiePlayer::new(0);
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT).try_into().unwrap()];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();
    let data = include_bytes!("../assets/animations/dotlottie/v2/themed.lottie");
    assert!(player.load_dotlottie_data(data, WIDTH, HEIGHT).is_ok());

    c.bench_function("set_theme", |b| {
        b.iter(|| {
            let _ = player.set_theme(c"test_theme");
        });
    });
}

fn state_machine_load_benchmark(c: &mut Criterion) {
    let mut player = DotLottiePlayer::new(0);
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT).try_into().unwrap()];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();
    let data = include_bytes!("../assets/statemachines/normal_usecases/sm_exploding_pigeon.lottie");
    assert!(player.load_dotlottie_data(data, WIDTH, HEIGHT).is_ok());

    c.bench_function("state_machine_load", |b| {
        b.iter(|| {
            let _ = player.state_machine_load(c"Exploding Pigeon");
        });
    });
}

fn state_machine_load_data_benchmark(c: &mut Criterion) {
    let mut player = DotLottiePlayer::new(0);
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT).try_into().unwrap()];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();
    let state_machine_data = std::str::from_utf8(include_bytes!(
        "../assets/statemachines/normal_usecases/exploding_pigeon.json"
    ))
    .unwrap();

    let animation_data =
        include_bytes!("../assets/statemachines/normal_usecases/sm_exploding_pigeon.lottie");
    assert!(player
        .load_dotlottie_data(animation_data, WIDTH, HEIGHT)
        .is_ok());

    c.bench_function("state_machine_load_data", |b| {
        b.iter(|| {
            let _ = player.state_machine_load_data(state_machine_data);
        });
    });
}

criterion_group!(
    benches,
    load_animation_data_benchmark,
    load_animation_path_benchmark,
    load_dotlottie_data_benchmark,
    animation_loop_benchmark,
    set_theme_benchmark,
    state_machine_load_benchmark,
    state_machine_load_data_benchmark,
);
criterion_main!(benches);
