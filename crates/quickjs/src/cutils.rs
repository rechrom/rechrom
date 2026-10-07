//! Direct translation of cutils.c. Copyright 2017 Fabrice Bellard,
//! 2018 Charlie Gordon. MIT; see ../LICENSE.
pub use super::cutils_header::*;
use core::{
    ffi::{c_char, c_void},
    ptr,
};
use std::alloc::{alloc, dealloc, realloc, Layout};

unsafe fn strlen(mut p: *const c_char) -> usize {
    let start = p;
    while *p != 0 {
        p = p.add(1);
    }
    p.offset_from(start) as usize
}
pub unsafe fn pstrcpy(buf: *mut c_char, buf_size: i32, mut str: *const c_char) {
    let mut q = buf;
    if buf_size <= 0 {
        return;
    }
    loop {
        let c = *str;
        str = str.add(1);
        if c == 0 || q >= buf.add(buf_size as usize - 1) {
            break;
        }
        *q = c;
        q = q.add(1);
    }
    *q = 0;
}
pub unsafe fn pstrcat(buf: *mut c_char, buf_size: i32, s: *const c_char) -> *mut c_char {
    let len = strlen(buf) as i32;
    if len < buf_size {
        pstrcpy(buf.add(len as usize), buf_size - len, s);
    }
    buf
}
pub unsafe fn strstart(str: *const c_char, val: *const c_char, out: *mut *const c_char) -> i32 {
    let (mut p, mut q) = (str, val);
    while *q != 0 {
        if *p != *q {
            return 0;
        }
        p = p.add(1);
        q = q.add(1);
    }
    if !out.is_null() {
        *out = p;
    }
    1
}
pub unsafe fn has_suffix(str: *const c_char, suffix: *const c_char) -> i32 {
    let (len, slen) = (strlen(str), strlen(suffix));
    if len < slen {
        return 0;
    }
    for i in 0..slen {
        if *str.add(len - slen + i) != *suffix.add(i) {
            return 0;
        }
    }
    1
}

