// Copyright 2011 Google Inc. BSD-style license.
use crate::src::core::SkBlitRow::{Proc32, SkBlitRow};
use crate::src::core::SkColorData::{SkBlendARGB32, SkPMLerp};
use crate::src::core::SkColorPriv::{SkAlpha255To256, SkGetPackedA32};
// Borrowed-slice API projections of the raw native bodies below. These
// adapters preserve existing callers; no duplicate per-pixel algorithm lives here.
fn blit_row_s32_opaque(dst: &mut [u32], src: &[u32], count: usize, alpha: u32) {
    assert!(count <= dst.len() && count <= src.len() && count <= i32::MAX as usize);
    unsafe { blit_row_s32_opaque_raw(dst.as_mut_ptr(), src.as_ptr(), count as i32, alpha) }
}
pub fn blit_row_s32_blend(dst: &mut [u32], src: &[u32], count: usize, alpha: u32) {
    assert!(count <= dst.len() && count <= src.len() && count <= i32::MAX as usize);
    unsafe { blit_row_s32_blend_raw(dst.as_mut_ptr(), src.as_ptr(), count as i32, alpha) }
}
pub fn blit_row_s32a_blend(dst: &mut [u32], src: &[u32], count: usize, alpha: u32) {
    assert!(count <= dst.len() && count <= src.len() && count <= i32::MAX as usize);
    unsafe { blit_row_s32a_blend_raw(dst.as_mut_ptr(), src.as_ptr(), count as i32, alpha) }
}
fn blit_row_s32a_opaque(dst: &mut [u32], src: &[u32], count: usize, alpha: u32) {
    assert!(count <= dst.len() && count <= src.len() && count <= i32::MAX as usize);
    unsafe {
        (SkBlitRow::Factory32_raw(SkBlitRow::kSrcPixelAlpha_Flag32))(
            dst.as_mut_ptr(),
            src.as_ptr(),
            count as i32,
            alpha,
        )
    }
}
impl SkBlitRow {
    pub fn Factory32(mut flags: u32) -> Proc32 {
        const PROCS: [Option<Proc32>; 4] = [
            Some(blit_row_s32_opaque),
            Some(blit_row_s32_blend),
            None,
            Some(blit_row_s32a_blend),
        ];
        debug_assert!((flags as usize) < PROCS.len());
        flags &= PROCS.len() as u32 - 1;
        if flags == Self::kSrcPixelAlpha_Flag32 {
            blit_row_s32a_opaque
        } else {
            PROCS[flags as usize].unwrap()
        }
    }
    pub fn Color32(dst: &mut [u32], count: usize, color: u32) {
        assert!(count <= dst.len() && count <= i32::MAX as usize);
        unsafe { Self::Color32_raw(dst.as_mut_ptr(), count as i32, color) }
    }
}

