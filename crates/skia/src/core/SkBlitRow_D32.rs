//! Source: src/core/SkBlitRow_D32.cpp, Color32 opaque-span dispatch. See ../../LICENSE.
use crate::src::core::SkBlitter_ARGB32::blend_anti_h2_prepared;
use crate::src::opts::SkBlitMask_opts::blend_premultiplied;

pub fn fill_solid(dst: &mut [u8], source: [u8; 4]) {
    debug_assert_eq!(dst.len() % 4, 0);
    if dst.is_empty() {
        return;
    }
    // Constant-byte N32 colors (notably opaque white) can use one memset
    // rather than repeatedly reading the target to expand a pixel prefix.
    // This is the constant fill represented by SkOpts::memset32 / Color32;
    // other packed colors retain the exact existing pattern fill below.
    if source == [source[0]; 4] {
        dst.fill(source[0]);
        return;
    }
    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    {
        // SkMemset_opts.h::memsetT<uint32_t>: broadcast the packed color,
        // store complete four-pixel vectors, then finish the scalar tail.
        // Four independent vector stores are batched to keep this streaming
        // erase loop in registers under the unoptimized debug profile.
        // SAFETY: each store is counted in complete accessible N32 pixels;
        // the exclusive byte slice permits unaligned stores and has no alias.
        unsafe {
            core::arch::asm!(
                "dup v0.4s, {word:w}",
                "cmp {count}, #16",
                "b.lo 3f",
                "2:",
                "stp q0, q0, [{pointer}], #32",
                "stp q0, q0, [{pointer}], #32",
                "sub {count}, {count}, #16",
                "cmp {count}, #16",
                "b.hs 2b",
                "3:",
                "cmp {count}, #4",
                "b.lo 5f",
                "4:",
                "str q0, [{pointer}], #16",
                "sub {count}, {count}, #4",
                "cmp {count}, #4",
                "b.hs 4b",
                "5:",
                "cbz {count}, 7f",
                "6:",
                "str {word:w}, [{pointer}], #4",
                "subs {count}, {count}, #1",
                "b.ne 6b",
                "7:",
                pointer = inout(reg) dst.as_mut_ptr() => _,
                count = inout(reg) dst.len() / 4 => _,
                word = in(reg) u32::from_ne_bytes(source),
                out("v0") _,
                options(nostack),
            );
        }
        return;
    }
    #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
    {
        dst[..4].copy_from_slice(&source);
        let mut written = 4;
        while written < dst.len() {
            let count = written.min(dst.len() - written);
            dst.copy_within(..count, written);
            written += count;
        }
    }
}

pub fn blend_span(dst: &mut [u8], source: [u8; 4], coverage: u8, pair: bool) {
    if coverage == 0 || source[3] == 0 {
        return;
    }
    if coverage == 255 && source[3] == 255 {
        fill_solid(dst, source);
    } else {
        #[cfg(all(feature = "simd", target_arch = "aarch64", target_endian = "little"))]
        {
            blend_constant_coverage_span(dst, source, coverage, pair, false);
            return;
        }
        #[cfg(not(all(feature = "simd", target_arch = "aarch64", target_endian = "little")))]
        for pixel in dst.chunks_exact_mut(4) {
            if pair {
                blend_anti_h2_prepared(pixel, source, coverage);
            } else {
                blend_premultiplied(pixel, source, coverage);
            }
        }
    }
}

