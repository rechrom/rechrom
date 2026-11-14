// Copyright 2018 Google LLC. BSD-style license.
// Source: src/opts/SkBitmapProcState_opts.h:36-42
// Scalar packed coordinate decoding used by the native fallback/NEON branch;
// tuple values replace C++ output pointers. Other SIMD instantiations pending.
pub fn decode_packed_coordinates_and_weight(packed: u32) -> (u32, u32, u32) {
    (packed >> 18, packed & 0x3fff, (packed >> 14) & 0xf)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod ssse3 {
    use super::decode_packed_coordinates_and_weight;
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;
    // Source: src/opts/SkBitmapProcState_opts.h:47-180
    /// Safety: SSSE3 is available and source pixel/packed-coordinate/output
    /// storage obeys the canonical state and native count/coordinate bounds.
    #[target_feature(enable = "ssse3")]
    pub unsafe fn S32_alpha_D32_filter_DX(
        s: &crate::src::core::SkBitmapProcState::SkBitmapProcState,
        xy: &[u32],
        mut count: i32,
        colors: &mut [u32],
    ) {
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),49; @condition b"count > 0 && colors != nullptr\0".as_ptr(); count>0 && !colors.as_ptr().is_null());
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),50; @condition b"s.fBilerp\0".as_ptr(); s.fBilerp.assume_init());
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),51; @condition b"kN32_SkColorType == s.fPixmap.colorType()\0".as_ptr(); s.fPixmap.colorType()==crate::include::core::SkColorType::kN32_SkColorType);
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),52; @condition b"s.fAlphaScale <= 256\0".as_ptr(); s.fAlphaScale.assume_init()<=256);
        let interpolate_in_x =
            |A0: u32, A1: u32, B0: u32, B1: u32, interlaced_x_weights: __m128i| {
                let interlaced_A =
                    _mm_unpacklo_epi8(_mm_cvtsi32_si128(A0 as i32), _mm_cvtsi32_si128(A1 as i32));
                let interlaced_B =
                    _mm_unpacklo_epi8(_mm_cvtsi32_si128(B0 as i32), _mm_cvtsi32_si128(B1 as i32));
                _mm_maddubs_epi16(
                    _mm_unpacklo_epi64(interlaced_A, interlaced_B),
                    interlaced_x_weights,
                )
            };
        let interpolate_in_x_and_y = |A0: u32,
                                      A1: u32,
                                      A2: u32,
                                      A3: u32,
                                      B0: u32,
                                      B1: u32,
                                      B2: u32,
                                      B3: u32,
                                      interlaced_x_weights: __m128i,
                                      wy: i32| {
            let top = interpolate_in_x(A0, A1, B0, B1, interlaced_x_weights);
            let bot = interpolate_in_x(A2, A3, B2, B3, interlaced_x_weights);
            let mut px = _mm_add_epi16(
                _mm_slli_epi16::<4>(top),
                _mm_mullo_epi16(_mm_sub_epi16(bot, top), _mm_set1_epi16(wy as i16)),
            );
            px = _mm_srli_epi16::<8>(px);
            if s.fAlphaScale.assume_init() < 256 {
                px = _mm_srli_epi16::<8>(_mm_mullo_epi16(
                    px,
                    _mm_set1_epi16(s.fAlphaScale.assume_init() as i16),
                ));
            }
            px
        };
        let (y0, y1, wy) = decode_packed_coordinates_and_weight(xy[0]);
        let row0 = s
            .fPixmap
            .addr()
            .add(y0 as usize * s.fPixmap.rowBytes())
            .cast::<u32>();
        let row1 = s
            .fPixmap
            .addr()
            .add(y1 as usize * s.fPixmap.rowBytes())
            .cast::<u32>();
        let mut xy_offset = 1;
        let mut color_offset = 0;
        while count >= 4 {
            // Each entire array is written by its unaligned vector store before
            // assume_init exposes its integer lanes; no uninitialized read.
            let mut x0 = core::mem::MaybeUninit::<[i32; 4]>::uninit();
            let mut x1 = core::mem::MaybeUninit::<[i32; 4]>::uninit();
            let packed = _mm_loadu_si128(xy.as_ptr().add(xy_offset).cast());
            _mm_storeu_si128(x0.as_mut_ptr().cast(), _mm_srli_epi32::<18>(packed));
            _mm_storeu_si128(
                x1.as_mut_ptr().cast(),
                _mm_and_si128(packed, _mm_set1_epi32(0x3fff)),
            );
            let x0 = x0.assume_init();
            let x1 = x1.assume_init();
            let wx = _mm_and_si128(_mm_srli_epi32::<14>(packed), _mm_set1_epi32(0xf));
            let wr = _mm_shuffle_epi8(
                wx,
                _mm_setr_epi8(0, 0, 0, 0, 4, 4, 4, 4, 8, 8, 8, 8, 12, 12, 12, 12),
            );
            let wl = _mm_sub_epi8(_mm_set1_epi8(16), wr);
            let interlaced_x_weights_AB = _mm_unpacklo_epi8(wl, wr);
            let interlaced_x_weights_CD = _mm_unpackhi_epi8(wl, wr);
            const A: usize = 0;
            const B: usize = 1;
            const C: usize = 2;
            const D: usize = 3;
            let AB = interpolate_in_x_and_y(
                *row0.add(x0[A] as usize),
                *row0.add(x1[A] as usize),
                *row1.add(x0[A] as usize),
                *row1.add(x1[A] as usize),
                *row0.add(x0[B] as usize),
                *row0.add(x1[B] as usize),
                *row1.add(x0[B] as usize),
                *row1.add(x1[B] as usize),
                interlaced_x_weights_AB,
                wy as i32,
            );
            let CD = interpolate_in_x_and_y(
                *row0.add(x0[C] as usize),
                *row0.add(x1[C] as usize),
                *row1.add(x0[C] as usize),
                *row1.add(x1[C] as usize),
                *row0.add(x0[D] as usize),
                *row0.add(x1[D] as usize),
                *row1.add(x0[D] as usize),
                *row1.add(x1[D] as usize),
                interlaced_x_weights_CD,
                wy as i32,
            );
            _mm_storeu_si128(
                colors.as_mut_ptr().add(color_offset).cast(),
                _mm_packus_epi16(AB, CD),
            );
            xy_offset += 4;
            color_offset += 4;
            count -= 4;
        }
        loop {
            let old = count;
            count -= 1;
            if old <= 0 {
                break;
            }
            let (x0, x1, wx) = decode_packed_coordinates_and_weight(xy[xy_offset]);
            xy_offset += 1;
            let wr = _mm_set1_epi8(wx as i8);
            let wl = _mm_sub_epi8(_mm_set1_epi8(16), wr);
            let interlaced_x_weights = _mm_unpacklo_epi8(wl, wr);
            let A = interpolate_in_x_and_y(
                *row0.add(x0 as usize),
                *row0.add(x1 as usize),
                *row1.add(x0 as usize),
                *row1.add(x1 as usize),
                0,
                0,
                0,
                0,
                interlaced_x_weights,
                wy as i32,
            );
            colors[color_offset] =
                _mm_cvtsi128_si32(_mm_packus_epi16(A, _mm_setzero_si128())) as u32;
            color_offset += 1;
        }
    }
}
#[cfg(all(target_arch = "x86_64", target_feature = "ssse3"))]
pub use ssse3::S32_alpha_D32_filter_DX;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod sse2 {
    use super::decode_packed_coordinates_and_weight;
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;
    // Source: src/opts/SkBitmapProcState_opts.h:186-261
    /// Safety: SSE2 is available; the canonical state's source pixels stay live
    /// and all decoded coordinates address readable N32 pixels. Input/output
    /// slices cover one Y entry plus count X entries and count output pixels.
    #[target_feature(enable = "sse2")]
    pub unsafe fn S32_alpha_D32_filter_DX(
        s: &crate::src::core::SkBitmapProcState::SkBitmapProcState,
        xy: &[u32],
        mut count: i32,
        colors: &mut [u32],
    ) {
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),188; @condition b"count > 0 && colors != nullptr\0".as_ptr(); count>0 && !colors.as_ptr().is_null());
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),189; @condition b"s.fBilerp\0".as_ptr(); s.fBilerp.assume_init());
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),190; @condition b"kN32_SkColorType == s.fPixmap.colorType()\0".as_ptr(); s.fPixmap.colorType()==crate::include::core::SkColorType::kN32_SkColorType);
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),191; @condition b"s.fAlphaScale <= 256\0".as_ptr(); s.fAlphaScale.assume_init()<=256);
        let (y0, y1, wy) = decode_packed_coordinates_and_weight(xy[0]);
        let row0 = s
            .fPixmap
            .addr()
            .add(y0 as usize * s.fPixmap.rowBytes())
            .cast::<u32>();
        let row1 = s
            .fPixmap
            .addr()
            .add(y1 as usize * s.fPixmap.rowBytes())
            .cast::<u32>();
        let allY = _mm_unpacklo_epi64(_mm_set1_epi16(wy as i16), _mm_set1_epi16(16 - wy as i16));
        let mut xy_offset = 1;
        let mut color_offset = 0;
        loop {
            let old = count;
            count -= 1;
            if old <= 0 {
                break;
            }
            let (x0, x1, wx) = decode_packed_coordinates_and_weight(xy[xy_offset]);
            xy_offset += 1;
            let tl = _mm_cvtsi32_si128(*row0.add(x0 as usize) as i32);
            let tr = _mm_cvtsi32_si128(*row0.add(x1 as usize) as i32);
            let bl = _mm_cvtsi32_si128(*row1.add(x0 as usize) as i32);
            let br = _mm_cvtsi32_si128(*row1.add(x1 as usize) as i32);
            let L = _mm_unpacklo_epi8(_mm_unpacklo_epi32(bl, tl), _mm_setzero_si128());
            let R = _mm_unpacklo_epi8(_mm_unpacklo_epi32(br, tr), _mm_setzero_si128());
            let inner = _mm_add_epi16(
                _mm_slli_epi16::<4>(L),
                _mm_mullo_epi16(_mm_sub_epi16(R, L), _mm_set1_epi16(wx as i16)),
            );
            let sum_in_x = _mm_mullo_epi16(inner, allY);
            let mut sum = _mm_add_epi16(sum_in_x, _mm_srli_si128::<8>(sum_in_x));
            sum = _mm_srli_epi16::<8>(sum);
            if s.fAlphaScale.assume_init() < 256 {
                sum = _mm_mullo_epi16(sum, _mm_set1_epi16(s.fAlphaScale.assume_init() as i16));
                sum = _mm_srli_epi16::<8>(sum);
            }
            colors[color_offset] =
                _mm_cvtsi128_si32(_mm_packus_epi16(sum, _mm_setzero_si128())) as u32;
            color_offset += 1;
        }
    }
}
// Native build-feature selection keeps SSE2 and SSSE3 implementations distinct.
#[cfg(all(target_arch = "x86_64", not(target_feature = "ssse3")))]
pub use sse2::S32_alpha_D32_filter_DX;

