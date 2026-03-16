#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, CStr};
use std::slice;

use crate::lottie_renderer::{
    ColorSlot, ColorValue, GlContext, ImageSlot, PositionSlot, ScalarSlot, ScalarValue,
    TextDocument, TextSlot, VectorSlot, WgpuDevice, WgpuInstance, WgpuTarget,
};
use crate::{DotLottiePlayer, DotLottiePlayerError, LayerBoundingBox, Layout, Mode};

use crate::ColorSpace;

use types::*;

pub mod types;

/// Wrapper for raw OpenGL context pointer that implements GlContext trait
struct RawGlContext(*mut std::ffi::c_void);

impl GlContext for RawGlContext {
    fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }
}

/// Wrapper for raw WebGPU device pointer that implements WgpuDevice trait
struct RawWgpuDevice(*mut std::ffi::c_void);

impl WgpuDevice for RawWgpuDevice {
    fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }
}

/// Wrapper for raw WebGPU instance pointer that implements WgpuInstance trait
struct RawWgpuInstance(*mut std::ffi::c_void);

impl WgpuInstance for RawWgpuInstance {
    fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }
}

/// Wrapper for raw WebGPU target pointer that implements WgpuTarget trait
struct RawWgpuTarget(*mut std::ffi::c_void);

impl WgpuTarget for RawWgpuTarget {
    fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.0
    }

    unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }
}

#[cfg(all(feature = "tvg-wg", target_os = "macos"))]
pub mod apple;

// Helper macro for DotLottiePlayer operations - wraps every C API call to check
// if the dotlottie player pointer is valid or not, and converts the body's
// return value to DotLottieResult
macro_rules! exec_dotlottie_player_op {
    ($ptr:expr, |$player:ident| $body:expr) => {{
        match $ptr.as_mut() {
            Some($player) => DotLottieResult::from($body),
            _ => DotLottieResult::InvalidParameter,
        }
    }};
}

// Helper macro for StateMachineEngine operations
#[cfg(feature = "state-machines")]
macro_rules! exec_state_machine_op {
    ($ptr:expr, |$sm:ident| $body:expr) => {{
        match $ptr.as_mut() {
            Some(wrapper) => {
                let $sm = &mut wrapper.inner;
                DotLottieResult::from($body)
            }
            _ => DotLottieResult::InvalidParameter,
        }
    }};
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_new_player(threads: u32) -> *mut DotLottiePlayer {
    let dotlottie_player = Box::new(DotLottiePlayer::with_threads(threads));
    Box::into_raw(dotlottie_player)
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_font(
    name: *const c_char,
    data: *const u8,
    size: usize,
) -> DotLottieResult {
    if name.is_null() || data.is_null() || size == 0 {
        return DotLottieResult::InvalidParameter;
    }
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return DotLottieResult::InvalidParameter,
    };
    let data = slice::from_raw_parts(data, size);
    DotLottiePlayer::load_font(name, data).into()
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_unload_font(name: *const c_char) -> DotLottieResult {
    if name.is_null() {
        return DotLottieResult::InvalidParameter;
    }
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return DotLottieResult::InvalidParameter,
    };
    DotLottiePlayer::unload_font(name).into()
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_destroy(ptr: *mut DotLottiePlayer) -> DotLottieResult {
    if ptr.is_null() {
        return DotLottieResult::InvalidParameter;
    }

    // Reconstruct the Box from raw pointer and drop it (frees memory)
    let _ = Box::from_raw(ptr);
    DotLottieResult::Success
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_animation_data(
    ptr: *mut DotLottiePlayer,
    animation_data: *const c_char,
    width: u32,
    height: u32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if animation_data.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let data = CStr::from_ptr(animation_data);
        dotlottie_player.load_animation_data(data, width, height)
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_animation_path(
    ptr: *mut DotLottiePlayer,
    animation_path: *const c_char,
    width: u32,
    height: u32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if animation_path.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let path = CStr::from_ptr(animation_path);
        dotlottie_player.load_animation_path(path, width, height)
    })
}

#[cfg_attr(not(feature = "dotlottie"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_animation(
    ptr: *mut DotLottiePlayer,
    animation_id: *const c_char,
    width: u32,
    height: u32,
) -> DotLottieResult {
    #[cfg(not(feature = "dotlottie"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "dotlottie")]
    {
        exec_dotlottie_player_op!(ptr, |dotlottie_player| {
            if animation_id.is_null() {
                return DotLottieResult::InvalidParameter;
            }
            let id = CStr::from_ptr(animation_id);
            dotlottie_player.load_animation(id, width, height)
        })
    }
}

#[cfg_attr(not(feature = "dotlottie"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_load_dotlottie_data(
    ptr: *mut DotLottiePlayer,
    file_data: *const c_char,
    file_size: usize,
    width: u32,
    height: u32,
) -> DotLottieResult {
    #[cfg(not(feature = "dotlottie"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "dotlottie")]
    {
        exec_dotlottie_player_op!(ptr, |dotlottie_player| {
            if file_data.is_null() || file_size == 0 {
                return DotLottieResult::InvalidParameter;
            }
            let file_slice = slice::from_raw_parts(file_data as *const u8, file_size);
            dotlottie_player.load_dotlottie_data(file_slice, width, height)
        })
    }
}

/// Get the manifest as a JSON string.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
/// - `buffer`: Buffer to store the JSON, or NULL to query required size
/// - `size_out`: Pointer to receive the required buffer size (including null terminator)
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::ManifestNotAvailable` if no manifest is available
/// - `DotLottieResult::InvalidParameter` if ptr is invalid
/// - `DotLottieResult::FeatureNotEnabled` if built without the `dotlottie` feature
/// - `DotLottieResult::Error` if the manifest cannot be serialized to JSON
#[cfg_attr(not(feature = "dotlottie"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_manifest(
    ptr: *mut DotLottiePlayer,
    buffer: *mut c_char,
    size_out: *mut usize,
) -> DotLottieResult {
    #[cfg(not(feature = "dotlottie"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "dotlottie")]
    {
        exec_dotlottie_player_op!(ptr, |dotlottie_player| {
            match dotlottie_player.manifest() {
                Some(manifest) => {
                    let json_str = match serde_json::to_string(manifest) {
                        Ok(s) => s,
                        Err(_) => return DotLottieResult::Error,
                    };
                    let json_bytes = json_str.as_bytes();
                    let size = json_bytes.len() + 1; // +1 for null terminator

                    if !size_out.is_null() {
                        *size_out = size;
                    }

                    if !buffer.is_null() {
                        std::ptr::copy_nonoverlapping(
                            json_bytes.as_ptr() as *const c_char,
                            buffer,
                            json_bytes.len(),
                        );
                        // Add null terminator
                        *buffer.add(json_bytes.len()) = 0;
                    }

                    DotLottieResult::Success
                }
                None => DotLottieResult::ManifestNotAvailable,
            }
        })
    }
}

// ============================================================================
// INDIVIDUAL CONFIG SETTERS
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_mode(
    ptr: *mut DotLottiePlayer,
    mode: Mode,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        dotlottie_player.set_mode(mode);
        DotLottieResult::Success
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_speed(
    ptr: *mut DotLottiePlayer,
    speed: f32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        dotlottie_player.set_speed(speed);
        DotLottieResult::Success
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_loop(
    ptr: *mut DotLottiePlayer,
    loop_animation: bool,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        dotlottie_player.set_loop(loop_animation);
        DotLottieResult::Success
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_loop_count(
    ptr: *mut DotLottiePlayer,
    loop_count: u32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        dotlottie_player.set_loop_count(loop_count);
        DotLottieResult::Success
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_autoplay(
    ptr: *mut DotLottiePlayer,
    autoplay: bool,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        dotlottie_player.set_autoplay(autoplay);
        DotLottieResult::Success
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_use_frame_interpolation(
    ptr: *mut DotLottiePlayer,
    enabled: bool,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        dotlottie_player.set_use_frame_interpolation(enabled);
        DotLottieResult::Success
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_background_color(
    ptr: *mut DotLottiePlayer,
    color: u32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        dotlottie_player.set_background_color(Some(color))
    })
}

/// Sets the playback segment for the animation.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
/// - `segment`: Pointer to an array of 2 floats [start_frame, end_frame], or NULL to clear
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::InvalidParameter` if the player pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_segment(
    ptr: *mut DotLottiePlayer,
    segment: *const [f32; 2],
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        let segment_opt = if segment.is_null() {
            None
        } else {
            Some(*segment)
        };
        dotlottie_player.set_segment(segment_opt)
    })
}

