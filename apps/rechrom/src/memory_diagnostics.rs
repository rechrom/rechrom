//! Optional host allocator accounting. No allocator policy changes or purge.
//! Kept in the embedding: engines expose owner bytes, never libc internals.

#[cfg(all(target_os = "linux", target_env = "gnu"))]
mod glibc {
    use std::ffi::{c_char, c_void};
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct Mallinfo2 {
        pub arena: usize,
        pub ordblks: usize,
        pub smblks: usize,
        pub hblks: usize,
        pub hblkhd: usize,
        pub usmblks: usize,
        pub fsmblks: usize,
        pub uordblks: usize,
        pub fordblks: usize,
        pub keepcost: usize,
    }
    type Query = unsafe extern "C" fn() -> Mallinfo2;
    #[repr(C)]
    struct DlInfo {
        filename: *const c_char,
        base: *mut c_void,
        symbol_name: *const c_char,
        symbol_address: *mut c_void,
    }
    #[link(name = "dl")]
    unsafe extern "C" {
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        fn dladdr(address: *const c_void, info: *mut DlInfo) -> i32;
    }
    pub fn malloc_library() -> Option<&'static str> {
        static LIBRARY: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
        LIBRARY
            .get_or_init(|| {
                // SAFETY: fixed public symbol and correctly sized C output. No
                // function is called through this address; dladdr only identifies it.
                unsafe {
                    let address = dlsym(std::ptr::null_mut(), c"malloc".as_ptr());
                    if address.is_null() {
                        return None;
                    }
                    let mut info = std::mem::MaybeUninit::<DlInfo>::uninit();
                    if dladdr(address, info.as_mut_ptr()) == 0 {
                        return None;
                    }
                    let info = info.assume_init();
                    if info.filename.is_null() {
                        return None;
                    }
                    Some(
                        std::ffi::CStr::from_ptr(info.filename)
                            .to_string_lossy()
                            .into_owned(),
                    )
                }
            })
            .as_deref()
    }
    pub fn snapshot() -> Option<Mallinfo2> {
        // Resolve optionally to avoid raising the supported glibc minimum to
        // 2.33 for a default-off diagnostic. GNU RTLD_DEFAULT is the null handle.
        static QUERY: std::sync::OnceLock<Option<Query>> = std::sync::OnceLock::new();
        let query = QUERY.get_or_init(|| {
            // SAFETY: fixed NUL-terminated public libc symbol, current process.
            let symbol = unsafe { dlsym(std::ptr::null_mut(), c"mallinfo2".as_ptr()) };
            if symbol.is_null() {
                None
            } else {
                // SAFETY: glibc malloc.h defines this exact C ABI and ten
                // size_t fields. The process-loaded libc remains resident.
                Some(unsafe { std::mem::transmute::<*mut c_void, Query>(symbol) })
            }
        });
        // SAFETY: allocator observation is thread-safe and takes no arguments.
        query.map(|query| unsafe { query() })
    }
}

/// Coarse glibc arena in-use/free and mmap allocations. Reserved bytes are not
/// resident bytes; tcache/metadata and non-malloc mappings limit comparison to RSS.
/// Interposed allocators can make this incomplete. Call only inside the existing
/// opt-in observation gate; no additional wake, collection or trim is requested.
pub(crate) fn report_allocator(time_ns: u128, phase: &str) {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    eprintln!(
        "allocator-memory time_ns={} phase={} malloc_library={:?} glibc={:?}",
        time_ns,
        phase,
        glibc::malloc_library(),
        glibc::snapshot()
    );
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    let _ = (time_ns, phase);
}

/// Observe only process allocator counters from this helper thread. Page, JS
/// and native heap owners remain confined to their existing owner-thread census.
pub(crate) fn start_sampler() {
    #[cfg(feature = "allocation_diagnostics")]
    if crate::engine::memory_diagnostics_enabled() {
        let _ = std::thread::Builder::new()
            .name("allocation-census".into())
            .spawn(|| {
                let _phase = browser_tracing::diagnostics::AllocationScope::new(17);
                loop {
                    let time_ns = crate::engine::diagnostic_time_ns();
                    browser_tracing::allocation::report(time_ns);
                    report_allocator(time_ns, "sampler");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            });
    }
}

#[cfg(all(test, feature = "allocation_diagnostics"))]
mod tests {
    #[test]
    #[ignore = "bounded explicit diagnostic memory experiment"]
    fn bounded_trace_snapshot_memory() {
        let _scope = browser_tracing::diagnostics::AllocationScope::new(16);
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop_worker = stop.clone();
        let sampler = std::thread::spawn(move || {
            let mut peak = 0;
            while !stop_worker.load(std::sync::atomic::Ordering::Relaxed) {
                peak = peak.max(browser_tracing::allocation::requested_bytes());
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            peak
        });
        let id = browser_tracing::start(browser_tracing::StartOptions {
            max_events: 200_000,
            target_id: 1,
            ..Default::default()
        })
        .unwrap();
        let _context = browser_tracing::scope(browser_tracing::Context {
            target_id: 1,
            ..Default::default()
        });
        for index in 0..100_000 {
            let mut span = browser_tracing::span("fixture", "BoundedSnapshot");
            span.set("index", index as f64);
            span.set("requested", 512.0);
        }
        browser_tracing::stop().unwrap();
        eprintln!(
            "snapshot-stage buffered status={}",
            browser_tracing::status()
        );
        browser_tracing::allocation::report(1);
        super::report_allocator(1, "buffered");
        let snapshot = browser_tracing::snapshot(Some(id)).unwrap();
        assert_eq!(snapshot.spans.len(), 100_000);
        eprintln!("snapshot-stage collected");
        browser_tracing::allocation::report(2);
        super::report_allocator(2, "collected");
        let summary = devtools::analyze_snapshot(&snapshot);
        let mut value = serde_json::to_value(snapshot).unwrap();
        value["summary"] = summary;
        eprintln!("snapshot-stage json-value");
        browser_tracing::allocation::report(3);
        super::report_allocator(3, "json-value");
        let encoded = serde_json::to_vec(&value).unwrap();
        eprintln!("snapshot-stage encoded bytes={}", encoded.len());
        browser_tracing::allocation::report(4);
        super::report_allocator(4, "encoded");
        drop(encoded);
        drop(value);
        eprintln!("snapshot-stage released");
        browser_tracing::allocation::report(5);
        super::report_allocator(5, "released");
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        eprintln!(
            "snapshot-sampled-peak requested={}",
            sampler.join().unwrap()
        );
        #[cfg(target_os = "linux")]
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status
                .lines()
                .filter(|line| line.starts_with("VmHWM:") || line.starts_with("VmRSS:"))
            {
                eprintln!("snapshot-process {line}");
            }
        }
    }
}
