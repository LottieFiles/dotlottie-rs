use std::ffi::c_void;
use std::sync::Mutex;

mod web_video_player;
use web_video_player::WebVideoPlayer;

static OPEN_PLAYERS: Mutex<Vec<usize>> = Mutex::new(Vec::new());

pub(crate) fn set_all_playing(playing: bool) -> usize {
    let Ok(players) = OPEN_PLAYERS.lock() else {
        return 0;
    };
    let mut reached = 0;
    for handle in players.iter() {
        if let Some(player) = unsafe { as_player(*handle as *mut c_void) } {
            player.set_playing(playing);
            reached += 1;
        }
    }
    reached
}

/// # Safety
/// `data` must point to `size` readable bytes for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn dlMediaOpen(data: *const u8, size: u32) -> *mut c_void {
    if data.is_null() || size == 0 {
        return std::ptr::null_mut();
    }
    let bytes = unsafe { std::slice::from_raw_parts(data, size as usize) };
    match WebVideoPlayer::open(bytes) {
        Some(player) => {
            let handle = Box::into_raw(Box::new(player)) as *mut c_void;
            if let Ok(mut players) = OPEN_PLAYERS.lock() {
                players.push(handle as usize);
            }
            handle
        }
        None => std::ptr::null_mut(),
    }
}

/// # Safety
/// `player` must be a pointer previously returned by [`dlMediaOpen`].
#[no_mangle]
pub unsafe extern "C" fn dlMediaClose(player: *mut c_void) {
    if !player.is_null() {
        if let Ok(mut players) = OPEN_PLAYERS.lock() {
            players.retain(|handle| *handle != player as usize);
        }
        drop(unsafe { Box::from_raw(player as *mut WebVideoPlayer) });
    }
}

/// # Safety
/// `player` must come from [`dlMediaOpen`]; the out-pointers must be writable.
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
unsafe fn as_player(player: *mut c_void) -> Option<&'static mut WebVideoPlayer> {
    if player.is_null() {
        return None;
    }
    Some(unsafe { &mut *(player as *mut WebVideoPlayer) })
}
