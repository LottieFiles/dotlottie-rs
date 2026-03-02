#![expect(dead_code)]
#![allow(non_snake_case)]

//! A small part of stdlib/libc that emscripten ends up adding and the linker
//! cannot remove.

use std::{
    alloc::{self, Layout},
    ffi::CStr,
    mem, process, ptr,
    ptr::NonNull,
    slice,
    sync::atomic::{AtomicU32, Ordering},
};

// Imports snprintf.
#[cfg(feature = "wasm")]
use nostd_printf as _;

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn modff(val: f32, i: *mut f32) -> f32 {
    unsafe {
        *i = val.trunc();
    }

    val.fract()
}

#[derive(Clone, Copy, Debug)]
struct Info {
    ptr: NonNull<u8>,
    layout: Layout,
}

fn with_allocator<F: FnOnce(Layout) -> *mut u8>(size: usize, allocator: F) -> Option<NonNull<u8>> {
    Layout::from_size_align(size, 16)
        .and_then(|layout| Layout::new::<Info>().extend(layout))
        .ok()
        .and_then(|(layout, offset)| {
            let ptr = NonNull::new(allocator(layout))?;

            let alloc_ptr = unsafe { ptr.add(offset) };
            let info_ptr = unsafe { alloc_ptr.sub(mem::size_of::<Info>()).cast() };

            unsafe {
                info_ptr.write_unaligned(Info { ptr, layout });
            }

            Some(alloc_ptr)
        })
}

