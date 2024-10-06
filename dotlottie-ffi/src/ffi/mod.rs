use std::{ffi::c_char, slice};

use dotlottie_rs::{Config, DotLottiePlayer, LayerBoundingBox};
use types::*;

pub mod types;

// Allows to wrap every C API call with some additional logic. This is currently used to
// check if the dotlottie player pointer is valid or not
unsafe fn exec_dotlottie_player_op<Op>(ptr: *mut DotLottiePlayer, op: Op) -> i32
where
    Op: Fn(&DotLottiePlayer) -> i32,
{
    match ptr.as_ref() {
        Some(dotlottie_player) => op(dotlottie_player),
        _ => DOTLOTTIE_INVALID_PARAMETER,
    }
}

// Translates rust boolean results into C return codes
fn to_exit_status(result: bool) -> i32 {
    if result {
        DOTLOTTIE_SUCCESS
    } else {
        DOTLOTTIE_ERROR
    }
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_new_player(ptr: *const DotLottieConfig) -> *mut DotLottiePlayer {
    if let Some(dotlottie_config) = ptr.as_ref() {
        if let Ok(config) = dotlottie_config.to_config() {
            let dotlottie_player = Box::new(DotLottiePlayer::new(config));
            return Box::into_raw(dotlottie_player);
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_destroy(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        std::mem::drop(std::ptr::read(dotlottie_player));
        DOTLOTTIE_SUCCESS
    })
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
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Ok(animation_data) = DotLottieString::read(animation_data) {
            to_exit_status(dotlottie_player.load_animation_data(&animation_data, width, height))
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_animation_path(
    ptr: *mut DotLottiePlayer,
    animation_path: *const c_char,
    width: u32,
    height: u32,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Ok(animation_path) = DotLottieString::read(animation_path) {
            to_exit_status(dotlottie_player.load_animation_path(&animation_path, width, height))
        } else {
            DOTLOTTIE_INVALID_PARAMETER
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
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Ok(animation_id) = DotLottieString::read(animation_id) {
            to_exit_status(dotlottie_player.load_animation(&animation_id, width, height))
        } else {
            DOTLOTTIE_INVALID_PARAMETER
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
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        let file_slice = slice::from_raw_parts(file_data as *const u8, file_size);
        to_exit_status(dotlottie_player.load_dotlottie_data(file_slice, width, height))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_manifest(
    ptr: *mut DotLottiePlayer,
    result: *mut types::DotLottieManifest,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Some(manifest) = dotlottie_player.manifest() {
            DotLottieManifest::transfer(&manifest, result)
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
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
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
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
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
pub unsafe extern "C" fn dotlottie_manifest_theme_animations(
    ptr: *mut DotLottiePlayer,
    theme: *const types::DotLottieManifestTheme,
    result: *mut types::DotLottieManifestThemeAnimation,
    size: *mut usize,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if theme.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let theme = match theme.as_ref() {
            Some(v) => v,
            None => return DOTLOTTIE_INVALID_PARAMETER,
        };
        let theme_id = theme.id.to_string();
        let manifest = match dotlottie_player.manifest() {
            Some(v) => v,
            None => return DOTLOTTIE_MANIFEST_NOT_AVAILABLE,
        };
        let themes = match manifest.themes {
            Some(v) => v,
            None => return DOTLOTTIE_MANIFEST_THEMES_NOT_AVAILABLE,
        };
        if let Some(theme) = themes.iter().find(|&v| v.id == theme_id) {
            DotLottieManifestThemeAnimation::transfer_all(&theme.animations, result, size)
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_manifest_states(
    ptr: *mut DotLottiePlayer,
    result: *mut types::DotLottieManifestState,
    size: *mut usize,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        let manifest = match dotlottie_player.manifest() {
            Some(v) => v,
            None => return DOTLOTTIE_MANIFEST_NOT_AVAILABLE,
        };
        if let Some(states) = manifest.states {
            DotLottieManifestState::transfer_all(&states, result, size)
        } else {
            *size = 0;
            DOTLOTTIE_SUCCESS
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_post_event(
    ptr: *mut DotLottiePlayer,
    event: *const DotLottieEvent,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Some(event) = event.as_ref() {
            dotlottie_player.post_event(&event.to_event())
        } else {
            DOTLOTTIE_ERROR
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_buffer_ptr(
    ptr: *mut DotLottiePlayer,
    result: *mut *const u32,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.buffer();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_buffer_len(ptr: *mut DotLottiePlayer, result: *mut u64) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.buffer_len();
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
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        DotLottieConfig::transfer(&dotlottie_player.config(), result)
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_total_frames(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
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
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
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
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
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
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
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
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.is_loaded())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_playing(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.is_playing())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_paused(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.is_paused())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_stopped(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.is_stopped())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_play(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.play())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_pause(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.pause())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_stop(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.stop())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_request_frame(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
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
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.set_frame(no))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_seek(ptr: *mut DotLottiePlayer, no: f32) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.seek(no))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_render(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.render())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_resize(
    ptr: *mut DotLottiePlayer,
    width: u32,
    height: u32,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.resize(width, height))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_clear(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        dotlottie_player.clear();
        DOTLOTTIE_SUCCESS
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_complete(
    ptr: *mut DotLottiePlayer,
    result: *mut bool,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.is_complete();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_theme(
    ptr: *mut DotLottiePlayer,
    theme_id: *const c_char,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Ok(theme_id) = DotLottieString::read(theme_id) {
            to_exit_status(dotlottie_player.load_theme(&theme_id))
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_theme_data(
    ptr: *mut DotLottiePlayer,
    theme_data: *const c_char,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Ok(theme_data) = DotLottieString::read(theme_data) {
            to_exit_status(dotlottie_player.load_theme_data(&theme_data))
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_markers(
    ptr: *mut DotLottiePlayer,
    result: *mut DotLottieMarker,
    size: *mut usize,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        DotLottieMarker::transfer_all(&dotlottie_player.markers(), result, size)
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_active_animation_id(
    ptr: *mut DotLottiePlayer,
    result: *mut c_char,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        let active_animation_id = dotlottie_player.active_animation_id();
        to_exit_status(
            DotLottieString::copy(&active_animation_id, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok(),
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_active_theme_id(
    ptr: *mut DotLottiePlayer,
    result: *mut c_char,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        let active_theme_id = dotlottie_player.active_theme_id();
        to_exit_status(
            DotLottieString::copy(&active_theme_id, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok(),
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
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.set_viewport(x, y, w, h))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_segment_duration(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
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
    width: *mut f32,
    height: *mut f32,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if width.is_null() || height.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        if let [picture_width, picture_height] = dotlottie_player.animation_size().as_slice() {
            *width = *picture_width;
            *height = *picture_height;
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_ERROR
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_layer_bounds(
    ptr: *mut DotLottiePlayer,
    layer_name: *const c_char,
    bounding_box: *mut LayerBoundingBox,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if bounding_box.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let bounding_box = match bounding_box.as_mut() {
            Some(v) => v,
            None => return DOTLOTTIE_INVALID_PARAMETER,
        };
        let layer_name = match DotLottieString::read(layer_name) {
            Ok(v) => v,
            Err(_) => return DOTLOTTIE_INVALID_PARAMETER,
        };
        match dotlottie_player.get_layer_bounds(&layer_name).as_slice() {
            [x, y, w, h] => {
                bounding_box.x = *x;
                bounding_box.y = *y;
                bounding_box.w = *w;
                bounding_box.h = *h;
                DOTLOTTIE_SUCCESS
            }
            _ => DOTLOTTIE_ERROR,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_state_machine(
    ptr: *mut DotLottiePlayer,
    state_machine_id: *const c_char,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Ok(state_machine_id) = DotLottieString::read(state_machine_id) {
            to_exit_status(dotlottie_player.load_state_machine(&state_machine_id))
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_start_state_machine(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.start_state_machine())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_stop_state_machine(ptr: *mut DotLottiePlayer) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.stop_state_machine())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_state_machine_numeric_context(
    ptr: *mut DotLottiePlayer,
    key: *const c_char,
    value: f32,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Ok(key) = DotLottieString::read(key) {
            to_exit_status(dotlottie_player.set_state_machine_numeric_context(&key, value))
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_state_machine_string_context(
    ptr: *mut DotLottiePlayer,
    key: *const c_char,
    value: *const c_char,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        match (DotLottieString::read(key), DotLottieString::read(value)) {
            (Ok(key), Ok(value)) => {
                to_exit_status(dotlottie_player.set_state_machine_string_context(&key, &value))
            }
            _ => DOTLOTTIE_INVALID_PARAMETER,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_state_machine_boolean_context(
    ptr: *mut DotLottiePlayer,
    key: *const c_char,
    value: bool,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Ok(key) = DotLottieString::read(key) {
            to_exit_status(dotlottie_player.set_state_machine_boolean_context(&key, value))
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_framework_setup(
    ptr: *mut DotLottiePlayer,
    result: *mut u16,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if result.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        let listener_types = dotlottie_player.state_machine_framework_setup();
        if let Ok(listener_type) = ListenerType::new(&listener_types) {
            *result = listener_type.bits();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_ERROR
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_state_machine_data(
    ptr: *mut DotLottiePlayer,
    state_machine_definition: *const c_char,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Ok(state_machine_definition) = DotLottieString::read(state_machine_definition) {
            to_exit_status(dotlottie_player.load_state_machine_data(&state_machine_definition))
        } else {
            DOTLOTTIE_ERROR
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_subscribe(
    ptr: *mut DotLottiePlayer,
    observer: *mut types::Observer,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if observer.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        if let Some(v) = observer.as_mut() {
            dotlottie_player.subscribe(v.as_observer());
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_ERROR
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_unsubscribe(
    ptr: *mut DotLottiePlayer,
    observer: *mut types::Observer,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if observer.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        if let Some(v) = observer.as_mut() {
            dotlottie_player.unsubscribe(&v.as_observer());
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_ERROR
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_subscribe(
    ptr: *mut DotLottiePlayer,
    observer: *mut types::StateMachineObserver,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if observer.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        if let Some(v) = observer.as_mut() {
            to_exit_status(dotlottie_player.state_machine_subscribe(v.as_observer()))
        } else {
            DOTLOTTIE_ERROR
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_unsubscribe(
    ptr: *mut DotLottiePlayer,
    observer: *mut types::StateMachineObserver,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if observer.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }
        if let Some(v) = observer.as_mut() {
            to_exit_status(dotlottie_player.state_machine_unsubscribe(v.as_observer()))
        } else {
            DOTLOTTIE_ERROR
        }
    })
}
