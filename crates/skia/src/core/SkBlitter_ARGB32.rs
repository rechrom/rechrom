// Source: src/core/SkBlitter_ARGB32.cpp, blitAntiH2 rounding and opaque specialization.
// See ../../LICENSE.
pub fn blend_anti_h2_prepared(dst: &mut [u8], src: [u8; 4], coverage: u8) {
    let alpha = src[3];
    if src == [0, 0, 0, 255] {
        for c in 0..4 {
            dst[c] = ((u32::from(dst[c]) * (256 - u32::from(coverage))) >> 8) as u8;
        }
        dst[3] = dst[3].saturating_add(coverage);
        return;
    }
    let (scale, inverse) = if alpha == 255 {
        let scale = u32::from(coverage) + u32::from(coverage >> 7);
        (scale, 256 - scale)
    } else {
        let scale = u32::from(coverage) + 1;
        let product = 65535 - u32::from(alpha) * scale;
        (scale, (product + (product >> 8)) >> 8)
    };
    for c in 0..4 {
        dst[c] = ((u32::from(src[c]) * scale + u32::from(dst[c]) * inverse) >> 8).min(255) as u8;
    }
}

/// Translate one constant-coverage run from SkARGB32_Blitter::blitAntiH,
/// SkARGB32_Opaque_Blitter::blitAntiH, or SkARGB32_Black_Blitter::blitAntiH.
/// The run-array traversal belongs to the caller; `src` is premultiplied RGBA.
/// This deliberately differs from blitAntiH2 at low alpha / coverage boundaries.
pub fn blend_anti_h_prepared(dst: &mut [u8], src: [u8; 4], coverage: u8) {
    debug_assert_eq!(dst.len() % 4, 0);
    debug_assert!(src[..3].iter().all(|&c| c <= src[3]));
    if coverage == 0 || src[3] == 0 {
        return;
    }
    // Opaque full coverage is SkOpts::memset32. Black's partial coverage
    // source is just aa << A32_SHIFT, exactly the same scaled RGBA color.
    let scaled = if coverage == 255 {
        src
    } else {
        let scale = u32::from(coverage) + 1; // SkAlpha255To256(aa)
        [
            ((u32::from(src[0]) * scale) >> 8) as u8,
            ((u32::from(src[1]) * scale) >> 8) as u8,
            ((u32::from(src[2]) * scale) >> 8) as u8,
            ((u32::from(src[3]) * scale) >> 8) as u8,
        ]
    };
    crate::src::core::SkBlitRow_D32::color32(dst, scaled);
}

#[cfg(test)]
mod anti_h_tests {
    use super::{blend_anti_h2_prepared, blend_anti_h_prepared};

    // Official SkColorPriv.h::SkAlphaMulQ's packed two-lane arithmetic forms an
    // independent oracle for coverage scaling and destination multiplication.
    fn alpha_mul_q(c: u32, scale: u32) -> u32 {
        const MASK: u32 = 0x00ff00ff;
        let rb = ((c & MASK) * scale) >> 8;
        let ag = ((c >> 8) & MASK) * scale;
        (rb & MASK) | (ag & !MASK)
    }

    fn oracle(dst: [u8; 4], src: [u8; 4], aa: u8) -> [u8; 4] {
        if aa == 0 || src[3] == 0 {
            return dst;
        }
        let sc = alpha_mul_q(u32::from_le_bytes(src), u32::from(aa) + 1);
        let alpha = sc >> 24;
        if alpha == 0 {
            dst
        } else if alpha == 255 {
            sc.to_le_bytes()
        } else {
            (sc + alpha_mul_q(u32::from_le_bytes(dst), 256 - alpha)).to_le_bytes()
        }
    }