// Source: src/opts/SkBitmapProcState_opts.h:438-477
#[cfg(target_arch = "aarch64")]
pub mod neon {
    use std::arch::aarch64::*;

    #[inline(always)]
    unsafe fn bilerp_eight_channels(
        a00: uint16x8_t,
        a01: uint16x8_t,
        a10: uint16x8_t,
        a11: uint16x8_t,
        x: uint16x8_t,
        y: uint16x8_t,
        scale: u32,
    ) -> uint16x8_t {
        let sixteen = vdupq_n_u16(16);
        let inverse_x = vsubq_u16(sixteen, x);
        let inverse_y = vsubq_u16(sixteen, y);
        let left = vmlaq_u16(vmulq_u16(a00, inverse_y), a10, y);
        let right = vmlaq_u16(vmulq_u16(a01, inverse_y), a11, y);
        let mut result = vmlaq_u16(vmulq_u16(left, inverse_x), right, x);
        if scale < 256 {
            result = vshrq_n_u16::<8>(result);
            result = vmulq_u16(result, vdupq_n_u16(scale as u16));
        }
        result
    }

    #[inline(always)]
    unsafe fn four_pixel_weights(values: [u16; 4]) -> (uint16x8_t, uint16x8_t) {
        let packed = vld1_u16(values.as_ptr());
        (
            vcombine_u16(
                vdup_n_u16(vget_lane_u16::<0>(packed)),
                vdup_n_u16(vget_lane_u16::<1>(packed)),
            ),
            vcombine_u16(
                vdup_n_u16(vget_lane_u16::<2>(packed)),
                vdup_n_u16(vget_lane_u16::<3>(packed)),
            ),
        )
    }