/// Sets the active marker for the animation.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
/// - `marker`: Pointer to a null-terminated C string with the marker name, or NULL to clear
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::InvalidParameter` if the player pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_marker(
    ptr: *mut DotLottiePlayer,
    marker: *const c_char,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        let marker_cstr = if marker.is_null() {
            None
        } else {
            Some(CStr::from_ptr(marker))
        };
        dotlottie_player.set_marker(marker_cstr);
        DotLottieResult::Success
    })
}

/// Sets the layout configuration for the animation.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
/// - `layout`: Layout configuration (fit mode and alignment)
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::InvalidParameter` if the player pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_layout(
    ptr: *mut DotLottiePlayer,
    layout: Layout,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        dotlottie_player.set_layout(layout)
    })
}

// ============================================================================
// INDIVIDUAL CONFIG GETTERS
// ============================================================================

/// Returns the current playback mode.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
///
/// # Returns
/// The current Mode, or Mode::Forward if the pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_mode(ptr: *mut DotLottiePlayer) -> Mode {
    match ptr.as_mut() {
        Some(p) => p.mode(),
        _ => Mode::Forward,
    }
}

/// Returns the current playback speed.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
///
/// # Returns
/// The current speed multiplier, or 1.0 if the pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_speed(ptr: *mut DotLottiePlayer) -> f32 {
    match ptr.as_mut() {
        Some(p) => p.speed(),
        _ => 1.0,
    }
}

/// Returns whether looping is enabled.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
///
/// # Returns
/// true if looping is enabled, false otherwise or if the pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_loop(ptr: *mut DotLottiePlayer) -> bool {
    match ptr.as_mut() {
        Some(p) => p.loop_animation(),
        _ => false,
    }
}

/// Returns the configured loop count (0 = infinite).
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
///
/// # Returns
/// The configured loop count, or 0 if the pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_loop_count(ptr: *mut DotLottiePlayer) -> u32 {
    match ptr.as_mut() {
        Some(p) => p.loop_count(),
        _ => 0,
    }
}

/// Returns whether autoplay is enabled.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
///
/// # Returns
/// true if autoplay is enabled, false otherwise or if the pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_autoplay(ptr: *mut DotLottiePlayer) -> bool {
    match ptr.as_mut() {
        Some(p) => p.autoplay(),
        _ => false,
    }
}

/// Returns whether frame interpolation is enabled.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
///
/// # Returns
/// true if frame interpolation is enabled, false otherwise or if the pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_use_frame_interpolation(ptr: *mut DotLottiePlayer) -> bool {
    match ptr.as_mut() {
        Some(p) => p.use_frame_interpolation(),
        _ => false,
    }
}

/// Returns the current background color.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
///
/// # Returns
/// The background color as ARGB u32, or 0 if the pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_background_color(ptr: *mut DotLottiePlayer) -> u32 {
    match ptr.as_mut() {
        Some(p) => p.background_color(),
        _ => 0,
    }
}

/// Returns the current segment.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
/// - `result`: Pointer to a [f32; 2] array to store [start_frame, end_frame]
///
/// # Returns
/// - `DotLottieResult::Success` if segment exists and was copied
/// - `DotLottieResult::InvalidParameter` if pointers are invalid or no segment is set
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_segment(
    ptr: *mut DotLottiePlayer,
    result: *mut [f32; 2],
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if result.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        match dotlottie_player.segment() {
            Some(segment) => {
                *result = segment;
                DotLottieResult::Success
            }
            None => DotLottieResult::InvalidParameter,
        }
    })
}

