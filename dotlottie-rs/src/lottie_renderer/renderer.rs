use core::error;
use std::ffi::CStr;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum ColorSpace {
    ABGR8888,
    ABGR8888S,
    ARGB8888,
    ARGB8888S,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum WgpuTargetType {
    Surface = 0,
    Texture = 1,
}

/// Trait for OpenGL context types that can be used with the renderer.
///
/// Implement this trait for your windowing library's OpenGL context type
/// (e.g., glutin::Context, sdl2::video::GLContext, etc.)
pub trait GlContext {
    /// Returns the raw OpenGL context pointer.
    ///
    /// # Safety
    /// The returned pointer must be valid for the lifetime of the context
    /// and point to a valid OpenGL context.
    fn as_ptr(&self) -> *mut std::ffi::c_void;

    /// Creates a wrapper from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime of the resulting wrapper
    /// and point to a valid OpenGL context.
    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self;
}

/// Trait for WebGPU device types that can be used with the renderer.
///
/// Implement this trait for your WebGPU device wrapper type.
pub trait WgpuDevice {
    /// Returns the raw WebGPU device pointer.
    ///
    /// # Safety
    /// The returned pointer must be valid for the lifetime of the device
    /// and point to a valid WebGPU device, or be null to let ThorVG create its own.
    fn as_ptr(&self) -> *mut std::ffi::c_void;

    /// Creates a wrapper from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime of the resulting wrapper
    /// and point to a valid WebGPU device, or be null.
    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self;
}

/// Trait for WebGPU instance types that can be used with the renderer.
///
/// Implement this trait for your WebGPU instance wrapper type.
pub trait WgpuInstance {
    /// Returns the raw WebGPU instance pointer.
    ///
    /// # Safety
    /// The returned pointer must be valid for the lifetime of the instance
    /// and point to a valid WebGPU instance.
    fn as_ptr(&self) -> *mut std::ffi::c_void;

    /// Creates a wrapper from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime of the resulting wrapper
    /// and point to a valid WebGPU instance.
    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self;
}

/// Trait for WebGPU render target types that can be used with the renderer.
///
/// Implement this trait for your WebGPU surface/target wrapper type.
pub trait WgpuTarget {
    /// Returns the raw WebGPU target pointer.
    ///
    /// # Safety
    /// The returned pointer must be valid for the lifetime of the target
    /// and point to a valid WebGPU render target.
    fn as_ptr(&self) -> *mut std::ffi::c_void;

    /// Creates a wrapper from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime of the resulting wrapper
    /// and point to a valid WebGPU render target.
    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self;
}

pub enum Drawable<'d, R: Renderer> {
    Shape(&'d R::Shape),
    Animation(&'d R::Animation),
}

pub trait Shape: Default {
    type Error: error::Error;

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
    type Error: error::Error;

    fn load_data(&mut self, data: &CStr, mimetype: &CStr) -> Result<(), Self::Error>;

    fn intersect(&self, x: f32, y: f32, layer_name: &str) -> Result<bool, Self::Error>;

    fn get_layer_bounds(&self, layer_name: &str) -> Result<[f32; 8], Self::Error>;

    fn get_size(&self) -> Result<(f32, f32), Self::Error>;

    fn set_size(&mut self, width: f32, height: f32) -> Result<(), Self::Error>;

    fn scale(&mut self, factor: f32) -> Result<(), Self::Error>;

    fn translate(&mut self, tx: f32, ty: f32) -> Result<(), Self::Error>;

    fn get_total_frame(&self) -> Result<f32, Self::Error>;

    fn get_duration(&self) -> Result<f32, Self::Error>;

    fn set_frame(&mut self, frame_no: f32) -> Result<(), Self::Error>;

    fn get_frame(&self) -> Result<f32, Self::Error>;

    fn set_slots_str(&mut self, slots: &CStr) -> Result<(), Self::Error>;

    /// Generate a slot override from JSON and return its code for later use
    fn gen_slot(&mut self, slot_json: &str) -> Result<u32, Self::Error>;

    /// Apply a previously generated slot by its code (0 = reset all slots to defaults)
    fn apply_slot(&mut self, slot_code: u32) -> Result<(), Self::Error>;

    /// Delete a previously generated slot by its code
    fn del_slot(&mut self, slot_code: u32) -> Result<(), Self::Error>;

    fn set_quality(&mut self, quality: u8) -> Result<(), Self::Error>;

    fn tween(
        &mut self,
        to: f32,
        duration: Option<f32>,
        easing: Option<[f32; 4]>,
    ) -> Result<(), Self::Error>;

    fn tween_update(&mut self, progress: Option<f32>) -> Result<bool, Self::Error>;

    fn tween_stop(&mut self) -> Result<(), Self::Error>;

    fn is_tweening(&self) -> bool;

    fn set_transform(&mut self, matrix: &[f32; 9]) -> Result<(), Self::Error>;

    fn get_transform(&self) -> Result<[f32; 9], Self::Error>;
}

pub trait Renderer: Sized + 'static {
    type Shape: Shape<Error = Self::Error>;
    type Animation: Animation<Error = Self::Error>;
    type Error: error::Error + 'static;
    type GlContext: GlContext;
    type WgpuDevice: WgpuDevice;
    type WgpuInstance: WgpuInstance;
    type WgpuTarget: WgpuTarget;

    fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> Result<(), Self::Error>;

    /// # Safety
    ///
    /// `buffer` must be a valid pointer to a mutable u32 array with at least
    /// `stride (Width))` elements. The buffer must remain valid for the lifetime
    /// of rendering operations using this target.
    fn set_sw_target(
        &mut self,
        buffer: &mut [u32],
        stride: u32,
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Result<(), Self::Error>;

    /// Sets an OpenGL rendering target using the associated context type.
    ///
    /// The GL context must remain valid for the lifetime of rendering operations.
    fn set_gl_target(
        &mut self,
        context: &Self::GlContext,
        id: i32,
        width: u32,
        height: u32,
    ) -> Result<(), Self::Error>;

    /// Sets a WebGPU rendering target using the associated types.
    ///
    /// All WebGPU objects must remain valid for the lifetime of rendering operations.
    #[allow(clippy::too_many_arguments)]
    fn set_wg_target(
        &mut self,
        device: &Self::WgpuDevice,
        instance: &Self::WgpuInstance,
        target: &Self::WgpuTarget,
        width: u32,
        height: u32,
        target_type: WgpuTargetType,
    ) -> Result<(), Self::Error>;

    fn clear(&self) -> Result<(), Self::Error>;

    fn push(&mut self, drawable: Drawable<Self>) -> Result<(), Self::Error>;

    fn draw(&mut self, clear_buffer: bool) -> Result<(), Self::Error>;

    fn sync(&mut self) -> Result<(), Self::Error>;

    fn update(&mut self) -> Result<(), Self::Error>;

    fn load_font(font_name: &str, font_data: &[u8]) -> Result<(), Self::Error>;

    fn unload_font(font_name: &str) -> Result<(), Self::Error>;
}
