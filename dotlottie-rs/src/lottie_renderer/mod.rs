use crate::Layout;
use std::ffi::CStr;

mod renderer;
pub mod slots;

#[cfg(feature = "tvg")]
mod fallback_font;

#[cfg(feature = "tvg")]
mod thorvg;

pub(crate) use renderer::Point;
pub use renderer::{
    Animation, ColorSpace, Drawable, GlContext, GlDisplay, GlSurface, Marker, Renderer, Rgba,
    Segment, Shape, WgpuDevice, WgpuInstance, WgpuTarget, WgpuTargetType,
};
pub use slots::{
    slots_from_json_string, Bezier, BezierValue, ColorSlot, ColorValue, GradientSlot, GradientStop,
    ImageSlot, LottieKeyframe, LottieProperty, PositionSlot, ScalarSlot, ScalarValue, SlotType,
    TextCaps, TextDocument, TextJustify, TextKeyframe, TextSlot, VectorSlot,
};
#[cfg(feature = "tvg")]
pub use thorvg::{TvgAnimation, TvgError, TvgRenderer, TvgShape};

use std::collections::BTreeMap;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum LottieRendererError {
    RendererError,
    InvalidColor,
    InvalidArgument,
    AnimationNotLoaded,
    BackgroundShapeNotInitialized,
    SlotNotFound,
    InvalidSlotValue,
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
    /// # Safety
    ///
    /// `buffer` must be a valid pointer to a mutable u32 array with at least
    /// `stride (Width)` elements. The buffer must remain valid for the lifetime
    /// of rendering operations using this target.
    fn set_sw_target(
        &mut self,
        buffer: &mut [u32],
        stride: u32,
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Result<(), LottieRendererError>;

    /// `display` and `surface` may carry null pointers on platforms that do not require them
    /// (e.g., macOS CGL only needs `context`). On EGL-based platforms (Android, Linux) all
    /// three handles are typically required.
    fn set_gl_target(
        &mut self,
        display: &dyn GlDisplay,
        surface: &dyn GlSurface,
        context: &dyn GlContext,
        id: i32,
        width: u32,
        height: u32,
    ) -> Result<(), LottieRendererError>;

    #[allow(clippy::too_many_arguments)]
    fn set_wg_target(
        &mut self,
        device: &dyn WgpuDevice,
        instance: &dyn WgpuInstance,
        target: &dyn WgpuTarget,
        width: u32,
        height: u32,
        target_type: WgpuTargetType,
    ) -> Result<(), LottieRendererError>;

    fn load_data(&mut self, data: &CStr) -> Result<(), LottieRendererError>;

    fn picture_width(&self) -> f32;

    fn picture_height(&self) -> f32;

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn total_frames(&self) -> Result<f32, LottieRendererError>;

    fn duration(&self) -> Result<f32, LottieRendererError>;

    fn current_frame(&self) -> f32;
    fn render(&mut self) -> Result<(), LottieRendererError>;

    fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) -> Result<(), LottieRendererError>;

    fn set_frame(&mut self, no: f32) -> Result<(), LottieRendererError>;

    fn background(&self) -> Rgba;

    fn set_background(&mut self, color: Rgba) -> Result<(), LottieRendererError>;

    fn set_color_slot(&mut self, slot_id: &str, slot: ColorSlot)
        -> Result<(), LottieRendererError>;

    fn set_gradient_slot(
        &mut self,
        slot_id: &str,
        slot: GradientSlot,
    ) -> Result<(), LottieRendererError>;

    fn set_image_slot(&mut self, slot_id: &str, slot: ImageSlot)
        -> Result<(), LottieRendererError>;

    fn set_text_slot(&mut self, slot_id: &str, slot: TextSlot) -> Result<(), LottieRendererError>;

    fn set_scalar_slot(
        &mut self,
        slot_id: &str,
        slot: ScalarSlot,
    ) -> Result<(), LottieRendererError>;

    fn set_vector_slot(
        &mut self,
        slot_id: &str,
        slot: VectorSlot,
    ) -> Result<(), LottieRendererError>;

    fn set_position_slot(
        &mut self,
        slot_id: &str,
        slot: PositionSlot,
    ) -> Result<(), LottieRendererError>;

    fn clear_slots(&mut self) -> Result<(), LottieRendererError>;

    fn clear_slot(&mut self, slot_id: &str) -> Result<(), LottieRendererError>;

    fn set_slots(&mut self, slots: BTreeMap<String, SlotType>) -> Result<(), LottieRendererError>;

    fn get_slot_ids(&self) -> Vec<String>;

    fn get_slot_type(&self, slot_id: &str) -> String;

    fn get_slot_str(&self, slot_id: &str) -> String;

    fn get_slots_str(&self) -> String;

    fn set_slot_str(&mut self, slot_id: &str, json: &str) -> Result<(), LottieRendererError>;

    fn store_default_slots(&mut self, slots: BTreeMap<String, SlotType>);

    fn reset_slot(&mut self, slot_id: &str) -> Result<(), LottieRendererError>;

    fn reset_slots(&mut self) -> bool;

    fn set_quality(&mut self, quality: u8) -> Result<(), LottieRendererError>;

    fn set_layout(&mut self, layout: &Layout) -> Result<(), LottieRendererError>;

    fn hit_test(&self, point: Point, layer_name: &str) -> Result<bool, LottieRendererError>;

    fn updated(&self) -> bool;

    fn tween(&mut self, from: f32, to: f32, progress: f32) -> Result<(), LottieRendererError>;

    fn sync_current_frame(&mut self, frame: f32);

    fn get_transform(&self) -> Result<[f32; 9], LottieRendererError>;

    fn set_transform(&mut self, transform: &[f32; 9]) -> Result<(), LottieRendererError>;

    fn load_font(&mut self, name: &str, data: &[u8]) -> Result<(), LottieRendererError>;

    fn unload_font(&mut self, name: &str) -> Result<(), LottieRendererError>;

    // ── Markers & Segments ───────────────────────────────────────────────

    fn markers(&self) -> &[Marker];

    fn set_segment(&mut self, segment: Option<Segment>) -> Result<(), LottieRendererError>;

    fn segment(&self) -> Result<Segment, LottieRendererError>;
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
            background: Rgba::TRANSPARENT,
            layout: Layout::default(),
            user_transform: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            batch_slot_code: None,
            slots_dirty: false,
            slot_json_buffer: Vec::with_capacity(512),
            slot_values: BTreeMap::new(),
            default_slots: BTreeMap::new(),
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
    background: Rgba,
    layout: Layout,
    user_transform: [f32; 9],
    /// ThorVG slot code for the current batch of all active slots
    batch_slot_code: Option<u32>,
    /// Dirty flag: set when slot_values changes, cleared after flush
    slots_dirty: bool,
    /// Reusable buffer for JSON serialization (avoids per-frame allocation)
    slot_json_buffer: Vec<u8>,
    /// Maps slot_id -> SlotType for value retrieval (get operations)
    slot_values: BTreeMap<String, SlotType>,
    default_slots: BTreeMap<String, SlotType>,
}