// Source: src/core/SkBlitRow_D32.cpp:20-26
/// Safety: count>=0, alpha==255; source/destination are aligned live initialized
/// count-pixel spans satisfying memcpy's disjoint-range contract.
unsafe fn blit_row_s32_opaque_raw(dst: *mut u32, src: *const u32, count: i32, alpha: u32) {
    crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 24; @condition b"255 == alpha\0".as_ptr(); (alpha) == (255));
    if count != 0 {
        unsafe { core::ptr::copy_nonoverlapping(src, dst, count as usize) }
    }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod sse2 {
    use crate::src::core::SkColorData::{SkBlendARGB32, SkPMLerp};
    use crate::src::core::SkColorPriv::SkAlpha255To256;
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;
    // Source: src/core/SkBlitRow_D32.cpp:37-63
    #[target_feature(enable = "sse2")]
    pub unsafe fn SkPMLerp_SSE2(src: __m128i, dst: __m128i, src_scale: u32) -> __m128i {
        let mask = _mm_set1_epi32(0x00ff00ff);
        let src_rb = _mm_and_si128(mask, src);
        let src_ag = _mm_srli_epi16::<8>(src);
        let dst_rb = _mm_and_si128(mask, dst);
        let dst_ag = _mm_srli_epi16::<8>(dst);
        let mut diff_rb = _mm_sub_epi16(src_rb, dst_rb);
        let mut diff_ag = _mm_sub_epi16(src_ag, dst_ag);
        let scale = _mm_set1_epi16(src_scale as i16);
        diff_rb = _mm_mullo_epi16(diff_rb, scale);
        diff_ag = _mm_mullo_epi16(diff_ag, scale);
        diff_rb = _mm_srli_epi16::<8>(diff_rb);
        diff_ag = _mm_andnot_si128(mask, diff_ag);
        _mm_add_epi8(dst, _mm_or_si128(diff_rb, diff_ag))
    }
    // Source: src/core/SkBlitRow_D32.cpp:66-89
    /// Safety: SSE2 available; count>=0, alpha<=255; initialized source and
    /// readable/exclusively writable destination span count pixels. No aliased
    /// Rust references; original four-pixel loads precede each batch store.
    #[target_feature(enable = "sse2")]
    pub unsafe fn blit_row_s32_blend(
        mut dst: *mut u32,
        mut src: *const u32,
        mut count: i32,
        alpha: u32,
    ) {
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 67; @condition b"alpha <= 255\0".as_ptr(); alpha<=255);
        let mut src4 = src.cast::<__m128i>();
        let mut dst4 = dst.cast::<__m128i>();
        while count >= 4 {
            _mm_storeu_si128(
                dst4,
                SkPMLerp_SSE2(
                    _mm_loadu_si128(src4),
                    _mm_loadu_si128(dst4),
                    SkAlpha255To256(alpha),
                ),
            );
            src4 = src4.add(1);
            dst4 = dst4.add(1);
            count -= 4;
        }
        src = src4.cast();
        dst = dst4.cast();
        while count > 0 {
            dst.write(SkPMLerp(src.read(), dst.read(), SkAlpha255To256(alpha)));
            src = src.add(1);
            dst = dst.add(1);
            count -= 1;
        }
    }
    // Source: src/core/SkBlitRow_D32.cpp:91-129
    #[target_feature(enable = "sse2")]
    pub unsafe fn SkBlendARGB32_SSE2(src: __m128i, dst: __m128i, aa: u32) -> __m128i {
        let alpha = SkAlpha255To256(aa);
        let src_scale = _mm_set1_epi16(alpha as i16);
        let mut dst_scale = _mm_srli_epi32::<24>(src);
        dst_scale = _mm_mullo_epi16(dst_scale, src_scale);
        dst_scale = _mm_sub_epi32(_mm_set1_epi32(0xffff), dst_scale);
        dst_scale = _mm_add_epi32(dst_scale, _mm_srli_epi32::<8>(dst_scale));
        dst_scale = _mm_srli_epi32::<8>(dst_scale);
        dst_scale = _mm_shufflelo_epi16::<0xa0>(dst_scale);
        dst_scale = _mm_shufflehi_epi16::<0xa0>(dst_scale);
        let mask = _mm_set1_epi32(0x00ff00ff);
        let mut src_rb = _mm_and_si128(mask, src);
        let mut src_ag = _mm_srli_epi16::<8>(src);
        let mut dst_rb = _mm_and_si128(mask, dst);
        let mut dst_ag = _mm_srli_epi16::<8>(dst);
        src_rb = _mm_mullo_epi16(src_rb, src_scale);
        src_ag = _mm_mullo_epi16(src_ag, src_scale);
        dst_rb = _mm_mullo_epi16(dst_rb, dst_scale);
        dst_ag = _mm_mullo_epi16(dst_ag, dst_scale);
        dst_rb = _mm_add_epi16(src_rb, dst_rb);
        dst_ag = _mm_add_epi16(src_ag, dst_ag);
        dst_rb = _mm_srli_epi16::<8>(dst_rb);
        dst_ag = _mm_andnot_si128(mask, dst_ag);
        _mm_or_si128(dst_rb, dst_ag)
    }
    // Source: src/core/SkBlitRow_D32.cpp:131-154
    /// Safety: SSE2 available, count>=0, alpha<=255; initialized source and
    /// readable/exclusively writable destination count-pixel spans. Native
    /// four-pixel batching and sequential scalar tail retain raw overlap order.
    #[target_feature(enable = "sse2")]
    pub unsafe fn blit_row_s32a_blend(
        mut dst: *mut u32,
        mut src: *const u32,
        mut count: i32,
        alpha: u32,
    ) {
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 132; @condition b"alpha <= 255\0".as_ptr(); alpha<=255);
        let mut src4 = src.cast::<__m128i>();
        let mut dst4 = dst.cast::<__m128i>();
        while count >= 4 {
            _mm_storeu_si128(
                dst4,
                SkBlendARGB32_SSE2(_mm_loadu_si128(src4), _mm_loadu_si128(dst4), alpha),
            );
            src4 = src4.add(1);
            dst4 = dst4.add(1);
            count -= 4;
        }
        src = src4.cast();
        dst = dst4.cast();
        while count > 0 {
            dst.write(SkBlendARGB32(src.read(), dst.read(), alpha));
            src = src.add(1);
            dst = dst.add(1);
            count -= 1;
        }
    }
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub mod neon {
    use crate::src::core::SkColorPriv::SkAlpha255To256;
    use core::arch::aarch64::*;
    // Source: src/core/SkBlitRow_D32.cpp:159-202
    /// Safety: alpha<=255, count>=0; src readable and dst initialized/readable/
    /// exclusively writable for count u32 pixels. Raw ranges may overlap with
    /// the exact native two-pixel sequential load/store behavior. All pointers
    /// remain live/aligned for u32; this function retains no pixel lease.
    pub unsafe fn blit_row_s32_blend(
        mut dst: *mut u32,
        mut src: *const u32,
        mut count: i32,
        alpha: u32,
    ) {
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 160; @condition b"alpha <= 255\0".as_ptr(); alpha<=255);
        let src_scale = SkAlpha255To256(alpha) as u16;
        let dst_scale = 256 - src_scale;
        while count >= 2 {
            let vsrc = vreinterpret_u8_u32(vld1_u32(src));
            let vdst = vreinterpret_u8_u32(vld1_u32(dst));
            let mut vsrc_wide = vmovl_u8(vsrc);
            vsrc_wide = vmulq_u16(vsrc_wide, vdupq_n_u16(src_scale));
            let mut vdst_wide = vmull_u8(vdst, vdup_n_u8(dst_scale as u8));
            vdst_wide = vaddq_u16(vdst_wide, vsrc_wide);
            let vres = vshrn_n_u16::<8>(vdst_wide);
            vst1_u32(dst, vreinterpret_u32_u8(vres));
            src = src.add(2);
            dst = dst.add(2);
            count -= 2;
        }
        if count == 1 {
            let mut vsrc = vdup_n_u8(0);
            let mut vdst = vdup_n_u8(0);
            vsrc = vreinterpret_u8_u32(vld1_lane_u32::<0>(src, vreinterpret_u32_u8(vsrc)));
            vdst = vreinterpret_u8_u32(vld1_lane_u32::<0>(dst, vreinterpret_u32_u8(vdst)));
            let mut vsrc_wide = vmovl_u8(vsrc);
            vsrc_wide = vmulq_u16(vsrc_wide, vdupq_n_u16(src_scale));
            let mut vdst_wide = vmull_u8(vdst, vdup_n_u8(dst_scale as u8));
            vdst_wide = vaddq_u16(vdst_wide, vsrc_wide);
            let vres = vshrn_n_u16::<8>(vdst_wide);
            vst1_lane_u32::<0>(dst, vreinterpret_u32_u8(vres));
        }
    }
    // Source-outside current ARM64 lowering of only the two original prefetch
    // builtins below. The register retains the original base pointer; encoding
    // the byte displacement avoids an out-of-allocation Rust inbounds add.
    // PRFM is a hint, not an ordinary pixel load/store. No literal source credit.
    #[inline(always)]
    unsafe fn prefetch_32_pixels(base: *const u32) {
        core::arch::asm!("prfm pldl1keep, [{base}, #128]",base=in(reg)base,options(nostack,readonly));
    }
    // Source: src/core/SkBlitRow_D32.cpp:204-278
    /// Safety: alpha<255,count>=0; live initialized readable source and
    /// initialized/exclusively writable destination count-pixel spans aligned
    /// for u32. Raw overlap has original odd-first then two-pixel batch order;
    /// no live Rust references alias those spans. No pixel lease is retained.
    pub unsafe fn blit_row_s32a_blend(
        mut dst: *mut u32,
        mut src: *const u32,
        mut count: i32,
        alpha: u32,
    ) {
        use crate::src::core::SkColorData::SkAlphaMulInv256;
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 205; @condition b"alpha < 255\0".as_ptr(); alpha<255);
        let alpha256 = SkAlpha255To256(alpha);
        if count & 1 != 0 {
            let mut vsrc = vdup_n_u8(0);
            let mut vdst = vdup_n_u8(0);
            vsrc = vreinterpret_u8_u32(vld1_lane_u32::<0>(src, vreinterpret_u32_u8(vsrc)));
            vdst = vreinterpret_u8_u32(vld1_lane_u32::<0>(dst, vreinterpret_u32_u8(vdst)));
            let mut dst_scale = vget_lane_u8::<3>(vsrc) as u32;
            dst_scale = SkAlphaMulInv256(dst_scale, alpha256);
            let mut vsrc_wide = vmovl_u8(vsrc);
            vsrc_wide = vmulq_n_u16(vsrc_wide, alpha256 as u16);
            let mut vdst_wide = vmovl_u8(vdst);
            vdst_wide = vmulq_n_u16(vdst_wide, dst_scale as u16);
            vdst_wide = vaddq_u16(vdst_wide, vsrc_wide);
            let vres = vshrn_n_u16::<8>(vdst_wide);
            vst1_lane_u32::<0>(dst, vreinterpret_u32_u8(vres));
            dst = dst.add(1);
            src = src.add(1);
            count -= 1;
        }
        let alpha_mask_setup = [3u8, 3, 3, 3, 7, 7, 7, 7];
        let alpha_mask = vld1_u8(alpha_mask_setup.as_ptr());
        while count != 0 {
            // Original src+32/dst+32 prefetch only: root-reviewed PRFM lowering.
            prefetch_32_pixels(src);
            prefetch_32_pixels(dst);
            let vsrc = vreinterpret_u8_u32(vld1_u32(src));
            let vdst = vreinterpret_u8_u32(vld1_u32(dst));
            let vsrc_scale = vdupq_n_u16(alpha256 as u16);
            let vsrc_alphas = vtbl1_u8(vsrc, alpha_mask);
            let mut vdst_scale = vmovl_u8(vsrc_alphas);
            vdst_scale = vmlsq_u16(vdupq_n_u16(0xff00), vdst_scale, vsrc_scale);
            vdst_scale = vsraq_n_u16::<8>(vdst_scale, vdst_scale);
            vdst_scale = vsraq_n_u16::<8>(vdupq_n_u16(1), vdst_scale);
            let mut vsrc_wide = vmovl_u8(vsrc);
            vsrc_wide = vmulq_u16(vsrc_wide, vsrc_scale);
            let mut vdst_wide = vmovl_u8(vdst);
            vdst_wide = vmulq_u16(vdst_wide, vdst_scale);
            vdst_wide = vaddq_u16(vdst_wide, vsrc_wide);
            let vres = vshrn_n_u16::<8>(vdst_wide);
            vst1_u32(dst, vreinterpret_u32_u8(vres));
            src = src.add(2);
            dst = dst.add(2);
            count -= 2;
        }
    }
}

