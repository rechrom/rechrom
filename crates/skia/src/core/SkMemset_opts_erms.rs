use super::SkMemset_opts::Dispatch;
#[cfg(all(target_arch = "x86_64", not(SK_ENABLE_OPTIMIZE_SIZE)))]
mod erms {
    use super::Dispatch;
    // Source: src/core/SkMemset_opts_erms.cpp:16-16
    const NOTE: &str = "MSAN can't see that repsto initializes memory.";
    // Compiler instrumentation adapter. Root must forward the native compiler's
    // memory-sanitizer detection to its native SK_MSAN build configuration.
    unsafe fn mark_initialized<T>(p: *mut T, n: usize) {
        debug_assert!(!NOTE.is_empty());
        #[cfg(SK_MSAN)]
        {
            unsafe extern "C" {
                fn __msan_unpoison(p: *const core::ffi::c_void, n: usize);
            }
            __msan_unpoison(p.cast(), n.wrapping_mul(core::mem::size_of::<T>()));
        }
    }

    // Source: src/core/SkMemset_opts_erms.cpp:20-23
    #[cfg(target_env = "msvc")]
    unsafe fn repsto16(p: *mut u16, v: u16, n: usize) {
        mark_initialized(p, n);
        core::arch::asm!("rep stosw",inout("rdi")p=>_,inout("rcx")n=>_,in("ax")v,options(nostack,preserves_flags));
    }
    // Source: src/core/SkMemset_opts_erms.cpp:24-28
    #[cfg(target_env = "msvc")]
    unsafe fn repsto32(p: *mut u32, v: u32, n: usize) {
        const {
            assert!(core::mem::size_of::<u32>() == core::mem::size_of::<core::ffi::c_ulong>());
        }
        mark_initialized(p, n);
        core::arch::asm!("rep stosd",inout("rdi")p=>_,inout("rcx")n=>_,in("eax")v,options(nostack,preserves_flags));
    }
    // Source: src/core/SkMemset_opts_erms.cpp:29-32
    #[cfg(target_env = "msvc")]
    unsafe fn repsto64(p: *mut u64, v: u64, n: usize) {
        mark_initialized(p, n);
        core::arch::asm!("rep stosq",inout("rdi")p=>_,inout("rcx")n=>_,in("rax")v,options(nostack,preserves_flags));
    }
    // Source: src/core/SkMemset_opts_erms.cpp:34-37
    #[cfg(not(target_env = "msvc"))]
    unsafe fn repsto16(p: *mut u16, v: u16, n: usize) {
        mark_initialized(p, n);
        core::arch::asm!("rep stosw",inout("rdi")p=>_,inout("rcx")n=>_,in("ax")v,options(nostack,preserves_flags));
    }
    // Source: src/core/SkMemset_opts_erms.cpp:38-41
    #[cfg(not(target_env = "msvc"))]
    unsafe fn repsto32(p: *mut u32, v: u32, n: usize) {
        mark_initialized(p, n);
        core::arch::asm!("rep stosd",inout("rdi")p=>_,inout("rcx")n=>_,in("eax")v,options(nostack,preserves_flags));
    }
    // Source: src/core/SkMemset_opts_erms.cpp:42-45
    #[cfg(not(target_env = "msvc"))]
    unsafe fn repsto64(p: *mut u64, v: u64, n: usize) {
        mark_initialized(p, n);
        core::arch::asm!("rep stosq",inout("rdi")p=>_,inout("rcx")n=>_,in("rax")v,options(nostack,preserves_flags));
    }
    // Source: src/core/SkMemset_opts_erms.cpp:50-55
    pub(super) static PREVIOUS: std::sync::OnceLock<Dispatch> = std::sync::OnceLock::new();
    // Source: src/core/SkMemset_opts_erms.cpp:58-58
    fn small(bytes: usize) -> bool {
        bytes < 1024
    }
    // Source: src/core/SkMemset_opts_erms.cpp:62-64
    pub(super) unsafe fn memset16(p: *mut u16, v: u16, n: i32) {
        if small(core::mem::size_of::<u16>().wrapping_mul(n as usize)) {
            (PREVIOUS.get().unwrap_unchecked().memset16)(p, v, n)
        } else {
            repsto16(p, v, n as usize)
        }
    }
    // Source: src/core/SkMemset_opts_erms.cpp:65-67
    pub(super) unsafe fn memset32(p: *mut u32, v: u32, n: i32) {
        if small(core::mem::size_of::<u32>().wrapping_mul(n as usize)) {
            (PREVIOUS.get().unwrap_unchecked().memset32)(p, v, n)
        } else {
            repsto32(p, v, n as usize)
        }
    }
    // Source: src/core/SkMemset_opts_erms.cpp:68-70
    pub(super) unsafe fn memset64(p: *mut u64, v: u64, n: i32) {
        if small(core::mem::size_of::<u64>().wrapping_mul(n as usize)) {
            (PREVIOUS.get().unwrap_unchecked().memset64)(p, v, n)
        } else {
            repsto64(p, v, n as usize)
        }
    }
    // Source: src/core/SkMemset_opts_erms.cpp:72-79
    pub(super) unsafe fn rect_memset16(mut p: *mut u16, v: u16, n: i32, rb: usize, mut h: i32) {
        if small(core::mem::size_of::<u16>().wrapping_mul(n as usize)) {
            return (PREVIOUS.get().unwrap_unchecked().rect_memset16)(p, v, n, rb, h);
        }
        let stride = (rb / core::mem::size_of::<u16>()) as i32;
        while h > 0 {
            repsto16(p, v, n as usize);
            p = p.offset(stride as isize);
            h -= 1;
        }
    }
    // Source: src/core/SkMemset_opts_erms.cpp:80-87
    pub(super) unsafe fn rect_memset32(mut p: *mut u32, v: u32, n: i32, rb: usize, mut h: i32) {
        if small(core::mem::size_of::<u32>().wrapping_mul(n as usize)) {
            return (PREVIOUS.get().unwrap_unchecked().rect_memset32)(p, v, n, rb, h);
        }
        let stride = (rb / core::mem::size_of::<u32>()) as i32;
        while h > 0 {
            repsto32(p, v, n as usize);
            p = p.offset(stride as isize);
            h -= 1;
        }
    }
    // Source: src/core/SkMemset_opts_erms.cpp:88-95
    pub(super) unsafe fn rect_memset64(mut p: *mut u64, v: u64, n: i32, rb: usize, mut h: i32) {
        if small(core::mem::size_of::<u64>().wrapping_mul(n as usize)) {
            return (PREVIOUS.get().unwrap_unchecked().rect_memset64)(p, v, n, rb, h);
        }
        let stride = (rb / core::mem::size_of::<u64>()) as i32;
        while h > 0 {
            repsto64(p, v, n as usize);
            p = p.offset(stride as isize);
            h -= 1;
        }
    }
}
// Source: src/core/SkMemset_opts_erms.cpp:102-118
pub(crate) unsafe fn Init_Memset_erms(d: &mut Dispatch) {
    #[cfg(all(target_arch = "x86_64", not(SK_ENABLE_OPTIMIZE_SIZE)))]
    {
        assert!(erms::PREVIOUS.set(*d).is_ok());
        d.memset16 = erms::memset16;
        d.memset32 = erms::memset32;
        d.memset64 = erms::memset64;
        d.rect_memset16 = erms::rect_memset16;
        d.rect_memset32 = erms::rect_memset32;
        d.rect_memset64 = erms::rect_memset64;
    }
}