/// Returns the current marker name.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
/// - `buffer`: Buffer to store the marker name, or NULL to query required size
/// - `size_out`: Pointer to receive the required buffer size (including null terminator)
///
/// # Usage
/// ```c
/// size_t size;
/// dotlottie_get_active_marker(player, NULL, &size);  // get required size
/// char* buf = malloc(size);
/// dotlottie_get_active_marker(player, buf, NULL);    // get string
/// ```
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::InvalidParameter` if no marker is set or player pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_active_marker(
    ptr: *mut DotLottiePlayer,
    buffer: *mut c_char,
    size_out: *mut usize,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        match dotlottie_player.marker() {
            Some(marker) => {
                let marker_bytes = marker.to_bytes_with_nul();
                let size = marker_bytes.len();

                if !size_out.is_null() {
                    *size_out = size;
                }

                if !buffer.is_null() {
                    std::ptr::copy_nonoverlapping(
                        marker_bytes.as_ptr() as *const c_char,
                        buffer,
                        size,
                    );
                }

                DotLottieResult::Success
            }
            None => DotLottieResult::InvalidParameter,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_layout(
    ptr: *mut DotLottiePlayer,
    result: *mut Layout,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if result.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        *result = *dotlottie_player.layout();
        DotLottieResult::Success
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_total_frames(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.total_frames();
            DotLottieResult::Success
        } else {
            DotLottieResult::InvalidParameter
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_duration(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.duration();
            DotLottieResult::Success
        } else {
            DotLottieResult::InvalidParameter
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_current_frame(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.current_frame();
            DotLottieResult::Success
        } else {
            DotLottieResult::InvalidParameter
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_current_loop_count(
    ptr: *mut DotLottiePlayer,
    result: *mut u32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.current_loop_count();
            DotLottieResult::Success
        } else {
            DotLottieResult::InvalidParameter
        }
    })
}

/// Returns whether an animation is loaded.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_loaded(ptr: *mut DotLottiePlayer) -> bool {
    match ptr.as_mut() {
        Some(p) => p.is_loaded(),
        _ => false,
    }
}

/// Returns the current playback status.
///
/// Priority order: Playing > Paused > Stopped
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
///
/// # Returns
/// The current PlaybackStatus (Playing, Paused, or Stopped)
/// Returns Stopped if the pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_playback_status(ptr: *mut DotLottiePlayer) -> PlaybackStatus {
    match ptr.as_mut() {
        Some(p) => {
            if p.is_playing() {
                PlaybackStatus::Playing
            } else if p.is_paused() {
                PlaybackStatus::Paused
            } else {
                PlaybackStatus::Stopped
            }
        }
        _ => PlaybackStatus::Stopped,
    }
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_play(ptr: *mut DotLottiePlayer) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| dotlottie_player.play())
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_pause(ptr: *mut DotLottiePlayer) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| dotlottie_player.pause())
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_stop(ptr: *mut DotLottiePlayer) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| dotlottie_player.stop())
}


#[no_mangle]
pub unsafe extern "C" fn dotlottie_mute_audio(ptr: *mut DotLottiePlayer) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        #[cfg(feature = "audio")]
        {
            dotlottie_player.mute_audio();
            DotLottieResult::Success
        }
        #[cfg(not(feature = "audio"))]
        {
            let _ = dotlottie_player;
            DotLottieResult::FeatureNotEnabled
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_unmute_audio(ptr: *mut DotLottiePlayer) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        #[cfg(feature = "audio")]
        {
            dotlottie_player.unmute_audio();
            DotLottieResult::Success
        }
        #[cfg(not(feature = "audio"))]
        {
            let _ = dotlottie_player;
            DotLottieResult::FeatureNotEnabled
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_audio_volume(
    ptr: *mut DotLottiePlayer,
    volume: f32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        #[cfg(feature = "audio")]
        {
            dotlottie_player.set_audio_volume(volume);
            DotLottieResult::Success
        }
        #[cfg(not(feature = "audio"))]
        {
            let _ = (dotlottie_player, volume);
            DotLottieResult::FeatureNotEnabled
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_audio_muted(
    ptr: *mut DotLottiePlayer,
    result: *mut bool,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if result.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        #[cfg(feature = "audio")]
        {
            *result = dotlottie_player.is_audio_muted();
            DotLottieResult::Success
        }
        #[cfg(not(feature = "audio"))]
        {
            let _ = dotlottie_player;
            DotLottieResult::FeatureNotEnabled
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_audio_volume(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if result.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        #[cfg(feature = "audio")]
        {
            *result = dotlottie_player.audio_volume();
            DotLottieResult::Success
        }
        #[cfg(not(feature = "audio"))]
        {
            let _ = dotlottie_player;
            DotLottieResult::FeatureNotEnabled
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_request_frame(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.request_frame();
            DotLottieResult::Success
        } else {
            DotLottieResult::InvalidParameter
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_frame(
    ptr: *mut DotLottiePlayer,
    no: f32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| dotlottie_player.set_frame(no))
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_seek(ptr: *mut DotLottiePlayer, no: f32) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| dotlottie_player.seek(no))
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_render(ptr: *mut DotLottiePlayer) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| dotlottie_player.render())
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
pub unsafe extern "C" fn dotlottie_tick(ptr: *mut DotLottiePlayer) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| dotlottie_player.tick())
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_resize(
    ptr: *mut DotLottiePlayer,
    width: u32,
    height: u32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        dotlottie_player.resize(width, height)
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_clear(ptr: *mut DotLottiePlayer) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        dotlottie_player.clear();
        DotLottieResult::Success
    })
}

/// Returns whether the animation has completed playback.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_is_complete(ptr: *mut DotLottiePlayer) -> bool {
    match ptr.as_mut() {
        Some(p) => p.is_complete(),
        _ => false,
    }
}

/// Sets the software rendering target.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
/// - `buffer`: Pointer to the pixel buffer (must be width * height in size)
/// - `width`: Width of the buffer in pixels
/// - `height`: Height of the buffer in pixels
/// - `color_space`: Color space for the buffer
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::InvalidParameter` if buffer is too small or pointer is invalid
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_sw_target(
    ptr: *mut DotLottiePlayer,
    buffer: *mut u32,
    width: u32,
    height: u32,
    color_space: ColorSpace,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        let buffer_slice = std::slice::from_raw_parts_mut(buffer, (width * height) as usize);
        dotlottie_player.set_sw_target(buffer_slice, width, height, color_space)
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_gl_target(
    ptr: *mut DotLottiePlayer,
    context: *mut std::ffi::c_void,
    id: i32,
    width: u32,
    height: u32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        let gl_context = RawGlContext(context);
        dotlottie_player.set_gl_target(&gl_context, id, width, height)
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
    target_type: DotLottieWgpuTargetType,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        let wgpu_device = RawWgpuDevice(device);
        let wgpu_instance = RawWgpuInstance(instance);
        let wgpu_target = RawWgpuTarget(target);
        dotlottie_player.set_wg_target(
            &wgpu_device,
            &wgpu_instance,
            &wgpu_target,
            width,
            height,
            target_type.to_wgpu_target_type(),
        )
    })
}

#[cfg_attr(not(feature = "theming"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_theme(
    ptr: *mut DotLottiePlayer,
    theme_id: *const c_char,
) -> DotLottieResult {
    #[cfg(not(feature = "theming"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "theming")]
    {
        exec_dotlottie_player_op!(ptr, |dotlottie_player| {
            if theme_id.is_null() {
                return DotLottieResult::InvalidParameter;
            }
            let id = CStr::from_ptr(theme_id);
            dotlottie_player.set_theme(id)
        })
    }
}

#[cfg_attr(not(feature = "theming"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_reset_theme(ptr: *mut DotLottiePlayer) -> DotLottieResult {
    #[cfg(not(feature = "theming"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "theming")]
    {
        exec_dotlottie_player_op!(ptr, |dotlottie_player| dotlottie_player.reset_theme())
    }
}

