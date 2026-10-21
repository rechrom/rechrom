// Source: src/core/SkCoreBlitters.h:60-67
// Source: src/core/SkCoreBlitters.h:81-85
// Source: src/core/SkCoreBlitters.h:94-96
// Native method declarations are expressed by the concrete inherent methods and
// SkBlitter implementations below; Rust composition replaces virtual inheritance.
// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::src::core::SkColorData::{
    SkGetPackedB16, SkGetPackedG16, SkGetPackedR16, SK_B16_BITS, SK_G16_BITS, SK_R16_BITS,
};
use crate::src::core::SkColorPriv::{
    SkGetPackedA32, SkGetPackedB32, SkGetPackedG32, SkGetPackedR32, SkPackARGB32,
};
// Source: src/core/SkBlitter_ARGB32.cpp:36-39
fn upscale_31_to_32(value: i32) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 37; @condition b"(unsigned)value <= 31\0".as_ptr(); value as u32 <= 31);
    value + (value >> 4)
}
// Source: src/core/SkBlitter_ARGB32.cpp:41-46
fn blend_32(src: i32, dst: i32, scale: i32) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 42; @condition b"(unsigned)src <= 0xFF\0".as_ptr(); src as u32 <= 255);
    crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 43; @condition b"(unsigned)dst <= 0xFF\0".as_ptr(); dst as u32 <= 255);
    crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 44; @condition b"(unsigned)scale <= 32\0".as_ptr(); scale as u32 <= 32);
    dst + ((src - dst) * scale >> 5)
}
// Source: src/core/SkBlitter_ARGB32.cpp:48-85
pub fn blend_lcd16(srcA: i32, srcR: i32, srcG: i32, srcB: i32, dst: u32, mask: u16) -> u32 {
    if mask == 0 {
        return dst;
    }
    let mut maskR = (SkGetPackedR16(mask as u32) >> (SK_R16_BITS - 5)) as i32;
    let mut maskG = (SkGetPackedG16(mask as u32) >> (SK_G16_BITS - 5)) as i32;
    let mut maskB = (SkGetPackedB16(mask as u32) >> (SK_B16_BITS - 5)) as i32;
    maskR = upscale_31_to_32(maskR);
    maskG = upscale_31_to_32(maskG);
    maskB = upscale_31_to_32(maskB);
    maskR = maskR * srcA >> 8;
    maskG = maskG * srcA >> 8;
    maskB = maskB * srcA >> 8;
    let dstA = SkGetPackedA32(dst) as i32;
    let dstR = SkGetPackedR32(dst) as i32;
    let dstG = SkGetPackedG32(dst) as i32;
    let dstB = SkGetPackedB32(dst) as i32;
    let maskA = if srcA - 1 < dstA {
        maskR.min(maskG.min(maskB))
    } else {
        maskR.max(maskG.max(maskB))
    };
    SkPackARGB32(
        blend_32(255, dstA, maskA) as u32,
        blend_32(srcR, dstR, maskR) as u32,
        blend_32(srcG, dstG, maskG) as u32,
        blend_32(srcB, dstB, maskB) as u32,
    )
}
// Source: src/core/SkBlitter_ARGB32.cpp:87-124
pub fn blend_lcd16_opaque(
    srcR: i32,
    srcG: i32,
    srcB: i32,
    dst: u32,
    mask: u16,
    opaqueDst: u32,
) -> u32 {
    if mask == 0 {
        return dst;
    }
    if mask == 65535 {
        return opaqueDst;
    }
    let mut maskR = (SkGetPackedR16(mask as u32) >> (SK_R16_BITS - 5)) as i32;
    let mut maskG = (SkGetPackedG16(mask as u32) >> (SK_G16_BITS - 5)) as i32;
    let mut maskB = (SkGetPackedB16(mask as u32) >> (SK_B16_BITS - 5)) as i32;
    maskR = upscale_31_to_32(maskR);
    maskG = upscale_31_to_32(maskG);
    maskB = upscale_31_to_32(maskB);
    let dstA = SkGetPackedA32(dst) as i32;
    let dstR = SkGetPackedR32(dst) as i32;
    let dstG = SkGetPackedG32(dst) as i32;
    let dstB = SkGetPackedB32(dst) as i32;
    let maskA = maskR.max(maskG.max(maskB));
    SkPackARGB32(
        blend_32(255, dstA, maskA) as u32,
        blend_32(srcR, dstR, maskR) as u32,
        blend_32(srcG, dstG, maskG) as u32,
        blend_32(srcB, dstB, maskB) as u32,
    )
}

