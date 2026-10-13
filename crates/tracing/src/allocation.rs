//! Optional Rust-only live requested-byte census by allocation-birth phase.
//! Direct C malloc, allocator rounding and semantic retention are not inferred.
//! Realloc retains the original birth phase; cross-thread frees use the header.
use std::{io::Write, alloc::{GlobalAlloc, Layout, System}, sync::atomic::{AtomicUsize, Ordering}};
use crate::diagnostics::{allocation_phase, PHASE_NAMES};
const N: usize = PHASE_NAMES.len();
struct Counters { live: AtomicUsize, count: AtomicUsize, allocated: AtomicUsize }
impl Counters { const fn new() -> Self { Self { live: AtomicUsize::new(0), count: AtomicUsize::new(0), allocated: AtomicUsize::new(0) } } }
static COUNTERS: [Counters; N] = [const { Counters::new() }; N];
static OVERHEAD: AtomicUsize = AtomicUsize::new(0);
#[derive(Clone, Copy)]
#[repr(C)]
struct Header { phase: usize }
fn extended(layout: Layout) -> Option<(Layout, usize)> {
    Layout::new::<Header>().extend(layout).ok()
}
/// This pointer format is fixed for the entire process, including startup and
/// TLS teardown. Never gate allocation layout on a runtime switch.
pub struct PhaseAllocator;
unsafe impl GlobalAlloc for PhaseAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { self.allocate(layout, false) }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 { self.allocate(layout, true) }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let Some((full, offset)) = extended(layout) else { return; };
        let base = ptr.sub(offset);
        let phase = (*base.cast::<Header>()).phase;
        COUNTERS[phase].live.fetch_sub(layout.size(), Ordering::Relaxed);
        COUNTERS[phase].count.fetch_sub(1, Ordering::Relaxed);
        OVERHEAD.fetch_sub(offset, Ordering::Relaxed);
        System.dealloc(base, full);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let Some((full, offset)) = extended(layout) else { return std::ptr::null_mut(); };
        let Ok(next) = Layout::from_size_align(new_size, layout.align()) else { return std::ptr::null_mut(); };
        let Some((next_full, next_offset)) = extended(next) else { return std::ptr::null_mut(); };
        // Equal alignment guarantees the same header offset on realloc.
        if next_offset != offset { return std::ptr::null_mut(); }
        let base = ptr.sub(offset);
        let phase = (*base.cast::<Header>()).phase;
        let result = System.realloc(base, full, next_full.size());
        if !result.is_null() {
            if new_size >= layout.size() {
                let delta = new_size - layout.size();
                COUNTERS[phase].live.fetch_add(delta, Ordering::Relaxed);
                COUNTERS[phase].allocated.fetch_add(delta, Ordering::Relaxed);
            } else { COUNTERS[phase].live.fetch_sub(layout.size() - new_size, Ordering::Relaxed); }
            result.add(offset)
        } else { result }
    }
}
impl PhaseAllocator {
    unsafe fn allocate(&self, layout: Layout, zeroed: bool) -> *mut u8 {
        let Some((full, offset)) = extended(layout) else { return std::ptr::null_mut(); };
        let base = if zeroed { System.alloc_zeroed(full) } else { System.alloc(full) };
        if base.is_null() { return base; }
        let phase = allocation_phase();
        base.cast::<Header>().write(Header { phase });
        COUNTERS[phase].live.fetch_add(layout.size(), Ordering::Relaxed);
        COUNTERS[phase].count.fetch_add(1, Ordering::Relaxed);
        COUNTERS[phase].allocated.fetch_add(layout.size(), Ordering::Relaxed);
        OVERHEAD.fetch_add(offset, Ordering::Relaxed);
        base.add(offset)
    }
}
/// Requested-byte total sampled without stopping producers. This is not an
/// atomic heap snapshot and is intended only for bounded diagnostic sampling.
pub fn requested_bytes() -> usize {
    COUNTERS.iter().map(|c| c.live.load(Ordering::Relaxed)).sum()
}
/// Atomics are sampled independently while producers run, not a heap snapshot.
pub fn report(time_ns: u128) {
    for (phase, counters) in COUNTERS.iter().enumerate() {
        let _ = writeln!(std::io::stderr().lock(), "rust-allocation time_ns={} phase={} live_requested={} live_allocations={} allocated_requested={}", time_ns,
            PHASE_NAMES[phase], counters.live.load(Ordering::Relaxed), counters.count.load(Ordering::Relaxed), counters.allocated.load(Ordering::Relaxed));
    }
    let _ = writeln!(std::io::stderr().lock(), "rust-allocation-overhead time_ns={} header_padding_live={}", time_ns, OVERHEAD.load(Ordering::Relaxed));
}
#[cfg(test)]
mod tests {
    use super::*;
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
    fn counters(phase: usize) -> (usize, usize, usize) {
        (COUNTERS[phase].live.load(Ordering::Relaxed), COUNTERS[phase].count.load(Ordering::Relaxed), OVERHEAD.load(Ordering::Relaxed))
    }
    #[test]
    fn header_alignment_zeroing_realloc_and_cross_thread_release() {
        let _serial = SERIAL.lock().unwrap();
        for align in [1, 2, 8, 16, 64, 4096] {
            let layout = Layout::from_size_align(57, align).unwrap();
            let _phase = crate::diagnostics::AllocationScope::new(14);
            let before = counters(14);
            let offset = extended(layout).unwrap().1;
            unsafe {
                let p = PhaseAllocator.alloc_zeroed(layout);
                assert!(!p.is_null()); assert_eq!(p as usize % align, 0);
                assert_eq!(counters(14), (before.0 + 57, before.1 + 1, before.2 + offset));
                assert!(std::slice::from_raw_parts(p, 57).iter().all(|b| *b == 0));
                p.write_bytes(0x73, 57);
                let _other = crate::diagnostics::AllocationScope::new(9);
                let p = PhaseAllocator.realloc(p, layout, 113);
                assert!(!p.is_null()); assert_eq!(p as usize % align, 0);
                assert!(std::slice::from_raw_parts(p, 57).iter().all(|b| *b == 0x73));
                assert_eq!(counters(14), (before.0 + 113, before.1 + 1, before.2 + offset));
                let grown = Layout::from_size_align(113, align).unwrap();
                let (full, offset) = extended(grown).unwrap(); let _ = full;
                assert_eq!((*p.sub(offset).cast::<Header>()).phase, 14);
                let p = PhaseAllocator.realloc(p, grown, 19);
                assert!(!p.is_null()); assert_eq!(p as usize % align, 0);
                assert!(std::slice::from_raw_parts(p, 19).iter().all(|b| *b == 0x73));
                assert_eq!(counters(14), (before.0 + 19, before.1 + 1, before.2 + offset));
                let address = p as usize;
                std::thread::spawn(move || PhaseAllocator.dealloc(address as *mut u8, Layout::from_size_align(19, align).unwrap())).join().unwrap();
                assert_eq!(counters(14), before);
            }
        }
    }
    #[test]
    fn overflow_leaves_original_allocation_intact() {
        let _serial = SERIAL.lock().unwrap();
        unsafe {
            let l = Layout::from_size_align(16, 8).unwrap();
            let p = PhaseAllocator.alloc(l); assert!(!p.is_null()); p.write(19);
            let before = counters(allocation_phase());
            assert!(PhaseAllocator.realloc(p, l, usize::MAX).is_null());
            assert_eq!(counters(allocation_phase()), before);
            assert_eq!(p.read(), 19); PhaseAllocator.dealloc(p, l);
        }
        assert!(extended(Layout::from_size_align(isize::MAX as usize, 1).unwrap()).is_none());
    }
}
