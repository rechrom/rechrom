// quickjs.c:48486-48493 and libregexp.h embedding contract. MIT.
use crate::libregexp::{lre_get_alloc_count, lre_get_capture_count, lre_get_flags, lre_get_groupnames};
use crate::libregexp_header::{
    LRE_FLAG_DOTALL, LRE_FLAG_GLOBAL, LRE_FLAG_IGNORECASE, LRE_FLAG_INDICES,
    LRE_FLAG_MULTILINE, LRE_FLAG_NAMED_GROUPS, LRE_FLAG_STICKY, LRE_FLAG_UNICODE,
    LRE_FLAG_UNICODE_SETS, LRE_RET_MEMORY_ERROR, LRE_RET_TIMEOUT,
};
#[repr(C)]
struct ValueBuffer {
    ctx: *mut JSContext,
    arr: *mut JSValue,
    def: [JSValue; 4],
    len: i32,
    size: i32,
    error_status: i32,
}
unsafe fn regexp_strlen(text: *const c_char) -> usize {
    core::ffi::CStr::from_ptr(text).to_bytes().len()
}
unsafe fn regexp_strchr(text: *const c_char, value: i32) -> *mut c_char {
    let needle = value as u8;
    match core::ffi::CStr::from_ptr(text).to_bytes_with_nul().iter().position(|&x| x == needle) {
        Some(i) => text.add(i).cast_mut(),
        None => ptr::null_mut(),
    }
}
unsafe fn regexp_escape_hex(dst: *mut c_char, capacity: usize, value: u32, width: usize) -> i32 {
    let hex_digit_count = ((32 - value.leading_zeros()) as usize).div_ceil(4).max(1).max(width);
    let mut bytes = [0u8; 16];
    bytes[0] = b'\\';
    bytes[1] = if width == 2 { b'x' } else { b'u' };
    let hex = b"0123456789abcdef";
    for i in 0..hex_digit_count {
        bytes[i + 2] = hex[((value >> (4 * (hex_digit_count - i - 1))) & 15) as usize];
    }
    let len = hex_digit_count + 2;
    if capacity != 0 {
        let written = len.min(capacity - 1);
        ptr::copy_nonoverlapping(bytes.as_ptr(), dst.cast(), written);
        *dst.add(written) = 0;
    }
    len as i32
}

// C uses embedding-supplied global callbacks. The pure Rust regexp crate
// receives the same callbacks through LREHost, preserving the original opaque
// context and allocator/stack/interrupt policies without global mutable state.
unsafe fn lre_compile(
    plen: *mut i32, error: *mut c_char, error_size: i32,
    text: *const c_char, text_len: usize, flags: i32, opaque: *mut c_void,
) -> *mut u8 {
    let mut host = crate::libregexp_header::LREHost {
        opaque,
        check_stack_overflow: lre_check_stack_overflow,
        check_timeout: lre_check_timeout,
        realloc: lre_realloc,
    };
    crate::libregexp::lre_compile(
        plen, error, error_size, text, text_len, flags,
        ptr::addr_of_mut!(host).cast(),
    )
}
unsafe fn lre_exec(
    capture: *mut *mut u8, bytecode: *const u8, text: *const u8,
    index: i32, len: i32, wide: i32, opaque: *mut c_void,
) -> i32 {
    let mut host = crate::libregexp_header::LREHost {
        opaque,
        check_stack_overflow: lre_check_stack_overflow,
        check_timeout: lre_check_timeout,
        realloc: lre_realloc,
    };
    crate::libregexp::lre_exec(
        capture, bytecode, text, index, len, wide, ptr::addr_of_mut!(host).cast(),
    )
}
