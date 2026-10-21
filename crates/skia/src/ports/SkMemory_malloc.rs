//! Source: src/ports/SkMemory_malloc.cpp. Actual libc allocation and native abort port.
use core::ffi::c_void;
// Source: include/private/SkMalloc.h:34-41
const SK_MALLOC_ZERO_INITIALIZE: u32 = 1 << 0;
const SK_MALLOC_THROW: u32 = 1 << 1;
// Source: src/ports/SkMemory_malloc.cpp:32-36
// This is a file-local native macro. In the non-framework release configuration
// neither the debug log arguments nor the false assertion expression are evaluated.
macro_rules! memory_debugfailf {
    ($format:literal,$argument:expr)=>{{
        #[cfg(sk_build_for_android_framework)]
        {crate::SK_ABORT!(@source b"src/ports/SkMemory_malloc.cpp\0".as_ptr(),39;$format,$argument);}
        #[cfg(all(not(sk_build_for_android_framework),sk_debug))]
        {crate::SkASSERT!(@source b"src/ports/SkMemory_malloc.cpp\0".as_ptr(),39;
          @condition b"(SkDebugf(fmt\"\\n\", __VA_ARGS__), false)\0".as_ptr();
          {crate::SkDebugf!(@source b"src/ports/SkMemory_malloc.cpp\0".as_ptr(),39;concat!($format,"\n\0").as_ptr(),$argument);false});}
    }};
}
// Source: src/ports/SkMemory_malloc.cpp:38-46
unsafe fn sk_out_of_memory_size(size: usize) -> ! {
    memory_debugfailf!("sk_out_of_memory (asked for %zu bytes)", size);
    #[cfg(SK_BUILD_FOR_AFL_FUZZ)]
    unsafe {
        libc::exit(1);
    }
    #[cfg(not(SK_BUILD_FOR_AFL_FUZZ))]
    unsafe {
        libc::abort();
    }
}
// Source: src/ports/SkMemory_malloc.cpp:48-54
unsafe fn throw_on_failure(size: usize, p: *mut c_void) -> *mut c_void {
    if size > 0 && p.is_null() {
        unsafe {
            sk_out_of_memory_size(size);
        }
    }
    p
}
// Source: src/ports/SkMemory_malloc.cpp:27-29
// Resolved native input applies the original #ifndef default (7) only when the
// actual FAST_FAIL_FATAL_APP_EXIT input is undefined. No override is discarded.
#[cfg(any(
    all(sk_debug, target_os = "windows"),
    all(sk_build_for_android_framework, sk_bionic)
))]
mod native_port_inputs {
    include!(concat!(
        env!("OUT_DIR"),
        "/sk_native_compiler_port_inputs.rs"
    ));
}
// Source: src/ports/SkMemory_malloc.cpp:56-64
pub unsafe fn sk_abort_no_print() -> ! {
    #[cfg(all(sk_debug, target_os = "windows"))]
    unsafe {
        crate::include::private::SkAssert::native_fastfail(
            native_port_inputs::FAST_FAIL_FATAL_APP_EXIT,
        );
    }
    #[cfg(all(not(all(sk_debug, target_os = "windows")), sk_native_clang))]
    {
        core::intrinsics::abort()
    }
    #[cfg(all(not(all(sk_debug, target_os = "windows")), not(sk_native_clang)))]
    unsafe {
        libc::abort()
    }
}
// Source: src/ports/SkMemory_malloc.cpp:66-73
pub unsafe fn sk_out_of_memory() -> ! {
    crate::SkDEBUGFAIL!(@source b"src/ports/SkMemory_malloc.cpp\0".as_ptr(),67;b"sk_out_of_memory\0".as_ptr());
    #[cfg(SK_BUILD_FOR_AFL_FUZZ)]
    unsafe {
        libc::exit(1);
    }
    #[cfg(not(SK_BUILD_FOR_AFL_FUZZ))]
    unsafe {
        libc::abort();
    }
}
// Source: src/ports/SkMemory_malloc.cpp:75-81
pub unsafe fn sk_realloc_throw(addr: *mut c_void, size: usize) -> *mut c_void {
    if size == 0 {
        unsafe {
            sk_free(addr);
        }
        return core::ptr::null_mut();
    }
    unsafe { throw_on_failure(size, libc::realloc(addr, size)) }
}
// Source: src/ports/SkMemory_malloc.cpp:83-89
pub unsafe fn sk_free(p: *mut c_void) {
    if !p.is_null() {
        unsafe {
            libc::free(p);
        }
    }
}
// Source: src/ports/SkMemory_malloc.cpp:91-117
pub unsafe fn sk_malloc_flags(size: usize, flags: u32) -> *mut c_void {
    let p;
    if flags & SK_MALLOC_ZERO_INITIALIZE != 0 {
        p = unsafe { libc::calloc(size, 1) };
    } else {
        // Source: src/ports/SkMemory_malloc.cpp:96-106
        // Bionic's real libc/include/malloc.h declares mallopt(int,int), API26,
        // and M_THREAD_DISABLE_MEM_INIT (API31). Root supplies the resolved SDK
        // macro value, rather than assuming a host SDK or inventing an option.
        #[cfg(all(sk_build_for_android_framework, sk_bionic))]
        {
            unsafe extern "C" {
                fn mallopt(option: core::ffi::c_int, value: core::ffi::c_int) -> core::ffi::c_int;
            }
            let _ = unsafe { mallopt(native_port_inputs::M_THREAD_DISABLE_MEM_INIT, 1) };
        }
        // Source: src/ports/SkMemory_malloc.cpp:107-107
        p = unsafe { libc::malloc(size) };
        // Source: src/ports/SkMemory_malloc.cpp:108-110
        #[cfg(all(sk_build_for_android_framework, sk_bionic))]
        {
            unsafe extern "C" {
                fn mallopt(option: core::ffi::c_int, value: core::ffi::c_int) -> core::ffi::c_int;
            }
            let _ = unsafe { mallopt(native_port_inputs::M_THREAD_DISABLE_MEM_INIT, 0) };
        }
    }
    // Source: src/ports/SkMemory_malloc.cpp:112-116
    if flags & SK_MALLOC_THROW != 0 {
        unsafe { throw_on_failure(size, p) }
    } else {
        p
    }
}
// Source: src/ports/SkMemory_malloc.cpp:119-138
/// Safety: addr is an allocation accepted by this platform's native malloc size API;
/// size is its actual original request. No Rust allocator pointer may be substituted.
pub unsafe fn sk_malloc_size(addr: *mut c_void, size: usize) -> usize {
    let mut completeSize = size;
    #[cfg(target_vendor = "apple")]
    {
        unsafe extern "C" {
            fn malloc_size(p: *const c_void) -> usize;
        }
        completeSize = unsafe { malloc_size(addr) }.max(size);
    }
    #[cfg(all(target_os = "android", sk_android_api_at_least_17))]
    {
        completeSize = unsafe { libc::malloc_usable_size(addr) };
        crate::SkASSERT!(@source b"src/ports/SkMemory_malloc.cpp\0".as_ptr(),128;@condition b"completeSize >= size\0".as_ptr();completeSize>=size);
    }
    #[cfg(all(unix, not(target_vendor = "apple"), not(target_os = "android")))]
    {
        completeSize = unsafe { libc::malloc_usable_size(addr) };
        crate::SkASSERT!(@source b"src/ports/SkMemory_malloc.cpp\0".as_ptr(),131;@condition b"completeSize >= size\0".as_ptr();completeSize>=size);
    }
    #[cfg(target_os = "windows")]
    {
        unsafe extern "C" {
            fn _msize(p: *mut c_void) -> usize;
        }
        completeSize = unsafe { _msize(addr) };
        crate::SkASSERT!(@source b"src/ports/SkMemory_malloc.cpp\0".as_ptr(),134;@condition b"completeSize >= size\0".as_ptr();completeSize>=size);
    }
    completeSize
}
