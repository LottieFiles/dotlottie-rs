use dotlottie_rs::{ColorSpace, Player};
use std::ffi::CString;

static RED_BLUE_GIF: &[u8] = include_bytes!("../assets/images/red_blue.gif");
const GIF_EXTERNAL: &str = include_str!("../assets/animations/lottie/gif_external.json");
const GIF_EMBEDDED: &str = include_str!("../assets/animations/lottie/gif_embedded.json");

const SIZE: u32 = 100;
// The GIF's first frame is red.
const RED: u32 = 0xff0000ff;

fn center(json: &str) -> u32 {
    let json = CString::new(json).unwrap();
    let mut player = Player::new();
    let mut buffer = vec![0u32; (SIZE * SIZE) as usize];
    player
        .set_sw_target(&mut buffer, SIZE, SIZE, SIZE, ColorSpace::ABGR8888)
        .unwrap();
    player.set_asset_resolver(|src: &str| {
        src.ends_with("red_blue.gif").then(|| RED_BLUE_GIF.to_vec())
    });
    player.load_animation_data(&json).unwrap();
    let _ = player.set_frame(0.0);
    let _ = player.render();
    buffer[(SIZE * SIZE / 2 + SIZE / 2) as usize]
}

#[test]
fn gif_asset_from_resolver_renders() {
    assert_eq!(center(GIF_EXTERNAL), RED);
}

#[test]
fn embedded_gif_asset_renders() {
    assert_eq!(center(GIF_EMBEDDED), RED);
}