#[cfg(target_arch = "x86_64")]
pub mod sse2 {
    use super::*;
    use crate::src::core::SkColor::{SkColorGetA, SkColorGetB, SkColorGetG, SkColorGetR};
    use crate::src::core::SkColorData::{SK_B16_SHIFT, SK_G16_SHIFT, SK_R16_SHIFT};
    use crate::src::core::SkColorPriv::{
        SK_A32_MASK, SK_BGRA_A32_SHIFT as SK_A32_SHIFT, SK_BGRA_B32_SHIFT as SK_B32_SHIFT,
        SK_BGRA_G32_SHIFT as SK_G32_SHIFT, SK_BGRA_R32_SHIFT as SK_R32_SHIFT,
    };
    use core::arch::x86_64::*;
    // Source: src/core/SkBlitter_ARGB32.cpp:135-137
    const R_SHIFT: i32 = SK_R32_SHIFT as i32 - SK_R16_SHIFT as i32 - SK_R16_BITS as i32 + 5;
    const G_SHIFT: i32 = SK_G32_SHIFT as i32 - SK_G16_SHIFT as i32 - SK_G16_BITS as i32 + 5;
    const B_SHIFT: i32 = SK_B32_SHIFT as i32 - SK_B16_SHIFT as i32 - SK_B16_BITS as i32 + 5;
    // Source: src/core/SkBlitter_ARGB32.cpp:139-145
    // Current canonical N32 is BGRA, so the native positive-shift arm is selected.
    #[target_feature(enable = "sse2")]
    unsafe fn packed_r(mask: __m128i) -> __m128i {
        _mm_slli_epi32::<R_SHIFT>(mask)
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:147-153
    #[target_feature(enable = "sse2")]
    unsafe fn packed_g(mask: __m128i) -> __m128i {
        _mm_slli_epi32::<G_SHIFT>(mask)
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:155-161
    // Blue's canonical shift is zero, selecting the original identity macro.
    #[target_feature(enable = "sse2")]
    unsafe fn packed_b(mask: __m128i) -> __m128i {
        debug_assert_eq!(B_SHIFT, 0);
        mask
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:163-263
    #[target_feature(enable = "sse2")]
    unsafe fn blend_lcd16_sse2(
        src: __m128i,
        dst: __m128i,
        mut mask: __m128i,
        srcA: __m128i,
    ) -> __m128i {
        let r = _mm_and_si128(
            packed_r(mask),
            _mm_set1_epi32((0x1fu32 << SK_R32_SHIFT) as i32),
        );
        let g = _mm_and_si128(
            packed_g(mask),
            _mm_set1_epi32((0x1fu32 << SK_G32_SHIFT) as i32),
        );
        let b = _mm_and_si128(
            packed_b(mask),
            _mm_set1_epi32((0x1fu32 << SK_B32_SHIFT) as i32),
        );
        let aMin = _mm_min_epu8(
            _mm_slli_epi32::<{ (SK_A32_SHIFT - SK_R32_SHIFT) as i32 }>(r),
            _mm_min_epu8(
                _mm_slli_epi32::<{ (SK_A32_SHIFT - SK_G32_SHIFT) as i32 }>(g),
                _mm_slli_epi32::<{ (SK_A32_SHIFT - SK_B32_SHIFT) as i32 }>(b),
            ),
        );
        let aMax = _mm_max_epu8(
            _mm_slli_epi32::<{ (SK_A32_SHIFT - SK_R32_SHIFT) as i32 }>(r),
            _mm_max_epu8(
                _mm_slli_epi32::<{ (SK_A32_SHIFT - SK_G32_SHIFT) as i32 }>(g),
                _mm_slli_epi32::<{ (SK_A32_SHIFT - SK_B32_SHIFT) as i32 }>(b),
            ),
        );
        // Preserve the native signed dword comparison and unshifted A32_MASK
        // literally; do not normalize this architecture's existing alpha rule.
        let mut a = _mm_cmplt_epi32(
            srcA,
            _mm_and_si128(
                _mm_add_epi32(dst, _mm_set1_epi32((1u32 << SK_A32_SHIFT) as i32)),
                _mm_set1_epi32(SK_A32_MASK as i32),
            ),
        );
        a = _mm_or_si128(_mm_and_si128(a, aMin), _mm_andnot_si128(a, aMax));
        mask = _mm_or_si128(_mm_or_si128(a, r), _mm_or_si128(g, b));
        let mut maskLo = _mm_unpacklo_epi8(mask, _mm_setzero_si128());
        let mut maskHi = _mm_unpackhi_epi8(mask, _mm_setzero_si128());
        maskLo = _mm_add_epi16(maskLo, _mm_srli_epi16::<4>(maskLo));
        maskHi = _mm_add_epi16(maskHi, _mm_srli_epi16::<4>(maskHi));
        maskLo = _mm_mullo_epi16(maskLo, srcA);
        maskHi = _mm_mullo_epi16(maskHi, srcA);
        maskLo = _mm_srli_epi16::<8>(maskLo);
        maskHi = _mm_srli_epi16::<8>(maskHi);
        let dstLo = _mm_unpacklo_epi8(dst, _mm_setzero_si128());
        let dstHi = _mm_unpackhi_epi8(dst, _mm_setzero_si128());
        maskLo = _mm_mullo_epi16(maskLo, _mm_sub_epi16(src, dstLo));
        maskHi = _mm_mullo_epi16(maskHi, _mm_sub_epi16(src, dstHi));
        maskLo = _mm_srai_epi16::<5>(maskLo);
        maskHi = _mm_srai_epi16::<5>(maskHi);
        let resultLo = _mm_add_epi16(dstLo, maskLo);
        let resultHi = _mm_add_epi16(dstHi, maskHi);
        _mm_packus_epi16(resultLo, resultHi)
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:265-343
    #[target_feature(enable = "sse2")]
    unsafe fn blend_lcd16_opaque_sse2(src: __m128i, dst: __m128i, mut mask: __m128i) -> __m128i {
        let r = _mm_and_si128(
            packed_r(mask),
            _mm_set1_epi32((0x1fu32 << SK_R32_SHIFT) as i32),
        );
        let g = _mm_and_si128(
            packed_g(mask),
            _mm_set1_epi32((0x1fu32 << SK_G32_SHIFT) as i32),
        );
        let b = _mm_and_si128(
            packed_b(mask),
            _mm_set1_epi32((0x1fu32 << SK_B32_SHIFT) as i32),
        );
        let a = _mm_max_epu8(
            _mm_slli_epi32::<{ (SK_A32_SHIFT - SK_R32_SHIFT) as i32 }>(r),
            _mm_max_epu8(
                _mm_slli_epi32::<{ (SK_A32_SHIFT - SK_G32_SHIFT) as i32 }>(g),
                _mm_slli_epi32::<{ (SK_A32_SHIFT - SK_B32_SHIFT) as i32 }>(b),
            ),
        );
        mask = _mm_or_si128(_mm_or_si128(a, r), _mm_or_si128(g, b));
        let mut maskLo = _mm_unpacklo_epi8(mask, _mm_setzero_si128());
        let mut maskHi = _mm_unpackhi_epi8(mask, _mm_setzero_si128());
        maskLo = _mm_add_epi16(maskLo, _mm_srli_epi16::<4>(maskLo));
        maskHi = _mm_add_epi16(maskHi, _mm_srli_epi16::<4>(maskHi));
        let dstLo = _mm_unpacklo_epi8(dst, _mm_setzero_si128());
        let dstHi = _mm_unpackhi_epi8(dst, _mm_setzero_si128());
        maskLo = _mm_mullo_epi16(maskLo, _mm_sub_epi16(src, dstLo));
        maskHi = _mm_mullo_epi16(maskHi, _mm_sub_epi16(src, dstHi));
        maskLo = _mm_srai_epi16::<5>(maskLo);
        maskHi = _mm_srai_epi16::<5>(maskHi);
        let resultLo = _mm_add_epi16(dstLo, maskLo);
        let resultHi = _mm_add_epi16(dstHi, maskHi);
        _mm_packus_epi16(resultLo, resultHi)
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:345-414
    /// Safety: SSE2 is available and dst/mask cover width initialized pixels;
    /// dst is four-byte aligned and its writable storage does not alias masks.
    #[target_feature(enable = "sse2")]
    pub unsafe fn blit_row_lcd16(
        mut dst: *mut u32,
        mut mask: *const u16,
        src: u32,
        mut width: i32,
        _: u32,
    ) {
        if width <= 0 {
            return;
        }
        let mut srcA = SkColorGetA(src) as i32;
        let srcR = SkColorGetR(src) as i32;
        let srcG = SkColorGetG(src) as i32;
        let srcB = SkColorGetB(src) as i32;
        srcA = crate::src::core::SkColorPriv::SkAlpha255To256(srcA as u32) as i32;
        if width >= 4 {
            crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 358; @condition b"SkIsAlign4((uintptr_t) dst)\0".as_ptr(); (dst as usize & 3) == ( 0));
            while dst as usize & 15 != 0 {
                *dst = blend_lcd16(srcA, srcR, srcG, srcB, *dst, *mask);
                mask = mask.add(1);
                dst = dst.add(1);
                width -= 1;
            }
            let mut d = dst.cast::<__m128i>();
            let mut src_sse =
                _mm_set1_epi32(SkPackARGB32(255, srcR as u32, srcG as u32, srcB as u32) as i32);
            src_sse = _mm_unpacklo_epi8(src_sse, _mm_setzero_si128());
            let srcA_sse = _mm_set1_epi16(srcA as i16);
            while width >= 4 {
                let dst_sse = _mm_load_si128(d);
                let mut mask_sse = _mm_loadl_epi64(mask.cast::<__m128i>());
                let pack_cmp = _mm_movemask_epi8(_mm_cmpeq_epi16(mask_sse, _mm_setzero_si128()));
                if pack_cmp != 0xffff {
                    mask_sse = _mm_unpacklo_epi16(mask_sse, _mm_setzero_si128());
                    let result = blend_lcd16_sse2(src_sse, dst_sse, mask_sse, srcA_sse);
                    _mm_store_si128(d, result);
                }
                d = d.add(1);
                mask = mask.add(4);
                width -= 4;
            }
            dst = d.cast();
        }
        while width > 0 {
            *dst = blend_lcd16(srcA, srcR, srcG, srcB, *dst, *mask);
            mask = mask.add(1);
            dst = dst.add(1);
            width -= 1;
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:416-481
    /// Safety: same pixel/mask storage and SSE2 contract as blit_row_lcd16.
    #[target_feature(enable = "sse2")]
    pub unsafe fn blit_row_lcd16_opaque(
        mut dst: *mut u32,
        mut mask: *const u16,
        src: u32,
        mut width: i32,
        opaqueDst: u32,
    ) {
        if width <= 0 {
            return;
        }
        let srcR = SkColorGetR(src) as i32;
        let srcG = SkColorGetG(src) as i32;
        let srcB = SkColorGetB(src) as i32;
        if width >= 4 {
            crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 427; @condition b"SkIsAlign4((uintptr_t) dst)\0".as_ptr(); (dst as usize & 3) == ( 0));
            while dst as usize & 15 != 0 {
                *dst = blend_lcd16_opaque(srcR, srcG, srcB, *dst, *mask, opaqueDst);
                mask = mask.add(1);
                dst = dst.add(1);
                width -= 1;
            }
            let mut d = dst.cast::<__m128i>();
            let mut src_sse =
                _mm_set1_epi32(SkPackARGB32(255, srcR as u32, srcG as u32, srcB as u32) as i32);
            src_sse = _mm_unpacklo_epi8(src_sse, _mm_setzero_si128());
            while width >= 4 {
                let dst_sse = _mm_load_si128(d);
                let mut mask_sse = _mm_loadl_epi64(mask.cast::<__m128i>());
                let pack_cmp = _mm_movemask_epi8(_mm_cmpeq_epi16(mask_sse, _mm_setzero_si128()));
                if pack_cmp != 0xffff {
                    mask_sse = _mm_unpacklo_epi16(mask_sse, _mm_setzero_si128());
                    let result = blend_lcd16_opaque_sse2(src_sse, dst_sse, mask_sse);
                    _mm_store_si128(d, result);
                }
                d = d.add(1);
                mask = mask.add(4);
                width -= 4;
            }
            dst = d.cast();
        }
        while width > 0 {
            *dst = blend_lcd16_opaque(srcR, srcG, srcB, *dst, *mask, opaqueDst);
            mask = mask.add(1);
            dst = dst.add(1);
            width -= 1;
        }
    }
}

#[cfg(target_arch = "aarch64")]
pub mod neon {
    use super::*;
    use crate::src::core::SkColor::{SkColorGetA, SkColorGetB, SkColorGetG, SkColorGetR};
    use std::arch::aarch64::*;
    // Source: src/core/SkBlitter_ARGB32.cpp:486-489
    const NEON_A: usize = crate::src::core::SkColorPriv::SK_BGRA_A32_SHIFT as usize / 8;
    const NEON_R: usize = crate::src::core::SkColorPriv::SK_BGRA_R32_SHIFT as usize / 8;
    const NEON_G: usize = crate::src::core::SkColorPriv::SK_BGRA_G32_SHIFT as usize / 8;
    const NEON_B: usize = crate::src::core::SkColorPriv::SK_BGRA_B32_SHIFT as usize / 8;
    // Source: src/core/SkBlitter_ARGB32.cpp:491-502
    unsafe fn blend_32_neon(src: uint8x8_t, dst: uint8x8_t, scale: uint16x8_t) -> uint8x8_t {
        let mut src_wide = vreinterpretq_s16_u16(vmovl_u8(src));
        let mut dst_wide = vreinterpretq_s16_u16(vmovl_u8(dst));
        src_wide = vmulq_s16(vsubq_s16(src_wide, dst_wide), vreinterpretq_s16_u16(scale));
        dst_wide = vaddq_s16(dst_wide, vshrq_n_s16::<5>(src_wide));
        vmovn_u16(vreinterpretq_u16_s16(dst_wide))
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:504-553
    /// Safety: dst/src address width live initialized pixels/masks, disjoint.
    pub unsafe fn blit_row_lcd16_opaque(
        mut dst: *mut u32,
        mut src: *const u16,
        color: u32,
        mut width: i32,
        opaqueDst: u32,
    ) {
        let colR = SkColorGetR(color) as i32;
        let colG = SkColorGetG(color) as i32;
        let colB = SkColorGetB(color) as i32;
        let vcolA = vdup_n_u8(255);
        let vcolR = vdup_n_u8(colR as u8);
        let vcolG = vdup_n_u8(colG as u8);
        let vcolB = vdup_n_u8(colB as u8);
        while width >= 8 {
            let vdst = vld4_u8(dst.cast());
            let mut vdst = [vdst.0, vdst.1, vdst.2, vdst.3];
            let vmask = vld1q_u16(src);
            let mut vmaskR =
                vshrq_n_u16::<{ crate::src::core::SkColorData::SK_R16_SHIFT as i32 }>(vmask);
            let mut vmaskG =
                vshrq_n_u16::<{ (SK_B16_BITS + SK_R16_BITS + 1) as i32 }>(vshlq_n_u16::<
                    { SK_R16_BITS as i32 },
                >(vmask));
            let mut vmaskB = vandq_u16(
                vmask,
                vdupq_n_u16(crate::src::core::SkColorData::SK_B16_MASK as u16),
            );
            vmaskR = vaddq_u16(vmaskR, vshrq_n_u16::<4>(vmaskR));
            vmaskG = vaddq_u16(vmaskG, vshrq_n_u16::<4>(vmaskG));
            vmaskB = vaddq_u16(vmaskB, vshrq_n_u16::<4>(vmaskB));
            let vmaskA = vmaxq_u16(vmaskR, vmaxq_u16(vmaskG, vmaskB));
            vdst[NEON_R] = blend_32_neon(vcolR, vdst[NEON_R], vmaskR);
            vdst[NEON_G] = blend_32_neon(vcolG, vdst[NEON_G], vmaskG);
            vdst[NEON_B] = blend_32_neon(vcolB, vdst[NEON_B], vmaskB);
            vdst[NEON_A] = blend_32_neon(vcolA, vdst[NEON_A], vmaskA);
            vst4_u8(dst.cast(), uint8x8x4_t(vdst[0], vdst[1], vdst[2], vdst[3]));
            dst = dst.add(8);
            src = src.add(8);
            width -= 8;
        }
        for i in 0..width {
            let i = i as usize;
            *dst.add(i) = blend_lcd16_opaque(colR, colG, colB, *dst.add(i), *src.add(i), opaqueDst);
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:555-615
    /// Safety: dst/src address width live initialized pixels/masks, disjoint.
    pub unsafe fn blit_row_lcd16(
        mut dst: *mut u32,
        mut src: *const u16,
        color: u32,
        mut width: i32,
        _opaqueDst: u32,
    ) {
        let mut colA = SkColorGetA(color) as i32;
        let colR = SkColorGetR(color) as i32;
        let colG = SkColorGetG(color) as i32;
        let colB = SkColorGetB(color) as i32;
        let vcolACmp = vdupq_n_u16(colA as u16);
        colA = crate::src::core::SkColorPriv::SkAlpha255To256(colA as u32) as i32;
        let vcolA = vdupq_n_u16(colA as u16);
        let vcolR = vdup_n_u8(colR as u8);
        let vcolG = vdup_n_u8(colG as u8);
        let vcolB = vdup_n_u8(colB as u8);
        while width >= 8 {
            let vdst = vld4_u8(dst.cast());
            let mut vdst = [vdst.0, vdst.1, vdst.2, vdst.3];
            let vmask = vld1q_u16(src);
            let mut vmaskR =
                vshrq_n_u16::<{ crate::src::core::SkColorData::SK_R16_SHIFT as i32 }>(vmask);
            let mut vmaskG =
                vshrq_n_u16::<{ (SK_B16_BITS + SK_R16_BITS + 1) as i32 }>(vshlq_n_u16::<
                    { SK_R16_BITS as i32 },
                >(vmask));
            let mut vmaskB = vandq_u16(
                vmask,
                vdupq_n_u16(crate::src::core::SkColorData::SK_B16_MASK as u16),
            );
            vmaskR = vaddq_u16(vmaskR, vshrq_n_u16::<4>(vmaskR));
            vmaskG = vaddq_u16(vmaskG, vshrq_n_u16::<4>(vmaskG));
            vmaskB = vaddq_u16(vmaskB, vshrq_n_u16::<4>(vmaskB));
            vmaskR = vshrq_n_u16::<8>(vmulq_u16(vmaskR, vcolA));
            vmaskG = vshrq_n_u16::<8>(vmulq_u16(vmaskG, vcolA));
            vmaskB = vshrq_n_u16::<8>(vmulq_u16(vmaskB, vcolA));
            let vmaskA = vbslq_u16(
                vcleq_u16(vcolACmp, vmovl_u8(vdst[NEON_A])),
                vminq_u16(vmaskR, vminq_u16(vmaskG, vmaskB)),
                vmaxq_u16(vmaskR, vmaxq_u16(vmaskG, vmaskB)),
            );
            vdst[NEON_R] = blend_32_neon(vcolR, vdst[NEON_R], vmaskR);
            vdst[NEON_G] = blend_32_neon(vcolG, vdst[NEON_G], vmaskG);
            vdst[NEON_B] = blend_32_neon(vcolB, vdst[NEON_B], vmaskB);
            vdst[NEON_A] = blend_32_neon(vdup_n_u8(255), vdst[NEON_A], vmaskA);
            vst4_u8(dst.cast(), uint8x8x4_t(vdst[0], vdst[1], vdst[2], vdst[3]));
            dst = dst.add(8);
            src = src.add(8);
            width -= 8;
        }
        for i in 0..width {
            let i = i as usize;
            *dst.add(i) = blend_lcd16(colA, colR, colG, colB, *dst.add(i), *src.add(i));
        }
    }
}

// Source: src/core/SkBlitter_ARGB32.cpp:1356-1368
// Native scalar fallback; architecture-specific vector branches remain pending.
pub fn blit_row_lcd16(dst: &mut [u32], mask: &[u16], src: u32, width: i32, _opaqueDst: u32) {
    use crate::src::core::SkColor::{SkColorGetA, SkColorGetB, SkColorGetG, SkColorGetR};
    let mut srcA = SkColorGetA(src) as i32;
    let srcR = SkColorGetR(src) as i32;
    let srcG = SkColorGetG(src) as i32;
    let srcB = SkColorGetB(src) as i32;
    srcA = crate::src::core::SkColorPriv::SkAlpha255To256(srcA as u32) as i32;
    for i in 0..width {
        let i = i as usize;
        dst[i] = blend_lcd16(srcA, srcR, srcG, srcB, dst[i], mask[i]);
    }
}
// Source: src/core/SkBlitter_ARGB32.cpp:1370-1380
pub fn blit_row_lcd16_opaque(dst: &mut [u32], mask: &[u16], src: u32, width: i32, opaqueDst: u32) {
    use crate::src::core::SkColor::{SkColorGetB, SkColorGetG, SkColorGetR};
    let srcR = SkColorGetR(src) as i32;
    let srcG = SkColorGetG(src) as i32;
    let srcB = SkColorGetB(src) as i32;
    for i in 0..width {
        let i = i as usize;
        dst[i] = blend_lcd16_opaque(srcR, srcG, srcB, dst[i], mask[i], opaqueDst);
    }
}

// Source: src/core/SkBlitter_ARGB32.cpp:1384-1420
unsafe fn blit_color(
    device: &crate::src::core::SkPixmap::SkPixmap,
    mask: &crate::src::core::SkMask::SkMask,
    clip: &crate::src::core::SkRect::SkIRect,
    color: u32,
) -> bool {
    use crate::src::core::SkColor::{SkColorGetA, SkPreMultiplyColor};
    use crate::src::core::SkMask::Format;
    let x = clip.fLeft;
    let y = clip.fTop;
    if device.colorType() == crate::include::core::SkColorType::kN32_SkColorType
        && mask.fFormat == Format::kA8_Format
    {
        crate::src::core::SkBlitMask::blit_mask_d32_a8(
            device.writable_addr32(x, y),
            device.rowBytes(),
            mask.getAddr(x, y).cast(),
            mask.fRowBytes as usize,
            color,
            clip.width(),
            clip.height(),
        );
        return true;
    }
    if device.colorType() == crate::include::core::SkColorType::kN32_SkColorType
        && mask.fFormat == Format::kLCD16_Format
    {
        let mut dstRow = device.writable_addr32(x, y);
        let mut maskRow = mask.getAddr(x, y).cast::<u16>();
        let mut opaqueDst = 0;
        #[cfg(target_arch = "aarch64")]
        let mut row: unsafe fn(*mut u32, *const u16, u32, i32, u32) = neon::blit_row_lcd16;
        #[cfg(target_arch = "x86_64")]
        let mut row: unsafe fn(*mut u32, *const u16, u32, i32, u32) = sse2::blit_row_lcd16;
        #[cfg(target_arch = "loongarch64")]
        let mut row: unsafe fn(*mut u32, *const u16, u32, i32, u32) = loongarch::blit_row_lcd16;
        #[cfg(not(any(
            target_arch = "aarch64",
            target_arch = "x86_64",
            target_arch = "loongarch64"
        )))]
        let mut row: unsafe fn(*mut u32, *const u16, u32, i32, u32) = scalar_lcd_row;
        if SkColorGetA(color) == 255 {
            #[cfg(target_arch = "aarch64")]
            {
                row = neon::blit_row_lcd16_opaque;
            }
            #[cfg(target_arch = "x86_64")]
            {
                row = sse2::blit_row_lcd16_opaque;
            }
            #[cfg(target_arch = "loongarch64")]
            {
                row = loongarch::blit_row_lcd16_opaque;
            }
            #[cfg(not(any(
                target_arch = "aarch64",
                target_arch = "x86_64",
                target_arch = "loongarch64"
            )))]
            {
                row = scalar_lcd_row_opaque;
            }
            opaqueDst = SkPreMultiplyColor(color);
        }
        let mut height = clip.height();
        loop {
            let old = height;
            height -= 1;
            if old <= 0 {
                break;
            }
            row(dstRow, maskRow, color, clip.width(), opaqueDst);
            dstRow = dstRow
                .cast::<u8>()
                .wrapping_add(device.rowBytes())
                .cast::<u32>();
            maskRow = maskRow
                .cast::<u8>()
                .wrapping_add(mask.fRowBytes as usize)
                .cast::<u16>();
        }
        return true;
    }
    false
}
// Rust raw-pointer adapters for the original scalar branch; SSE/LSX native
// branch selection stays pending and is not presented as implemented.
#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "x86_64",
    target_arch = "loongarch64"
)))]
unsafe fn scalar_lcd_row(d: *mut u32, m: *const u16, c: u32, n: i32, o: u32) {
    blit_row_lcd16(
        core::slice::from_raw_parts_mut(d, n as usize),
        core::slice::from_raw_parts(m, n as usize),
        c,
        n,
        o,
    );
}
#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "x86_64",
    target_arch = "loongarch64"
)))]
unsafe fn scalar_lcd_row_opaque(d: *mut u32, m: *const u16, c: u32, n: i32, o: u32) {
    blit_row_lcd16_opaque(
        core::slice::from_raw_parts_mut(d, n as usize),
        core::slice::from_raw_parts(m, n as usize),
        c,
        n,
        o,
    );
}

