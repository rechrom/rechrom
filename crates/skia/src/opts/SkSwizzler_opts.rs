// Copyright 2016 Google Inc. BSD-style license.
// Selected original ARM64/NEON default target. Native intrinsics retain their exact lane operations.
use std::arch::aarch64::*;
// Source: src/opts/SkSwizzler_opts.inc:47-47
const kFastUnpremul: bool = false;
// Source: src/opts/SkSwizzler_opts.inc:50-52
fn reciprocal_alpha_times_255_portable(a: f32) -> f32 {
    if a != 0.0 {
        255.0 / a
    } else {
        0.0
    }
}
// Source: src/opts/SkSwizzler_opts.inc:54-56
fn reciprocal_alpha_portable(a: f32) -> f32 {
    if a != 0.0 {
        1.0 / a
    } else {
        0.0
    }
}
// Source: src/opts/SkSwizzler_opts.inc:61-63
fn reciprocal_alpha_times_255(a: f32) -> f32 {
    reciprocal_alpha_times_255_portable(a)
}
// Source: src/opts/SkSwizzler_opts.inc:65-67
fn reciprocal_alpha(a: f32) -> f32 {
    reciprocal_alpha_portable(a)
}
// Source: src/opts/SkSwizzler_opts.inc:98-112
pub unsafe fn RGBA_to_rgbA_portable(dst: *mut u32, src: *const u32, count: i32) {
    for i in 0..count {
        let p = src.add(i as usize).read_unaligned();
        let a = (p >> 24) & 255;
        let mut b = (p >> 16) & 255;
        let mut g = (p >> 8) & 255;
        let mut r = p & 255;
        b = (b * a + 127) / 255;
        g = (g * a + 127) / 255;
        r = (r * a + 127) / 255;
        dst.add(i as usize)
            .write_unaligned((a << 24) | (b << 16) | (g << 8) | r);
    }
}
// Source: src/opts/SkSwizzler_opts.inc:123-123
// Source: src/opts/SkSwizzler_opts.inc:125-125
// Source: src/opts/SkSwizzler_opts.inc:134-134
fn pixel_round_as_RP(n: f32) -> u32 {
    unsafe { vrndns_f32(n) as u32 }
}
// Source: src/opts/SkSwizzler_opts.inc:147-149
fn unpremul_quick(reciprocalA: f32, c: f32) -> u32 {
    let v = c * reciprocalA + 0.5;
    if v < 255.0 {
        v as u32
    } else {
        255
    }
}
// Source: src/opts/SkSwizzler_opts.inc:153-157
fn unpremul_simulating_RP(reciprocalA: f32, c: f32) -> u32 {
    let normalizedC = c * (1.0 / 255.0);
    let v = normalizedC * reciprocalA * 255.0;
    let answer = if v < 255.0 { v } else { 255.0 };
    pixel_round_as_RP(answer)
}
// Source: src/opts/SkSwizzler_opts.inc:159-160
// Source: src/opts/SkSwizzler_opts.inc:169-180
fn rgbA_to_CCCA(c00: f32, c08: f32, c16: f32, a: f32) -> u32 {
    if kFastUnpremul {
        let reciprocalA = reciprocal_alpha_times_255(a);
        let unpremul = |c| unpremul_quick(reciprocalA, c);
        ((a as u32) << 24) | (unpremul(c16) << 16) | (unpremul(c08) << 8) | unpremul(c00)
    } else {
        let normalizedA = a * (1.0 / 255.0);
        let reciprocalA = reciprocal_alpha(normalizedA);
        let unpremul = |c| unpremul_simulating_RP(reciprocalA, c);
        ((a as u32) << 24) | (unpremul(c16) << 16) | (unpremul(c08) << 8) | unpremul(c00)
    }
}
// Source: src/opts/SkSwizzler_opts.inc:182-184
// Source: src/opts/SkSwizzler_opts.inc:186-189
// Source: src/opts/SkSwizzler_opts.inc:191-193
pub unsafe fn rgbA_to_RGBA_portable(dst: *mut u32, src: *const u32, count: i32) {
    for i in 0..count {
        let p = src.add(i as usize).read_unaligned();
        let a = ((p >> 24) & 255) as f32;
        let b = ((p >> 16) & 255) as f32;
        let g = ((p >> 8) & 255) as f32;
        let r = (p & 255) as f32;
        dst.add(i as usize)
            .write_unaligned(rgbA_to_CCCA(r, g, b, a));
    }
}
// Source: src/opts/SkSwizzler_opts.inc:195-197
// Source: src/opts/SkSwizzler_opts.inc:199-202
// Source: src/opts/SkSwizzler_opts.inc:204-206
pub unsafe fn rgbA_to_BGRA_portable(dst: *mut u32, src: *const u32, count: i32) {
    for i in 0..count {
        let p = src.add(i as usize).read_unaligned();
        let a = (p >> 24) & 255;
        let b = (p >> 16) & 255;
        let g = (p >> 8) & 255;
        let r = p & 255;
        dst.add(i as usize)
            .write_unaligned(rgbA_to_CCCA(b as f32, g as f32, r as f32, a as f32));
    }
}
// Source: src/opts/SkSwizzler_opts.inc:208-222
pub unsafe fn RGBA_to_bgrA_portable(dst: *mut u32, src: *const u32, count: i32) {
    for i in 0..count {
        let p = src.add(i as usize).read_unaligned();
        let a = (p >> 24) & 255;
        let mut b = (p >> 16) & 255;
        let mut g = (p >> 8) & 255;
        let mut r = p & 255;
        b = (b * a + 127) / 255;
        g = (g * a + 127) / 255;
        r = (r * a + 127) / 255;
        dst.add(i as usize)
            .write_unaligned((a << 24) | (r << 16) | (g << 8) | b);
    }
}
// Source: src/opts/SkSwizzler_opts.inc:224-235
pub unsafe fn RGBA_to_BGRA_portable(dst: *mut u32, src: *const u32, count: i32) {
    for i in 0..count {
        let p = src.add(i as usize).read_unaligned();
        let a = (p >> 24) & 255;
        let b = (p >> 16) & 255;
        let g = (p >> 8) & 255;
        let r = p & 255;
        dst.add(i as usize)
            .write_unaligned((a << 24) | (r << 16) | (g << 8) | b);
    }
}
// Source: src/opts/SkSwizzler_opts.inc:237-247
pub unsafe fn grayA_to_RGBA_portable(dst: *mut u32, mut src: *const u8, count: i32) {
    for i in 0..count {
        let g = *src as u32;
        let a = *src.add(1) as u32;
        src = src.add(2);
        dst.add(i as usize)
            .write_unaligned((a << 24) | (g << 16) | (g << 8) | g);
    }
}
// Source: src/opts/SkSwizzler_opts.inc:249-260
pub unsafe fn grayA_to_rgbA_portable(dst: *mut u32, mut src: *const u8, count: i32) {
    for i in 0..count {
        let mut g = *src as u32;
        let a = *src.add(1) as u32;
        src = src.add(2);
        g = (g * a + 127) / 255;
        dst.add(i as usize)
            .write_unaligned((a << 24) | (g << 16) | (g << 8) | g);
    }
}
// Source: src/opts/SkSwizzler_opts.inc:262-267
// Source: src/opts/SkSwizzler_opts.inc:269-277
pub unsafe fn inverted_CMYK_to_RGB1_portable(dst: *mut u32, src: *const u32, count: i32) {
    for i in 0..count {
        let p = src.add(i as usize).read_unaligned();
        let k = (p >> 24) & 255;
        let y = (p >> 16) & 255;
        let m = (p >> 8) & 255;
        let c = p & 255;
        let b = (y * k + 127) / 255;
        let g = (m * k + 127) / 255;
        let r = (c * k + 127) / 255;
        dst.add(i as usize)
            .write_unaligned((255 << 24) | (b << 16) | (g << 8) | r);
    }
}
// Source: src/opts/SkSwizzler_opts.inc:279-293
pub unsafe fn inverted_CMYK_to_BGR1_portable(dst: *mut u32, src: *const u32, count: i32) {
    for i in 0..count {
        let p = src.add(i as usize).read_unaligned();
        let k = (p >> 24) & 255;
        let y = (p >> 16) & 255;
        let m = (p >> 8) & 255;
        let c = p & 255;
        let b = (y * k + 127) / 255;
        let g = (m * k + 127) / 255;
        let r = (c * k + 127) / 255;
        dst.add(i as usize)
            .write_unaligned((255 << 24) | (r << 16) | (g << 8) | b);
    }
}
// Source: src/opts/SkSwizzler_opts.inc:298-298
// Source: src/opts/SkSwizzler_opts.inc:318-319
unsafe fn div255_round(x: uint16x8_t) -> uint8x8_t {
    vraddhn_u16(x, vrshrq_n_u16::<8>(x))
}
// Source: src/opts/SkSwizzler_opts.inc:322-324
unsafe fn scale(x: uint8x8_t, y: uint8x8_t) -> uint8x8_t {
    div255_round(vmull_u8(x, y))
}
// Source: src/opts/SkSwizzler_opts.inc:326-327
// Source: src/opts/SkSwizzler_opts.inc:329-329
// Source: src/opts/SkSwizzler_opts.inc:331-334
// Source: src/opts/SkSwizzler_opts.inc:337-339
// Source: src/opts/SkSwizzler_opts.inc:342-355
// Source: src/opts/SkSwizzler_opts.inc:358-360
unsafe fn premul_should_swapRB(
    kSwapRB: bool,
    mut dst: *mut u32,
    mut src: *const u32,
    mut count: i32,
) {
    while count >= 8 {
        let mut rgba = vld4_u8(src.cast());
        let a = rgba.3;
        let mut b = rgba.2;
        let mut g = rgba.1;
        let mut r = rgba.0;
        b = scale(b, a);
        g = scale(g, a);
        r = scale(r, a);
        if kSwapRB {
            rgba.2 = r;
            rgba.1 = g;
            rgba.0 = b;
        } else {
            rgba.2 = b;
            rgba.1 = g;
            rgba.0 = r;
        }
        vst4_u8(dst.cast(), rgba);
        src = src.add(8);
        dst = dst.add(8);
        count -= 8;
    }
    let proc: unsafe fn(*mut u32, *const u32, i32) = if kSwapRB {
        RGBA_to_bgrA_portable
    } else {
        RGBA_to_rgbA_portable
    };
    proc(dst, src, count);
}
// Source: src/opts/SkSwizzler_opts.inc:362-364
pub unsafe fn RGBA_to_rgbA(dst: *mut u32, src: *const u32, count: i32) {
    premul_should_swapRB(false, dst, src, count)
}
// Source: src/opts/SkSwizzler_opts.inc:366-368
pub unsafe fn RGBA_to_bgrA(dst: *mut u32, src: *const u32, count: i32) {
    premul_should_swapRB(true, dst, src, count)
}
// Source: src/opts/SkSwizzler_opts.inc:370-372
// Source: src/opts/SkSwizzler_opts.inc:374-374
// Source: src/opts/SkSwizzler_opts.inc:377-377
// Source: src/opts/SkSwizzler_opts.inc:380-384
// Source: src/opts/SkSwizzler_opts.inc:386-386
// Source: src/opts/SkSwizzler_opts.inc:388-388
// Source: src/opts/SkSwizzler_opts.inc:391-391
// Source: src/opts/SkSwizzler_opts.inc:394-398
// Source: src/opts/SkSwizzler_opts.inc:400-401
pub unsafe fn RGBA_to_BGRA(mut dst: *mut u32, mut src: *const u32, mut count: i32) {
    while count >= 16 {
        let mut rgba = vld4q_u8(src.cast());
        std::mem::swap(&mut rgba.0, &mut rgba.2);
        vst4q_u8(dst.cast(), rgba);
        src = src.add(16);
        dst = dst.add(16);
        count -= 16;
    }
    if count >= 8 {
        let mut rgba = vld4_u8(src.cast());
        std::mem::swap(&mut rgba.0, &mut rgba.2);
        vst4_u8(dst.cast(), rgba);
        src = src.add(8);
        dst = dst.add(8);
        count -= 8;
    }
    RGBA_to_BGRA_portable(dst, src, count);
}
// Source: src/opts/SkSwizzler_opts.inc:403-404
// Source: src/opts/SkSwizzler_opts.inc:406-406
// Source: src/opts/SkSwizzler_opts.inc:409-413
// Source: src/opts/SkSwizzler_opts.inc:416-420
// Source: src/opts/SkSwizzler_opts.inc:423-427
// Source: src/opts/SkSwizzler_opts.inc:429-429
// Source: src/opts/SkSwizzler_opts.inc:431-431
// Source: src/opts/SkSwizzler_opts.inc:434-436
// Source: src/opts/SkSwizzler_opts.inc:439-443
// Source: src/opts/SkSwizzler_opts.inc:446-450
// Source: src/opts/SkSwizzler_opts.inc:452-454
unsafe fn expand_grayA(kPremul: bool, mut dst: *mut u32, mut src: *const u8, mut count: i32) {
    while count >= 16 {
        let mut ga = vld2q_u8(src);
        if kPremul {
            ga.0 = vcombine_u8(
                scale(vget_low_u8(ga.0), vget_low_u8(ga.1)),
                scale(vget_high_u8(ga.0), vget_high_u8(ga.1)),
            );
        }
        let rgba = uint8x16x4_t(ga.0, ga.0, ga.0, ga.1);
        vst4q_u8(dst.cast(), rgba);
        src = src.add(32);
        dst = dst.add(16);
        count -= 16;
    }
    if count >= 8 {
        let mut ga = vld2_u8(src);
        if kPremul {
            ga.0 = scale(ga.0, ga.1);
        }
        let rgba = uint8x8x4_t(ga.0, ga.0, ga.0, ga.1);
        vst4_u8(dst.cast(), rgba);
        src = src.add(16);
        dst = dst.add(8);
        count -= 8;
    }
    let proc: unsafe fn(*mut u32, *const u8, i32) = if kPremul {
        grayA_to_rgbA_portable
    } else {
        grayA_to_RGBA_portable
    };
    proc(dst, src, count);
}
// Source: src/opts/SkSwizzler_opts.inc:456-458
pub unsafe fn grayA_to_RGBA(dst: *mut u32, src: *const u8, count: i32) {
    expand_grayA(false, dst, src, count)
}
// Source: src/opts/SkSwizzler_opts.inc:460-462
pub unsafe fn grayA_to_rgbA(dst: *mut u32, src: *const u8, count: i32) {
    expand_grayA(true, dst, src, count)
}
// Source: src/opts/SkSwizzler_opts.inc:464-464
#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    kRGB1,
    kBGR1,
}
// Source: src/opts/SkSwizzler_opts.inc:465-466
// Source: src/opts/SkSwizzler_opts.inc:468-468
// Source: src/opts/SkSwizzler_opts.inc:470-473
// Source: src/opts/SkSwizzler_opts.inc:476-478
// Source: src/opts/SkSwizzler_opts.inc:481-496
// Source: src/opts/SkSwizzler_opts.inc:498-500
unsafe fn inverted_cmyk_to(format: Format, mut dst: *mut u32, mut src: *const u32, mut count: i32) {
    while count >= 8 {
        let mut pixels = vld4_u8(src.cast());
        let k = pixels.3;
        let y = pixels.2;
        let m = pixels.1;
        let c = pixels.0;
        let b = scale(y, k);
        let g = scale(m, k);
        let r = scale(c, k);
        if format == Format::kBGR1 {
            pixels.3 = vdup_n_u8(255);
            pixels.2 = r;
            pixels.1 = g;
            pixels.0 = b;
        } else {
            pixels.3 = vdup_n_u8(255);
            pixels.2 = b;
            pixels.1 = g;
            pixels.0 = r;
        }
        vst4_u8(dst.cast(), pixels);
        src = src.add(8);
        dst = dst.add(8);
        count -= 8;
    }
    let proc: unsafe fn(*mut u32, *const u32, i32) = if format == Format::kBGR1 {
        inverted_CMYK_to_BGR1_portable
    } else {
        inverted_CMYK_to_RGB1_portable
    };
    proc(dst, src, count);
}
// Source: src/opts/SkSwizzler_opts.inc:502-504
pub unsafe fn inverted_CMYK_to_RGB1(dst: *mut u32, src: *const u32, count: i32) {
    inverted_cmyk_to(Format::kRGB1, dst, src, count)
}
// Source: src/opts/SkSwizzler_opts.inc:506-508
pub unsafe fn inverted_CMYK_to_BGR1(dst: *mut u32, src: *const u32, count: i32) {
    inverted_cmyk_to(Format::kBGR1, dst, src, count)
}
// Source: src/opts/SkSwizzler_opts.inc:510-511
// Source: src/opts/SkSwizzler_opts.inc:515-517
// Source: src/opts/SkSwizzler_opts.inc:519-519
// Source: src/opts/SkSwizzler_opts.inc:521-521
// Source: src/opts/SkSwizzler_opts.inc:525-525
// Source: src/opts/SkSwizzler_opts.inc:527-530
// Source: src/opts/SkSwizzler_opts.inc:532-539
// Source: src/opts/SkSwizzler_opts.inc:541-545
// Source: src/opts/SkSwizzler_opts.inc:547-550
// Source: src/opts/SkSwizzler_opts.inc:552-553
// Source: src/opts/SkSwizzler_opts.inc:555-556
// Source: src/opts/SkSwizzler_opts.inc:558-560
// Source: src/opts/SkSwizzler_opts.inc:562-564
// Source: src/opts/SkSwizzler_opts.inc:566-572
// Source: src/opts/SkSwizzler_opts.inc:574-578
// Source: src/opts/SkSwizzler_opts.inc:581-586
unsafe fn common_rgbA_to_RGBA<const swapRB: bool>(
    mut dst: *mut u32,
    mut src: *const u32,
    mut count: i32,
) {
    if !kFastUnpremul {
        while count >= 8 {
            let input = vld4_u8(src.cast());
            let round = |v| vcvtnq_u32_f32(v);
            const kN: f32 = 1.0 / 255.0;
            let toNormalized = |v| vmulq_f32(vcvtq_f32_u32(vmovl_u16(v)), vdupq_n_f32(kN));
            let unpremulHalf = |invA, v| {
                let normalizedV = toNormalized(v);
                let divided = vmulq_f32(invA, normalizedV);
                let denormalized = vmulq_f32(divided, vdupq_n_f32(255.0));
                let rounded = round(denormalized);
                vqmovn_u32(rounded)
            };
            let reciprocal = |a| {
                let mask = vmvnq_u32(vceqq_f32(a, vdupq_n_f32(0.0)));
                let recip = vdivq_f32(vdupq_n_f32(1.0), a);
                vreinterpretq_f32_u32(vandq_u32(mask, vreinterpretq_u32_f32(recip)))
            };
            let a = input.3;
            let intA = vmovl_u8(a);
            let invALow = reciprocal(toNormalized(vget_low_u16(intA)));
            let invAHigh = reciprocal(toNormalized(vget_high_u16(intA)));
            let unpremul = |v| {
                let to16 = vmovl_u8(v);
                let low = unpremulHalf(invALow, vget_low_u16(to16));
                let high = unpremulHalf(invAHigh, vget_high_u16(to16));
                let combined = vcombine_u16(low, high);
                vqmovn_u16(combined)
            };
            let b = unpremul(input.2);
            let g = unpremul(input.1);
            let r = unpremul(input.0);
            if swapRB {
                vst4_u8(dst.cast(), uint8x8x4_t(b, g, r, a));
            } else {
                vst4_u8(dst.cast(), uint8x8x4_t(r, g, b, a));
            }
            src = src.add(8);
            dst = dst.add(8);
            count -= 8;
        }
    }
    if swapRB {
        rgbA_to_BGRA_portable(dst, src, count);
    } else {
        rgbA_to_RGBA_portable(dst, src, count);
    }
}
// Source: src/opts/SkSwizzler_opts.inc:588-590
pub unsafe fn rgbA_to_RGBA(dst: *mut u32, src: *const u32, count: i32) {
    common_rgbA_to_RGBA::<false>(dst, src, count)
}
// Source: src/opts/SkSwizzler_opts.inc:592-594
pub unsafe fn rgbA_to_BGRA(dst: *mut u32, src: *const u32, count: i32) {
    common_rgbA_to_RGBA::<true>(dst, src, count)
}
// Source: src/opts/SkSwizzler_opts.inc:1621-1628
pub unsafe fn gray_to_RGB1_portable(dst: *mut u32, src: *const u8, count: i32) {
    for i in 0..count {
        let g = *src.add(i as usize) as u32;
        dst.add(i as usize)
            .write_unaligned((255 << 24) | (g << 16) | (g << 8) | g);
    }
}
// Source: src/opts/SkSwizzler_opts.inc:1630-1631
// Source: src/opts/SkSwizzler_opts.inc:1633-1633
// Source: src/opts/SkSwizzler_opts.inc:1636-1640
// Source: src/opts/SkSwizzler_opts.inc:1643-1648
// Source: src/opts/SkSwizzler_opts.inc:1650-1650
// Source: src/opts/SkSwizzler_opts.inc:1653-1657
// Source: src/opts/SkSwizzler_opts.inc:1660-1666
pub unsafe fn gray_to_RGB1(mut dst: *mut u32, mut src: *const u8, mut count: i32) {
    while count >= 16 {
        let gray = vld1q_u8(src);
        let rgba = uint8x16x4_t(gray, gray, gray, vdupq_n_u8(255));
        vst4q_u8(dst.cast(), rgba);
        src = src.add(16);
        dst = dst.add(16);
        count -= 16;
    }
    if count >= 8 {
        let gray = vld1_u8(src);
        let rgba = uint8x8x4_t(gray, gray, gray, vdup_n_u8(255));
        vst4_u8(dst.cast(), rgba);
        src = src.add(8);
        dst = dst.add(8);
        count -= 8;
    }
    gray_to_RGB1_portable(dst, src, count);
}
// Source: src/opts/SkSwizzler_opts.inc:1805-1816
pub unsafe fn RGB_to_RGB1_portable(dst: *mut u32, mut src: *const u8, count: i32) {
    for i in 0..count {
        let r = *src as u32;
        let g = *src.add(1) as u32;
        let b = *src.add(2) as u32;
        src = src.add(3);
        dst.add(i as usize)
            .write_unaligned((255 << 24) | (b << 16) | (g << 8) | r);
    }
}
// Source: src/opts/SkSwizzler_opts.inc:1817-1828
pub unsafe fn RGB_to_BGR1_portable(dst: *mut u32, mut src: *const u8, count: i32) {
    for i in 0..count {
        let r = *src as u32;
        let g = *src.add(1) as u32;
        let b = *src.add(2) as u32;
        src = src.add(3);
        dst.add(i as usize)
            .write_unaligned((255 << 24) | (r << 16) | (g << 8) | b);
    }
}
// Source: src/opts/SkSwizzler_opts.inc:1830-1832
// Source: src/opts/SkSwizzler_opts.inc:1834-1834
// Source: src/opts/SkSwizzler_opts.inc:1837-1846
// Source: src/opts/SkSwizzler_opts.inc:1849-1853
// Source: src/opts/SkSwizzler_opts.inc:1855-1855
// Source: src/opts/SkSwizzler_opts.inc:1857-1857
// Source: src/opts/SkSwizzler_opts.inc:1860-1869
// Source: src/opts/SkSwizzler_opts.inc:1872-1876
// Source: src/opts/SkSwizzler_opts.inc:1879-1881
unsafe fn insert_alpha_should_swaprb(
    kSwapRB: bool,
    mut dst: *mut u32,
    mut src: *const u8,
    mut count: i32,
) {
    while count >= 16 {
        let rgb = vld3q_u8(src);
        let rgba = if kSwapRB {
            uint8x16x4_t(rgb.2, rgb.1, rgb.0, vdupq_n_u8(255))
        } else {
            uint8x16x4_t(rgb.0, rgb.1, rgb.2, vdupq_n_u8(255))
        };
        vst4q_u8(dst.cast(), rgba);
        src = src.add(48);
        dst = dst.add(16);
        count -= 16;
    }
    if count >= 8 {
        let rgb = vld3_u8(src);
        let rgba = if kSwapRB {
            uint8x8x4_t(rgb.2, rgb.1, rgb.0, vdup_n_u8(255))
        } else {
            uint8x8x4_t(rgb.0, rgb.1, rgb.2, vdup_n_u8(255))
        };
        vst4_u8(dst.cast(), rgba);
        src = src.add(24);
        dst = dst.add(8);
        count -= 8;
    }
    let proc: unsafe fn(*mut u32, *const u8, i32) = if kSwapRB {
        RGB_to_BGR1_portable
    } else {
        RGB_to_RGB1_portable
    };
    proc(dst, src, count);
}
// Source: src/opts/SkSwizzler_opts.inc:1883-1885
pub unsafe fn RGB_to_RGB1(dst: *mut u32, src: *const u8, count: i32) {
    insert_alpha_should_swaprb(false, dst, src, count)
}
// Source: src/opts/SkSwizzler_opts.inc:1886-1888
pub unsafe fn RGB_to_BGR1(dst: *mut u32, src: *const u8, count: i32) {
    insert_alpha_should_swaprb(true, dst, src, count)
}
