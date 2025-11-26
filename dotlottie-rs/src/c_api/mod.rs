use std::{ffi::c_char, slice};

use crate::actions::open_url_policy::OpenUrlPolicy;
use crate::state_machine_engine::events::Event;
use crate::{Config, DotLottieRuntime, LayerBoundingBox, StateMachineEngine};

use types::*;

pub mod types;

// Allows to wrap every C API call with some additional logic. This is currently used to
// check if the dotlottie player pointer is valid or not
unsafe fn exec_dotlottie_player_op<Op>(ptr: *mut DotLottieRuntime, op: Op) -> i32
where
    Op: Fn(&mut DotLottieRuntime) -> i32,
{
    match ptr.as_mut() {
        Some(dotlottie_player) => op(dotlottie_player),
        _ => DOTLOTTIE_INVALID_PARAMETER,
    }
}

// Helper for StateMachineEngine operations
// Note: Using 'static here is safe because we manage the lifetime manually
// The actual lifetime is tied to the DotLottieRuntime, enforced by Rust's ownership
unsafe fn exec_state_machine_op<Op>(ptr: *mut StateMachineEngine<'static>, op: Op) -> i32
where
    Op: Fn(&mut StateMachineEngine<'static>) -> i32,
{
    match ptr.as_mut() {
        Some(state_machine) => op(state_machine),
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

// Translates rust boolean to C boolean (1 for true, 0 for false)
fn to_bool_i32(result: bool) -> i32 {
    if result {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_new_player(
    ptr: *const DotLottieConfig,
) -> *mut DotLottieRuntime {
    if let Some(dotlottie_config) = ptr.as_ref() {
        if let Ok(config) = dotlottie_config.to_config() {
            let dotlottie_player = Box::new(DotLottieRuntime::new(config, 0));
            return Box::into_raw(dotlottie_player);
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_destroy(ptr: *mut DotLottieRuntime) -> i32 {
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
    ptr: *mut DotLottieRuntime,
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
    ptr: *mut DotLottieRuntime,
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
    ptr: *mut DotLottieRuntime,
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
    ptr: *mut DotLottieRuntime,
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
    ptr: *mut DotLottieRuntime,
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
    ptr: *mut DotLottieRuntime,
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
    ptr: *mut DotLottieRuntime,
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
pub unsafe extern "C" fn dotlottie_manifest_state_machines(
    ptr: *mut DotLottieRuntime,
    result: *mut types::DotLottieManifestStateMachine,
    size: *mut usize,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
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
    ptr: *mut DotLottieRuntime,
    result: *mut *const u32,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.buffer().as_ptr();
            DOTLOTTIE_SUCCESS
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_buffer_len(ptr: *mut DotLottieRuntime, result: *mut u64) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
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
    ptr: *mut DotLottieRuntime,
    result: *mut DotLottieConfig,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        DotLottieConfig::transfer(&dotlottie_player.config(), result)
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_total_frames(
    ptr: *mut DotLottieRuntime,
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
pub unsafe extern "C" fn dotlottie_duration(ptr: *mut DotLottieRuntime, result: *mut f32) -> i32 {
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
    ptr: *mut DotLottieRuntime,
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
pub unsafe extern "C" fn dotlottie_loop_count(ptr: *mut DotLottieRuntime, result: *mut u32) -> i32 {
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
pub unsafe extern "C" fn dotlottie_is_loaded(ptr: *mut DotLottieRuntime) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_bool_i32(dotlottie_player.is_loaded())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_playing(ptr: *mut DotLottieRuntime) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_bool_i32(dotlottie_player.is_playing())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_paused(ptr: *mut DotLottieRuntime) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_bool_i32(dotlottie_player.is_paused())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_stopped(ptr: *mut DotLottieRuntime) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_bool_i32(dotlottie_player.is_stopped())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_play(ptr: *mut DotLottieRuntime) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.play())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_pause(ptr: *mut DotLottieRuntime) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.pause())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_stop(ptr: *mut DotLottieRuntime) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.stop())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_request_frame(
    ptr: *mut DotLottieRuntime,
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
pub unsafe extern "C" fn dotlottie_set_frame(ptr: *mut DotLottieRuntime, no: f32) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.set_frame(no))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_seek(ptr: *mut DotLottieRuntime, no: f32) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.seek(no))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_render(ptr: *mut DotLottieRuntime) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
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
pub unsafe extern "C" fn dotlottie_tick(ptr: *mut DotLottieRuntime) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.tick())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_resize(
    ptr: *mut DotLottieRuntime,
    width: u32,
    height: u32,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.resize(width, height))
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_clear(ptr: *mut DotLottieRuntime) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        dotlottie_player.clear();
        DOTLOTTIE_SUCCESS
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_complete(ptr: *mut DotLottieRuntime) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_bool_i32(dotlottie_player.is_complete())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_theme(
    ptr: *mut DotLottieRuntime,
    theme_id: *const c_char,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Ok(theme_id) = DotLottieString::read(theme_id) {
            to_exit_status(dotlottie_player.set_theme(&theme_id))
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_reset_theme(ptr: *mut DotLottieRuntime) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        to_exit_status(dotlottie_player.reset_theme())
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_theme_data(
    ptr: *mut DotLottieRuntime,
    theme_data: *const c_char,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if let Ok(theme_data) = DotLottieString::read(theme_data) {
            to_exit_status(dotlottie_player.set_theme_data(&theme_data))
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_markers(
    ptr: *mut DotLottieRuntime,
    result: *mut types::DotLottieMarker,
    size: *mut usize,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        DotLottieMarker::transfer_all(&dotlottie_player.markers(), result, size)
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_active_animation_id(
    ptr: *mut DotLottieRuntime,
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
    ptr: *mut DotLottieRuntime,
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
    ptr: *mut DotLottieRuntime,
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
    ptr: *mut DotLottieRuntime,
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
    ptr: *mut DotLottieRuntime,
    picture_width: *mut f32,
    picture_height: *mut f32,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
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
    ptr: *mut DotLottieRuntime,
    layer_name: *const c_char,
    result: *mut LayerBoundingBox,
) -> i32 {
    exec_dotlottie_player_op(ptr, |dotlottie_player| {
        if layer_name.is_null() || result.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }

        if let Ok(layer_name) = DotLottieString::read(layer_name) {
            match dotlottie_player.get_layer_bounds(&layer_name).as_slice() {
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
            }
        } else {
            DOTLOTTIE_INVALID_PARAMETER
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
/// DotLottieRuntimeEvent event;
/// while (dotlottie_poll_event(player, &event) == 1) {
///     switch (event.event_type) {
///         case DotLottieRuntimeEventType_Load:
///             printf("Animation loaded\n");
///             break;
///         case DotLottieRuntimeEventType_Frame:
///             printf("Frame: %f\n", event.data.frame_no);
///             break;
///     }
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn dotlottie_poll_event(
    player: *mut DotLottieRuntime,
    event: *mut types::DotLottieRuntimeEvent,
) -> i32 {
    if player.is_null() || event.is_null() {
        return -1;
    }

    let player = &mut *player;

    match player.poll_event() {
        Some(rust_event) => {
            let c_event = types::DotLottieRuntimeEvent::from(rust_event);
            std::ptr::write(event, c_event);
            1 // Event retrieved
        }
        None => 0, // No events available
    }
}

// ============================================================================
// STATE MACHINE C API
// Separate StateMachineEngine object with lifetime to DotLottieRuntime
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
    runtime: *mut DotLottieRuntime,
    state_machine_id: *const c_char,
) -> *mut StateMachineEngine<'static> {
    if runtime.is_null() {
        return std::ptr::null_mut();
    }

    let runtime_ref = &mut *runtime;

    if let Ok(sm_id) = DotLottieString::read(state_machine_id) {
        match runtime_ref.state_machine_load(&sm_id) {
            Ok(sm) => {
                // Transmute lifetime to 'static for FFI boundary
                // Safety: The C caller must ensure SM is destroyed before Runtime
                let sm_static: StateMachineEngine<'static> = std::mem::transmute(sm);
                Box::into_raw(Box::new(sm_static))
            }
            Err(_) => std::ptr::null_mut(),
        }
    } else {
        std::ptr::null_mut()
    }
}

/// Load a state machine from a JSON definition string
///
/// Returns a pointer to the StateMachineEngine or NULL on error.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_load_data(
    runtime: *mut DotLottieRuntime,
    state_machine_definition: *const c_char,
) -> *mut StateMachineEngine<'static> {
    if runtime.is_null() {
        return std::ptr::null_mut();
    }

    let runtime_ref = &mut *runtime;

    if let Ok(sm_def) = DotLottieString::read(state_machine_definition) {
        match runtime_ref.state_machine_load_data(&sm_def) {
            Ok(sm) => {
                // Transmute lifetime to 'static for FFI boundary
                let sm_static: StateMachineEngine<'static> = std::mem::transmute(sm);
                Box::into_raw(Box::new(sm_static))
            }
            Err(_) => std::ptr::null_mut(),
        }
    } else {
        std::ptr::null_mut()
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
    exec_state_machine_op(sm, |state_machine| {
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
    exec_state_machine_op(sm, |state_machine| {
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
    exec_state_machine_op(sm, |state_machine| to_exit_status(state_machine.tick()))
}

/// Post a pointer/click event to the state machine
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_event(
    sm: *mut StateMachineEngine<'static>,
    event: *const DotLottieEvent,
) -> i32 {
    exec_state_machine_op(sm, |state_machine| {
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
    exec_state_machine_op(sm, |state_machine| {
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
    exec_state_machine_op(sm, |state_machine| {
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
    exec_state_machine_op(sm, |state_machine| {
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
    exec_state_machine_op(sm, |state_machine| {
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
    exec_state_machine_op(sm, |state_machine| {
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
    exec_state_machine_op(sm, |state_machine| {
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
    exec_state_machine_op(sm, |state_machine| {
        if let Ok(event) = DotLottieString::read(event_name) {
            match state_machine.fire(&event, true) {
                Ok(_) => {
                    let _ = state_machine.run_current_state_pipeline();
                    DOTLOTTIE_SUCCESS
                }
                Err(_) => DOTLOTTIE_ERROR,
            }
        } else {
            DOTLOTTIE_INVALID_PARAMETER
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
    exec_state_machine_op(sm, |state_machine| {
        if let Ok(key) = DotLottieString::read(key) {
            let result = state_machine.set_numeric_input(&key, value, true, false);
            if result.is_some() {
                let _ = state_machine.run_current_state_pipeline();
                DOTLOTTIE_SUCCESS
            } else {
                DOTLOTTIE_ERROR
            }
        } else {
            DOTLOTTIE_INVALID_PARAMETER
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
    exec_state_machine_op(sm, |state_machine| {
        match (DotLottieString::read(key), DotLottieString::read(value)) {
            (Ok(key), Ok(value)) => {
                let result = state_machine.set_string_input(&key, &value, true, false);
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
    exec_state_machine_op(sm, |state_machine| {
        if let Ok(key) = DotLottieString::read(key) {
            let result = state_machine.set_boolean_input(&key, value, true, false);
            if result.is_some() {
                let _ = state_machine.run_current_state_pipeline();
                DOTLOTTIE_SUCCESS
            } else {
                DOTLOTTIE_ERROR
            }
        } else {
            DOTLOTTIE_INVALID_PARAMETER
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
    exec_state_machine_op(sm, |state_machine| {
        if result.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }

        if let Ok(key) = DotLottieString::read(key) {
            if let Some(value) = state_machine.get_numeric_input(&key) {
                *result = value;
                DOTLOTTIE_SUCCESS
            } else {
                DOTLOTTIE_ERROR
            }
        } else {
            DOTLOTTIE_INVALID_PARAMETER
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
    exec_state_machine_op(sm, |state_machine| {
        if result.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }

        if let Ok(key) = DotLottieString::read(key) {
            if let Some(value) = state_machine.get_string_input(&key) {
                return to_exit_status(
                    DotLottieString::copy(&value, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok(),
                );
            }
        }
        DOTLOTTIE_ERROR
    })
}

/// Get a boolean input value
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_get_boolean_input(
    sm: *mut StateMachineEngine<'static>,
    key: *const c_char,
    result: *mut bool,
) -> i32 {
    exec_state_machine_op(sm, |state_machine| {
        if result.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }

        if let Ok(key) = DotLottieString::read(key) {
            if let Some(value) = state_machine.get_boolean_input(&key) {
                *result = value;
                DOTLOTTIE_SUCCESS
            } else {
                DOTLOTTIE_ERROR
            }
        } else {
            DOTLOTTIE_INVALID_PARAMETER
        }
    })
}

/// Get current state name
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_current_state(
    sm: *mut StateMachineEngine<'static>,
    result: *mut c_char,
) -> i32 {
    exec_state_machine_op(sm, |state_machine| {
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
    exec_state_machine_op(sm, |state_machine| {
        let status = state_machine.status();
        to_exit_status(DotLottieString::copy(&status, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok())
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
    runtime: *mut DotLottieRuntime,
    state_machine_id: *const c_char,
    result: *mut c_char,
) -> i32 {
    exec_dotlottie_player_op(runtime, |dotlottie_player| {
        if result.is_null() {
            return DOTLOTTIE_INVALID_PARAMETER;
        }

        if let Ok(sm_id) = DotLottieString::read(state_machine_id) {
            if let Some(sm_json) = dotlottie_player.get_state_machine(&sm_id) {
                return to_exit_status(
                    DotLottieString::copy(&sm_json, result, DOTLOTTIE_MAX_STR_LENGTH).is_ok(),
                );
            }
        }
        DOTLOTTIE_ERROR
    })
}