unsafe fn read_info(alloc_ptr: NonNull<u8>) -> Info {
    let info_ptr = unsafe { alloc_ptr.sub(mem::size_of::<Info>()) };
    unsafe { info_ptr.cast().read_unaligned() }
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn malloc(size: usize) -> Option<NonNull<u8>> {
    with_allocator(size, |layout| unsafe { alloc::alloc(layout) })
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn free(ptr: Option<NonNull<u8>>) {
    if let Some(alloc_ptr) = ptr {
        let info: Info = read_info(alloc_ptr);

        unsafe {
            alloc::dealloc(info.ptr.as_ptr(), info.layout);
        }
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn realloc(ptr: Option<NonNull<u8>>, size: usize) -> Option<NonNull<u8>> {
    let Some(alloc_ptr) = ptr else {
        return malloc(size);
    };

    let info: Info = read_info(alloc_ptr);

    let new_layout = Layout::from_size_align(size, 16)
        .and_then(|layout| Layout::new::<Info>().extend(layout))
        .ok()?
        .0;

    with_allocator(size, |_| unsafe {
        alloc::realloc(info.ptr.as_ptr(), info.layout, new_layout.size())
    })
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn calloc(num: usize, size: usize) -> Option<NonNull<u8>> {
    with_allocator(num * size, |layout| unsafe { alloc::alloc_zeroed(layout) })
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn _ZdlPvm(ptr: Option<NonNull<u8>>, _size: usize) {
    free(ptr)
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn _Znam(size: usize) -> *mut u8 {
    // C++ operator new[] - allocate array
    match malloc(size) {
        Some(ptr) => ptr.as_ptr(),
        None => ptr::null_mut(),
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn _ZdaPvm(ptr: *mut u8, _size: usize) {
    // C++ operator delete[] - deallocate array
    if !ptr.is_null() {
        free(Some(NonNull::new_unchecked(ptr)));
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn _ZdaPv(ptr: *mut u8) {
    // C++ operator delete[] - deallocate array
    if !ptr.is_null() {
        free(Some(NonNull::new_unchecked(ptr)));
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn atoi(s: Option<NonNull<i8>>) -> i32 {
    fn conv(s: Option<NonNull<i8>>) -> Option<i32> {
        let s = unsafe { CStr::from_ptr(s?.as_ptr()).to_str().ok()? };

        let trimmed = s.trim_start();
        let trimmed = trimmed
            .get(..11)
            .unwrap_or(trimmed)
            .trim_end_matches(|c: char| !c.is_ascii_digit());

        trimmed.parse().ok()
    }

    conv(s).unwrap_or_default()
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn __cxa_pure_virtual() {}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn __cxa_atexit(
    _func: extern "C" fn(*const ()),
    _arg: *const (),
    _dso_handle: *const (),
) -> u32 {
    0
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn abort() -> ! {
    process::abort()
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn __assert_fail(
    expr: *const i8,
    file: *const i8,
    line: u32,
    func: *const i8,
) -> ! {
    panic!(
        "Assertion failed: {} at {}:{} in {}",
        unsafe { CStr::from_ptr(expr).to_str().unwrap_or("unknown") },
        unsafe { CStr::from_ptr(file).to_str().unwrap_or("unknown") },
        line,
        unsafe { CStr::from_ptr(func).to_str().unwrap_or("unknown") }
    );
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn strchr(s: Option<NonNull<i8>>, c: u32) -> Option<NonNull<i8>> {
    let c = u8::try_from(c).ok()?;
    s.and_then(|p| {
        CStr::from_ptr(p.as_ptr() as *const i8)
            .to_bytes()
            .iter()
            .enumerate()
            .find_map(|(o, &b)| (b == c).then(|| p.add(o)))
    })
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn strdup(s: Option<NonNull<i8>>) -> Option<NonNull<i8>> {
    s.and_then(|p| {
        let bytes = CStr::from_ptr(p.as_ptr() as *const i8).to_bytes_with_nul();

        let p = malloc(bytes.len())?;

        let slice = slice::from_raw_parts_mut(p.as_ptr(), bytes.len());
        slice.copy_from_slice(bytes);

        Some(p.cast())
    })
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn strcmp(s1: *const i8, s2: *const i8) -> i32 {
    CStr::from_ptr(s1).cmp(CStr::from_ptr(s2)) as i32
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8 {
    let src = CStr::from_ptr(src).to_bytes_with_nul();
    let dest_slice = slice::from_raw_parts_mut(dest as *mut u8, src.len());

    dest_slice.copy_from_slice(src);

    dest
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn strcat(dest: *mut i8, src: *const i8) -> *mut i8 {
    let dest_len = CStr::from_ptr(dest).to_bytes().len();
    let src = CStr::from_ptr(src).to_bytes_with_nul();
    let dest_slice = slice::from_raw_parts_mut(dest as *mut u8, dest_len + src.len());

    dest_slice[dest_len..].copy_from_slice(src);

    dest
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn strstr(haystack: *const i8, needle: *const i8) -> *mut i8 {
    if haystack.is_null() || needle.is_null() {
        return ptr::null_mut();
    }

    let haystack_cstr = CStr::from_ptr(haystack);
    let needle_cstr = CStr::from_ptr(needle);

    let haystack_bytes = haystack_cstr.to_bytes();
    let needle_bytes = needle_cstr.to_bytes();

    // Empty needle should return the beginning of haystack (per C standard)
    if needle_bytes.is_empty() {
        return haystack as *mut i8;
    }

    // Search for needle in haystack
    if let Some(pos) = haystack_bytes
        .windows(needle_bytes.len())
        .position(|window| window == needle_bytes)
    {
        haystack.add(pos) as *mut i8
    } else {
        ptr::null_mut()
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn bsearch(
    key: *const (),
    base: *const (),
    nmemb: usize,
    size: usize,
    compar: unsafe extern "C" fn(*const (), *const ()) -> i32,
) -> *mut () {
    if key.is_null() || base.is_null() || size == 0 || nmemb == 0 {
        return ptr::null_mut();
    }

    let mut left = 0;
    let mut right = nmemb;

    while left < right {
        let mid = left + (right - left) / 2;
        let mid_ptr = (base as *const u8).add(mid * size) as *const ();

        let cmp_result = compar(key, mid_ptr);

        if cmp_result == 0 {
            return mid_ptr as *mut ();
        } else if cmp_result < 0 {
            right = mid;
        } else {
            left = mid + 1;
        }
    }

    ptr::null_mut()
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn strncmp(s1: *const i8, s2: *const i8, n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        let c1 = unsafe { *s1.add(i) as u8 };
        let c2 = unsafe { *s2.add(i) as u8 };
        if c1 != c2 {
            return c1 as i32 - c2 as i32;
        }
        if c1 == 0 {
            return 0;
        }
        i += 1;
    }
    0
}

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn isspace(c: i32) -> i32 {
    matches!(c as u8, b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r') as i32
}

static RAND_STATE: AtomicU32 = AtomicU32::new(12345);

#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn rand() -> i32 {
    // Xorshift32 — fast, no libc dependency
    let mut x = RAND_STATE.load(Ordering::Relaxed);
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    RAND_STATE.store(x, Ordering::Relaxed);
    (x & 0x7fff_ffff) as i32
}

/// `strtol(s, endptr, base)` — string to long (wasm32: long == i32).
#[cfg_attr(feature = "wasm", no_mangle)]
unsafe extern "C" fn strtol(s: *const i8, endptr: *mut *mut i8, base: i32) -> i32 {
    let set_end = |p: *const i8| {
        if !endptr.is_null() {
            unsafe { *endptr = p as *mut i8 };
        }
    };

    if s.is_null() {
        set_end(s);
        return 0;
    }

    let bytes = CStr::from_ptr(s).to_bytes();
    let mut idx = 0;

    // Skip leading whitespace
    while idx < bytes.len() && matches!(bytes[idx], b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r') {
        idx += 1;
    }

    // Optional sign
    let negative = if idx < bytes.len() && bytes[idx] == b'-' {
        idx += 1;
        true
    } else {
        if idx < bytes.len() && bytes[idx] == b'+' {
            idx += 1;
        }
        false
    };

    // Determine radix and consume optional prefix
    let radix: u32 = if base == 0 {
        if idx + 1 < bytes.len() && bytes[idx] == b'0' && matches!(bytes[idx + 1], b'x' | b'X') {
            idx += 2;
            16
        } else if idx < bytes.len() && bytes[idx] == b'0' {
            8
        } else {
            10
        }
    } else {
        if base == 16 && idx + 1 < bytes.len() && bytes[idx] == b'0' && matches!(bytes[idx + 1], b'x' | b'X') {
            idx += 2;
        }
        base as u32
    };

    let start = idx;
    let mut result: i64 = 0;
    while idx < bytes.len() {
        let digit = match bytes[idx] {
            b'0'..=b'9' => bytes[idx] - b'0',
            b'a'..=b'z' => bytes[idx] - b'a' + 10,
            b'A'..=b'Z' => bytes[idx] - b'A' + 10,
            _ => break,
        } as u32;
        if digit >= radix {
            break;
        }
        result = result * radix as i64 + digit as i64;
        idx += 1;
    }

    set_end(if idx == start { s } else { unsafe { s.add(idx) } });
    if negative { -(result as i32) } else { result as i32 }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn _ZNSt3__212__next_primeEm(n: usize) -> usize {
    let mut current = n;

    match n {
        0 => return 0,
        ..=2 => return 2,
        n if n % 2 == 0 => current += 1,
        _ => (),
    }

    fn is_prime(num: usize) -> bool {
        let limit = (num as f64).sqrt() as usize;
        let mut i = 3;
        while i <= limit {
            if num % i == 0 {
                return false;
            }
            i += 2;
        }
        true
    }

    loop {
        if is_prime(current) {
            return current;
        }

        if let Some(next_val) = current.checked_add(2) {
            current = next_val;
        } else {
            panic!("__next_prime overflow");
        }
    }
}