/// Sets the theme using raw theme data.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
/// - `theme_data`: Null-terminated C string containing the theme JSON data
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::FeatureNotEnabled` if built without the `theming` feature
/// - `DotLottieResult::InvalidParameter` if the data is invalid or pointer is invalid
#[cfg_attr(not(feature = "theming"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_theme_data(
    ptr: *mut DotLottiePlayer,
    theme_data: *const c_char,
) -> DotLottieResult {
    #[cfg(not(feature = "theming"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "theming")]
    {
        exec_dotlottie_player_op!(ptr, |dotlottie_player| {
            if theme_data.is_null() {
                return DotLottieResult::InvalidParameter;
            }
            let data = CStr::from_ptr(theme_data);
            dotlottie_player.set_theme_data(data)
        })
    }
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
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slots_json.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let json = CStr::from_ptr(slots_json);
        match json.to_str() {
            Ok(json_str) => dotlottie_player.set_slots_str(json_str),
            Err(_) => Err(DotLottiePlayerError::InvalidParameter),
        }
    })
}

/// Clear all slots
#[no_mangle]
pub unsafe extern "C" fn dotlottie_clear_slots(ptr: *mut DotLottiePlayer) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| dotlottie_player.clear_slots())
}

/// Clear a specific slot by ID
#[no_mangle]
pub unsafe extern "C" fn dotlottie_clear_slot(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => dotlottie_player.clear_slot(id_str),
            Err(_) => Err(DotLottiePlayerError::InvalidParameter),
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
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => {
                let slot = ColorSlot::static_value(ColorValue([r, g, b]));
                dotlottie_player.set_color_slot(id_str, slot)
            }
            Err(_) => Err(DotLottiePlayerError::InvalidParameter),
        }
    })
}

/// Set a scalar slot with a single float value
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_scalar_slot(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    value: f32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => {
                let slot = ScalarSlot::static_value(ScalarValue(value));
                dotlottie_player.set_scalar_slot(id_str, slot)
            }
            Err(_) => Err(DotLottiePlayerError::InvalidParameter),
        }
    })
}

/// Set a text slot with a text string
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_text_slot(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    text: *const c_char,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() || text.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let id = CStr::from_ptr(slot_id);
        let text_cstr = CStr::from_ptr(text);
        match (id.to_str(), text_cstr.to_str()) {
            (Ok(id_str), Ok(text_str)) => {
                let slot = TextSlot::with_document(TextDocument::new(text_str.to_string()));
                dotlottie_player.set_text_slot(id_str, slot)
            }
            _ => Err(DotLottiePlayerError::InvalidParameter),
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
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => {
                let slot = VectorSlot::static_value([x, y]);
                dotlottie_player.set_vector_slot(id_str, slot)
            }
            Err(_) => Err(DotLottiePlayerError::InvalidParameter),
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
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => {
                let slot = PositionSlot::static_value([x, y]);
                dotlottie_player.set_position_slot(id_str, slot)
            }
            Err(_) => Err(DotLottiePlayerError::InvalidParameter),
        }
    })
}

/// Set an image slot from a file path
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_image_slot_path(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    path: *const c_char,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() || path.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let id = CStr::from_ptr(slot_id);
        let path_cstr = CStr::from_ptr(path);
        match (id.to_str(), path_cstr.to_str()) {
            (Ok(id_str), Ok(path_str)) => {
                let slot = ImageSlot::from_path(path_str.to_string());
                dotlottie_player.set_image_slot(id_str, slot)
            }
            _ => Err(DotLottiePlayerError::InvalidParameter),
        }
    })
}

/// Set an image slot from a data URL (base64 encoded)
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_image_slot_data_url(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    data_url: *const c_char,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() || data_url.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let id = CStr::from_ptr(slot_id);
        let url = CStr::from_ptr(data_url);
        match (id.to_str(), url.to_str()) {
            (Ok(id_str), Ok(url_str)) => {
                let slot = ImageSlot::from_data_url(url_str.to_string());
                dotlottie_player.set_image_slot(id_str, slot)
            }
            _ => Err(DotLottiePlayerError::InvalidParameter),
        }
    })
}

// ============================================================================
// SLOT GETTERS / RESET C API
// ============================================================================

/// Returns the number of slot IDs in the current animation.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_slot_ids_count(
    ptr: *mut DotLottiePlayer,
    count: *mut u32,
) -> DotLottieResult {
    if ptr.is_null() || count.is_null() {
        return DotLottieResult::InvalidParameter;
    }
    let player = &*ptr;
    *count = player.get_slot_ids().len() as u32;
    DotLottieResult::Success
}

/// Returns a slot ID by index.
///
/// Call `dotlottie_get_slot_ids_count` first to know how many IDs exist.
/// Pass `buffer = NULL` to query the required size via `size_out`.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_slot_id(
    ptr: *mut DotLottiePlayer,
    index: u32,
    buffer: *mut c_char,
    size_out: *mut usize,
) -> DotLottieResult {
    if ptr.is_null() {
        return DotLottieResult::InvalidParameter;
    }
    let player = &*ptr;
    let ids = player.get_slot_ids();
    let idx = index as usize;
    if idx >= ids.len() {
        return DotLottieResult::InvalidParameter;
    }

    let id_str = &ids[idx];
    let bytes_with_nul = id_str.len() + 1;

    if !size_out.is_null() {
        *size_out = bytes_with_nul;
    }

    if !buffer.is_null() {
        std::ptr::copy_nonoverlapping(id_str.as_ptr() as *const c_char, buffer, id_str.len());
        *buffer.add(id_str.len()) = 0; // null terminator
    }

    DotLottieResult::Success
}

/// Returns the type name of a slot (e.g., "color", "scalar", "text").
///
/// Pass `buffer = NULL` to query the required size via `size_out`.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_slot_type(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    buffer: *mut c_char,
    size_out: *mut usize,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => {
                let type_name = dotlottie_player.get_slot_type(id_str);
                let bytes_with_nul = type_name.len() + 1;

                if !size_out.is_null() {
                    *size_out = bytes_with_nul;
                }

                if !buffer.is_null() {
                    std::ptr::copy_nonoverlapping(
                        type_name.as_ptr() as *const c_char,
                        buffer,
                        type_name.len(),
                    );
                    *buffer.add(type_name.len()) = 0;
                }

                DotLottieResult::Success
            }
            Err(_) => DotLottieResult::InvalidParameter,
        }
    })
}

/// Returns a slot value as a JSON string.
///
/// Pass `buffer = NULL` to query the required size via `size_out`.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_slot_str(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    buffer: *mut c_char,
    size_out: *mut usize,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => {
                let json = dotlottie_player.get_slot_str(id_str);
                let bytes_with_nul = json.len() + 1;

                if !size_out.is_null() {
                    *size_out = bytes_with_nul;
                }

                if !buffer.is_null() {
                    std::ptr::copy_nonoverlapping(
                        json.as_ptr() as *const c_char,
                        buffer,
                        json.len(),
                    );
                    *buffer.add(json.len()) = 0;
                }

                DotLottieResult::Success
            }
            Err(_) => DotLottieResult::InvalidParameter,
        }
    })
}

/// Returns all slots as a JSON string.
///
/// Pass `buffer = NULL` to query the required size via `size_out`.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_slots_str(
    ptr: *mut DotLottiePlayer,
    buffer: *mut c_char,
    size_out: *mut usize,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        let json = dotlottie_player.get_slots_str();
        let bytes_with_nul = json.len() + 1;

        if !size_out.is_null() {
            *size_out = bytes_with_nul;
        }

        if !buffer.is_null() {
            std::ptr::copy_nonoverlapping(json.as_ptr() as *const c_char, buffer, json.len());
            *buffer.add(json.len()) = 0;
        }

        DotLottieResult::Success
    })
}

/// Set a slot value from a JSON string.
///
/// The slot must already exist (i.e., its ID must be in the current slot values).
/// The JSON should match the format for the slot's type.
#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_slot_str(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
    json: *const c_char,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() || json.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let id = CStr::from_ptr(slot_id);
        let json_cstr = CStr::from_ptr(json);
        match (id.to_str(), json_cstr.to_str()) {
            (Ok(id_str), Ok(json_str)) => dotlottie_player.set_slot_str(id_str, json_str),
            _ => Err(DotLottiePlayerError::InvalidParameter),
        }
    })
}

/// Reset a single slot to its default value (from the animation).
#[no_mangle]
pub unsafe extern "C" fn dotlottie_reset_slot(
    ptr: *mut DotLottiePlayer,
    slot_id: *const c_char,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if slot_id.is_null() {
            return DotLottieResult::InvalidParameter;
        }
        let id = CStr::from_ptr(slot_id);
        match id.to_str() {
            Ok(id_str) => dotlottie_player.reset_slot(id_str),
            Err(_) => Err(DotLottiePlayerError::InvalidParameter),
        }
    })
}

/// Reset all slots to their default values (from the animation).
#[no_mangle]
pub unsafe extern "C" fn dotlottie_reset_slots(ptr: *mut DotLottiePlayer) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if dotlottie_player.reset_slots() {
            Ok(())
        } else {
            Err(DotLottiePlayerError::Unknown)
        }
    })
}

/// Gets the number of markers in the current animation.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
/// - `count`: Pointer to receive the marker count
///
/// # Returns
/// - `DOTLOTTIE_SUCCESS` on success
/// - `DOTLOTTIE_INVALID_PARAMETER` if pointers are null
#[no_mangle]
pub unsafe extern "C" fn dotlottie_markers_count(
    ptr: *mut DotLottiePlayer,
    count: *mut u32,
) -> DotLottieResult {
    if ptr.is_null() || count.is_null() {
        return DotLottieResult::InvalidParameter;
    }
    let player = &*ptr;
    *count = player.marker_names().len() as u32;
    DotLottieResult::Success
}

/// Gets a marker by index.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
/// - `idx`: Index of the marker (0-based)
/// - `name`: Pointer to receive the marker name (library-owned, do not free)
/// - `time`: Pointer to receive the marker time (start frame), or NULL to skip
/// - `duration`: Pointer to receive the marker duration (in frames), or NULL to skip
///
/// # Returns
/// - `DOTLOTTIE_SUCCESS` on success
/// - `DOTLOTTIE_INVALID_PARAMETER` if ptr/name is null or index is out of bounds
#[no_mangle]
pub unsafe extern "C" fn dotlottie_marker(
    ptr: *mut DotLottiePlayer,
    idx: u32,
    name: *mut *const c_char,
    time: *mut f32,
    duration: *mut f32,
) -> DotLottieResult {
    if ptr.is_null() || name.is_null() {
        return DotLottieResult::InvalidParameter;
    }
    let player = &*ptr;
    let idx = idx as usize;

    let marker_names = player.marker_names();
    let marker_data = player.marker_data();

    if idx >= marker_names.len() {
        return DotLottieResult::InvalidParameter;
    }

    *name = marker_names[idx].as_ptr();
    if !time.is_null() {
        *time = marker_data[idx].0;
    }
    if !duration.is_null() {
        *duration = marker_data[idx].1;
    }

    DotLottieResult::Success
}

/// Returns the active animation ID.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
/// - `buffer`: Buffer to store the ID, or NULL to query required size
/// - `size_out`: Pointer to receive the required buffer size (including null terminator)
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::FeatureNotEnabled` if built without the `dotlottie` feature
/// - `DotLottieResult::InvalidParameter` if no animation is active or player pointer is invalid
#[cfg_attr(not(feature = "dotlottie"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_animation_id(
    ptr: *mut DotLottiePlayer,
    buffer: *mut c_char,
    size_out: *mut usize,
) -> DotLottieResult {
    #[cfg(not(feature = "dotlottie"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "dotlottie")]
    {
        exec_dotlottie_player_op!(ptr, |dotlottie_player| {
            match dotlottie_player.animation_id() {
                Some(id) => {
                    let id_bytes = id.to_bytes_with_nul();

                    if !size_out.is_null() {
                        *size_out = id_bytes.len();
                    }

                    if !buffer.is_null() {
                        std::ptr::copy_nonoverlapping(
                            id_bytes.as_ptr() as *const c_char,
                            buffer,
                            id_bytes.len(),
                        );
                    }

                    DotLottieResult::Success
                }
                None => DotLottieResult::InvalidParameter,
            }
        })
    }
}

