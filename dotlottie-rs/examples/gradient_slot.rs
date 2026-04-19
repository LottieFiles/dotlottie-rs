#![allow(clippy::print_stdout)]

/// Gradient Slot Example
///
/// This example demonstrates how to use the `set_gradient_slot` API to dynamically
/// change gradient fills in a Lottie animation. The gradient.json animation has
/// a slot with ID "gradient_fill" that we can modify.
///
/// Demonstrates both static and animated slot values.
use dotlottie_rs::{ColorSpace, Player, GradientSlot, GradientStop, LottieKeyframe};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;

mod common;

const WIDTH: u32 = 550;
const HEIGHT: u32 = 550;

fn main() {
    let mut window = Window::new(
        "Gradient Slot Example - Press T to toggle, SPACE to cycle",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    // Create player and load animation
    let mut player = Player::new();
    player.set_loop(true);
    player.set_autoplay(true);

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888)
        .unwrap();

    let animation_data = include_str!("../assets/animations/lottie/gradient.json");
    let c_data = CString::new(animation_data).expect("CString conversion failed");

    if player.load_animation_data(&c_data).is_err() {
        eprintln!("Failed to load animation");
        return;
    }

    println!("Animation loaded successfully!");
    println!("Press T to toggle between static and animated modes");
    println!("Press SPACE to cycle through different gradients (static mode)");
    println!("Press ESC to quit");

    let gradients = [
        (
            "Sunset",
            vec![
                GradientStop {
                    offset: 0.0,
                    color: [1.0, 0.4, 0.0, 1.0],
                }, // Orange
                GradientStop {
                    offset: 0.5,
                    color: [1.0, 0.6, 0.2, 1.0],
                }, // Light Orange
                GradientStop {
                    offset: 1.0,
                    color: [1.0, 0.8, 0.0, 1.0],
                }, // Yellow
            ],
        ),
        (
            "Ocean",
            vec![
                GradientStop {
                    offset: 0.0,
                    color: [0.0, 0.3, 0.6, 1.0],
                }, // Deep Blue
                GradientStop {
                    offset: 0.5,
                    color: [0.0, 0.5, 0.8, 1.0],
                }, // Ocean Blue
                GradientStop {
                    offset: 1.0,
                    color: [0.0, 0.8, 1.0, 1.0],
                }, // Light Blue
            ],
        ),
        (
            "Forest",
            vec![
                GradientStop {
                    offset: 0.0,
                    color: [0.1, 0.3, 0.1, 1.0],
                }, // Dark Green
                GradientStop {
                    offset: 0.5,
                    color: [0.2, 0.6, 0.2, 1.0],
                }, // Green
                GradientStop {
                    offset: 1.0,
                    color: [0.5, 0.8, 0.3, 1.0],
                }, // Light Green
            ],
        ),
        (
            "Fire",
            vec![
                GradientStop {
                    offset: 0.0,
                    color: [1.0, 0.0, 0.0, 1.0],
                }, // Red
                GradientStop {
                    offset: 0.33,
                    color: [1.0, 0.5, 0.0, 1.0],
                }, // Orange
                GradientStop {
                    offset: 0.66,
                    color: [1.0, 1.0, 0.0, 1.0],
                }, // Yellow
                GradientStop {
                    offset: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                }, // White
            ],
        ),
        (
            "Purple Haze",
            vec![
                GradientStop {
                    offset: 0.0,
                    color: [0.3, 0.0, 0.5, 1.0],
                }, // Deep Purple
                GradientStop {
                    offset: 0.5,
                    color: [0.6, 0.2, 0.8, 1.0],
                }, // Purple
                GradientStop {
                    offset: 1.0,
                    color: [1.0, 0.4, 1.0, 1.0],
                }, // Pink
            ],
        ),
        (
            "Grayscale",
            vec![
                GradientStop {
                    offset: 0.0,
                    color: [0.0, 0.0, 0.0, 1.0],
                }, // Black
                GradientStop {
                    offset: 0.5,
                    color: [0.5, 0.5, 0.5, 1.0],
                }, // Gray
                GradientStop {
                    offset: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                }, // White
            ],
        ),
    ];

    let mut current_gradient_index = 0;
    let mut last_space_press = std::time::Instant::now();
    let mut last_toggle_press = std::time::Instant::now();
    let mut clock = common::Clock::new();
    let mut is_animated = false;

    let gradient_slot = GradientSlot::new(gradients[current_gradient_index].1.clone());
    let _ = player.set_gradient_slot("gradient_fill", gradient_slot);
    println!(
        "Mode: STATIC | Current gradient: {}",
        gradients[current_gradient_index].0
    );

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        let dt = clock.dt();

        if window.is_key_down(Key::T) && now.duration_since(last_toggle_press).as_millis() > 200 {
            is_animated = !is_animated;

            if is_animated {
                let gradient_slot = GradientSlot::with_keyframes(vec![
                    LottieKeyframe {
                        frame: 0,
                        start_value: vec![
                            GradientStop {
                                offset: 0.0,
                                color: [1.0, 0.4, 0.0, 1.0],
                            },
                            GradientStop {
                                offset: 0.5,
                                color: [1.0, 0.6, 0.2, 1.0],
                            },
                            GradientStop {
                                offset: 1.0,
                                color: [1.0, 0.8, 0.0, 1.0],
                            },
                        ],
                        in_tangent: None,
                        out_tangent: None,
                        value_in_tangent: None,
                        value_out_tangent: None,
                        hold: None,
                    },
                    LottieKeyframe {
                        frame: 60,
                        start_value: vec![
                            GradientStop {
                                offset: 0.0,
                                color: [0.0, 0.3, 0.6, 1.0],
                            },
                            GradientStop {
                                offset: 0.5,
                                color: [0.0, 0.5, 0.8, 1.0],
                            },
                            GradientStop {
                                offset: 1.0,
                                color: [0.0, 0.8, 1.0, 1.0],
                            },
                        ],
                        in_tangent: None,
                        out_tangent: None,
                        value_in_tangent: None,
                        value_out_tangent: None,
                        hold: None,
                    },
                ]);
                let _ = player.set_gradient_slot("gradient_fill", gradient_slot);
                println!("Mode: ANIMATED (Sunset -> Ocean)");
            } else {
                let gradient_slot = GradientSlot::new(gradients[current_gradient_index].1.clone());
                let _ = player.set_gradient_slot("gradient_fill", gradient_slot);
                println!(
                    "Mode: STATIC | Current gradient: {}",
                    gradients[current_gradient_index].0
                );
            }

            last_toggle_press = now;
        }

        if !is_animated
            && window.is_key_down(Key::Space)
            && now.duration_since(last_space_press).as_millis() > 200
        {
            current_gradient_index = (current_gradient_index + 1) % gradients.len();

            let gradient_slot = GradientSlot::new(gradients[current_gradient_index].1.clone());
            let _ = player.set_gradient_slot("gradient_fill", gradient_slot);

            println!(
                "Mode: STATIC | Current gradient: {}",
                gradients[current_gradient_index].0
            );
            last_space_press = now;
        }

        if player.tick(dt).unwrap_or(false) {
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!("Example finished!");
}
