// Copyright 2008 Google Inc. BSD-style license.
// SkFixed and SkFixed3232 are represented by their native signed i32 and i64 storage.
use super::SkBitmapProcState::{
    pack_two_shorts, MatrixProc, SkBitmapProcState, SkBitmapProcStateAutoMapper,
};
use crate::include::private::SkFixed::{SkFixed3232ToFixed, SkFixed3232ToInt};
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:34-37
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:42-44
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:52-54
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:57-57
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:59-60
pub(crate) fn can_truncate_to_fixed_for_decal(fx: i32, dx: i32, count: i32, max: u32) -> bool {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),37; @condition b"count > 0\0".as_ptr(); count>0);
    if dx <= (1 << 16) / 256 {
        return false;
    }
    if ((fx >> 16) as u32) >= max {
        return false;
    }
    let last_fx = (fx as i64 + dx as i64 * (count - 1) as i64) as u64;
    last_fx <= i32::MAX as u64 && (((last_fx as i32) >> 16) as u32) < max
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:67-67
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:70-74
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:76-81
// A u16 slice replaces the packed uint32_t buffer view; paired writes retain native memory order.
pub(crate) fn decal_nofilter_scale(dst: &mut [u16], mut fx: i32, dx: i32, mut count: i32) {
    let mut offset = 0;
    while count > 2 {
        dst[offset] = (fx >> 16) as u16;
        dst[offset + 1] = (fx.wrapping_add(dx) >> 16) as u16;
        offset += 2;
        fx = fx.wrapping_add(dx.wrapping_add(dx));
        count -= 2;
    }
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),76; @condition b"count <= 2\0".as_ptr(); count<=2);
    match count {
        2 => {
            dst[offset + 1] = (fx.wrapping_add(dx) >> 16) as u16;
            dst[offset] = (fx >> 16) as u16;
        }
        1 => {
            dst[offset] = (fx >> 16) as u16;
        }
        _ => {}
    }
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:83-83
pub(crate) type TileProc = fn(i32, i32) -> u32;
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:86-89
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:92-97
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:99-100
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:102-104
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:106-106
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:108-110
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:112-116
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:119-123
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:125-130
unsafe fn nofilter_scale<const TRY_DECAL: bool>(
    s: &SkBitmapProcState<'_>,
    xy: &mut [u32],
    mut count: i32,
    x: i32,
    y: i32,
    tilex: TileProc,
    tiley: TileProc,
) {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),89; @condition b"s.fInvMatrix.isScaleTranslate()\0".as_ptr(); s.fInvMatrix.isScaleTranslate());
    let mut fx;
    {
        let mapper = SkBitmapProcStateAutoMapper::new(s, x, y, None);
        xy[0] = tiley(mapper.fixedY(), s.fPixmap.height() - 1);
        fx = mapper.fixed3232X();
    }
    let maxX = (s.fPixmap.width() - 1) as u32;
    if maxX == 0 {
        std::ptr::write_bytes(xy.as_mut_ptr().add(1).cast::<u16>(), 0, count as usize);
        return;
    }
    let dx = s.fInvSx.assume_init();
    if TRY_DECAL {
        let fixedFx = SkFixed3232ToFixed(fx);
        let fixedDx = SkFixed3232ToFixed(dx);
        if can_truncate_to_fixed_for_decal(fixedFx, fixedDx, count, maxX) {
            let xs = std::slice::from_raw_parts_mut(
                xy.as_mut_ptr().add(1).cast::<u16>(),
                count as usize,
            );
            decal_nofilter_scale(xs, fixedFx, fixedDx, count);
            return;
        }
    }
    let mut out = 1usize;
    while count >= 2 {
        xy[out] = pack_two_shorts(
            tilex(SkFixed3232ToFixed(fx), maxX as i32),
            tilex(SkFixed3232ToFixed(fx.wrapping_add(dx)), maxX as i32),
        );
        out += 1;
        fx = fx.wrapping_add(dx.wrapping_add(dx));
        count -= 2;
    }
    let mut xx = xy.as_mut_ptr().add(out).cast::<u16>();
    while count > 0 {
        count -= 1;
        *xx = tilex(SkFixed3232ToFixed(fx), maxX as i32) as u16;
        xx = xx.add(1);
        fx = fx.wrapping_add(dx);
    }
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:132-135
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:137-137
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:139-144
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:146-152
unsafe fn nofilter_affine(
    s: &SkBitmapProcState<'_>,
    xy: &mut [u32],
    count: i32,
    x: i32,
    y: i32,
    tilex: TileProc,
    tiley: TileProc,
) {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),135; @condition b"!s.fInvMatrix.hasPerspective()\0".as_ptr(); !s.fInvMatrix.hasPerspective());
    let mapper = SkBitmapProcStateAutoMapper::new(s, x, y, None);
    let mut fx = mapper.fixed3232X();
    let mut fy = mapper.fixed3232Y();
    let dx = s.fInvSx.assume_init();
    let dy = s.fInvKy.assume_init();
    let maxX = s.fPixmap.width() - 1;
    let maxY = s.fPixmap.height() - 1;
    for i in 0..count as usize {
        xy[i] = (tiley(SkFixed3232ToFixed(fy), maxY) << 16) | tilex(SkFixed3232ToFixed(fx), maxX);
        fx = fx.wrapping_add(dx);
        fy = fy.wrapping_add(dy);
    }
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:156-156
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:159-160
pub(crate) fn extract_low_bits_clamp_clamp(fx: i32, _max: i32) -> u32 {
    ((fx >> 12) & 15) as u32
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:163-163
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:166-167
pub(crate) fn extract_low_bits_general(fx: i32, max: i32) -> u32 {
    extract_low_bits_clamp_clamp((fx & 65535).wrapping_mul(max.wrapping_add(1)), max)
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:183-190
// Function parameters substitute the C++ compile-time function-pointer arguments.
pub(crate) fn pack(f: i32, max: u32, one: i32, tile: TileProc, extract_low_bits: TileProc) -> u32 {
    let mut packed = tile(f, max as i32);
    packed = (packed << 4) | extract_low_bits(f, max as i32);
    packed = (packed << 14) | tile(f.wrapping_add(one), max as i32);
    packed
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:192-195
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:197-201
const MAX_HI: i64 = i16::MAX as i64;
const MIN_HI: i64 = i16::MIN as i64;
const MAX_FX: i64 = (i16::MAX as i64) << 32;
const MIN_FX: i64 = (i16::MIN as i64) << 32;
pub(crate) const fn sk_fixed3232_saturate2fixed(mut x: i64) -> i32 {
    x = if (x >> 32) < MAX_HI { x } else { MAX_FX };
    x = if (x >> 32) > MIN_HI { x } else { MIN_FX };
    (x >> 16) as i32
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:203-206
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:208-213
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:215-216
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:218-219
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:223-233
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:235-240
unsafe fn filter_scale<const TRY_DECAL: bool>(
    s: &SkBitmapProcState<'_>,
    xy: &mut [u32],
    count: i32,
    x: i32,
    y: i32,
    tilex: TileProc,
    tiley: TileProc,
    extract: TileProc,
) {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),206; @condition b"s.fInvMatrix.isScaleTranslate()\0".as_ptr(); s.fInvMatrix.isScaleTranslate());
    let maxX = (s.fPixmap.width() - 1) as u32;
    let dx = s.fInvSx.assume_init();
    let mut fx;
    {
        let mapper = SkBitmapProcStateAutoMapper::new(s, x, y, None);
        let maxY = (s.fPixmap.height() - 1) as u32;
        xy[0] = pack(
            sk_fixed3232_saturate2fixed(mapper.fixed3232Y()),
            maxY,
            s.fFilterOneY.assume_init(),
            tiley,
            extract,
        );
        fx = mapper.fixed3232X();
    }
    if TRY_DECAL
        && (SkFixed3232ToInt(fx) as u32) < maxX
        && (SkFixed3232ToInt(fx.wrapping_add(dx.wrapping_mul((count - 1) as i64))) as u32) < maxX
    {
        for i in 0..count as usize {
            let fixedFx = sk_fixed3232_saturate2fixed(fx);
            crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),228; @condition b"(fixedFx >> (16 + 14)) == 0\0".as_ptr(); (fixedFx>>(16+14))==0);
            xy[1 + i] = ((fixedFx >> 12 << 14) | ((fixedFx >> 16) + 1)) as u32;
            fx = fx.wrapping_add(dx);
        }
        return;
    }
    for i in 0..count as usize {
        xy[1 + i] = pack(
            sk_fixed3232_saturate2fixed(fx),
            maxX,
            s.fFilterOneX.assume_init(),
            tilex,
            extract,
        );
        fx = fx.wrapping_add(dx);
    }
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:242-245
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:247-247
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:249-250
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:252-260
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:262-265
unsafe fn filter_affine(
    s: &SkBitmapProcState<'_>,
    xy: &mut [u32],
    count: i32,
    x: i32,
    y: i32,
    tilex: TileProc,
    tiley: TileProc,
    extract: TileProc,
) {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),245; @condition b"!s.fInvMatrix.hasPerspective()\0".as_ptr(); !s.fInvMatrix.hasPerspective());
    let mapper = SkBitmapProcStateAutoMapper::new(s, x, y, None);
    let oneX = s.fFilterOneX.assume_init();
    let oneY = s.fFilterOneY.assume_init();
    let mut fx = mapper.fixed3232X();
    let mut fy = mapper.fixed3232Y();
    let dx = s.fInvSx.assume_init();
    let dy = s.fInvKy.assume_init();
    let maxX = (s.fPixmap.width() - 1) as u32;
    let maxY = (s.fPixmap.height() - 1) as u32;
    for i in 0..count as usize {
        xy[2 * i] = pack(SkFixed3232ToFixed(fy), maxY, oneY, tiley, extract);
        xy[2 * i + 1] = pack(SkFixed3232ToFixed(fx), maxX, oneX, tilex, extract);
        fy = fy.wrapping_add(dy);
        fx = fx.wrapping_add(dx);
    }
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:269-271
pub(crate) fn SK_USHIFT16(x: u32) -> u32 {
    x >> 16
}

