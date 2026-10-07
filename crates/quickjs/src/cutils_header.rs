//! Direct translation of cutils.h. Copyright 2017 Fabrice Bellard,
//! 2018 Charlie Gordon. MIT; see ../LICENSE.
use core::{ffi::c_void, ptr};

pub type BOOL = i32;
pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;
pub const UTF8_CHAR_LEN_MAX: usize = 6;

#[macro_export]
macro_rules! countof {
    ($array:expr) => {
        $array.len()
    };
}
#[macro_export]
macro_rules! container_of {
    ($pointer:expr, $ty:ty, $member:ident) => {
        ($pointer as *mut u8).wrapping_sub(core::mem::offset_of!($ty, $member)) as *mut $ty
    };
}
// Compiler hints/attributes have no observable C runtime behavior. Rust call
// sites use #[inline(always)], #[inline(never)], and #[allow(unused)] as needed.
#[inline(always)]
pub const fn likely(v: bool) -> bool {
    v
}
#[inline(always)]
pub const fn unlikely(v: bool) -> bool {
    v
}

pub unsafe fn memcpy_no_ub(dest: *mut c_void, src: *const c_void, n: usize) {
    if n != 0 {
        ptr::copy_nonoverlapping(src.cast::<u8>(), dest.cast::<u8>(), n);
    }
}
#[inline]
pub const fn max_int(a: i32, b: i32) -> i32 {
    if a > b {
        a
    } else {
        b
    }
}
#[inline]
pub const fn min_int(a: i32, b: i32) -> i32 {
    if a < b {
        a
    } else {
        b
    }
}
#[inline]
pub const fn max_uint32(a: u32, b: u32) -> u32 {
    if a > b {
        a
    } else {
        b
    }
}
#[inline]
pub const fn min_uint32(a: u32, b: u32) -> u32 {
    if a < b {
        a
    } else {
        b
    }
}
#[inline]
pub const fn max_int64(a: i64, b: i64) -> i64 {
    if a > b {
        a
    } else {
        b
    }
}
#[inline]
pub const fn min_int64(a: i64, b: i64) -> i64 {
    if a < b {
        a
    } else {
        b
    }
}
// The C builtins have a nonzero-input precondition.
#[inline]
pub fn clz32(a: u32) -> i32 {
    assert_ne!(a, 0);
    a.leading_zeros() as i32
}
#[inline]
pub fn clz64(a: u64) -> i32 {
    assert_ne!(a, 0);
    a.leading_zeros() as i32
}
#[inline]
pub fn ctz32(a: u32) -> i32 {
    assert_ne!(a, 0);
    a.trailing_zeros() as i32
}
#[inline]
pub fn ctz64(a: u64) -> i32 {
    assert_ne!(a, 0);
    a.trailing_zeros() as i32
}

#[repr(C, packed)]
pub struct packed_u64 {
    pub v: u64,
}
#[repr(C, packed)]
pub struct packed_u32 {
    pub v: u32,
}
#[repr(C, packed)]
pub struct packed_u16 {
    pub v: u16,
}
#[inline]
pub unsafe fn get_u64(p: *const u8) -> u64 {
    ptr::read_unaligned(p.cast())
}
#[inline]
pub unsafe fn get_i64(p: *const u8) -> i64 {
    get_u64(p) as i64
}
#[inline]
pub unsafe fn put_u64(p: *mut u8, v: u64) {
    ptr::write_unaligned(p.cast(), v);
}
#[inline]
pub unsafe fn get_u32(p: *const u8) -> u32 {
    ptr::read_unaligned(p.cast())
}
#[inline]
pub unsafe fn get_i32(p: *const u8) -> i32 {
    get_u32(p) as i32
}
#[inline]
pub unsafe fn put_u32(p: *mut u8, v: u32) {
    ptr::write_unaligned(p.cast(), v);
}
#[inline]
pub unsafe fn get_u16(p: *const u8) -> u32 {
    ptr::read_unaligned(p.cast::<u16>()) as u32
}
#[inline]
pub unsafe fn get_i16(p: *const u8) -> i32 {
    get_u16(p) as i16 as i32
}
#[inline]
pub unsafe fn put_u16(p: *mut u8, v: u16) {
    ptr::write_unaligned(p.cast(), v);
}
#[inline]
pub unsafe fn get_u8(p: *const u8) -> u32 {
    *p as u32
}
#[inline]
pub unsafe fn get_i8(p: *const u8) -> i32 {
    *p as i8 as i32
}
#[inline]
pub unsafe fn put_u8(p: *mut u8, v: u8) {
    *p = v;
}
#[inline]
pub const fn bswap16(v: u16) -> u16 {
    v.swap_bytes()
}
#[inline]
pub const fn bswap32(v: u32) -> u32 {
    v.swap_bytes()
}
#[inline]
pub const fn bswap64(v: u64) -> u64 {
    v.swap_bytes()
}

pub type DynBufReallocFunc =
    unsafe fn(opaque: *mut c_void, ptr: *mut c_void, size: usize) -> *mut c_void;
