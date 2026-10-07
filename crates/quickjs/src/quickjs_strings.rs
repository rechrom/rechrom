// quickjs.c:3923-4427. MIT. Context/error and ToString dependencies are
// not yet integrated. This file is compiled by the isolated source oracle.
use crate::cutils_header::{
    from_surrogate, get_hi_surrogate, get_lo_surrogate, is_hi_surrogate, is_lo_surrogate,
};
// quickjs.c:45376-45394. Preserve the first lone-surrogate index.
unsafe fn js_string_find_invalid_codepoint(p: *mut JSString) -> i32 {
    if (*p).is_wide_char() == 0 {
        return -1;
    }
    let units = ptr::addr_of!((*p).u).cast::<u16>();
    let mut i = 0i32;
    while (i as u32) < (*p).len() {
        let c = *units.offset(i as isize) as u32;
        if crate::cutils_header::is_surrogate(c) != 0 {
            if is_hi_surrogate(c) != 0
                && ((i + 1) as u32) < (*p).len()
                && is_lo_surrogate(*units.offset((i + 1) as isize) as u32) != 0
            {
                i += 1;
            } else {
                return i;
            }
        }
        i += 1;
    }
    -1
}
const JS_STRING_LEN_MAX: i32 = (1 << 30) - 1;
unsafe fn js_new_string8_len(ctx: *mut JSContext, buf: *const c_char, len: i32) -> JSValue {
    if len <= 0 {
        return JS_AtomToString(ctx, JS_ATOM_empty_string);
    }
    let str = js_alloc_string(ctx, len, 0);
    if str.is_null() {
        return JS_EXCEPTION;
    }
    let dst = ptr::addr_of_mut!((*str).u).cast::<u8>();
    ptr::copy_nonoverlapping(buf.cast::<u8>(), dst, len as usize);
    *dst.add(len as usize) = 0;
    JS_MKPTR(JS_TAG_STRING, str.cast())
}
unsafe fn js_new_string8(ctx: *mut JSContext, buf: *const c_char) -> JSValue {
    js_new_string8_len(
        ctx,
        buf,
        core::ffi::CStr::from_ptr(buf).to_bytes().len() as i32,
    )
}
unsafe fn js_new_string16_len(ctx: *mut JSContext, buf: *const u16, len: i32) -> JSValue {
    let str = js_alloc_string(ctx, len, 1);
    if str.is_null() {
        return JS_EXCEPTION;
    }
    ptr::copy_nonoverlapping(buf, ptr::addr_of_mut!((*str).u).cast::<u16>(), len as usize);
    JS_MKPTR(JS_TAG_STRING, str.cast())
}
unsafe fn js_new_string_char(ctx: *mut JSContext, c: u16) -> JSValue {
    if c < 0x100 {
        let ch8 = c as u8;
        js_new_string8_len(ctx, ptr::addr_of!(ch8).cast(), 1)
    } else {
        js_new_string16_len(ctx, &c, 1)
    }
}
unsafe fn js_sub_string(ctx: *mut JSContext, p: *mut JSString, start: i32, end: i32) -> JSValue {
    let len = end.wrapping_sub(start);
    if start == 0 && end as u32 == (*p).len() {
        return JS_DupValue(ctx, JS_MKPTR(JS_TAG_STRING, p.cast()));
    }
    if (*p).is_wide_char() != 0 && len > 0 {
        let src = ptr::addr_of!((*p).u).cast::<u16>();
        let mut c = 0u16;
        for i in start..end {
            c |= *src.offset(i as isize);
        }
        if c > 0xff {
            return js_new_string16_len(ctx, src.offset(start as isize), len);
        }
        let str = js_alloc_string(ctx, len, 0);
        if str.is_null() {
            return JS_EXCEPTION;
        }
        let dst = ptr::addr_of_mut!((*str).u).cast::<u8>();
        for i in 0..len {
            *dst.offset(i as isize) = *src.offset(start.wrapping_add(i) as isize) as u8;
        }
        *dst.offset(len as isize) = 0;
        JS_MKPTR(JS_TAG_STRING, str.cast())
    } else {
        js_new_string8_len(
            ctx,
            ptr::addr_of!((*p).u)
                .cast::<u8>()
                .offset(start as isize)
                .cast(),
            len,
        )
    }
}
#[repr(C)]
struct StringBuffer {
    ctx: *mut JSContext,
    str: *mut JSString,
    len: i32,
    size: i32,
    is_wide_char: i32,
    error_status: i32,
}
unsafe fn string_buffer_init2(
    ctx: *mut JSContext,
    s: *mut StringBuffer,
    size: i32,
    is_wide: i32,
) -> i32 {
    (*s).ctx = ctx;
    (*s).size = size;
    (*s).len = 0;
    (*s).is_wide_char = is_wide;
    (*s).error_status = 0;
    (*s).str = js_alloc_string(ctx, size, is_wide);
    if (*s).str.is_null() {
        (*s).size = 0;
        (*s).error_status = -1;
        return -1;
    }
    0
}
#[inline]
unsafe fn string_buffer_init(ctx: *mut JSContext, s: *mut StringBuffer, size: i32) -> i32 {
    string_buffer_init2(ctx, s, size, 0)
}
unsafe fn string_buffer_free(s: *mut StringBuffer) {
    js_free((*s).ctx, (*s).str.cast());
    (*s).str = ptr::null_mut();
}
unsafe fn string_buffer_set_error(s: *mut StringBuffer) -> i32 {
    js_free((*s).ctx, (*s).str.cast());
    (*s).str = ptr::null_mut();
    (*s).size = 0;
    (*s).len = 0;
    (*s).error_status = -1;
    -1
}
#[inline(never)]
unsafe fn string_buffer_widen(s: *mut StringBuffer, mut size: i32) -> i32 {
    if (*s).error_status != 0 {
        return -1;
    }
    let mut slack = 0;
    let str = js_realloc2(
        (*s).ctx,
        (*s).str.cast(),
        size_of::<JSString>().wrapping_add(size.wrapping_shl(1) as usize),
        &mut slack,
    )
    .cast::<JSString>();
    if str.is_null() {
        return string_buffer_set_error(s);
    }
    size = size.wrapping_add((slack >> 1) as i32);
    let str8 = ptr::addr_of_mut!((*str).u).cast::<u8>();
    let str16 = str8.cast::<u16>();
    for i in (0..(*s).len).rev() {
        *str16.offset(i as isize) = *str8.offset(i as isize) as u16;
    }
    (*s).is_wide_char = 1;
    (*s).size = size;
    (*s).str = str;
    0
}
#[inline(never)]
unsafe fn string_buffer_realloc(s: *mut StringBuffer, new_len: i32, c: i32) -> i32 {
    if (*s).error_status != 0 {
        return -1;
    }
    if new_len > JS_STRING_LEN_MAX {
        JS_ThrowInternalError((*s).ctx, c"string too long".as_ptr());
        return string_buffer_set_error(s);
    }
    let mut new_size = new_len
        .max((*s).size.wrapping_mul(3) / 2)
        .min(JS_STRING_LEN_MAX);
    if (*s).is_wide_char == 0 && c >= 0x100 {
        return string_buffer_widen(s, new_size);
    }
    let new_size_bytes = size_of::<JSString>()
        .wrapping_add(new_size.wrapping_shl((*s).is_wide_char as u32) as usize)
        .wrapping_add(1)
        .wrapping_sub((*s).is_wide_char as usize);
    let mut slack = 0;
    let new_str =
        js_realloc2((*s).ctx, (*s).str.cast(), new_size_bytes, &mut slack).cast::<JSString>();
    if new_str.is_null() {
        return string_buffer_set_error(s);
    }
    // C min_int takes int arguments: conversion precedes the comparison.
    new_size = (new_size as usize).wrapping_add(slack >> (*s).is_wide_char) as i32;
    new_size = new_size.min(JS_STRING_LEN_MAX);
    (*s).size = new_size;
    (*s).str = new_str;
    0
}
#[inline(never)]
unsafe fn string_buffer_putc16_slow(s: *mut StringBuffer, c: u32) -> i32 {
    if (*s).len >= (*s).size && string_buffer_realloc(s, (*s).len.wrapping_add(1), c as i32) != 0 {
        return -1;
    }
    if (*s).is_wide_char != 0 {
        *ptr::addr_of_mut!((*(*s).str).u)
            .cast::<u16>()
            .offset((*s).len as isize) = c as u16;
    } else if c < 0x100 {
        *ptr::addr_of_mut!((*(*s).str).u)
            .cast::<u8>()
            .offset((*s).len as isize) = c as u8;
    } else {
        if string_buffer_widen(s, (*s).size) != 0 {
            return -1;
        }
        *ptr::addr_of_mut!((*(*s).str).u)
            .cast::<u16>()
            .offset((*s).len as isize) = c as u16;
    }
    (*s).len = (*s).len.wrapping_add(1);
    0
}
unsafe fn string_buffer_putc8(s: *mut StringBuffer, c: u32) -> i32 {
    if (*s).len >= (*s).size && string_buffer_realloc(s, (*s).len.wrapping_add(1), c as i32) != 0 {
        return -1;
    }
    if (*s).is_wide_char != 0 {
        *ptr::addr_of_mut!((*(*s).str).u)
            .cast::<u16>()
            .offset((*s).len as isize) = c as u16;
    } else {
        *ptr::addr_of_mut!((*(*s).str).u)
            .cast::<u8>()
            .offset((*s).len as isize) = c as u8;
    }
    (*s).len = (*s).len.wrapping_add(1);
    0
}
unsafe fn string_buffer_putc16(s: *mut StringBuffer, c: u32) -> i32 {
    if (*s).len < (*s).size {
        if (*s).is_wide_char != 0 {
            *ptr::addr_of_mut!((*(*s).str).u)
                .cast::<u16>()
                .offset((*s).len as isize) = c as u16;
            (*s).len = (*s).len.wrapping_add(1);
            return 0;
        } else if c < 0x100 {
            *ptr::addr_of_mut!((*(*s).str).u)
                .cast::<u8>()
                .offset((*s).len as isize) = c as u8;
            (*s).len = (*s).len.wrapping_add(1);
            return 0;
        }
    }
    string_buffer_putc16_slow(s, c)
}
unsafe fn string_buffer_putc_slow(s: *mut StringBuffer, mut c: u32) -> i32 {
    if c >= 0x10000 {
        if string_buffer_putc16(s, get_hi_surrogate(c)) != 0 {
            return -1;
        }
        c = get_lo_surrogate(c);
    }
    string_buffer_putc16(s, c)
}
#[inline]
unsafe fn string_buffer_putc(s: *mut StringBuffer, c: u32) -> i32 {
    if (*s).len < (*s).size {
        if (*s).is_wide_char != 0 {
            let dst = ptr::addr_of_mut!((*(*s).str).u).cast::<u16>();
            if c < 0x10000 {
                *dst.offset((*s).len as isize) = c as u16;
                (*s).len = (*s).len.wrapping_add(1);
                return 0;
            } else if (*s).len.wrapping_add(1) < (*s).size {
                *dst.offset((*s).len as isize) = get_hi_surrogate(c) as u16;
                (*s).len = (*s).len.wrapping_add(1);
                *dst.offset((*s).len as isize) = get_lo_surrogate(c) as u16;
                (*s).len = (*s).len.wrapping_add(1);
                return 0;
            }
        } else if c < 0x100 {
            *ptr::addr_of_mut!((*(*s).str).u)
                .cast::<u8>()
                .offset((*s).len as isize) = c as u8;
            (*s).len = (*s).len.wrapping_add(1);
            return 0;
        }
    }
    string_buffer_putc_slow(s, c)
}
unsafe fn string_getc(p: *const JSString, pidx: *mut i32) -> i32 {
    let mut idx = *pidx;
    let mut c;
    if (*p).is_wide_char() != 0 {
        let src = ptr::addr_of!((*p).u).cast::<u16>();
        c = *src.offset(idx as isize) as u32;
        idx = idx.wrapping_add(1);
        if is_hi_surrogate(c) != 0 && (idx as u32) < (*p).len() {
            let c1 = *src.offset(idx as isize) as u32;
            if is_lo_surrogate(c1) != 0 {
                c = from_surrogate(c, c1);
                idx = idx.wrapping_add(1);
            }
        }
    } else {
        c = *ptr::addr_of!((*p).u).cast::<u8>().offset(idx as isize) as u32;
        idx = idx.wrapping_add(1);
    }
    *pidx = idx;
    c as i32
}
unsafe fn string_buffer_write8(s: *mut StringBuffer, p: *const u8, len: i32) -> i32 {
    if (*s).len.wrapping_add(len) > (*s).size
        && string_buffer_realloc(s, (*s).len.wrapping_add(len), 0) != 0
    {
        return -1;
    }
    if (*s).is_wide_char != 0 {
        let dst = ptr::addr_of_mut!((*(*s).str).u).cast::<u16>();
        for i in 0..len {
            *dst.offset((*s).len.wrapping_add(i) as isize) = *p.offset(i as isize) as u16;
        }
    } else {
        ptr::copy_nonoverlapping(
            p,
            ptr::addr_of_mut!((*(*s).str).u)
                .cast::<u8>()
                .offset((*s).len as isize),
            len as usize,
        );
    }
    (*s).len = (*s).len.wrapping_add(len);
    0
}
unsafe fn string_buffer_write16(s: *mut StringBuffer, p: *const u16, len: i32) -> i32 {
    let mut c = 0;
    for i in 0..len {
        c |= *p.offset(i as isize) as i32;
    }
    if (*s).len.wrapping_add(len) > (*s).size {
        if string_buffer_realloc(s, (*s).len.wrapping_add(len), c) != 0 {
            return -1;
        }
    } else if (*s).is_wide_char == 0 && c >= 0x100 && string_buffer_widen(s, (*s).size) != 0 {
        return -1;
    }
    if (*s).is_wide_char != 0 {
        ptr::copy_nonoverlapping(
            p,
            ptr::addr_of_mut!((*(*s).str).u)
                .cast::<u16>()
                .offset((*s).len as isize),
            len as usize,
        );
    } else {
        let dst = ptr::addr_of_mut!((*(*s).str).u).cast::<u8>();
        for i in 0..len {
            *dst.offset((*s).len.wrapping_add(i) as isize) = *p.offset(i as isize) as u8;
        }
    }
    (*s).len = (*s).len.wrapping_add(len);
    0
}
unsafe fn string_buffer_puts8(s: *mut StringBuffer, str: *const c_char) -> i32 {
    string_buffer_write8(
        s,
        str.cast(),
        core::ffi::CStr::from_ptr(str).to_bytes().len() as i32,
    )
}
unsafe fn string_buffer_concat(
    s: *mut StringBuffer,
    p: *const JSString,
    from: u32,
    to: u32,
) -> i32 {
    if to <= from {
        return 0;
    }
    if (*p).is_wide_char() != 0 {
        string_buffer_write16(
            s,
            ptr::addr_of!((*p).u).cast::<u16>().add(from as usize),
            to.wrapping_sub(from) as i32,
        )
    } else {
        string_buffer_write8(
            s,
            ptr::addr_of!((*p).u).cast::<u8>().add(from as usize),
            to.wrapping_sub(from) as i32,
        )
    }
}
unsafe fn string_buffer_fill(s: *mut StringBuffer, c: i32, mut count: i32) -> i32 {
    if (*s).len.wrapping_add(count) > (*s).size
        && string_buffer_realloc(s, (*s).len.wrapping_add(count), c) != 0
    {
        return -1;
    }
    while count > 0 {
        count = count.wrapping_sub(1);
        if string_buffer_putc16(s, c as u32) != 0 {
            return -1;
        }
    }
    0
}
unsafe fn string_buffer_end(s: *mut StringBuffer) -> JSValue {
    let mut str = (*s).str;
    if (*s).error_status != 0 {
        return JS_EXCEPTION;
    }
    if (*s).len == 0 {
        js_free((*s).ctx, str.cast());
        (*s).str = ptr::null_mut();
        return JS_AtomToString((*s).ctx, JS_ATOM_empty_string);
    }
    if (*s).len < (*s).size {
        str = js_realloc_rt(
            (*(*s).ctx).rt,
            str.cast(),
            size_of::<JSString>()
                .wrapping_add((*s).len.wrapping_shl((*s).is_wide_char as u32) as usize)
                .wrapping_add(1)
                .wrapping_sub((*s).is_wide_char as usize),
        )
        .cast();
        if str.is_null() {
            str = (*s).str;
        }
        (*s).str = str;
    }
    if (*s).is_wide_char == 0 {
        *ptr::addr_of_mut!((*str).u)
            .cast::<u8>()
            .offset((*s).len as isize) = 0;
    }
    (*str).set_is_wide_char((*s).is_wide_char as u32);
    (*str).set_len((*s).len as u32);
    (*s).str = ptr::null_mut();
    JS_MKPTR(JS_TAG_STRING, str.cast())
}
pub unsafe fn JS_NewStringLen(ctx: *mut JSContext, buf: *const c_char, buf_len: usize) -> JSValue {
    let p_start = buf.cast::<u8>();
    let p_end = p_start.add(buf_len);
    let len1 = count_ascii(p_start, buf_len);
    let mut p = p_start.add(len1);
    if len1 > JS_STRING_LEN_MAX as usize {
        return JS_ThrowInternalError(ctx, c"string too long".as_ptr());
    }
    if p == p_end {
        return js_new_string8_len(ctx, buf, buf_len as i32);
    }
    let mut b: StringBuffer = core::mem::zeroed();
    if string_buffer_init(ctx, &mut b, buf_len as i32) != 0 {
        string_buffer_free(&mut b);
        return JS_EXCEPTION;
    }
    string_buffer_write8(&mut b, p_start, len1 as i32);
    while p < p_end {
        if *p < 128 {
            string_buffer_putc8(&mut b, *p as u32);
            p = p.add(1);
        } else {
            let mut p_next = ptr::null();
            let mut c =
                crate::cutils::unicode_from_utf8(p, p_end.offset_from(p) as i32, &mut p_next)
                    as u32;
            if c < 0x10000 {
                p = p_next;
            } else if c <= 0x10ffff {
                p = p_next;
                string_buffer_putc16(&mut b, get_hi_surrogate(c));
                c = get_lo_surrogate(c);
            } else {
                c = 0xfffd;
                // Preserve C's invalid-sequence skipping, including continuation runs.
                while p < p_end && *p >= 0x80 && *p < 0xc0 {
                    p = p.add(1);
                }
                if p < p_end {
                    p = p.add(1);
                    while p < p_end && *p >= 0x80 && *p < 0xc0 {
                        p = p.add(1);
                    }
                }
            }
            string_buffer_putc16(&mut b, c);
        }
    }
    string_buffer_end(&mut b)
}
// quickjs.h inline UTF-8 string constructor.
#[inline]
pub unsafe fn JS_NewString(ctx: *mut JSContext, str: *const c_char) -> JSValue {
    JS_NewStringLen(ctx, str, core::ffi::CStr::from_ptr(str).to_bytes().len())
}