impl<R: Renderer> LottieRendererImpl<R> {
    fn clear(&mut self) -> Result<(), LottieRendererError> {
        if self.animation.is_some() || self.background_shape.is_some() {
            self.animation = None;
            self.background_shape = None;
        }
        self.current_frame = 0.0;
        self.updated = false;
        self.batch_slot_code = None;
        self.slots_dirty = false;
        self.slot_json_buffer.clear();
        self.slot_values.clear();
        self.default_slots.clear();
        Ok(())
    }

    /// Updates the animation layout after the canvas dimensions have changed.
    /// Called by `set_*_target` when width/height differ from the previous values.
    fn resize(&mut self) -> Result<(), LottieRendererError> {
        if self.animation.is_some() {
            let _ = self.renderer.sync();
            self.apply_user_transform()?;
        }

        if self.background_shape.is_some() {
            let w = self.width as f32;
            let h = self.height as f32;
            self.get_background_shape_mut()?
                .append_rect(0.0, 0.0, w, h, 0.0, 0.0)
                .map_err(into_lottie::<R>)?;
        }

        if self.animation.is_some() || self.background_shape.is_some() {
            self.updated = true;
        }

        Ok(())
    }

    fn load_animation(&mut self, data: &CStr) -> Result<R::Animation, LottieRendererError> {
        let mut animation = R::Animation::default();

        let mimetype = c"lottie+json";
        animation
            .load_data(data, mimetype)
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

        background_shape
            .fill(self.background)
            .map_err(into_lottie::<R>)?;

        Ok(background_shape)
    }