#[cfg(target_arch = "loongarch64")]
pub mod lasx {
    use crate::src::core::SkColorData::{SkBlendARGB32, SkPMLerp};
    use crate::src::core::SkColorPriv::SkAlpha255To256;
    use core::arch::loongarch64::*;
    // Actual stdarch register/intrinsic API requires stdarch_loongarch on the
    // target toolchain. Root owns that crate-level feature gate; no fallback ISA
    // or duplicate register representation is introduced here.
    // Source: src/core/SkBlitRow_D32.cpp:283-309
    #[target_feature(enable = "lasx")]
    pub unsafe fn SkPMLerp_LASX(src: m256i, dst: m256i, src_scale: u32) -> m256i {
        let mask = lasx_xvreplgr2vr_w(0x00ff00ff);
        let src_rb = lasx_xvand_v(mask, src);
        let src_ag = lasx_xvsrli_h::<8>(src);
        let dst_rb = lasx_xvand_v(mask, dst);
        let dst_ag = lasx_xvsrli_h::<8>(dst);
        let mut diff_rb = lasx_xvsub_h(src_rb, dst_rb);
        let mut diff_ag = lasx_xvsub_h(src_ag, dst_ag);
        let scale = lasx_xvreplgr2vr_h(src_scale as i32);
        diff_rb = lasx_xvmul_h(diff_rb, scale);
        diff_ag = lasx_xvmul_h(diff_ag, scale);
        diff_rb = lasx_xvsrli_h::<8>(diff_rb);
        diff_ag = lasx_xvandn_v(mask, diff_ag);
        let diff = lasx_xvor_v(diff_rb, diff_ag);
        lasx_xvadd_b(dst, diff)
    }
    // Source: src/core/SkBlitRow_D32.cpp:312-335
    /// Safety: LASX available; alpha<=255,count>=0; initialized readable
    /// src and initialized readable/exclusively writable dst count-pixel spans.
    /// Original batch loads precede stores; raw overlap ordering is retained.
    #[target_feature(enable = "lasx")]
    pub unsafe fn blit_row_s32_blend(
        mut dst: *mut u32,
        mut src: *const u32,
        mut count: i32,
        alpha: u32,
    ) {
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 313; @condition b"alpha <= 255\0".as_ptr(); alpha<=255);
        let mut srcv = src.cast::<m256i>();
        let mut dstv = dst.cast::<m256i>();
        while count >= 8 {
            lasx_xvst::<0>(
                SkPMLerp_LASX(
                    lasx_xvld::<0>(srcv.cast()),
                    lasx_xvld::<0>(dstv.cast()),
                    SkAlpha255To256(alpha),
                ),
                dstv.cast(),
            );
            srcv = srcv.add(1);
            dstv = dstv.add(1);
            count -= 8;
        }
        src = srcv.cast();
        dst = dstv.cast();
        while count > 0 {
            dst.write(SkPMLerp(src.read(), dst.read(), SkAlpha255To256(alpha)));
            src = src.add(1);
            dst = dst.add(1);
            count -= 1;
        }
    }
    // Source: src/core/SkBlitRow_D32.cpp:337-373
    #[target_feature(enable = "lasx")]
    pub unsafe fn SkBlendARGB32_LASX(src: m256i, dst: m256i, aa: u32) -> m256i {
        let alpha = SkAlpha255To256(aa);
        let src_scale = lasx_xvreplgr2vr_h(alpha as i32);
        let mut dst_scale = lasx_xvsrli_w::<24>(src);
        dst_scale = lasx_xvmul_h(dst_scale, src_scale);
        dst_scale = lasx_xvsub_w(lasx_xvreplgr2vr_w(0xffff), dst_scale);
        dst_scale = lasx_xvadd_w(dst_scale, lasx_xvsrli_w::<8>(dst_scale));
        dst_scale = lasx_xvsrli_w::<8>(dst_scale);
        dst_scale = lasx_xvshuf4i_h::<0xa0>(dst_scale);
        let mask = lasx_xvreplgr2vr_w(0x00ff00ff);
        let mut src_rb = lasx_xvand_v(mask, src);
        let mut src_ag = lasx_xvsrli_h::<8>(src);
        let mut dst_rb = lasx_xvand_v(mask, dst);
        let mut dst_ag = lasx_xvsrli_h::<8>(dst);
        src_rb = lasx_xvmul_h(src_rb, src_scale);
        src_ag = lasx_xvmul_h(src_ag, src_scale);
        dst_rb = lasx_xvmul_h(dst_rb, dst_scale);
        dst_ag = lasx_xvmul_h(dst_ag, dst_scale);
        dst_rb = lasx_xvadd_h(src_rb, dst_rb);
        dst_ag = lasx_xvadd_h(src_ag, dst_ag);
        dst_rb = lasx_xvsrli_h::<8>(dst_rb);
        dst_ag = lasx_xvandn_v(mask, dst_ag);
        lasx_xvor_v(dst_rb, dst_ag)
    }
    // Source: src/core/SkBlitRow_D32.cpp:375-398
    /// Safety: same actual ISA/raw pixel-span conditions as non-source-alpha
    /// producer above; count>=0, alpha<=255 and no aliasing Rust references.
    #[target_feature(enable = "lasx")]
    pub unsafe fn blit_row_s32a_blend(
        mut dst: *mut u32,
        mut src: *const u32,
        mut count: i32,
        alpha: u32,
    ) {
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 376; @condition b"alpha <= 255\0".as_ptr(); alpha<=255);
        let mut srcv = src.cast::<m256i>();
        let mut dstv = dst.cast::<m256i>();
        while count >= 8 {
            lasx_xvst::<0>(
                SkBlendARGB32_LASX(
                    lasx_xvld::<0>(srcv.cast()),
                    lasx_xvld::<0>(dstv.cast()),
                    alpha,
                ),
                dstv.cast(),
            );
            srcv = srcv.add(1);
            dstv = dstv.add(1);
            count -= 8;
        }
        src = srcv.cast();
        dst = dstv.cast();
        while count > 0 {
            dst.write(SkBlendARGB32(src.read(), dst.read(), alpha));
            src = src.add(1);
            dst = dst.add(1);
            count -= 1;
        }
    }
}

