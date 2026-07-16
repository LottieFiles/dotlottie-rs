#![allow(non_snake_case)]

//! libc/C++ runtime stubs for wasm32-unknown-unknown.
//!
//! ThorVG's C++ sources reference a number of libc symbols that are normally
//! provided by the platform's C runtime. For `wasm32-unknown-unknown` there is
//! no such runtime; most libc symbols (`malloc`/`free`/`realloc`/`calloc`,
//! `str*`, `atoi`/`strtol`, `is*`) come from the `tinyrlibc` crate and
//! `snprintf`/`vsnprintf` from `nostd-printf`. This file provides only what no
//! crate covers: C++ ABI/runtime symbols, libc++ internals, math functions,
//! `strdup`/`tolower`/`bsearch`, `__assert_fail`, setjmp/longjmp, and `rand`
//! (kept local so random() sequences stay identical across releases).

use std::{
    ffi::CStr,
    process, ptr, slice,
    sync::atomic::{AtomicU32, Ordering},
};

// Imports snprintf.
use nostd_printf as _;
// Imports malloc/free/realloc/calloc, str*, atoi/strtol, isdigit/isspace.
use tinyrlibc as _;

#[no_mangle]
unsafe extern "C" fn modff(val: f32, i: *mut f32) -> f32 {
    unsafe {
        *i = val.trunc();
    }

    val.fract()
}

#[no_mangle]
unsafe extern "C" fn _ZdlPvm(ptr: *mut u8, _size: usize) {
    // C++ operator delete(void*, size_t). ThorVG's operator new allocates via
    // the C malloc symbol, so this must pair with tinyrlibc's free.
    tinyrlibc::free(ptr)
}

#[no_mangle]
unsafe extern "C" fn _ZdaPvm(ptr: *mut u8, _size: usize) {
    // C++ operator delete[](void*, size_t) - sized array deallocation
    tinyrlibc::free(ptr)
}

#[no_mangle]
unsafe extern "C" fn __cxa_pure_virtual() {}

#[no_mangle]
unsafe extern "C" fn __cxa_atexit(
    _func: extern "C" fn(*const ()),
    _arg: *const (),
    _dso_handle: *const (),
) -> u32 {
    0
}

#[no_mangle]
unsafe extern "C" fn __cxa_thread_atexit(
    _func: extern "C" fn(*const ()),
    _arg: *const (),
    _dso_handle: *const (),
) -> u32 {
    0
}

#[no_mangle]
unsafe extern "C" fn abort() -> ! {
    process::abort()
}

#[no_mangle]
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

#[no_mangle]
unsafe extern "C" fn strdup(s: *const i8) -> *mut i8 {
    if s.is_null() {
        return ptr::null_mut();
    }

    let bytes = CStr::from_ptr(s).to_bytes_with_nul();

    // Callers (e.g. ThorVG's LottieFont) release the result with C free(),
    // so the allocation must come from tinyrlibc's malloc.
    let p = tinyrlibc::malloc(bytes.len());
    if p.is_null() {
        return ptr::null_mut();
    }

    slice::from_raw_parts_mut(p, bytes.len()).copy_from_slice(bytes);

    p as *mut i8
}

#[no_mangle]
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

#[no_mangle]
unsafe extern "C" fn tolower(c: i32) -> i32 {
    if (c as u8).is_ascii_uppercase() {
        c + 32
    } else {
        c
    }
}

static RAND_STATE: AtomicU32 = AtomicU32::new(12345);

#[no_mangle]
unsafe extern "C" fn rand() -> i32 {
    // Xorshift32 — fast, no libc dependency
    let mut x = RAND_STATE.load(Ordering::Relaxed);
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    RAND_STATE.store(x, Ordering::Relaxed);
    (x & 0x7fff_ffff) as i32
}

#[no_mangle]
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

// ── C++ std::mutex stubs ─────────────────────────────────────────────────
// ThorVG's renderer files use a raw `static mutex _rendererMtx` for renderer
// ref-counting.  WASM is single-threaded so these are safe no-ops.

#[no_mangle]
unsafe extern "C" fn _ZNSt3__25mutexD1Ev(this: *mut u8) -> *mut u8 {
    this
}

#[no_mangle]
unsafe extern "C" fn _ZNSt3__25mutex4lockEv(_this: *mut u8) {}

#[no_mangle]
unsafe extern "C" fn _ZNSt3__25mutex6unlockEv(_this: *mut u8) {}

// ── Math stubs ───────────────────────────────────────────────────────────
// Used by JerryScript's ecma-helpers-number.cpp and ecma-builtin-math.cpp.

#[no_mangle]
unsafe extern "C" fn nextafter(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    if x == y {
        return y;
    }
    if x == 0.0 {
        return if y > 0.0 {
            f64::from_bits(1)
        } else {
            f64::from_bits(1 | (1u64 << 63))
        };
    }
    let bits = x.to_bits() as i64;
    let result_bits = if (x < y) == (x > 0.0) {
        bits + 1
    } else {
        bits - 1
    };
    f64::from_bits(result_bits as u64)
}

#[no_mangle]
unsafe extern "C" fn acosh(x: f64) -> f64 {
    x.acosh()
}

#[no_mangle]
unsafe extern "C" fn asinh(x: f64) -> f64 {
    x.asinh()
}

#[no_mangle]
unsafe extern "C" fn atanh(x: f64) -> f64 {
    x.atanh()
}

#[no_mangle]
unsafe extern "C" fn acoshf(x: f32) -> f32 {
    x.acosh()
}

#[no_mangle]
unsafe extern "C" fn asinhf(x: f32) -> f32 {
    x.asinh()
}

#[no_mangle]
unsafe extern "C" fn atanhf(x: f32) -> f32 {
    x.atanh()
}

// ── setjmp / longjmp stubs ───────────────────────────────────────────────
// JerryScript's parser uses setjmp/longjmp for error recovery.  True
// non-local jumps are impossible in WASM's structured control flow, so
// setjmp always returns 0 (normal path) and longjmp aborts.

#[no_mangle]
unsafe extern "C" fn setjmp(_buf: *mut u8) -> i32 {
    0
}

#[no_mangle]
unsafe extern "C" fn longjmp(_buf: *mut u8, _val: i32) -> ! {
    process::abort()
}
