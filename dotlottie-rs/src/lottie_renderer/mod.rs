use crate::Layout;
use std::ffi::CStr;

mod renderer;
pub mod slots;

#[cfg(feature = "tvg")]
mod fallback_font;

#[cfg(feature = "tvg")]
mod thorvg;

pub use renderer::{
    Animation, ColorSpace, Drawable, GlContext, Renderer, Shape, WgpuDevice, WgpuInstance,
    WgpuTarget, WgpuTargetType,
};
pub use slots::{
    slots_from_json_string, Bezier, BezierValue, ColorSlot, ColorValue, GradientSlot, GradientStop,
    ImageSlot, LottieKeyframe, LottieProperty, PositionSlot, ScalarSlot, ScalarValue, SlotType,
    TextCaps, TextDocument, TextJustify, TextKeyframe, TextSlot, VectorSlot,
};
#[cfg(feature = "tvg")]
pub use thorvg::{TvgAnimation, TvgEngine, TvgError, TvgRenderer, TvgShape};

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

    /// # Safety
    ///
    /// `context` must be a valid pointer to an OpenGL context. The context must
    /// remain valid for the lifetime of rendering operations using this target.
    unsafe fn set_gl_target(
        &mut self,
        context: *mut std::ffi::c_void,
        id: i32,
        width: u32,
        height: u32,
    ) -> Result<(), LottieRendererError>;

    /// # Safety
    ///
    /// `device` must be a valid pointer to a WebGPU device, `instance` must be a valid
    /// pointer to a WebGPU instance, and `target` must be a valid pointer to a WebGPU
    /// render target. All pointers must remain valid for the lifetime of rendering
    /// operations using this target.
    #[allow(clippy::too_many_arguments)]
    unsafe fn set_wg_target(
        &mut self,
        device: *mut std::ffi::c_void,
        instance: *mut std::ffi::c_void,
        target: *mut std::ffi::c_void,
        width: u32,
        height: u32,
        target_type: WgpuTargetType,
    ) -> Result<(), LottieRendererError>;

    fn load_data(
        &mut self,
        data: &CStr,
        width: u32,
        height: u32,
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

    fn set_background_color(&mut self, hex_color: u32) -> Result<(), LottieRendererError>;

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

    fn load_font(&mut self, name: &str, data: &[u8]) -> Result<(), LottieRendererError>;

    fn unload_font(&mut self, name: &str) -> Result<(), LottieRendererError>;
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
            slot_codes: BTreeMap::new(),
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
    background_color: u32,
    buffer: Vec<u32>,
    layout: Layout,
    user_transform: [f32; 9],
    /// Maps slot_id -> ThorVG slot code for per-slot lifecycle management
    slot_codes: BTreeMap<String, u32>,
    /// Maps slot_id -> SlotType for value retrieval (get operations)
    slot_values: BTreeMap<String, SlotType>,
    default_slots: BTreeMap<String, SlotType>,
}