/// Returns the active theme ID.
///
/// # Parameters
/// - `ptr`: Pointer to the DotLottiePlayer instance
/// - `buffer`: Buffer to store the ID, or NULL to query required size
/// - `size_out`: Pointer to receive the required buffer size (including null terminator)
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::FeatureNotEnabled` if built without the `theming` feature
/// - `DotLottieResult::InvalidParameter` if no theme is active or player pointer is invalid
#[cfg_attr(not(feature = "theming"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_theme_id(
    ptr: *mut DotLottiePlayer,
    buffer: *mut c_char,
    size_out: *mut usize,
) -> DotLottieResult {
    #[cfg(not(feature = "theming"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "theming")]
    {
        exec_dotlottie_player_op!(ptr, |dotlottie_player| {
            match dotlottie_player.theme_id() {
                Some(id) => {
                    let id_bytes = id.to_bytes_with_nul();

                    if !size_out.is_null() {
                        *size_out = id_bytes.len();
                    }

                    if !buffer.is_null() {
                        std::ptr::copy_nonoverlapping(
                            id_bytes.as_ptr() as *const c_char,
                            buffer,
                            id_bytes.len(),
                        );
                    }

                    DotLottieResult::Success
                }
                None => DotLottieResult::InvalidParameter,
            }
        })
    }
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_set_viewport(
    ptr: *mut DotLottiePlayer,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        dotlottie_player.set_viewport(x, y, w, h)
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_segment_duration(
    ptr: *mut DotLottiePlayer,
    result: *mut f32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if !result.is_null() {
            *result = dotlottie_player.segment_duration();
            DotLottieResult::Success
        } else {
            DotLottieResult::InvalidParameter
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_animation_size(
    ptr: *mut DotLottiePlayer,
    picture_width: *mut f32,
    picture_height: *mut f32,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if picture_width.is_null() || picture_height.is_null() {
            return DotLottieResult::InvalidParameter;
        }

        if let [w, h] = dotlottie_player.animation_size().as_slice() {
            *picture_width = *w;
            *picture_height = *h;
            return DotLottieResult::Success;
        }

        DotLottieResult::Error
    })
}

#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_layer_bounds(
    ptr: *mut DotLottiePlayer,
    layer_name: *const c_char,
    result: *mut LayerBoundingBox,
) -> DotLottieResult {
    exec_dotlottie_player_op!(ptr, |dotlottie_player| {
        if layer_name.is_null() || result.is_null() {
            return DotLottieResult::InvalidParameter;
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
                    DotLottieResult::Success
                }
                _ => DotLottieResult::Error,
            },
            Err(_) => DotLottieResult::InvalidParameter,
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
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_load(
    runtime: *mut DotLottiePlayer,
    state_machine_id: *const c_char,
) -> *mut DotLottieStateMachine {
    #[cfg(not(feature = "state-machines"))]
    {
        return std::ptr::null_mut();
    }
    #[cfg(feature = "state-machines")]
    {
        use crate::StateMachineEngine;

        if runtime.is_null() || state_machine_id.is_null() {
            return std::ptr::null_mut();
        }

        let runtime_ref = &mut *runtime;
        let sm_id = CStr::from_ptr(state_machine_id);

        match runtime_ref.state_machine_load(sm_id) {
            Ok(sm) => {
                // Transmute lifetime to 'static for FFI boundary
                // Safety: The C caller must ensure SM is destroyed before Runtime
                let sm_static: StateMachineEngine<'static> = std::mem::transmute(sm);
                Box::into_raw(Box::new(DotLottieStateMachine { inner: sm_static }))
            }
            Err(_) => std::ptr::null_mut(),
        }
    }
}

/// Load a state machine from a JSON definition string
///
/// Returns a pointer to the StateMachineEngine or NULL on error.
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_load_data(
    runtime: *mut DotLottiePlayer,
    state_machine_definition: *const c_char,
) -> *mut DotLottieStateMachine {
    #[cfg(not(feature = "state-machines"))]
    {
        return std::ptr::null_mut();
    }
    #[cfg(feature = "state-machines")]
    {
        use crate::StateMachineEngine;

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
                    Box::into_raw(Box::new(DotLottieStateMachine { inner: sm_static }))
                }
                Err(_) => std::ptr::null_mut(),
            },
            Err(_) => std::ptr::null_mut(),
        }
    }
}

/// Start the state machine with the specified URL policy.
///
/// # Parameters
/// - `sm`: Pointer to the StateMachineEngine instance
/// - `whitelist`: Comma-separated list of allowed URL patterns (or NULL for empty)
/// - `require_user_interaction`: Whether user interaction is required before opening URLs
///
/// # Returns
/// DotLottieResult::Success if started, error variant if failed
/// - `DotLottieResult::FeatureNotEnabled` if built without the `state-machines` feature
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_start(
    sm: *mut DotLottieStateMachine,
    whitelist: *const c_char,
    require_user_interaction: bool,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        use crate::actions::open_url_policy::OpenUrlPolicy;
        exec_state_machine_op!(sm, |state_machine| {
            let whitelist_vec = if whitelist.is_null() {
                vec![]
            } else {
                let whitelist_cstr = CStr::from_ptr(whitelist);
                match whitelist_cstr.to_str() {
                    Ok("") => vec![],
                    Ok(s) => s.split(',').map(|p| p.trim().to_string()).collect(),
                    Err(_) => return DotLottieResult::InvalidParameter,
                }
            };

            let policy = OpenUrlPolicy::new(whitelist_vec, require_user_interaction);
            state_machine.start(&policy)
        })
    }
}

/// Stop the state machine (does not release the borrow)
///
/// Call dotlottie_state_machine_release() to actually destroy the state machine
/// and release the runtime borrow.
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_stop(
    sm: *mut DotLottieStateMachine,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| {
            state_machine.stop();
            DotLottieResult::Success
        })
    }
}

/// Destroy the state machine and release the runtime borrow
///
/// After calling this, the state machine pointer is invalid and the runtime
/// can be used again.
///
/// # Safety
/// - State machine pointer must be valid
/// - Must not use state machine pointer after this call
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_release(sm: *mut DotLottieStateMachine) {
    #[cfg(not(feature = "state-machines"))]
    {
        return;
    }
    #[cfg(feature = "state-machines")]
    {
        if !sm.is_null() {
            let boxed = Box::from_raw(sm);
            boxed.inner.release(); // Calls consuming release()
        }
    }
}

/// Tick the state machine (advances animation and processes state logic)
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_tick(
    sm: *mut DotLottieStateMachine,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| state_machine.tick())
    }
}

/// Post a pointer/click event to the state machine
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_event(
    sm: *mut DotLottieStateMachine,
    event: *const DotLottieEvent,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| {
            if let Some(event) = event.as_ref() {
                state_machine.post_event(&event.to_event());
                DotLottieResult::Success
            } else {
                DotLottieResult::Error
            }
        })
    }
}

/// Helper functions for posting specific event types
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_click(
    sm: *mut DotLottieStateMachine,
    x: f32,
    y: f32,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        use crate::state_machine_engine::events::Event;
        exec_state_machine_op!(sm, |state_machine| {
            let event = Event::Click { x, y };
            state_machine.post_event(&event);
            DotLottieResult::Success
        })
    }
}

#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_pointer_down(
    sm: *mut DotLottieStateMachine,
    x: f32,
    y: f32,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        use crate::state_machine_engine::events::Event;
        exec_state_machine_op!(sm, |state_machine| {
            let event = Event::PointerDown { x, y };
            state_machine.post_event(&event);
            DotLottieResult::Success
        })
    }
}

#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_pointer_up(
    sm: *mut DotLottieStateMachine,
    x: f32,
    y: f32,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        use crate::state_machine_engine::events::Event;
        exec_state_machine_op!(sm, |state_machine| {
            let event = Event::PointerUp { x, y };
            state_machine.post_event(&event);
            DotLottieResult::Success
        })
    }
}

#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_pointer_move(
    sm: *mut DotLottieStateMachine,
    x: f32,
    y: f32,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        use crate::state_machine_engine::events::Event;
        exec_state_machine_op!(sm, |state_machine| {
            let event = Event::PointerMove { x, y };
            state_machine.post_event(&event);
            DotLottieResult::Success
        })
    }
}

#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_pointer_enter(
    sm: *mut DotLottieStateMachine,
    x: f32,
    y: f32,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        use crate::state_machine_engine::events::Event;
        exec_state_machine_op!(sm, |state_machine| {
            let event = Event::PointerEnter { x, y };
            state_machine.post_event(&event);
            DotLottieResult::Success
        })
    }
}

#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_post_pointer_exit(
    sm: *mut DotLottieStateMachine,
    x: f32,
    y: f32,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        use crate::state_machine_engine::events::Event;
        exec_state_machine_op!(sm, |state_machine| {
            let event = Event::PointerExit { x, y };
            state_machine.post_event(&event);
            DotLottieResult::Success
        })
    }
}

/// Fire a named event input
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_fire_event(
    sm: *mut DotLottieStateMachine,
    event_name: *const c_char,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| {
            if event_name.is_null() {
                return DotLottieResult::InvalidParameter;
            }
            let event = CStr::from_ptr(event_name);
            match event.to_str() {
                Ok(event_str) => match state_machine.fire(event_str, true) {
                    Ok(_) => {
                        let _ = state_machine.run_current_state_pipeline();
                        DotLottieResult::Success
                    }
                    Err(_) => DotLottieResult::Error,
                },
                Err(_) => DotLottieResult::InvalidParameter,
            }
        })
    }
}

/// Set a numeric input
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_set_numeric_input(
    sm: *mut DotLottieStateMachine,
    key: *const c_char,
    value: f32,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| {
            if key.is_null() {
                return DotLottieResult::InvalidParameter;
            }
            let key_cstr = CStr::from_ptr(key);
            match key_cstr.to_str() {
                Ok(key_str) => {
                    let result = state_machine.set_numeric_input(key_str, value, true, false);
                    if result.is_some() {
                        let _ = state_machine.run_current_state_pipeline();
                        DotLottieResult::Success
                    } else {
                        DotLottieResult::Error
                    }
                }
                Err(_) => DotLottieResult::InvalidParameter,
            }
        })
    }
}

/// Set a string input
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_set_string_input(
    sm: *mut DotLottieStateMachine,
    key: *const c_char,
    value: *const c_char,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| {
            if key.is_null() || value.is_null() {
                return DotLottieResult::InvalidParameter;
            }
            let key_cstr = CStr::from_ptr(key);
            let value_cstr = CStr::from_ptr(value);
            match (key_cstr.to_str(), value_cstr.to_str()) {
                (Ok(key_str), Ok(value_str)) => {
                    let result = state_machine.set_string_input(key_str, value_str, true, false);
                    if result.is_some() {
                        let _ = state_machine.run_current_state_pipeline();
                        DotLottieResult::Success
                    } else {
                        DotLottieResult::Error
                    }
                }
                _ => DotLottieResult::InvalidParameter,
            }
        })
    }
}

/// Set a boolean input
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_set_boolean_input(
    sm: *mut DotLottieStateMachine,
    key: *const c_char,
    value: bool,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| {
            if key.is_null() {
                return DotLottieResult::InvalidParameter;
            }
            let key_cstr = CStr::from_ptr(key);
            match key_cstr.to_str() {
                Ok(key_str) => {
                    let result = state_machine.set_boolean_input(key_str, value, true, false);
                    if result.is_some() {
                        let _ = state_machine.run_current_state_pipeline();
                        DotLottieResult::Success
                    } else {
                        DotLottieResult::Error
                    }
                }
                Err(_) => DotLottieResult::InvalidParameter,
            }
        })
    }
}

/// Get a numeric input value
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_get_numeric_input(
    sm: *mut DotLottieStateMachine,
    key: *const c_char,
    result: *mut f32,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| {
            if key.is_null() || result.is_null() {
                return DotLottieResult::InvalidParameter;
            }
            let key_cstr = CStr::from_ptr(key);
            match key_cstr.to_str() {
                Ok(key_str) => {
                    if let Some(value) = state_machine.get_numeric_input(key_str) {
                        *result = value;
                        DotLottieResult::Success
                    } else {
                        DotLottieResult::Error
                    }
                }
                Err(_) => DotLottieResult::InvalidParameter,
            }
        })
    }
}

/// Get a string input value.
///
/// # Parameters
/// - `sm`: Pointer to the StateMachineEngine instance
/// - `key`: Null-terminated C string with the input key
/// - `buffer`: Buffer to store the value, or NULL to query required size
/// - `size_out`: Pointer to receive the required buffer size (including null terminator)
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::FeatureNotEnabled` if built without the `state-machines` feature
/// - `DotLottieResult::InvalidParameter` if the input doesn't exist or pointers are invalid
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_get_string_input(
    sm: *mut DotLottieStateMachine,
    key: *const c_char,
    buffer: *mut c_char,
    size_out: *mut usize,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| {
            if key.is_null() {
                return DotLottieResult::InvalidParameter;
            }
            let key_cstr = CStr::from_ptr(key);
            match key_cstr.to_str() {
                Ok(key_str) => {
                    if let Some(value) = state_machine.get_string_input(key_str) {
                        let value_bytes = value.as_bytes();
                        let size = value_bytes.len() + 1;

                        if !size_out.is_null() {
                            *size_out = size;
                        }

                        if !buffer.is_null() {
                            std::ptr::copy_nonoverlapping(
                                value_bytes.as_ptr() as *const c_char,
                                buffer,
                                value_bytes.len(),
                            );
                            *buffer.add(value_bytes.len()) = 0;
                        }

                        DotLottieResult::Success
                    } else {
                        DotLottieResult::InvalidParameter
                    }
                }
                Err(_) => DotLottieResult::InvalidParameter,
            }
        })
    }
}

