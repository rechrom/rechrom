//! Source: src/opts/SkBlitRow_opts.h::{blit_row_color32, blit_row_s32a_opaque}.
//! See ../../LICENSE.
//! Premultiplied RGBA8888 SrcOver subsets: constant four-pixel skvx color
//! arithmetic and variable-source ARM64 NEON rounded byte arithmetic.
//! RGBA byte slices adapt SkPMColor's platform channel order; alpha remains byte 3.

/// SkOpts::blit_row_s32a_opaque: variable premultiplied source, global alpha 255.
/// The source is RGBA; destination channel order is a local storage adapter.
pub(crate) fn blit_row_s32a_opaque(dst: &mut [u8], src: &[u8], format: crate::raster::PixelFormat) {
    assert_eq!(dst.len(), src.len());
    assert_eq!(dst.len() % 4, 0);
    #[cfg(all(target_arch = "aarch64", feature = "simd"))]
    unsafe {
        use crate::raster::PixelFormat;
        match format {
            PixelFormat::Rgba8888 => neon_s32a::<false, false>(dst, src),
            PixelFormat::Bgra8888 => neon_s32a::<true, false>(dst, src),
            PixelFormat::Bgrx8888 => neon_s32a::<true, true>(dst, src),
        }
        return;
    }
    #[cfg(not(all(target_arch = "aarch64", feature = "simd")))]
    scalar_s32a(dst, src, format);
}

fn scalar_s32a(dst: &mut [u8], src: &[u8], format: crate::raster::PixelFormat) {
    for (d, s) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
        let inverse = u32::from(255 - s[3]);
        for c in 0..4 {
            let channel = format.channel(c);
            let product = u32::from(d[channel]) * inverse;
            // SkMulDiv255Round_neon8: two rounded narrowing shifts.
            let scaled = (product + ((product + 128) >> 8) + 128) >> 8;
            d[channel] = s[c].saturating_add(scaled as u8);
        }
        if format == crate::raster::PixelFormat::Bgrx8888 {
            d[3] = 0;
        }
    }
}