    fn setup_drawables(
        &mut self,
        background_shape: Option<&R::Shape>,
        animation: &R::Animation,
    ) -> Result<(), LottieRendererError> {
        if let Some(bg) = background_shape {
            self.renderer
                .push(Drawable::Shape(bg))
                .map_err(into_lottie::<R>)?;
        }

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

    /// Flush all pending slot changes to ThorVG as a single batch.
    /// Called once per render() — reduces 3N FFI calls to 3 constant.
    fn flush_slots(&mut self) -> Result<(), LottieRendererError> {
        if !self.slots_dirty {
            return Ok(());
        }

        let animation = self
            .animation
            .as_mut()
            .ok_or(LottieRendererError::AnimationNotLoaded)?;

        // 1. Delete previous batch code if it exists (1 FFI call)
        if let Some(old_code) = self.batch_slot_code.take() {
            let _ = animation.del_slot(old_code);
        }

        // 2. If no slots active, reset to defaults
        if self.slot_values.is_empty() {
            animation.apply_slot(0).map_err(into_lottie::<R>)?;
            self.slots_dirty = false;
            self.updated = true;
            return Ok(());
        }

        // 3. Serialize all slots into the reusable buffer
        slots::slots_to_json_writer(&self.slot_values, &mut self.slot_json_buffer)
            .map_err(|_| LottieRendererError::InvalidArgument)?;

        // 4. Append null terminator for CStr
        self.slot_json_buffer.push(0);

        // 5. Create CStr from buffer — zero allocation
        let cstr = CStr::from_bytes_with_nul(&self.slot_json_buffer)
            .map_err(|_| LottieRendererError::InvalidArgument)?;

        // 6. Generate new batch slot code (1 FFI call)
        let new_code = animation.gen_slot(cstr).map_err(into_lottie::<R>)?;

        // 7. Apply the batch (1 FFI call)
        animation.apply_slot(new_code).map_err(into_lottie::<R>)?;

        self.batch_slot_code = Some(new_code);
        self.slots_dirty = false;
        self.updated = true;

        Ok(())
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
    fn load_font(&mut self, font_name: &str, font_data: &[u8]) -> Result<(), LottieRendererError> {
        R::load_font(font_name, font_data).map_err(into_lottie::<R>)
    }

    fn unload_font(&mut self, name: &str) -> Result<(), LottieRendererError> {
        R::unload_font(name).map_err(into_lottie::<R>)
    }

    fn set_sw_target(
        &mut self,
        buffer_ptr: &mut [u32],
        stride: u32,
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Result<(), LottieRendererError> {
        self.renderer
            .set_sw_target(buffer_ptr, stride, width, height, color_space)
            .map_err(into_lottie::<R>)?;
        let changed = (self.width, self.height) != (width, height);
        self.width = width;
        self.height = height;
        if changed {
            self.resize()?;
        }
        Ok(())
    }

    fn set_gl_target(
        &mut self,
        display: &dyn GlDisplay,
        surface: &dyn GlSurface,
        context: &dyn GlContext,
        id: i32,
        width: u32,
        height: u32,
    ) -> Result<(), LottieRendererError> {
        self.renderer
            .set_gl_target(display, surface, context, id, width, height)
            .map_err(into_lottie::<R>)?;
        let changed = (self.width, self.height) != (width, height);
        self.width = width;
        self.height = height;
        if changed {
            self.resize()?;
        }
        Ok(())
    }

    fn set_wg_target(
        &mut self,
        device: &dyn WgpuDevice,
        instance: &dyn WgpuInstance,
        target: &dyn WgpuTarget,
        width: u32,
        height: u32,
        target_type: WgpuTargetType,
    ) -> Result<(), LottieRendererError> {
        self.renderer
            .set_wg_target(device, instance, target, width, height, target_type)
            .map_err(into_lottie::<R>)?;
        let changed = (self.width, self.height) != (width, height);
        self.width = width;
        self.height = height;
        if changed {
            self.resize()?;
        }
        Ok(())
    }

    fn load_data(&mut self, data: &CStr) -> Result<(), LottieRendererError> {
        self.clear()?;

        // Extract default slot values BEFORE passing to ThorVG, because
        // ThorVG's load_data with copy=false may parse the JSON in-place
        // and mutate the buffer (nulling out string terminators).
        let default_slots = data
            .to_str()
            .map(slots::extract_slots_from_animation)
            .unwrap_or_default();

        let animation = self.load_animation(data)?;

        let background_shape = if !self.background.is_transparent() {
            Some(self.create_background_shape()?)
        } else {
            None
        };

        self.setup_drawables(background_shape.as_ref(), &animation)?;

        self.animation = Some(animation);
        self.background_shape = background_shape;
        self.updated = true;

        self.store_default_slots(default_slots);

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

    fn background(&self) -> Rgba {
        self.background
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

    fn render(&mut self) -> Result<(), LottieRendererError> {
        self.flush_slots()?;

        if self.updated {
            // Sync before update to ensure previous frame's rendering is complete
            // This is crucial for async renderers like WebGL
            self.renderer.sync().map_err(into_lottie::<R>)?;

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
        self.get_animation_mut()?
            .set_frame(no)
            .map_err(into_lottie::<R>)?;

        self.updated = true;

        self.current_frame = no;

        Ok(())
    }

    fn set_background(&mut self, color: Rgba) -> Result<(), LottieRendererError> {
        self.background = color;

        if let Some(bg) = self.background_shape.as_mut() {
            bg.fill(color).map_err(into_lottie::<R>)?;
            self.updated = true;
        } else if !color.is_transparent() && self.animation.is_some() {
            // Background shape was skipped at load (was transparent). Now need it.
            // Insert before the animation to maintain correct z-order.
            let background_shape = self.create_background_shape()?;
            let animation = self
                .animation
                .as_ref()
                .ok_or(LottieRendererError::AnimationNotLoaded)?;
            self.renderer
                .insert(
                    Drawable::Shape(&background_shape),
                    Drawable::Animation(animation),
                )
                .map_err(into_lottie::<R>)?;
            self.background_shape = Some(background_shape);
            self.updated = true;
        }

        Ok(())
    }

    fn set_color_slot(
        &mut self,
        slot_id: &str,
        slot: ColorSlot,
    ) -> Result<(), LottieRendererError> {
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Color(slot));
        self.slots_dirty = true;
        Ok(())
    }

    fn set_gradient_slot(
        &mut self,
        slot_id: &str,
        slot: GradientSlot,
    ) -> Result<(), LottieRendererError> {
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Gradient(slot));
        self.slots_dirty = true;
        Ok(())
    }

    fn set_image_slot(
        &mut self,
        slot_id: &str,
        slot: ImageSlot,
    ) -> Result<(), LottieRendererError> {
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Image(slot));
        self.slots_dirty = true;
        Ok(())
    }

    fn set_text_slot(&mut self, slot_id: &str, slot: TextSlot) -> Result<(), LottieRendererError> {
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Text(slot));
        self.slots_dirty = true;
        Ok(())
    }

    fn set_scalar_slot(
        &mut self,
        slot_id: &str,
        slot: ScalarSlot,
    ) -> Result<(), LottieRendererError> {
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Scalar(slot));
        self.slots_dirty = true;
        Ok(())
    }

    fn set_vector_slot(
        &mut self,
        slot_id: &str,
        slot: VectorSlot,
    ) -> Result<(), LottieRendererError> {
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Vector(slot));
        self.slots_dirty = true;
        Ok(())
    }

