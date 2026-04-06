//! A global allocator wrapper that tracks memory usage across both Rust heap
//! allocations and ThorVG's C-level allocations, providing per-domain and
//! combined statistics for basic budget checks in tests.

use std::alloc::GlobalAlloc;
use std::sync::atomic::{AtomicIsize, AtomicU64, Ordering};

pub(super) struct AllocStats {
    current_bytes: AtomicIsize,
    peak_bytes: AtomicIsize,
    total_allocs: AtomicU64,
    total_frees: AtomicU64,
}

impl AllocStats {
    pub(super) const fn new() -> Self {
        Self {
            current_bytes: AtomicIsize::new(0),
            peak_bytes: AtomicIsize::new(0),
            total_allocs: AtomicU64::new(0),
            total_frees: AtomicU64::new(0),
        }
    }

    pub(super) fn record_alloc(&self, size: usize) {
        let current = self
            .current_bytes
            .fetch_add(size as isize, Ordering::Relaxed)
            + size as isize;
        self.total_allocs.fetch_add(1, Ordering::Relaxed);
        self.peak_bytes.fetch_max(current, Ordering::Relaxed);
    }

    pub(super) fn record_free(&self, size: usize) {
        self.current_bytes
            .fetch_sub(size as isize, Ordering::Relaxed);
        self.total_frees.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_realloc(&self, old_size: usize, new_size: usize) {
        let delta = new_size as isize - old_size as isize;
        let current = self.current_bytes.fetch_add(delta, Ordering::Relaxed) + delta;
        self.total_frees.fetch_add(1, Ordering::Relaxed);
        self.total_allocs.fetch_add(1, Ordering::Relaxed);
        if delta > 0 {
            self.peak_bytes.fetch_max(current, Ordering::Relaxed);
        }
    }

    pub(super) fn snapshot(&self) -> MemoryStats {
        MemoryStats {
            current_bytes: self.current_bytes.load(Ordering::Relaxed) as i64,
            peak_bytes: self.peak_bytes.load(Ordering::Relaxed) as i64,
            total_allocs: self.total_allocs.load(Ordering::Relaxed),
            total_frees: self.total_frees.load(Ordering::Relaxed),
        }
    }

    pub(super) fn reset(&self) {
        self.current_bytes.store(0, Ordering::Relaxed);
        self.peak_bytes.store(0, Ordering::Relaxed);
        self.total_allocs.store(0, Ordering::Relaxed);
        self.total_frees.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryStats {
    pub current_bytes: i64,
    pub peak_bytes: i64,
    pub total_allocs: u64,
    pub total_frees: u64,
}

static HEAP_STATS: AllocStats = AllocStats::new();

#[cfg(feature = "tvg")]
pub(super) static TVG_STATS: AllocStats = AllocStats::new();

/// An allocator that wraps [`std::alloc::System`] and tracks memory usage.
///
/// When the `tvg` feature is active, ThorVG's C-level allocations are
/// intercepted and included in the returned stats.
#[derive(Default)]
pub struct TrackingAllocator;

impl TrackingAllocator {
    pub const fn new() -> Self {
        Self
    }

    /// Combined memory statistics (heap + ThorVG when available).
    pub fn stats(&self) -> MemoryStats {
        let heap = self.heap_stats();
        self.combine_with_tvg(heap)
    }

    #[cfg(feature = "tvg")]
    fn combine_with_tvg(&self, heap: MemoryStats) -> MemoryStats {
        let tvg = self.tvg_stats();
        MemoryStats {
            current_bytes: heap.current_bytes + tvg.current_bytes,
            peak_bytes: heap.peak_bytes + tvg.peak_bytes,
            total_allocs: heap.total_allocs + tvg.total_allocs,
            total_frees: heap.total_frees + tvg.total_frees,
        }
    }

    #[cfg(not(feature = "tvg"))]
    fn combine_with_tvg(&self, heap: MemoryStats) -> MemoryStats {
        heap
    }

    pub fn heap_stats(&self) -> MemoryStats {
        HEAP_STATS.snapshot()
    }

    #[cfg(feature = "tvg")]
    pub fn tvg_stats(&self) -> MemoryStats {
        TVG_STATS.snapshot()
    }

    pub fn reset(&self) {
        HEAP_STATS.reset();
        #[cfg(feature = "tvg")]
        TVG_STATS.reset();
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let ptr = std::alloc::System.alloc(layout);
        if !ptr.is_null() {
            HEAP_STATS.record_alloc(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        HEAP_STATS.record_free(layout.size());
        std::alloc::System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
        let new_ptr = std::alloc::System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            HEAP_STATS.record_realloc(layout.size(), new_size);
        }
        new_ptr
    }

    unsafe fn alloc_zeroed(&self, layout: std::alloc::Layout) -> *mut u8 {
        let ptr = std::alloc::System.alloc_zeroed(layout);
        if !ptr.is_null() {
            HEAP_STATS.record_alloc(layout.size());
        }
        ptr
    }
}