    #[inline(always)]
    unsafe fn filter_four_vectors_by_alpha(
        a00: uint32x4_t,
        a01: uint32x4_t,
        a10: uint32x4_t,
        a11: uint32x4_t,
        x: [u16; 4],
        y: [u16; 4],
        dst: *mut u32,
        scale: u32,
    ) {
        let pixels = |values: uint32x4_t| {
            let bytes = vreinterpretq_u8_u32(values);
            (vmovl_u8(vget_low_u8(bytes)), vmovl_u8(vget_high_u8(bytes)))
        };
        let (a00_low, a00_high) = pixels(a00);
        let (a01_low, a01_high) = pixels(a01);
        let (a10_low, a10_high) = pixels(a10);
        let (a11_low, a11_high) = pixels(a11);
        let (x_low, x_high) = four_pixel_weights(x);
        let (y_low, y_high) = four_pixel_weights(y);
        let low = bilerp_eight_channels(a00_low, a01_low, a10_low, a11_low, x_low, y_low, scale);
        let high = bilerp_eight_channels(
            a00_high, a01_high, a10_high, a11_high, x_high, y_high, scale,
        );
        let result = vcombine_u8(vshrn_n_u16::<8>(low), vshrn_n_u16::<8>(high));
        vst1q_u32(dst, vreinterpretq_u32_u8(result));
    }

