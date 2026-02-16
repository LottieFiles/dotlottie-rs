#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, CStr};
use std::slice;

#[cfg(feature = "state-machines")]
use crate::actions::open_url_policy::OpenUrlPolicy;

use crate::lottie_renderer::{
    ColorSlot, ImageSlot, PositionSlot, ScalarSlot, TextDocument, TextSlot, VectorSlot,
};
#[cfg(feature = "state-machines")]
use crate::state_machine_engine::events::Event;
#[cfg(feature = "state-machines")]
use crate::StateMachineEngine;
use crate::{Config, DotLottiePlayer, LayerBoundingBox};

use types::*;

pub mod types;

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
#[cfg(feature = "state-machines")]
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
#[cfg(feature = "theming")]
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
#[cfg(feature = "state-machines")]
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
pub unsafe extern "C" fn dotlottie_buffer_ptr(
    ptr: *mut DotLottiePlayer,
    result: *mut *const u32,
) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.buffer().as_ptr();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_buffer_len(ptr: *mut DotLottiePlayer, result: *mut u64) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.buffer().len() as u64;
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_INVALID_PARAMETER
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
///       dotlottie_buffer_ptr(player, &buffer);
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
#[cfg(feature = "theming")]
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
#[cfg(feature = "theming")]
pub unsafe extern "C" fn dotlottie_reset_theme(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.reset_theme())
    })
}

#[no_mangle]
#[cfg(feature = "theming")]
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
#[cfg(feature = "theming")]
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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
/// Tick the state machine (advances animation and processes state logic)
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_tick(sm: *mut StateMachineEngine<'static>) -> i32 {
    exec_state_machine_op!(sm, |state_machine| to_exit_status(state_machine.tick()))
}

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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
#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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
#[cfg(feature = "state-machines")]

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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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

#[cfg(feature = "state-machines")]
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
            #[cfg(feature = "state-machines")]
            Err(_) => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

/// Get current state name
#[cfg(feature = "state-machines")]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_current_state(
    sm: *mut StateMachineEngine<'static>,
    result: *mut c_char,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        let current_state = state_machine.get_current_state_name();
        #[cfg(feature = "state-machines")]
        to_exit_status(
            DotLottieString::copy(&current_state, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok(),
        )
    })
}

/// Get state machine status
#[cfg(feature = "state-machines")]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_status(
    sm: *mut StateMachineEngine<'static>,
    result: *mut c_char,
) -> i32 {
    exec_state_machine_op!(sm, |state_machine| {
        let status = state_machine.status();
        #[cfg(feature = "state-machines")]
        to_exit_status(DotLottieString::copy(&status, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok())
    })
}

/// Get interaction types for framework setup
///
/// Returns bit flags indicating which interaction types are needed.
/// Frameworks should register listeners for the returned interaction types.
#[cfg(feature = "state-machines")]
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
#[cfg(feature = "state-machines")]
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
#[cfg(feature = "state-machines")]
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
#[cfg(feature = "state-machines")]
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
