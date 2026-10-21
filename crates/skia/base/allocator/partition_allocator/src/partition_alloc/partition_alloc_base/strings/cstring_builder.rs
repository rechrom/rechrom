use core::ffi::c_char;
use core::{marker::PhantomPinned, mem::MaybeUninit, pin::Pin};
// Source: partition_alloc_base/strings/cstring_builder.h:21-67.
// ptr_ is a true pointer into the same native buffer, initialized at final pinned address.
pub struct CStringBuilder {
    pub(crate) buffer_: [MaybeUninit<c_char>; 256],
    pub(crate) ptr_: *mut c_char,
    _pin: PhantomPinned,
}
impl CStringBuilder {
    pub const kBufferSize: usize = 256;
    // Safety: out is exclusive uninitialized final storage; no move through destruction.
    pub unsafe fn init_at(out: *mut Self) {
        core::ptr::addr_of_mut!((*out).buffer_).write([MaybeUninit::uninit(); 256]);
        core::ptr::addr_of_mut!((*out).ptr_)
            .write(core::ptr::addr_of_mut!((*out).buffer_).cast::<c_char>());
        core::ptr::addr_of_mut!((*out)._pin).write(PhantomPinned);
    }
}
// Ordinary operator<</c_str/private helpers are declared by the native header but defined
// in cstring_builder.cc, not admitted yet. No generated formatter or fallback body.
