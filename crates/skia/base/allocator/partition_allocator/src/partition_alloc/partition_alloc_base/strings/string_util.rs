use core::ffi::c_char;
// Source: partition_alloc_base/strings/string_util.cc: FindLastOf/FindLastNotOf.
// Reverse index represents the native reverse pointer walk without forming a before-begin pointer.
pub unsafe fn FindLastOf(text: *const c_char, characters: *const c_char) -> *const c_char {
    let mut length = libc::strlen(text);
    while length != 0 {
        length -= 1;
        let p = text.add(length);
        if !libc::strchr(characters, *p as core::ffi::c_int).is_null() {
            return p;
        }
    }
    core::ptr::null()
}
pub unsafe fn FindLastNotOf(text: *const c_char, characters: *const c_char) -> *const c_char {
    let mut length = libc::strlen(text);
    while length != 0 {
        length -= 1;
        let p = text.add(length);
        if libc::strchr(characters, *p as core::ffi::c_int).is_null() {
            return p;
        }
    }
    core::ptr::null()
}