    /// Four-pixel form of the same native NEON bilerp producer.  Each weight
    /// is replicated across that pixel's BGRA lanes; arithmetic order and
    /// narrowing match `filter_and_scale_by_alpha` exactly.
    pub(crate) unsafe fn filter_four_by_alpha(
        a00: [u32; 4],
        a01: [u32; 4],
        a10: [u32; 4],
        a11: [u32; 4],
        x: [u16; 4],
        y: [u16; 4],
        dst: *mut u32,
        scale: u32,
    ) {
        filter_four_vectors_by_alpha(
            vld1q_u32(a00.as_ptr()),
            vld1q_u32(a01.as_ptr()),
            vld1q_u32(a10.as_ptr()),
            vld1q_u32(a11.as_ptr()),
            x,
            y,
            dst,
            scale,
        );
    }

    /// Fast form for the common shallow-affine run where four adjacent output
    /// samples address the same two source rows and consecutive source columns.
    /// The bilerp weights remain per-pixel, so this is bit-identical to the
    /// general translated SkBitmapProcState DXDY producer.
    pub(crate) unsafe fn filter_four_contiguous_by_alpha(
        row0: *const u32,
        row1: *const u32,
        x0: usize,
        x: [u16; 4],
        y: [u16; 4],
        dst: *mut u32,
        scale: u32,
    ) {
        filter_four_vectors_by_alpha(
            vld1q_u32(row0.add(x0)),
            vld1q_u32(row0.add(x0 + 1)),
            vld1q_u32(row1.add(x0)),
            vld1q_u32(row1.add(x0 + 1)),
            x,
            y,
            dst,
            scale,
        );
    }

    /// Four-pixel specialization of the bilerp-alpha plus DstIn channel scale
    /// used by the software renderer's fractional mask tiles.
    pub(crate) unsafe fn dst_in_bilerp_four(
        a00: [u32; 4],
        a10: [u32; 4],
        a01: [u32; 4],
        a11: [u32; 4],
        wx0: f32,
        fx: f32,
        wy0: f32,
        fy: f32,
        paint_scale: f32,
        destination: *mut u32,
    ) {
        let alpha = |values: &[u32; 4]| vcvtq_f32_u32(vld1q_u32(values.as_ptr()));
        let a00 = vmulq_n_f32(vmulq_n_f32(alpha(&a00), wx0), wy0);
        let a10 = vmulq_n_f32(vmulq_n_f32(alpha(&a10), fx), wy0);
        let a01 = vmulq_n_f32(vmulq_n_f32(alpha(&a01), wx0), fy);
        let a11 = vmulq_n_f32(vmulq_n_f32(alpha(&a11), fx), fy);
        let sampled = vaddq_f32(vaddq_f32(vaddq_f32(a00, a10), a01), a11);
        let scale = vmulq_n_f32(sampled, paint_scale);
        let bytes = vreinterpretq_u8_u32(vld1q_u32(destination));
        let scale_two = |channels: uint8x8_t, first: f32, second: f32| {
            let channels = vmovl_u8(channels);
            let scale_channels = |channels: uint16x4_t, scale: f32| {
                let channels = vcvtq_f32_u32(vmovl_u16(channels));
                let rounded = vaddq_f32(vmulq_n_f32(channels, scale), vdupq_n_f32(0.5));
                vminq_u32(vcvtq_u32_f32(rounded), vdupq_n_u32(255))
            };
            let first = scale_channels(vget_low_u16(channels), first);
            let second = scale_channels(vget_high_u16(channels), second);
            vmovn_u16(vcombine_u16(vmovn_u32(first), vmovn_u32(second)))
        };
        let low = scale_two(
            vget_low_u8(bytes),
            vgetq_lane_f32::<0>(scale),
            vgetq_lane_f32::<1>(scale),
        );
        let high = scale_two(
            vget_high_u8(bytes),
            vgetq_lane_f32::<2>(scale),
            vgetq_lane_f32::<3>(scale),
        );
        vst1q_u32(destination, vreinterpretq_u32_u8(vcombine_u8(low, high)));
    }