#[cfg(all(target_arch = "aarch64", feature = "simd"))]
unsafe fn neon_s32a<const SWAP: bool, const ZERO_X: bool>(dst: &mut [u8], src: &[u8]) {
    let mut d = dst.as_mut_ptr();
    let mut s = src.as_ptr();
    let mut count = dst.len() / 4;
    // The exact SkPMSrcOver_neon8 arithmetic with contiguous packed loads and
    // stores. The inverse alpha is replicated by the same table technique as
    // SkPMSrcOver_neon2; urshr+raddhn retain both rounded narrowing shifts.
    // SAFETY: the caller checks equal slice sizes and complete N32 pixels.
    // Each 32-byte load/store runs only while count >= 8, advances both
    // pointers together, and leaves the final 0..7 pixels to the bounded tail.
    // Source/destination slices may be unaligned and the mutable target is
    // exclusive; fixed table indexes refer only to their loaded source vector.
    const TABLES: [[u8; 16]; 2] = [
        [2, 1, 0, 3, 6, 5, 4, 7, 10, 9, 8, 11, 14, 13, 12, 15],
        [3, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15],
    ];
    unsafe {
        core::arch::asm!(
            "ldp q28, q29, [{tables}]",
            "mov {mask:w}, #-1",
            ".if {zero_x}",
            "lsr {mask:w}, {mask:w}, #8",
            ".endif",
            "dup v30.4s, {mask:w}",
            "cmp {count}, #8",
            "b.lo 3f",
            "2:",
            "ldp q0, q1, [{src}], #32",
            // All 32 source bytes must be zero. Alpha alone is insufficient:
            // non-PM zero-alpha colors retain the original saturating blend.
            "orr v6.16b, v0.16b, v1.16b",
            "umaxv b6, v6.16b",
            "umov {mask:w}, v6.b[0]",
            "cbz {mask:w}, 4f",
            ".if {swap}",
            "tbl v0.16b, {{v0.16b}}, v28.16b",
            "tbl v1.16b, {{v1.16b}}, v28.16b",
            ".endif",
            "tbl v4.16b, {{v0.16b}}, v29.16b",
            "tbl v5.16b, {{v1.16b}}, v29.16b",
            "mvn v4.16b, v4.16b",
            "mvn v5.16b, v5.16b",
            // Zero inverse-alpha in both complete vectors proves all eight
            // pixels opaque; the destination coefficient is exactly zero.
            "orr v6.16b, v4.16b, v5.16b",
            "umaxv b6, v6.16b",
            "umov {mask:w}, v6.b[0]",
            "cbz {mask:w}, 5f",
            "ldp q2, q3, [{dst}]",
            "umull v18.8h, v2.8b, v4.8b",
            "umull2 v19.8h, v2.16b, v4.16b",
            "umull v20.8h, v3.8b, v5.8b",
            "umull2 v21.8h, v3.16b, v5.16b",
            "urshr v22.8h, v18.8h, #8",
            "urshr v23.8h, v19.8h, #8",
            "urshr v24.8h, v20.8h, #8",
            "urshr v25.8h, v21.8h, #8",
            "raddhn v18.8b, v18.8h, v22.8h",
            "raddhn2 v18.16b, v19.8h, v23.8h",
            "raddhn v20.8b, v20.8h, v24.8h",
            "raddhn2 v20.16b, v21.8h, v25.8h",
            "uqadd v2.16b, v0.16b, v18.16b",
            "uqadd v3.16b, v1.16b, v20.16b",
            "6:",
            ".if {zero_x}",
            "and v2.16b, v2.16b, v30.16b",
            "and v3.16b, v3.16b, v30.16b",
            ".endif",
            "stp q2, q3, [{dst}], #32",
            "7:",
            "sub {count}, {count}, #8",
            "cmp {count}, #8",
            "b.hs 2b",
            "b 3f",
            "4:",
            ".if {zero_x}",
            // BGRX still clears each raw X byte exactly as the original row.
            "ldp q2, q3, [{dst}]",
            "b 6b",
            ".else",
            "add {dst}, {dst}, #32",
            "b 7b",
            ".endif",
            "5:",
            "mov v2.16b, v0.16b",
            "mov v3.16b, v1.16b",
            "b 6b",
            "3:",
            src=inout(reg) s,dst=inout(reg) d,count=inout(reg) count,
            tables=in(reg) TABLES.as_ptr(),mask=out(reg) _,
            swap=const SWAP as u8,zero_x=const ZERO_X as u8,
            out("v0") _,out("v1") _,out("v2") _,out("v3") _,
            out("v4") _,out("v5") _,out("v6") _,out("v18") _,out("v19") _,
            out("v20") _,out("v21") _,out("v22") _,out("v23") _,
            out("v24") _,out("v25") _,out("v28") _,out("v29") _,out("v30") _,
            options(nostack),
        );
        let format = if ZERO_X {
            crate::raster::PixelFormat::Bgrx8888
        } else if SWAP {
            crate::raster::PixelFormat::Bgra8888
        } else {
            crate::raster::PixelFormat::Rgba8888
        };
        scalar_s32a(
            core::slice::from_raw_parts_mut(d, count * 4),
            core::slice::from_raw_parts(s, count * 4),
            format,
        );
    }
}

