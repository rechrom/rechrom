#![cfg(not(any(target_os = "windows", target_os = "android")))]
use crate::include::private::SkLogPriority::SkLogPriority;
use core::{
    ffi::{c_int, c_void, VaList},
    mem::ManuallyDrop,
};
#[cfg(target_vendor = "apple")]
unsafe extern "C" {
    static mut __stderrp: *mut c_void;
}
#[cfg(not(target_vendor = "apple"))]
unsafe extern "C" {
    static mut stderr: *mut c_void;
}
unsafe extern "C" {
    // ManuallyDrop is transparent over the compiler's actual native VaList.
    // C borrows this FFI carrier and cannot execute a Rust destructor for it.
    fn vfprintf(stream: *mut c_void, format: *const u8, args: ManuallyDrop<VaList<'_>>) -> c_int;
}
// Source: include/private/SkLog.h:20-20
// Source: src/ports/SkLog_stdio.cpp:19-24
/// Safety: actual native va_list and format/pointer backing remain valid for
/// the complete synchronous vfprintf call. No list escapes its variadic frame.
pub unsafe extern "C" fn SkLogVAList(
    _priority: SkLogPriority,
    format: *const u8,
    args: VaList<'_>,
) {
    #[cfg(target_vendor = "apple")]
    let stream = unsafe { __stderrp };
    #[cfg(not(target_vendor = "apple"))]
    let stream = unsafe { stderr };
    // This is only a non-owning synchronous ABI projection, not va_copy or a
    // fabricated layout. args remains the real va_end owner until libc returns.
    let forwarded = ManuallyDrop::new(unsafe { core::ptr::read(&args) });
    unsafe {
        vfprintf(stream, format, forwarded);
    }
    drop(args); // native va_end, after C has finished using the list
}