    pub fn filter_and_scale_by_alpha(
        x: u32,
        y: u32,
        a00: u32,
        a01: u32,
        a10: u32,
        a11: u32,
        dst: &mut u32,
        scale: u32,
    ) {
        // The referenced destination is initialized/aligned writable storage;
        // all remaining operations use register lanes. Native input contracts
        // bound the interpolation weights and alpha scale.
        unsafe {
            let vy = vdup_n_u8(y as u8);
            let vconst16_8 = vdup_n_u8(16);
            let v16_y = vsub_u8(vconst16_8, vy);
            let mut va0 = vdup_n_u32(a00);
            let mut va1 = vdup_n_u32(a10);
            va0 = vset_lane_u32::<1>(a01, va0);
            va1 = vset_lane_u32::<1>(a11, va1);
            let tmp1 = vmull_u8(vreinterpret_u8_u32(va0), v16_y);
            let tmp2 = vmull_u8(vreinterpret_u8_u32(va1), vy);
            let vx = vdup_n_u16(x as u16);
            let vconst16_16 = vdup_n_u16(16);
            let v16_x = vsub_u16(vconst16_16, vx);
            let mut tmp = vmul_u16(vget_high_u16(tmp1), vx);
            tmp = vmla_u16(tmp, vget_high_u16(tmp2), vx);
            tmp = vmla_u16(tmp, vget_low_u16(tmp1), v16_x);
            tmp = vmla_u16(tmp, vget_low_u16(tmp2), v16_x);
            if scale < 256 {
                let vscale = vdup_n_u16(scale as u16);
                tmp = vshr_n_u16::<8>(tmp);
                tmp = vmul_u16(tmp, vscale);
            }
            let vres = vshrn_n_u16::<8>(vcombine_u16(tmp, vcreate_u16(0)));
            vst1_lane_u32::<0>(dst, vreinterpret_u32_u8(vres));
        }
    }
}
// Rust architecture dispatch mirrors the native preprocessor selection;
// arm32, x86 and LoongArch SIMD variants remain pending in this target.
#[cfg(target_arch = "aarch64")]
pub use neon::filter_and_scale_by_alpha;

// Source: src/opts/SkBitmapProcState_opts.h:479-512
// Native non-NEON fallback; unsigned wrapping operations retain C++ arithmetic.
#[cfg(not(target_arch = "aarch64"))]
pub fn filter_and_scale_by_alpha(
    x: u32,
    y: u32,
    a00: u32,
    a01: u32,
    a10: u32,
    a11: u32,
    dstColor: &mut u32,
    alphaScale: u32,
) {
    crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),484; @condition b"(unsigned)x <= 0xF\0".as_ptr(); x <= 0xf);
    crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),485; @condition b"(unsigned)y <= 0xF\0".as_ptr(); y <= 0xf);
    crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),486; @condition b"alphaScale <= 256\0".as_ptr(); alphaScale <= 256);
    let xy = x * y;
    let mask = 0xff00ff;
    let mut scale = (256i32 - 16 * y as i32 - 16 * x as i32 + xy as i32) as u32;
    let mut lo = (a00 & mask).wrapping_mul(scale);
    let mut hi = ((a00 >> 8) & mask).wrapping_mul(scale);
    scale = 16 * x - xy;
    lo = lo.wrapping_add((a01 & mask).wrapping_mul(scale));
    hi = hi.wrapping_add(((a01 >> 8) & mask).wrapping_mul(scale));
    scale = 16 * y - xy;
    lo = lo.wrapping_add((a10 & mask).wrapping_mul(scale));
    hi = hi.wrapping_add(((a10 >> 8) & mask).wrapping_mul(scale));
    lo = lo.wrapping_add((a11 & mask).wrapping_mul(xy));
    hi = hi.wrapping_add(((a11 >> 8) & mask).wrapping_mul(xy));
    if alphaScale < 256 {
        lo = ((lo >> 8) & mask).wrapping_mul(alphaScale);
        hi = ((hi >> 8) & mask).wrapping_mul(alphaScale);
    }
    *dstColor = ((lo >> 8) & mask) | (hi & !mask);
}