impl<R: Renderer> LottieRendererImpl<R> {
    fn clear(&mut self) -> Result<(), LottieRendererError> {
        if self.animation.is_some() || self.background_shape.is_some() {
            self.renderer.clear().map_err(into_lottie::<R>)?;
            self.animation = None;
            self.background_shape = None;
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

    /// Apply a single slot by generating its JSON, deleting any old code, and applying the new one
    fn apply_single_slot(
        &mut self,
        slot_id: &str,
        slot_type: &SlotType,
    ) -> Result<(), LottieRendererError> {
        // 1. Delete old slot code if it exists
        if let Some(old_code) = self.slot_codes.remove(slot_id) {
            let _ = self.get_animation_mut()?.del_slot(old_code);
        }

        // 2. Generate JSON for just this one slot: {"slot_id": {"p": value}}
        let single_slot_json = slots::single_slot_to_json_string(slot_id, slot_type)
            .map_err(|_| LottieRendererError::InvalidArgument)?;

        // 3. Create new slot and get its code
        let new_code = self
            .get_animation_mut()?
            .gen_slot(&single_slot_json)
            .map_err(into_lottie::<R>)?;

        // 4. Apply the new slot
        self.get_animation_mut()?
            .apply_slot(new_code)
            .map_err(into_lottie::<R>)?;

        // 5. Store the code for future management
        self.slot_codes.insert(slot_id.to_string(), new_code);

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
            .map_err(into_lottie::<R>)
    }

    unsafe fn set_gl_target(
        &mut self,
        context: *mut std::ffi::c_void,
        id: i32,
        width: u32,
        height: u32,
    ) -> Result<(), LottieRendererError> {
        let gl_context = R::GlContext::from_ptr(context);
        self.renderer
            .set_gl_target(&gl_context, id, width, height)
            .map_err(into_lottie::<R>)
    }

    unsafe fn set_wg_target(
        &mut self,
        device: *mut std::ffi::c_void,
        instance: *mut std::ffi::c_void,
        target: *mut std::ffi::c_void,
        width: u32,
        height: u32,
        target_type: WgpuTargetType,
    ) -> Result<(), LottieRendererError> {
        let wgpu_device = R::WgpuDevice::from_ptr(device);
        let wgpu_instance = R::WgpuInstance::from_ptr(instance);
        let wgpu_target = R::WgpuTarget::from_ptr(target);
        self.renderer
            .set_wg_target(&wgpu_device, &wgpu_instance, &wgpu_target, width, height, target_type)
            .map_err(into_lottie::<R>)
    }

    fn load_data(
        &mut self,
        data: &CStr,
        width: u32,
        height: u32,
    ) -> Result<(), LottieRendererError> {
        self.clear()?;

        self.width = width;
        self.height = height;

        // Extract default slot values BEFORE passing to ThorVG, because
        // ThorVG's load_data with copy=false may parse the JSON in-place
        // and mutate the buffer (nulling out string terminators).
        let default_slots = data
            .to_str()
            .map(slots::extract_slots_from_animation)
            .unwrap_or_default();

        let animation = self.load_animation(data)?;

        let background_shape = self.create_background_shape()?;

        self.setup_drawables(&background_shape, &animation)?;

        self.animation = Some(animation);
        self.background_shape = Some(background_shape);
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

    fn set_color_slot(
        &mut self,
        slot_id: &str,
        slot: ColorSlot,
    ) -> Result<(), LottieRendererError> {
        let slot_type = SlotType::Color(slot.clone());
        self.apply_single_slot(slot_id, &slot_type)?;
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Color(slot));
        Ok(())
    }

    fn set_gradient_slot(
        &mut self,
        slot_id: &str,
        slot: GradientSlot,
    ) -> Result<(), LottieRendererError> {
        let slot_type = SlotType::Gradient(slot.clone());
        self.apply_single_slot(slot_id, &slot_type)?;
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Gradient(slot));
        Ok(())
    }

    fn set_image_slot(
        &mut self,
        slot_id: &str,
        slot: ImageSlot,
    ) -> Result<(), LottieRendererError> {
        let slot_type = SlotType::Image(slot.clone());
        self.apply_single_slot(slot_id, &slot_type)?;
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Image(slot));
        Ok(())
    }