// A constant SkBlitMask A8 / blitAntiH2 span. Resolve the invariant source
// products and inverse once, but retain their two distinct truncation rules:
// A8 truncates source before adding destination; AntiH2 truncates their sum.
// The black AntiH2 specialization adds coverage only to destination alpha.
#[cfg(all(feature = "simd", target_arch = "aarch64", target_endian = "little"))]
fn blend_constant_coverage_span(
    dst: &mut [u8],
    src: [u8; 4],
    coverage: u8,
    pair: bool,
    zero_unused: bool,
) {
    debug_assert_eq!(dst.len() % 4, 0);
    if dst.is_empty() {
        return;
    }
    let (bias, inverse) = if pair && src == [0, 0, 0, 255] {
        (
            [0, 0, 0, u16::from(coverage) << 8],
            256 - u16::from(coverage),
        )
    } else if pair {
        let (scale, inverse) = if src[3] == 255 {
            let scale = u16::from(coverage) + u16::from(coverage >> 7);
            (scale, 256 - scale)
        } else {
            let scale = u16::from(coverage) + 1;
            let p = 65535 - u32::from(src[3]) * u32::from(scale);
            (scale, ((p + (p >> 8)) >> 8) as u16)
        };
        (
            [
                u16::from(src[0]) * scale,
                u16::from(src[1]) * scale,
                u16::from(src[2]) * scale,
                u16::from(src[3]) * scale,
            ],
            inverse,
        )
    } else {
        let scale = u16::from(coverage) + 1;
        (
            [
                ((u16::from(src[0]) * scale) >> 8) << 8,
                ((u16::from(src[1]) * scale) >> 8) << 8,
                ((u16::from(src[2]) * scale) >> 8) << 8,
                ((u16::from(src[3]) * scale) >> 8) << 8,
            ],
            256 - ((u16::from(src[3]) * scale) >> 8),
        )
    };
    let bias = [
        bias[0], bias[1], bias[2], bias[3], bias[0], bias[1], bias[2], bias[3],
    ];
    let store_mask = if zero_unused {
        0x00ff_ffffu32
    } else {
        u32::MAX
    };
    // SAFETY: the slice contains complete initialized N32 pixels. The vector
    // loop reads/writes four pixels only while count >= 4; the tail reads and
    // writes exactly one word. All source products fit u16. Saturating their
    // sum before >>8 is exactly min((source+dest)>>8,255), including channels
    // outside the premultiplied range; no rounding or pixel is omitted.
    unsafe {
        core::arch::asm!(
            "ldr q10, [{bias}]",
            "dup v11.8h, {inverse:w}",
            "dup v12.4s, {store_mask:w}",
            "cmp {count}, #4",
            "b.lo 3f",
            "2:",
            "ldr q0, [{pointer}]",
            "uxtl v1.8h, v0.8b",
            "uxtl2 v2.8h, v0.16b",
            "mul v1.8h, v1.8h, v11.8h",
            "mul v2.8h, v2.8h, v11.8h",
            "uqadd v1.8h, v1.8h, v10.8h",
            "uqadd v2.8h, v2.8h, v10.8h",
            "shrn v0.8b, v1.8h, #8",
            "shrn2 v0.16b, v2.8h, #8",
            "and v0.16b, v0.16b, v12.16b",
            "str q0, [{pointer}], #16",
            "sub {count}, {count}, #4",
            "cmp {count}, #4",
            "b.hs 2b",
            "3:",
            "cbz {count}, 5f",
            "4:",
            "ldr s0, [{pointer}]",
            "uxtl v1.8h, v0.8b",
            "mul v1.8h, v1.8h, v11.8h",
            "uqadd v1.8h, v1.8h, v10.8h",
            "shrn v0.8b, v1.8h, #8",
            "and v0.16b, v0.16b, v12.16b",
            "str s0, [{pointer}], #4",
            "subs {count}, {count}, #1",
            "b.ne 4b",
            "5:",
            pointer=inout(reg) dst.as_mut_ptr()=>_,count=inout(reg) dst.len()/4=>_,
            bias=in(reg) bias.as_ptr(),inverse=in(reg) u32::from(inverse),
            store_mask=in(reg) store_mask,
            out("v0") _,out("v1") _,out("v2") _,out("v10") _,out("v11") _,out("v12") _,
            options(nostack),
        );
    }
}

/// Source: SkBlitRow::Color32, constant premultiplied RGBA8888 color dispatch.
/// This is distinct from the A8 mask arithmetic used by `blend_span`.
pub fn color32(dst: &mut [u8], color: [u8; 4]) {
    debug_assert_eq!(dst.len() % 4, 0);
    debug_assert!(color[..3].iter().all(|&c| c <= color[3]));
    match color[3] {
        0 => (),
        255 => fill_solid(dst, color),
        _ => crate::src::opts::SkBlitRow_opts::blit_row_color32(dst, color),
    }
}

#[cfg(test)]
mod color32_tests {
    use super::color32;

    #[test]
    fn packed_erase_matches_scalar_with_unaligned_vectors_and_tails() {
        for color in [
            [255, 255, 255, 0],
            [13, 67, 211, 255],
            [0, 0, 0, 0],
            [255, 255, 255, 255],
        ] {
            for pixels in 0..=97usize {
                for offset in 0..8usize {
                    let mut actual = vec![0xA7; offset + pixels * 4 + 8];
                    super::fill_solid(&mut actual[offset..offset + pixels * 4], color);
                    let expected: Vec<u8> = (0..pixels).flat_map(|_| color).collect();
                    assert_eq!(&actual[offset..offset + pixels * 4], &expected);
                    assert!(actual[..offset]
                        .iter()
                        .chain(actual[offset + pixels * 4..].iter())
                        .all(|&v| v == 0xA7));
                }
            }
        }
    }

