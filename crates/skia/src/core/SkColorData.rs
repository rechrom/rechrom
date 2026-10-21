// Copyright 2006 The Android Open Source Project. BSD-style license.
use super::SkColor::*;
use super::SkColorPriv::*;
// Source: src/core/SkColorData.h:24-26
// Source: src/core/SkColorData.h:28-30
// Source: src/core/SkColorData.h:32-34
// Source: src/core/SkColorData.h:36-38
pub const SK_R16_BITS: u32 = 5;
pub const SK_G16_BITS: u32 = 6;
pub const SK_B16_BITS: u32 = 5;
pub const SK_R16_SHIFT: u32 = 11;
pub const SK_G16_SHIFT: u32 = 5;
pub const SK_B16_SHIFT: u32 = 0;
pub const SK_R16_MASK: u32 = 31;
pub const SK_G16_MASK: u32 = 63;
pub const SK_B16_MASK: u32 = 31;
pub fn SkGetPackedR16(c: u32) -> u32 {
    (c >> 11) & 31
}
pub fn SkGetPackedG16(c: u32) -> u32 {
    (c >> 5) & 63
}
pub fn SkGetPackedB16(c: u32) -> u32 {
    c & 31
}
// Source: src/core/SkColorData.h:40-42
// Source: src/core/SkColorData.h:44-46
// Source: src/core/SkColorData.h:48-50
pub fn SkR16ToR32(r: u32) -> u32 {
    (r << 3) | (r >> 2)
}
pub fn SkG16ToG32(g: u32) -> u32 {
    (g << 2) | (g >> 4)
}
pub fn SkB16ToB32(b: u32) -> u32 {
    (b << 3) | (b >> 2)
}
// Source: src/core/SkColorData.h:52-54
pub fn SkPacked16ToR32(c: u32) -> u32 {
    SkR16ToR32(SkGetPackedR16(c))
}
pub fn SkPacked16ToG32(c: u32) -> u32 {
    SkG16ToG32(SkGetPackedG16(c))
}
pub fn SkPacked16ToB32(c: u32) -> u32 {
    SkB16ToB32(SkGetPackedB16(c))
}
// Source: src/core/SkColorData.h:58-58
// Keep macro argument evaluation under debug_assert, as in native SkASSERT.
macro_rules! SkASSERT_IS_BYTE {
    (@source $file:expr,$line:expr; @condition $native:expr; $x:expr) => {
        $crate::SkASSERT!(@source $file,$line; @condition $native; 0 == (($x) & !0xFFu32))
    };
    ($x:expr) => { compile_error!("SkASSERT_IS_BYTE requires original native @source and @condition") };
}
pub(crate) use SkASSERT_IS_BYTE;
// Source: src/core/SkColorData.h:63-64
// Source: src/core/SkColorData.h:66-69
pub fn SkSwizzle_RB(c: u32) -> u32 {
    let c0 = (c >> 16) & 255;
    let c1 = c & 255;
    (c & !0x00ff00ff) | c0 | (c1 << 16)
}
// Source: src/core/SkColorData.h:71-78
// Source: src/core/SkColorData.h:80-87
pub fn SkPackARGB_as_RGBA(a: u32, r: u32, g: u32, b: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),72; @condition b"0 == ((a) & ~0xFFu)\0".as_ptr(); 0==(a&!0xffu32));
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),73; @condition b"0 == ((r) & ~0xFFu)\0".as_ptr(); 0==(r&!0xffu32));
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),74; @condition b"0 == ((g) & ~0xFFu)\0".as_ptr(); 0==(g&!0xffu32));
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),75; @condition b"0 == ((b) & ~0xFFu)\0".as_ptr(); 0==(b&!0xffu32));
    (a << 24) | r | (g << 8) | (b << 16)
}
pub fn SkPackARGB_as_BGRA(a: u32, r: u32, g: u32, b: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),81; @condition b"0 == ((a) & ~0xFFu)\0".as_ptr(); 0==(a&!0xffu32));
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),82; @condition b"0 == ((r) & ~0xFFu)\0".as_ptr(); 0==(r&!0xffu32));
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),83; @condition b"0 == ((g) & ~0xFFu)\0".as_ptr(); 0==(g&!0xffu32));
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),84; @condition b"0 == ((b) & ~0xFFu)\0".as_ptr(); 0==(b&!0xffu32));
    (a << 24) | (r << 16) | (g << 8) | b
}
// Source: src/core/SkColorData.h:89-95
pub fn SkSwizzle_RGBA_to_PMColor(c: u32) -> u32 {
    if SK_PMCOLOR_IS_RGBA {
        c
    } else {
        SkSwizzle_RB(c)
    }
}
// Source: src/core/SkColorData.h:97-103
pub fn SkSwizzle_BGRA_to_PMColor(c: u32) -> u32 {
    if SK_PMCOLOR_IS_BGRA {
        c
    } else {
        SkSwizzle_RB(c)
    }
}
// Source: src/core/SkColorData.h:109-111
pub const SK_ITU_BT709_LUM_COEFF_R: f32 = 0.2126;
pub const SK_ITU_BT709_LUM_COEFF_G: f32 = 0.7152;
pub const SK_ITU_BT709_LUM_COEFF_B: f32 = 0.0722;
// Source: src/core/SkColorData.h:116-118
pub const SK_LUM_COEFF_R: f32 = SK_ITU_BT709_LUM_COEFF_R;
pub const SK_LUM_COEFF_G: f32 = SK_ITU_BT709_LUM_COEFF_G;
pub const SK_LUM_COEFF_B: f32 = SK_ITU_BT709_LUM_COEFF_B;
// Source: src/core/SkColorData.h:124-124
// Source: src/core/SkColorData.h:128-129
pub fn SkComputeLuminance(r: u32, g: u32, b: u32) -> u32 {
    (r * 54 + g * 183 + b * 19) >> 8
}
// Source: src/core/SkColorData.h:134-137
pub fn SkAlphaMulInv256(value: u32, alpha256: u32) -> u32 {
    let prod = 0xffff - value * alpha256;
    (prod + (prod >> 8)) >> 8
}
// Source: src/core/SkColorData.h:142-145
pub fn SkAlphaBlend(src: i32, dst: i32, scale256: i32) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),143; @condition b"(unsigned)scale256 <= 256\0".as_ptr(); (scale256 as u32)<=256);
    dst + ((src - dst) * scale256 >> 8)
}
// Source: src/core/SkColorData.h:147-150
// Source: src/core/SkColorData.h:152-153
// Source: src/core/SkColorData.h:155-157
pub fn SkPackRGB16(r: u32, g: u32, b: u32) -> u16 {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),148; @condition b"r <= SK_R16_MASK\0".as_ptr(); r<=31);
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),149; @condition b"g <= SK_G16_MASK\0".as_ptr(); g<=63);
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),150; @condition b"b <= SK_B16_MASK\0".as_ptr(); b<=31);
    ((r << 11) | (g << 5) | b) as u16
}
pub const SK_R16_MASK_IN_PLACE: u32 = 31 << 11;
pub const SK_G16_MASK_IN_PLACE: u32 = 63 << 5;
pub const SK_B16_MASK_IN_PLACE: u32 = 31;
// Source: src/core/SkColorData.h:168-172
// Source: src/core/SkColorData.h:174-175
pub fn SkFourByteInterp256(src: u32, dst: u32, scale: i32) -> u32 {
    let a = SkAlphaBlend(
        SkGetPackedA32(src) as i32,
        SkGetPackedA32(dst) as i32,
        scale,
    ) as u8;
    let r = SkAlphaBlend(
        SkGetPackedR32(src) as i32,
        SkGetPackedR32(dst) as i32,
        scale,
    ) as u8;
    let g = SkAlphaBlend(
        SkGetPackedG32(src) as i32,
        SkGetPackedG32(dst) as i32,
        scale,
    ) as u8;
    let b = SkAlphaBlend(
        SkGetPackedB32(src) as i32,
        SkGetPackedB32(dst) as i32,
        scale,
    ) as u8;
    SkPackARGB32(a as u32, r as u32, g as u32, b as u32)
}
// Source: src/core/SkColorData.h:183-186
pub fn SkFourByteInterp(src: u32, dst: u32, srcWeight: u32) -> u32 {
    let scale = SkAlpha255To256(srcWeight) as i32;
    SkFourByteInterp256(src, dst, scale)
}
// Source: src/core/SkColorData.h:191-195
// Distinct Rust names for overloads; tuple substitutes two output pointers.
pub fn SkSplay32(color: u32) -> (u32, u32) {
    ((color >> 8) & 0x00ff00ff, color & 0x00ff00ff)
}
// Source: src/core/SkColorData.h:201-207
pub fn SkSplay64(color: u32) -> u64 {
    let mut agrb = ((color >> 8) & 0x00ff00ff) as u64;
    agrb <<= 32;
    agrb |= (color & 0x00ff00ff) as u64;
    agrb
}
// Source: src/core/SkColorData.h:212-215
pub fn SkUnsplay32(ag: u32, rb: u32) -> u32 {
    (ag & 0xff00ff00) | ((rb & 0xff00ff00) >> 8)
}
// Source: src/core/SkColorData.h:221-226
pub fn SkUnsplay64(agrb: u64) -> u32 {
    (((agrb & 0xff00ff00) >> 8) | ((agrb >> 32) & 0xff00ff00)) as u32
}
// Source: src/core/SkColorData.h:228-229
// Source: src/core/SkColorData.h:232-234
// Source: src/core/SkColorData.h:236-237
// Source: src/core/SkColorData.h:239-240
pub fn SkFastFourByteInterp256_32(src: u32, dst: u32, scale: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),229; @condition b"scale <= 256\0".as_ptr(); scale<=256);
    let (src_ag, src_rb) = SkSplay32(src);
    let (dst_ag, dst_rb) = SkSplay32(dst);
    let ret_ag = src_ag
        .wrapping_mul(scale)
        .wrapping_add((256 - scale).wrapping_mul(dst_ag));
    let ret_rb = src_rb
        .wrapping_mul(scale)
        .wrapping_add((256 - scale).wrapping_mul(dst_rb));
    SkUnsplay32(ret_ag, ret_rb)
}
// Source: src/core/SkColorData.h:242-243
// Source: src/core/SkColorData.h:245-246
pub fn SkFastFourByteInterp256_64(src: u32, dst: u32, scale: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),243; @condition b"scale <= 256\0".as_ptr(); scale<=256);
    SkUnsplay64(
        SkSplay64(src)
            .wrapping_mul(scale as u64)
            .wrapping_add((256 - scale) as u64 * SkSplay64(dst)),
    )
}
// Source: src/core/SkColorData.h:253-253
// Source: src/core/SkColorData.h:255-260
pub fn SkFastFourByteInterp256(src: u32, dst: u32, scale: u32) -> u32 {
    if std::mem::size_of::<usize>() == 4 {
        SkFastFourByteInterp256_32(src, dst, scale)
    } else {
        SkFastFourByteInterp256_64(src, dst, scale)
    }
}
// Source: src/core/SkColorData.h:266-267
// Source: src/core/SkColorData.h:270-271
pub fn SkFastFourByteInterp(src: u32, dst: u32, srcWeight: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),267; @condition b"srcWeight <= 255\0".as_ptr(); srcWeight<=255);
    SkFastFourByteInterp256(src, dst, srcWeight + (srcWeight >> 7))
}
// Source: src/core/SkColorData.h:276-278
pub fn SkPMLerp(src: u32, dst: u32, scale: u32) -> u32 {
    SkFastFourByteInterp256(src, dst, scale)
}
// Source: src/core/SkColorData.h:280-281
// Source: src/core/SkColorData.h:283-284
// Source: src/core/SkColorData.h:286-286
// Source: src/core/SkColorData.h:288-289
// Source: src/core/SkColorData.h:291-292
// Source: src/core/SkColorData.h:294-295
pub fn SkBlendARGB32(src: u32, dst: u32, aa: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),281; @condition b"(unsigned)aa <= 255\0".as_ptr(); aa<=255);
    let src_scale = SkAlpha255To256(aa);
    let dst_scale = SkAlphaMulInv256(SkGetPackedA32(src), src_scale);
    const MASK: u32 = 0x00ff00ff;
    let src_rb = (src & MASK).wrapping_mul(src_scale);
    let src_ag = ((src >> 8) & MASK).wrapping_mul(src_scale);
    let dst_rb = (dst & MASK).wrapping_mul(dst_scale);
    let dst_ag = ((dst >> 8) & MASK).wrapping_mul(dst_scale);
    ((src_rb.wrapping_add(dst_rb) >> 8) & MASK) | (src_ag.wrapping_add(dst_ag) & !MASK)
}
// Source: src/core/SkColorData.h:300-302
// Source: src/core/SkColorData.h:305-316
// Source: src/core/SkColorData.h:318-320
pub fn SkR32ToR16(r: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),306; @condition b"(unsigned)(r) <= SK_R32_MASK\0".as_ptr(); r<=255);
    r >> 3
}
pub fn SkG32ToG16(g: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),310; @condition b"(unsigned)(g) <= SK_G32_MASK\0".as_ptr(); g<=255);
    g >> 2
}
pub fn SkB32ToB16(b: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),314; @condition b"(unsigned)(b) <= SK_B32_MASK\0".as_ptr(); b<=255);
    b >> 3
}
// Source: src/core/SkColorData.h:323-328
pub fn SkPixel32ToPixel16(c: u32) -> u32 {
    let r = ((c >> (16 + 3)) & 31) << 11;
    let g = ((c >> (8 + 2)) & 63) << 5;
    let b = (c >> 3) & 31;
    r | g | b
}
// Source: src/core/SkColorData.h:330-334
pub fn SkPack888ToRGB16(r: u32, g: u32, b: u32) -> u32 {
    (SkR32ToR16(r) << 11) | (SkG32ToG16(g) << 5) | SkB32ToB16(b)
}
// Source: src/core/SkColorData.h:338-339
// Source: src/core/SkColorData.h:341-343
// Source: src/core/SkColorData.h:345-347
// Source: src/core/SkColorData.h:349-350
pub fn SkPixel16ToColor(src: u32) -> SkColor {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),339; @condition b"src == SkToU16(src)\0".as_ptr(); src==src as u16 as u32);
    let r = SkPacked16ToR32(src);
    let g = SkPacked16ToG32(src);
    let b = SkPacked16ToB32(src);
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),345; @condition b"(r >> (8 - SK_R16_BITS)) == SkGetPackedR16(src)\0".as_ptr(); (r>>3)==SkGetPackedR16(src));
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),346; @condition b"(g >> (8 - SK_G16_BITS)) == SkGetPackedG16(src)\0".as_ptr(); (g>>2)==SkGetPackedG16(src));
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),347; @condition b"(b >> (8 - SK_B16_BITS)) == SkGetPackedB16(src)\0".as_ptr(); (b>>3)==SkGetPackedB16(src));
    SkColorSetRGB(r, g, b)
}
// Source: src/core/SkColorData.h:354-354
// Source: src/core/SkColorData.h:357-360
pub type SkPMColor16 = u16;
pub const SK_A4444_SHIFT: u32 = 0;
pub const SK_R4444_SHIFT: u32 = 12;
pub const SK_G4444_SHIFT: u32 = 8;
pub const SK_B4444_SHIFT: u32 = 4;
// Source: src/core/SkColorData.h:362-365
pub fn SkReplicateNibble(nib: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorData.h\0".as_ptr(),363; @condition b"nib <= 0xF\0".as_ptr(); nib<=15);
    (nib << 4) | nib
}
// Source: src/core/SkColorData.h:367-370
// Source: src/core/SkColorData.h:372-372
pub fn SkGetPackedA4444(c: u32) -> u32 {
    c & 15
}
pub fn SkGetPackedR4444(c: u32) -> u32 {
    (c >> 12) & 15
}
pub fn SkGetPackedG4444(c: u32) -> u32 {
    (c >> 8) & 15
}
pub fn SkGetPackedB4444(c: u32) -> u32 {
    (c >> 4) & 15
}
pub fn SkPacked4444ToA32(c: u32) -> u32 {
    SkReplicateNibble(SkGetPackedA4444(c))
}
// Source: src/core/SkColorData.h:374-380
pub fn SkPixel4444ToPixel32(c: u32) -> u32 {
    let d = (SkGetPackedA4444(c) << 24)
        | (SkGetPackedR4444(c) << 16)
        | (SkGetPackedG4444(c) << 8)
        | SkGetPackedB4444(c);
    d | (d << 4)
}
// Source: src/core/SkColorData.h:382-382
// Source: src/core/SkColorData.h:384-390
pub type SkPMColor4f = SkRGBA4f<true>;
pub const SK_PMColor4fTRANSPARENT: SkPMColor4f = SkRGBA4f {
    fR: 0.0,
    fG: 0.0,
    fB: 0.0,
    fA: 0.0,
};
pub const SK_PMColor4fBLACK: SkPMColor4f = SkRGBA4f {
    fR: 0.0,
    fG: 0.0,
    fB: 0.0,
    fA: 1.0,
};
pub const SK_PMColor4fWHITE: SkPMColor4f = SkRGBA4f {
    fR: 1.0,
    fG: 1.0,
    fB: 1.0,
    fA: 1.0,
};
pub const SK_PMColor4fILLEGAL: SkPMColor4f = SkRGBA4f {
    fR: f32::NEG_INFINITY,
    fG: f32::NEG_INFINITY,
    fB: f32::NEG_INFINITY,
    fA: f32::NEG_INFINITY,
};