#[cfg(target_arch = "loongarch64")]
pub mod lsx {
    use crate::src::core::SkColorData::{SkBlendARGB32, SkPMLerp};
    use crate::src::core::SkColorPriv::SkAlpha255To256;
    use core::arch::loongarch64::*;
    // Actual stdarch register/intrinsic API requires stdarch_loongarch on the
    // target toolchain. Root owns that crate-level feature gate; no fallback ISA
    // or duplicate register representation is introduced here.
    // Source: src/core/SkBlitRow_D32.cpp:403-429
    #[target_feature(enable = "lsx")]
    pub unsafe fn SkPMLerp_LSX(src: m128i, dst: m128i, src_scale: u32) -> m128i {
        let mask = lsx_vreplgr2vr_w(0x00ff00ff);
        let src_rb = lsx_vand_v(mask, src);
        let src_ag = lsx_vsrli_h::<8>(src);
        let dst_rb = lsx_vand_v(mask, dst);
        let dst_ag = lsx_vsrli_h::<8>(dst);
        let mut diff_rb = lsx_vsub_h(src_rb, dst_rb);
        let mut diff_ag = lsx_vsub_h(src_ag, dst_ag);
        let scale = lsx_vreplgr2vr_h(src_scale as i32);
        diff_rb = lsx_vmul_h(diff_rb, scale);
        diff_ag = lsx_vmul_h(diff_ag, scale);
        diff_rb = lsx_vsrli_h::<8>(diff_rb);
        diff_ag = lsx_vandn_v(mask, diff_ag);
        let diff = lsx_vor_v(diff_rb, diff_ag);
        lsx_vadd_b(dst, diff)
    }
    // Source: src/core/SkBlitRow_D32.cpp:432-455
    /// Safety: LSX available; alpha<=255,count>=0; initialized readable
    /// src and initialized readable/exclusively writable dst count-pixel spans.
    /// Original batch loads precede stores; raw overlap ordering is retained.
    #[target_feature(enable = "lsx")]
    pub unsafe fn blit_row_s32_blend(
        mut dst: *mut u32,
        mut src: *const u32,
        mut count: i32,
        alpha: u32,
    ) {
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 433; @condition b"alpha <= 255\0".as_ptr(); alpha<=255);
        let mut srcv = src.cast::<m128i>();
        let mut dstv = dst.cast::<m128i>();
        while count >= 4 {
            lsx_vst::<0>(
                SkPMLerp_LSX(
                    lsx_vld::<0>(srcv.cast()),
                    lsx_vld::<0>(dstv.cast()),
                    SkAlpha255To256(alpha),
                ),
                dstv.cast(),
            );
            srcv = srcv.add(1);
            dstv = dstv.add(1);
            count -= 4;
        }
        src = srcv.cast();
        dst = dstv.cast();
        while count > 0 {
            dst.write(SkPMLerp(src.read(), dst.read(), SkAlpha255To256(alpha)));
            src = src.add(1);
            dst = dst.add(1);
            count -= 1;
        }
    }
    // Source: src/core/SkBlitRow_D32.cpp:457-493
    #[target_feature(enable = "lsx")]
    pub unsafe fn SkBlendARGB32_LSX(src: m128i, dst: m128i, aa: u32) -> m128i {
        let alpha = SkAlpha255To256(aa);
        let src_scale = lsx_vreplgr2vr_h(alpha as i32);
        let mut dst_scale = lsx_vsrli_w::<24>(src);
        dst_scale = lsx_vmul_h(dst_scale, src_scale);
        dst_scale = lsx_vsub_w(lsx_vreplgr2vr_w(0xffff), dst_scale);
        dst_scale = lsx_vadd_w(dst_scale, lsx_vsrli_w::<8>(dst_scale));
        dst_scale = lsx_vsrli_w::<8>(dst_scale);
        dst_scale = lsx_vshuf4i_h::<0xa0>(dst_scale);
        let mask = lsx_vreplgr2vr_w(0x00ff00ff);
        let mut src_rb = lsx_vand_v(mask, src);
        let mut src_ag = lsx_vsrli_h::<8>(src);
        let mut dst_rb = lsx_vand_v(mask, dst);
        let mut dst_ag = lsx_vsrli_h::<8>(dst);
        src_rb = lsx_vmul_h(src_rb, src_scale);
        src_ag = lsx_vmul_h(src_ag, src_scale);
        dst_rb = lsx_vmul_h(dst_rb, dst_scale);
        dst_ag = lsx_vmul_h(dst_ag, dst_scale);
        dst_rb = lsx_vadd_h(src_rb, dst_rb);
        dst_ag = lsx_vadd_h(src_ag, dst_ag);
        dst_rb = lsx_vsrli_h::<8>(dst_rb);
        dst_ag = lsx_vandn_v(mask, dst_ag);
        lsx_vor_v(dst_rb, dst_ag)
    }
    // Source: src/core/SkBlitRow_D32.cpp:495-518
    /// Safety: same actual ISA/raw pixel-span conditions as non-source-alpha
    /// producer above; count>=0, alpha<=255 and no aliasing Rust references.
    #[target_feature(enable = "lsx")]
    pub unsafe fn blit_row_s32a_blend(
        mut dst: *mut u32,
        mut src: *const u32,
        mut count: i32,
        alpha: u32,
    ) {
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 496; @condition b"alpha <= 255\0".as_ptr(); alpha<=255);
        let mut srcv = src.cast::<m128i>();
        let mut dstv = dst.cast::<m128i>();
        while count >= 4 {
            lsx_vst::<0>(
                SkBlendARGB32_LSX(lsx_vld::<0>(srcv.cast()), lsx_vld::<0>(dstv.cast()), alpha),
                dstv.cast(),
            );
            srcv = srcv.add(1);
            dstv = dstv.add(1);
            count -= 4;
        }
        src = srcv.cast();
        dst = dstv.cast();
        while count > 0 {
            dst.write(SkBlendARGB32(src.read(), dst.read(), alpha));
            src = src.add(1);
            dst = dst.add(1);
            count -= 1;
        }
    }
}

