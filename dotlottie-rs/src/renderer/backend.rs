use core::error;
use std::ffi::{CStr, CString};

// A 2D vector for representing a point
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// A frame range within a Lottie animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub start: f32,
    pub end: f32,
}

/// A named marker within a Lottie animation.
#[derive(Debug, Clone)]
pub struct Marker {
    pub name: CString,
    pub segment: Segment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn is_transparent(&self) -> bool {
        self.a == 0
    }
}

/// Layout: 0xRRGGBBAA
impl From<u32> for Rgba {
    fn from(hex: u32) -> Self {
        Self {
            r: ((hex >> 24) & 0xFF) as u8,
            g: ((hex >> 16) & 0xFF) as u8,
            b: ((hex >> 8) & 0xFF) as u8,
            a: (hex & 0xFF) as u8,
        }
    }
}

impl From<Rgba> for u32 {
    fn from(c: Rgba) -> u32 {
        (c.r as u32) << 24 | (c.g as u32) << 16 | (c.b as u32) << 8 | c.a as u32
    }
}

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

/// Trait for OpenGL display types that can be used with the renderer.
///
/// Implement this trait for your platform's display handle
/// (e.g., EGLDisplay on Linux/Android, HDC on Windows).
/// Pass a null pointer when the platform does not require a display handle (e.g., macOS CGL).
pub trait GlDisplay {
    /// Returns the raw display pointer, or null if not applicable.
    ///
    /// # Safety
    /// The returned pointer must be valid for the lifetime of the display.
    fn as_ptr(&self) -> *mut std::ffi::c_void;

    /// Creates a wrapper from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime of the resulting wrapper.
    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self
    where
        Self: Sized;
}

/// Trait for OpenGL surface types that can be used with the renderer.
///
/// Implement this trait for your platform's surface handle
/// (e.g., EGLSurface on Linux/Android).
/// Pass a null pointer when the platform does not require a surface handle (e.g., macOS CGL).
pub trait GlSurface {
    /// Returns the raw surface pointer, or null if not applicable.
    ///
    /// # Safety
    /// The returned pointer must be valid for the lifetime of the surface.
    fn as_ptr(&self) -> *mut std::ffi::c_void;

    /// Creates a wrapper from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime of the resulting wrapper.
    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self
    where
        Self: Sized;
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
    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self
    where
        Self: Sized;
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
    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self
    where
        Self: Sized;
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
    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self
    where
        Self: Sized;
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
    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self
    where
        Self: Sized;
}

pub enum Drawable<'d, R: Renderer> {
    Shape(&'d R::Shape),
    Animation(&'d R::Animation),
}

pub trait Shape: Default {
    type Error: error::Error;

    fn fill(&mut self, color: Rgba) -> Result<(), Self::Error>;

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

/// Source of audio data for a Lottie audio layer.
#[cfg(feature = "audio")]
pub enum AudioSource<'a> {
    Embedded {
        bytes: &'a [u8],
        mime: Option<&'a str>,
    },
    External(&'a str),
}

/// A playback-state change for a Lottie audio layer, fired when it enters or
/// leaves its active range.
#[cfg(feature = "audio")]
pub struct AudioEvent<'a> {
    pub source: AudioSource<'a>,
    /// Seek position in seconds; valid when `active`.
    pub offset: f32,
    /// Volume on a 0–100 scale; valid when `active`.
    pub volume: f32,
    pub active: bool,
}

#[cfg(feature = "audio")]
pub type AudioResolver = Box<dyn for<'a> FnMut(AudioEvent<'a>)>;

/// Feathered circular alpha mask. Coordinates are canvas px on `@stage`,
/// comp px on layers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpotMask {
    pub cx: f32,
    pub cy: f32,
    pub radius: f32,
    /// 0..1 fraction of the radius that fades to transparent.
    pub feather: f32,
}

impl Default for SpotMask {
    fn default() -> Self {
        Self {
            cx: 0.0,
            cy: 0.0,
            radius: 0.0,
            feather: 0.4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClipRegion {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: f32,
    },
    Circle {
        cx: f32,
        cy: f32,
        r: f32,
    },
}

/// Duotone grade: black point → white point, blended by intensity 0..1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tint {
    pub black: [u8; 3],
    pub white: [u8; 3],
    pub intensity: f32,
}

impl Default for Tint {
    fn default() -> Self {
        Self {
            black: [0, 0, 0],
            white: [255, 255, 255],
            intensity: 0.0,
        }
    }
}

/// User override props for a named node, composed on top of the authored animation
/// every frame. `None` fields leave the authored value untouched.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeProps {
    /// Comp-space translation offset, px.
    pub x: Option<f32>,
    pub y: Option<f32>,
    /// Degrees, clockwise, on top of the authored rotation.
    pub rotate: Option<f32>,
    /// Scale factors multiplying the authored scale (1 = unchanged).
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    /// Comp-space pivot for rotate/scale; `None` = center of the node's animated bounds.
    pub anchor: Option<(f32, f32)>,
    /// 0..1 multiplier on the authored opacity.
    pub opacity: Option<f32>,
    pub visible: Option<bool>,
    /// Gaussian blur sigma, px. Overriding suppresses authored layer effects.
    pub blur: Option<f32>,
    /// `Tvg_Blend_Method` value; set-only, not tweened.
    pub blend_mode: Option<u8>,
    pub tint: Option<Tint>,
    pub spot: Option<SpotMask>,
    pub clip: Option<ClipRegion>,
    /// One-shot: re-apply pristine base values, then the entry is dropped.
    pub restore: bool,
}