/// Get a boolean input value
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_get_boolean_input(
    sm: *mut DotLottieStateMachine,
    key: *const c_char,
    result: *mut bool,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| {
            if key.is_null() || result.is_null() {
                return DotLottieResult::InvalidParameter;
            }
            let key_cstr = CStr::from_ptr(key);
            match key_cstr.to_str() {
                Ok(key_str) => {
                    if let Some(value) = state_machine.get_boolean_input(key_str) {
                        *result = value;
                        DotLottieResult::Success
                    } else {
                        DotLottieResult::Error
                    }
                }
                Err(_) => DotLottieResult::InvalidParameter,
            }
        })
    }
}

/// Get current state name.
///
/// # Parameters
/// - `sm`: Pointer to the StateMachineEngine instance
/// - `buffer`: Buffer to store the state name, or NULL to query required size
/// - `size_out`: Pointer to receive the required buffer size (including null terminator)
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::FeatureNotEnabled` if built without the `state-machines` feature
/// - `DotLottieResult::InvalidParameter` if pointer is invalid
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_current_state(
    sm: *mut DotLottieStateMachine,
    buffer: *mut c_char,
    size_out: *mut usize,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| {
            let current_state = state_machine.get_current_state_name();
            let state_bytes = current_state.as_bytes();
            let size = state_bytes.len() + 1;

            if !size_out.is_null() {
                *size_out = size;
            }

            if !buffer.is_null() {
                std::ptr::copy_nonoverlapping(
                    state_bytes.as_ptr() as *const c_char,
                    buffer,
                    state_bytes.len(),
                );
                *buffer.add(state_bytes.len()) = 0;
            }

            DotLottieResult::Success
        })
    }
}

/// Get state machine status.
///
/// # Parameters
/// - `sm`: Pointer to the StateMachineEngine instance
/// - `buffer`: Buffer to store the status, or NULL to query required size
/// - `size_out`: Pointer to receive the required buffer size (including null terminator)
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::FeatureNotEnabled` if built without the `state-machines` feature
/// - `DotLottieResult::InvalidParameter` if pointer is invalid
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_status(
    sm: *mut DotLottieStateMachine,
    buffer: *mut c_char,
    size_out: *mut usize,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| {
            let status = state_machine.status();
            let status_bytes = status.as_bytes();
            let size = status_bytes.len() + 1;

            if !size_out.is_null() {
                *size_out = size;
            }

            if !buffer.is_null() {
                std::ptr::copy_nonoverlapping(
                    status_bytes.as_ptr() as *const c_char,
                    buffer,
                    status_bytes.len(),
                );
                *buffer.add(status_bytes.len()) = 0;
            }

            DotLottieResult::Success
        })
    }
}