// Source: src/core/SkBlitRow_D32.cpp:521-528
/// Safety: live initialized readable source and exclusively writable destination
/// for max(count,0) pixels. Native sequential overlap is permitted without Rust
/// references aliasing either range. The global alpha is at most255.
pub unsafe fn blit_row_s32_blend_scalar_raw(
    mut dst: *mut u32,
    mut src: *const u32,
    mut count: i32,
    alpha: u32,
) {
    crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 522; @condition b"alpha <= 255\0".as_ptr(); alpha<=255);
    while count > 0 {
        unsafe {
            dst.write(SkPMLerp(src.read(), dst.read(), SkAlpha255To256(alpha)));
            src = src.add(1);
            dst = dst.add(1);
        }
        count -= 1;
    }
}
// Source: src/core/SkBlitRow_D32.cpp:530-537
pub unsafe fn blit_row_s32a_blend_scalar_raw(
    mut dst: *mut u32,
    mut src: *const u32,
    mut count: i32,
    alpha: u32,
) {
    crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 531; @condition b"alpha <= 255\0".as_ptr(); alpha<=255);
    while count > 0 {
        unsafe {
            dst.write(SkBlendARGB32(src.read(), dst.read(), alpha));
            src = src.add(1);
            dst = dst.add(1);
        }
        count -= 1;
    }
}
// Compile-target bindings for the native conditional bodies above. Rust's
// target_feature cfg is the compiler's ISA contract, not a runtime CPU probe.
// The original Scalar kernels remain available explicitly; normal callers use
// these selected native kernels without copying their pixel algorithms.
// Source: src/core/SkBlitRow_D32.cpp:33-36
// Source: src/core/SkBlitRow_opts.cpp:21-22
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2"
))]
mod selected {
    pub use super::sse2::{blit_row_s32_blend, blit_row_s32a_blend};
    #[cfg(target_feature = "avx2")]
    pub(super) use crate::src::opts::SkBlitRow_opts::x86::blit_row_s32a_opaque_avx2 as blit_row_s32a_opaque;
    #[cfg(not(target_feature = "avx2"))]
    pub(super) use crate::src::opts::SkBlitRow_opts::x86::blit_row_s32a_opaque_sse2 as blit_row_s32a_opaque;
}
// Source: src/core/SkBlitRow_D32.cpp:156-158
// Source: src/core/SkBlitRow_opts.cpp:21-22
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
mod selected {
    pub use super::neon::{blit_row_s32_blend, blit_row_s32a_blend};
    pub(super) use crate::src::opts::SkBlitRow_opts::neon::blit_row_s32a_opaque;
}
// Source: src/core/SkBlitRow_D32.cpp:280-282
// Source: src/core/SkBlitRow_opts.cpp:21-22
#[cfg(all(target_arch = "loongarch64", target_feature = "lasx"))]
mod selected {
    pub use super::lasx::{blit_row_s32_blend, blit_row_s32a_blend};
    pub(super) use crate::src::opts::SkBlitRow_opts::loong::blit_row_s32a_opaque_lasx_raw as blit_row_s32a_opaque;
}
// Source: src/core/SkBlitRow_D32.cpp:400-402
// Source: src/core/SkBlitRow_opts.cpp:21-22
#[cfg(all(
    target_arch = "loongarch64",
    target_feature = "lsx",
    not(target_feature = "lasx")
))]
mod selected {
    pub use super::lsx::{blit_row_s32_blend, blit_row_s32a_blend};
    pub(super) use crate::src::opts::SkBlitRow_opts::loong::blit_row_s32a_opaque_lsx_raw as blit_row_s32a_opaque;
}
// Source: src/core/SkBlitRow_D32.cpp:520-538
// Source: src/core/SkBlitRow_opts.cpp:21-22
#[cfg(not(any(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2"
    ),
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        target_arch = "loongarch64",
        any(target_feature = "lsx", target_feature = "lasx")
    )
)))]
mod selected {
    pub use super::{
        blit_row_s32_blend_scalar_raw as blit_row_s32_blend,
        blit_row_s32a_blend_scalar_raw as blit_row_s32a_blend,
    };
    pub(super) use crate::src::opts::SkBlitRow_opts::blit_row_s32a_opaque_raw as blit_row_s32a_opaque;
}
pub use selected::{
    blit_row_s32_blend as blit_row_s32_blend_raw, blit_row_s32a_blend as blit_row_s32a_blend_raw,
};

