#[cfg(test)]
mod tests {

    use crate::lottie_renderer::LottieRenderer;
    use crate::lottie_renderer::LottieRendererError;

    #[test]
    fn test_new_lottie_renderer() {
        let renderer = LottieRenderer::new();

        assert_eq!(renderer.width, 0);
        assert_eq!(renderer.height, 0);

        assert_eq!(renderer.buffer.len(), 0);
    }

    #[test]
    fn test_load_data() {
        let mut renderer = LottieRenderer::new();

        let lottie_data = include_str!("./fixtures/lottie.json");

        let result = renderer.load_data(lottie_data, 100, 100, true);

        assert!(result.is_ok());
    }

    #[test]
    fn test_total_frames() {
        let mut renderer = LottieRenderer::new();

        let lottie_data = include_str!("./fixtures/lottie.json");

        let result = renderer.load_data(lottie_data, 100, 100, true);

        assert!(result.is_ok());

        let result = renderer.total_frames();

        assert!(result.is_ok());

        let total_frames = result.unwrap();

        assert!(total_frames > 0.0);
    }

    #[test]
    fn test_duration() {
        let mut renderer = LottieRenderer::new();

        let lottie_data = include_str!("./fixtures/lottie.json");

        let result = renderer.load_data(lottie_data, 100, 100, true);

        assert!(result.is_ok());

        let result = renderer.duration();

        assert!(result.is_ok());

        let duration = result.unwrap();

        assert!(duration > 0.0);
    }

    #[test]
    fn test_current_frame() {
        let mut renderer = LottieRenderer::new();

        let lottie_data = include_str!("./fixtures/lottie.json");

        let result = renderer.load_data(lottie_data, 100, 100, true);

        assert!(result.is_ok());

        let result = renderer.current_frame();

        assert!(result.is_ok());

        let current_frame = result.unwrap();

        assert_eq!(current_frame, 0.0);
    }

    #[test]
    fn test_clear() {
        let mut renderer = LottieRenderer::new();

        let lottie_data = include_str!("./fixtures/lottie.json");

        let result = renderer.load_data(lottie_data, 100, 100, true);

        assert!(result.is_ok());

        renderer.clear();

        assert_eq!(renderer.buffer.len(), 0);
    }

    #[test]
    fn test_render() {
        let mut renderer = LottieRenderer::new();

        let lottie_data = include_str!("./fixtures/lottie.json");

        let result = renderer.load_data(lottie_data, 100, 100, true);

        assert!(result.is_ok());

        let result = renderer.render();

        assert!(result.is_ok());

        let buffer = renderer.buffer;

        assert!(buffer.len() > 0);

        let mut has_non_zero = false;

        for pixel in buffer {
            if pixel != 0 {
                has_non_zero = true;
                break;
            }
        }

        assert!(has_non_zero);
    }

    #[test]
    fn test_set_frame() {
        let mut renderer = LottieRenderer::new();

        let lottie_data = include_str!("./fixtures/lottie.json");

        let result = renderer.load_data(lottie_data, 100, 100, true);

        assert!(result.is_ok());

        let result = renderer.set_frame(10.0);

        assert!(result.is_ok());

        let result = renderer.current_frame();

        assert!(result.is_ok());

        let current_frame = result.unwrap();

        assert_eq!(current_frame, 10.0);
    }

    #[test]
    fn test_resize() {
        let mut renderer = LottieRenderer::new();

        let lottie_data = include_str!("./fixtures/lottie.json");

        let result = renderer.load_data(lottie_data, 100, 100, true);

        assert!(result.is_ok());

        let result = renderer.resize(400, 400);

        assert!(result.is_ok());
    }
}