#[repr(C)]
pub struct DynBuf {
    pub buf: *mut u8,
    pub size: usize,
    pub allocated_size: usize,
    pub error: BOOL,
    pub realloc_func: Option<DynBufReallocFunc>,
    pub opaque: *mut c_void,
}
impl Default for DynBuf {
    fn default() -> Self {
        Self {
            buf: ptr::null_mut(),
            size: 0,
            allocated_size: 0,
            error: FALSE,
            realloc_func: None,
            opaque: ptr::null_mut(),
        }
    }
}
#[inline]
pub unsafe fn dbuf_putc(s: *mut DynBuf, val: u8) -> i32 {
    if (*s).allocated_size.wrapping_sub((*s).size) < 1 {
        super::cutils::__dbuf_putc(s, val)
    } else {
        *(*s).buf.add((*s).size) = val;
        (*s).size += 1;
        0
    }
}
#[inline]
pub unsafe fn dbuf_put_u16(s: *mut DynBuf, val: u16) -> i32 {
    if (*s).allocated_size.wrapping_sub((*s).size) < 2 {
        super::cutils::__dbuf_put_u16(s, val)
    } else {
        put_u16((*s).buf.add((*s).size), val);
        (*s).size += 2;
        0
    }
}
#[inline]
pub unsafe fn dbuf_put_u32(s: *mut DynBuf, val: u32) -> i32 {
    if (*s).allocated_size.wrapping_sub((*s).size) < 4 {
        super::cutils::__dbuf_put_u32(s, val)
    } else {
        put_u32((*s).buf.add((*s).size), val);
        (*s).size += 4;
        0
    }
}
#[inline]
pub unsafe fn dbuf_put_u64(s: *mut DynBuf, val: u64) -> i32 {
    if (*s).allocated_size.wrapping_sub((*s).size) < 8 {
        super::cutils::__dbuf_put_u64(s, val)
    } else {
        put_u64((*s).buf.add((*s).size), val);
        (*s).size += 8;
        0
    }
}
#[inline]
pub unsafe fn dbuf_error(s: *const DynBuf) -> BOOL {
    (*s).error
}
#[inline]
pub unsafe fn dbuf_set_error(s: *mut DynBuf) {
    (*s).error = TRUE;
}

#[inline]
pub const fn is_surrogate(c: u32) -> BOOL {
    ((c >> 11) == (0xD800 >> 11)) as BOOL
}
#[inline]
pub const fn is_hi_surrogate(c: u32) -> BOOL {
    ((c >> 10) == (0xD800 >> 10)) as BOOL
}
#[inline]
pub const fn is_lo_surrogate(c: u32) -> BOOL {
    ((c >> 10) == (0xDC00 >> 10)) as BOOL
}
#[inline]
pub const fn get_hi_surrogate(c: u32) -> u32 {
    (c >> 10).wrapping_sub(0x10000 >> 10).wrapping_add(0xD800)
}
#[inline]
pub const fn get_lo_surrogate(c: u32) -> u32 {
    (c & 0x3FF) | 0xDC00
}
#[inline]
pub const fn from_surrogate(hi: u32, lo: u32) -> u32 {
    0x10000u32
        .wrapping_add(0x400u32.wrapping_mul(hi.wrapping_sub(0xD800)))
        .wrapping_add(lo.wrapping_sub(0xDC00))
}
#[inline]
pub const fn from_hex(c: i32) -> i32 {
    if c >= b'0' as i32 && c <= b'9' as i32 {
        c - b'0' as i32
    } else if c >= b'A' as i32 && c <= b'F' as i32 {
        c - b'A' as i32 + 10
    } else if c >= b'a' as i32 && c <= b'f' as i32 {
        c - b'a' as i32 + 10
    } else {
        -1
    }
}
#[inline]
pub fn float64_as_uint64(d: f64) -> u64 {
    d.to_bits()
}
#[inline]
pub fn uint64_as_float64(u: u64) -> f64 {
    f64::from_bits(u)
}
#[inline]
pub fn fromfp16(v: u16) -> f64 {
    let mut v1 = (v & 0x7fff) as u32;
    if v1 >= 0x7c00 {
        v1 += 0x1f8000;
    }
    let d = uint64_as_float64(((v as u64 >> 15) << 63) | ((v1 as u64) << (52 - 10)));
    d * f64::from_bits((1008u64 + 1023) << 52)
}
#[inline]
pub fn tofp16(d: f64) -> u16 {
    let mut a = float64_as_uint64(d);
    let sgn = (a >> 63) as u32;
    a &= 0x7fffffffffffffff;
    let mut v: u32;
    if a > 0x7ff0000000000000 {
        v = 0x7c01;
    } else if a < 0x3f10000000000000 {
        if a <= 0x3e60000000000000 {
            v = 0;
        } else {
            let shift = 1051 - (a >> 52);
            a = (1u64 << 52) | (a & ((1u64 << 52) - 1));
            let addend = ((a >> shift) & 1) + ((1u64 << (shift - 1)) - 1);
            v = ((a + addend) >> shift) as u32;
        }
    } else {
        a -= 0x3f00000000000000;
        let addend = ((a >> (52 - 10)) & 1) + ((1u64 << (52 - 11)) - 1);
        v = ((a + addend) >> (52 - 10)) as u32;
        if v > 0x7c00 {
            v = 0x7c00;
        }
    }
    (v | (sgn << 15)) as u16
}
#[inline]
pub const fn isfp16nan(v: u16) -> i32 {
    ((v & 0x7fff) > 0x7c00) as i32
}
#[inline]
pub const fn isfp16zero(v: u16) -> i32 {
    ((v & 0x7fff) == 0) as i32
}