impl SkBlitRow {
    // Source: src/core/SkBlitRow_D32.cpp:540-552
    // Compile-target native table. Runtime Init_BlitRow_ml3/lasx replacement
    // remains separate from the compiler-selected default implementation.
    pub fn Factory32_raw(mut flags: u32) -> crate::src::core::SkBlitRow::RawProc32 {
        use crate::src::core::SkBlitRow::RawProc32;
        const PROCS: [Option<RawProc32>; 4] = [
            Some(blit_row_s32_opaque_raw),
            Some(blit_row_s32_blend_raw),
            None,
            Some(blit_row_s32a_blend_raw),
        ];
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 548; @condition b"flags < std::size(kProcs)\0".as_ptr(); (flags as usize)<PROCS.len());
        flags &= PROCS.len() as u32 - 1;
        if flags == Self::kSrcPixelAlpha_Flag32 {
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            {
                return crate::src::core::SkOpts::blit_row_s32a_opaque;
            }
            #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
            {
                return crate::src::core::SkOpts::blit_row_s32a_opaque;
            }
        } else {
            PROCS[flags as usize].expect("native non-source-alpha table entry")
        }
    }
    // Source: src/core/SkBlitRow_D32.cpp:554-560
    /// Safety: dst retains max(count,0) initialized exclusively writable pixels.
    pub unsafe fn Color32_raw(dst: *mut u32, count: i32, color: u32) {
        match SkGetPackedA32(color) {
            0 => return,
            255 => {
                if count > 0 {
                    unsafe {
                        crate::src::opts::SkMemset_opts::memsetT_raw(dst, color, count as usize)
                    }
                }
                return;
            }
            _ => {}
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            (crate::src::core::SkOpts::blit_row_color32)(dst, count, color);
        }
        #[cfg(not(target_arch = "aarch64"))]
        unsafe {
            crate::src::core::SkOpts::blit_row_color32(dst, count, color);
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl SkBlitRow {
    // Source: src/core/SkBlitRow_D32.cpp:540-552
    // Architecture-specific projection of the original compile-time SSE2 table.
    // Default Factory32_raw follows the compile target. The returned unsafe proc
    // requires SSE2 at invocation; no target detection or CPU claim is invented.
    pub fn Factory32_sse2_raw(mut flags: u32) -> crate::src::core::SkBlitRow::RawProc32 {
        use crate::src::core::SkBlitRow::RawProc32;
        const PROCS: [Option<RawProc32>; 4] = [
            Some(blit_row_s32_opaque_raw),
            Some(sse2::blit_row_s32_blend),
            None,
            Some(sse2::blit_row_s32a_blend),
        ];
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 548; @condition b"flags < std::size(kProcs)\0".as_ptr(); (flags as usize)<PROCS.len());
        flags &= PROCS.len() as u32 - 1;
        if flags == Self::kSrcPixelAlpha_Flag32 {
            crate::src::opts::SkBlitRow_opts::x86::blit_row_s32a_opaque_sse2
        } else {
            PROCS[flags as usize].expect("native non-source-alpha table entry")
        }
    }
    // Source: src/core/SkBlitRow_D32.cpp:540-552
    // Same native table, with the real AVX2 SkOpts producer for source alpha.
    // Returned procedures require their documented ISA.
    pub fn Factory32_avx2_raw(mut flags: u32) -> crate::src::core::SkBlitRow::RawProc32 {
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 548; @condition b"flags < std::size(kProcs)\0".as_ptr(); flags<4);
        flags &= 3;
        if flags == Self::kSrcPixelAlpha_Flag32 {
            crate::src::opts::SkBlitRow_opts::x86::blit_row_s32a_opaque_avx2
        } else {
            Self::Factory32_sse2_raw(flags)
        }
    }
}