// Source: src/opts/SkBitmapProcState_opts.h:516-540
#[cfg(not(target_arch = "x86_64"))]
// Native scalar fallback. The canonical SkBitmapProcState state object and its
// fields must be provided by its owner; no projected replacement is introduced.
/// Safety: every decoded coordinate must address valid, readable source pixel
/// storage in s.fPixmap, which must remain live throughout this call.
/// State.setup must have succeeded: init writes fBilerp and chooseProcs writes
/// fAlphaScale before installing this proc. Constructor/failed-setup states do
/// not establish those initialized fields; no implicit zero initialization.
pub unsafe fn S32_alpha_D32_filter_DX(
    s: &crate::src::core::SkBitmapProcState::SkBitmapProcState,
    xy: &[u32],
    mut count: i32,
    colors: &mut [u32],
) {
    crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),519; @condition b"count > 0 && colors != nullptr\0".as_ptr(); count>0 && !colors.as_ptr().is_null());
    crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),520; @condition b"s.fBilerp\0".as_ptr(); s.fBilerp.assume_init());
    crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),521; @condition b"4 == s.fPixmap.info().bytesPerPixel()\0".as_ptr(); s.fPixmap.info().bytesPerPixel()==4);
    crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),522; @condition b"s.fAlphaScale <= 256\0".as_ptr(); s.fAlphaScale.assume_init()<=256);
    let (y0, y1, wy) = decode_packed_coordinates_and_weight(xy[0]);
    let row0 = s
        .fPixmap
        .addr()
        .add(y0 as usize * s.fPixmap.rowBytes())
        .cast::<u32>();
    let row1 = s
        .fPixmap
        .addr()
        .add(y1 as usize * s.fPixmap.rowBytes())
        .cast::<u32>();
    let mut xy_offset = 1;
    let mut color_offset = 0;
    loop {
        let previous_count = count;
        count -= 1;
        if previous_count <= 0 {
            break;
        }
        let (x0, x1, wx) = decode_packed_coordinates_and_weight(xy[xy_offset]);
        xy_offset += 1;
        filter_and_scale_by_alpha(
            wx,
            wy,
            *row0.add(x0 as usize),
            *row0.add(x1 as usize),
            *row1.add(x0 as usize),
            *row1.add(x1 as usize),
            &mut colors[color_offset],
            s.fAlphaScale.assume_init() as u32,
        );
        color_offset += 1;
    }
}

// Source: src/opts/SkBitmapProcState_opts.h:545-571
#[cfg(target_arch = "aarch64")]
/// Safety: each packed source coordinate must address initialized readable N32
/// storage in the state's Pixmap for the whole operation. Input/output slices
/// must cover count coordinate pairs and pixels, respectively. State.setup
/// must have succeeded: init initializes fBilerp and chooseProcs initializes
/// fAlphaScale before this proc is installed. Do not call on a constructor-only
/// or failed-setup state; MaybeUninit is read only under that native contract.
pub unsafe fn S32_alpha_D32_filter_DXDY(
    s: &crate::src::core::SkBitmapProcState::SkBitmapProcState,
    xy: &[u32],
    mut count: i32,
    colors: &mut [u32],
) {
    crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),548; @condition b"count > 0 && colors != nullptr\0".as_ptr(); count>0 && !colors.as_ptr().is_null());
    crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),549; @condition b"s.fBilerp\0".as_ptr(); s.fBilerp.assume_init());
    crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),550; @condition b"4 == s.fPixmap.info().bytesPerPixel()\0".as_ptr(); s.fPixmap.info().bytesPerPixel()==4);
    crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),551; @condition b"s.fAlphaScale <= 256\0".as_ptr(); s.fAlphaScale.assume_init()<=256);
    let src = s.fPixmap.addr();
    let rb = s.fPixmap.rowBytes();
    let mut xy_offset = 0;
    let mut color_offset = 0;
    static CONTIGUOUS_SIMD_ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let contiguous_simd = *CONTIGUOUS_SIMD_ENABLED
        .get_or_init(|| std::env::var("BROWSER_DISABLE_N32_CONTIGUOUS_SIMD").as_deref() != Ok("1"));
    while count >= 4 {
        let mut x0s = [0usize; 4];
        let mut x1s = [0usize; 4];
        let mut y0s = [0usize; 4];
        let mut y1s = [0usize; 4];
        let mut x = [0; 4];
        let mut y = [0; 4];
        for lane in 0..4 {
            let (y0, y1, wy) = decode_packed_coordinates_and_weight(xy[xy_offset]);
            xy_offset += 1;
            let (x0, x1, wx) = decode_packed_coordinates_and_weight(xy[xy_offset]);
            xy_offset += 1;
            x0s[lane] = x0 as usize;
            x1s[lane] = x1 as usize;
            y0s[lane] = y0 as usize;
            y1s[lane] = y1 as usize;
            x[lane] = wx as u16;
            y[lane] = wy as u16;
        }
        let contiguous = (1..4).all(|lane| {
            x0s[lane] == x0s[0] + lane
                && x1s[lane] == x1s[0] + lane
                && y0s[lane] == y0s[0]
                && y1s[lane] == y1s[0]
        }) && x1s[0] == x0s[0] + 1;
        if contiguous_simd && contiguous {
            let row0 = src.add(y0s[0] * rb).cast::<u32>();
            let row1 = src.add(y1s[0] * rb).cast::<u32>();
            neon::filter_four_contiguous_by_alpha(
                row0,
                row1,
                x0s[0],
                x,
                y,
                colors.as_mut_ptr().add(color_offset),
                s.fAlphaScale.assume_init() as u32,
            );
        } else {
            let mut a00 = [0; 4];
            let mut a01 = [0; 4];
            let mut a10 = [0; 4];
            let mut a11 = [0; 4];
            for lane in 0..4 {
                let row0 = src.add(y0s[lane] * rb).cast::<u32>();
                let row1 = src.add(y1s[lane] * rb).cast::<u32>();
                a00[lane] = row0.add(x0s[lane]).read();
                a01[lane] = row0.add(x1s[lane]).read();
                a10[lane] = row1.add(x0s[lane]).read();
                a11[lane] = row1.add(x1s[lane]).read();
            }
            neon::filter_four_by_alpha(
                a00,
                a01,
                a10,
                a11,
                x,
                y,
                colors.as_mut_ptr().add(color_offset),
                s.fAlphaScale.assume_init() as u32,
            );
        }
        color_offset += 4;
        count -= 4;
    }
    loop {
        let old = count;
        count -= 1;
        if old <= 0 {
            break;
        }
        let (y0, y1, wy) = decode_packed_coordinates_and_weight(xy[xy_offset]);
        xy_offset += 1;
        let (x0, x1, wx) = decode_packed_coordinates_and_weight(xy[xy_offset]);
        xy_offset += 1;
        let row0 = src.add(y0 as usize * rb).cast::<u32>();
        let row1 = src.add(y1 as usize * rb).cast::<u32>();
        filter_and_scale_by_alpha(
            wx,
            wy,
            *row0.add(x0 as usize),
            *row0.add(x1 as usize),
            *row1.add(x0 as usize),
            *row1.add(x1 as usize),
            &mut colors[color_offset],
            s.fAlphaScale.assume_init() as u32,
        );
        color_offset += 1;
    }
}
// Source: src/opts/SkBitmapProcState_opts.h:574-576
// Native unsupported-architecture entry is a real null function pointer.
#[cfg(not(target_arch = "aarch64"))]
pub const S32_alpha_D32_filter_DXDY: Option<
    unsafe fn(&crate::src::core::SkBitmapProcState::SkBitmapProcState, &[u32], i32, &mut [u32]),
