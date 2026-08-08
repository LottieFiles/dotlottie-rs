use std::ffi::c_void;

trait MediaPlayer {
    fn seek(&mut self, seconds: f32);
    fn info(&self) -> Option<(u32, u32, f32)>;
    fn frame(&mut self) -> Option<&[u8]>;
    fn set_playing(&mut self, _playing: bool) {}
    fn set_volume(&mut self, _volume: f32) {}
    fn set_muted(&mut self, _muted: bool) {}
    fn time(&self) -> f32;
}

#[cfg(not(target_arch = "wasm32"))]
mod synthetic;
#[cfg(not(target_arch = "wasm32"))]
use synthetic::SyntheticPlayer as PlatformPlayer;

#[cfg(target_arch = "wasm32")]
mod web_video_player;
#[cfg(target_arch = "wasm32")]
use web_video_player::WebVideoPlayer as PlatformPlayer;

/// # Safety
/// `data` must point to `size` readable bytes for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn dlMediaOpen(data: *const u8, size: u32) -> *mut c_void {
    if data.is_null() || size == 0 {
        return std::ptr::null_mut();
    }
    let bytes = unsafe { std::slice::from_raw_parts(data, size as usize) };
    match PlatformPlayer::open(bytes) {
        Some(player) => {
            let boxed: Box<dyn MediaPlayer> = Box::new(player);
            Box::into_raw(Box::new(boxed)) as *mut c_void
        }
        None => std::ptr::null_mut(),
    }
}

/// # Safety
/// `player` must be a pointer previously returned by [`dlMediaOpen`].
#[no_mangle]
pub unsafe extern "C" fn dlMediaClose(player: *mut c_void) {
    if !player.is_null() {
        drop(unsafe { Box::from_raw(player as *mut Box<dyn MediaPlayer>) });
    }
}

/// # Safety
/// `player` must come from [`dlMediaOpen`]; the out-pointers must be writable.
/// `*frame` stays valid until the next call for the same player.
#[no_mangle]
pub unsafe extern "C" fn dlMediaSync(
    player: *mut c_void,
    frame: *mut *const u8,
    width: *mut u32,
    height: *mut u32,
    duration: *mut f32,
    time: *mut f32,
) -> i32 {
    let Some(player) = (unsafe { as_player(player) }) else {
        return 0;
    };

    let Some((w, h, dur)) = player.info() else {
        return 0;
    };
    unsafe {
        *width = w;
        *height = h;
        *duration = dur;
        *time = player.time();
    }

    let expected = (w as usize) * (h as usize) * 4;
    match player.frame() {
        Some(pixels) if pixels.len() >= expected => {
            unsafe { *frame = pixels.as_ptr() };
            1
        }
        _ => 0,
    }
}

/// # Safety
/// `player` must be a pointer previously returned by [`dlMediaOpen`].
#[no_mangle]
pub unsafe extern "C" fn dlMediaSeek(player: *mut c_void, seconds: f32) {
    if let Some(player) = unsafe { as_player(player) } {
        player.seek(seconds);
    }
}

/// # Safety
/// `player` must be a pointer previously returned by [`dlMediaOpen`].
#[no_mangle]
pub unsafe extern "C" fn dlMediaSetPlaying(player: *mut c_void, on: i32) {
    if let Some(player) = unsafe { as_player(player) } {
        player.set_playing(on != 0);
    }
}

/// # Safety
/// `player` must be a pointer previously returned by [`dlMediaOpen`].
#[no_mangle]
pub unsafe extern "C" fn dlMediaSetVolume(player: *mut c_void, volume: f32) {
    if let Some(player) = unsafe { as_player(player) } {
        player.set_volume(volume.clamp(0.0, 1.0));
    }
}

/// # Safety
/// `player` must be a pointer previously returned by [`dlMediaOpen`].
#[no_mangle]
pub unsafe extern "C" fn dlMediaSetMute(player: *mut c_void, on: i32) {
    if let Some(player) = unsafe { as_player(player) } {
        player.set_muted(on != 0);
    }
}

/// # Safety
/// `player` must be null or a pointer previously returned by [`dlMediaOpen`].
unsafe fn as_player(player: *mut c_void) -> Option<&'static mut dyn MediaPlayer> {
    if player.is_null() {
        return None;
    }
    let boxed = unsafe { &mut *(player as *mut Box<dyn MediaPlayer>) };
    Some(boxed.as_mut())
}
