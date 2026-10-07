//! libregexp.h. Copyright 2017-2018 Fabrice Bellard; MIT, see ../LICENSE.
//! Stack/timeout/allocation hooks are injected by the caller, as required by C.
use super::cutils::DynBufReallocFunc;
use core::ffi::c_void;
pub const LRE_FLAG_GLOBAL: i32 = 1 << 0;
pub const LRE_FLAG_IGNORECASE: i32 = 1 << 1;
pub const LRE_FLAG_MULTILINE: i32 = 1 << 2;
pub const LRE_FLAG_DOTALL: i32 = 1 << 3;
pub const LRE_FLAG_UNICODE: i32 = 1 << 4;
pub const LRE_FLAG_STICKY: i32 = 1 << 5;
pub const LRE_FLAG_INDICES: i32 = 1 << 6;
pub const LRE_FLAG_NAMED_GROUPS: i32 = 1 << 7;
pub const LRE_FLAG_UNICODE_SETS: i32 = 1 << 8;
pub const LRE_RET_MEMORY_ERROR: i32 = -1;
pub const LRE_RET_TIMEOUT: i32 = -2;
pub const LRE_GROUP_NAME_TRAILER_LEN: usize = 2;
/// C leaves these three functions to the embedding application. Rust supplies
/// a concrete dispatcher; opaque passed to libregexp is a stable LREHost
/// pointer. Callbacks receive the embedding application's original opaque.
#[repr(C)]
pub struct LREHost {
    pub opaque: *mut c_void,
    pub check_stack_overflow: unsafe fn(*mut c_void, usize) -> i32,
    pub check_timeout: unsafe fn(*mut c_void) -> i32,
    pub realloc: DynBufReallocFunc,
}
pub unsafe fn lre_check_stack_overflow(opaque: *mut c_void, alloca_size: usize) -> i32 {
    let host = &*opaque.cast::<LREHost>();
    (host.check_stack_overflow)(host.opaque, alloca_size)
}
pub unsafe fn lre_check_timeout(opaque: *mut c_void) -> i32 {
    let host = &*opaque.cast::<LREHost>();
    (host.check_timeout)(host.opaque)
}
pub unsafe fn lre_realloc(opaque: *mut c_void, p: *mut c_void, size: usize) -> *mut c_void {
    let host = &*opaque.cast::<LREHost>();
    (host.realloc)(host.opaque, p, size)
}