> = None;

#[cfg(target_arch = "loongarch64")]
pub mod lasx {
    use core::arch::loongarch64::*;
    // Source: src/opts/SkBitmapProcState_opts.h:265-345
    #[target_feature(enable = "lasx")]
    pub unsafe fn S32_alpha_D32_filter_DX(
        s: &crate::src::core::SkBitmapProcState::SkBitmapProcState,
        xy: &[u32],
        mut count: i32,
        colors: &mut [u32],
    ) {
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),267; @condition b"count > 0 && colors != nullptr\0".as_ptr(); count>0 && !colors.as_ptr().is_null());
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),268; @condition b"s.fBilerp\0".as_ptr(); s.fBilerp.assume_init());
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),269; @condition b"kN32_SkColorType == s.fPixmap.colorType()\0".as_ptr(); s.fPixmap.colorType()==crate::include::core::SkColorType::kN32_SkColorType);
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),270; @condition b"s.fAlphaScale <= 256\0".as_ptr(); s.fAlphaScale.assume_init()<=256);
        let (y0, y1, wy) = super::decode_packed_coordinates_and_weight(xy[0]);
        let row0 = s
            .fPixmap
            .addr()
            .add(y0 as usize * s.fPixmap.rowBytes())
            .cast::<u32>();
        let row1 = s
            .fPixmap
            .addr()
            .add(y1 as usize * s.fPixmap.rowBytes())
            .cast::<u32>();
        let all_y = lasx_xvilvl_d(
            lasx_xvreplgr2vr_h((16 - wy) as i32),
            lasx_xvreplgr2vr_h(wy as i32),
        );
        let mut input = 1;
        let mut output = 0;
        while count > 0 {
            count -= 1;
            let (x0, x1, wx) = super::decode_packed_coordinates_and_weight(xy[input]);
            input += 1;
            let zero = lasx_xvldi::<0>();
            let tl = lasx_xvinsgr2vr_w::<0>(zero, row0.add(x0 as usize).read() as i32);
            let tr = lasx_xvinsgr2vr_w::<0>(zero, row0.add(x1 as usize).read() as i32);
            let bl = lasx_xvinsgr2vr_w::<0>(zero, row1.add(x0 as usize).read() as i32);
            let br = lasx_xvinsgr2vr_w::<0>(zero, row1.add(x1 as usize).read() as i32);
            let l = lasx_xvilvl_b(lasx_xvldi::<0>(), lasx_xvilvl_w(tl, bl));
            let r = lasx_xvilvl_b(lasx_xvldi::<0>(), lasx_xvilvl_w(tr, br));
            let inner = lasx_xvadd_h(
                lasx_xvslli_h::<4>(l),
                lasx_xvmul_h(lasx_xvsub_h(r, l), lasx_xvreplgr2vr_h(wx as i32)),
            );
            let sum_x = lasx_xvmul_h(inner, all_y);
            let mut sum = lasx_xvadd_h(sum_x, lasx_xvbsrl_v::<8>(sum_x));
            sum = lasx_xvsrli_h::<8>(sum);
            if s.fAlphaScale.assume_init() < 256 {
                sum = lasx_xvmul_h(sum, lasx_xvreplgr2vr_h(s.fAlphaScale.assume_init() as i32));
                sum = lasx_xvsrli_h::<8>(sum);
            }
            colors[output] = lasx_xvpickve2gr_w::<0>(lasx_xvpickev_b(
                lasx_xvldi::<0>(),
                lasx_xvsat_hu::<8>(sum),
            )) as u32;
            output += 1;
        }
    }
}

