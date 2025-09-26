use crate::Layout;

mod renderer;
#[cfg(any(feature = "tvg-v0", feature = "tvg-v1"))]
mod thorvg;

pub use renderer::{Animation, ColorSpace, Drawable, Renderer, Shape};
#[cfg(any(feature = "tvg-v0", feature = "tvg-v1"))]
pub use thorvg::{TvgAnimation, TvgEngine, TvgError, TvgRenderer, TvgShape};

use std::{error::Error, fmt};

#[derive(Debug)]
pub enum LottieRendererError {
    RendererError,
    InvalidColor,
    InvalidArgument,
    AnimationNotLoaded,
    BackgroundShapeNotInitialized,
}

impl fmt::Display for LottieRendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for LottieRendererError {}

#[inline]
fn into_lottie<R: Renderer>(_err: R::Error) -> LottieRendererError {
    LottieRendererError::RendererError
}

pub trait LottieRenderer {
    fn load_data(&mut self, data: &str, width: u32, height: u32)
        -> Result<(), LottieRendererError>;

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

    fn get_layer_bounds(&self, layer_name: &str) -> Result<[f32; 8], LottieRendererError>;

    fn intersect(&self, x: f32, y: f32, layer_name: &str) -> Result<bool, LottieRendererError>;

    fn updated(&self) -> bool;

    fn tween(
        &mut self,
        to: f32,
        duration: Option<f32>,
        easing: Option<[f32; 4]>,
    ) -> Result<(), LottieRendererError>;

    fn is_tweening(&self) -> bool;

    fn tween_update(&mut self, progress: Option<f32>) -> Result<bool, LottieRendererError>;

    fn tween_stop(&mut self) -> Result<(), LottieRendererError>;

    fn get_transform(&self) -> Result<[f32; 9], LottieRendererError>;

    fn set_transform(&mut self, transform: &[f32; 9]) -> Result<(), LottieRendererError>;
}

impl dyn LottieRenderer {
    pub fn new<R: Renderer>(renderer: R) -> Box<Self> {
        Box::new(LottieRendererImpl {
            animation: None,
            background_shape: None,
            renderer,
            width: 0,
            height: 0,
            picture_width: 0.0,
            picture_height: 0.0,
            current_frame: 0.0,
            updated: false,
            background_color: 0,
            buffer: vec![],
            layout: Layout::default(),
            user_transform: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        })
    }
}

#[derive(Default)]
struct LottieRendererImpl<R: Renderer> {
    animation: Option<R::Animation>,
    background_shape: Option<R::Shape>,
    renderer: R,
    width: u32,
    height: u32,
    picture_width: f32,
    picture_height: f32,
    current_frame: f32,
    updated: bool,
    background_color: u32,
    buffer: Vec<u32>,
    layout: Layout,
    user_transform: [f32; 9],
}

impl<R: Renderer> LottieRendererImpl<R> {
    fn clear(&mut self) -> Result<(), LottieRendererError> {
        if self.animation.is_some() || self.background_shape.is_some() {
            self.renderer.clear(true).map_err(into_lottie::<R>)?;
            self.animation = None;
            self.background_shape = None;
        }
        Ok(())
    }

    fn resize_buffer(&mut self, width: u32, height: u32) -> Result<(), LottieRendererError> {
        let buffer_size = (width as u64)
            .checked_mul(height as u64)
            .ok_or(LottieRendererError::InvalidArgument)? as usize;

        self.buffer = vec![0; buffer_size];

        Ok(())
    }

