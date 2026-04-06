//! Development and testing tools.
//!
//! Enable the `tracking_allocator` feature to use [`TrackingAllocator`] as a
//! `#[global_allocator]` in tests or benchmarks.

#[cfg(all(feature = "tracking_allocator", feature = "tvg"))]
mod tvg_alloc;

#[cfg(feature = "tracking_allocator")]
mod tracking_allocator;

#[cfg(feature = "tracking_allocator")]
pub use tracking_allocator::{MemoryStats, TrackingAllocator};