#[cfg(target_arch = "loongarch64")]
pub mod lsx {
    use core::arch::loongarch64::*;
    // Source: src/opts/SkBitmapProcState_opts.h:350-430
    #[target_feature(enable = "lsx")]
    pub unsafe fn S32_alpha_D32_filter_DX(
        s: &crate::src::core::SkBitmapProcState::SkBitmapProcState,
        xy: &[u32],
        mut count: i32,
        colors: &mut [u32],
    ) {
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),352; @condition b"count > 0 && colors != nullptr\0".as_ptr(); count>0 && !colors.as_ptr().is_null());
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),353; @condition b"s.fBilerp\0".as_ptr(); s.fBilerp.assume_init());
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),354; @condition b"kN32_SkColorType == s.fPixmap.colorType()\0".as_ptr(); s.fPixmap.colorType()==crate::include::core::SkColorType::kN32_SkColorType);
        crate::SkASSERT!(@source b"src/opts/SkBitmapProcState_opts.h\0".as_ptr(),355; @condition b"s.fAlphaScale <= 256\0".as_ptr(); s.fAlphaScale.assume_init()<=256);
        let (y0, y1, wy) = super::decode_packed_coordinates_and_weight(xy[0]);
        let row0 = s
            .fPixmap
            .addr()
            .add(y0 as usize * s.fPixmap.rowBytes())
            .cast::<u32>();
        let row1 = s
            .fPixmap
            .addr()
            .add(y1 as usize * s.fPixmap.rowBytes())
            .cast::<u32>();
        let all_y = lsx_vilvl_d(
            lsx_vreplgr2vr_h((16 - wy) as i32),
            lsx_vreplgr2vr_h(wy as i32),
        );
        let mut input = 1;
        let mut output = 0;
        while count > 0 {
            count -= 1;
            let (x0, x1, wx) = super::decode_packed_coordinates_and_weight(xy[input]);
            input += 1;
            let zero = lsx_vldi::<0>();
            let tl = lsx_vinsgr2vr_w::<0>(zero, row0.add(x0 as usize).read() as i32);
            let tr = lsx_vinsgr2vr_w::<0>(zero, row0.add(x1 as usize).read() as i32);
            let bl = lsx_vinsgr2vr_w::<0>(zero, row1.add(x0 as usize).read() as i32);
            let br = lsx_vinsgr2vr_w::<0>(zero, row1.add(x1 as usize).read() as i32);
            let l = lsx_vilvl_b(lsx_vldi::<0>(), lsx_vilvl_w(tl, bl));
            let r = lsx_vilvl_b(lsx_vldi::<0>(), lsx_vilvl_w(tr, br));
            let inner = lsx_vadd_h(
                lsx_vslli_h::<4>(l),
                lsx_vmul_h(lsx_vsub_h(r, l), lsx_vreplgr2vr_h(wx as i32)),
            );
            let sum_x = lsx_vmul_h(inner, all_y);
            let mut sum = lsx_vadd_h(sum_x, lsx_vbsrl_v::<8>(sum_x));
            sum = lsx_vsrli_h::<8>(sum);
            if s.fAlphaScale.assume_init() < 256 {
                sum = lsx_vmul_h(sum, lsx_vreplgr2vr_h(s.fAlphaScale.assume_init() as i32));
                sum = lsx_vsrli_h::<8>(sum);
            }
            colors[output] =
                lsx_vpickve2gr_w::<0>(lsx_vpickev_b(lsx_vldi::<0>(), lsx_vsat_hu::<8>(sum))) as u32;
            output += 1;
        }
    }
}
