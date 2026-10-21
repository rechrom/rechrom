// Source: partition_alloc_base/scoped_clear_last_error.h:27-43
pub struct ScopedClearLastErrorBase {
    last_errno_: core::ffi::c_int,
}
// Real errno accessor is target libc ABI; unsupported targets do not guess a slot.
pub unsafe fn errno_location() -> *mut core::ffi::c_int {
    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
    {
        return libc::__error();
    }
    #[cfg(any(target_os = "linux", target_os = "hurd"))]
    {
        return libc::__errno_location();
    }
    #[cfg(target_os = "android")]
    {
        return libc::__errno();
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "hurd",
        target_os = "android"
    )))]
    compile_error!("actual libc errno accessor ABI pending");
}
impl ScopedClearLastErrorBase {
    pub unsafe fn init_at(out: *mut Self) {
        let slot = errno_location();
        core::ptr::addr_of_mut!((*out).last_errno_).write(*slot);
        *slot = 0;
    }
}
impl Drop for ScopedClearLastErrorBase {
    fn drop(&mut self) {
        unsafe {
            *errno_location() = self.last_errno_;
        }
    }
}
#[cfg(any(pa_is_posix, pa_is_fuchsia))]
pub type ScopedClearLastError = ScopedClearLastErrorBase;
// Actual Windows subclass is declared in header and defined outside this source; not fabricated.
