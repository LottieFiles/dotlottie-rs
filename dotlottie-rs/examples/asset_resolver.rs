#![allow(clippy::print_stdout)]

use dotlottie_rs::{AssetResolver, ColorSpace, DotLottiePlayer, ResolvedAsset};
use minifb::{Key, Window, WindowOptions};
use std::ffi::CString;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

struct DemoAssetResolver;

impl AssetResolver for DemoAssetResolver {
    fn resolve(&self, src: &str) -> Option<ResolvedAsset> {
        println!("[resolver] asked to resolve: {src:?}");

        if src.starts_with("name:") || src.ends_with(".ttf") || src.ends_with(".otf") {
            let font_name = src.strip_prefix("name:").unwrap_or(src);
            println!("[resolver] font request for {font_name:?}");

            let font_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets/fonts/PressStart2P.ttf");

            match std::fs::read(&font_path) {
                Ok(data) => {
                    println!("[resolver] loaded PressStart2P.ttf ({} bytes)", data.len());
                    return Some(ResolvedAsset {
                        data,
                        mimetype: "ttf".to_string(),
                    });
                }
                Err(e) => {
                    println!("[resolver] failed to read {}: {e}", font_path.display());
                    return None;
                }
            }
        }

        if src.ends_with(".png") || src.ends_with(".jpg") || src.ends_with(".webp") {
            let image_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets/images/logo.png");

            match std::fs::read(&image_path) {
                Ok(data) => {
                    println!("[resolver] loaded logo.png ({} bytes)", data.len());
                    return Some(ResolvedAsset {
                        data,
                        mimetype: "png".to_string(),
                    });
                }
                Err(e) => {
                    println!("[resolver] failed to read {}: {e}", image_path.display());
                    return None;
                }
            }
        }

        println!("[resolver] unrecognised asset — returning None");
        None
    }
}

const LOTTIE_JSON_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/animations/lottie/asset_resolver.json");

fn main() {
    let mut window = Window::new(
        "Asset Resolver Example — ESC to exit",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut player = DotLottiePlayer::new();
    player.set_autoplay(true);
    player.set_loop(true);
    let _ = player.set_background_color(Some(0xFFFFFFFF));

    player.set_asset_resolver(DemoAssetResolver);

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    player
        .set_sw_target(&mut buffer, WIDTH, HEIGHT, ColorSpace::ABGR8888S)
        .expect("Failed to set render target");

    let json = std::fs::read_to_string(LOTTIE_JSON_PATH)
        .expect("Failed to read asset_resolver.json");
    let c_data = CString::new(json).expect("CString conversion failed");

    match player.load_animation_data(&c_data, WIDTH, HEIGHT) {
        Ok(()) => println!("Animation loaded (check console for resolver calls)"),
        Err(e) => {
            eprintln!("Failed to load animation: {e:?}");
            return;
        }
    }

    println!("Press ESC to quit");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if player.tick().is_ok() {
            window
                .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
                .expect("Failed to update window");
        }
    }

    println!("Done!");
}
