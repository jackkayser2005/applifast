//! Native Rust heap counters for an isolated demo, without account access.
//!
//! cargo run --locked --features demo --example memory_profile -- --demo
//!     --demo-data .cache/memory-demo --demo-page liked --demo-shot .cache/memory.png
//!
//! Counts requested Rust allocation sizes, not allocator overhead, graphics
//! driver allocations, mapped files, GPU memory or the WebView2 process tree.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
struct Counted {
    live: AtomicUsize,
    peak: AtomicUsize,
}

impl Counted {
    fn allocated(&self, size: usize) {
        let live = self.live.fetch_add(size, Ordering::Relaxed) + size;
        self.peak.fetch_max(live, Ordering::Relaxed);
    }
}

// Safety: every pointer and layout is passed unchanged to System. Counter
// updates perform no allocation and never inspect the allocation's contents.
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            self.allocated(layout.size());
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            self.allocated(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
        self.live.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let replacement = unsafe { System.realloc(pointer, layout, size) };
        if !replacement.is_null() {
            if size >= layout.size() {
                self.allocated(size - layout.size());
            } else {
                self.live.fetch_sub(layout.size() - size, Ordering::Relaxed);
            }
        }
        replacement
    }
}

#[global_allocator]
static ALLOCATOR: Counted = Counted {
    live: AtomicUsize::new(0),
    peak: AtomicUsize::new(0),
};

#[cfg(feature = "demo")]
#[path = "../src/entrypoint.rs"]
mod entrypoint;

#[cfg(any(test, feature = "demo"))]
fn isolated_demo(arguments: &[std::ffi::OsString]) -> bool {
    ["--demo", "--demo-data", "--demo-shot"]
        .iter()
        .all(|flag| arguments.iter().any(|argument| argument == flag))
}

fn main() -> eframe::Result<()> {
    #[cfg(feature = "demo")]
    {
        let arguments: Vec<_> = std::env::args_os().collect();
        if !isolated_demo(&arguments) {
            eprintln!(
                "Use --demo, --demo-data <isolated-directory> and --demo-shot <image>; live-account profiling is disabled."
            );
            std::process::exit(2);
        }
        std::thread::spawn(|| {
            let start = std::time::Instant::now();
            loop {
                eprintln!(
                    "MEMORY elapsed_ms={} rust_live_bytes={} rust_peak_bytes={}",
                    start.elapsed().as_millis(),
                    ALLOCATOR.live.load(Ordering::Relaxed),
                    ALLOCATOR.peak.load(Ordering::Relaxed)
                );
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });
        entrypoint::run()
    }
    #[cfg(not(feature = "demo"))]
    {
        eprintln!("Build this diagnostic with --features demo.");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiling_requires_offline_demo_isolation_and_capture_mode() {
        let arguments: Vec<_> = [
            "--demo",
            "--demo-data",
            "profile",
            "--demo-shot",
            "image.png",
        ]
        .map(std::ffi::OsString::from)
        .into();
        assert!(isolated_demo(&arguments));
        for flag in ["--demo", "--demo-data", "--demo-shot"] {
            let missing: Vec<_> = arguments
                .iter()
                .filter(|argument| *argument != flag)
                .cloned()
                .collect();
            assert!(!isolated_demo(&missing));
        }
    }

    #[test]
    fn counted_system_preserves_payload_through_grow_shrink_and_zeroed_allocations() {
        let layout = Layout::from_size_align(64, 8).unwrap();
        let counter = Counted::default();
        // Safety: every operation uses this allocation's actual layout and
        // size, never reads beyond it, and frees each successful pointer once.
        unsafe {
            let pointer = counter.alloc_zeroed(layout);
            assert!(!pointer.is_null());
            assert_eq!(counter.live.load(Ordering::Relaxed), 64);
            assert!((0..64).all(|index| *pointer.add(index) == 0));
            pointer.write_bytes(0x5a, 64);
            let pointer = counter.realloc(pointer, layout, 128);
            assert!(!pointer.is_null());
            assert_eq!(counter.live.load(Ordering::Relaxed), 128);
            assert!((0..64).all(|index| *pointer.add(index) == 0x5a));
            let grown = Layout::from_size_align(128, 8).unwrap();
            let pointer = counter.realloc(pointer, grown, 32);
            assert!(!pointer.is_null());
            assert_eq!(counter.live.load(Ordering::Relaxed), 32);
            assert!((0..32).all(|index| *pointer.add(index) == 0x5a));
            counter.dealloc(pointer, Layout::from_size_align(32, 8).unwrap());
        }
        assert_eq!(counter.live.load(Ordering::Relaxed), 0);
        assert_eq!(counter.peak.load(Ordering::Relaxed), 128);
    }
}
