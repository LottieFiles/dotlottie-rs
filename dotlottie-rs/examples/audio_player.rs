//! Audio feature demonstration.
//!
//! Loads a Lottie animation (JSON or .lottie) that contains audio layers and
//! plays it. Audio events are printed to stdout as they fire so you can verify
//! the correct frame boundaries. Audio is played via rodio on native and
//! wasm32-unknown-unknown targets.
//!
//! Usage:
//!   cargo run --example audio_player --features audio,dev -- path/to/animation.json
//!   cargo run --example audio_player --features audio,dev -- path/to/animation.lottie
//!
//! Controls:
//!   P     – play / resume
//!   S     – pause
//!   X     – stop
//!   ESC   – quit

#![allow(clippy::print_stdout)]

use dotlottie_rs::{ColorSpace, DotLottieEvent, DotLottiePlayer};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;
use std::fs;
use std::path::PathBuf;

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

fn load_animation(player: &mut DotLottiePlayer, path: &PathBuf) {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "lottie" => {
            let data = fs::read(path).expect("Failed to read .lottie file");
            player
                .load_dotlottie_data(&data, WIDTH as u32, HEIGHT as u32)
                .expect("Failed to load .lottie animation");
        }
        "json" => {
            let data = fs::read_to_string(path).expect("Failed to read JSON file");
            let c_data = CString::new(data).expect("CString conversion failed");
            player
                .load_animation_data(&c_data, WIDTH as u32, HEIGHT as u32)
                .expect("Failed to load JSON animation");
        }
        other => panic!("Unsupported file extension: {other:?}  (expected .json or .lottie)"),
    }
}

fn main() {
    let path: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            eprintln!("Usage: audio_player <path/to/animation.json|.lottie>");
            std::process::exit(1);
        });

    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        std::process::exit(1);
    }

    // -------------------------------------------------------------------------
    // Player setup
    // -------------------------------------------------------------------------
    let mut player = DotLottiePlayer::new();
    player.set_autoplay(false);
    player.set_loop(true);
    player.set_mode(dotlottie_rs::Mode::Forward);
    let _ = player.set_background_color(Some(0x000000));

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "dotLottie Audio — P=play  S=pause  X=stop  ESC=quit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    player
        .set_sw_target(
            &mut buffer,
            WIDTH as u32,
            HEIGHT as u32,
            ColorSpace::ABGR8888,
        )
        .expect("Failed to set software rendering target");

    println!("Loading: {}", path.display());
    load_animation(&mut player, &path);

    // -------------------------------------------------------------------------
    // Report audio assets
    // -------------------------------------------------------------------------
    let assets = player.audio_assets();
    println!("Audio assets found: {}", assets.len());
    for asset in &assets {
        println!(
            "  id={}  mime={}  decoded={} bytes",
            asset.id,
            asset.mime_type,
            asset.data.len()
        );
    }
    if assets.is_empty() {
        println!("  (no audio layers — try a Lottie file that contains ty:6 layers)");
    }

    println!(
        "\nAnimation: {:.0} frames  ({:.2} s)",
        player.total_frames(),
        player.duration(),
    );
    println!("\nControls:  P = play/resume   S = pause   X = stop   ESC = quit");
    println!("           (press P to start)\n");

    // -------------------------------------------------------------------------
    // Render loop
    // -------------------------------------------------------------------------
    let mut p_was_down = false;
    let mut s_was_down = false;
    let mut x_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let p_down = window.is_key_down(Key::P);
        let s_down = window.is_key_down(Key::S);
        let x_down = window.is_key_down(Key::X);

        if p_was_down && !p_down && (player.is_paused() || player.is_stopped()) {
            let _ = player.play();
        }
        if s_was_down && !s_down && player.is_playing() {
            let _ = player.pause();
        }
        if x_was_down && !x_down && !player.is_stopped() {
            let _ = player.stop();
        }

        p_was_down = p_down;
        s_was_down = s_down;
        x_was_down = x_down;

        let _ = player.tick();

        let state = if player.is_playing() {
            "PLAYING"
        } else if player.is_paused() {
            "PAUSED "
        } else {
            "STOPPED"
        };
        let frame = player.current_frame();
        window.set_title(&format!("dotLottie Audio  [{state}]  frame {frame:.1}"));
        // Always update the window so the OS event loop runs and
        // keyboard / close events are processed every frame.
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        // Print audio (and notable playback) events as they arrive.
        while let Some(event) = player.poll_event() {
            let frame = player.current_frame();
            match event {
                DotLottieEvent::AudioPlay { ref ref_id, volume } => {
                    println!("  >> AudioPlay   ref_id={ref_id:<12}  frame={frame:.1}");
                }
                DotLottieEvent::AudioStop { ref ref_id } => {
                    println!("  >> AudioStop   ref_id={ref_id:<12}  frame={frame:.1}");
                }
                DotLottieEvent::AudioPause { ref ref_id } => {
                    println!("  >> AudioPause  ref_id={ref_id:<12}  frame={frame:.1}");
                }
                DotLottieEvent::Load => println!("  -- Load  (is_loaded={})", player.is_loaded()),
                DotLottieEvent::LoadError => {
                    eprintln!("  !! LoadError — animation failed to load into ThorVG");
                }
                DotLottieEvent::Play => println!("  -- Play"),
                DotLottieEvent::Pause => println!("  -- Pause"),
                DotLottieEvent::Stop => println!("  -- Stop"),
                DotLottieEvent::Loop { loop_count } => println!("  -- Loop #{loop_count}"),
                DotLottieEvent::Complete => println!("  -- Complete"),
                _ => {}
            }
        }
    }

    println!("\nBye.");
}
