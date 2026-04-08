use super::tracking_allocator::TVG_STATS;
use std::alloc::{GlobalAlloc, Layout, System};

/// Minimum alignment guaranteed by the system allocator (16 on 64-bit, 8 on 32-bit).
const MALLOC_ALIGN: usize = std::mem::size_of::<usize>() * 2;

/// Each allocation prepends a header storing the payload size, padded to MALLOC_ALIGN.
const HEADER_SIZE: usize = MALLOC_ALIGN;

#[no_mangle]
pub unsafe extern "C" fn tvg_malloc(size: usize) -> *mut u8 {
    let total = HEADER_SIZE + size;
    let layout = Layout::from_size_align_unchecked(total, MALLOC_ALIGN);
    let ptr = System.alloc(layout);
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    *(ptr as *mut usize) = size;
    TVG_STATS.record_alloc(size);
    ptr.add(HEADER_SIZE)
}

#[no_mangle]
pub unsafe extern "C" fn tvg_calloc(nmemb: usize, size: usize) -> *mut u8 {
    let payload = match nmemb.checked_mul(size) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    let total = HEADER_SIZE + payload;
    let layout = Layout::from_size_align_unchecked(total, MALLOC_ALIGN);
    let ptr = System.alloc_zeroed(layout);
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    *(ptr as *mut usize) = payload;
    TVG_STATS.record_alloc(payload);
    ptr.add(HEADER_SIZE)
}

#[no_mangle]
pub unsafe extern "C" fn tvg_realloc(user_ptr: *mut u8, new_size: usize) -> *mut u8 {
    if user_ptr.is_null() {
        return tvg_malloc(new_size);
    }
    if new_size == 0 {
        tvg_free(user_ptr);
        return std::ptr::null_mut();
    }

    let real_ptr = user_ptr.sub(HEADER_SIZE);
    let old_size = *(real_ptr as *const usize);
    let old_total = HEADER_SIZE + old_size;
    let new_total = HEADER_SIZE + new_size;
    let old_layout = Layout::from_size_align_unchecked(old_total, MALLOC_ALIGN);
    let new_ptr = System.realloc(real_ptr, old_layout, new_total);
    if new_ptr.is_null() {
        return std::ptr::null_mut();
    }
    *(new_ptr as *mut usize) = new_size;
    TVG_STATS.record_realloc(old_size, new_size);
    new_ptr.add(HEADER_SIZE)
}

#[no_mangle]
pub unsafe extern "C" fn tvg_free(user_ptr: *mut u8) {
    if user_ptr.is_null() {
        return;
    }
    let real_ptr = user_ptr.sub(HEADER_SIZE);
    let size = *(real_ptr as *const usize);
    let total = HEADER_SIZE + size;
    let layout = Layout::from_size_align_unchecked(total, MALLOC_ALIGN);
    TVG_STATS.record_free(size);
    System.dealloc(real_ptr, layout);
}
