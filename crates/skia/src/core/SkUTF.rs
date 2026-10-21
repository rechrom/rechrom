// Source: src/core/SkUTF.h:11-11
pub type SkUnichar = i32;
// Source: src/core/SkUTF.h:59-59
pub const kMaxBytesInUTF8Sequence: usize = 4;
// Source: src/core/SkUTF.h:91-91
pub fn IsLeadingSurrogateUTF16(c: u16) -> bool {
    c & 0xFC00 == 0xD800
}
// Source: src/core/SkUTF.h:97-97
pub fn IsTrailingSurrogateUTF16(c: u16) -> bool {
    c & 0xFC00 == 0xDC00
}
// Pointer APIs retain native alignment, null handling, and cursor-to-end failure semantics.
// Source: src/core/SkUTF.cpp:8-10
fn left_shift(value: i32, shift: i32) -> i32 {
    ((value as u32) << shift) as i32
}
// Source: src/core/SkUTF.cpp:12-12
fn is_align2(x: usize) -> bool {
    x & 1 == 0
}
// Source: src/core/SkUTF.cpp:14-14
fn is_align4(x: usize) -> bool {
    x & 3 == 0
}
// Source: src/core/SkUTF.cpp:16-16
fn utf16_is_high_surrogate(c: u16) -> bool {
    c & 0xFC00 == 0xD800
}
// Source: src/core/SkUTF.cpp:18-18
fn utf16_is_low_surrogate(c: u16) -> bool {
    c & 0xFC00 == 0xDC00
}
// Source: src/core/SkUTF.cpp:28-36
// Source: src/core/SkUTF.cpp:38-40
fn utf8_byte_type(c: u8) -> i32 {
    if c < 0x80 {
        1
    } else if c < 0xC0 {
        0
    } else if c >= 0xF5 || c & 0xFE == 0xC0 {
        -1
    } else {
        ((((0xe5u32 << 24) >> ((c as u32 >> 4) << 1)) & 3) + 1) as i32
    }
}
// Source: src/core/SkUTF.cpp:41-41
fn utf8_type_is_valid_leading_byte(t: i32) -> bool {
    t > 0
}
// Source: src/core/SkUTF.cpp:43-43
fn utf8_byte_is_continuation(c: u8) -> bool {
    utf8_byte_type(c) == 0
}
// Source: src/core/SkUTF.cpp:47-68
pub unsafe fn CountUTF8(mut p: *const u8, len: usize) -> i32 {
    if p.is_null() && len != 0 {
        return -1;
    }
    let stop = p.wrapping_add(len);
    let mut count = 0i32;
    while p < stop {
        let mut t = utf8_byte_type(*p);
        if !utf8_type_is_valid_leading_byte(t) || p.wrapping_add(t as usize) > stop {
            return -1;
        }
        while t > 1 {
            t -= 1;
            p = p.add(1);
            if !utf8_byte_is_continuation(*p) {
                return -1;
            }
        }
        p = p.add(1);
        count = count.wrapping_add(1);
    }
    count
}
// Source: src/core/SkUTF.cpp:70-94
pub unsafe fn CountUTF16(p: *const u16, len: usize) -> i32 {
    if p.is_null() || !is_align2(p as usize) || !is_align2(len) {
        return -1;
    }
    let mut src = p;
    let stop = src.add(len >> 1);
    let mut count = 0i32;
    while src < stop {
        let mut c = *src;
        src = src.add(1);
        if utf16_is_low_surrogate(c) {
            return -1;
        }
        if utf16_is_high_surrogate(c) {
            if src >= stop {
                return -1;
            }
            c = *src;
            src = src.add(1);
            if !utf16_is_low_surrogate(c) {
                return -1;
            }
        }
        count = count.wrapping_add(1);
    }
    count
}
// Source: src/core/SkUTF.cpp:96-110
pub unsafe fn CountUTF32(p: *const i32, len: usize) -> i32 {
    if !is_align4(p as usize) || !is_align4(len) || (len >> 2) > i32::MAX as usize {
        return -1;
    }
    let mut ptr = p;
    let stop = ptr.wrapping_add(len >> 2);
    while ptr < stop {
        if (*ptr as u32) & 0xFF000000 != 0 {
            return -1;
        }
        ptr = ptr.add(1)
    }
    (len >> 2) as i32
}
// Source: src/core/SkUTF.cpp:112-116
unsafe fn next_fail<T>(ptr: *mut *const T, end: *const T) -> i32 {
    *ptr = end;
    -1
}
// Source: src/core/SkUTF.cpp:118-127
// Source: src/core/SkUTF.cpp:129-139
// Source: src/core/SkUTF.cpp:141-152
pub unsafe fn NextUTF8(ptr: *mut *const u8, end: *const u8) -> i32 {
    if ptr.is_null() || end.is_null() {
        return -1;
    }
    let mut p = *ptr;
    if p.is_null() || p >= end {
        return next_fail(ptr, end);
    }
    let mut c = *p as i32;
    let mut hic = left_shift(c, 24);
    if !utf8_type_is_valid_leading_byte(utf8_byte_type(c as u8)) {
        return next_fail(ptr, end);
    }
    if hic < 0 {
        let mut mask = !0x3Fu32;
        hic = left_shift(hic, 1);
        loop {
            p = p.add(1);
            if p >= end {
                return next_fail(ptr, end);
            }
            let next = *p;
            if !utf8_byte_is_continuation(next) {
                return next_fail(ptr, end);
            }
            c = (c << 6) | (next as i32 & 0x3F);
            mask <<= 5;
            hic = left_shift(hic, 1);
            if hic >= 0 {
                break;
            }
        }
        c &= !(mask as i32);
    }
    *ptr = p.add(1);
    c
}
// Source: src/core/SkUTF.cpp:154-157
pub unsafe fn NextUTF8WithReplacement(ptr: *mut *const u8, end: *const u8) -> i32 {
    let val = NextUTF8(ptr, end);
    if val < 0 {
        0xFFFD
    } else {
        val
    }
}
// Source: src/core/SkUTF.cpp:159-179
// Source: src/core/SkUTF.cpp:191-195
pub unsafe fn NextUTF16(ptr: *mut *const u16, end: *const u16) -> i32 {
    if ptr.is_null() || end.is_null() {
        return -1;
    }
    let mut src = *ptr;
    if src.is_null() || src.wrapping_add(1) > end || !is_align2(src as usize) {
        return next_fail(ptr, end);
    }
    let c = *src;
    src = src.add(1);
    let mut result = c as i32;
    if utf16_is_low_surrogate(c) {
        return next_fail(ptr, end);
    }
    if utf16_is_high_surrogate(c) {
        if src.wrapping_add(1) > end {
            return next_fail(ptr, end);
        }
        let low = *src;
        src = src.add(1);
        if !utf16_is_low_surrogate(low) {
            return next_fail(ptr, end);
        }
        result = (result << 10) + low as i32 - ((0xD800 << 10) + 0xDC00 - 0x10000);
    }
    *ptr = src;
    result
}
// Source: src/core/SkUTF.cpp:197-212
pub unsafe fn NextUTF32(ptr: *mut *const i32, end: *const i32) -> i32 {
    if ptr.is_null() || end.is_null() {
        return -1;
    }
    let s = *ptr;
    if s.is_null() || s.wrapping_add(1) > end || !is_align4(s as usize) {
        return next_fail(ptr, end);
    }
    let value = *s;
    if value as u32 & 0xFF000000 != 0 {
        return next_fail(ptr, end);
    }
    *ptr = s.add(1);
    value
}
// Source: src/core/SkUTF.cpp:214-241
pub unsafe fn ToUTF8(mut uni: i32, utf8: *mut u8) -> usize {
    if uni as u32 > 0x10FFFF {
        return 0;
    }
    if uni <= 127 {
        if !utf8.is_null() {
            *utf8 = uni as u8
        }
        return 1;
    }
    let mut tmp = [0u8; 4];
    let mut p = 0;
    let mut count = 1;
    while uni > 0x7F >> count {
        tmp[p] = (0x80 | (uni & 0x3F)) as u8;
        p += 1;
        uni >>= 6;
        count += 1
    }
    if !utf8.is_null() {
        p = 0;
        let mut dst = count;
        while p < count - 1 {
            dst -= 1;
            *utf8.add(dst) = tmp[p];
            p += 1
        }
        dst -= 1;
        *utf8.add(dst) = (!(0xFFi32 >> count) | uni) as u8
    }
    count
}
// Source: src/core/SkUTF.cpp:243-257
pub unsafe fn ToUTF16(uni: i32, utf16: *mut u16) -> usize {
    if uni as u32 > 0x10FFFF {
        return 0;
    }
    let extra = usize::from(uni > 0xFFFF);
    if !utf16.is_null() {
        if extra != 0 {
            *utf16 = ((0xD800 - 64) + (uni >> 10)) as u16;
            *utf16.add(1) = (0xDC00 | (uni & 0x3FF)) as u16
        } else {
            *utf16 = uni as u16
        }
    }
    1 + extra
}
// Source: src/core/SkUTF.cpp:259-262
// Source: src/core/SkUTF.cpp:264-271
// Source: src/core/SkUTF.cpp:273-278
// Source: src/core/SkUTF.cpp:280-289
pub unsafe fn UTF8ToUTF16(
    mut dst: *mut u16,
    mut capacity: i32,
    mut src: *const u8,
    len: usize,
) -> i32 {
    if dst.is_null() {
        capacity = 0
    }
    let mut length = 0i32;
    let end_dst = dst.wrapping_offset(capacity as isize);
    let end_src = src.wrapping_add(len);
    while src < end_src {
        let uni = NextUTF8(&mut src, end_src);
        if uni < 0 {
            return -1;
        }
        let mut buf = [0u16; 2];
        let mut count = ToUTF16(uni, buf.as_mut_ptr());
        if count == 0 {
            return -1;
        }
        length = length.wrapping_add(count as i32);
        if !dst.is_null() {
            let mut elems = buf.as_ptr();
            while dst < end_dst && count > 0 {
                *dst = *elems;
                dst = dst.add(1);
                elems = elems.add(1);
                count -= 1
            }
        }
    }
    length
}
// Source: src/core/SkUTF.cpp:291-294
// Source: src/core/SkUTF.cpp:296-303
// Source: src/core/SkUTF.cpp:305-310
// Source: src/core/SkUTF.cpp:312-321
pub unsafe fn UTF16ToUTF8(
    mut dst: *mut u8,
    mut capacity: i32,
    mut src: *const u16,
    len: usize,
) -> i32 {
    if dst.is_null() {
        capacity = 0
    }
    let mut length = 0i32;
    let end_dst = dst.wrapping_offset(capacity as isize);
    let end_src = src.wrapping_add(len);
    while src < end_src {
        let uni = NextUTF16(&mut src, end_src);
        if uni < 0 {
            return -1;
        }
        let mut buf = [0u8; 4];
        let mut count = ToUTF8(uni, buf.as_mut_ptr());
        if count == 0 {
            return -1;
        }
        length = length.wrapping_add(count as i32);
        if !dst.is_null() {
            let mut elems = buf.as_ptr();
            while dst < end_dst && count > 0 {
                *dst = *elems;
                dst = dst.add(1);
                elems = elems.add(1);
                count -= 1
            }
        }
    }
    length
}

// Source: src/core/SkUTF.h:66-66
/// Native omitted output argument computes the encoded size only.
pub unsafe fn ToUTF8_default(uni: SkUnichar) -> usize {
    unsafe { ToUTF8(uni, core::ptr::null_mut()) }
}
// Source: src/core/SkUTF.h:73-73
pub unsafe fn ToUTF16_default(uni: SkUnichar) -> usize {
    unsafe { ToUTF16(uni, core::ptr::null_mut()) }
}
