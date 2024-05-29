use thiserror::Error;

use crate::{Animation, Canvas, Layout, Shape, TvgColorspace, TvgEngine, TvgError};

#[derive(Error, Debug)]
pub enum LottieRendererError {
    #[error("Thorvg error: {0}")]
    ThorvgError(#[from] TvgError),

    #[error("Invalid color: {0}")]
    InvalidColor(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

pub struct LottieRenderer {
    thorvg_animation: Animation,
    thorvg_canvas: Canvas,
    thorvg_background_shape: Shape,
    picture_width: f32,
    picture_height: f32,
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u32>,
    pub background_color: u32,
    pub current_frame: f32,
    layout: Layout,
}

impl LottieRenderer {
    pub fn new() -> Self {
        let mut thorvg_canvas = Canvas::new(TvgEngine::TvgEngineSw, 0);

        let thorvg_animation = Animation::new();
        let thorvg_background_shape = Shape::new();

        thorvg_canvas.push(&thorvg_background_shape).unwrap();
        thorvg_canvas.push(&thorvg_animation).unwrap();

        Self {
            thorvg_animation,
            thorvg_canvas,
            thorvg_background_shape,
            buffer: vec![],
            width: 0,
            height: 0,
            picture_width: 0.0,
            picture_height: 0.0,
            background_color: 0,
            current_frame: 0.0,
            layout: Layout::default(),
        }
    }

    pub fn load_data(
        &mut self,
        data: &str,
        width: u32,
        height: u32,
        copy: bool,
    ) -> Result<(), LottieRendererError> {
        self.thorvg_canvas.clear(true)?;

        self.width = width;
        self.height = height;

        self.buffer
            .resize((self.width * self.height * 4) as usize, 0);
        self.thorvg_canvas
            .set_target(
                &mut self.buffer,
                self.width,
                self.width,
                self.height,
                get_color_space_for_target(),
            )
            .map_err(LottieRendererError::ThorvgError)?;

        self.thorvg_animation = Animation::new();
        self.thorvg_background_shape = Shape::new();

        self.thorvg_animation.load_data(data, "lottie", copy)?;

        let (pw, ph) = self.thorvg_animation.get_size()?;
        self.picture_width = pw;
        self.picture_height = ph;

        let (scaled_picture_width, scaled_picture_height, shift_x, shift_y) =
            self.layout.compute_layout_transform(
                self.width as f32,
                self.height as f32,
                self.picture_width,
                self.picture_height,
            );

        self.thorvg_animation
            .set_size(scaled_picture_width, scaled_picture_height)?;
        self.thorvg_animation.translate(shift_x, shift_y)?;

        self.thorvg_background_shape.append_rect(
            0.0,
            0.0,
            self.width as f32,
            self.height as f32,
            0.0,
            0.0,
        )?;
        let (red, green, blue, alpha) = hex_to_rgba(self.background_color);
        self.thorvg_background_shape
            .fill((red, green, blue, alpha))?;

        self.thorvg_canvas.push(&self.thorvg_background_shape)?;
        self.thorvg_canvas.push(&self.thorvg_animation)?;

        Ok(())
    }

    pub fn total_frames(&self) -> Result<f32, LottieRendererError> {
        self.thorvg_animation
            .get_total_frame()
            .map_err(|e| LottieRendererError::ThorvgError(e))
    }

    pub fn duration(&self) -> Result<f32, LottieRendererError> {
        self.thorvg_animation
            .get_duration()
            .map_err(|e| LottieRendererError::ThorvgError(e))
    }

    pub fn current_frame(&self) -> f32 {
        self.current_frame
    }

    pub fn clear(&mut self) {
        self.buffer.clear()
    }

    pub fn render(&mut self) -> Result<(), LottieRendererError> {
        self.thorvg_canvas.update()?;
        self.thorvg_canvas.draw()?;
        self.thorvg_canvas.sync()?;

        Ok(())
    }

    pub fn set_viewport(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) -> Result<(), LottieRendererError> {
        self.thorvg_canvas
            .set_viewport(x, y, w, h)
            .map_err(LottieRendererError::ThorvgError)
    }

    pub fn set_frame(&mut self, no: f32) -> Result<(), LottieRendererError> {
        let total_frames = self
            .thorvg_animation
            .get_total_frame()
            .map_err(|e| LottieRendererError::ThorvgError(e))?;

        if no < 0.0 || no >= total_frames {
            return Err(LottieRendererError::InvalidArgument(format!(
                "Frame number must be between 0 and {}",
                total_frames - 1.0
            )));
        }

        self.thorvg_animation
            .set_frame(no)
            .map_err(LottieRendererError::ThorvgError)?;

        self.current_frame = no;

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), LottieRendererError> {
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }

        if width <= 0 || height <= 0 {
            return Err(LottieRendererError::InvalidArgument(
                "Width and height must be greater than 0".to_string(),
            ));
        }

        self.width = width;
        self.height = height;

        self.buffer
            .resize((self.width * self.height * 4) as usize, 0);

        self.thorvg_canvas
            .set_target(
                &mut self.buffer,
                self.width,
                self.width,
                self.height,
                get_color_space_for_target(),
            )
            .map_err(LottieRendererError::ThorvgError)?;

        let (scaled_picture_width, scaled_picture_height, shift_x, shift_y) =
            self.layout.compute_layout_transform(
                self.width as f32,
                self.height as f32,
                self.picture_width,
                self.picture_height,
            );

        self.thorvg_animation
            .set_size(scaled_picture_width, scaled_picture_height)?;
        self.thorvg_animation.translate(shift_x, shift_y)?;

        self.thorvg_background_shape.append_rect(
            0.0,
            0.0,
            self.width as f32,
            self.height as f32,
            0.0,
            0.0,
        )?;

        Ok(())
    }

    pub fn buffer_ptr(&self) -> *const u32 {
        self.buffer.as_ptr()
    }

    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn set_background_color(&mut self, hex_color: u32) -> Result<(), LottieRendererError> {
        self.background_color = hex_color;

        let (red, green, blue, alpha) = hex_to_rgba(self.background_color);

        self.thorvg_background_shape
            .fill((red, green, blue, alpha))
            .map_err(|e| LottieRendererError::ThorvgError(e))
    }

    pub fn load_theme_data(&mut self, slots: &str) -> Result<(), LottieRendererError> {
        self.thorvg_animation
            .set_slots(slots)
            .map_err(|e| LottieRendererError::ThorvgError(e))
    }

    pub fn set_layout(&mut self, layout: &Layout) -> Result<(), LottieRendererError> {
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

        self.thorvg_animation
            .set_size(scaled_picture_width, scaled_picture_height)?;
        self.thorvg_animation.translate(shift_x, shift_y)?;

        Ok(())
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
fn get_color_space_for_target() -> TvgColorspace {
    #[cfg(target_arch = "wasm32")]
    {
        TvgColorspace::ABGR8888S
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        TvgColorspace::ABGR8888
    }
}