// C realloc needs no old-size argument; Rust's allocator does. Keep this
// bookkeeping private to the default allocator. Custom callbacks receive the
// original pointer/opaque/size contract and do not use this prefix.
const ALLOC_HEADER: usize = 16;
pub unsafe fn dbuf_default_realloc(
    _opaque: *mut c_void,
    p: *mut c_void,
    size: usize,
) -> *mut c_void {
    if size == 0 {
        if !p.is_null() {
            let base = p.cast::<u8>().sub(ALLOC_HEADER);
            let old_size = ptr::read(base.cast::<usize>());
            dealloc(base, Layout::from_size_align_unchecked(old_size, 16));
        }
        return ptr::null_mut();
    }
    let Some(total) = size.checked_add(ALLOC_HEADER) else {
        return ptr::null_mut();
    };
    let Ok(layout) = Layout::from_size_align(total, 16) else {
        return ptr::null_mut();
    };
    let base = if p.is_null() {
        alloc(layout)
    } else {
        let old = p.cast::<u8>().sub(ALLOC_HEADER);
        let old_size = ptr::read(old.cast::<usize>());
        realloc(old, Layout::from_size_align_unchecked(old_size, 16), total)
    };
    if base.is_null() {
        return ptr::null_mut();
    }
    ptr::write(base.cast::<usize>(), total);
    base.add(ALLOC_HEADER).cast()
}
pub unsafe fn dbuf_init2(
    s: *mut DynBuf,
    opaque: *mut c_void,
    realloc_func: Option<DynBufReallocFunc>,
) {
    ptr::write(s, DynBuf::default());
    (*s).opaque = opaque;
    (*s).realloc_func = Some(realloc_func.unwrap_or(dbuf_default_realloc));
}
pub unsafe fn dbuf_init(s: *mut DynBuf) {
    dbuf_init2(s, ptr::null_mut(), None);
}
pub unsafe fn dbuf_claim(s: *mut DynBuf, len: usize) -> i32 {
    let mut new_size = (*s).size.wrapping_add(len);
    if new_size < len {
        return -1;
    }
    if new_size > (*s).allocated_size {
        if (*s).error != 0 {
            return -1;
        }
        let size = (*s).allocated_size.wrapping_add((*s).allocated_size / 2);
        if size < (*s).allocated_size {
            return -1;
        }
        if size > new_size {
            new_size = size;
        }
        let new_buf =
            ((*s).realloc_func.unwrap())((*s).opaque, (*s).buf.cast(), new_size).cast::<u8>();
        if new_buf.is_null() {
            (*s).error = TRUE;
            return -1;
        }
        (*s).buf = new_buf;
        (*s).allocated_size = new_size;
    }
    0
}
pub unsafe fn dbuf_put(s: *mut DynBuf, data: *const u8, len: usize) -> i32 {
    if (*s).allocated_size.wrapping_sub((*s).size) < len && dbuf_claim(s, len) != 0 {
        return -1;
    }
    memcpy_no_ub((*s).buf.wrapping_add((*s).size).cast(), data.cast(), len);
    (*s).size = (*s).size.wrapping_add(len);
    0
}
pub unsafe fn dbuf_put_self(s: *mut DynBuf, offset: usize, len: usize) -> i32 {
    if (*s).allocated_size.wrapping_sub((*s).size) < len && dbuf_claim(s, len) != 0 {
        return -1;
    }
    memcpy_no_ub(
        (*s).buf.wrapping_add((*s).size).cast(),
        (*s).buf.wrapping_add(offset).cast(),
        len,
    );
    (*s).size = (*s).size.wrapping_add(len);
    0
}
pub unsafe fn __dbuf_putc(s: *mut DynBuf, c: u8) -> i32 {
    dbuf_put(s, &c, 1)
}
pub unsafe fn __dbuf_put_u16(s: *mut DynBuf, val: u16) -> i32 {
    dbuf_put(s, (&val as *const u16).cast(), 2)
}
pub unsafe fn __dbuf_put_u32(s: *mut DynBuf, val: u32) -> i32 {
    dbuf_put(s, (&val as *const u32).cast(), 4)
}
pub unsafe fn __dbuf_put_u64(s: *mut DynBuf, val: u64) -> i32 {
    dbuf_put(s, (&val as *const u64).cast(), 8)
}
pub unsafe fn dbuf_putstr(s: *mut DynBuf, str: *const c_char) -> i32 {
    dbuf_put(s, str.cast(), strlen(str))
}

/// Rust cannot define a stable C-variadic function. The formatter is passed
/// explicitly and follows vsnprintf's contract: NUL-terminate when capacity
/// permits, return the full byte length excluding NUL, or a negative error.
/// It may be called twice, exactly as the original va_start/vsnprintf path.
/// This preserves allocation, truncation, error and size semantics while the
/// future engine call sites supply typed Rust formatting rather than C varargs.
pub unsafe fn dbuf_printf<F>(s: *mut DynBuf, mut format: F) -> i32
where
    F: FnMut(*mut u8, usize) -> i32,
{
    let mut buf = [0u8; 128];
    let len = format(buf.as_mut_ptr(), buf.len());
    if len < 0 {
        return -1;
    }
    if (len as usize) < buf.len() {
        return dbuf_put(s, buf.as_ptr(), len as usize);
    }
    if dbuf_claim(s, len as usize + 1) != 0 {
        return -1;
    }
    format((*s).buf.add((*s).size), (*s).allocated_size - (*s).size);
    (*s).size += len as usize;
    0
}
/// Typed formatting adapter. Byte lengths and NUL behavior mirror snprintf;
/// formatting choices (C % specifiers) are translated at each engine call site.
pub unsafe fn dbuf_printf_args(s: *mut DynBuf, args: core::fmt::Arguments<'_>) -> i32 {
    struct Writer {
        p: *mut u8,
        cap: usize,
        len: usize,
    }
    impl core::fmt::Write for Writer {
        fn write_str(&mut self, text: &str) -> core::fmt::Result {
            let copy = text
                .len()
                .min(self.cap.saturating_sub(1).saturating_sub(self.len));
            unsafe {
                if copy != 0 {
                    ptr::copy_nonoverlapping(text.as_ptr(), self.p.add(self.len), copy);
                }
            }
            self.len = self.len.checked_add(text.len()).ok_or(core::fmt::Error)?;
            Ok(())
        }
    }
    dbuf_printf(s, |p, cap| {
        let mut w = Writer { p, cap, len: 0 };
        if core::fmt::write(&mut w, args).is_err() || w.len > i32::MAX as usize {
            return -1;
        }
        if cap != 0 {
            *p.add(w.len.min(cap - 1)) = 0;
        }
        w.len as i32
    })
}
pub unsafe fn dbuf_free(s: *mut DynBuf) {
    if !(*s).buf.is_null() {
        ((*s).realloc_func.unwrap())((*s).opaque, (*s).buf.cast(), 0);
    }
    ptr::write(s, DynBuf::default());
}