impl NodeProps {
    pub fn has_overrides(&self) -> bool {
        self.x.is_some()
            || self.y.is_some()
            || self.rotate.is_some()
            || self.scale_x.is_some()
            || self.scale_y.is_some()
            || self.opacity.is_some()
            || self.visible.is_some()
            || self.blur.is_some()
            || self.blend_mode.is_some()
            || self.tint.is_some()
            || self.spot.is_some()
            || self.clip.is_some()
    }

    pub fn scalar(&self, prop: crate::motion::Prop) -> Option<f32> {
        use crate::motion::Prop;
        match prop {
            Prop::X => self.x,
            Prop::Y => self.y,
            Prop::Rotate => self.rotate,
            Prop::ScaleX => self.scale_x,
            Prop::ScaleY => self.scale_y,
            Prop::Opacity => self.opacity,
            Prop::Blur => self.blur,
            Prop::TintIntensity => self.tint.map(|t| t.intensity),
            Prop::SpotCx => self.spot.map(|s| s.cx),
            Prop::SpotCy => self.spot.map(|s| s.cy),
            Prop::SpotRadius => self.spot.map(|s| s.radius),
            Prop::SpotFeather => self.spot.map(|s| s.feather),
            Prop::ClipCx | Prop::ClipCy | Prop::ClipRadius => match self.clip {
                Some(ClipRegion::Circle { cx, cy, r }) => Some(match prop {
                    Prop::ClipCx => cx,
                    Prop::ClipCy => cy,
                    _ => r,
                }),
                _ => None,
            },
            Prop::Value => None,
        }
    }

    pub fn set_scalar(&mut self, prop: crate::motion::Prop, v: f32) {
        use crate::motion::Prop;
        match prop {
            Prop::X => self.x = Some(v),
            Prop::Y => self.y = Some(v),
            Prop::Rotate => self.rotate = Some(v),
            Prop::ScaleX => self.scale_x = Some(v),
            Prop::ScaleY => self.scale_y = Some(v),
            Prop::Opacity => self.opacity = Some(v),
            Prop::Blur => self.blur = Some(v),
            Prop::TintIntensity => self.tint.get_or_insert_with(Default::default).intensity = v,
            Prop::SpotCx => self.spot.get_or_insert_with(Default::default).cx = v,
            Prop::SpotCy => self.spot.get_or_insert_with(Default::default).cy = v,
            Prop::SpotRadius => self.spot.get_or_insert_with(Default::default).radius = v,
            Prop::SpotFeather => self.spot.get_or_insert_with(Default::default).feather = v,
            Prop::ClipCx | Prop::ClipCy | Prop::ClipRadius => {
                let clip = self.clip.get_or_insert(ClipRegion::Circle {
                    cx: 0.0,
                    cy: 0.0,
                    r: 0.0,
                });
                if let ClipRegion::Circle { cx, cy, r } = clip {
                    match prop {
                        Prop::ClipCx => *cx = v,
                        Prop::ClipCy => *cy = v,
                        _ => *r = v,
                    }
                }
            }
            Prop::Value => {}
        }
        self.restore = false;
    }

    /// Clear one scalar (whole substructure for grouped props).
    pub fn clear_scalar(&mut self, prop: crate::motion::Prop) {
        use crate::motion::Prop;
        match prop {
            Prop::X => self.x = None,
            Prop::Y => self.y = None,
            Prop::Rotate => self.rotate = None,
            Prop::ScaleX => self.scale_x = None,
            Prop::ScaleY => self.scale_y = None,
            Prop::Opacity => self.opacity = None,
            Prop::Blur => self.blur = None,
            Prop::TintIntensity => self.tint = None,
            Prop::SpotCx | Prop::SpotCy | Prop::SpotRadius | Prop::SpotFeather => {
                self.spot = None;
            }
            Prop::ClipCx | Prop::ClipCy | Prop::ClipRadius => self.clip = None,
            Prop::Value => {}
        }
    }

    /// Overwrite this entry's fields with the `Some` fields of `other`.
    pub fn merge(&mut self, other: &NodeProps) {
        macro_rules! take {
            ($($field:ident),*) => {
                $(if other.$field.is_some() { self.$field = other.$field; })*
            };
        }
        take!(
            x, y, rotate, scale_x, scale_y, anchor, opacity, visible, blur, blend_mode, tint, spot,
            clip
        );
        self.restore = false;
    }
}

