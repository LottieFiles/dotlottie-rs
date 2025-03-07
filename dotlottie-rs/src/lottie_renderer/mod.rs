use std::error::Error;

use thiserror::Error;

use crate::Layout;

mod renderer;
#[cfg(any(feature = "thorvg-v0", feature = "thorvg-v1"))]
mod thorvg;

pub use renderer::{Animation, ColorSpace, Drawable, Renderer, Shape};
#[cfg(any(feature = "thorvg-v0", feature = "thorvg-v1"))]
pub use thorvg::{TvgAnimation, TvgEngine, TvgError, TvgRenderer, TvgShape};

#[derive(Error, Debug)]
pub enum LottieRendererError {
    #[error("Renderer error: {0}")]
    RendererError(Box<dyn Error>),

    #[error("Invalid color: {0}")]
    InvalidColor(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

fn into_lottie<R: Renderer>(err: R::Error) -> LottieRendererError {
    LottieRendererError::RendererError(Box::new(err))
}

pub trait LottieRenderer {
    fn load_data(
        &mut self,
        data: &str,
        width: u32,
        height: u32,
        copy: bool,
    ) -> Result<(), LottieRendererError>;

    fn picture_width(&self) -> f32;

    fn picture_height(&self) -> f32;

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn total_frames(&self) -> Result<f32, LottieRendererError>;

    fn duration(&self) -> Result<f32, LottieRendererError>;

    fn current_frame(&self) -> f32;

    fn buffer(&self) -> &[u32];

    fn clear(&mut self);

    fn render(&mut self) -> Result<(), LottieRendererError>;

    fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> Result<(), LottieRendererError>;

    fn set_frame(&mut self, no: f32) -> Result<(), LottieRendererError>;

    fn resize(&mut self, width: u32, height: u32) -> Result<(), LottieRendererError>;

    fn buffer_ptr(&self) -> *const u32;

    fn buffer_len(&self) -> usize;

    fn set_background_color(&mut self, hex_color: u32) -> Result<(), LottieRendererError>;

    fn set_slots(&mut self, slots: &str) -> Result<(), LottieRendererError>;

    fn set_layout(&mut self, layout: &Layout) -> Result<(), LottieRendererError>;

    fn hit_check(&self, layer_name: &str, x: f32, y: f32) -> Result<bool, LottieRendererError>;

    fn get_layer_bounds(
        &self,
        layer_name: &str,
    ) -> Result<(f32, f32, f32, f32), LottieRendererError>;

    fn tween(
        &mut self,
        to: f32,
        duration: Option<f32>,
        easing: Option<[f32; 4]>,
    ) -> Result<(), LottieRendererError>;

    fn is_tweening(&self) -> bool;

    fn tween_update(&mut self, progress: Option<f32>) -> Result<bool, LottieRendererError>;

    fn tween_stop(&mut self) -> Result<(), LottieRendererError>;
}

impl dyn LottieRenderer {
    pub fn new<R: Renderer>(renderer: R) -> Box<Self> {
        let mut renderer = renderer;
        let background_shape = R::Shape::default();

        renderer.push(Drawable::Shape(&background_shape)).unwrap();
        renderer.sync().unwrap();

        Box::new(LottieRendererImpl {
            animation: R::Animation::default(),
            background_shape,
            renderer,
            buffer: vec![],
            width: 0,
            height: 0,
            picture_width: 0.0,
            picture_height: 0.0,
            background_color: 0,
            current_frame: 0.0,
            layout: Layout::default(),
        })
    }
}

#[derive(Default)]
struct LottieRendererImpl<R: Renderer> {
    animation: R::Animation,
    background_shape: R::Shape,
    renderer: R,
    picture_width: f32,
    picture_height: f32,
    width: u32,
    height: u32,
    buffer: Vec<u32>,
    background_color: u32,
    current_frame: f32,
    layout: Layout,
}

impl<R: Renderer> LottieRenderer for LottieRendererImpl<R> {
    fn load_data(
        &mut self,
        data: &str,
        width: u32,
        height: u32,
        copy: bool,
    ) -> Result<(), LottieRendererError> {
        self.renderer.clear(true).map_err(into_lottie::<R>)?;

        self.picture_width = 0.0;
        self.picture_height = 0.0;

        self.width = width;
        self.height = height;

        self.buffer
            .resize((self.width * self.height * 4) as usize, 0);
        self.renderer
            .set_target(
                &mut self.buffer,
                self.width,
                self.width,
                self.height,
                get_color_space_for_target(),
            )
            .map_err(into_lottie::<R>)?;

        self.animation = R::Animation::default();
        self.background_shape = R::Shape::default();

        self.animation
            .load_data(data, "lottie", copy)
            .map_err(into_lottie::<R>)?;

        let (pw, ph) = self.animation.get_size().map_err(into_lottie::<R>)?;
        self.picture_width = pw;
        self.picture_height = ph;

        let (scaled_picture_width, scaled_picture_height, shift_x, shift_y) =
            self.layout.compute_layout_transform(
                self.width as f32,
                self.height as f32,
                self.picture_width,
                self.picture_height,
            );

        self.animation
            .set_size(scaled_picture_width, scaled_picture_height)
            .map_err(into_lottie::<R>)?;
        self.animation
            .translate(shift_x, shift_y)
            .map_err(into_lottie::<R>)?;

        self.background_shape
            .append_rect(0.0, 0.0, self.width as f32, self.height as f32, 0.0, 0.0)
            .map_err(into_lottie::<R>)?;
        let (red, green, blue, alpha) = hex_to_rgba(self.background_color);
        self.background_shape
            .fill((red, green, blue, alpha))
            .map_err(into_lottie::<R>)?;

        self.renderer
            .push(Drawable::Shape(&self.background_shape))
            .map_err(into_lottie::<R>)?;
        self.renderer
            .push(Drawable::Animation(&self.animation))
            .map_err(into_lottie::<R>)?;

        Ok(())
    }

    fn picture_width(&self) -> f32 {
        self.picture_width
    }

    fn picture_height(&self) -> f32 {
        self.picture_height
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn total_frames(&self) -> Result<f32, LottieRendererError> {
        self.animation.get_total_frame().map_err(into_lottie::<R>)
    }

    fn duration(&self) -> Result<f32, LottieRendererError> {
        self.animation.get_duration().map_err(into_lottie::<R>)
    }

    fn current_frame(&self) -> f32 {
        self.current_frame
    }

    fn buffer(&self) -> &[u32] {
        &self.buffer
    }

    fn clear(&mut self) {
        self.buffer.clear()
    }

    fn render(&mut self) -> Result<(), LottieRendererError> {
        self.renderer.update().map_err(into_lottie::<R>)?;
        self.renderer.draw(true).map_err(into_lottie::<R>)?;
        self.renderer.sync().map_err(into_lottie::<R>)?;

        Ok(())
    }

    fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> Result<(), LottieRendererError> {
        self.renderer
            .set_viewport(x, y, w, h)
            .map_err(into_lottie::<R>)
    }

    fn set_frame(&mut self, no: f32) -> Result<(), LottieRendererError> {
        let total_frames = self.animation.get_total_frame().map_err(into_lottie::<R>)?;

        if no < 0.0 || no >= total_frames {
            return Err(LottieRendererError::InvalidArgument(format!(
                "Frame number must be between 0 and {}",
                total_frames - 1.0
            )));
        }

        self.animation.set_frame(no).map_err(into_lottie::<R>)?;

        self.current_frame = no;

        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), LottieRendererError> {
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }

        if width == 0 || height == 0 {
            return Err(LottieRendererError::InvalidArgument(
                "Width and height must be greater than 0".to_string(),
            ));
        }

        let _ = self.renderer.sync();

        self.width = width;
        self.height = height;

        self.buffer
            .resize((self.width * self.height * 4) as usize, 0);

        self.renderer
            .set_target(
                &mut self.buffer,
                self.width,
                self.width,
                self.height,
                get_color_space_for_target(),
            )
            .map_err(into_lottie::<R>)?;

        let (scaled_picture_width, scaled_picture_height, shift_x, shift_y) =
            self.layout.compute_layout_transform(
                self.width as f32,
                self.height as f32,
                self.picture_width,
                self.picture_height,
            );

        self.animation
            .set_size(scaled_picture_width, scaled_picture_height)
            .map_err(into_lottie::<R>)?;
        self.animation
            .translate(shift_x, shift_y)
            .map_err(into_lottie::<R>)?;

        self.background_shape
            .append_rect(0.0, 0.0, self.width as f32, self.height as f32, 0.0, 0.0)
            .map_err(into_lottie::<R>)?;

        Ok(())
    }

    fn buffer_ptr(&self) -> *const u32 {
        self.buffer.as_ptr()
    }

    fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    fn set_background_color(&mut self, hex_color: u32) -> Result<(), LottieRendererError> {
        self.background_color = hex_color;

        let (red, green, blue, alpha) = hex_to_rgba(self.background_color);

        self.background_shape
            .fill((red, green, blue, alpha))
            .map_err(into_lottie::<R>)
    }

    fn set_slots(&mut self, slots: &str) -> Result<(), LottieRendererError> {
        self.animation.set_slots(slots).map_err(into_lottie::<R>)
    }

    fn tween(
        &mut self,
        to: f32,
        duration: Option<f32>,
        easing: Option<[f32; 4]>,
    ) -> Result<(), LottieRendererError> {
        self.animation
            .tween(to, duration, easing)
            .map_err(into_lottie::<R>)
    }

    fn is_tweening(&self) -> bool {
        self.animation.is_tweening()
    }

    fn tween_update(&mut self, progress: Option<f32>) -> Result<bool, LottieRendererError> {
        self.animation
            .tween_update(progress)
            .map_err(into_lottie::<R>)
    }

    fn tween_stop(&mut self) -> Result<(), LottieRendererError> {
        self.animation.tween_stop().map_err(into_lottie::<R>)
    }

    fn set_layout(&mut self, layout: &Layout) -> Result<(), LottieRendererError> {
        if self.layout == *layout {
            return Ok(());
        }

        self.layout = layout.clone();

        let (scaled_picture_width, scaled_picture_height, shift_x, shift_y) =
            self.layout.compute_layout_transform(
                self.width as f32,
                self.height as f32,
                self.picture_width,
                self.picture_height,
            );

        self.animation
            .set_size(scaled_picture_width, scaled_picture_height)
            .map_err(into_lottie::<R>)?;
        self.animation
            .translate(shift_x, shift_y)
            .map_err(into_lottie::<R>)?;

        Ok(())
    }

    fn hit_check(&self, layer_name: &str, x: f32, y: f32) -> Result<bool, LottieRendererError> {
        self.animation
            .hit_check(layer_name, x, y)
            .map_err(into_lottie::<R>)
    }

    fn get_layer_bounds(
        &self,
        layer_name: &str,
    ) -> Result<(f32, f32, f32, f32), LottieRendererError> {
        self.animation
            .get_layer_bounds(layer_name)
            .map_err(into_lottie::<R>)
    }
}

fn hex_to_rgba(hex_color: u32) -> (u8, u8, u8, u8) {
    let red = ((hex_color >> 24) & 0xFF) as u8;
    let green = ((hex_color >> 16) & 0xFF) as u8;
    let blue = ((hex_color >> 8) & 0xFF) as u8;
    let alpha = (hex_color & 0xFF) as u8;

    (red, green, blue, alpha)
}

#[inline]
fn get_color_space_for_target() -> ColorSpace {
    #[cfg(target_arch = "wasm32")]
    {
        ColorSpace::ABGR8888S
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        ColorSpace::ABGR8888
    }
}