    fn set_position_slot(
        &mut self,
        slot_id: &str,
        slot: PositionSlot,
    ) -> Result<(), LottieRendererError> {
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Position(slot));
        self.slots_dirty = true;
        Ok(())
    }

    fn clear_slots(&mut self) -> Result<(), LottieRendererError> {
        // Delete batch code if it exists
        if let Some(old_code) = self.batch_slot_code.take() {
            let _ = self.get_animation_mut()?.del_slot(old_code);
        }
        self.slot_values.clear();

        // Immediate reset to defaults via ThorVG (0 = reset all)
        self.get_animation_mut()?
            .apply_slot(0)
            .map_err(into_lottie::<R>)?;

        self.slots_dirty = false;
        self.updated = true;
        Ok(())
    }

    fn clear_slot(&mut self, slot_id: &str) -> Result<(), LottieRendererError> {
        self.slot_values.remove(slot_id);
        self.slots_dirty = true;
        Ok(())
    }

    fn set_slots(&mut self, slots: BTreeMap<String, SlotType>) -> Result<(), LottieRendererError> {
        self.slot_values = slots;
        self.slots_dirty = true;
        Ok(())
    }

    fn get_slot_ids(&self) -> Vec<String> {
        self.slot_values.keys().cloned().collect()
    }

    fn get_slot_type(&self, slot_id: &str) -> String {
        self.slot_values
            .get(slot_id)
            .map(|s| slots::slot_type_name(s).to_string())
            .unwrap_or_default()
    }

    fn get_slot_str(&self, slot_id: &str) -> String {
        self.slot_values
            .get(slot_id)
            .and_then(|s| slots::slot_to_json_string(s).ok())
            .unwrap_or_default()
    }

    fn get_slots_str(&self) -> String {
        slots::slots_to_json_string(&self.slot_values).unwrap_or_default()
    }

    fn set_slot_str(&mut self, slot_id: &str, json: &str) -> Result<(), LottieRendererError> {
        let slot_type = self.get_slot_type(slot_id);
        if slot_type.is_empty() {
            return Err(LottieRendererError::SlotNotFound);
        }

        match slots::parse_slot_from_json(&slot_type, json) {
            Some(slot) => {
                self.slot_values.insert(slot_id.to_string(), slot);
                self.slots_dirty = true;
                Ok(())
            }
            None => Err(LottieRendererError::InvalidSlotValue),
        }
    }

    fn store_default_slots(&mut self, slots: BTreeMap<String, SlotType>) {
        self.default_slots = slots.clone();
        self.slot_values = slots;
    }

    fn reset_slot(&mut self, slot_id: &str) -> Result<(), LottieRendererError> {
        match self.default_slots.get(slot_id).cloned() {
            Some(default_value) => {
                self.slot_values.insert(slot_id.to_string(), default_value);
                self.slots_dirty = true;
                Ok(())
            }
            None => Err(LottieRendererError::SlotNotFound),
        }
    }

    fn reset_slots(&mut self) -> bool {
        self.slot_values = self.default_slots.clone();
        self.slots_dirty = true;
        true
    }

    fn set_quality(&mut self, quality: u8) -> Result<(), LottieRendererError> {
        self.get_animation_mut()?
            .set_quality(quality)
            .map_err(into_lottie::<R>)
    }

    fn updated(&self) -> bool {
        self.updated
    }

    fn tween(&mut self, from: f32, to: f32, progress: f32) -> Result<(), LottieRendererError> {
        self.get_animation_mut()?
            .tween(from, to, progress)
            .map_err(into_lottie::<R>)?;
        self.updated = true;
        Ok(())
    }

    fn sync_current_frame(&mut self, frame: f32) {
        self.current_frame = frame;
    }

    fn set_layout(&mut self, layout: &Layout) -> Result<(), LottieRendererError> {
        if self.layout == *layout {
            return Ok(());
        }

        self.layout = *layout;

        if self.animation.is_some() {
            self.apply_user_transform()?;
        }

        Ok(())
    }

    fn hit_test(&self, point: Point, layer_name: &str) -> Result<bool, LottieRendererError> {
        self.get_animation()?
            .hit_test(point, layer_name)
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

    // ── Markers & Segments ───────────────────────────────────────────────

    fn markers(&self) -> &[Marker] {
        static EMPTY: &[Marker] = &[];
        self.animation.as_ref().map_or(EMPTY, |a| a.markers())
    }

    fn set_segment(&mut self, segment: Option<Segment>) -> Result<(), LottieRendererError> {
        if let Some(Segment { start, end }) = segment {
            if start >= end {
                return Err(LottieRendererError::InvalidArgument);
            }
        }
        if let Some(a) = self.animation.as_mut() {
            a.set_segment(segment);
        }
        Ok(())
    }

    fn segment(&self) -> Result<Segment, LottieRendererError> {
        self.get_animation()?.segment().map_err(into_lottie::<R>)
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
