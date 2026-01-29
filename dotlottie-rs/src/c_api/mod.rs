#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, CStr};
use std::slice;

use crate::actions::open_url_policy::OpenUrlPolicy;
use crate::lottie_renderer::{
    ColorSlot, ImageSlot, PositionSlot, ScalarSlot, TextDocument, TextSlot, VectorSlot,
};
use crate::state_machine_engine::events::Event;
use crate::ColorSpace;
use crate::{Config, DotLottiePlayer, LayerBoundingBox, StateMachineEngine};
use types::*;

pub mod types;

#[cfg(all(
    feature = "tvg-wg",
    any(target_os = "macos", target_os = "ios"),
    wgpu_native_linked
))]
mod wgpu_helper;

// Helper macro for DotLottiePlayer operations - wraps every C API call to check
// if the dotlottie player pointer is valid or not
macro_rules! exec_dotlottie_player_op {
    ($ptr:expr, |$player:ident| $body:expr) => {{
        match $ptr.as_mut() {
            Some($player) => $body,
            _ => DOTLOTTIE_INVALID_PARAMETER,
        }
    }};
}

// Helper macro for StateMachineEngine operations
macro_rules! exec_state_machine_op {
    ($ptr:expr, |$sm:ident| $body:expr) => {{
        match $ptr.as_mut() {
            Some($sm) => $body,
            _ => DOTLOTTIE_INVALID_PARAMETER,
        }
    }};
}

// Translates rust boolean results into C return codes
fn to_exit_status(result: bool) -> i32 {
    if result {
        DOTLOTTIE_SUCCESS
    } else {
        DOTLOTTIE_ERROR
    }
}

