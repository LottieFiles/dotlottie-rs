use thiserror::Error;

mod tests;

use crate::thorvg;

#[derive(Error, Debug)]
pub enum LottieRendererError {
    #[error("Thorvg error: {0}")]
    ThorvgError(#[from] thorvg::TvgError),

    #[error("Animation not loaded")]
    AnimationNotLoaded,

    #[error("Invalid color: {0}")]
    InvalidColor(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

pub struct LottieRenderer {
    thorvg_animation: Option<thorvg::Animation>,
    thorvg_canvas: Option<thorvg::Canvas>,
    thorvg_background_shape: Option<thorvg::Shape>,
    thorvg_picture: Option<thorvg::Picture>,
    picture_width: f32,
    picture_height: f32,
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u32>,
    pub background_color: u32,
}

impl LottieRenderer {
    pub fn new() -> Self {
        Self {
            thorvg_animation: None,
            thorvg_canvas: None,
            thorvg_background_shape: None,
            thorvg_picture: None,
            buffer: vec![],
            width: 0,
            height: 0,
            picture_width: 0.0,
            picture_height: 0.0,
            background_color: 0,
        }
    }

    pub fn load_path(
        &mut self,
        path: &str,
        width: u32,
        height: u32,
    ) -> Result<(), LottieRendererError> {
        let thorvg_animation = thorvg::Animation::new();
        self.thorvg_picture = thorvg_animation.new_picture();
        self.thorvg_animation = Some(thorvg_animation);
        self.thorvg_background_shape = Some(thorvg::Shape::new());
        self.thorvg_canvas = Some(thorvg::Canvas::new(thorvg::TvgEngine::TvgEngineSw, 0));

        self.width = width;
        self.height = height;
        self.buffer
            .resize((self.width * self.height * 4) as usize, 0);

        let thorvg_canvas = self
            .thorvg_canvas
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        let thorvg_background_shape = self
            .thorvg_background_shape
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        thorvg_canvas
            .set_target(
                &mut self.buffer,
                width,
                width,
                height,
                thorvg::TvgColorspace::ABGR8888,
            )
            .map_err(LottieRendererError::ThorvgError)?;

        if let Some(picture) = &mut self.thorvg_picture {
            picture.load(path)?;

            let (pw, ph) = picture.get_size()?;
            let (scale, shift_x, shift_y) = calculate_scale_and_shift(pw, ph, width, height);

            picture.scale(scale)?;
            picture.translate(shift_x, shift_y)?;

            self.picture_width = pw;
            self.picture_height = ph;

            thorvg_background_shape.append_rect(0.0, 0.0, pw, ph, 0.0, 0.0)?;
            let (red, green, blue, alpha) = hex_to_rgba(self.background_color);
            thorvg_background_shape.fill((red, green, blue, alpha))?;

            thorvg_canvas.push(thorvg_background_shape)?;
            thorvg_canvas.push(picture)?;
        }

        Ok(())
    }

    pub fn load_data(
        &mut self,
        data: &str,
        width: u32,
        height: u32,
        copy: bool,
    ) -> Result<(), LottieRendererError> {
        let thorvg_animation = thorvg::Animation::new();
        self.thorvg_picture = thorvg_animation.new_picture();
        self.thorvg_animation = Some(thorvg_animation);
        self.thorvg_background_shape = Some(thorvg::Shape::new());
        self.thorvg_canvas = Some(thorvg::Canvas::new(thorvg::TvgEngine::TvgEngineSw, 0));

        self.width = width;
        self.height = height;
        self.buffer
            .resize((self.width * self.height * 4) as usize, 0);

        let thorvg_canvas = self
            .thorvg_canvas
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        let thorvg_background_shape = self
            .thorvg_background_shape
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        thorvg_canvas
            .set_target(
                &mut self.buffer,
                width,
                width,
                height,
                thorvg::TvgColorspace::ABGR8888,
            )
            .map_err(LottieRendererError::ThorvgError)?;

        if let Some(picture) = &mut self.thorvg_picture {
            picture.load_data(data, "lottie", copy)?;

            let (pw, ph) = picture.get_size()?;
            let (scale, shift_x, shift_y) = calculate_scale_and_shift(pw, ph, width, height);

            picture.scale(scale)?;
            picture.translate(shift_x, shift_y)?;

            self.picture_width = pw;
            self.picture_height = ph;

            thorvg_background_shape.append_rect(0.0, 0.0, pw, ph, 0.0, 0.0)?;
            let (red, green, blue, alpha) = hex_to_rgba(self.background_color);
            thorvg_background_shape.fill((red, green, blue, alpha))?;

            thorvg_canvas.push(thorvg_background_shape)?;
            thorvg_canvas.push(picture)?;
        }

        Ok(())
    }

    pub fn total_frames(&self) -> Result<f32, LottieRendererError> {
        let thorvg_animation = self
            .thorvg_animation
            .as_ref()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        thorvg_animation
            .get_total_frame()
            .map_err(|e| LottieRendererError::ThorvgError(e))
    }

    pub fn duration(&self) -> Result<f32, LottieRendererError> {
        let thorvg_animation = self
            .thorvg_animation
            .as_ref()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        thorvg_animation
            .get_duration()
            .map_err(|e| LottieRendererError::ThorvgError(e))
    }

    pub fn current_frame(&self) -> Result<f32, LottieRendererError> {
        let thorvg_animation = self
            .thorvg_animation
            .as_ref()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        thorvg_animation
            .get_frame()
            .map_err(|e| LottieRendererError::ThorvgError(e))
    }

    pub fn clear(&mut self) {
        self.buffer.clear()
    }

    pub fn render(&mut self) -> Result<(), LottieRendererError> {
        let thorvg_canvas = self
            .thorvg_canvas
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        thorvg_canvas.update()?;
        thorvg_canvas.draw()?;
        thorvg_canvas.sync()?;

        Ok(())
    }

    pub fn set_frame(&mut self, no: f32) -> Result<(), LottieRendererError> {
        let thorvg_animation = self
            .thorvg_animation
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        let total_frames = thorvg_animation
            .get_total_frame()
            .map_err(|e| LottieRendererError::ThorvgError(e))?;

        if no < 0.0 || no >= total_frames {
            return Err(LottieRendererError::InvalidArgument(format!(
                "Frame number must be between 0 and {}",
                total_frames - 1.0
            )));
        }

        thorvg_animation
            .set_frame(no)
            .map_err(|e| LottieRendererError::ThorvgError(e))
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), LottieRendererError> {
        let thorvg_canvas = self
            .thorvg_canvas
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

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

        thorvg_canvas
            .set_target(
                &mut self.buffer,
                self.width,
                self.width,
                self.height,
                thorvg::TvgColorspace::ABGR8888,
            )
            .map_err(LottieRendererError::ThorvgError)?;

        if let Some(picture) = &mut self.thorvg_picture {
            let (pw, ph) = picture.get_size()?;
            let (scale, shift_x, shift_y) = calculate_scale_and_shift(pw, ph, width, height);

            picture.scale(scale)?;
            picture.translate(shift_x, shift_y)?;
        }

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

        let thorvg_background_shape = self
            .thorvg_background_shape
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        let (red, green, blue, alpha) = hex_to_rgba(self.background_color);

        thorvg_background_shape
            .fill((red, green, blue, alpha))
            .map_err(|e| LottieRendererError::ThorvgError(e))
    }
}

fn calculate_scale_and_shift(
    picture_width: f32,
    picture_height: f32,
    width: u32,
    height: u32,
) -> (f32, f32, f32) {
    let scale = if picture_width > picture_height {
        width as f32 / picture_width
    } else {
        height as f32 / picture_height
    };

    let shift_x = (width as f32 - picture_width * scale) / 2.0;
    let shift_y = (height as f32 - picture_height * scale) / 2.0;

    (scale, shift_x, shift_y)
}

fn hex_to_rgba(hex_color: u32) -> (u8, u8, u8, u8) {
    let red = ((hex_color >> 24) & 0xFF) as u8;
    let green = ((hex_color >> 16) & 0xFF) as u8;
    let blue = ((hex_color >> 8) & 0xFF) as u8;
    let alpha = (hex_color & 0xFF) as u8;

    (red, green, blue, alpha)
}