    #[test]
    fn anti_h_all_coverage_source_alpha_and_destination_alpha() {
        let mut row = vec![0; 256 * 4];
        for alpha in 0..=255u8 {
            // Include the official black specialization, nonuniform opaque color,
            // and two representative translucent premultiplied color patterns.
            for src in [
                [0, 0, 0, alpha],
                [alpha, alpha / 3, alpha / 2, alpha],
                [alpha / 7, alpha, 0, alpha],
            ] {
                for aa in 0..=255u8 {
                    for da in 0..=255u8 {
                        row[usize::from(da) * 4..][..4].copy_from_slice(&[da / 2, da / 3, da, da]);
                    }
                    blend_anti_h_prepared(&mut row, src, aa);
                    for da in 0..=255u8 {
                        let expected = oracle([da / 2, da / 3, da, da], src, aa);
                        assert_eq!(
                            &row[usize::from(da) * 4..][..4],
                            &expected,
                            "src={src:?}, aa={aa}, da={da}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn anti_h_rounding_is_distinct_from_anti_h2() {
        // Black blitAntiH uses 256-aa, blitAntiH2 also does; ordinary opaque
        // blitAntiH2 uses aa+(aa>>7), whereas blitAntiH uses aa+1.
        let src = [255, 17, 3, 255];
        let initial = [255, 255, 255, 255];
        let mut run = initial;
        let mut pair = initial;
        blend_anti_h_prepared(&mut run, src, 1);
        blend_anti_h2_prepared(&mut pair, src, 1);
        assert_eq!(run, [255, 254, 254, 255]);
        assert_eq!(pair, [255, 254, 254, 255]);
        let mut run = [0, 0, 0, 0];
        let mut pair = run;
        blend_anti_h_prepared(&mut run, src, 1);
        blend_anti_h2_prepared(&mut pair, src, 1);
        assert_eq!(run, [1, 0, 0, 1]);
        assert_eq!(pair, [0, 0, 0, 0]);
    }

    #[test]
    fn anti_h_unaligned_short_and_empty_runs_preserve_guards() {
        for offset in 0..16 {
            for pixels in 0..10 {
                for aa in [0, 1, 7, 127, 128, 254, 255] {
                    for src in [
                        [0, 0, 0, 255],
                        [255, 53, 113, 255],
                        [15, 37, 97, 127],
                        [1, 0, 1, 1],
                    ] {
                        let mut bytes = vec![0x9d; offset + pixels * 4 + 17];
                        let mut expected = bytes.clone();
                        for p in 0..pixels {
                            let i = offset + p * 4;
                            let dst = [17, 71, 119, (p * 29) as u8];
                            bytes[i..i + 4].copy_from_slice(&dst);
                            expected[i..i + 4].copy_from_slice(&oracle(dst, src, aa));
                        }
                        blend_anti_h_prepared(&mut bytes[offset..offset + pixels * 4], src, aa);
                        assert_eq!(
                            bytes, expected,
                            "offset={offset}, pixels={pixels}, aa={aa}, src={src:?}"
                        );
                    }
                }
            }
        }
    }
}

/// Packed storage adapter for the canonical SrcOver constant-color blitter.
pub(crate) fn blend_anti_h_format(
    dst: &mut [u8],
    src: [u8; 4],
    coverage: u8,
    format: crate::raster::PixelFormat,
) {
    if format != crate::raster::PixelFormat::Bgrx8888 {
        return blend_anti_h_prepared(dst, format.swizzle(src), coverage);
    }
    if coverage == 0 || src[3] == 0 {
        return;
    }
    let scale = if coverage == 255 {
        256
    } else {
        u32::from(coverage) + 1
    };
    let scaled = [
        ((u32::from(src[0]) * scale) >> 8) as u8,
        ((u32::from(src[1]) * scale) >> 8) as u8,
        ((u32::from(src[2]) * scale) >> 8) as u8,
        ((u32::from(src[3]) * scale) >> 8) as u8,
    ];
    crate::src::core::SkBlitRow_D32::color32_format(dst, scaled, format);
}