    // Independent byte-domain oracle for SkBlitRow_opts.h's skvx lambda:
    // c + trunc(d * SkAlpha255To256(255-a) / 256). No packed arithmetic,
    // SIMD intrinsics, or production helper is shared with this oracle.
    fn oracle(dst: [u8; 4], src: [u8; 4]) -> [u8; 4] {
        match src[3] {
            0 => dst,
            255 => src,
            alpha => core::array::from_fn(|i| {
                (u64::from(src[i]) + u64::from(dst[i]) * (256 - u64::from(alpha)) / 256) as u8
            }),
        }
    }

    #[test]
    fn color32_all_premultiplied_channel_values_and_destination_bytes() {
        // Every legal (alpha, source-channel) and every destination byte value.
        // Destination alpha cycles independently and includes translucent pixels.
        let mut row = vec![0; 256 * 4];
        for alpha in 0..=255u8 {
            for channel in 0..=alpha {
                let src = [channel, alpha - channel, channel / 2, alpha];
                for value in 0..=255u8 {
                    row[usize::from(value) * 4..][..4].copy_from_slice(&[
                        value,
                        255 - value,
                        value,
                        value.wrapping_mul(37),
                    ]);
                }
                color32(&mut row, src);
                for value in 0..=255u8 {
                    let expected = oracle([value, 255 - value, value, value.wrapping_mul(37)], src);
                    assert_eq!(
                        &row[usize::from(value) * 4..][..4],
                        &expected,
                        "a={alpha}, c={channel}, d={value}"
                    );
                }
            }
        }
    }

    #[test]
    fn color32_unaligned_lengths_and_simd_tails_preserve_guards() {
        for offset in 0..32 {
            for pixels in 0..34 {
                for src in [
                    [0, 0, 0, 0],
                    [255, 83, 11, 255],
                    [1, 0, 1, 1],
                    [63, 37, 24, 127],
                    [253, 2, 173, 254],
                ] {
                    let mut bytes = vec![0xa7; offset + pixels * 4 + 31];
                    let mut expected = bytes.clone();
                    for p in 0..pixels {
                        let i = offset + p * 4;
                        let dst = [p as u8, (p * 13) as u8, (p * 31) as u8, (p * 17) as u8];
                        bytes[i..i + 4].copy_from_slice(&dst);
                        expected[i..i + 4].copy_from_slice(&oracle(dst, src));
                    }
                    color32(&mut bytes[offset..offset + pixels * 4], src);
                    assert_eq!(
                        bytes, expected,
                        "offset={offset}, pixels={pixels}, src={src:?}"
                    );
                }
            }
        }
    }
}

/// Destination color-layout adapter; source shader math remains canonical RGBA.
pub(crate) fn blend_span_format(
    dst: &mut [u8],
    source: [u8; 4],
    coverage: u8,
    pair: bool,
    format: crate::raster::PixelFormat,
) {
    let source = format.swizzle(source);
    if format != crate::raster::PixelFormat::Bgrx8888 {
        return blend_span(dst, source, coverage, pair);
    }
    if coverage == 0 || source[3] == 0 {
        return;
    }
    if coverage == 255 && source[3] == 255 {
        let mut source = source;
        source[3] = 0;
        return fill_solid(dst, source);
    }
    #[cfg(all(feature = "simd", target_arch = "aarch64", target_endian = "little"))]
    {
        blend_constant_coverage_span(dst, source, coverage, pair, true);
        return;
    }
    #[cfg(not(all(feature = "simd", target_arch = "aarch64", target_endian = "little")))]
    for pixel in dst.chunks_exact_mut(4) {
        if pair {
            blend_anti_h2_prepared(pixel, source, coverage);
        } else {
            blend_premultiplied(pixel, source, coverage);
        }
        // Opaque-alpha target stores only RGB. Zero is the local host X-byte
        // encoding; no shader or blending stage observes it as transparent.
        pixel[3] = 0;
    }
}

pub(crate) fn color32_format(dst: &mut [u8], color: [u8; 4], format: crate::raster::PixelFormat) {
    let color = format.swizzle(color);
    if format != crate::raster::PixelFormat::Bgrx8888 {
        return color32(dst, color);
    }
    match color[3] {
        0 => (),
        255 => {
            let mut color = color;
            color[3] = 0;
            fill_solid(dst, color);
        }
        alpha => {
            #[repr(C, packed)]
            struct Pixel {
                bits: u32,
            }
            let source = u32::from_le_bytes(color);
            let inverse = 256 - u32::from(alpha);
            let ptr = dst.as_mut_ptr();
            for i in 0..dst.len() / 4 {
                // Slice bounds establish initialized exclusive 4-byte storage.
                unsafe {
                    let p = ptr.add(i * 4).cast::<Pixel>();
                    let d = u32::from_le((*p).bits);
                    let rb = (((d & 0x00ff00ff) * inverse) >> 8) & 0x00ff00ff;
                    let ga = (((d >> 8) & 0x00ff00ff) * inverse) & 0xff00ff00;
                    (*p).bits = ((source + (rb | ga)) & 0x00ffffff).to_le();
                }
            }
        }
    }
}