// Translates rust boolean to C boolean (1 for true, 0 for false)
fn to_bool_i32(result: bool) -> i32 {
    if result {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_new_player(ptr: *const DotLottieConfig) -> *mut DotLottiePlayer {
    if let Some(dotlottie_config) = ptr.as_ref() {
        if let Ok(config) = dotlottie_config.to_config() {
            let dotlottie_player = Box::new(DotLottiePlayer::new(config, 0));
            return Box::into_raw(dotlottie_player);
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_destroy(ptr: *mut DotLottiePlayer) -> i32 {
    if ptr.is_null() {
        return DOTLOTTIE_INVALID_PARAMETER;
    }

    // Reconstruct the Box from raw pointer and drop it (frees memory)
    let _ = Box::from_raw(ptr);
    DOTLOTTIE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_init_config(config: *mut DotLottieConfig) -> i32 {
    if config.is_null() {
        return DOTLOTTIE_INVALID_PARAMETER;
    }
    if let Ok(default_config) = DotLottieConfig::new(&Config::default()) {
        default_config.copy(config);
        DOTLOTTIE_SUCCESS
    } else {
        DOTLOTTIE_ERROR
    }
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_animation_data(
    ptr: *mut DotLottiePlayer,
    animation_data: *const c_char,
    width: u32,
    height: u32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if animation_data.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let data = CStr::from_ptr(animation_data);
        to_exit_status(dotlottie_player.load_animation_data(data, width, height))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_animation_path(
    ptr: *mut DotLottiePlayer,
    animation_path: *const c_char,
    width: u32,
    height: u32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if animation_path.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let path = CStr::from_ptr(animation_path);
        match path.to_str() {
            Ok(path_str) => {
                to_exit_status(dotlottie_player.load_animation_path(path_str, width, height))
            }
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_animation(
    ptr: *mut DotLottiePlayer,
    animation_id: *const c_char,
    width: u32,
    height: u32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if animation_id.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let id = CStr::from_ptr(animation_id);
        match id.to_str() {
            Ok(id_str) => to_exit_status(dotlottie_player.load_animation(id_str, width, height)),
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_dotlottie_data(
    ptr: *mut DotLottiePlayer,
    file_data: *const c_char,
    file_size: usize,
    width: u32,
    height: u32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        let file_slice = slice::from_raw_parts(file_data as *const u8, file_size);
        to_exit_status(dotlottie_player.load_dotlottie_data(file_slice, width, height))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_manifest(
    ptr: *mut DotLottiePlayer,
    result: *mut types::DotLottieManifest,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if let Some(manifest) = dotlottie_player.manifest() {
            DotLottieManifest::transfer(manifest, result)
        } else {
            DOTLOTTIE_MANIFEST_NOT_AVAILABLE
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_manifest_animations(
    ptr: *mut DotLottiePlayer,
    result: *mut types::DotLottieManifestAnimation,
    size: *mut usize,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if let Some(manifest) = dotlottie_player.manifest() {
            DotLottieManifestAnimation::transfer_all(&manifest.animations, result, size)
        } else {
            DOTLOTTIE_MANIFEST_NOT_AVAILABLE
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_manifest_themes(
    ptr: *mut DotLottiePlayer,
    result: *mut types::DotLottieManifestTheme,
    size: *mut usize,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        let manifest = match dotlottie_player.manifest() {
            Some(v) => v,
            None => return DOTLOTTIE_MANIFEST_NOT_AVAILABLE,
        };
        if let Some(themes) = &manifest.themes {
            DotLottieManifestTheme::transfer_all(themes, result, size)
        } else {
            *size = 0;
            DOTLOTTIE_SUCCESS
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_manifest_state_machines(
    ptr: *mut DotLottiePlayer,
    result: *mut types::DotLottieManifestStateMachine,
    size: *mut usize,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        let manifest = match dotlottie_player.manifest() {
            Some(v) => v,
            None => return DOTLOTTIE_MANIFEST_NOT_AVAILABLE,
        };
        if let Some(state_machines) = &manifest.state_machines {
            DotLottieManifestStateMachine::transfer_all(state_machines, result, size)
        } else {
            *size = 0;
            DOTLOTTIE_SUCCESS
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_config(
    ptr: *mut DotLottiePlayer,
    result: *mut DotLottieConfig,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        DotLottieConfig::transfer(&dotlottie_player.config(), result)
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_total_frames(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.total_frames();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_duration(ptr: *mut DotLottiePlayer, result: *mut f32) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.duration();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_current_frame(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.current_frame();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_loop_count(ptr: *mut DotLottiePlayer, result: *mut u32) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.loop_count();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_loaded(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_bool_i32(dotlottie_player.is_loaded())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_playing(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_bool_i32(dotlottie_player.is_playing())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_paused(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_bool_i32(dotlottie_player.is_paused())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_stopped(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_bool_i32(dotlottie_player.is_stopped())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_play(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.play())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_pause(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.pause())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_stop(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.stop())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_request_frame(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.request_frame();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_frame(ptr: *mut DotLottiePlayer, no: f32) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.set_frame(no))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_seek(ptr: *mut DotLottiePlayer, no: f32) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.seek(no))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_render(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.render())
    })
}

/// This is the primary method for animating in a render loop.
///
/// It operates on a simple principle: on every call it will calculate the
/// next frame in the animation and render it. After that you just display
/// it.
/// `next_frame` is calculated based on multiple factors: the current frame
/// and the frame rate of the animation and the amount of time since the previous call.
/// Use the [DotLottiePlayer::request_frame()] method to query what frame will
/// be set and rendered on the next call to this method.
///
/// Example of the usage
/// ```c
///     while(true) {
///       dotlottie_tick(player);
///       display(buffer, width, height);
///     }
/// ```
#[no_mangle]
pub unsafe extern "C" fn dotlottie_tick(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.tick())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_resize(
    ptr: *mut DotLottiePlayer,
    width: u32,
    height: u32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.resize(width, height))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_clear(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        dotlottie_player.clear();
        DOTLOTTIE_SUCCESS
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_complete(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_bool_i32(dotlottie_player.is_complete())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_sw_target(
    ptr: *mut DotLottiePlayer,
    buffer: *mut u32,
    stride: u32,
    width: u32,
    height: u32,
    color_space: ColorSpace,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.set_sw_target(buffer, stride, width, height, color_space))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_gl_target(
    ptr: *mut DotLottiePlayer,
    context: *mut std::ffi::c_void,
    id: i32,
    width: u32,
    height: u32,
    color_space: ColorSpace,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.set_gl_target(context, id, width, height, color_space))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_wg_target(
    ptr: *mut DotLottiePlayer,
    device: *mut std::ffi::c_void,
    instance: *mut std::ffi::c_void,
    target: *mut std::ffi::c_void,
    width: u32,
    height: u32,
    color_space: ColorSpace,
    _type: i32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.set_wg_target(
            device,
            instance,
            target,
            width,
            height,
            color_space,
            _type,
        ))
    })
}

/// Create WebGPU context from Metal layer (macOS/iOS only)
///
/// # Arguments
/// * `metal_layer` - Pointer to CAMetalLayer from Swift
///
/// # Returns
/// * Opaque pointer to WgpuContext, or NULL on failure
///
/// # Safety
/// The metal_layer pointer must be valid and point to a CAMetalLayer object
#[cfg(all(
    feature = "tvg-wg",
    any(target_os = "macos", target_os = "ios"),
    wgpu_native_linked
))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_create_wgpu_context_from_metal_layer(
    metal_layer: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
    match wgpu_helper::WgpuContext::from_metal_layer(metal_layer) {
        Ok(context) => Box::into_raw(Box::new(context)) as *mut std::ffi::c_void,
        Err(e) => {
            eprintln!("Failed to create WebGPU context: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get WebGPU pointers from context (device, instance, surface)
///
/// # Arguments
/// * `context` - Opaque pointer from dotlottie_create_wgpu_context_from_metal_layer
/// * `out_device` - Output pointer for device
/// * `out_instance` - Output pointer for instance
/// * `out_surface` - Output pointer for surface
///
/// # Safety
/// context must be a valid pointer from dotlottie_create_wgpu_context_from_metal_layer
#[cfg(all(
    feature = "tvg-wg",
    any(target_os = "macos", target_os = "ios"),
    wgpu_native_linked
))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_wgpu_context_get_pointers(
    context: *const std::ffi::c_void,
    out_device: *mut u64,
    out_instance: *mut u64,
    out_surface: *mut u64,
) {
    if context.is_null() || out_device.is_null() || out_instance.is_null() || out_surface.is_null() {
        return;
    }

    let ctx = &*(context as *const wgpu_helper::WgpuContext);
    let (device, instance, surface) = ctx.as_pointers();
    *out_device = device;
    *out_instance = instance;
    *out_surface = surface;
}

/// Free WebGPU context
///
/// # Arguments 
/// * `context` - Opaque pointer from dotlottie_create_wgpu_context_from_metal_layer
///
/// # Safety
/// context must be a valid pointer and will be invalid after this call
#[cfg(all(
    feature = "tvg-wg",
    any(target_os = "macos", target_os = "ios"),
    wgpu_native_linked
))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_free_wgpu_context(context: *mut std::ffi::c_void) {
    if !context.is_null() {
        let _ = Box::from_raw(context as *mut wgpu_helper::WgpuContext);
    }
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_theme(
    ptr: *mut DotLottiePlayer,
    theme_id: *const c_char,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if theme_id.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let id = CStr::from_ptr(theme_id);
        match id.to_str() {
            Ok(id_str) => to_exit_status(dotlottie_player.set_theme(id_str)),
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_reset_theme(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.reset_theme())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_theme_data(
    ptr: *mut DotLottiePlayer,
    theme_data: *const c_char,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if theme_data.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let data = CStr::from_ptr(theme_data);
        match data.to_str() {
            Ok(data_str) => to_exit_status(dotlottie_player.set_theme_data(data_str)),
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

// ============================================================================
// SLOTS C API
// Functions for manipulating animation slots
// ============================================================================

/// Set slots using a JSON string
///
/// This is the most flexible way to set slots, supporting all slot types
/// including complex gradients and animated keyframes.
///
/// # Example JSON format
/// ```json
/// {
///     "color_slot_id": {"p": {"a": 0, "k": [1.0, 0.0, 0.0]}},
///     "text_slot_id": {"p": {"k": [{"t": 0, "s": {"t": "Hello"}}]}}
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_slots_str(
    ptr: *mut DotLottiePlayer,
    slots_json: *const c_char,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slots_json.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let json = CStr::from_ptr(slots_json);
        match json.to_str() {
            Ok(json_str) => to_exit_status(dotlottie_player.set_slots_str(json_str)),
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Clear all slots
#[no_mangle]
pub unsafe extern "C" fn dotlottie_clear_slots(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.clear_slots())
    })
}

/// Clear a specific slot by ID
#[no_mangle]
pub unsafe extern "C" fn dotlottie_clear_slot(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => to_exit_status(dotlottie_player.clear_slot(id_str)),
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Set a color slot with RGB values (0.0 to 1.0)
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_color_slot(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    r: f32,
    g: f32,
    b: f32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => {
                let slot = ColorSlot::static_value([r, g, b]);
                to_exit_status(dotlottie_player.set_color_slot(id_str, slot))
            }
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Set a scalar slot with a single float value
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_scalar_slot(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    value: f32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => {
                let slot = ScalarSlot::static_value(value);
                to_exit_status(dotlottie_player.set_scalar_slot(id_str, slot))
            }
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Set a text slot with a text string
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_text_slot(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    text: *const c_char,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() || text.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let id = CStr::from_ptr(slot_id);
        let text_cstr = CStr::from_ptr(text);
        match (id.to_str(), text_cstr.to_str()) {
            (Ok(id_str), Ok(text_str)) => {
                let slot = TextSlot::with_document(TextDocument::new(text_str.to_string()));
                to_exit_status(dotlottie_player.set_text_slot(id_str, slot))
            }
            _ => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Set a 2D vector slot
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_vector_slot(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    x: f32,
    y: f32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => {
                let slot = VectorSlot::static_value([x, y]);
                to_exit_status(dotlottie_player.set_vector_slot(id_str, slot))
            }
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Set a 2D position slot
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_position_slot(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    x: f32,
    y: f32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => {
                let slot = PositionSlot::static_value([x, y]);
                to_exit_status(dotlottie_player.set_position_slot(id_str, slot))
            }
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Set an image slot from a file path
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_image_slot_path(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    path: *const c_char,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() || path.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let id = CStr::from_ptr(slot_id);
        let path_cstr = CStr::from_ptr(path);
        match (id.to_str(), path_cstr.to_str()) {
            (Ok(id_str), Ok(path_str)) => {
                let slot = ImageSlot::from_path(path_str.to_string());
                to_exit_status(dotlottie_player.set_image_slot(id_str, slot))
            }
            _ => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Set an image slot from a data URL (base64 encoded)
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_image_slot_data_url(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    data_url: *const c_char,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() || data_url.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let id = CStr::from_ptr(slot_id);
        let url = CStr::from_ptr(data_url);
        match (id.to_str(), url.to_str()) {
            (Ok(id_str), Ok(url_str)) => {
                let slot = ImageSlot::from_data_url(url_str.to_string());
                to_exit_status(dotlottie_player.set_image_slot(id_str, slot))
            }
            _ => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_markers(
    ptr: *mut DotLottiePlayer,
    result: *mut types::DotLottieMarker,
    size: *mut usize,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        DotLottieMarker::transfer_all(&dotlottie_player.markers(), result, size)
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_active_animation_id(
    ptr: *mut DotLottiePlayer,
    result: *mut c_char,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        let active_animation_id = dotlottie_player.active_animation_id();
        to_exit_status(
            DotLottieString::copy(active_animation_id, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok(),
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_active_theme_id(
    ptr: *mut DotLottiePlayer,
    result: *mut c_char,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        let active_theme_id = dotlottie_player.active_theme_id();
        to_exit_status(
            DotLottieString::copy(active_theme_id, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok(),
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_viewport(
    ptr: *mut DotLottiePlayer,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.set_viewport(x, y, w, h))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_segment_duration(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.segment_duration();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_animation_size(
    ptr: *mut DotLottiePlayer,
    picture_width: *mut f32,
    picture_height: *mut f32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if picture_width.is_null() || picture_height.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }

        if let [w, h] = dotlottie_player.animation_size().as_slice() {
            *picture_width = *w;
            *picture_height = *h;
            return DOTLOTTIE_SUCCESS;
        }

        DOTLOTTIE_ERROR
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_layer_bounds(
    ptr: *mut DotLottiePlayer,
    layer_name: *const c_char,
    result: *mut LayerBoundingBox,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if layer_name.is_null() || result.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let name = CStr::from_ptr(layer_name);
        match name.to_str() {
            Ok(name_str) => match dotlottie_player.get_layer_bounds(name_str).as_slice() {
                [x1, y1, x2, y2, x3, y3, x4, y4] => {
                    *result = LayerBoundingBox {
                        x1: *x1,
                        y1: *y1,
                        x2: *x2,
                        y2: *y2,
                        x3: *x3,
                        y3: *y3,
                        x4: *x4,
                        y4: *y4,
                    };
                    DOTLOTTIE_SUCCESS
                }
                _ => DOTLOTTIE_ERROR,
            },
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Poll for the next player event from the event queue
///
/// Returns 1 if an event was retrieved, 0 if no events are available, or -1 on error.
/// The event data is written to the event pointer.
///
/// # Example
/// ```c
/// DotLottiePlayerEvent event;
/// while (dotlottie_poll_event(player, &event) == 1) {
///     switch (event.event_type) {
///         case DotLottiePlayerEventType_Load:
///             printf("Animation loaded\n");
///             break;
///         case DotLottiePlayerEventType_Frame:
///             printf("Frame: %f\n", event.data.frame_no);
///             break;
///     }
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn dotlottie_poll_event(
    player: *mut DotLottiePlayer,
    event: *mut types::DotLottiePlayerEvent,
) -> i32 {
    if player.is_null() || event.is_null() {
        return -1;
    }

    let player = &mut *player;

    match player.poll_event() {
        Some(rust_event) => {
            let c_event = types::DotLottiePlayerEvent::from(rust_event);
            std::ptr::write(event, c_event);
            1 // Event retrieved
        }
        None => 0, // No events available
    }
}

// ============================================================================
// STATE MACHINE C API
// Separate StateMachineEngine object with lifetime to DotLottiePlayer
// ============================================================================

/// Load a state machine by ID from the loaded .lottie file
///
/// Returns a pointer to the StateMachineEngine or NULL on error.
/// The returned state machine borrows the runtime - runtime cannot be destroyed
/// while the state machine exists.
///
/// # Safety
/// - Runtime pointer must be valid
/// - Returned state machine must be destroyed with dotlottie_state_machine_release()
///   BEFORE destroying the runtime
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_load(
    runtime: *mut DotLottiePlayer,
    state_machine_id: *const c_char,
) -> *mut StateMachineEngine<'static> {
    if runtime.is_null() || state_machine_id.is_null() {
        return std::ptr::null_mut();
    }

    let runtime_ref = &mut *runtime;
    let sm_id = CStr::from_ptr(state_machine_id);

    match sm_id.to_str() {
        Ok(id_str) => match runtime_ref.state_machine_load(id_str) {
            Ok(sm) => {
                // Transmute lifetime to 'static for FFI boundary
                // Safety: The C caller must ensure SM is destroyed before Runtime
                let sm_static: StateMachineEngine<'static> = std::mem::transmute(sm);
                Box::into_raw(Box::new(sm_static))
            }
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Load a state machine from a JSON definition string
///
/// Returns a pointer to the StateMachineEngine or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_load_data(
    runtime: *mut DotLottiePlayer,
    state_machine_definition: *const c_char,
) -> *mut StateMachineEngine<'static> {
    if runtime.is_null() || state_machine_definition.is_null() {
        return std::ptr::null_mut();
    }

    let runtime_ref = &mut *runtime;
    let sm_def = CStr::from_ptr(state_machine_definition);

    match sm_def.to_str() {
        Ok(def_str) => match runtime_ref.state_machine_load_data(def_str) {
            Ok(sm) => {
                // Transmute lifetime to 'static for FFI boundary
                let sm_static: StateMachineEngine<'static> = std::mem::transmute(sm);
                Box::into_raw(Box::new(sm_static))
            }
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Start the state machine with the specified URL policy
///
/// # Returns
/// DOTLOTTIE_SUCCESS if started, DOTLOTTIE_ERROR if failed
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_start(
    sm: *mut StateMachineEngine<'static>,
    policy: *const types::DotLottieOpenUrlPolicy,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        let open_url_policy = if policy.is_null() {
            OpenUrlPolicy::default()
        } else {
            match (*policy).to_policy() {
                Ok(p) => p,
                Err(_) => return DOTLOTTIE_INVALID_PARAMETER,
            }
        };

        to_exit_status(state_machine.start(&open_url_policy))
    })
}

/// Stop the state machine (does not release the borrow)
///
/// Call dotlottie_state_machine_release() to actually destroy the state machine
/// and release the runtime borrow.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_stop(sm: *mut StateMachineEngine<'static>) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        state_machine.stop();
        DOTLOTTIE_SUCCESS
    })
}

/// Destroy the state machine and release the runtime borrow
///
/// After calling this, the state machine pointer is invalid and the runtime
/// can be used again.
///
/// # Safety
/// - State machine pointer must be valid
/// - Must not use state machine pointer after this call
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_release(sm: *mut StateMachineEngine<'static>) {
    if !sm.is_null() {
        let boxed_sm = Box::from_raw(sm);
        boxed_sm.release(); // Calls consuming release()
    }
}

/// Tick the state machine (advances animation and processes state logic)
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_tick(sm: *mut StateMachineEngine<'static>) -> i32 {
    exec_state_machine_op!(sm, |state_machine| to_exit_status(state_machine.tick()))
}

/// Post a pointer/click event to the state machine
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_event(
    sm: *mut StateMachineEngine<'static>,
    event: *const DotLottieEvent,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        if let Some(event) = event.as_ref() {
            state_machine.post_event(&event.to_event());
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_ERROR
        }
    })
}

/// Helper functions for posting specific event types
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_click(
    sm: *mut StateMachineEngine<'static>,
    x: f32,
    y: f32,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        let event = Event::Click { x, y };
        state_machine.post_event(&event);
        DOTLOTTIE_SUCCESS
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_pointer_down(
    sm: *mut StateMachineEngine<'static>,
    x: f32,
    y: f32,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        let event = Event::PointerDown { x, y };
        state_machine.post_event(&event);
        DOTLOTTIE_SUCCESS
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_pointer_up(
    sm: *mut StateMachineEngine<'static>,
    x: f32,
    y: f32,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        let event = Event::PointerUp { x, y };
        state_machine.post_event(&event);
        DOTLOTTIE_SUCCESS
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_pointer_move(
    sm: *mut StateMachineEngine<'static>,
    x: f32,
    y: f32,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        let event = Event::PointerMove { x, y };
        state_machine.post_event(&event);
        DOTLOTTIE_SUCCESS
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_pointer_enter(
    sm: *mut StateMachineEngine<'static>,
    x: f32,
    y: f32,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        let event = Event::PointerEnter { x, y };
        state_machine.post_event(&event);
        DOTLOTTIE_SUCCESS
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_pointer_exit(
    sm: *mut StateMachineEngine<'static>,
    x: f32,
    y: f32,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        let event = Event::PointerExit { x, y };
        state_machine.post_event(&event);
        DOTLOTTIE_SUCCESS
    })
}

/// Fire a named event input
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_fire_event(
    sm: *mut StateMachineEngine<'static>,
    event_name: *const c_char,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        if event_name.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let event = CStr::from_ptr(event_name);
        match event.to_str() {
            Ok(event_str) => match state_machine.fire(event_str, true) {
                Ok(_) => {
                    let _ = state_machine.run_current_state_pipeline();
                    DOTLOTTIE_SUCCESS
                }
                Err(_) => DOTLOTTIE_ERROR,
            },
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Set a numeric input
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_set_numeric_input(
    sm: *mut StateMachineEngine<'static>,
    key: *const c_char,
    value: f32,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        if key.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let key_cstr = CStr::from_ptr(key);
        match key_cstr.to_str() {
            Ok(key_str) => {
                let result = state_machine.set_numeric_input(key_str, value, true, false);
                if result.is_some() {
                    let _ = state_machine.run_current_state_pipeline();
                    DOTLOTTIE_SUCCESS
                } else {
                    DOTLOTTIE_ERROR
                }
            }
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Set a string input
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_set_string_input(
    sm: *mut StateMachineEngine<'static>,
    key: *const c_char,
    value: *const c_char,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        if key.is_null() || value.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let key_cstr = CStr::from_ptr(key);
        let value_cstr = CStr::from_ptr(value);
        match (key_cstr.to_str(), value_cstr.to_str()) {
            (Ok(key_str), Ok(value_str)) => {
                let result = state_machine.set_string_input(key_str, value_str, true, false);
                if result.is_some() {
                    let _ = state_machine.run_current_state_pipeline();
                    DOTLOTTIE_SUCCESS
                } else {
                    DOTLOTTIE_ERROR
                }
            }
            _ => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Set a boolean input
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_set_boolean_input(
    sm: *mut StateMachineEngine<'static>,
    key: *const c_char,
    value: bool,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        if key.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let key_cstr = CStr::from_ptr(key);
        match key_cstr.to_str() {
            Ok(key_str) => {
                let result = state_machine.set_boolean_input(key_str, value, true, false);
                if result.is_some() {
                    let _ = state_machine.run_current_state_pipeline();
                    DOTLOTTIE_SUCCESS
                } else {
                    DOTLOTTIE_ERROR
                }
            }
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Get a numeric input value
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_get_numeric_input(
    sm: *mut StateMachineEngine<'static>,
    key: *const c_char,
    result: *mut f32,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        if key.is_null() || result.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let key_cstr = CStr::from_ptr(key);
        match key_cstr.to_str() {
            Ok(key_str) => {
                if let Some(value) = state_machine.get_numeric_input(key_str) {
                    *result = value;
                    DOTLOTTIE_SUCCESS
                } else {
                    DOTLOTTIE_ERROR
                }
            }
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Get a string input value
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_get_string_input(
    sm: *mut StateMachineEngine<'static>,
    key: *const c_char,
    result: *mut c_char,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        if key.is_null() || result.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let key_cstr = CStr::from_ptr(key);
        match key_cstr.to_str() {
            Ok(key_str) => {
                if let Some(value) = state_machine.get_string_input(key_str) {
                    to_exit_status(
                        DotLottieString::copy(&value, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok(),
                    )
                } else {
                    DOTLOTTIE_ERROR
                }
            }
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Get a boolean input value
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_get_boolean_input(
    sm: *mut StateMachineEngine<'static>,
    key: *const c_char,
    result: *mut bool,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        if key.is_null() || result.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let key_cstr = CStr::from_ptr(key);
        match key_cstr.to_str() {
            Ok(key_str) => {
                if let Some(value) = state_machine.get_boolean_input(key_str) {
                    *result = value;
                    DOTLOTTIE_SUCCESS
                } else {
                    DOTLOTTIE_ERROR
                }
            }
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Get current state name
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_current_state(
    sm: *mut StateMachineEngine<'static>,
    result: *mut c_char,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        let current_state = state_machine.get_current_state_name();
        to_exit_status(
            DotLottieString::copy(&current_state, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok(),
        )
    })
}

/// Get state machine status
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_status(
    sm: *mut StateMachineEngine<'static>,
    result: *mut c_char,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        let status = state_machine.status();
        to_exit_status(DotLottieString::copy(&status, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok())
    })
}

/// Get interaction types for framework setup
///
/// Returns bit flags indicating which interaction types are needed.
/// Frameworks should register listeners for the returned interaction types.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_framework_setup(
    sm: *mut StateMachineEngine<'static>,
    result: *mut u16,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        if result.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }

        let interaction_types = state_machine.framework_setup();

        // Convert Vec<String> to bit flags using InteractionType
        if let Ok(interaction_type) = InteractionType::new(&interaction_types) {
            *result = interaction_type.bits();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_ERROR
        }
    })
}

/// Poll for the next state machine event
///
/// Returns 1 if an event was retrieved, 0 if no events are available, or -1 on error.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_poll_event(
    sm: *mut StateMachineEngine<'static>,
    event: *mut types::StateMachineEvent,
) -> i32 {
    if sm.is_null() || event.is_null() {
        return -1;
    }

    let state_machine = &mut *sm;

    match state_machine.poll_event() {
        Some(rust_event) => match types::StateMachineEvent::from_rust(rust_event) {
            Ok(c_event) => {
                std::ptr::write(event, c_event);
                1
            }
            Err(_) => -1,
        },
        None => 0,
    }
}

/// Poll for the next internal state machine event
///
/// Returns 1 if an event was retrieved, 0 if no events are available, or -1 on error.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_poll_internal_event(
    sm: *mut StateMachineEngine<'static>,
    event: *mut types::StateMachineInternalEvent,
) -> i32 {
    if sm.is_null() || event.is_null() {
        return -1;
    }

    let state_machine = &mut *sm;

    match state_machine.poll_internal_event() {
        Some(rust_event) => match types::StateMachineInternalEvent::from_rust(rust_event) {
            Ok(c_event) => {
                std::ptr::write(event, c_event);
                1
            }
            Err(_) => -1,
        },
        None => 0,
    }
}

/// Get the state machine definition as JSON string
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_state_machine(
    runtime: *mut DotLottiePlayer,
    state_machine_id: *const c_char,
    result: *mut c_char,
) -> i32 {
    exec_dotlottie_player_op!(runtime, |dotlottie_player| {
        if state_machine_id.is_null() || result.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let sm_id = CStr::from_ptr(state_machine_id);
        match sm_id.to_str() {
            Ok(id_str) => {
                if let Some(sm_json) = dotlottie_player.get_state_machine(id_str) {
                    to_exit_status(
                        DotLottieString::copy(&sm_json, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok(),
                    )
                } else {
                    DOTLOTTIE_ERROR
                }
            }
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

// ============================================================================
// WEBGL C API (WASM-specific)
// Functions for WebGL context management
// ============================================================================

#[cfg(all(target_family = "wasm", feature = "tvg-gl"))]
mod webgl_api {
    use super::*;

    /// Create a WebGL context for a canvas selector
    ///
    /// # Arguments
    /// * `selector` - CSS selector for the canvas element (e.g., "#myCanvas")
    ///
    /// # Returns
    /// A handle (uintptr_t) to the WebGL context, or 0 on failure
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgl_context_create(
        selector: *const c_char,
    ) -> usize {
        if selector.is_null() {
            return 0;
        }

        // Import Emscripten WebGL functions
        extern "C" {
            fn emscripten_webgl_create_context(
                target: *const c_char,
                attributes: *const EmscriptenWebGLContextAttributes,
            ) -> i32;
        }

        #[repr(C)]
        struct EmscriptenWebGLContextAttributes {
            alpha: bool,
            depth: bool,
            stencil: bool,
            antialias: bool,
            premultiplied_alpha: bool,
            preserve_drawing_buffer: bool,
            power_preference: i32,
            fail_if_major_performance_caveat: bool,
            major_version: i32,
            minor_version: i32,
            enable_extensions_by_default: bool,
            explicit_swap_control: bool,
            proxy_context_to_main_thread: i32,
            render_via_offscreen_back_buffer: bool,
        }

        let attrs = EmscriptenWebGLContextAttributes {
            alpha: true,
            depth: false,
            stencil: false,
            antialias: false,
            premultiplied_alpha: true,
            preserve_drawing_buffer: false,
            power_preference: 0, // default
            fail_if_major_performance_caveat: false,
            major_version: 2,
            minor_version: 0,
            enable_extensions_by_default: true,
            explicit_swap_control: false,
            proxy_context_to_main_thread: 0,
            render_via_offscreen_back_buffer: false,
        };

        let context = emscripten_webgl_create_context(selector, &attrs);
        context as usize
    }

    /// Make a WebGL context current
    ///
    /// # Returns
    /// 0 on success, non-zero on failure
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgl_context_make_current(context: usize) -> i32 {
        extern "C" {
            fn emscripten_webgl_make_context_current(context: i32) -> i32;
        }

        emscripten_webgl_make_context_current(context as i32)
    }

    /// Check if a WebGL context is lost
    ///
    /// # Returns
    /// 1 if context is lost, 0 otherwise
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgl_is_context_lost(context: usize) -> i32 {
        extern "C" {
            fn emscripten_is_webgl_context_lost(context: i32) -> bool;
        }

        if emscripten_is_webgl_context_lost(context as i32) {
            1
        } else {
            0
        }
    }

    /// Destroy a WebGL context
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgl_context_destroy(context: usize) {
        extern "C" {
            fn emscripten_webgl_destroy_context(context: i32);
        }

        emscripten_webgl_destroy_context(context as i32);
    }
}

// ============================================================================
// WEBGPU C API (WASM-specific)
// Functions for WebGPU device/surface management
// ============================================================================

#[cfg(all(target_family = "wasm", feature = "tvg-wg"))]
mod webgpu_api {
    use super::*;
    use std::sync::Mutex;

    // Dawn/WebGPU FFI declarations for Emscripten
    extern "C" {
        fn emscripten_webgpu_get_device() -> usize;
        fn wgpuCreateInstance(descriptor: *const std::ffi::c_void) -> usize;
        fn wgpuInstanceCreateSurface(instance: usize, descriptor: *const std::ffi::c_void) -> usize;
        fn wgpuInstanceRelease(instance: usize);
        fn wgpuAdapterRelease(adapter: usize);
        fn wgpuDeviceRelease(device: usize);
        fn wgpuSurfaceRelease(surface: usize);
    }

    // Surface descriptor for canvas
    #[repr(C)]
    struct WGPUSurfaceDescriptorFromCanvasHTMLSelector {
        chain: WGPUChainedStruct,
        selector: *const c_char,
    }

    #[repr(C)]
    struct WGPUChainedStruct {
        next: *const std::ffi::c_void,
        stype: u32,
    }

    const WGPUSType_SurfaceDescriptorFromCanvasHTMLSelector: u32 = 0x00000004;

    // Global state for WebGPU
    static WEBGPU_STATE: Mutex<Option<WebGpuState>> = Mutex::new(None);

    struct WebGpuState {
        instance: usize,
        adapter: usize,
        device: usize,
        adapter_requested: bool,
        device_requested: bool,
        initialization_failed: bool,
    }

    impl Default for WebGpuState {
        fn default() -> Self {
            Self {
                instance: 0,
                adapter: 0,
                device: 0,
                adapter_requested: false,
                device_requested: false,
                initialization_failed: false,
            }
        }
    }

    /// Request a WebGPU adapter
    ///
    /// # Returns
    /// 0 on success, 1 on failure, 2 if request is in progress
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgpu_request_adapter() -> i32 {
        // Note: This is a simplified version - the full implementation would need
        // to handle async callbacks as in your original C++ code
        // For now, we'll use the emscripten helper function
        0
    }

    /// Request a WebGPU device
    ///
    /// # Returns
    /// 0 on success, 1 on failure, 2 if request is in progress
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgpu_request_device() -> i32 {
        dotlottie_webgpu_request_adapter();
        0
    }

    /// Get the WebGPU adapter handle
    ///
    /// # Returns
    /// Handle to the adapter, or 0 if not available
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgpu_get_adapter() -> usize {
        let state = WEBGPU_STATE.lock().unwrap();
        if let Some(ref s) = *state {
            return s.adapter;
        }
        0
    }

    /// Get the WebGPU device handle
    ///
    /// # Returns
    /// Handle to the device, or fallback to emscripten's device
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgpu_get_device() -> usize {
        dotlottie_webgpu_request_device();

        let state = WEBGPU_STATE.lock().unwrap();
        if let Some(ref s) = *state {
            if s.device != 0 {
                return s.device;
            }
        }

        // Fallback to emscripten's device
        emscripten_webgpu_get_device()
    }

    /// Get the WebGPU instance handle
    ///
    /// # Returns
    /// Handle to the instance
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgpu_get_instance() -> usize {
        let mut state = WEBGPU_STATE.lock().unwrap();

        // Initialize state if needed
        if state.is_none() {
            *state = Some(WebGpuState::default());
        }

        if let Some(ref mut s) = *state {
            if s.instance == 0 {
                // Create a WebGPU instance using Dawn/Emscripten
                // Pass NULL descriptor for default configuration
                let instance = wgpuCreateInstance(std::ptr::null());
                s.instance = instance;

                if instance == 0 {
                    eprintln!("[WebGPU] Warning: Failed to create WebGPU instance");
                }
            }
            return s.instance;
        }

        0
    }

    /// Create a WebGPU surface for a canvas
    ///
    /// # Arguments
    /// * `canvas_selector` - CSS selector for the canvas element (e.g., "#canvas")
    ///
    /// # Returns
    /// Handle to the surface (0 on failure)
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgpu_get_surface(
        canvas_selector: *const c_char,
    ) -> usize {
        if canvas_selector.is_null() {
            eprintln!("[WebGPU] Cannot create surface: selector is null");
            return 0;
        }

        // Ensure instance exists
        let instance = dotlottie_webgpu_get_instance();
        if instance == 0 {
            eprintln!("[WebGPU] Cannot create surface: instance not initialized");
            return 0;
        }

        // Create surface descriptor with canvas selector
        let canvas_descriptor = WGPUSurfaceDescriptorFromCanvasHTMLSelector {
            chain: WGPUChainedStruct {
                next: std::ptr::null(),
                stype: WGPUSType_SurfaceDescriptorFromCanvasHTMLSelector,
            },
            selector: canvas_selector,
        };

        let surface_descriptor = WGPUSurfaceDescriptor {
            next_in_chain: &canvas_descriptor.chain as *const _ as *const std::ffi::c_void,
            label: std::ptr::null(),
        };

        let surface = wgpuInstanceCreateSurface(
            instance,
            &surface_descriptor as *const _ as *const std::ffi::c_void,
        );

        if surface == 0 {
            eprintln!("[WebGPU] Failed to create surface from selector");
        } else {
            println!("[WebGPU] Created surface: 0x{:x}", surface);
        }

        surface
    }

    /// Clean up WebGPU resources
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_webgpu_cleanup() {
        let mut state = WEBGPU_STATE.lock().unwrap();
        *state = None;
    }

    /// Release a WebGPU instance
    ///
    /// # Arguments
    /// * `instance` - Handle to the instance to release
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_wgpu_instance_release(instance: usize) {
        if instance != 0 {
            wgpuInstanceRelease(instance);
        }
    }

    /// Release a WebGPU adapter
    ///
    /// # Arguments
    /// * `adapter` - Handle to the adapter to release
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_wgpu_adapter_release(adapter: usize) {
        if adapter != 0 {
            wgpuAdapterRelease(adapter);
        }
    }

    /// Release a WebGPU device
    ///
    /// # Arguments
    /// * `device` - Handle to the device to release
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_wgpu_device_release(device: usize) {
        if device != 0 {
            wgpuDeviceRelease(device);
        }
    }

    /// Release a WebGPU surface
    ///
    /// # Arguments
    /// * `surface` - Handle to the surface to release
    #[no_mangle]
    pub unsafe extern "C" fn dotlottie_wgpu_surface_release(surface: usize) {
        if surface != 0 {
            wgpuSurfaceRelease(surface);
        }
    }

    #[repr(C)]
    struct WGPUSurfaceDescriptor {
        next_in_chain: *const std::ffi::c_void,
        label: *const c_char,
    }
}