pub trait Animation: Default {
    type Error: error::Error;

    fn load_data(&mut self, data: &CStr, mimetype: &CStr) -> Result<(), Self::Error>;

    /// Register a callback for audio layer playback changes, or `None` to disable.
    #[cfg(feature = "audio")]
    fn set_audio_resolver(&mut self, resolver: Option<AudioResolver>) -> Result<(), Self::Error>;

    fn hit_test(&self, point: Point, layer_name: &str) -> Result<bool, Self::Error>;

    /// Compose `props` onto the named layer's pristine base values. The base is
    /// captured per paint pointer: a rebuilt layer scene carries fresh authored
    /// values, while a stable pointer keeps the cached base valid — recapturing on a
    /// stable pointer would read back our own composed values and accumulate.
    /// `canvas_to_comp` maps canvas space back to composition space (auto anchor).
    fn apply_node_props(
        &mut self,
        name: &str,
        props: &NodeProps,
        canvas_to_comp: &[f32; 9],
    ) -> Result<(), Self::Error>;

    /// Deep-copy a named layer as of the current frame, registered under `as_name` in
    /// the node namespace (canvas space, stable paint). `comp_to_canvas` bakes the
    /// picture transform into the copy.
    fn duplicate_node(
        &mut self,
        source: &str,
        as_name: &str,
        comp_to_canvas: &[f32; 9],
    ) -> Result<(), Self::Error>;

    /// Remove a node created by `duplicate_node`.
    fn remove_node(&mut self, name: &str) -> Result<(), Self::Error>;

    fn get_size(&self) -> Result<(f32, f32), Self::Error>;

    fn set_size(&mut self, width: f32, height: f32) -> Result<(), Self::Error>;

    fn get_total_frame(&self) -> Result<f32, Self::Error>;

    fn get_duration(&self) -> Result<f32, Self::Error>;

    fn set_frame(&mut self, frame_no: f32) -> Result<(), Self::Error>;

    /// Generate a slot override from JSON and return its code for later use
    fn gen_slot(&mut self, slot_json: &CStr) -> Result<u32, Self::Error>;

    /// Apply a previously generated slot by its code (0 = reset all slots to defaults)
    fn apply_slot(&mut self, slot_code: u32) -> Result<(), Self::Error>;

    /// Delete a previously generated slot by its code
    fn del_slot(&mut self, slot_code: u32) -> Result<(), Self::Error>;

    fn set_quality(&mut self, quality: u8) -> Result<(), Self::Error>;

    fn tween_to(&mut self, to: f32) -> Result<(), Self::Error>;

    fn tween_go(&mut self, progress: f32) -> Result<(), Self::Error>;

    fn set_transform(&mut self, matrix: &[f32; 9]) -> Result<(), Self::Error>;

    // ── Markers & Segments ───────────────────────────────────────────────

    fn markers(&self) -> &[Marker];

    fn set_segment(&mut self, segment: Option<Segment>);

    fn segment(&self) -> Result<Segment, Self::Error>;
}

pub trait Renderer: Sized + 'static {
    type Shape: Shape<Error = Self::Error>;
    type Animation: Animation<Error = Self::Error>;
    type Error: error::Error + 'static;

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

    /// Sets an OpenGL rendering target.
    ///
    /// `display` and `surface` may carry null pointers on platforms that do not require them
    /// (e.g., macOS CGL only needs `context`). On EGL-based platforms (Android, Linux) all
    /// three handles are typically required.
    ///
    /// All non-null handles must remain valid for the lifetime of rendering operations.
    fn set_gl_target(
        &mut self,
        display: &dyn GlDisplay,
        surface: &dyn GlSurface,
        context: &dyn GlContext,
        id: i32,
        width: u32,
        height: u32,
    ) -> Result<(), Self::Error>;

    /// Sets a WebGPU rendering target.
    ///
    /// All handles must remain valid for the lifetime of rendering operations.
    #[allow(clippy::too_many_arguments)]
    fn set_wg_target(
        &mut self,
        device: &dyn WgpuDevice,
        instance: &dyn WgpuInstance,
        target: &dyn WgpuTarget,
        width: u32,
        height: u32,
        target_type: WgpuTargetType,
    ) -> Result<(), Self::Error>;

    fn clear(&self) -> Result<(), Self::Error>;

    fn push(&mut self, drawable: Drawable<Self>) -> Result<(), Self::Error>;

    /// Insert `drawable` immediately before `at` in the scene.
    fn insert(&mut self, drawable: Drawable<Self>, at: Drawable<Self>) -> Result<(), Self::Error>;

    fn draw(&mut self, clear_buffer: bool) -> Result<(), Self::Error>;

    fn sync(&mut self) -> Result<(), Self::Error>;

    fn update(&mut self) -> Result<(), Self::Error>;

    fn load_font(font_name: &str, font_data: &[u8]) -> Result<(), Self::Error>;

    fn unload_font(font_name: &str) -> Result<(), Self::Error>;
}
