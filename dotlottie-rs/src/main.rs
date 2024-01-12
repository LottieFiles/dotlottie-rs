use dotlottie_player_core::LottieRenderer;

fn main() {
    let mut renderer = LottieRenderer::new();

    let lottie_data = include_str!("./lottie_renderer/fixtures/lottie.json");

    println!("Loading data...");

    let result = renderer.load_data(lottie_data, 200, 200, true);

    println!("Result: {:?}", result);
}