#[cfg(target_arch = "loongarch64")]
impl SkBlitRow {
    // Source: src/core/SkBlitRow_D32.cpp:540-552
    pub fn Factory32_lsx_raw(mut flags: u32) -> crate::src::core::SkBlitRow::RawProc32 {
        use crate::src::core::SkBlitRow::RawProc32;
        const PROCS: [Option<RawProc32>; 4] = [
            Some(blit_row_s32_opaque_raw),
            Some(lsx::blit_row_s32_blend),
            None,
            Some(lsx::blit_row_s32a_blend),
        ];
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 548; @condition b"flags < std::size(kProcs)\0".as_ptr(); (flags as usize)<PROCS.len());
        flags &= PROCS.len() as u32 - 1;
        if flags == Self::kSrcPixelAlpha_Flag32 {
            crate::src::opts::SkBlitRow_opts::loong::blit_row_s32a_opaque_lsx_raw
        } else {
            PROCS[flags as usize].expect("native non-source-alpha table entry")
        }
    }
    // Source: src/core/SkBlitRow_D32.cpp:540-552
    pub fn Factory32_lasx_raw(mut flags: u32) -> crate::src::core::SkBlitRow::RawProc32 {
        use crate::src::core::SkBlitRow::RawProc32;
        const PROCS: [Option<RawProc32>; 4] = [
            Some(blit_row_s32_opaque_raw),
            Some(lasx::blit_row_s32_blend),
            None,
            Some(lasx::blit_row_s32a_blend),
        ];
        crate::SkASSERT!(@source b"src/core/SkBlitRow_D32.cpp\0".as_ptr(), 548; @condition b"flags < std::size(kProcs)\0".as_ptr(); (flags as usize)<PROCS.len());
        flags &= PROCS.len() as u32 - 1;
        if flags == Self::kSrcPixelAlpha_Flag32 {
            crate::src::opts::SkBlitRow_opts::loong::blit_row_s32a_opaque_lasx_raw
        } else {
            PROCS[flags as usize].expect("native non-source-alpha table entry")
        }
    }
}
