use super::SkMemset::{
    Memset16Proc, Memset32Proc, Memset64Proc, RectMemset16Proc, RectMemset32Proc, RectMemset64Proc,
};
// Rust-only signature adapters: native signed counts/heights are kept at the
// pointer boundary. Existing actual 16-byte SkVx store kernels take usize.
// Nonpositive counts/heights execute the same zero native loop iterations.
// Writable initialized T storage and byte-pitched rows retain native leases;
// no references to possibly overlapping raw spans are manufactured.
unsafe fn default_memset16(buffer: *mut u16, value: u16, count: i32) {
    unsafe { crate::src::opts::SkMemset_opts::memsetT_raw(buffer, value, count.max(0) as usize) }
}
unsafe fn default_memset32(buffer: *mut u32, value: u32, count: i32) {
    unsafe { crate::src::opts::SkMemset_opts::memsetT_raw(buffer, value, count.max(0) as usize) }
}
unsafe fn default_memset64(buffer: *mut u64, value: u64, count: i32) {
    unsafe { crate::src::opts::SkMemset_opts::memsetT_raw(buffer, value, count.max(0) as usize) }
}
unsafe fn default_rect_memset16(
    buffer: *mut u16,
    value: u16,
    count: i32,
    row_bytes: usize,
    height: i32,
) {
    unsafe {
        crate::src::opts::SkMemset_opts::rect_memsetT_raw(
            buffer,
            value,
            count.max(0) as usize,
            row_bytes,
            height.max(0) as usize,
        )
    }
}
unsafe fn default_rect_memset32(
    buffer: *mut u32,
    value: u32,
    count: i32,
    row_bytes: usize,
    height: i32,
) {
    unsafe {
        crate::src::opts::SkMemset_opts::rect_memsetT_raw(
            buffer,
            value,
            count.max(0) as usize,
            row_bytes,
            height.max(0) as usize,
        )
    }
}
unsafe fn default_rect_memset64(
    buffer: *mut u64,
    value: u64,
    count: i32,
    row_bytes: usize,
    height: i32,
) {
    unsafe {
        crate::src::opts::SkMemset_opts::rect_memsetT_raw(
            buffer,
            value,
            count.max(0) as usize,
            row_bytes,
            height.max(0) as usize,
        )
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Dispatch {
    pub memset16: Memset16Proc,
    pub memset32: Memset32Proc,
    pub memset64: Memset64Proc,
    pub rect_memset16: RectMemset16Proc,
    pub rect_memset32: RectMemset32Proc,
    pub rect_memset64: RectMemset64Proc,
}
// Source: src/core/SkMemset_opts.cpp:20-26
const fn defaults() -> Dispatch {
    Dispatch {
        memset16: default_memset16,
        memset32: default_memset32,
        memset64: default_memset64,
        rect_memset16: default_rect_memset16,
        rect_memset32: default_rect_memset32,
        rect_memset64: default_rect_memset64,
    }
}
// Source: src/core/SkMemset_opts.cpp:31-42
fn selected() -> &'static Dispatch {
    static SELECTED: std::sync::OnceLock<Dispatch> = std::sync::OnceLock::new();
    SELECTED.get_or_init(|| {
        let mut d = defaults();
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if cfg!(target_feature = "avx") {
                d = avx_dispatch();
            }
            #[cfg(not(SK_ENABLE_OPTIMIZE_SIZE))]
            {
                use super::SkCpu::{self, SkX64};
                SkCpu::CacheRuntimeFeatures();
                if !cfg!(target_feature = "avx") && SkCpu::Supports(SkX64::AVX) {
                    unsafe {
                        super::SkMemset_opts_avx::Init_Memset_avx(&mut d);
                    }
                }
                if SkCpu::Supports(SkX64::ERMS) {
                    unsafe {
                        super::SkMemset_opts_erms::Init_Memset_erms(&mut d);
                    }
                }
            }
        }
        d
    })
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) const fn avx_dispatch() -> Dispatch {
    use crate::src::opts::SkMemset_opts::avx;
    Dispatch {
        memset16: avx::memset16,
        memset32: avx::memset32,
        memset64: avx::memset64,
        rect_memset16: avx::rect_memset16,
        rect_memset32: avx::rect_memset32,
        rect_memset64: avx::rect_memset64,
    }
}
pub unsafe fn memset16(p: *mut u16, v: u16, n: i32) {
    (selected().memset16)(p, v, n)
}
pub unsafe fn rect_memset16(p: *mut u16, v: u16, n: i32, rb: usize, h: i32) {
    (selected().rect_memset16)(p, v, n, rb, h)
}
pub unsafe fn memset32(p: *mut u32, v: u32, n: i32) {
    (selected().memset32)(p, v, n)
}
pub unsafe fn rect_memset32(p: *mut u32, v: u32, n: i32, rb: usize, h: i32) {
    (selected().rect_memset32)(p, v, n, rb, h)
}
pub unsafe fn memset64(p: *mut u64, v: u64, n: i32) {
    (selected().memset64)(p, v, n)
}
pub unsafe fn rect_memset64(p: *mut u64, v: u64, n: i32, rb: usize, h: i32) {
    (selected().rect_memset64)(p, v, n, rb, h)
}
// Source: src/core/SkMemset_opts.cpp:44-46
pub fn Init_Memset() {
    let _ = selected();
}