#[cfg(test)]
mod constant_coverage_span_tests {
    use super::*;
    use crate::raster::PixelFormat;

    fn scalar(dst: &mut [u8], source: [u8; 4], coverage: u8, pair: bool, format: PixelFormat) {
        if coverage == 0 || source[3] == 0 {
            return;
        }
        let source = format.swizzle(source);
        for pixel in dst.chunks_exact_mut(4) {
            if coverage == 255 && source[3] == 255 {
                pixel.copy_from_slice(&source);
            } else if pair {
                blend_anti_h2_prepared(pixel, source, coverage);
            } else {
                blend_premultiplied(pixel, source, coverage);
            }
            if format == PixelFormat::Bgrx8888 {
                pixel[3] = 0;
            }
        }
    }

    #[test]
    fn constant_coverage_spans_match_scalar_all_alpha_coverage_destination_bytes() {
        let mut seed = vec![0; 256 * 4];
        for (i, p) in seed.chunks_exact_mut(4).enumerate() {
            let b = i as u8;
            p.copy_from_slice(&[b, 255 - b, b.wrapping_mul(37), b]);
        }
        let mut actual = seed.clone();
        let mut expected = seed.clone();
        for alpha in 0..=255u8 {
            // Black specialization, legal nonuniform colors, and nonpremul
            // channels exercise saturation independently of its usual input.
            for source in [
                [0, 0, 0, alpha],
                [alpha / 7, alpha, alpha / 3, alpha],
                [alpha.wrapping_mul(37), 255 - alpha, alpha, alpha],
            ] {
                for coverage in 0..=255u8 {
                    for pair in [false, true] {
                        actual.copy_from_slice(&seed);
                        expected.copy_from_slice(&seed);
                        scalar(&mut expected, source, coverage, pair, PixelFormat::Rgba8888);
                        blend_span_format(
                            &mut actual,
                            source,
                            coverage,
                            pair,
                            PixelFormat::Rgba8888,
                        );
                        assert_eq!(
                            actual, expected,
                            "source={source:?} coverage={coverage} pair={pair}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn constant_coverage_spans_preserve_formats_unaligned_vector_tails_and_guards() {
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for count in 0..=35 {
                for offset in 0..8 {
                    for source in [
                        [0, 0, 0, 255],
                        [37, 89, 113, 179],
                        [131, 7, 250, 255],
                        [1, 1, 1, 1],
                        [255, 73, 231, 11],
                    ] {
                        for coverage in [0, 1, 91, 127, 128, 254, 255] {
                            for pair in [false, true] {
                                let mut actual: Vec<u8> = (0..offset + count * 4 + 8)
                                    .map(|i| (i * 101) as u8)
                                    .collect();
                                let mut expected = actual.clone();
                                scalar(
                                    &mut expected[offset..offset + count * 4],
                                    source,
                                    coverage,
                                    pair,
                                    format,
                                );
                                blend_span_format(
                                    &mut actual[offset..offset + count * 4],
                                    source,
                                    coverage,
                                    pair,
                                    format,
                                );
                                assert_eq!(actual,expected,"format={format:?} count={count} offset={offset} source={source:?} coverage={coverage} pair={pair}");
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn profile_google_constant_coverage_span() {
        for round in 0..5 {
            for fast in [false, true] {
                let mut bytes = vec![157; 1390 * 294 * 4];
                let started = std::time::Instant::now();
                if fast {
                    blend_span_format(
                        &mut bytes,
                        [37, 89, 113, 179],
                        213,
                        false,
                        PixelFormat::Bgrx8888,
                    );
                } else {
                    scalar(
                        &mut bytes,
                        [37, 89, 113, 179],
                        213,
                        false,
                        PixelFormat::Bgrx8888,
                    );
                }
                let ms = started.elapsed().as_secs_f64() * 1000.0;
                let hash = bytes
                    .iter()
                    .fold(0u64, |h, &v| h.wrapping_mul(31).wrapping_add(u64::from(v)));
                eprintln!(
                    "constant-coverage-google round={round} fast={fast} ms={ms:.3} hash={hash}"
                );
            }
        }
    }
}
