// Copyright 2015 Google Inc. BSD-style license.
use crate::src::core::SkColorPriv::{SkAlpha255To256, SkGetPackedA32, SkPMSrcOver};
// Borrowed-slice projections invoke the raw Scalar native bodies below.
pub fn blit_row_s32a_opaque(dst: &mut [u32], src: &[u32], len: usize, alpha: u32) {
    assert!(len <= dst.len() && len <= src.len() && len <= i32::MAX as usize);
    unsafe { blit_row_s32a_opaque_raw(dst.as_mut_ptr(), src.as_ptr(), len as i32, alpha) }
}
pub fn blit_row_color32(dst: &mut [u32], count: usize, color: u32) {
    assert!(count <= dst.len() && count <= i32::MAX as usize);
    unsafe { blit_row_color32_raw(dst.as_mut_ptr(), count as i32, color) }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86 {
    use crate::src::core::SkColorPriv::SkPMSrcOver;
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;
    // Source: src/opts/SkBlitRow_opts.h:25-68
    #[target_feature(enable = "avx2")]
    pub unsafe fn SkPMSrcOver_AVX2(src: __m256i, dst: __m256i) -> __m256i {
        let src_a_x2 = _mm256_shuffle_epi8(
            src,
            _mm256_setr_epi8(
                3, -1, 3, -1, 7, -1, 7, -1, 11, -1, 11, -1, 15, -1, 15, -1, 3, -1, 3, -1, 7, -1, 7,
                -1, 11, -1, 11, -1, 15, -1, 15, -1,
            ),
        );
        let scale_x2 = _mm256_sub_epi16(_mm256_set1_epi16(256), src_a_x2);
        let mut rb = _mm256_and_si256(_mm256_set1_epi32(0x00ff00ff), dst);
        rb = _mm256_mullo_epi16(rb, scale_x2);
        rb = _mm256_srli_epi16::<8>(rb);
        let mut ga = _mm256_srli_epi16::<8>(dst);
        ga = _mm256_mullo_epi16(ga, scale_x2);
        ga = _mm256_andnot_si256(_mm256_set1_epi32(0x00ff00ff), ga);
        _mm256_adds_epu8(src, _mm256_or_si256(rb, ga))
    }
    // Source: src/opts/SkBlitRow_opts.h:74-88
    #[target_feature(enable = "sse2")]
    pub unsafe fn SkPMSrcOver_SSE2(src: __m128i, dst: __m128i) -> __m128i {
        let scale = _mm_sub_epi32(_mm_set1_epi32(256), _mm_srli_epi32::<24>(src));
        let scale_x2 = _mm_or_si128(_mm_slli_epi32::<16>(scale), scale);
        let mut rb = _mm_and_si128(_mm_set1_epi32(0x00ff00ff), dst);
        rb = _mm_mullo_epi16(rb, scale_x2);
        rb = _mm_srli_epi16::<8>(rb);
        let mut ga = _mm_srli_epi16::<8>(dst);
        ga = _mm_mullo_epi16(ga, scale_x2);
        ga = _mm_andnot_si128(_mm_set1_epi32(0x00ff00ff), ga);
        _mm_adds_epu8(src, _mm_or_si128(rb, ga))
    }
    // Source: src/opts/SkBlitRow_opts.h:164-187,234-239
    /// Safety: SSE2 available, alpha==255, len>=0; live initialized readable
    /// source and readable/exclusively writable destination spans of len pixels.
    /// Native load/store batch order permits raw overlap, without aliasing refs.
    #[target_feature(enable = "sse2")]
    pub unsafe fn blit_row_s32a_opaque_sse2(
        mut dst: *mut u32,
        mut src: *const u32,
        mut len: i32,
        alpha: u32,
    ) {
        crate::SkASSERT!(@source b"src/opts/SkBlitRow_opts.h\0".as_ptr(),165; @condition b"alpha == 0xFF\0".as_ptr(); alpha==255);
        // Native MSAN check becomes the explicit initialized-span contract.
        while len >= 4 {
            _mm_storeu_si128(
                dst.cast(),
                SkPMSrcOver_SSE2(_mm_loadu_si128(src.cast()), _mm_loadu_si128(dst.cast())),
            );
            src = src.add(4);
            dst = dst.add(4);
            len -= 4;
        }
        while len > 0 {
            dst.write(SkPMSrcOver(src.read(), dst.read()));
            src = src.add(1);
            dst = dst.add(1);
            len -= 1;
        }
    }
    // Source: src/opts/SkBlitRow_opts.h:164-187,234-239
    /// Safety: AVX2 available; same raw span contract as SSE2 producer above.
    /// AVX2 consumes eight-pixel batches before the native SSE2/scalar remainder.
    #[target_feature(enable = "avx2")]
    pub unsafe fn blit_row_s32a_opaque_avx2(
        mut dst: *mut u32,
        mut src: *const u32,
        mut len: i32,
        alpha: u32,
    ) {
        crate::SkASSERT!(@source b"src/opts/SkBlitRow_opts.h\0".as_ptr(),165; @condition b"alpha == 0xFF\0".as_ptr(); alpha==255);
        while len >= 8 {
            _mm256_storeu_si256(
                dst.cast(),
                SkPMSrcOver_AVX2(
                    _mm256_loadu_si256(src.cast()),
                    _mm256_loadu_si256(dst.cast()),
                ),
            );
            src = src.add(8);
            dst = dst.add(8);
            len -= 8;
        }
        blit_row_s32a_opaque_sse2(dst, src, len, alpha);
    }
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub mod neon {
    use core::arch::aarch64::*;
    // Source: src/opts/SkBlitRow_opts.h:95-98
    pub unsafe fn SkMulDiv255Round_neon8(x: uint8x8_t, y: uint8x8_t) -> uint8x8_t {
        let prod = vmull_u8(x, y);
        vraddhn_u16(prod, vrshrq_n_u16::<8>(prod))
    }
    // Source: src/opts/SkBlitRow_opts.h:100-109
    pub unsafe fn SkPMSrcOver_neon8(dst: uint8x8x4_t, src: uint8x8x4_t) -> uint8x8x4_t {
        let nalphas = vmvn_u8(src.3);
        uint8x8x4_t(
            vqadd_u8(src.0, SkMulDiv255Round_neon8(nalphas, dst.0)),
            vqadd_u8(src.1, SkMulDiv255Round_neon8(nalphas, dst.1)),
            vqadd_u8(src.2, SkMulDiv255Round_neon8(nalphas, dst.2)),
            vqadd_u8(src.3, SkMulDiv255Round_neon8(nalphas, dst.3)),
        )
    }
    // Source: src/opts/SkBlitRow_opts.h:111-115
    pub unsafe fn SkPMSrcOver_neon2(dst: uint8x8_t, src: uint8x8_t) -> uint8x8_t {
        let alpha_indices = vcreate_u8(0x0707070703030303);
        let nalphas = vmvn_u8(vtbl1_u8(src, alpha_indices));
        vqadd_u8(src, SkMulDiv255Round_neon8(nalphas, dst))
    }
    // Source: src/opts/SkBlitRow_opts.h:164-166
    // Source: src/opts/SkBlitRow_opts.h:189-212
    /// Safety: alpha==255,len>=0; source initialized/readable and destination
    /// initialized/readable/exclusively writable for len aligned u32 pixels.
    /// Raw overlap retains native8/2/1 pixel batch ordering; no Rust references
    /// alias those spans while executing. MSAN instrumentation is not installed;
    /// initialized input is an explicit unsafe caller condition, not zero-fill.
    pub unsafe fn blit_row_s32a_opaque(
        mut dst: *mut u32,
        mut src: *const u32,
        mut len: i32,
        alpha: u32,
    ) {
        crate::SkASSERT!(@source b"src/opts/SkBlitRow_opts.h\0".as_ptr(),165; @condition b"alpha == 0xFF\0".as_ptr(); alpha==255);
        while len >= 8 {
            vst4_u8(
                dst.cast(),
                SkPMSrcOver_neon8(vld4_u8(dst.cast()), vld4_u8(src.cast())),
            );
            src = src.add(8);
            dst = dst.add(8);
            len -= 8;
        }
        while len >= 2 {
            vst1_u8(
                dst.cast(),
                SkPMSrcOver_neon2(vld1_u8(dst.cast()), vld1_u8(src.cast())),
            );
            src = src.add(2);
            dst = dst.add(2);
            len -= 2;
        }
        if len != 0 {
            let result =
                SkPMSrcOver_neon2(vcreate_u8(dst.read() as u64), vcreate_u8(src.read() as u64));
            vst1_lane_u32::<0>(dst, vreinterpret_u32_u8(result));
        }
        return;
    }
}

#[cfg(target_arch = "loongarch64")]
pub mod loong {
    use crate::src::core::SkColorPriv::SkPMSrcOver;
    use core::arch::loongarch64::*;
    // The actual LoongArch stdarch API needs the target's stdarch_loongarch
    // feature gate. Root owns crate wiring. No Scalar/ARM operation is relabeled.
    // Source: src/opts/SkBlitRow_opts.h:122-137
    #[target_feature(enable = "lasx")]
    pub unsafe fn SkPMSrcOver_LASX(src: m256i, dst: m256i) -> m256i {
        let mut val = lasx_xvreplgr2vr_w(256);
        let scale = lasx_xvsub_w(val, lasx_xvsrli_w::<24>(src));
        let scale_x2 = lasx_xvor_v(lasx_xvslli_w::<16>(scale), scale);
        val = lasx_xvreplgr2vr_w(0x00ff00ff);
        let mut rb = lasx_xvand_v(val, dst);
        rb = lasx_xvmul_h(rb, scale_x2);
        rb = lasx_xvsrli_h::<8>(rb);
        let mut ga = lasx_xvsrli_h::<8>(dst);
        ga = lasx_xvmul_h(ga, scale_x2);
        ga = lasx_xvandn_v(val, ga);
        lasx_xvsadd_bu(src, lasx_xvor_v(rb, ga))
    }
    // Source: src/opts/SkBlitRow_opts.h:143-158
    #[target_feature(enable = "lsx")]
    pub unsafe fn SkPMSrcOver_LSX(src: m128i, dst: m128i) -> m128i {
        let mut val = lsx_vreplgr2vr_w(256);
        let scale = lsx_vsub_w(val, lsx_vsrli_w::<24>(src));
        let scale_x2 = lsx_vor_v(lsx_vslli_w::<16>(scale), scale);
        val = lsx_vreplgr2vr_w(0x00ff00ff);
        let mut rb = lsx_vand_v(val, dst);
        rb = lsx_vmul_h(rb, scale_x2);
        rb = lsx_vsrli_h::<8>(rb);
        let mut ga = lsx_vsrli_h::<8>(dst);
        ga = lsx_vmul_h(ga, scale_x2);
        ga = lsx_vandn_v(val, ga);
        lsx_vsadd_bu(src, lsx_vor_v(rb, ga))
    }
    // Source: src/opts/SkBlitRow_opts.h:164-166,215-222,225-232,234-239
    /// Safety: LASX/LSX available, alpha==255,len>=0. Source initialized/readable
    /// and destination initialized/readable/exclusively writable for len pixels.
    /// Native eight-pixel then four-pixel then scalar tail ordering retained.
    #[target_feature(enable = "lasx")]
    pub unsafe fn blit_row_s32a_opaque_lasx_raw(
        mut dst: *mut u32,
        mut src: *const u32,
        mut len: i32,
        alpha: u32,
    ) {
        crate::SkASSERT!(@source b"src/opts/SkBlitRow_opts.h\0".as_ptr(),165; @condition b"alpha == 0xFF\0".as_ptr(); alpha==255);
        // Native MSAN read check is the initialized-span unsafe caller precondition.
        while len >= 8 {
            lasx_xvst::<0>(
                SkPMSrcOver_LASX(lasx_xvld::<0>(src.cast()), lasx_xvld::<0>(dst.cast())),
                dst.cast(),
            );
            src = src.add(8);
            dst = dst.add(8);
            len -= 8;
        }
        while len >= 4 {
            lsx_vst::<0>(
                SkPMSrcOver_LSX(lsx_vld::<0>(src.cast()), lsx_vld::<0>(dst.cast())),
                dst.cast(),
            );
            src = src.add(4);
            dst = dst.add(4);
            len -= 4;
        }
        while len > 0 {
            dst.write(SkPMSrcOver(src.read(), dst.read()));
            src = src.add(1);
            dst = dst.add(1);
            len -= 1;
        }
    }
    // Source: src/opts/SkBlitRow_opts.h:164-166,225-232,234-239
    /// Safety: same pixel/alpha/overlap contract, requiring LSX only.
    #[target_feature(enable = "lsx")]
    pub unsafe fn blit_row_s32a_opaque_lsx_raw(
        mut dst: *mut u32,
        mut src: *const u32,
        mut len: i32,
        alpha: u32,
    ) {
        crate::SkASSERT!(@source b"src/opts/SkBlitRow_opts.h\0".as_ptr(),165; @condition b"alpha == 0xFF\0".as_ptr(); alpha==255);
        while len >= 4 {
            lsx_vst::<0>(
                SkPMSrcOver_LSX(lsx_vld::<0>(src.cast()), lsx_vld::<0>(dst.cast())),
                dst.cast(),
            );
            src = src.add(4);
            dst = dst.add(4);
            len -= 4;
        }
        while len > 0 {
            dst.write(SkPMSrcOver(src.read(), dst.read()));
            src = src.add(1);
            dst = dst.add(1);
            len -= 1;
        }
    }
}

// Source: src/opts/SkBlitRow_opts.h:164-166
// Source: src/opts/SkBlitRow_opts.h:234-239
/// Safety: initialized/readable source, initialized/exclusively writable dst for
/// max(len,0) pixels, alpha==255. Raw sequential overlap follows native Scalar;
/// no Rust references alias these spans. No synthetic MSAN zero initialization.
pub unsafe fn blit_row_s32a_opaque_raw(
    mut dst: *mut u32,
    mut src: *const u32,
    mut len: i32,
    alpha: u32,
) {
    crate::SkASSERT!(@source b"src/opts/SkBlitRow_opts.h\0".as_ptr(),165; @condition b"alpha == 0xFF\0".as_ptr(); alpha==255);
    while len > 0 {
        unsafe {
            dst.write(SkPMSrcOver(src.read(), dst.read()));
            src = src.add(1);
            dst = dst.add(1);
        }
        len -= 1;
    }
}
// Source: src/opts/SkBlitRow_opts.h:243-275
/// Safety: dst retains max(count,0) initialized exclusively writable u32 pixels;
/// four-pixel batch loads precede stores exactly as in the native skvx kernel.
pub unsafe fn blit_row_color32_raw(mut dst: *mut u32, mut count: i32, color: u32) {
    const N: usize = 4;
    let kernel = |src: [u32; N]| {
        let inv_a = SkAlpha255To256(255 - SkGetPackedA32(color));
        crate::SkASSERT!(@source b"src/opts/SkBlitRow_opts.h\0".as_ptr(),252; @condition b"0 < invA && invA < 256\0".as_ptr(); inv_a>0&&inv_a<256);
        let c = color.to_ne_bytes();
        let mut result = [0u32; N];
        for i in 0..N {
            let v = src[i].to_ne_bytes();
            let mut r = [0u8; 4];
            for j in 0..4 {
                r[j] = (((v[j] as u16 * inv_a as u16) >> 8) + c[j] as u16) as u8;
            }
            result[i] = u32::from_ne_bytes(r);
        }
        result
    };
    while count >= N as i32 {
        unsafe {
            let pixels = core::array::from_fn(|i| dst.add(i).read());
            let out = kernel(pixels);
            for i in 0..N {
                dst.add(i).write(out[i]);
            }
            dst = dst.add(N);
        }
        count -= N as i32;
    }
    while count > 0 {
        unsafe {
            dst.write(kernel([dst.read(); N])[0]);
            dst = dst.add(1);
        }
        count -= 1;
    }
}