/// Get interaction types for framework setup
///
/// Returns bit flags indicating which interaction types are needed.
/// Frameworks should register listeners for the returned interaction types.
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_framework_setup(
    sm: *mut DotLottieStateMachine,
    result: *mut u16,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_state_machine_op!(sm, |state_machine| {
            if result.is_null() {
                return DotLottieResult::InvalidParameter;
            }

            let interaction_types = state_machine.framework_setup();

            // Convert Vec<String> to bit flags using InteractionType
            if let Ok(interaction_type) = InteractionType::new(&interaction_types) {
                *result = interaction_type.bits();
                DotLottieResult::Success
            } else {
                DotLottieResult::Error
            }
        })
    }
}

/// Poll for the next state machine event
///
/// Returns 1 if an event was retrieved, 0 if no events are available, or -1 on error.
/// String pointers in the event struct are valid until the next poll call.
///
/// # Example
/// ```c
/// StateMachineEvent event;
/// while (dotlottie_state_machine_poll_event(sm, &event) == 1) {
///     switch (event.event_type) {
///         case StateMachineEventType_StateMachineTransition:
///             // Pointers valid until next poll
///             printf("Transition: %s -> %s\n",
///                    event.data.transition.previous_state,
///                    event.data.transition.new_state);
///             break;
///         case StateMachineEventType_StateMachineStateEntered:
///             printf("Entered: %s\n", event.data.state.state);
///             break;
///     }
/// }
/// ```
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_poll_event(
    sm: *mut DotLottieStateMachine,
    event: *mut types::StateMachineEvent,
) -> i32 {
    #[cfg(not(feature = "state-machines"))]
    {
        return -1;
    }
    #[cfg(feature = "state-machines")]
    {
        use crate::StateMachineEvent;

        if sm.is_null() || event.is_null() {
            return -1;
        }

        let state_machine = &mut (*sm).inner;

        // Poll from queue and store in current_event (keeps CStrings alive)
        match state_machine.event_queue.poll() {
            Some(rust_event) => {
                state_machine.current_event = Some(rust_event);
            }
            None => return 0, // No events available
        }

        // Get reference to stored event
        let e = state_machine.current_event.as_ref().unwrap();

        // Build C event struct with pointers into the stored event
        let c_event = match e {
            StateMachineEvent::Start => types::StateMachineEvent {
                event_type: types::StateMachineEventType::StateMachineStart,
                data: types::StateMachineEventData {
                    message: types::StateMachineMessageData {
                        message: std::ptr::null(),
                    },
                },
            },
            StateMachineEvent::Stop => types::StateMachineEvent {
                event_type: types::StateMachineEventType::StateMachineStop,
                data: types::StateMachineEventData {
                    message: types::StateMachineMessageData {
                        message: std::ptr::null(),
                    },
                },
            },
            StateMachineEvent::Transition {
                previous_state,
                new_state,
            } => types::StateMachineEvent {
                event_type: types::StateMachineEventType::StateMachineTransition,
                data: types::StateMachineEventData {
                    transition: types::StateMachineTransitionData {
                        previous_state: previous_state.as_ptr(),
                        new_state: new_state.as_ptr(),
                    },
                },
            },
            StateMachineEvent::StateEntered { state } => types::StateMachineEvent {
                event_type: types::StateMachineEventType::StateMachineStateEntered,
                data: types::StateMachineEventData {
                    state: types::StateMachineStateData {
                        state: state.as_ptr(),
                    },
                },
            },
            StateMachineEvent::StateExit { state } => types::StateMachineEvent {
                event_type: types::StateMachineEventType::StateMachineStateExit,
                data: types::StateMachineEventData {
                    state: types::StateMachineStateData {
                        state: state.as_ptr(),
                    },
                },
            },
            StateMachineEvent::CustomEvent { message } => types::StateMachineEvent {
                event_type: types::StateMachineEventType::StateMachineCustomEvent,
                data: types::StateMachineEventData {
                    message: types::StateMachineMessageData {
                        message: message.as_ptr(),
                    },
                },
            },
            StateMachineEvent::Error { message } => types::StateMachineEvent {
                event_type: types::StateMachineEventType::StateMachineError,
                data: types::StateMachineEventData {
                    message: types::StateMachineMessageData {
                        message: message.as_ptr(),
                    },
                },
            },
            StateMachineEvent::StringInputChange {
                name,
                old_value,
                new_value,
            } => types::StateMachineEvent {
                event_type: types::StateMachineEventType::StateMachineStringInputChange,
                data: types::StateMachineEventData {
                    string_input: types::StateMachineStringInputData {
                        name: name.as_ptr(),
                        old_value: old_value.as_ptr(),
                        new_value: new_value.as_ptr(),
                    },
                },
            },
            StateMachineEvent::NumericInputChange {
                name,
                old_value,
                new_value,
            } => types::StateMachineEvent {
                event_type: types::StateMachineEventType::StateMachineNumericInputChange,
                data: types::StateMachineEventData {
                    numeric_input: types::StateMachineNumericInputData {
                        name: name.as_ptr(),
                        old_value: *old_value,
                        new_value: *new_value,
                    },
                },
            },
            StateMachineEvent::BooleanInputChange {
                name,
                old_value,
                new_value,
            } => types::StateMachineEvent {
                event_type: types::StateMachineEventType::StateMachineBooleanInputChange,
                data: types::StateMachineEventData {
                    boolean_input: types::StateMachineBooleanInputData {
                        name: name.as_ptr(),
                        old_value: *old_value,
                        new_value: *new_value,
                    },
                },
            },
            StateMachineEvent::InputFired { name } => types::StateMachineEvent {
                event_type: types::StateMachineEventType::StateMachineInputFired,
                data: types::StateMachineEventData {
                    input_fired: types::StateMachineInputFiredData {
                        name: name.as_ptr(),
                    },
                },
            },
        };

        std::ptr::write(event, c_event);
        1 // Event retrieved
    } // end #[cfg(feature = "state-machines")]
}

