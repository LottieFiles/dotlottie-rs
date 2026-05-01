//! Measure heap memory usage of loading a `.lottie` file end-to-end.
//!
//! Run with:
//!   cargo run --release --example mem_loader \
//!       --features tracking_allocator -- <path-to-.lottie>

#![allow(clippy::print_stdout)]

use dotlottie_rs::dotlottie::Reader;
use dotlottie_rs::tools::{MemoryStats, TrackingAllocator};

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator::new();

fn fmt_bytes(n: i64) -> String {
    let abs = n.unsigned_abs() as f64;
    let sign = if n < 0 { "-" } else { "" };
    if abs >= 1024.0 * 1024.0 {
        format!("{sign}{:.2} MiB", abs / (1024.0 * 1024.0))
    } else if abs >= 1024.0 {
        format!("{sign}{:.2} KiB", abs / 1024.0)
    } else {
        format!("{n} B")
    }
}

fn print_phase(label: &str, s: MemoryStats) {
    println!(
        "  {label:<28} current={:>12}  peak={:>12}  allocs={:>7}  frees={:>7}",
        fmt_bytes(s.current_bytes),
        fmt_bytes(s.peak_bytes),
        s.total_allocs,
        s.total_frees,
    );
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: mem_loader <path-to-.lottie>");

    let bytes = std::fs::read(&path).expect("failed to read file");
    let file_len = bytes.len();

    let allocator = TrackingAllocator;

    // Reset so phase deltas exclude the cost of std::env::args + fs::read.
    allocator.reset();

    println!(
        "fixture: {path}  ({})  ({} bytes on disk)",
        fmt_bytes(file_len as i64),
        file_len
    );
    print_phase("baseline (input held)", allocator.heap_stats());

    let reader = Reader::new(&bytes).expect("Reader::new failed");
    print_phase("after Reader::new", allocator.heap_stats());

    let json = reader.initial_animation().expect("initial_animation failed");
    let json_len = json.len();
    print_phase("after initial_animation", allocator.heap_stats());

    drop(json);
    print_phase("dropped JSON", allocator.heap_stats());

    drop(reader);
    print_phase("dropped Reader", allocator.heap_stats());

    drop(bytes);
    let final_stats = allocator.heap_stats();
    print_phase("dropped input bytes", final_stats);

    println!(
        "json output: {} bytes ({})",
        json_len,
        fmt_bytes(json_len as i64)
    );
    println!(
        "PEAK heap during load = {}  (input was {})",
        fmt_bytes(final_stats.peak_bytes),
        fmt_bytes(file_len as i64),
    );
}