// Previous deinterleaved implementation, retained only for a same-input timing
// comparison of the load/store representation; it is never a production path.
#[cfg(all(test, target_arch = "aarch64", feature = "simd"))]
unsafe fn reference_ld4_s32a<const SWAP: bool, const ZERO_X: bool>(dst: &mut [u8], src: &[u8]) {
    let mut d = dst.as_mut_ptr();
    let mut s = src.as_ptr();
    let mut count = dst.len() / 4;
    // Official SkPMSrcOver_neon8: deinterleave eight pixels, invert alpha,
    // SkMulDiv255Round each channel, saturating add and interleaved store.
    // Complete vectors only; unaligned byte loads are valid. The scalar tail
    // handles the remaining 0..7 pixels without reading outside either slice.
    unsafe {
        core::arch::asm!(
            "cmp {count}, #8",
            "b.lo 3f",
            "2:",
            "ld4 {{v0.8b, v1.8b, v2.8b, v3.8b}}, [{src}], #32",
            "ld4 {{v4.8b, v5.8b, v6.8b, v7.8b}}, [{dst}]",
            "mvn v8.8b, v3.8b",
            ".if {swap}",
            "mov v11.8b, v0.8b",
            "mov v0.8b, v2.8b",
            "mov v2.8b, v11.8b",
            ".endif",
            "umull v9.8h, v8.8b, v4.8b",
            "urshr v10.8h, v9.8h, #8",
            "raddhn v9.8b, v9.8h, v10.8h",
            "uqadd v4.8b, v0.8b, v9.8b",
            "umull v9.8h, v8.8b, v5.8b",
            "urshr v10.8h, v9.8h, #8",
            "raddhn v9.8b, v9.8h, v10.8h",
            "uqadd v5.8b, v1.8b, v9.8b",
            "umull v9.8h, v8.8b, v6.8b",
            "urshr v10.8h, v9.8h, #8",
            "raddhn v9.8b, v9.8h, v10.8h",
            "uqadd v6.8b, v2.8b, v9.8b",
            "umull v9.8h, v8.8b, v7.8b",
            "urshr v10.8h, v9.8h, #8",
            "raddhn v9.8b, v9.8h, v10.8h",
            "uqadd v7.8b, v3.8b, v9.8b",
            ".if {zero_x}",
            "movi v7.8b, #0",
            ".endif",
            "st4 {{v4.8b, v5.8b, v6.8b, v7.8b}}, [{dst}], #32",
            "sub {count}, {count}, #8",
            "cmp {count}, #8",
            "b.hs 2b",
            "3:",
            src = inout(reg) s,
            dst = inout(reg) d,
            count = inout(reg) count,
            swap = const SWAP as u8,
            zero_x = const ZERO_X as u8,
            out("v0") _, out("v1") _, out("v2") _, out("v3") _,
            out("v4") _, out("v5") _, out("v6") _, out("v7") _,
            out("v8") _, out("v9") _, out("v10") _, out("v11") _,
            options(nostack),
        );
        let format = if ZERO_X {
            crate::raster::PixelFormat::Bgrx8888
        } else if SWAP {
            crate::raster::PixelFormat::Bgra8888
        } else {
            crate::raster::PixelFormat::Rgba8888
        };
        scalar_s32a(
            core::slice::from_raw_parts_mut(d, count * 4),
            core::slice::from_raw_parts(s, count * 4),
            format,
        );
    }
}

/// Blend a constant premultiplied color whose alpha is strictly between 0 and 255.
/// The caller handles the transparent and opaque dispatch, just like Color32.
pub fn blit_row_color32(dst: &mut [u8], color: [u8; 4]) {
    debug_assert_eq!(dst.len() % 4, 0);
    debug_assert!(color[3] > 0 && color[3] < 255);
    debug_assert!(color[..3].iter().all(|&c| c <= color[3]));
    let inverse = 256 - u32::from(color[3]);
    let packed = u32::from_le_bytes(color);
    #[cfg(all(target_arch = "aarch64", feature = "simd"))]
    unsafe {
        // Official Color32 executes its four-pixel loop only for count >= N.
        // Small AA runs go straight to the packed equivalent of its tail;
        // avoid preparing unused vector constants for one to three pixels.
        if dst.len() < 16 {
            scalar_color32(dst.as_mut_ptr(), dst.len() / 4, packed, inverse);
            return;
        }
        // The source's skvx Vec<4,u32> widens sixteen bytes, multiplies by
        // invA, shifts by 8, adds the constant color, then narrows back to U8.
        return neon_color32(dst, packed, inverse as u16);
    }
    #[cfg(not(all(target_arch = "aarch64", feature = "simd")))]
    unsafe {
        scalar_color32(dst.as_mut_ptr(), dst.len() / 4, packed, inverse);
    }
}

// Equivalent to the official kernel's widened byte arithmetic. For premultiplied
// input no byte addition can overflow: c <= a and d*(256-a)/256 <= 255-a.
unsafe fn scalar_color32(mut dst: *mut u8, count: usize, color: u32, inverse: u32) {
    const MASK: u32 = 0x00ff00ff;
    for _ in 0..count {
        let d = unsafe { core::ptr::read_unaligned(dst.cast::<u32>()) }.to_le();
        let rb = (((d & MASK) * inverse) >> 8) & MASK;
        let ga = (((d >> 8) & MASK) * inverse) & !MASK;
        unsafe { core::ptr::write_unaligned(dst.cast::<u32>(), (color + (rb | ga)).to_le()) };
        dst = unsafe { dst.add(4) };
    }
}