// Source: src/core/SkBitmapProcState_matrixProcs.cpp:273-278
pub(crate) fn repeat(fx: i32, max: i32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),274; @condition b"max < 65535\0".as_ptr(); max<65535);
    SK_USHIFT16(((fx & 65535) as u32).wrapping_mul((max + 1) as u32))
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:280-280
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:283-284
pub(crate) fn mirror(fx: i32, max: i32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),278; @condition b"max < 65535\0".as_ptr(); max<65535);
    let s = fx.wrapping_shl(15) >> 31;
    SK_USHIFT16(((fx ^ s) & 65535).wrapping_mul(max + 1) as u32)
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:286-288
pub(crate) fn clamp(fx: i32, max: i32) -> u32 {
    (fx >> 16).max(0).min(max) as u32
}
// Uncredited monomorphization glue for native compile-time TileProc arguments.
macro_rules! matrix_variants {
    ($n:ident,$f:ident,$na:ident,$fa:ident,$tile:ident,$extract:ident,$decal:literal) => {
        unsafe fn $n(s: &SkBitmapProcState<'_>, xy: &mut [u32], n: i32, x: i32, y: i32) {
            nofilter_scale::<$decal>(s, xy, n, x, y, $tile, $tile);
        }
        unsafe fn $f(s: &SkBitmapProcState<'_>, xy: &mut [u32], n: i32, x: i32, y: i32) {
            filter_scale::<$decal>(s, xy, n, x, y, $tile, $tile, $extract);
        }
        unsafe fn $na(s: &SkBitmapProcState<'_>, xy: &mut [u32], n: i32, x: i32, y: i32) {
            nofilter_affine(s, xy, n, x, y, $tile, $tile);
        }
        unsafe fn $fa(s: &SkBitmapProcState<'_>, xy: &mut [u32], n: i32, x: i32, y: i32) {
            filter_affine(s, xy, n, x, y, $tile, $tile, $extract);
        }
    };
}
matrix_variants!(
    clamp_ns,
    clamp_fs,
    clamp_na,
    clamp_fa,
    clamp,
    extract_low_bits_clamp_clamp,
    true
);
matrix_variants!(
    repeat_ns,
    repeat_fs,
    repeat_na,
    repeat_fa,
    repeat,
    extract_low_bits_general,
    false
);
matrix_variants!(
    mirror_ns,
    mirror_fs,
    mirror_na,
    mirror_fa,
    mirror,
    extract_low_bits_general,
    false
);
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:290-301
const ClampX_ClampY_Procs: [MatrixProc; 4] = [clamp_ns, clamp_fs, clamp_na, clamp_fa];
const RepeatX_RepeatY_Procs: [MatrixProc; 4] = [repeat_ns, repeat_fs, repeat_na, repeat_fa];
const MirrorX_MirrorY_Procs: [MatrixProc; 4] = [mirror_ns, mirror_fs, mirror_na, mirror_fa];
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:307-311
pub(crate) fn int_clamp(mut x: i32, n: i32) -> u32 {
    if x < 0 {
        x = 0;
    }
    if x >= n {
        x = n - 1;
    }
    x as u32
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:320-330
pub(crate) fn sk_int_mod(mut x: i32, n: i32) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),321; @condition b"n > 0\0".as_ptr(); n>0);
    if x as u32 >= n as u32 {
        if x < 0 {
            x = n + !(!x % n);
        } else {
            x %= n;
        }
    }
    x
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:332-334
pub(crate) fn int_repeat(x: i32, n: i32) -> u32 {
    sk_int_mod(x, n) as u32
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:336-342
pub(crate) fn int_mirror(mut x: i32, n: i32) -> u32 {
    x = sk_int_mod(x, 2 * n);
    if x >= n {
        x = n + !(x - n);
    }
    x as u32
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:344-348
pub(crate) fn fill_sequential(xptr: &mut [u16], mut pos: i32, mut count: i32) {
    let mut offset = 0;
    while count > 0 {
        count -= 1;
        xptr[offset] = pos as u16;
        offset += 1;
        pos += 1;
    }
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:350-355
pub(crate) fn fill_backwards(xptr: &mut [u16], mut pos: i32, mut count: i32) {
    let mut offset = 0;
    while count > 0 {
        count -= 1;
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),352; @condition b"pos >= 0\0".as_ptr(); pos>=0);
        xptr[offset] = pos as u16;
        offset += 1;
        pos -= 1;
    }
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:357-360
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:362-364
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:366-367
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:369-371
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:373-374
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:377-389
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:392-403
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:406-407
unsafe fn clampx_nofilter_trans(
    s: &SkBitmapProcState<'_>,
    xy: &mut [u32],
    mut count: i32,
    x: i32,
    y: i32,
    tiley: TileProc,
) {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),360; @condition b"s.fInvMatrix.isTranslate()\0".as_ptr(); s.fInvMatrix.isTranslate());
    let mapper = SkBitmapProcStateAutoMapper::new(s, x, y, None);
    xy[0] = tiley(mapper.intY(), s.fPixmap.height());
    let mut xpos = mapper.intX();
    let width = s.fPixmap.width();
    if width == 1 {
        std::ptr::write_bytes(xy.as_mut_ptr().add(1).cast::<u16>(), 0, count as usize);
        return;
    }
    let xptr = std::slice::from_raw_parts_mut(xy.as_mut_ptr().add(1).cast::<u16>(), count as usize);
    let mut at = 0usize;
    let mut n;
    if xpos < 0 {
        n = -xpos;
        if n > count {
            n = count;
        }
        std::ptr::write_bytes(xptr.as_mut_ptr(), 0, n as usize);
        count -= n;
        if count == 0 {
            return;
        }
        at += n as usize;
        xpos = 0;
    }
    if xpos < width {
        n = width - xpos;
        if n > count {
            n = count;
        }
        fill_sequential(&mut xptr[at..], xpos, n);
        count -= n;
        if count == 0 {
            return;
        }
        at += n as usize;
    }
    crate::src::opts::SkMemset_opts::memset16(&mut xptr[at..], (width - 1) as u16, count as usize);
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:409-412
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:414-416
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:418-419
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:421-423
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:425-433
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:435-439
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:441-444
unsafe fn repeatx_nofilter_trans(
    s: &SkBitmapProcState<'_>,
    xy: &mut [u32],
    mut count: i32,
    x: i32,
    y: i32,
    tiley: TileProc,
) {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),412; @condition b"s.fInvMatrix.isTranslate()\0".as_ptr(); s.fInvMatrix.isTranslate());
    let mapper = SkBitmapProcStateAutoMapper::new(s, x, y, None);
    xy[0] = tiley(mapper.intY(), s.fPixmap.height());
    let xpos = mapper.intX();
    let width = s.fPixmap.width();
    if width == 1 {
        std::ptr::write_bytes(xy.as_mut_ptr().add(1).cast::<u16>(), 0, count as usize);
        return;
    }
    let xptr = std::slice::from_raw_parts_mut(xy.as_mut_ptr().add(1).cast::<u16>(), count as usize);
    let mut at = 0usize;
    let start = sk_int_mod(xpos, width);
    let mut n = width - start;
    if n > count {
        n = count;
    }
    fill_sequential(&mut xptr[at..], start, n);
    at += n as usize;
    count -= n;
    while count >= width {
        fill_sequential(&mut xptr[at..], 0, width);
        at += width as usize;
        count -= width;
    }
    if count > 0 {
        fill_sequential(&mut xptr[at..], 0, count);
    }
}
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:446-449
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:451-453
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:455-456
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:458-460
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:462-462
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:464-485
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:487-496
// Source: src/core/SkBitmapProcState_matrixProcs.cpp:498-505
unsafe fn mirrorx_nofilter_trans(
    s: &SkBitmapProcState<'_>,
    xy: &mut [u32],
    mut count: i32,
    x: i32,
    y: i32,
    tiley: TileProc,
) {
    crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),449; @condition b"s.fInvMatrix.isTranslate()\0".as_ptr(); s.fInvMatrix.isTranslate());
    let mapper = SkBitmapProcStateAutoMapper::new(s, x, y, None);
    xy[0] = tiley(mapper.intY(), s.fPixmap.height());
    let xpos = mapper.intX();
    let width = s.fPixmap.width();
    if width == 1 {
        std::ptr::write_bytes(xy.as_mut_ptr().add(1).cast::<u16>(), 0, count as usize);
        return;
    }
    let xptr = std::slice::from_raw_parts_mut(xy.as_mut_ptr().add(1).cast::<u16>(), count as usize);
    let mut at = 0usize;
    let mut forward;
    let mut n;
    let mut start = sk_int_mod(xpos, 2 * width);
    if start >= width {
        start = width + !(start - width);
        forward = false;
        n = start + 1;
    } else {
        forward = true;
        n = width - start;
    }
    if n > count {
        n = count;
    }
    if forward {
        fill_sequential(&mut xptr[at..], start, n);
    } else {
        fill_backwards(&mut xptr[at..], start, n);
    }
    forward = !forward;
    at += n as usize;
    count -= n;
    while count >= width {
        if forward {
            fill_sequential(&mut xptr[at..], 0, width);
        } else {
            fill_backwards(&mut xptr[at..], width - 1, width);
        }
        forward = !forward;
        at += width as usize;
        count -= width;
    }
    if count > 0 {
        if forward {
            fill_sequential(&mut xptr[at..], 0, count);
        } else {
            fill_backwards(&mut xptr[at..], width - 1, count);
        }
    }
}
// Uncredited native template-instantiation forwarders; scratch retains the real
// packed 32-bit Y + consecutive 16-bit X byte layout, not a State-layout cast.
unsafe fn clampx_clamp(s: &SkBitmapProcState<'_>, xy: &mut [u32], n: i32, x: i32, y: i32) {
    clampx_nofilter_trans(s, xy, n, x, y, int_clamp);
}
unsafe fn repeatx_repeat(s: &SkBitmapProcState<'_>, xy: &mut [u32], n: i32, x: i32, y: i32) {
    repeatx_nofilter_trans(s, xy, n, x, y, int_repeat);
}
unsafe fn mirrorx_mirror(s: &SkBitmapProcState<'_>, xy: &mut [u32], n: i32, x: i32, y: i32) {
    mirrorx_nofilter_trans(s, xy, n, x, y, int_mirror);
}
impl SkBitmapProcState<'_> {
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:511-513
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:515-515
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:517-524
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:527-530
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:532-532
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:534-537
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:540-541
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:543-547
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:549-551
    pub(crate) unsafe fn chooseMatrixProc(
        &mut self,
        translate_only_matrix: bool,
    ) -> Option<MatrixProc> {
        use crate::include::core::SkTileMode::SkTileMode;
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),512; @condition b"!fInvMatrix.hasPerspective()\0".as_ptr(); !self.fInvMatrix.hasPerspective());
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),513; @condition b"fTileModeX != SkTileMode::kDecal\0".as_ptr(); self.fTileModeX!=SkTileMode::kDecal);
        if self.fTileModeX == self.fTileModeY {
            if translate_only_matrix && !self.fBilerp.assume_init() {
                return Some(match self.fTileModeX {
                    SkTileMode::kDecal => {
                        crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),519; @condition b"false\0".as_ptr(); false);
                        clampx_clamp
                    }
                    SkTileMode::kClamp => clampx_clamp,
                    SkTileMode::kRepeat => repeatx_repeat,
                    SkTileMode::kMirror => mirrorx_mirror,
                });
            }
            let mut index = if self.fBilerp.assume_init() { 1 } else { 0 };
            if !self.fInvMatrix.isScaleTranslate() {
                index |= 2;
            }
            if self.fTileModeX == SkTileMode::kClamp {
                self.fFilterOneX.write(1 << 16);
                self.fFilterOneY.write(1 << 16);
                return Some(ClampX_ClampY_Procs[index]);
            }
            let oneX = (1 << 16) / self.fPixmap.width();
            let oneY = (1 << 16) / self.fPixmap.height();
            self.fFilterOneX.write(oneX);
            self.fFilterOneY.write(oneY);
            if self.fTileModeX == SkTileMode::kRepeat {
                return Some(RepeatX_RepeatY_Procs[index]);
            }
            return Some(MirrorX_MirrorY_Procs[index]);
        }
        crate::SkASSERT!(@source b"src/core/SkBitmapProcState_matrixProcs.cpp\0".as_ptr(),549; @condition b"fTileModeX == fTileModeY\0".as_ptr(); self.fTileModeX==self.fTileModeY);
        None
    }
}
pub(crate) mod sktests {
    use super::*;
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:553-553
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:555-556
    pub fn pack_clamp(f: i32, max: u32) -> u32 {
        pack(f, max, 1 << 16, clamp, extract_low_bits_clamp_clamp)
    }
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:558-558
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:560-561
    pub fn pack_repeat(f: i32, max: u32, width: usize) -> u32 {
        pack(
            f,
            max,
            ((1usize << 16) / width) as i32,
            repeat,
            extract_low_bits_general,
        )
    }
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:563-563
    // Source: src/core/SkBitmapProcState_matrixProcs.cpp:565-566
    pub fn pack_mirror(f: i32, max: u32, width: usize) -> u32 {
        pack(
            f,
            max,
            ((1usize << 16) / width) as i32,
            mirror,
            extract_low_bits_general,
        )
    }
}
