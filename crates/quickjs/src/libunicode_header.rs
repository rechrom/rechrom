//! libunicode.h, Unicode 17.0.0. Copyright 2017-2018 Fabrice Bellard.
//! MIT; see ../LICENSE. CONFIG_ALL_UNICODE is enabled as in the source header.
use super::cutils::DynBufReallocFunc;
use super::libunicode::*;
use core::{ffi::c_void, ptr};
pub const LIBUNICODE_UNICODE_VERSION_MAJOR: i32 = 17;
pub const LIBUNICODE_UNICODE_VERSION_MINOR: i32 = 0;
pub const LIBUNICODE_UNICODE_VERSION_PATCH: i32 = 0;
pub const LRE_CC_RES_LEN_MAX: usize = 3;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CharRange {
    pub len: i32,
    pub size: i32,
    pub points: *mut u32,
    pub mem_opaque: *mut c_void,
    pub realloc_func: Option<DynBufReallocFunc>,
}
impl Default for CharRange {
    fn default() -> Self {
        Self {
            len: 0,
            size: 0,
            points: ptr::null_mut(),
            mem_opaque: ptr::null_mut(),
            realloc_func: None,
        }
    }
}
pub type CharRangeOpEnum = i32;
pub const CR_OP_UNION: CharRangeOpEnum = 0;
pub const CR_OP_INTER: CharRangeOpEnum = 1;
pub const CR_OP_XOR: CharRangeOpEnum = 2;
pub const CR_OP_SUB: CharRangeOpEnum = 3;
#[inline]
pub unsafe fn cr_add_point(cr: *mut CharRange, v: u32) -> i32 {
    if (*cr).len >= (*cr).size && cr_realloc(cr, (*cr).len + 1) != 0 {
        return -1;
    }
    *(*cr).points.add((*cr).len as usize) = v;
    (*cr).len += 1;
    0
}
#[inline]
pub unsafe fn cr_add_interval(cr: *mut CharRange, c1: u32, c2: u32) -> i32 {
    if (*cr).len + 2 > (*cr).size && cr_realloc(cr, (*cr).len + 2) != 0 {
        return -1;
    }
    *(*cr).points.add((*cr).len as usize) = c1;
    (*cr).len += 1;
    *(*cr).points.add((*cr).len as usize) = c2;
    (*cr).len += 1;
    0
}
#[inline]
pub unsafe fn cr_union_interval(cr: *mut CharRange, c1: u32, c2: u32) -> i32 {
    let b_pt = [c1, c2.wrapping_add(1)];
    cr_op1(cr, b_pt.as_ptr(), 2, CR_OP_UNION)
}
pub type UnicodeNormalizationEnum = i32;
pub const UNICODE_NFC: UnicodeNormalizationEnum = 0;
pub const UNICODE_NFD: UnicodeNormalizationEnum = 1;
pub const UNICODE_NFKC: UnicodeNormalizationEnum = 2;
pub const UNICODE_NFKD: UnicodeNormalizationEnum = 3;
pub type UnicodeSequencePropCB = unsafe fn(*mut c_void, *const u32, i32);
pub const UNICODE_C_SPACE: u8 = 1 << 0;
pub const UNICODE_C_DIGIT: u8 = 1 << 1;
pub const UNICODE_C_UPPER: u8 = 1 << 2;
pub const UNICODE_C_LOWER: u8 = 1 << 3;
pub const UNICODE_C_UNDER: u8 = 1 << 4;
pub const UNICODE_C_DOLLAR: u8 = 1 << 5;
pub const UNICODE_C_XDIGIT: u8 = 1 << 6;
#[inline]
pub fn lre_is_space_byte(c: u8) -> i32 {
    (lre_ctype_bits[c as usize] & UNICODE_C_SPACE) as i32
}
#[inline]
pub fn lre_is_id_start_byte(c: u8) -> i32 {
    (lre_ctype_bits[c as usize]
        & (UNICODE_C_UPPER | UNICODE_C_LOWER | UNICODE_C_UNDER | UNICODE_C_DOLLAR)) as i32
}
#[inline]
pub fn lre_is_id_continue_byte(c: u8) -> i32 {
    (lre_ctype_bits[c as usize]
        & (UNICODE_C_UPPER
            | UNICODE_C_LOWER
            | UNICODE_C_UNDER
            | UNICODE_C_DOLLAR
            | UNICODE_C_DIGIT)) as i32
}
#[inline]
pub fn lre_is_word_byte(c: u8) -> i32 {
    (lre_ctype_bits[c as usize]
        & (UNICODE_C_UPPER | UNICODE_C_LOWER | UNICODE_C_UNDER | UNICODE_C_DIGIT)) as i32
}
#[inline]
pub fn lre_is_space(c: u32) -> i32 {
    if c < 256 {
        lre_is_space_byte(c as u8)
    } else {
        lre_is_space_non_ascii(c)
    }
}
#[inline]
pub fn lre_js_is_ident_first(c: u32) -> i32 {
    if c < 128 {
        lre_is_id_start_byte(c as u8)
    } else {
        lre_is_id_start(c)
    }
}
#[inline]
pub fn lre_js_is_ident_next(c: u32) -> i32 {
    if c < 128 {
        lre_is_id_continue_byte(c as u8)
    } else if c >= 0x200C && c <= 0x200D {
        1
    } else {
        lre_is_id_continue(c)
    }
}