#[cfg(all(target_arch = "aarch64", feature = "simd"))]
unsafe fn neon_color32(dst: &mut [u8], color: u32, inverse: u16) {
    use core::arch::aarch64::*;
    unsafe {
        let c = vreinterpretq_u8_u32(vdupq_n_u32(color));
        let clo = vmovl_u8(vget_low_u8(c));
        let chi = vmovl_u8(vget_high_u8(c));
        let mut ptr = dst.as_mut_ptr();
        let mut count = dst.len() / 4;
        while count >= 4 {
            // Unaligned byte loads are valid; the Rust adapter supports every
            // byte-slice alignment rather than requiring SkPMColor alignment.
            let d = vld1q_u8(ptr);
            let lo = vaddq_u16(
                vshrq_n_u16::<8>(vmulq_n_u16(vmovl_u8(vget_low_u8(d)), inverse)),
                clo,
            );
            let hi = vaddq_u16(
                vshrq_n_u16::<8>(vmulq_n_u16(vmovl_u8(vget_high_u8(d)), inverse)),
                chi,
            );
            vst1q_u8(ptr, vcombine_u8(vmovn_u16(lo), vmovn_u16(hi)));
            ptr = ptr.add(16);
            count -= 4;
        }
        scalar_color32(ptr, count, color, u32::from(inverse));
    }
}

#[cfg(test)]
mod source_over_tests {
    use super::*;
    use crate::raster::PixelFormat;

    fn oracle(dst: &mut [u8], src: &[u8], format: PixelFormat) {
        for (d, s) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
            for c in 0..4 {
                let k = format.channel(c);
                let value = u32::from(s[c]) + (u32::from(d[k]) * u32::from(255 - s[3]) + 127) / 255;
                d[k] = value.min(255) as u8;
            }
            if format == PixelFormat::Bgrx8888 {
                d[3] = 0;
            }
        }
    }

    #[test]
    fn varying_alpha_rows_match_division_oracle_and_preserve_guards() {
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            // Every alpha/destination byte product, with varying source channels.
            let mut source = Vec::new();
            let mut target = Vec::new();
            for a in 0u16..256 {
                for d in 0u16..256 {
                    let a = a as u8;
                    source.extend_from_slice(&[a, a / 2, (d as u8).min(a), a]);
                    target.extend_from_slice(&format.encode([d as u8; 4]));
                }
            }
            let mut expected = target.clone();
            oracle(&mut expected, &source, format);
            blit_row_s32a_opaque(&mut target, &source, format);
            assert_eq!(target, expected, "{format:?}");
            let mut scalar = vec![0; target.len()];
            for (i, d) in scalar.chunks_exact_mut(4).enumerate() {
                d.copy_from_slice(&format.encode([(i % 256) as u8; 4]));
            }
            scalar_s32a(&mut scalar, &source, format);
            assert_eq!(scalar, expected);

            for offset in 0..16 {
                for count in 0..42 {
                    let size = count * 4;
                    let mut dst = vec![91; offset + size + 16];
                    let mut src = vec![77; offset + size + 16];
                    for i in 0..count {
                        let a = (i * 43) as u8;
                        src[offset + i * 4..offset + i * 4 + 4].copy_from_slice(&[
                            a / 2,
                            a / 3,
                            a / 4,
                            a,
                        ]);
                        dst[offset + i * 4..offset + i * 4 + 4]
                            .copy_from_slice(&format.encode([91; 4]));
                    }
                    let saved_src = src.clone();
                    let mut expected = dst.clone();
                    oracle(
                        &mut expected[offset..offset + size],
                        &src[offset..offset + size],
                        format,
                    );
                    blit_row_s32a_opaque(
                        &mut dst[offset..offset + size],
                        &src[offset..offset + size],
                        format,
                    );
                    assert_eq!(
                        dst, expected,
                        "format={format:?} offset={offset} count={count}"
                    );
                    assert_eq!(src, saved_src);
                }
            }
        }
    }
}

