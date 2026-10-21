// Copyright 2006 The Android Open Source Project. BSD-style license.
// Packing follows the actual resolved SkTypes configuration, as in native.
// Source: src/core/SkColorPriv.h:29-30
// Source: src/core/SkColorPriv.h:33-34
pub fn SkAlpha255To256(alpha: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),30; @condition b"SkToU8(alpha) == alpha\0".as_ptr(); alpha<=255);
    alpha + 1
}
// Source: src/core/SkColorPriv.h:39-39
pub fn SkAlphaMul(value: u32, alpha256: u32) -> u32 {
    value.wrapping_mul(alpha256) >> 8
}
// Source: src/core/SkColorPriv.h:41-43
pub fn SkUnitScalarClampToByte(x: f32) -> u32 {
    let pinned = crate::include::private::SkTPin::SkTPin(x, 0.0, 1.0);
    (pinned * 255.0 + 0.5) as u32
}
// Source: src/core/SkColorPriv.h:45-48
// Source: src/core/SkColorPriv.h:50-53
pub const SK_A32_BITS: u32 = 8;
pub const SK_R32_BITS: u32 = 8;
pub const SK_G32_BITS: u32 = 8;
pub const SK_B32_BITS: u32 = 8;
pub const SK_A32_MASK: u32 = 255;
pub const SK_R32_MASK: u32 = 255;
pub const SK_G32_MASK: u32 = 255;
pub const SK_B32_MASK: u32 = 255;
// Source: src/core/SkColorPriv.h:67-70
// Source: src/core/SkColorPriv.h:72-75
pub const SK_RGBA_R32_SHIFT: u32 = 0;
pub const SK_RGBA_G32_SHIFT: u32 = 8;
pub const SK_RGBA_B32_SHIFT: u32 = 16;
pub const SK_RGBA_A32_SHIFT: u32 = 24;
pub const SK_BGRA_B32_SHIFT: u32 = 0;
pub const SK_BGRA_G32_SHIFT: u32 = 8;
pub const SK_BGRA_R32_SHIFT: u32 = 16;
pub const SK_BGRA_A32_SHIFT: u32 = 24;
// Source: src/core/SkColorPriv.h:77-79
// Native build-definition rejection maps to Rust build cfg definitions. C/C++
// preprocessor flags do not automatically cross that separate compiler boundary.
#[cfg(any(SK_PMCOLOR_IS_RGBA, SK_PMCOLOR_IS_BGRA))]
compile_error!("Configure PMCOLOR by setting SK_R32_SHIFT.");
// Source: src/core/SkColorPriv.h:83-87
// Macro presence becomes a typed boolean computed from the actual existing
// SkTypes packing inputs. This does not rewrite the selected pixel algorithms.
pub const SK_PMCOLOR_IS_RGBA: bool = crate::include::core::SkTypes::SK_A32_SHIFT
    == SK_RGBA_A32_SHIFT
    && crate::include::core::SkTypes::SK_R32_SHIFT == SK_RGBA_R32_SHIFT
    && crate::include::core::SkTypes::SK_G32_SHIFT == SK_RGBA_G32_SHIFT
    && crate::include::core::SkTypes::SK_B32_SHIFT == SK_RGBA_B32_SHIFT;
// Source: src/core/SkColorPriv.h:88-92
pub const SK_PMCOLOR_IS_BGRA: bool = !SK_PMCOLOR_IS_RGBA
    && crate::include::core::SkTypes::SK_A32_SHIFT == SK_BGRA_A32_SHIFT
    && crate::include::core::SkTypes::SK_R32_SHIFT == SK_BGRA_R32_SHIFT
    && crate::include::core::SkTypes::SK_G32_SHIFT == SK_BGRA_G32_SHIFT
    && crate::include::core::SkTypes::SK_B32_SHIFT == SK_BGRA_B32_SHIFT;
