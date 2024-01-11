use thiserror::Error;

mod tests;

use crate::thorvg;

#[derive(Error, Debug)]
pub enum LottieRendererError {
    #[error("Thorvg error: {0}")]
    ThorvgError(#[from] thorvg::TvgError),

    #[error("Animation not loaded")]
    AnimationNotLoaded,
}

pub struct LottieRenderer {
    thorvg_animation: Option<thorvg::Animation>,
    thorvg_canvas: Option<thorvg::Canvas>,
    picture_width: f32,
    picture_height: f32,
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u32>,
}

impl LottieRenderer {
    pub fn new() -> Self {
        Self {
            thorvg_animation: None,
            thorvg_canvas: None,
            buffer: vec![],
            width: 0,
            height: 0,
            picture_width: 0.0,
            picture_height: 0.0,
        }
    }

    pub fn load_data(
        &mut self,
        data: &str,
        width: u32,
        height: u32,
        copy: bool,
    ) -> Result<(), LottieRendererError> {
        self.thorvg_animation = Some(thorvg::Animation::new());
        self.thorvg_canvas = Some(thorvg::Canvas::new(thorvg::TvgEngine::TvgEngineSw, 0));
        self.buffer = vec![0; (width * height * 4) as usize];
        self.width = width;
        self.height = height;

        let thorvg_animation = self
            .thorvg_animation
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        let thorvg_canvas = self
            .thorvg_canvas
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

        if let Some(picture) = &mut thorvg_animation.get_picture() {
            picture.load_data(data.as_bytes(), "lottie", copy)?;

            thorvg_canvas.push(picture)?;

            let (pw, ph) = picture.get_size()?;
            let (scale, shift_x, shift_y) = calculate_scale_and_shift(pw, ph, width, height);

            picture.scale(scale)?;
            picture.translate(shift_x, shift_y)?;

            self.picture_width = pw;
            self.picture_height = ph;
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

    pub fn clear(&mut self, paints: bool, buffer: bool) -> Result<(), LottieRendererError> {
        let thorvg_canvas = self
            .thorvg_canvas
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        thorvg_canvas
            .clear(paints, buffer)
            .map_err(|e| LottieRendererError::ThorvgError(e))?;

        Ok(())
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

        thorvg_animation
            .set_frame(no)
            .map_err(|e| LottieRendererError::ThorvgError(e))
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), LottieRendererError> {
        let thorvg_canvas = self
            .thorvg_canvas
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        let thorvg_animation = self
            .thorvg_animation
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        if (width, height) == (self.width, self.height) {
            return Ok(());
        }

        let mut buffer = vec![0; (width * height * 4) as usize];
        thorvg_canvas
            .set_target(
                &mut buffer,
                width,
                width,
                height,
                thorvg::TvgColorspace::ABGR8888,
            )
            .map_err(LottieRendererError::ThorvgError)?;

        if let Some(picture) = &mut thorvg_animation.get_picture() {
            let (pw, ph) = picture.get_size()?;
            let (scale, shift_x, shift_y) = calculate_scale_and_shift(pw, ph, width, height);

            picture.scale(scale)?;
            picture.translate(shift_x, shift_y)?;

            self.buffer = buffer;
        }

        Ok(())
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
