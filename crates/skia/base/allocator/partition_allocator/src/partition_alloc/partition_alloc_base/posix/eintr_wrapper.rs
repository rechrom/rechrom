// Source: partition_alloc_base/posix/eintr_wrapper.h: actual copied lambda and native int out.
// Known producer instantiations return int/ssize_t. Every attempt narrows BEFORE the -1 check.
pub trait IntResult {
    fn int(self) -> core::ffi::c_int;
}
impl IntResult for core::ffi::c_int {
    fn int(self) -> core::ffi::c_int {
        self
    }
}
impl IntResult for isize {
    fn int(self) -> core::ffi::c_int {
        self as core::ffi::c_int
    }
}
#[cfg(pa_is_posix)]
pub fn WrapEINTR<F, R>(mut f: F) -> impl FnMut() -> core::ffi::c_int
where
    F: FnMut() -> R,
    R: IntResult,
{
    move || {
        let mut out = -1;
        #[cfg(pa_is_debug)]
        let mut retry_count = 0;
        loop {
            #[cfg(pa_is_debug)]
            {
                if retry_count >= 100 {
                    break;
                }
                retry_count += 1;
            }
            out = f().int();
            if out != -1
                || unsafe { *super::super::scoped_clear_last_error::errno_location() }
                    != libc::EINTR
            {
                return out;
            }
        }
        out
    }
}
#[cfg(not(pa_is_posix))]
pub fn WrapEINTR<F>(f: F) -> F {
    f
}