// Source: src/core/SkColorPriv.h:93-95
const _: () = assert!(
    SK_PMCOLOR_IS_RGBA || SK_PMCOLOR_IS_BGRA,
    "need 32bit packing to be either RGBA or BGRA"
);
// Source: src/core/SkColorPriv.h:97-100
pub const fn SkGetPackedA32(c: u32) -> u32 {
    (c << (24 - crate::include::core::SkTypes::SK_A32_SHIFT)) >> 24
}
pub const fn SkGetPackedR32(c: u32) -> u32 {
    (c << (24 - crate::include::core::SkTypes::SK_R32_SHIFT)) >> 24
}
pub const fn SkGetPackedG32(c: u32) -> u32 {
    (c << (24 - crate::include::core::SkTypes::SK_G32_SHIFT)) >> 24
}
pub const fn SkGetPackedB32(c: u32) -> u32 {
    (c << (24 - crate::include::core::SkTypes::SK_B32_SHIFT)) >> 24
}
// Source: src/core/SkColorPriv.h:102-105
pub fn SkA32Assert(a: u32) {
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),102; @condition b"(unsigned)(a) <= SK_A32_MASK\0".as_ptr(); a<=SK_A32_MASK)
}
pub fn SkR32Assert(r: u32) {
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),103; @condition b"(unsigned)(r) <= SK_R32_MASK\0".as_ptr(); r<=SK_R32_MASK)
}
pub fn SkG32Assert(g: u32) {
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),104; @condition b"(unsigned)(g) <= SK_G32_MASK\0".as_ptr(); g<=SK_G32_MASK)
}
pub fn SkB32Assert(b: u32) {
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),105; @condition b"(unsigned)(b) <= SK_B32_MASK\0".as_ptr(); b<=SK_B32_MASK)
}
// Source: src/core/SkColorPriv.h:110-114
// Source: src/core/SkColorPriv.h:116-118
pub fn SkPackARGB32(a: u32, r: u32, g: u32, b: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),111; @condition b"(unsigned)(a) <= SK_A32_MASK\0".as_ptr(); a<=SK_A32_MASK);
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),112; @condition b"(unsigned)(r) <= SK_R32_MASK\0".as_ptr(); r<=SK_R32_MASK);
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),113; @condition b"(unsigned)(g) <= SK_G32_MASK\0".as_ptr(); g<=SK_G32_MASK);
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),114; @condition b"(unsigned)(b) <= SK_B32_MASK\0".as_ptr(); b<=SK_B32_MASK);
    (a << crate::include::core::SkTypes::SK_A32_SHIFT)
        | (r << crate::include::core::SkTypes::SK_R32_SHIFT)
        | (g << crate::include::core::SkTypes::SK_G32_SHIFT)
        | (b << crate::include::core::SkTypes::SK_B32_SHIFT)
}
// Source: src/core/SkColorPriv.h:120-125
// Source: src/core/SkColorPriv.h:127-133
pub fn SkPremultiplyARGBInline(a: u32, mut r: u32, mut g: u32, mut b: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),122; @condition b"(unsigned)(a) <= SK_A32_MASK\0".as_ptr(); a<=SK_A32_MASK);
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),123; @condition b"(unsigned)(r) <= SK_R32_MASK\0".as_ptr(); r<=SK_R32_MASK);
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),124; @condition b"(unsigned)(g) <= SK_G32_MASK\0".as_ptr(); g<=SK_G32_MASK);
    crate::SkASSERT!(@source b"src/core/SkColorPriv.h\0".as_ptr(),125; @condition b"(unsigned)(b) <= SK_B32_MASK\0".as_ptr(); b<=SK_B32_MASK);
    if a != 255 {
        r = crate::include::private::SkMath::SkMulDiv255Round(r, a);
        g = crate::include::private::SkMath::SkMulDiv255Round(g, a);
        b = crate::include::private::SkMath::SkMulDiv255Round(b, a);
    }
    SkPackARGB32(a, r, g, b)
}
// Source: src/core/SkColorPriv.h:137-138
// Source: src/core/SkColorPriv.h:140-143
#[inline(always)]
pub fn SkAlphaMulQ(c: u32, scale: u32) -> u32 {
    const MASK: u32 = 0x00ff00ff;
    let rb = (c & MASK).wrapping_mul(scale) >> 8;
    let ag = ((c >> 8) & MASK).wrapping_mul(scale);
    (rb & MASK) | (ag & !MASK)
}
// Source: src/core/SkColorPriv.h:145-146
// Source: src/core/SkColorPriv.h:148-150
// Source: src/core/SkColorPriv.h:152-153
// Source: src/core/SkColorPriv.h:156-160
pub fn SkPMSrcOver(src: u32, dst: u32) -> u32 {
    let scale = SkAlpha255To256(255 - SkGetPackedA32(src));
    const MASK: u32 = 0x00ff00ff;
    let mut rb = ((dst & MASK).wrapping_mul(scale) >> 8) & MASK;
    let mut ag = ((dst >> 8) & MASK).wrapping_mul(scale) & !MASK;
    rb = rb.wrapping_add(src & MASK);
    ag = ag.wrapping_add(src & !MASK);
    (rb & 0x000001ff).min(0xff)
        | (ag & 0x0001ff00).min(0xff00)
        | (rb & 0x01ff0000).min(0xff0000)
        | (ag & 0xff000000)
}

// Source: src/core/SkColorPriv.h:162-162
// Source: src/core/SkColorPriv.h:164-164
// Source: src/core/SkColorPriv.h:166-166
// Source: src/core/SkColorPriv.h:168-170
// Vec replaces STArray storage without changing ownership or component values.
pub struct SkColorConverter {
    pub(crate) fColors4f: Vec<super::SkColor::SkColor4f>,
}
impl SkColorConverter {
    pub fn colors4f(&mut self) -> &mut [super::SkColor::SkColor4f] {
        &mut self.fColors4f
    }
}