    fn set_text_slot(&mut self, slot_id: &str, slot: TextSlot) -> Result<(), LottieRendererError> {
        let slot_type = SlotType::Text(slot.clone());
        self.apply_single_slot(slot_id, &slot_type)?;
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Text(slot));
        Ok(())
    }

    fn set_scalar_slot(
        &mut self,
        slot_id: &str,
        slot: ScalarSlot,
    ) -> Result<(), LottieRendererError> {
        let slot_type = SlotType::Scalar(slot.clone());
        self.apply_single_slot(slot_id, &slot_type)?;
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Scalar(slot));
        Ok(())
    }

    fn set_vector_slot(
        &mut self,
        slot_id: &str,
        slot: VectorSlot,
    ) -> Result<(), LottieRendererError> {
        let slot_type = SlotType::Vector(slot.clone());
        self.apply_single_slot(slot_id, &slot_type)?;
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Vector(slot));
        Ok(())
    }

    fn set_position_slot(
        &mut self,
        slot_id: &str,
        slot: PositionSlot,
    ) -> Result<(), LottieRendererError> {
        let slot_type = SlotType::Position(slot.clone());
        self.apply_single_slot(slot_id, &slot_type)?;
        self.slot_values
            .insert(slot_id.to_string(), SlotType::Position(slot));
        Ok(())
    }

    fn clear_slots(&mut self) -> Result<(), LottieRendererError> {
        // Collect slot codes first to avoid borrow conflict
        let codes: Vec<u32> = self.slot_codes.values().copied().collect();

        // Delete all tracked slot codes from ThorVG
        for code in codes {
            let _ = self.get_animation_mut()?.del_slot(code);
        }
        self.slot_codes.clear();
        self.slot_values.clear();

        // Reset to defaults via ThorVG (0 = reset all)
        self.get_animation_mut()?
            .apply_slot(0)
            .map_err(into_lottie::<R>)?;

        self.updated = true;
        Ok(())
    }

    fn clear_slot(&mut self, slot_id: &str) -> Result<(), LottieRendererError> {
        // Delete the slot code from ThorVG if it exists
        if let Some(code) = self.slot_codes.remove(slot_id) {
            self.get_animation_mut()?
                .del_slot(code)
                .map_err(into_lottie::<R>)?;
        }
        self.slot_values.remove(slot_id);
        self.updated = true;
        Ok(())
    }

    fn set_slots(&mut self, slots: BTreeMap<String, SlotType>) -> Result<(), LottieRendererError> {
        // Clear all existing slots first
        self.clear_slots()?;

        // Apply each slot individually
        for (slot_id, slot_type) in slots {
            self.apply_single_slot(&slot_id, &slot_type)?;
            self.slot_values.insert(slot_id, slot_type);
        }
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
            Some(slot) => match slot {
                SlotType::Color(s) => self.set_color_slot(slot_id, s),
                SlotType::Scalar(s) => self.set_scalar_slot(slot_id, s),
                SlotType::Vector(s) => self.set_vector_slot(slot_id, s),
                SlotType::Position(s) => self.set_position_slot(slot_id, s),
                SlotType::Gradient(s) => self.set_gradient_slot(slot_id, s),
                SlotType::Image(s) => self.set_image_slot(slot_id, s),
                SlotType::Text(s) => self.set_text_slot(slot_id, s),
            },
            None => Err(LottieRendererError::InvalidSlotValue),
        }
    }

    fn store_default_slots(&mut self, slots: BTreeMap<String, SlotType>) {
        self.default_slots = slots.clone();
        self.slot_values = slots;
    }

    fn reset_slot(&mut self, slot_id: &str) -> Result<(), LottieRendererError> {
        match self.default_slots.get(slot_id).cloned() {
            Some(slot) => match slot {
                SlotType::Color(s) => self.set_color_slot(slot_id, s),
                SlotType::Scalar(s) => self.set_scalar_slot(slot_id, s),
                SlotType::Vector(s) => self.set_vector_slot(slot_id, s),
                SlotType::Position(s) => self.set_position_slot(slot_id, s),
                SlotType::Gradient(s) => self.set_gradient_slot(slot_id, s),
                SlotType::Image(s) => self.set_image_slot(slot_id, s),
                SlotType::Text(s) => self.set_text_slot(slot_id, s),
            },
            None => Err(LottieRendererError::SlotNotFound),
        }
    }

    fn reset_slots(&mut self) -> bool {
        for (slot_id, slot) in self.default_slots.clone() {
            let result = match slot {
                SlotType::Color(s) => self.set_color_slot(&slot_id, s),
                SlotType::Scalar(s) => self.set_scalar_slot(&slot_id, s),
                SlotType::Vector(s) => self.set_vector_slot(&slot_id, s),
                SlotType::Position(s) => self.set_position_slot(&slot_id, s),
                SlotType::Gradient(s) => self.set_gradient_slot(&slot_id, s),
                SlotType::Image(s) => self.set_image_slot(&slot_id, s),
                SlotType::Text(s) => self.set_text_slot(&slot_id, s),
            };
            if result.is_err() {
                return false;
            }
        }
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

        self.layout = *layout;

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