// Source: src/core/SkBlitter_ARGB32.cpp:1424-1446
/// Safety: destination and mask storage must contain every addressed pixel,
/// with source and destination disjoint as required by the native row proc.
pub unsafe fn SkARGB32_Blit32(
    device: &crate::src::core::SkPixmap::SkPixmap,
    mask: &crate::src::core::SkMask::SkMask,
    clip: &crate::src::core::SkRect::SkIRect,
    srcColor: u32,
) {
    use crate::src::core::SkBlitRow::SkBlitRow;
    let alpha = SkGetPackedA32(srcColor);
    let mut flags = SkBlitRow::kSrcPixelAlpha_Flag32;
    if alpha != 255 {
        flags |= SkBlitRow::kGlobalAlpha_Flag32;
    }
    let proc = SkBlitRow::Factory32(flags);
    let x = clip.fLeft;
    let y = clip.fTop;
    let width = clip.width();
    let mut height = clip.height();
    let mut dstRow = device.writable_addr32(x, y);
    let mut srcRow = mask.getAddr8(x, y).cast::<u32>();
    loop {
        proc(
            core::slice::from_raw_parts_mut(dstRow, width as usize),
            core::slice::from_raw_parts(srcRow, width as usize),
            width as usize,
            alpha,
        );
        dstRow = dstRow.cast::<u8>().add(device.rowBytes()).cast::<u32>();
        srcRow = srcRow
            .cast::<u8>()
            .wrapping_add(mask.fRowBytes as usize)
            .cast::<u32>();
        height -= 1;
        if height == 0 {
            break;
        }
    }
}