pub unsafe fn unicode_to_utf8(buf: *mut u8, c: u32) -> i32 {
    let mut q = buf;
    if c < 0x80 {
        *q = c as u8;
        q = q.add(1);
    } else {
        if c < 0x800 {
            *q = ((c >> 6) | 0xc0) as u8;
            q = q.add(1);
        } else {
            if c < 0x10000 {
                *q = ((c >> 12) | 0xe0) as u8;
                q = q.add(1);
            } else {
                if c < 0x00200000 {
                    *q = ((c >> 18) | 0xf0) as u8;
                    q = q.add(1);
                } else {
                    if c < 0x04000000 {
                        *q = ((c >> 24) | 0xf8) as u8;
                        q = q.add(1);
                    } else if c < 0x80000000 {
                        *q = ((c >> 30) | 0xfc) as u8;
                        q = q.add(1);
                        *q = (((c >> 24) & 0x3f) | 0x80) as u8;
                        q = q.add(1);
                    } else {
                        return 0;
                    }
                    *q = (((c >> 18) & 0x3f) | 0x80) as u8;
                    q = q.add(1);
                }
                *q = (((c >> 12) & 0x3f) | 0x80) as u8;
                q = q.add(1);
            }
            *q = (((c >> 6) & 0x3f) | 0x80) as u8;
            q = q.add(1);
        }
        *q = ((c & 0x3f) | 0x80) as u8;
        q = q.add(1);
    }
    q.offset_from(buf) as i32
}
static utf8_min_code: [u32; 5] = [0x80, 0x800, 0x10000, 0x00200000, 0x04000000];
static utf8_first_code_mask: [u8; 5] = [0x1f, 0xf, 0x7, 0x3, 0x1];
pub unsafe fn unicode_from_utf8(mut p: *const u8, max_len: i32, pp: *mut *const u8) -> i32 {
    let mut c = *p as u32;
    p = p.add(1);
    if c < 0x80 {
        *pp = p;
        return c as i32;
    }
    let l: usize = match c {
        0xc0..=0xdf => 1,
        0xe0..=0xef => 2,
        0xf0..=0xf7 => 3,
        0xf8..=0xfb => 4,
        0xfc..=0xfd => 5,
        _ => return -1,
    };
    if l as i32 > max_len - 1 {
        return -1;
    }
    c &= utf8_first_code_mask[l - 1] as u32;
    for _ in 0..l {
        let b = *p as u32;
        p = p.add(1);
        if b < 0x80 || b >= 0xc0 {
            return -1;
        }
        c = (c << 6) | (b & 0x3f);
    }
    if c < utf8_min_code[l - 1] {
        return -1;
    }
    *pp = p;
    c as i32
}

