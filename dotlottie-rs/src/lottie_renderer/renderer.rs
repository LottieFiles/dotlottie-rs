use std::{error::Error, fmt};

pub enum ColorSpace {
    ABGR8888,
    ABGR8888S,
    ARGB8888,
    ARGB8888S,
}

pub enum Drawable<'d, R: Renderer> {
    Shape(&'d R::Shape),
    Animation(&'d R::Animation),
}

pub trait Shape: Default {
    type Error: Error;

    fn fill(&mut self, color: (u8, u8, u8, u8)) -> Result<(), Self::Error>;

    fn append_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rx: f32,
        ry: f32,
    ) -> Result<(), Self::Error>;

    fn reset(&mut self) -> Result<(), Self::Error>;
}

pub trait Animation: Default {
    type Error: Error;

    fn load_data(&mut self, data: &str, mimetype: &str, copy: bool) -> Result<(), Self::Error>;

    fn get_layer_bounds(&self, layer_name: &str) -> Result<(f32, f32, f32, f32), Self::Error>;

    fn hit_check(&self, layer_name: &str, x: f32, y: f32) -> Result<bool, Self::Error>;

    fn get_size(&self) -> Result<(f32, f32), Self::Error>;

    fn set_size(&mut self, width: f32, height: f32) -> Result<(), Self::Error>;

    fn scale(&mut self, factor: f32) -> Result<(), Self::Error>;

    fn translate(&mut self, tx: f32, ty: f32) -> Result<(), Self::Error>;

    fn get_total_frame(&self) -> Result<f32, Self::Error>;

    fn get_duration(&self) -> Result<f32, Self::Error>;

    fn set_frame(&mut self, frame_no: f32) -> Result<(), Self::Error>;

    fn get_frame(&self) -> Result<f32, Self::Error>;

    fn set_slots(&mut self, slots: &str) -> Result<(), Self::Error>;

    fn tween(
        &mut self,
        to: f32,
        duration: Option<f32>,
        easing: Option<[f32; 4]>,
    ) -> Result<(), Self::Error>;

    fn tween_update(&mut self, progress: Option<f32>) -> Result<bool, Self::Error>;

    fn tween_stop(&mut self) -> Result<(), Self::Error>;
    
    fn is_tweening(&self) -> bool;
}

pub trait Renderer: Sized + 'static {
    type Shape: Shape<Error = Self::Error>;
    type Animation: Animation<Error = Self::Error>;
    type Error: fmt::Debug + Error + 'static;

    fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> Result<(), Self::Error>;

    fn set_target(
        &mut self,
        buffer: &mut Vec<u32>,
        stride: u32,
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Result<(), Self::Error>;

    fn clear(&self, free: bool) -> Result<(), Self::Error>;

    fn push(&mut self, drawable: Drawable<Self>) -> Result<(), Self::Error>;

    fn draw(&mut self, clear_buffer: bool) -> Result<(), Self::Error>;

    fn sync(&mut self) -> Result<(), Self::Error>;

    fn update(&mut self) -> Result<(), Self::Error>;
}