// Source: src/core/SkBlitter_ARGB32.cpp:1450-1456
impl crate::src::core::SkCoreBlitters::SkARGB32_Blitter {
    /// Safety: writable initialized pixel storage must remain live for every call.
    pub unsafe fn new(
        device: &crate::src::core::SkPixmap::SkPixmap,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Self {
        let color = paint.getColor();
        Self {
            raster: crate::src::core::SkCoreBlitters::SkRasterBlitter::new(device),
            fColor: color,
            fSrcA: crate::src::core::SkColor::SkColorGetA(color) as u8,
            fPMColor: crate::src::core::SkColor::SkPreMultiplyColor(color),
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:1463-1468
    pub unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        let d = &this.raster.fDevice;
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1464; @condition b"x >= 0 && y >= 0 && x + width <= fDevice.width()\0".as_ptr(); x >= 0 && y >= 0 && x + width <= d.width());
        let device = d.writable_addr32(x, y);
        unsafe {
            crate::src::core::SkBlitRow::SkBlitRow::Color32(
                core::slice::from_raw_parts_mut(device, width as usize),
                width as usize,
                this.fPMColor,
            );
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:1470-1494
    pub unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        antialias: &[u8],
        runs: &[i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1472; @condition b"fSrcA != 0xFF\0".as_ptr(); this.fSrcA != 255);
        if this.fSrcA == 0 {
            return;
        }
        let mut device = this.raster.fDevice.writable_addr32(x, y);
        let mut offset = 0;
        loop {
            let count = runs[offset];
            crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1481; @condition b"count >= 0\0".as_ptr(); count >= 0);
            if count <= 0 {
                return;
            }
            let aa = antialias[offset];
            unsafe {
                if aa != 0 {
                    let sc = crate::src::core::SkColorPriv::SkAlphaMulQ(
                        this.fPMColor,
                        crate::src::core::SkColorPriv::SkAlpha255To256(aa as u32),
                    );
                    crate::src::core::SkBlitRow::SkBlitRow::Color32(
                        core::slice::from_raw_parts_mut(device, count as usize),
                        count as usize,
                        sc,
                    );
                }
                device = device.add(count as usize);
            }
            offset += count as usize;
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:1496-1502
    pub unsafe fn blitAntiH2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        let device = this.raster.fDevice.writable_addr32(x, y);
        #[cfg(sk_debug)]
        {
            let _ = this.raster.fDevice.writable_addr32(x + 1, y);
        };
        unsafe {
            *device = crate::src::core::SkColorData::SkBlendARGB32(this.fPMColor, *device, a0);
            *device.add(1) =
                crate::src::core::SkColorData::SkBlendARGB32(this.fPMColor, *device.add(1), a1);
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:1504-1511
    pub unsafe fn blitAntiV2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        let mut device = this.raster.fDevice.writable_addr32(x, y);
        #[cfg(sk_debug)]
        {
            let _ = this.raster.fDevice.writable_addr32(x, y + 1);
        };
        unsafe {
            *device = crate::src::core::SkColorData::SkBlendARGB32(this.fPMColor, *device, a0);
            device = device
                .cast::<u8>()
                .wrapping_add(this.raster.fDevice.rowBytes())
                .cast::<u32>();
            *device = crate::src::core::SkColorData::SkBlendARGB32(this.fPMColor, *device, a1);
        }
    }
}

// Source: src/core/SkBlitter_ARGB32.cpp:1515-1525
unsafe fn solid_8_pixels(mask: u8, dst: *mut u32, color: u32) {
    if mask & 128 != 0 {
        *dst.wrapping_add(0) = color;
    }
    if mask & 64 != 0 {
        *dst.wrapping_add(1) = color;
    }
    if mask & 32 != 0 {
        *dst.wrapping_add(2) = color;
    }
    if mask & 16 != 0 {
        *dst.wrapping_add(3) = color;
    }
    if mask & 8 != 0 {
        *dst.wrapping_add(4) = color;
    }
    if mask & 4 != 0 {
        *dst.wrapping_add(5) = color;
    }
    if mask & 2 != 0 {
        *dst.wrapping_add(6) = color;
    }
    if mask & 1 != 0 {
        *dst.wrapping_add(7) = color;
    }
}
// Source: src/core/SkBlitter_ARGB32.cpp:1527-1532
pub unsafe fn SkARGB32_BlitBW(
    device: &crate::src::core::SkPixmap::SkPixmap,
    mask: &crate::src::core::SkMask::SkMask,
    clip: &crate::src::core::SkRect::SkIRect,
    color: u32,
) {
    crate::src::core::SkBlitBWMaskTemplate::blit_bw_mask::<u32>(device, mask, clip, |m, d| {
        solid_8_pixels(m, d, color)
    });
}
// Source: src/core/SkBlitter_ARGB32.cpp:1534-1544
unsafe fn blend_8_pixels(mask: u8, dst: *mut u32, sc: u32, dst_scale: u32) {
    if mask & 128 != 0 {
        let p = dst.wrapping_add(0);
        *p = sc + crate::src::core::SkColorPriv::SkAlphaMulQ(*p, dst_scale);
    }
    if mask & 64 != 0 {
        let p = dst.wrapping_add(1);
        *p = sc + crate::src::core::SkColorPriv::SkAlphaMulQ(*p, dst_scale);
    }
    if mask & 32 != 0 {
        let p = dst.wrapping_add(2);
        *p = sc + crate::src::core::SkColorPriv::SkAlphaMulQ(*p, dst_scale);
    }
    if mask & 16 != 0 {
        let p = dst.wrapping_add(3);
        *p = sc + crate::src::core::SkColorPriv::SkAlphaMulQ(*p, dst_scale);
    }
    if mask & 8 != 0 {
        let p = dst.wrapping_add(4);
        *p = sc + crate::src::core::SkColorPriv::SkAlphaMulQ(*p, dst_scale);
    }
    if mask & 4 != 0 {
        let p = dst.wrapping_add(5);
        *p = sc + crate::src::core::SkColorPriv::SkAlphaMulQ(*p, dst_scale);
    }
    if mask & 2 != 0 {
        let p = dst.wrapping_add(6);
        *p = sc + crate::src::core::SkColorPriv::SkAlphaMulQ(*p, dst_scale);
    }
    if mask & 1 != 0 {
        let p = dst.wrapping_add(7);
        *p = sc + crate::src::core::SkColorPriv::SkAlphaMulQ(*p, dst_scale);
    }
}
// Source: src/core/SkBlitter_ARGB32.cpp:1546-1551
pub unsafe fn SkARGB32_BlendBW(
    device: &crate::src::core::SkPixmap::SkPixmap,
    mask: &crate::src::core::SkMask::SkMask,
    clip: &crate::src::core::SkRect::SkIRect,
    sc: u32,
    dst_scale: u32,
) {
    crate::src::core::SkBlitBWMaskTemplate::blit_bw_mask::<u32>(device, mask, clip, |m, d| {
        blend_8_pixels(m, d, sc, dst_scale)
    });
}

impl crate::src::core::SkCoreBlitters::SkARGB32_Blitter {
    // Source: src/core/SkBlitter_ARGB32.cpp:1553-1575
    /// Safety: mask and clipped destination storage must be live and valid.
    pub unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        use crate::src::core::SkMask::Format;
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1554; @condition b"mask.fBounds.contains(clip)\0".as_ptr(); mask.fBounds.contains_rect(clip));
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1555; @condition b"fSrcA != 0xFF\0".as_ptr(); this.fSrcA != 255);
        if this.fSrcA == 0 {
            return;
        }
        if blit_color(&this.raster.fDevice, mask, clip, this.fColor) {
            return;
        }
        match mask.fFormat {
            Format::kBW_Format => SkARGB32_BlendBW(
                &this.raster.fDevice,
                mask,
                clip,
                this.fPMColor,
                crate::src::core::SkColorPriv::SkAlpha255To256(255 - this.fSrcA as u32),
            ),
            Format::kARGB32_Format => {
                SkARGB32_Blit32(&this.raster.fDevice, mask, clip, this.fPMColor)
            }
            _ => {
                crate::SK_ABORT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1573; "Mask format not handled.")
            }
        }
    }
}
impl crate::src::core::SkCoreBlitters::SkARGB32_Opaque_Blitter {
    // Source: src/core/SkBlitter_ARGB32.cpp:1577-1596
    /// Safety: mask and clipped destination storage must be live and valid.
    pub unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        use crate::src::core::SkMask::Format;
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1579; @condition b"fSrcA == 0xFF\0".as_ptr(); this.argb.fSrcA == 255);
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1580; @condition b"mask.fBounds.contains(clip)\0".as_ptr(); mask.fBounds.contains_rect(clip));
        if blit_color(&this.argb.raster.fDevice, mask, clip, this.argb.fColor) {
            return;
        }
        match mask.fFormat {
            Format::kBW_Format => {
                SkARGB32_BlitBW(&this.argb.raster.fDevice, mask, clip, this.argb.fPMColor)
            }
            Format::kARGB32_Format => {
                SkARGB32_Blit32(&this.argb.raster.fDevice, mask, clip, this.argb.fPMColor)
            }
            _ => {
                crate::SK_ABORT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1594; "Mask format not handled.")
            }
        }
    }
}
impl crate::src::core::SkCoreBlitters::SkARGB32_Opaque_Blitter {
    // Source: src/core/SkBlitter_ARGB32.cpp:1598-1622
    pub unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        antialias: &[u8],
        runs: &[i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        use crate::src::core::SkColorPriv::{SkAlpha255To256, SkAlphaMulQ};
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1602; @condition b"fSrcA == 0xFF\0".as_ptr(); this.argb.fSrcA == 255);
        let mut device = this.argb.raster.fDevice.writable_addr32(x, y);
        let mut offset = 0;
        loop {
            let count = runs[offset];
            crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1607; @condition b"count >= 0\0".as_ptr(); count >= 0);
            if count <= 0 {
                return;
            }
            let aa = antialias[offset];
            unsafe {
                if aa == 255 {
                    crate::src::opts::SkMemset_opts::memset32(
                        core::slice::from_raw_parts_mut(device, count as usize),
                        this.argb.fPMColor,
                        count as usize,
                    );
                } else if aa > 0 {
                    let sc = SkAlphaMulQ(this.argb.fPMColor, SkAlpha255To256(aa as u32));
                    crate::src::core::SkBlitRow::SkBlitRow::Color32(
                        core::slice::from_raw_parts_mut(device, count as usize),
                        count as usize,
                        sc,
                    );
                }
                device = device.add(count as usize);
            }
            offset += count as usize;
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:1624-1630
    pub unsafe fn blitAntiH2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        let device = this.argb.raster.fDevice.writable_addr32(x, y);
        #[cfg(sk_debug)]
        {
            let _ = this.argb.raster.fDevice.writable_addr32(x + 1, y);
        };
        unsafe {
            *device = crate::src::core::SkColorData::SkFastFourByteInterp(
                this.argb.fPMColor,
                *device,
                a0,
            );
            *device.add(1) = crate::src::core::SkColorData::SkFastFourByteInterp(
                this.argb.fPMColor,
                *device.add(1),
                a1,
            );
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:1632-1639
    pub unsafe fn blitAntiV2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        let mut device = this.argb.raster.fDevice.writable_addr32(x, y);
        #[cfg(sk_debug)]
        {
            let _ = this.argb.raster.fDevice.writable_addr32(x, y + 1);
        };
        unsafe {
            *device = crate::src::core::SkColorData::SkFastFourByteInterp(
                this.argb.fPMColor,
                *device,
                a0,
            );
            device = device
                .cast::<u8>()
                .wrapping_add(this.argb.raster.fDevice.rowBytes())
                .cast::<u32>();
            *device = crate::src::core::SkColorData::SkFastFourByteInterp(
                this.argb.fPMColor,
                *device,
                a1,
            );
        }
    }
}
impl crate::src::core::SkCoreBlitters::SkARGB32_Opaque_Blitter {
    // Source: src/core/SkBlitter_ARGB32.cpp:1641-1643
    pub fn canDirectBlit(
        self: core::pin::Pin<&mut Self>,
    ) -> Option<crate::src::core::SkBlitter::DirectBlit> {
        let this = unsafe { self.get_unchecked_mut() };
        Some(crate::src::core::SkBlitter::DirectBlit {
            pm: this.argb.raster.fDevice.clone(),
            value: this.argb.fPMColor as u64,
        })
    }
}
impl crate::src::core::SkCoreBlitters::SkARGB32_Blitter {
    // Source: src/core/SkBlitter_ARGB32.cpp:1647-1665
    pub unsafe fn blitV(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        mut height: i32,
        alpha: u8,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        use crate::src::core::SkColorPriv::{SkAlpha255To256, SkAlphaMulQ};
        if alpha == 0 || this.fSrcA == 0 {
            return;
        }
        let mut device = this.raster.fDevice.writable_addr32(x, y);
        let mut color = this.fPMColor;
        if alpha != 255 {
            color = SkAlphaMulQ(color, SkAlpha255To256(alpha as u32));
        }
        let dst_scale = SkAlpha255To256(255 - SkGetPackedA32(color));
        let rowBytes = this.raster.fDevice.rowBytes();
        loop {
            height -= 1;
            if height < 0 {
                break;
            }
            unsafe {
                *device = color + SkAlphaMulQ(*device, dst_scale);
                device = device.cast::<u8>().wrapping_add(rowBytes).cast::<u32>();
            }
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:1667-1686
    pub unsafe fn blitRect(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        width: i32,
        mut height: i32,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        use crate::src::core::SkBlitRow::SkBlitRow;
        let d = &this.raster.fDevice;
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1668; @condition b"x >= 0 && y >= 0 && x + width <= fDevice.width() && y + height <= fDevice.height()\0".as_ptr(); x >= 0 && y >= 0 && x + width <= d.width() && y + height <= d.height());
        if this.fSrcA == 0 {
            return;
        }
        let mut device = d.writable_addr32(x, y);
        let rowBytes = d.rowBytes();
        unsafe {
            if SkGetPackedA32(this.fPMColor) == 255 {
                crate::src::opts::SkMemset_opts::rect_memset32_raw(
                    device,
                    this.fPMColor,
                    width as usize,
                    rowBytes,
                    height as usize,
                );
            } else {
                loop {
                    let old = height;
                    height -= 1;
                    if old <= 0 {
                        break;
                    }
                    SkBlitRow::Color32(
                        core::slice::from_raw_parts_mut(device, width as usize),
                        width as usize,
                        this.fPMColor,
                    );
                    device = device.cast::<u8>().wrapping_add(rowBytes).cast::<u32>();
                }
            }
        }
    }
}

impl crate::src::core::SkCoreBlitters::SkARGB32_Black_Blitter {
    // Source: src/core/SkBlitter_ARGB32.cpp:1693-1722
    pub unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        antialias: &[u8],
        runs: &[i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        use crate::src::core::SkColorPriv::{
            SkAlpha255To256, SkAlphaMulQ, SK_A32_MASK, SK_BGRA_A32_SHIFT,
        };
        let mut device = this.opaque.argb.raster.fDevice.writable_addr32(x, y);
        const kBlack: u32 = SK_A32_MASK << SK_BGRA_A32_SHIFT;
        let mut offset = 0;
        loop {
            let count = runs[offset];
            crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1700; @condition b"count >= 0\0".as_ptr(); count >= 0);
            if count <= 0 {
                return;
            }
            let aa = antialias[offset] as u32;
            unsafe {
                if aa != 0 {
                    if aa == 255 {
                        crate::src::opts::SkMemset_opts::memset32(
                            core::slice::from_raw_parts_mut(device, count as usize),
                            kBlack,
                            count as usize,
                        );
                    } else {
                        let src = aa << SK_BGRA_A32_SHIFT;
                        let dst_scale = SkAlpha255To256(255 - aa);
                        let mut n = count;
                        loop {
                            n -= 1;
                            *device.add(n as usize) =
                                src + SkAlphaMulQ(*device.add(n as usize), dst_scale);
                            if n <= 0 {
                                break;
                            }
                        }
                    }
                }
                device = device.add(count as usize);
            }
            offset += count as usize;
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:1724-1730
    pub unsafe fn blitAntiH2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        use crate::src::core::SkColorPriv::{SkAlphaMulQ, SK_BGRA_A32_SHIFT};
        let device = this.opaque.argb.raster.fDevice.writable_addr32(x, y);
        #[cfg(sk_debug)]
        {
            let _ = this.opaque.argb.raster.fDevice.writable_addr32(x + 1, y);
        };
        unsafe {
            *device = (a0 << SK_BGRA_A32_SHIFT) + SkAlphaMulQ(*device, 256 - a0);
            *device.add(1) = (a1 << SK_BGRA_A32_SHIFT) + SkAlphaMulQ(*device.add(1), 256 - a1);
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:1732-1740
    pub unsafe fn blitAntiV2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        use crate::src::core::SkColorPriv::{SkAlphaMulQ, SK_BGRA_A32_SHIFT};
        let mut device = this.opaque.argb.raster.fDevice.writable_addr32(x, y);
        #[cfg(sk_debug)]
        {
            let _ = this.opaque.argb.raster.fDevice.writable_addr32(x, y + 1);
        };
        unsafe {
            *device = (a0 << SK_BGRA_A32_SHIFT) + SkAlphaMulQ(*device, 256 - a0);
            device = device
                .cast::<u8>()
                .wrapping_add(this.opaque.argb.raster.fDevice.rowBytes())
                .cast::<u32>();
            *device = (a1 << SK_BGRA_A32_SHIFT) + SkAlphaMulQ(*device, 256 - a1);
        }
    }
}

impl<'shader> crate::src::core::SkCoreBlitters::SkARGB32_Shader_Blitter<'shader> {
    // Source: src/core/SkBlitter_ARGB32.cpp:1743-1762
    /// Safety: context belongs to the live caller arena and borrows stable live
    /// shader backing for every use and destruction. It must initialize every
    /// requested span with valid premultiplied pixels. All later calls require
    /// exclusive in-bounds destination access, count>0 and intact context backing.
    pub unsafe fn new(
        device: &crate::src::core::SkPixmap::SkPixmap,
        paint: &crate::src::core::SkPaint::SkPaint,
        shaderContext: *mut (dyn crate::src::shaders::SkShaderBase::Context + 'shader),
    ) -> Self {
        use crate::src::core::SkBlitRow::SkBlitRow;
        let shader =
            crate::src::core::SkCoreBlitters::SkShaderBlitter::new(device, paint, shaderContext);
        let buffer = crate::src::core::SkAutoMalloc::SkAutoMalloc::new(
            device.width() as usize * core::mem::size_of::<u32>(),
        );
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1749; @condition b"paint.isSrcOver()\0".as_ptr(); paint.isSrcOver());
        let mut flags = 0;
        if (*shaderContext).getFlags() & crate::src::shaders::SkShaderBase::kOpaqueAlpha_Flag == 0 {
            flags |= SkBlitRow::kSrcPixelAlpha_Flag32;
        }
        let proc32 = SkBlitRow::Factory32(flags);
        let proc32_blend = SkBlitRow::Factory32(flags | SkBlitRow::kGlobalAlpha_Flag32);
        let shade_directly =
            (*shaderContext).getFlags() & crate::src::shaders::SkShaderBase::kOpaqueAlpha_Flag != 0;
        Self {
            fBuffer: buffer,
            fProc32: proc32,
            fProc32Blend: proc32_blend,
            fShadeDirectlyIntoDevice: shade_directly,
            shader,
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:1764-1766
    // The real fBuffer SkAutoMalloc field's Drop performs this native sk_free;
    // it is declared before the base to preserve derived-before-base teardown.

    // Source: src/core/SkBlitter_ARGB32.cpp:1768-1780
    pub unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        let pm = &this.shader.raster.fDevice;
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1769; @condition b"x >= 0 && y >= 0 && x + width <= fDevice.width()\0".as_ptr(); x >= 0 && y >= 0 && x + width <= pm.width());
        let device = pm.writable_addr32(x, y);
        if this.fShadeDirectlyIntoDevice {
            this.shader
                .fShaderContext
                .as_mut()
                .shadeSpan(x, y, device, width);
        } else {
            let span = this.fBuffer.get().cast::<u32>();
            this.shader
                .fShaderContext
                .as_mut()
                .shadeSpan(x, y, span, width);
            (this.fProc32)(
                core::slice::from_raw_parts_mut(device, width as usize),
                core::slice::from_raw_parts(span, width as usize),
                width as usize,
                255,
            );
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:1782-1806
    pub unsafe fn blitRect(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        mut y: i32,
        width: i32,
        mut height: i32,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let pm = &this.shader.raster.fDevice;
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1783; @condition b"x >= 0 && y >= 0 && x + width <= fDevice.width() && y + height <= fDevice.height()\0".as_ptr(); x >= 0 && y >= 0 && x + width <= pm.width() && y + height <= pm.height());
        let mut device = pm.writable_addr32(x, y);
        let deviceRB = pm.rowBytes();
        let mut context = this.shader.fShaderContext;
        let span = this.fBuffer.get().cast::<u32>();
        if this.fShadeDirectlyIntoDevice {
            loop {
                context.as_mut().shadeSpan(x, y, device, width);
                y += 1;
                device = device.cast::<u8>().wrapping_add(deviceRB).cast();
                height -= 1;
                if height <= 0 {
                    break;
                }
            }
        } else {
            let proc = this.fProc32;
            loop {
                context.as_mut().shadeSpan(x, y, span, width);
                proc(
                    core::slice::from_raw_parts_mut(device, width as usize),
                    core::slice::from_raw_parts(span, width as usize),
                    width as usize,
                    255,
                );
                y += 1;
                device = device.cast::<u8>().wrapping_add(deviceRB).cast();
                height -= 1;
                if height <= 0 {
                    break;
                }
            }
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:1808-1856
    pub unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        mut x: i32,
        y: i32,
        aa: &[u8],
        runs: &[i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let span = this.fBuffer.get().cast::<u32>();
        let mut device = this.shader.raster.fDevice.writable_addr32(x, y);
        let mut context = this.shader.fShaderContext;
        let mut offset = 0usize;
        if this.fShadeDirectlyIntoDevice
            || context.as_ref().getFlags() & crate::src::shaders::SkShaderBase::kOpaqueAlpha_Flag
                != 0
        {
            loop {
                let count = runs[offset];
                if count <= 0 {
                    break;
                }
                let alpha = aa[offset] as u32;
                if alpha != 0 {
                    if alpha == 255 {
                        context.as_mut().shadeSpan(x, y, device, count as i32);
                    } else {
                        context.as_mut().shadeSpan(x, y, span, count as i32);
                        (this.fProc32Blend)(
                            core::slice::from_raw_parts_mut(device, count as usize),
                            core::slice::from_raw_parts(span, count as usize),
                            count as usize,
                            alpha,
                        );
                    }
                }
                device = device.add(count as usize);
                offset += count as usize;
                x += count as i32;
            }
        } else {
            loop {
                let count = runs[offset];
                if count <= 0 {
                    break;
                }
                let alpha = aa[offset] as u32;
                if alpha != 0 {
                    context.as_mut().shadeSpan(x, y, span, count as i32);
                    if alpha == 255 {
                        (this.fProc32)(
                            core::slice::from_raw_parts_mut(device, count as usize),
                            core::slice::from_raw_parts(span, count as usize),
                            count as usize,
                            255,
                        );
                    } else {
                        (this.fProc32Blend)(
                            core::slice::from_raw_parts_mut(device, count as usize),
                            core::slice::from_raw_parts(span, count as usize),
                            count as usize,
                            alpha,
                        );
                    }
                }
                device = device.add(count as usize);
                offset += count as usize;
                x += count as i32;
            }
        }
    }
}

// Source: src/core/SkBlitter_ARGB32.cpp:1858-1860
// Native skvx lane shapes use the actual canonical SkVx array representation.
type ShaderU32 = [u32; 4];
type ShaderU8x4 = [u8; 16];
type ShaderU8 = [u8; 4];
// Source: src/core/SkBlitter_ARGB32.cpp:1862-1884
unsafe fn drive_shader_a8(
    mut dst: *mut u32,
    mut src: *const u32,
    mut cov: *const u8,
    mut n: i32,
    kernel: fn(ShaderU8x4, ShaderU8x4, ShaderU8x4) -> ShaderU8x4,
) {
    let apply = |d: ShaderU32, s: ShaderU32, c: ShaderU8| -> ShaderU32 {
        let cov_splat =
            crate::src::core::SkVx::shuffle(c, [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);
        core::mem::transmute(kernel(
            core::mem::transmute(d),
            core::mem::transmute(s),
            cov_splat,
        ))
    };
    while n >= 4 {
        let output = apply(
            core::ptr::read_unaligned(dst.cast::<ShaderU32>()),
            core::ptr::read_unaligned(src.cast::<ShaderU32>()),
            core::ptr::read_unaligned(cov.cast::<ShaderU8>()),
        );
        core::ptr::write_unaligned(dst.cast::<ShaderU32>(), output);
        dst = dst.add(4);
        src = src.add(4);
        cov = cov.add(4);
        n -= 4;
    }
    loop {
        let old = n;
        n -= 1;
        if old <= 0 {
            break;
        }
        // Native Vec{value} initializes only lane0 and zero-fills others.
        *dst = apply([*dst, 0, 0, 0], [*src, 0, 0, 0], [*cov, 0, 0, 0])[0];
        dst = dst.add(1);
        src = src.add(1);
        cov = cov.add(1);
    }
}
// Source: src/core/SkBlitter_ARGB32.cpp:1886-1893
/// Safety: n>=0; source/mask contain n initialized premultiplied pixels/bytes,
/// destination contains n exclusive initialized pixels and is disjoint from them.
pub unsafe fn blend_row_A8(dst: *mut u32, mask: *const u8, src: *const u32, n: i32) {
    drive_shader_a8(dst, src, mask, n, |d, s, c| {
        let s_aa = crate::src::core::SkVx::approx_scale(s, c);
        let alpha = crate::src::core::SkVx::shuffle(
            s_aa,
            [3, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15],
        );
        let right = crate::src::core::SkVx::approx_scale(d, alpha.map(|a| 255 - a));
        core::array::from_fn(|i| s_aa[i].wrapping_add(right[i]))
    });
}
// Source: src/core/SkBlitter_ARGB32.cpp:1895-1901
/// Safety: same row bounds/disjointness contract as blend_row_A8; source is opaque.
pub unsafe fn blend_row_A8_opaque(dst: *mut u32, mask: *const u8, src: *const u32, n: i32) {
    drive_shader_a8(dst, src, mask, n, |d, s, c| {
        crate::src::core::SkVx::div255(&core::array::from_fn(|i| {
            s[i] as u16 * c[i] as u16 + d[i] as u16 * (255 - c[i]) as u16
        }))
    });
}

// Source: src/core/SkBlitter_ARGB32.cpp:1903-1945
pub fn blend_row_lcd16(dst: &mut [u32], mask: &[u16], src: &[u32], n: i32) {
    // SkAlphaMul is a C++ macro here: preserve signed multiplication and
    // arithmetic shift for the negative intermediate, rather than calling the
    // unsigned-only SkColorPriv Rust helper.
    let src_alpha_blend = |s: i32, d: i32, sa: i32, m: i32| d + ((s - ((sa * d) >> 8)) * m >> 8);
    let upscale_31_to_255 = |v: i32| (v << 3) | (v >> 2);
    for i in 0..n {
        let i = i as usize;
        let m = mask[i];
        if m == 0 {
            continue;
        }
        let s = src[i];
        let d = dst[i];
        let mut srcA = SkGetPackedA32(s) as i32;
        let srcR = SkGetPackedR32(s) as i32;
        let srcG = SkGetPackedG32(s) as i32;
        let srcB = SkGetPackedB32(s) as i32;
        srcA += srcA >> 7;
        let mut maskR = (SkGetPackedR16(m as u32) >> (SK_R16_BITS - 5)) as i32;
        let mut maskG = (SkGetPackedG16(m as u32) >> (SK_G16_BITS - 5)) as i32;
        let mut maskB = (SkGetPackedB16(m as u32) >> (SK_B16_BITS - 5)) as i32;
        maskR = upscale_31_to_255(maskR);
        maskG = upscale_31_to_255(maskG);
        maskB = upscale_31_to_255(maskB);
        dst[i] = SkPackARGB32(
            255,
            src_alpha_blend(srcR, SkGetPackedR32(d) as i32, srcA, maskR) as u32,
            src_alpha_blend(srcG, SkGetPackedG32(d) as i32, srcA, maskG) as u32,
            src_alpha_blend(srcB, SkGetPackedB32(d) as i32, srcA, maskB) as u32,
        );
    }
}

// Source: src/core/SkBlitter_ARGB32.cpp:1947-1979
pub fn blend_row_LCD16_opaque(dst: &mut [u32], mask: &[u16], src: &[u32], n: i32) {
    for i in 0..n {
        let i = i as usize;
        let m = mask[i];
        if m == 0 {
            continue;
        }
        let s = src[i];
        let d = dst[i];
        let srcR = SkGetPackedR32(s) as i32;
        let srcG = SkGetPackedG32(s) as i32;
        let srcB = SkGetPackedB32(s) as i32;
        let mut maskR = (SkGetPackedR16(m as u32) >> (SK_R16_BITS - 5)) as i32;
        let mut maskG = (SkGetPackedG16(m as u32) >> (SK_G16_BITS - 5)) as i32;
        let mut maskB = (SkGetPackedB16(m as u32) >> (SK_B16_BITS - 5)) as i32;
        maskR = upscale_31_to_32(maskR);
        maskG = upscale_31_to_32(maskG);
        maskB = upscale_31_to_32(maskB);
        dst[i] = SkPackARGB32(
            255,
            blend_32(srcR, SkGetPackedR32(d) as i32, maskR) as u32,
            blend_32(srcG, SkGetPackedG32(d) as i32, maskG) as u32,
            blend_32(srcB, SkGetPackedB32(d) as i32, maskB) as u32,
        );
    }
}

// Rust virtual-dispatch adapters. All concrete algorithms are mapped to their
// original regions above; these forwarders add no translated source-line count.
impl crate::src::core::SkCoreBlitters::SkARGB32_Shader_Blitter<'_> {
    // Source: src/core/SkBlitter_ARGB32.cpp:1981-2020
    pub unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        use crate::src::core::SkMask::Format;
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 1982; @condition b"mask.fBounds.contains(clip)\0".as_ptr(); mask.fBounds.contains_rect(clip));
        let opaque = this.shader.fShaderContext.as_ref().getFlags()
            & crate::src::shaders::SkShaderBase::kOpaqueAlpha_Flag
            != 0;
        let blend_row: unsafe fn(*mut u32, *const u8, *const u32, i32) =
            if mask.fFormat == Format::kA8_Format && opaque {
                blend_row_A8_opaque
            } else if mask.fFormat == Format::kA8_Format {
                blend_row_A8
            } else if mask.fFormat == Format::kLCD16_Format && opaque {
                |d, m, s, n| {
                    blend_row_LCD16_opaque(
                        core::slice::from_raw_parts_mut(d, n as usize),
                        core::slice::from_raw_parts(m.cast::<u16>(), n as usize),
                        core::slice::from_raw_parts(s, n as usize),
                        n,
                    )
                }
            } else if mask.fFormat == Format::kLCD16_Format {
                |d, m, s, n| {
                    blend_row_lcd16(
                        core::slice::from_raw_parts_mut(d, n as usize),
                        core::slice::from_raw_parts(m.cast::<u16>(), n as usize),
                        core::slice::from_raw_parts(s, n as usize),
                        n,
                    )
                }
            } else {
                crate::src::core::SkBlitter::blitMask(
                    core::pin::Pin::new_unchecked(&mut *this),
                    mask,
                    clip,
                );
                return;
            };
        let x = clip.fLeft;
        let width = clip.width();
        let mut y = clip.fTop;
        let mut height = clip.height();
        let mut dst_row = this
            .shader
            .raster
            .fDevice
            .writable_addr32(x, y)
            .cast::<u8>();
        let dst_rb = this.shader.raster.fDevice.rowBytes();
        let mut mask_row = mask.getAddr(x, y).cast::<u8>();
        let mask_rb = mask.fRowBytes as usize;
        let span = this.fBuffer.get().cast::<u32>();
        loop {
            this.shader
                .fShaderContext
                .as_mut()
                .shadeSpan(x, y, span, width);
            blend_row(dst_row.cast(), mask_row, span, width);
            dst_row = dst_row.wrapping_add(dst_rb);
            mask_row = mask_row.wrapping_add(mask_rb);
            y += 1;
            height -= 1;
            if height <= 0 {
                break;
            }
        }
    }
    // Source: src/core/SkBlitter_ARGB32.cpp:2022-2054
    pub unsafe fn blitV(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        mut y: i32,
        mut height: i32,
        alpha: u8,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let pm = &this.shader.raster.fDevice;
        crate::SkASSERT!(@source b"src/core/SkBlitter_ARGB32.cpp\0".as_ptr(), 2023; @condition b"x >= 0 && y >= 0 && y + height <= fDevice.height()\0".as_ptr(); x >= 0 && y >= 0 && y + height <= pm.height());
        let mut device = pm.writable_addr32(x, y);
        let device_rb = pm.rowBytes();
        if this.fShadeDirectlyIntoDevice {
            if alpha == 255 {
                loop {
                    this.shader
                        .fShaderContext
                        .as_mut()
                        .shadeSpan(x, y, device, 1);
                    y += 1;
                    device = device.cast::<u8>().wrapping_add(device_rb).cast();
                    height -= 1;
                    if height <= 0 {
                        break;
                    }
                }
            } else {
                loop {
                    let mut c = core::mem::MaybeUninit::<u32>::uninit();
                    this.shader
                        .fShaderContext
                        .as_mut()
                        .shadeSpan(x, y, c.as_mut_ptr(), 1);
                    *device = crate::src::core::SkColorData::SkFourByteInterp(
                        c.assume_init(),
                        *device,
                        alpha as u32,
                    );
                    y += 1;
                    device = device.cast::<u8>().wrapping_add(device_rb).cast();
                    height -= 1;
                    if height <= 0 {
                        break;
                    }
                }
            }
        } else {
            let span = this.fBuffer.get().cast::<u32>();
            let proc = if alpha == 255 {
                this.fProc32
            } else {
                this.fProc32Blend
            };
            loop {
                this.shader.fShaderContext.as_mut().shadeSpan(x, y, span, 1);
                proc(
                    core::slice::from_raw_parts_mut(device, 1),
                    core::slice::from_raw_parts(span, 1),
                    1,
                    alpha as u32,
                );
                y += 1;
                device = device.cast::<u8>().wrapping_add(device_rb).cast();
                height -= 1;
                if height <= 0 {
                    break;
                }
            }
        }
    }
}

// Rust virtual-inheritance forwarding to the same actual shader/context/scratch
// owner. The unsafe primitive caller contract includes all destination/run and
// Context shadeSpan initialization/lease obligations recorded on new().
unsafe impl crate::src::core::SkBlitter::SkBlitter
    for crate::src::core::SkCoreBlitters::SkARGB32_Shader_Blitter<'_>
{
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.shader.raster.base.allocBlitMemory(sz)
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, w: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitH(core::pin::Pin::new_unchecked(&mut *this), x, y, w)
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        a: &mut [u8],
        r: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitAntiH(core::pin::Pin::new_unchecked(&mut *this), x, y, a, r)
    }
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, h: i32, a: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitV(core::pin::Pin::new_unchecked(&mut *this), x, y, h, a)
    }
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, w: i32, h: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitRect(core::pin::Pin::new_unchecked(&mut *this), x, y, w, h)
    }
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        m: &crate::src::core::SkMask::SkMask,
        c: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitMask(core::pin::Pin::new_unchecked(&mut *this), m, c)
    }
}