type exchange_f = unsafe fn(*mut u8, *mut u8, usize);
pub type cmp_f = unsafe fn(*const c_void, *const c_void, *mut c_void) -> i32;
unsafe fn exchange_bytes(mut ap: *mut u8, mut bp: *mut u8, size: usize) {
    for _ in 0..size {
        let t = *ap;
        *ap = *bp;
        *bp = t;
        ap = ap.add(1);
        bp = bp.add(1);
    }
}
unsafe fn exchange_one_byte(ap: *mut u8, bp: *mut u8, _size: usize) {
    let t = *ap;
    *ap = *bp;
    *bp = t;
}
macro_rules! exchange_words {
    ($many:ident, $one:ident, $ty:ty) => {
        unsafe fn $many(a: *mut u8, b: *mut u8, size: usize) {
            let (mut ap, mut bp) = (a.cast::<$ty>(), b.cast::<$ty>());
            for _ in 0..size / core::mem::size_of::<$ty>() {
                let t = *ap;
                *ap = *bp;
                *bp = t;
                ap = ap.add(1);
                bp = bp.add(1);
            }
        }
        unsafe fn $one(a: *mut u8, b: *mut u8, _size: usize) {
            let (ap, bp) = (a.cast::<$ty>(), b.cast::<$ty>());
            let t = *ap;
            *ap = *bp;
            *bp = t;
        }
    };
}
exchange_words!(exchange_int16s, exchange_one_int16, u16);
exchange_words!(exchange_int32s, exchange_one_int32, u32);
exchange_words!(exchange_int64s, exchange_one_int64, u64);
unsafe fn exchange_int128s(a: *mut u8, b: *mut u8, size: usize) {
    let (mut ap, mut bp) = (a.cast::<u64>(), b.cast::<u64>());
    for _ in 0..size / 16 {
        let (t, u) = (*ap, *ap.add(1));
        *ap = *bp;
        *ap.add(1) = *bp.add(1);
        *bp = t;
        *bp.add(1) = u;
        ap = ap.add(2);
        bp = bp.add(2);
    }
}
unsafe fn exchange_one_int128(a: *mut u8, b: *mut u8, _size: usize) {
    let (ap, bp) = (a.cast::<u64>(), b.cast::<u64>());
    let (t, u) = (*ap, *ap.add(1));
    *ap = *bp;
    *ap.add(1) = *bp.add(1);
    *bp = t;
    *bp.add(1) = u;
}
fn exchange_func(base: *const u8, size: usize) -> exchange_f {
    match (base as usize | size) & 15 {
        0 => {
            if size == 16 {
                exchange_one_int128
            } else {
                exchange_int128s
            }
        }
        8 => {
            if size == 8 {
                exchange_one_int64
            } else {
                exchange_int64s
            }
        }
        4 | 12 => {
            if size == 4 {
                exchange_one_int32
            } else {
                exchange_int32s
            }
        }
        2 | 6 | 10 | 14 => {
            if size == 2 {
                exchange_one_int16
            } else {
                exchange_int16s
            }
        }
        _ => {
            if size == 1 {
                exchange_one_byte
            } else {
                exchange_bytes
            }
        }
    }
}
unsafe fn heapsortx(basep: *mut u8, nmemb: usize, size: usize, cmp: cmp_f, opaque: *mut c_void) {
    let swap = exchange_func(basep, size);
    if nmemb > 1 {
        let mut i = (nmemb / 2) * size;
        let n = nmemb * size;
        while i > 0 {
            i -= size;
            let mut r = i;
            loop {
                let mut c = r * 2 + size;
                if c >= n {
                    break;
                }
                if c < n - size && cmp(basep.add(c).cast(), basep.add(c + size).cast(), opaque) <= 0
                {
                    c += size;
                }
                if cmp(basep.add(r).cast(), basep.add(c).cast(), opaque) > 0 {
                    break;
                }
                swap(basep.add(r), basep.add(c), size);
                r = c;
            }
        }
        i = n - size;
        while i > 0 {
            swap(basep, basep.add(i), size);
            let mut r = 0;
            loop {
                let mut c = r * 2 + size;
                if c >= i {
                    break;
                }
                if c < i - size && cmp(basep.add(c).cast(), basep.add(c + size).cast(), opaque) <= 0
                {
                    c += size;
                }
                if cmp(basep.add(r).cast(), basep.add(c).cast(), opaque) > 0 {
                    break;
                }
                swap(basep.add(r), basep.add(c), size);
                r = c;
            }
            i -= size;
        }
    }
}
unsafe fn med3(a: *mut u8, b: *mut u8, c: *mut u8, cmp: cmp_f, opaque: *mut c_void) -> *mut u8 {
    if cmp(a.cast(), b.cast(), opaque) < 0 {
        if cmp(b.cast(), c.cast(), opaque) < 0 {
            b
        } else if cmp(a.cast(), c.cast(), opaque) < 0 {
            c
        } else {
            a
        }
    } else if cmp(b.cast(), c.cast(), opaque) > 0 {
        b
    } else if cmp(a.cast(), c.cast(), opaque) < 0 {
        a
    } else {
        c
    }
}
pub unsafe fn rqsort(
    base: *mut c_void,
    mut nmemb: usize,
    size: usize,
    cmp: cmp_f,
    opaque: *mut c_void,
) {
    #[derive(Clone, Copy)]
    struct Frame {
        base: *mut u8,
        count: usize,
        depth: i32,
    }
    let mut stack = [Frame {
        base: ptr::null_mut(),
        count: 0,
        depth: 0,
    }; 50];
    let mut sp = 0;
    let swap = exchange_func(base.cast(), size);
    let swap_block = exchange_func(base.cast(), size | 128);
    if nmemb < 2 || size == 0 {
        return;
    }
    stack[sp] = Frame {
        base: base.cast(),
        count: nmemb,
        depth: 0,
    };
    sp += 1;
    while sp > 0 {
        sp -= 1;
        let mut ptr = stack[sp].base;
        nmemb = stack[sp].count;
        let mut depth = stack[sp].depth;
        while nmemb > 6 {
            depth += 1;
            if depth > 50 {
                heapsortx(ptr, nmemb, size, cmp, opaque);
                nmemb = 0;
                break;
            }
            let m4 = (nmemb >> 2) * size;
            let m = med3(ptr.add(m4), ptr.add(2 * m4), ptr.add(3 * m4), cmp, opaque);
            swap(ptr, m, size);
            let (mut i, mut lt) = (1usize, 1usize);
            let (mut pi, mut plt) = (ptr.add(size), ptr.add(size));
            let mut gt = nmemb;
            let top = ptr.add(nmemb * size);
            let (mut pj, mut pgt) = (top, top);
            loop {
                while pi < pj {
                    let c = cmp(ptr.cast(), pi.cast(), opaque);
                    if c < 0 {
                        break;
                    }
                    if c == 0 {
                        swap(plt, pi, size);
                        lt += 1;
                        plt = plt.add(size);
                    }
                    i += 1;
                    pi = pi.add(size);
                }
                loop {
                    pj = pj.sub(size);
                    if pi >= pj {
                        break;
                    }
                    let c = cmp(ptr.cast(), pj.cast(), opaque);
                    if c > 0 {
                        break;
                    }
                    if c == 0 {
                        gt -= 1;
                        pgt = pgt.sub(size);
                        swap(pgt, pj, size);
                    }
                }
                if pi >= pj {
                    break;
                }
                swap(pi, pj, size);
                i += 1;
                pi = pi.add(size);
            }
            let mut span = plt.offset_from(ptr) as usize;
            let mut span2 = pi.offset_from(plt) as usize;
            lt = i - lt;
            if span > span2 {
                span = span2;
            }
            swap_block(ptr, pi.sub(span), span);
            span = top.offset_from(pgt) as usize;
            span2 = pgt.offset_from(pi) as usize;
            pgt = top.sub(span2);
            gt = nmemb - (gt - i);
            if span > span2 {
                span = span2;
            }
            swap_block(pi, top.sub(span), span);
            if lt > nmemb - gt {
                stack[sp] = Frame {
                    base: ptr,
                    count: lt,
                    depth,
                };
                sp += 1;
                ptr = pgt;
                nmemb -= gt;
            } else {
                stack[sp] = Frame {
                    base: pgt,
                    count: nmemb - gt,
                    depth,
                };
                sp += 1;
                nmemb = lt;
            }
        }
        // wrapping_add handles the empty one-past-end fragment without forming
        // an out-of-allocation pointer; it is never dereferenced in that case.
        let mut pi = ptr.wrapping_add(size);
        let top = ptr.add(nmemb * size);
        while pi < top {
            let mut pj = pi;
            while pj > ptr && cmp(pj.sub(size).cast(), pj.cast(), opaque) > 0 {
                swap(pj, pj.sub(size), size);
                pj = pj.sub(size);
            }
            pi = pi.add(size);
        }
    }
}

#[cfg(test)]
#[path = "../tests/cutils_differential.rs"]
mod tests;
