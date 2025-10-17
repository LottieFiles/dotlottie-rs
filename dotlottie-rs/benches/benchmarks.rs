use criterion::{criterion_group, criterion_main, Criterion};
use dotlottie_rs::{Config, DotLottiePlayer};

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 1000;

fn load_animation_data_benchmark(c: &mut Criterion) {
    let player = DotLottiePlayer::new(Config::default());
    let data = r#"
    {
        "nm": "Main Scene",
        "ddd": 0,
        "h": 512,
        "w": 512,
        "meta": { "g": "@lottiefiles/creator 1.51.2" },
        "layers": [
          {
            "ty": 4,
            "nm": "Rectangle 1",
            "sr": 1,
            "st": 0,
            "op": 150,
            "ip": 0,
            "hd": false,
            "ddd": 0,
            "bm": 0,
            "hasMask": false,
            "ao": 0,
            "ks": {
              "a": { "a": 0, "k": [0, 0] },
              "s": { "a": 0, "k": [100, 67.80823613896936] },
              "sk": { "a": 0, "k": 0 },
              "p": { "a": 0, "k": [251.17295566502384, 141.08045123817675] },
              "r": { "a": 0, "k": 0 },
              "sa": { "a": 0, "k": 0 },
              "o": { "a": 0, "k": 100 }
            },
            "shapes": [
              {
                "ty": "fl",
                "bm": 0,
                "hd": false,
                "nm": "Fill",
                "c": { "a": 0, "k": [0.8863, 0.2902, 0.2588] },
                "r": 1,
                "o": { "a": 0, "k": 100 }
              }
            ],
            "ind": 1
          },
          {
            "ty": 4,
            "nm": "rect_r",
            "sr": 1,
            "st": 0,
            "op": 150,
            "ip": 0,
            "hd": false,
            "ddd": 0,
            "bm": 0,
            "hasMask": false,
            "ao": 0,
            "ks": {
              "a": { "a": 0, "k": [0, 0] },
              "s": { "a": 0, "k": [100, 100] },
              "sk": { "a": 0, "k": 0 },
              "p": {
                "a": 1,
                "k": [
                  {
                    "o": { "x": 0, "y": 0 },
                    "i": { "x": 0.36, "y": 1 },
                    "s": [440.5542754563847, 256.00002264598976],
                    "t": 0
                  },
                  {
                    "o": { "x": 0.65, "y": 0 },
                    "i": { "x": 1, "y": 1 },
                    "s": [291.15499051010715, 256.000004233706],
                    "t": 37
                  },
                  { "s": [462.09785796394937, 256.00002264598976], "t": 90 }
                ]
              },
              "r": { "a": 0, "k": 0 },
              "sa": { "a": 0, "k": 0 },
              "o": { "a": 0, "k": 100 }
            },
            "shapes": [
              {
                "ty": "rc",
                "bm": 0,
                "hd": false,
                "nm": "Rect Shape 1",
                "d": 1,
                "p": { "a": 0, "k": [0, 0] },
                "r": { "a": 0, "k": 0 },
                "s": { "a": 0, "k": [77.824, 77.824] }
              },
              {
                "ty": "fl",
                "bm": 0,
                "hd": false,
                "nm": "Fill",
                "c": { "a": 0, "k": [1, 0.7451, 0] },
                "r": 1,
                "o": { "a": 0, "k": 100 }
              }
            ],
            "ind": 2
          },
          {
            "ty": 4,
            "nm": "rect_l",
            "sr": 1,
            "st": 0,
            "op": 150,
            "ip": 0,
            "hd": false,
            "ddd": 0,
            "bm": 0,
            "hasMask": false,
            "ao": 0,
            "ks": {
              "a": { "a": 0, "k": [0, 0] },
              "s": { "a": 0, "k": [100, 100] },
              "sk": { "a": 0, "k": 0 },
              "p": {
                "a": 1,
                "k": [
                  {
                    "o": { "x": 0, "y": 0 },
                    "i": { "x": 0.36, "y": 1 },
                    "s": [54.20602915405893, 255.99998627010996],
                    "t": 0
                  },
                  {
                    "o": { "x": 0.65, "y": 0 },
                    "i": { "x": 1, "y": 1 },
                    "s": [221.08800415357564, 255.99996785782622],
                    "t": 37
                  },
                  { "s": [54.20602915405893, 255.99998627010996], "t": 90 }
                ]
              },
              "r": { "a": 0, "k": 0 },
              "sa": { "a": 0, "k": 0 },
              "o": { "a": 0, "k": 100 }
            },
            "shapes": [
              {
                "ty": "rc",
                "bm": 0,
                "hd": false,
                "nm": "Rect Shape 1",
                "d": 1,
                "p": { "a": 0, "k": [0, 0] },
                "r": { "a": 0, "k": 0 },
                "s": { "a": 0, "k": [77.824, 77.824] }
              },
              {
                "ty": "fl",
                "bm": 0,
                "hd": false,
                "nm": "Fill",
                "c": { "a": 0, "k": [0.3294, 0.6863, 0.9059] },
                "r": 1,
                "o": { "a": 0, "k": 100 }
              }
            ],
            "ind": 3
          }
        ],
        "v": "5.7.0",
        "fr": 30,
        "op": 90,
        "ip": 0,
        "assets": []
      }"#;

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

fn set_theme_benchmark(c: &mut Criterion) {
    let player = DotLottiePlayer::new(Config::default());

    let data = include_bytes!("../tests/fixtures/test.lottie");
    assert!(player.load_dotlottie_data(data, WIDTH, HEIGHT));

    c.bench_function("set_theme", |b| {
        b.iter(|| {
            player.set_theme("test_theme");
        });
    });
}

fn state_machine_load_benchmark(c: &mut Criterion) {
    let player = DotLottiePlayer::new(Config::default());

    let data = include_bytes!(
        "../tests/fixtures/statemachines/normal_usecases/sm_exploding_pigeon.lottie"
    );
    assert!(player.load_dotlottie_data(data, WIDTH, HEIGHT));

    c.bench_function("state_machine_load", |b| {
        b.iter(|| {
            player.state_machine_load("Exploding Pigeon");
        });
    });
}

fn state_machine_load_data_benchmark(c: &mut Criterion) {
    let player = DotLottiePlayer::new(Config::default());
    let state_machine_data = std::str::from_utf8(include_bytes!(
        "../tests/fixtures/statemachines/normal_usecases/exploding_pigeon.json"
    ))
    .unwrap();

    let animation_data = include_bytes!(
        "../tests/fixtures/statemachines/normal_usecases/sm_exploding_pigeon.lottie"
    );
    assert!(player.load_dotlottie_data(animation_data, WIDTH, HEIGHT));

    c.bench_function("state_machine_load_data", |b| {
        b.iter(|| {
            player.state_machine_load_data(state_machine_data);
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