// Safety: scratch storage delegates to the canonical base owner; row calls
// retain no alpha/run borrows and preserve native sparse-run termination.
unsafe impl crate::src::core::SkBlitter::SkBlitter
    for crate::src::core::SkCoreBlitters::SkARGB32_Blitter
{
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, n: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.raster.base.allocBlitMemory(n)
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, w: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitH(core::pin::Pin::new_unchecked(&mut *this), x, y, w)
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        a: &mut [u8],
        r: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitAntiH(core::pin::Pin::new_unchecked(&mut *this), x, y, a, r)
    }
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, h: i32, a: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitV(core::pin::Pin::new_unchecked(&mut *this), x, y, h, a)
    }
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, w: i32, h: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitRect(core::pin::Pin::new_unchecked(&mut *this), x, y, w, h)
    }
    unsafe fn blitAntiH2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a: u32, b: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitAntiH2(core::pin::Pin::new_unchecked(&mut *this), x, y, a, b)
    }
    unsafe fn blitAntiV2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a: u32, b: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitAntiV2(core::pin::Pin::new_unchecked(&mut *this), x, y, a, b)
    }
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        m: &crate::src::core::SkMask::SkMask,
        c: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitMask(core::pin::Pin::new_unchecked(&mut *this), m, c)
    }
}
// Safety: scratch storage delegates to the canonical base owner; row calls
// retain no alpha/run borrows and preserve native sparse-run termination.
unsafe impl crate::src::core::SkBlitter::SkBlitter
    for crate::src::core::SkCoreBlitters::SkARGB32_Opaque_Blitter
{
    fn canDirectBlit(
        self: core::pin::Pin<&mut Self>,
    ) -> Option<crate::src::core::SkBlitter::DirectBlit> {
        Self::canDirectBlit(self)
    }
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, n: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.argb.raster.base.allocBlitMemory(n)
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, w: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        core::pin::Pin::new_unchecked(&mut this.argb).blitH(x, y, w)
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        a: &mut [u8],
        r: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitAntiH(core::pin::Pin::new_unchecked(&mut *this), x, y, a, r)
    }
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, h: i32, a: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        core::pin::Pin::new_unchecked(&mut this.argb).blitV(x, y, h, a)
    }
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, w: i32, h: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        core::pin::Pin::new_unchecked(&mut this.argb).blitRect(x, y, w, h)
    }
    unsafe fn blitAntiH2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a: u32, b: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitAntiH2(core::pin::Pin::new_unchecked(&mut *this), x, y, a, b)
    }
    unsafe fn blitAntiV2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a: u32, b: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitAntiV2(core::pin::Pin::new_unchecked(&mut *this), x, y, a, b)
    }
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        m: &crate::src::core::SkMask::SkMask,
        c: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitMask(core::pin::Pin::new_unchecked(&mut *this), m, c)
    }
}
// Safety: scratch storage delegates to the canonical base owner; row calls
// retain no alpha/run borrows and preserve native sparse-run termination.
unsafe impl crate::src::core::SkBlitter::SkBlitter
    for crate::src::core::SkCoreBlitters::SkARGB32_Black_Blitter
{
    fn canDirectBlit(
        self: core::pin::Pin<&mut Self>,
    ) -> Option<crate::src::core::SkBlitter::DirectBlit> {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { core::pin::Pin::new_unchecked(&mut this.opaque) }.canDirectBlit()
    }
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, n: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.opaque.argb.raster.base.allocBlitMemory(n)
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, w: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        core::pin::Pin::new_unchecked(&mut this.opaque.argb).blitH(x, y, w)
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        a: &mut [u8],
        r: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitAntiH(core::pin::Pin::new_unchecked(&mut *this), x, y, a, r)
    }
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, h: i32, a: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        core::pin::Pin::new_unchecked(&mut this.opaque.argb).blitV(x, y, h, a)
    }
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, w: i32, h: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        core::pin::Pin::new_unchecked(&mut this.opaque.argb).blitRect(x, y, w, h)
    }
    unsafe fn blitAntiH2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a: u32, b: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitAntiH2(core::pin::Pin::new_unchecked(&mut *this), x, y, a, b)
    }
    unsafe fn blitAntiV2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a: u32, b: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        Self::blitAntiV2(core::pin::Pin::new_unchecked(&mut *this), x, y, a, b)
    }
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        m: &crate::src::core::SkMask::SkMask,
        c: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        core::pin::Pin::new_unchecked(&mut this.opaque).blitMask(m, c)
    }
}