/// Poll for the next internal state machine event
///
/// Returns 1 if an event was retrieved, 0 if no events are available, or -1 on error.
/// The message pointer is valid until the next poll call.
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_state_machine_poll_internal_event(
    sm: *mut DotLottieStateMachine,
    event: *mut types::StateMachineInternalEvent,
) -> i32 {
    #[cfg(not(feature = "state-machines"))]
    {
        return -1;
    }
    #[cfg(feature = "state-machines")]
    {
        use crate::StateMachineInternalEvent;

        if sm.is_null() || event.is_null() {
            return -1;
        }

        let state_machine = &mut (*sm).inner;

        // Poll from queue and store (keeps CStrings alive)
        match state_machine.internal_event_queue.poll() {
            Some(rust_event) => {
                state_machine.current_internal_event = Some(rust_event);
            }
            None => return 0, // No events available
        }

        // Get reference to stored event
        let e = state_machine.current_internal_event.as_ref().unwrap();

        let c_event = match e {
            StateMachineInternalEvent::Message { message } => types::StateMachineInternalEvent {
                message: message.as_ptr(),
            },
        };

        std::ptr::write(event, c_event);
        1 // Event retrieved
    } // end #[cfg(feature = "state-machines")]
}

/// Get the state machine definition as JSON string.
///
/// # Parameters
/// - `runtime`: Pointer to the DotLottiePlayer instance
/// - `state_machine_id`: Null-terminated C string with the state machine ID
/// - `buffer`: Buffer to store the JSON, or NULL to query required size
/// - `size_out`: Pointer to receive the required buffer size (including null terminator)
///
/// # Returns
/// - `DotLottieResult::Success` on success
/// - `DotLottieResult::FeatureNotEnabled` if built without the `state-machines` feature
/// - `DotLottieResult::InvalidParameter` if state machine not found or pointers are invalid
#[cfg_attr(not(feature = "state-machines"), allow(unused_variables))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_get_state_machine(
    runtime: *mut DotLottiePlayer,
    state_machine_id: *const c_char,
    buffer: *mut c_char,
    size_out: *mut usize,
) -> DotLottieResult {
    #[cfg(not(feature = "state-machines"))]
    {
        return DotLottieResult::FeatureNotEnabled;
    }
    #[cfg(feature = "state-machines")]
    {
        exec_dotlottie_player_op!(runtime, |dotlottie_player| {
            if state_machine_id.is_null() {
                return DotLottieResult::InvalidParameter;
            }
            let sm_id = CStr::from_ptr(state_machine_id);

            match dotlottie_player.get_state_machine(sm_id) {
                Some(sm_json) => {
                    let json_bytes = sm_json.as_bytes();
                    let size = json_bytes.len() + 1; // +1 for null terminator

                    if !size_out.is_null() {
                        *size_out = size;
                    }

                    if !buffer.is_null() {
                        std::ptr::copy_nonoverlapping(
                            json_bytes.as_ptr() as *const c_char,
                            buffer,
                            json_bytes.len(),
                        );
                        // Add null terminator
                        *buffer.add(json_bytes.len()) = 0;
                    }

                    DotLottieResult::Success
                }
                None => DotLottieResult::InvalidParameter,
            }
        })
    }
}

// ============================================================================
// Android context initialisation (audio support)
// ============================================================================

/// Initialise the Android JVM context required by cpal/rodio for audio output.
///
/// Must be called once before loading any animation that contains audio.
/// Safe to call multiple times — subsequent calls are ignored by ndk-context.
///
/// # Arguments
/// * `vm`  - pointer to the `JavaVM` struct (cast from `JavaVM*`)
/// * `ctx` - JNI global reference to an `android.content.Context` object
#[cfg(all(feature = "audio", target_os = "android"))]
#[no_mangle]
pub unsafe extern "C" fn dotlottie_init_android(
    vm: *mut std::ffi::c_void,
    ctx: *mut std::ffi::c_void,
) {
    ndk_context::initialize_android_context(vm, ctx);
}