    fn setup_buffer_and_target(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<(), LottieRendererError> {
        if self.width == width && self.height == height && !self.buffer.is_empty() {
            return Ok(());
        }

        let _ = self.renderer.sync();

        self.picture_width = 0.0;
        self.picture_height = 0.0;
        self.width = width;
        self.height = height;

        self.resize_buffer(width, height)?;

        self.renderer
            .set_target(
                &mut self.buffer,
                self.width,
                self.width,
                self.height,
                get_color_space_for_target(),
            )
            .map_err(into_lottie::<R>)
    }

    fn load_animation(&mut self, data: &str) -> Result<R::Animation, LottieRendererError> {
        let mut animation = R::Animation::default();

        #[cfg(feature = "tvg-v0")]
        animation
            .load_data(data, "lottie")
            .map_err(into_lottie::<R>)?;

        #[cfg(feature = "tvg-v1")]
        animation
            .load_data(data, "lottie+json")
            .map_err(into_lottie::<R>)?;

        let (pw, ph) = animation.get_size().map_err(into_lottie::<R>)?;
        self.picture_width = pw;
        self.picture_height = ph;

        self.apply_layout_transform(&mut animation)?;

        Ok(animation)
    }

    #[inline]
    fn apply_layout_transform(
        &mut self,
        animation: &mut R::Animation,
    ) -> Result<(), LottieRendererError> {
        // Set animation to its original size
        animation
            .set_size(self.picture_width, self.picture_height)
            .map_err(into_lottie::<R>)?;

        let layout_matrix = self.layout.to_transform_matrix(
            self.width as f32,
            self.height as f32,
            self.picture_width,
            self.picture_height,
        );

        let combined_matrix = multiply_matrices(&self.user_transform, &layout_matrix);

        animation
            .set_transform(&combined_matrix)
            .map_err(into_lottie::<R>)?;

        self.updated = true;

        Ok(())
    }

    fn create_background_shape(&self) -> Result<R::Shape, LottieRendererError> {
        let mut background_shape = R::Shape::default();

        background_shape
            .append_rect(0.0, 0.0, self.width as f32, self.height as f32, 0.0, 0.0)
            .map_err(into_lottie::<R>)?;

        let (red, green, blue, alpha) = hex_to_rgba(self.background_color);
        background_shape
            .fill((red, green, blue, alpha))
            .map_err(into_lottie::<R>)?;

        Ok(background_shape)
    }

    fn setup_drawables(
        &mut self,
        background_shape: &R::Shape,
        animation: &R::Animation,
    ) -> Result<(), LottieRendererError> {
        self.renderer
            .push(Drawable::Shape(background_shape))
            .map_err(into_lottie::<R>)?;

        self.renderer
            .push(Drawable::Animation(animation))
            .map_err(into_lottie::<R>)?;

        Ok(())
    }

    #[inline]
    fn get_animation(&self) -> Result<&R::Animation, LottieRendererError> {
        self.animation
            .as_ref()
            .ok_or(LottieRendererError::AnimationNotLoaded)
    }

    #[inline]
    fn get_animation_mut(&mut self) -> Result<&mut R::Animation, LottieRendererError> {
        self.animation
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)
    }

    #[inline]
    fn get_background_shape_mut(&mut self) -> Result<&mut R::Shape, LottieRendererError> {
        self.background_shape
            .as_mut()
            .ok_or(LottieRendererError::BackgroundShapeNotInitialized)
    }

    fn apply_user_transform(&mut self) -> Result<(), LottieRendererError> {
        if self.animation.is_none() {
            return Ok(());
        }

        let layout_matrix = self.layout.to_transform_matrix(
            self.width as f32,
            self.height as f32,
            self.picture_width,
            self.picture_height,
        );

        let combined_matrix = multiply_matrices(&self.user_transform, &layout_matrix);

        self.get_animation_mut()?
            .set_transform(&combined_matrix)
            .map_err(into_lottie::<R>)?;

        self.updated = true;
        Ok(())
    }
}

impl<R: Renderer> LottieRenderer for LottieRendererImpl<R> {
    fn load_data(
        &mut self,
        data: &str,
        width: u32,
        height: u32,
    ) -> Result<(), LottieRendererError> {
        self.clear()?;

        self.setup_buffer_and_target(width, height)?;

        let animation = self.load_animation(data)?;

        let background_shape = self.create_background_shape()?;

        self.setup_drawables(&background_shape, &animation)?;

        self.animation = Some(animation);
        self.background_shape = Some(background_shape);
        self.updated = true;

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
        self.get_animation()?
            .get_total_frame()
            .map_err(into_lottie::<R>)
    }

    fn duration(&self) -> Result<f32, LottieRendererError> {
        self.get_animation()?
            .get_duration()
            .map_err(into_lottie::<R>)
    }

    fn current_frame(&self) -> f32 {
        self.current_frame
    }

    #[inline]
    fn buffer(&self) -> &[u32] {
        &self.buffer
    }

    #[inline]
    fn clear(&mut self) {
        self.buffer.clear()
    }

    fn render(&mut self) -> Result<(), LottieRendererError> {
        if self.updated {
            self.renderer.update().map_err(into_lottie::<R>)?;
            self.renderer.draw(true).map_err(into_lottie::<R>)?;
            self.renderer.sync().map_err(into_lottie::<R>)?;
            self.updated = false;

            return Ok(());
        }

        Err(LottieRendererError::RendererError)
    }

    #[inline]
    fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> Result<(), LottieRendererError> {
        self.renderer
            .set_viewport(x, y, w, h)
            .map_err(into_lottie::<R>)
    }

    fn set_frame(&mut self, no: f32) -> Result<(), LottieRendererError> {
        let total_frames = self.total_frames()?;

        if no < 0.0 || no >= total_frames {
            return Err(LottieRendererError::InvalidArgument);
        }

        self.get_animation_mut()?
            .set_frame(no)
            .map_err(into_lottie::<R>)?;

        self.updated = true;

        self.current_frame = no;

        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), LottieRendererError> {
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }

        if width == 0 || height == 0 {
            return Err(LottieRendererError::InvalidArgument);
        }

        let _ = self.renderer.sync();

        self.width = width;
        self.height = height;

        self.resize_buffer(width, height)?;

        self.renderer
            .set_target(
                &mut self.buffer,
                self.width,
                self.width,
                self.height,
                get_color_space_for_target(),
            )
            .map_err(into_lottie::<R>)?;

        if self.animation.is_some() {
            self.apply_user_transform()?;
        }

        if self.background_shape.is_some() {
            let current_width = self.width as f32;
            let current_height = self.height as f32;

            self.get_background_shape_mut()?
                .append_rect(0.0, 0.0, current_width, current_height, 0.0, 0.0)
                .map_err(into_lottie::<R>)?;
        }

        self.updated = true;

        self.render()?;

        Ok(())
    }

    #[inline]
    fn buffer_ptr(&self) -> *const u32 {
        self.buffer.as_ptr()
    }

    #[inline]
    fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    fn set_background_color(&mut self, hex_color: u32) -> Result<(), LottieRendererError> {
        self.background_color = hex_color;

        if self.background_shape.is_none() {
            return Ok(());
        }

        let (red, green, blue, alpha) = hex_to_rgba(self.background_color);

        let set_background = self
            .get_background_shape_mut()?
            .fill((red, green, blue, alpha))
            .map_err(into_lottie::<R>);

        self.updated = true;

        set_background
    }

    fn set_slots(&mut self, slots: &str) -> Result<(), LottieRendererError> {
        self.get_animation_mut()?
            .set_slots(slots)
            .map_err(into_lottie::<R>)
    }

    fn updated(&self) -> bool {
        self.updated
    }

    fn tween(
        &mut self,
        to: f32,
        duration: Option<f32>,
        easing: Option<[f32; 4]>,
    ) -> Result<(), LottieRendererError> {
        self.get_animation_mut()?
            .tween(to, duration, easing)
            .map_err(into_lottie::<R>)
    }

    fn is_tweening(&self) -> bool {
        self.get_animation()
            .map(|animation| animation.is_tweening())
            .unwrap_or(false)
    }

    fn tween_update(&mut self, progress: Option<f32>) -> Result<bool, LottieRendererError> {
        let updated_tween = self
            .get_animation_mut()?
            .tween_update(progress)
            .map_err(into_lottie::<R>);

        self.updated = true;

        updated_tween
    }

    fn tween_stop(&mut self) -> Result<(), LottieRendererError> {
        self.get_animation_mut()?
            .tween_stop()
            .map_err(into_lottie::<R>)
    }

    fn set_layout(&mut self, layout: &Layout) -> Result<(), LottieRendererError> {
        if self.layout == *layout {
            return Ok(());
        }

        self.layout = layout.clone();

        if self.animation.is_some() {
            self.apply_user_transform()?;
        }

        Ok(())
    }

    fn get_layer_bounds(&self, layer_name: &str) -> Result<[f32; 8], LottieRendererError> {
        self.get_animation()?
            .get_layer_bounds(layer_name)
            .map_err(into_lottie::<R>)
    }

    fn intersect(&self, x: f32, y: f32, layer_name: &str) -> Result<bool, LottieRendererError> {
        self.get_animation()?
            .intersect(x, y, layer_name)
            .map_err(into_lottie::<R>)
    }

    fn get_transform(&self) -> Result<[f32; 9], LottieRendererError> {
        Ok(self.user_transform)
    }

    fn set_transform(&mut self, transform: &[f32; 9]) -> Result<(), LottieRendererError> {
        self.user_transform = *transform;

        if self.animation.is_some() {
            self.apply_user_transform()?;
        }

        Ok(())
    }
}

#[inline]
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

fn multiply_matrices(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    [
        a[0] * b[0] + a[1] * b[3] + a[2] * b[6], // e11
        a[0] * b[1] + a[1] * b[4] + a[2] * b[7], // e12
        a[0] * b[2] + a[1] * b[5] + a[2] * b[8], // e13
        a[3] * b[0] + a[4] * b[3] + a[5] * b[6], // e21
        a[3] * b[1] + a[4] * b[4] + a[5] * b[7], // e22
        a[3] * b[2] + a[4] * b[5] + a[5] * b[8], // e23
        a[6] * b[0] + a[7] * b[3] + a[8] * b[6], // e31
        a[6] * b[1] + a[7] * b[4] + a[8] * b[7], // e32
        a[6] * b[2] + a[7] * b[5] + a[8] * b[8], // e33
    ]
}