#[cfg(all(test, target_arch = "aarch64", feature = "simd"))]
mod packed_s32a_profile {
    use super::*;
    #[test]
    #[ignore = "full-frame component profile, run explicitly"]
    fn packed_and_deinterleaved_google_size_rows() {
        let width = 2560usize;
        let height = 1440usize;
        let source: Vec<u8> = (0..width * height)
            .flat_map(|i| {
                let a = i.wrapping_mul(37) as u8;
                [a / 3, a / 2, a, a]
            })
            .collect();
        let input: Vec<u8> = (0..width * height)
            .flat_map(|i| [(i * 7) as u8, (i * 13) as u8, (i * 19) as u8, 0xa7])
            .collect();
        let mut expected_hash = None;
        for round in 0..5 {
            // Alternate the two vector implementations; retain all cold values.
            let modes = if round % 2 == 0 { [0, 1, 2] } else { [1, 0, 2] };
            for mode in modes {
                let mut bytes = input.clone();
                let started = std::time::Instant::now();
                for (dst, src) in bytes
                    .chunks_exact_mut(width * 4)
                    .zip(source.chunks_exact(width * 4))
                {
                    unsafe {
                        match mode {
                            0 => reference_ld4_s32a::<true, true>(dst, src),
                            1 => neon_s32a::<true, true>(dst, src),
                            _ => scalar_s32a(dst, src, crate::raster::PixelFormat::Bgrx8888),
                        }
                    }
                }
                let elapsed = started.elapsed();
                let hash = bytes.iter().fold(14695981039346656037u64, |a, &b| {
                    (a ^ u64::from(b)).wrapping_mul(1099511628211)
                });
                if let Some(expected) = expected_hash {
                    assert_eq!(hash, expected);
                } else {
                    expected_hash = Some(hash);
                }
                let mode = ["deinterleaved", "packed", "scalar"][mode];
                eprintln!(
                    "packed-s32a-google round={round} mode={mode} ms={:.3} hash={hash:016x}",
                    elapsed.as_secs_f64() * 1000.0
                );
            }
        }
    }
}

// Whole-block and mixed-block source-over boundaries, including raw BGRX X.
#[cfg(all(test, target_arch = "aarch64", feature = "simd"))]
mod zerochunk_candidate_tests {
    use super::blit_row_s32a_opaque;
    use crate::raster::PixelFormat;

    // Independent scalar division form of native SkMulDiv255Round_neon8,
    // followed by unsigned saturating addition. Raw X is deliberately arbitrary.
    fn original_row_oracle(dst: &mut [u8], src: &[u8], format: PixelFormat) {
        let channel = match format {
            PixelFormat::Rgba8888 => [0, 1, 2, 3],
            PixelFormat::Bgra8888 | PixelFormat::Bgrx8888 => [2, 1, 0, 3],
        };
        for (d, s) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
            for c in 0..4 {
                let scaled = (u32::from(d[channel[c]]) * u32::from(255 - s[3]) + 127) / 255;
                d[channel[c]] = (u32::from(s[c]) + scaled).min(255) as u8;
            }
            if format == PixelFormat::Bgrx8888 {
                d[3] = 0;
            }
        }
    }
    fn check(src: Vec<u8>, offset: usize, format: PixelFormat, salt: usize) {
        let size = src.len();
        let mut source = vec![0x6d; offset + size + 17];
        source[offset..offset + size].copy_from_slice(&src);
        let saved_source = source.clone();
        let mut destination: Vec<u8> = (0..offset + size + 17)
            .map(|i| (i.wrapping_mul(73).wrapping_add(salt)) as u8)
            .collect();
        let mut expected = destination.clone();
        original_row_oracle(&mut expected[offset..offset + size], &src, format);
        blit_row_s32a_opaque(
            &mut destination[offset..offset + size],
            &source[offset..offset + size],
            format,
        );
        assert_eq!(
            destination,
            expected,
            "format={format:?} offset={offset} count={} salt={salt}",
            size / 4
        );
        assert_eq!(source, saved_source);
    }

    #[test]
    fn zero_and_mixed_chunks_keep_native_arithmetic_all_formats_tails_and_guards() {
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for offset in 0..32 {
                for count in 0..=73usize {
                    for mode in 0..6 {
                        let mut source = vec![0; count * 4];
                        for i in 0..count {
                            let a = ((i * 43) % 254 + 1) as u8;
                            let live = match mode {
                                0 => false,
                                1 => i / 8 % 2 == 0,
                                2 => i / 8 % 2 != 0,
                                _ => true,
                            };
                            if live {
                                let value = match mode {
                                    // Deliberately invalid PM: alpha0, nonzero RGB.
                                    // It must always go through the original nonzero arithmetic.
                                    3 => [1 + (i % 255) as u8, 255, (i * 13) as u8, 0],
                                    5 => [(i * 7) as u8, (i * 13) as u8, (i * 19) as u8, 255],
                                    _ => [a / 3, a / 2, a, a],
                                };
                                source[i * 4..i * 4 + 4].copy_from_slice(&value);
                            }
                        }
                        check(source, offset, format, mode * 31 + count);
                    }
                }
                // Both q-register halves and every RGBA byte, between zero chunks.
                for byte in 0..32 {
                    for value in [1, 127, 255] {
                        let mut source = vec![0; 24 * 4];
                        source[32 + byte] = value;
                        check(source, offset, format, byte + usize::from(value));
                    }
                }
            }
            // The zero-source arithmetic identity holds for all256 raw destination
            // bytes, including each possible X. Keep all surrounding guards too.
            for destination_byte in 0..=255usize {
                let source = vec![0; 24 * 4];
                // Offset0: stride73 visits every value in raw memory over the salts.
                check(source, 0, format, destination_byte);
            }
        }
    }

    #[test]
    fn double_rounded_zero_source_product_is_identity_for_every_destination_byte() {
        for destination in 0..=255u32 {
            let product = destination * 255;
            let original_neon = (product + ((product + 128) >> 8) + 128) >> 8;
            assert_eq!(original_neon, destination);
            assert_eq!((product + 127) / 255, destination);
        }
    }
}

