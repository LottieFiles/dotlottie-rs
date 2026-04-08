//! Audio feature demonstration.
//!
//! Loads a Lottie animation (JSON or .lottie) that contains audio layers and
//! plays it. Audio events are printed to stdout as they fire so you can verify
//! the correct frame boundaries. Audio is played via rodio on native targets.
//!
//! Usage:
//!   cargo run --example audio_player --features audio,dev -- path/to/animation.json
//!   cargo run --example audio_player --features audio,dev -- path/to/animation.lottie
//!
//! Controls:
//!   P       – play / resume
//!   S       – pause
//!   X       – stop
//!   M       – toggle mute
//!   +/=     – volume up   (+10%)
//!   -       – volume down (-10%)
//!   ESC     – quit

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
        "dotLottie Audio — P=play  S=pause  X=stop  M=mute  +/-=volume  ESC=quit",
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
    println!(
        "\nAnimation: {:.0} frames  ({:.2} s)",
        player.total_frames(),
        player.duration(),
    );
    println!("\nControls:");
    println!("  P     – play / resume");
    println!("  S     – pause");
    println!("  X     – stop");
    println!("  M     – toggle mute");
    println!("  +/=   – volume up   (+10%)");
    println!("  -     – volume down (-10%)");
    println!("  ESC   – quit");
    println!("\n(press P to start)\n");

    // -------------------------------------------------------------------------
    // Render loop
    // -------------------------------------------------------------------------
    let mut p_was_down = false;
    let mut s_was_down = false;
    let mut x_was_down = false;
    let mut m_was_down = false;
    let mut plus_was_down = false;
    let mut minus_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let p_down = window.is_key_down(Key::P);
        let s_down = window.is_key_down(Key::S);
        let x_down = window.is_key_down(Key::X);
        let m_down = window.is_key_down(Key::M);
        let plus_down = window.is_key_down(Key::Equal) || window.is_key_down(Key::NumPadPlus);
        let minus_down = window.is_key_down(Key::Minus) || window.is_key_down(Key::NumPadMinus);

        if p_was_down && !p_down && (player.is_paused() || player.is_stopped()) {
            let _ = player.play();
        }
        if s_was_down && !s_down && player.is_playing() {
            let _ = player.pause();
        }
        if x_was_down && !x_down && !player.is_stopped() {
            let _ = player.stop();
        }

        if m_was_down && !m_down {
            if player.audio_volume() == 0.0 {
                player.set_audio_volume(1.0);
                println!(
                    "  ** Unmuted  (volume={:.0}%)",
                    player.audio_volume() * 100.0
                );
            } else {
                player.set_audio_volume(0.0);
                println!("  ** Muted");
            }
        }

        if plus_was_down && !plus_down {
            let new_vol = (player.audio_volume() + 0.1).min(1.0);
            player.set_audio_volume(new_vol);
            println!("  ** Volume → {:.0}%", player.audio_volume() * 100.0);
        }
        if minus_was_down && !minus_down {
            let new_vol = (player.audio_volume() - 0.1).max(0.0);
            player.set_audio_volume(new_vol);
            println!("  ** Volume → {:.0}%", player.audio_volume() * 100.0);
        }

        p_was_down = p_down;
        s_was_down = s_down;
        x_was_down = x_down;
        m_was_down = m_down;
        plus_was_down = plus_down;
        minus_was_down = minus_down;

        let _ = player.tick();

        let mute_indicator = if player.audio_volume() == 0.0 {
            " [MUTED]"
        } else {
            ""
        };
        let state = if player.is_playing() {
            "PLAYING"
        } else if player.is_paused() {
            "PAUSED "
        } else {
            "STOPPED"
        };
        let frame = player.current_frame();
        let vol = player.audio_volume();
        window.set_title(&format!(
            "dotLottie Audio  [{state}]  frame {frame:.1}  vol {:.0}%{mute_indicator}",
            vol * 100.0,
        ));

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        // Print audio (and notable playback) events as they arrive.
        while let Some(event) = player.poll_event() {
            let _ = player.current_frame();
            match event {
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
