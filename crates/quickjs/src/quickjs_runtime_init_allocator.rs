// quickjs.c:2137..2214. Default host allocation policy, MIT.
// The platform-specific usable-size call is the allocator boundary in C.
// Rust GlobalAlloc instead requires its original Layout to release a block:
// a private 16-byte prefix stores that size. Custom JSMallocFunctions keep
// the original C callback contract and bypass this prefix entirely.
const DEFAULT_ALLOC_HEADER: usize = 16;
#[cfg(target_vendor = "apple")]
const MALLOC_OVERHEAD: usize = 0;
#[cfg(not(target_vendor = "apple"))]
const MALLOC_OVERHEAD: usize = 8;
unsafe fn js_def_malloc_usable_size(p: *const c_void) -> usize {
    if p.is_null() {
        return 0;
    }
    ptr::read(p.cast::<u8>().sub(DEFAULT_ALLOC_HEADER).cast::<usize>())
        .wrapping_sub(DEFAULT_ALLOC_HEADER)
}
unsafe fn js_def_malloc(s: *mut JSMallocState, size: usize) -> *mut c_void {
    assert_ne!(size, 0);
    if (*s).malloc_size.wrapping_add(size) > (*s).malloc_limit {
        return ptr::null_mut();
    }
    let Some(total) = size.checked_add(DEFAULT_ALLOC_HEADER) else {
        return ptr::null_mut();
    };
    let Ok(layout) = std::alloc::Layout::from_size_align(total, 16) else {
        return ptr::null_mut();
    };
    let base = std::alloc::alloc(layout);
    if base.is_null() {
        return ptr::null_mut();
    }
    ptr::write(base.cast::<usize>(), total);
    let p = base.add(DEFAULT_ALLOC_HEADER).cast::<c_void>();
    (*s).malloc_count = (*s).malloc_count.wrapping_add(1);
    (*s).malloc_size = (*s)
        .malloc_size
        .wrapping_add(js_def_malloc_usable_size(p))
        .wrapping_add(MALLOC_OVERHEAD);
    p
}
unsafe fn js_def_free(s: *mut JSMallocState, p: *mut c_void) {
    if p.is_null() {
        return;
    }
    (*s).malloc_count = (*s).malloc_count.wrapping_sub(1);
    (*s).malloc_size = (*s)
        .malloc_size
        .wrapping_sub(js_def_malloc_usable_size(p))
        .wrapping_sub(MALLOC_OVERHEAD);
    let base = p.cast::<u8>().sub(DEFAULT_ALLOC_HEADER);
    let total = ptr::read(base.cast::<usize>());
    std::alloc::dealloc(
        base,
        std::alloc::Layout::from_size_align_unchecked(total, 16),
    );
}
unsafe fn js_def_realloc(s: *mut JSMallocState, p: *mut c_void, size: usize) -> *mut c_void {
    if p.is_null() {
        if size == 0 {
            return ptr::null_mut();
        }
        return js_def_malloc(s, size);
    }
    let old_size = js_def_malloc_usable_size(p);
    if size == 0 {
        js_def_free(s, p);
        return ptr::null_mut();
    }
    if (*s).malloc_size.wrapping_add(size).wrapping_sub(old_size) > (*s).malloc_limit {
        return ptr::null_mut();
    }
    let Some(total) = size.checked_add(DEFAULT_ALLOC_HEADER) else {
        return ptr::null_mut();
    };
    if std::alloc::Layout::from_size_align(total, 16).is_err() {
        return ptr::null_mut();
    }
    let base = p.cast::<u8>().sub(DEFAULT_ALLOC_HEADER);
    let old_total = ptr::read(base.cast::<usize>());
    let q = std::alloc::realloc(
        base,
        std::alloc::Layout::from_size_align_unchecked(old_total, 16),
        total,
    );
    if q.is_null() {
        return ptr::null_mut();
    }
    ptr::write(q.cast::<usize>(), total);
    let p = q.add(DEFAULT_ALLOC_HEADER).cast::<c_void>();
    (*s).malloc_size = (*s)
        .malloc_size
        .wrapping_add(js_def_malloc_usable_size(p))
        .wrapping_sub(old_size);
    p
}
static def_malloc_funcs: JSMallocFunctions = JSMallocFunctions {
    js_malloc: Some(js_def_malloc),
    js_free: Some(js_def_free),
    js_realloc: Some(js_def_realloc),
    js_malloc_usable_size: Some(js_def_malloc_usable_size),
};