#[cfg(all(test, target_arch = "aarch64", feature = "simd"))]
mod zerochunk_component_profile {
    use super::*;
    fn hash(bytes: &[u8]) -> u64 {
        bytes.iter().fold(0xcbf29ce484222325u64, |h, &b| {
            (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
        })
    }
    #[test]
    #[ignore = "complete row component diagnostic; invoke explicitly, retain every sample"]
    fn fullzero_mixed_dense_complete_rows() {
        let (width, height) = (2560usize, 1440usize);
        let rounds = std::env::var("S32A_PROFILE_ROUNDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5usize);
        let target: Vec<u8> = (0..width * height)
            .flat_map(|i| {
                [
                    (i * 7) as u8,
                    (i * 13) as u8,
                    (i * 19) as u8,
                    (i * 73 + 167) as u8,
                ]
            })
            .collect();
        let mut sources = Vec::new();
        let mut expected_hashes = Vec::new();
        let mut input_hashes = Vec::new();
        for mode in 0..3 {
            let source: Vec<u8> = (0..width * height)
                .flat_map(|i| {
                    let a = (i.wrapping_mul(37) % 255 + 1) as u8;
                    if mode == 0 || (mode == 1 && i / 8 % 2 == 0) {
                        [0; 4]
                    } else {
                        [a / 3, a / 2, a, a]
                    }
                })
                .collect();
            let mut expected = target.clone();
            scalar_s32a(&mut expected, &source, crate::raster::PixelFormat::Bgrx8888);
            expected_hashes.push(hash(&expected));
            input_hashes.push((hash(&source), hash(&target)));
            sources.push(source);
        }
        for round in 0..rounds {
            let order = if round % 2 == 0 { [0, 1, 2] } else { [2, 1, 0] };
            for mode in order {
                let mut destination = target.clone();
                let source = &sources[mode];
                let start = std::time::Instant::now();
                for (dst, src) in destination
                    .chunks_exact_mut(width * 4)
                    .zip(source.chunks_exact(width * 4))
                {
                    blit_row_s32a_opaque(dst, src, crate::raster::PixelFormat::Bgrx8888);
                }
                let ms = start.elapsed().as_secs_f64() * 1000.0;
                let output_hash = hash(&destination);
                assert_eq!(output_hash, expected_hashes[mode]);
                let name = ["fullzero", "mixed", "dense"][mode];
                eprintln!("s32a-complete-component distribution={name} round={round} first_per_distribution={} rows={height} pixels={} ms={ms:.6} src_hash={:016x} dst_input_hash={:016x} output_hash={output_hash:016x}", round == 0, width * height, input_hashes[mode].0, input_hashes[mode].1);
            }
        }
    }
}
