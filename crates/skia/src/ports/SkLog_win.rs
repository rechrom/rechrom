#![cfg(target_os = "windows")]
//! Translation of src/ports/SkLog_win.cpp (Skia cacf77bdba7b, SK_BUILD_FOR_WIN).
use crate::include::private::SkLogPriority::SkLogPriority;
use core::{
    ffi::{c_char, c_int, c_void, VaList},
    mem::ManuallyDrop,
};

unsafe extern "C" {
    fn __acrt_iob_func(index: u32) -> *mut c_void;
    fn fflush(stream: *mut c_void) -> c_int;
    // ManuallyDrop is transparent over the compiler's actual native VaList.
    fn vfprintf(stream: *mut c_void, format: *const u8, args: ManuallyDrop<VaList<'_>>) -> c_int;
    fn vsnprintf(
        buffer: *mut c_char,
        count: usize,
        format: *const u8,
        args: ManuallyDrop<VaList<'_>>,
    ) -> c_int;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn OutputDebugStringA(output: *const c_char);
}

// Source: src/ports/SkLog_win.cpp:20-20
const kBufferSize: usize = 2048;

// Source: include/private/SkLog.h:20-20
// Source: src/ports/SkLog_win.cpp:22-35
/// Safety: actual native va_list and format/pointer backing remain valid for
/// both synchronous formatting calls. No list escapes its variadic frame.
pub unsafe extern "C" fn SkLogVAList(_priority: SkLogPriority, format: *const u8, args: VaList<'_>) {
    let mut buffer = [0 as c_char; kBufferSize + 1];
    let stderr = unsafe { __acrt_iob_func(2) };

    // va_copy: on Windows x64 the va_list is a pointer to the argument area, so
    // the copy is a second non-owning projection of the same list.
    let args_copy = ManuallyDrop::new(unsafe { core::ptr::read(&args) });
    unsafe {
        vfprintf(stderr, format, args_copy);
        fflush(stderr); // stderr seems to be buffered on Windows.
    }

    let forwarded = ManuallyDrop::new(unsafe { core::ptr::read(&args) });
    unsafe {
        vsnprintf(buffer.as_mut_ptr(), kBufferSize, format, forwarded);
        OutputDebugStringA(buffer.as_ptr());
    }
    drop(args); // native va_end, after C has finished using the list
}
