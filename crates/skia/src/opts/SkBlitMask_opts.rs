//! Source: src/opts/SkBlitMask_opts.h, A8-to-N32 alpha+1 arithmetic. See ../../LICENSE.
//! The row kernels below translate the ARM NEON general, opaque, and black
//! subsets. The premultiplied RGBA entry point and slice-bounded single-row
//! driver adapt the upstream SkColor/SkPMColor, row-stride, and height interface.
//! PreparedA8Blitter retains color and kernel selection across clipped glyph
//! rows. Each direct NEON row broadcasts once before its bounded pixel loop.
pub fn blend_premultiplied(dst: &mut [u8], src: [u8; 4], coverage: u8) {
    if coverage == 0 {
        return;
    }
    let alpha = src[3];
    let mask = u32::from(coverage) + 1;
    let masked_alpha = (u32::from(alpha) * mask) >> 8;
    let inverse = 256 - masked_alpha;
    for i in 0..4 {
        dst[i] = (((u32::from(src[i]) * mask) >> 8) + ((u32::from(dst[i]) * inverse) >> 8)).min(255)
            as u8;
    }
}

/// Blend one A8 mask row over premultiplied RGBA8888 destination pixels.
/// Only `mask.len()` destination pixels are touched; trailing bytes are retained.
pub fn blend_a8_row(dst: &mut [u8], src: [u8; 4], mask: &[u8]) {
    PreparedA8Blitter::new(src).blend_row(dst, mask);
}

/// Constant-color A8 blitter setup, reusable across rows of the same glyph run.
/// Kernel selection and source color are invariant across those rows, as in
/// the upstream blitter's dispatch outside its height loop.
pub struct PreparedA8Blitter {
    src: [u8; 4],
    zero_unused: bool,
    kernel: unsafe fn(&PreparedA8Blitter, &mut [u8], &[u8]),
}

impl PreparedA8Blitter {
    pub fn new(src: [u8; 4]) -> Self {
        Self::new_with_format(src, crate::PixelFormat::Rgba8888)
    }
    pub(crate) fn with_format(&self, format: crate::PixelFormat) -> Self {
        Self::new_with_format(self.src, format)
    }
    pub(crate) fn new_with_format(src: [u8; 4], format: crate::PixelFormat) -> Self {
        let src = format.swizzle(src);
        let zero_unused = format == crate::PixelFormat::Bgrx8888;
        debug_assert!(src[..3].iter().all(|&c| c <= src[3]));
        #[cfg(all(target_arch = "aarch64", feature = "simd"))]
        {
            Self {
                src,
                zero_unused,
                kernel: if src[3] == 0 {
                    transparent_a8_row
                } else if zero_unused {
                    if src == [0, 0, 0, 255] {
                        neon_a8_direct_row::<false, true, true>
                    } else if src[3] == 255 {
                        neon_a8_direct_row::<false, false, true>
                    } else {
                        neon_a8_direct_row::<true, false, true>
                    }
                } else if src == [0, 0, 0, 255] {
                    neon_a8_direct_row::<false, true, false>
                } else if src[3] == 255 {
                    neon_a8_direct_row::<false, false, false>
                } else {
                    neon_a8_direct_row::<true, false, false>
                },
            }
        }
        #[cfg(not(all(target_arch = "aarch64", feature = "simd")))]
        Self {
            src,
            zero_unused,
            kernel: if src[3] == 0 {
                transparent_a8_row
            } else {
                scalar_a8_row
            },
        }
    }

    /// Blend `mask.len()` pixels, retaining all subsequent destination bytes.
    pub fn blend_row(&self, dst: &mut [u8], mask: &[u8]) {
        assert!(mask.len() <= dst.len() / 4, "A8 row exceeds destination");
        // Every selected kernel consumes only this checked row. Source color
        // and kernel selection were retained by new().
        unsafe { (self.kernel)(self, dst, mask) };
    }
}

unsafe fn transparent_a8_row(_: &PreparedA8Blitter, _: &mut [u8], _: &[u8]) {}

#[cfg(not(all(target_arch = "aarch64", feature = "simd")))]
unsafe fn scalar_a8_row(context: &PreparedA8Blitter, dst: &mut [u8], mask: &[u8]) {
    scalar_a8_tail_format(dst, context.src, mask, context.zero_unused);
}

#[cfg(not(all(target_arch = "aarch64", feature = "simd")))]
fn scalar_a8_tail_format(dst: &mut [u8], src: [u8; 4], mask: &[u8], zero_unused: bool) {
    // This is the NEON source's scalar tail arithmetic, including alpha+1
    // and truncation by 256. It is deliberately not the non-NEON Sk4px
    // approxMulDiv255 implementation or AntiH's different coverage arithmetic.
    for (pixel, &coverage) in dst.chunks_exact_mut(4).zip(mask) {
        blend_premultiplied(pixel, src, coverage);
        if zero_unused && coverage != 0 {
            pixel[3] = 0;
        }
    }
}

// SkBlitMask_opts.h's NEON row expressed as one bounded Rust inline-assembly
// loop. The arithmetic is unchanged; direct instructions avoid Debug's calls
// through stdarch wrappers for every vector operation. Source setup and the
// translucent/opaque/black dispatch remain outside the eight-pixel loop.
#[cfg(all(target_arch = "aarch64", feature = "simd"))]
unsafe fn neon_a8_direct_row<
    const TRANSLUCENT: bool,
    const BLACK: bool,
    const ZERO_UNUSED: bool,
>(
    context: &PreparedA8Blitter,
    dst: &mut [u8],
    mask: &[u8],
) {
    let blocks = mask.len() / 8;
    if blocks != 0 {
        macro_rules! run {
            ($inverse:literal, $colors:literal, $alpha:literal) => {
                core::arch::asm!(
                    "movi v20.8h, #1",
                    "movi v21.8h, #1, lsl #8",
                    "dup v22.8h, {red:w}",
                    "dup v23.8h, {green:w}",
                    "dup v24.8h, {blue:w}",
                    "dup v25.8h, {alpha:w}",
                    "2:",
                    "ld1 {{v0.8b}}, [{mask}], #8",
                    "uaddw v1.8h, v20.8h, v0.8b",
                    $inverse,
                    "ld4 {{v4.8b, v5.8b, v6.8b, v7.8b}}, [{dst}]",
                    $colors,
                    $alpha,
                    "st4 {{v4.8b, v5.8b, v6.8b, v7.8b}}, [{dst}], #32",
                    "subs {blocks}, {blocks}, #1",
                    "b.ne 2b",
                    mask = inout(reg) mask.as_ptr() => _,
                    dst = inout(reg) dst.as_mut_ptr() => _,
                    blocks = inout(reg) blocks => _,
                    red = in(reg) u32::from(context.src[0]),
                    green = in(reg) u32::from(context.src[1]),
                    blue = in(reg) u32::from(context.src[2]),
                    alpha = in(reg) u32::from(context.src[3]),
                    out("v0") _, out("v1") _, out("v2") _, out("v3") _,
                    out("v4") _, out("v5") _, out("v6") _, out("v7") _,
                    out("v16") _, out("v20") _, out("v21") _,
                    out("v22") _, out("v23") _, out("v24") _, out("v25") _,
                    options(nostack),
                );
            };
        }
        // These fragments exactly match SkAlphaMulQ_neon8's widening,
        // multiplication, truncation, and separately truncated source add.
        macro_rules! opaque {
            ($colors:literal, $alpha:literal) => {
                run!("usubw v2.8h, v21.8h, v0.8b", $colors, $alpha)
            };
        }
        macro_rules! general {
            ($colors:literal, $alpha:literal) => {
                run!(
                    "mul v2.8h, v25.8h, v1.8h\n\
                     shrn v2.8b, v2.8h, #8\n\
                     usubw v2.8h, v21.8h, v2.8b",
                    $colors,
                    $alpha
                )
            };
        }
        macro_rules! rgb {
            ($kernel:ident, $alpha:literal) => {
                $kernel!(
                    "ushll v3.8h, v4.8b, #0\n\
                     mul v3.8h, v3.8h, v2.8h\n\
                     shrn v4.8b, v3.8h, #8\n\
                     mul v16.8h, v22.8h, v1.8h\n\
                     shrn v16.8b, v16.8h, #8\n\
                     add v4.8b, v4.8b, v16.8b\n\
                     ushll v3.8h, v5.8b, #0\n\
                     mul v3.8h, v3.8h, v2.8h\n\
                     shrn v5.8b, v3.8h, #8\n\
                     mul v16.8h, v23.8h, v1.8h\n\
                     shrn v16.8b, v16.8h, #8\n\
                     add v5.8b, v5.8b, v16.8b\n\
                     ushll v3.8h, v6.8b, #0\n\
                     mul v3.8h, v3.8h, v2.8h\n\
                     shrn v6.8b, v3.8h, #8\n\
                     mul v16.8h, v24.8h, v1.8h\n\
                     shrn v16.8b, v16.8h, #8\n\
                     add v6.8b, v6.8b, v16.8b",
                    $alpha
                )
            };
        }
        macro_rules! black_rgb {
            ($alpha:literal) => {
                opaque!(
                    "ushll v3.8h, v4.8b, #0\n\
                     mul v3.8h, v3.8h, v2.8h\n\
                     shrn v4.8b, v3.8h, #8\n\
                     ushll v3.8h, v5.8b, #0\n\
                     mul v3.8h, v3.8h, v2.8h\n\
                     shrn v5.8b, v3.8h, #8\n\
                     ushll v3.8h, v6.8b, #0\n\
                     mul v3.8h, v3.8h, v2.8h\n\
                     shrn v6.8b, v3.8h, #8",
                    $alpha
                )
            };
        }
        if ZERO_UNUSED {
            // At zero coverage preserve every destination byte, including X.
            // At positive coverage BGRX's unused byte is zeroed as before.
            if BLACK {
                black_rgb!("cmeq v0.8b, v0.8b, #0\nand v7.8b, v7.8b, v0.8b");
            } else if TRANSLUCENT {
                rgb!(general, "cmeq v0.8b, v0.8b, #0\nand v7.8b, v7.8b, v0.8b");
            } else {
                rgb!(opaque, "cmeq v0.8b, v0.8b, #0\nand v7.8b, v7.8b, v0.8b");
            }
        } else if BLACK {
            black_rgb!(
                "ushll v3.8h, v7.8b, #0\n\
                 mul v3.8h, v3.8h, v2.8h\n\
                 shrn v7.8b, v3.8h, #8\n\
                 add v7.8b, v7.8b, v0.8b"
            );
        } else {
            macro_rules! alpha {
                ($kernel:ident) => {
                    rgb!(
                        $kernel,
                        "ushll v3.8h, v7.8b, #0\n\
                         mul v3.8h, v3.8h, v2.8h\n\
                         shrn v7.8b, v3.8h, #8\n\
                         mul v16.8h, v25.8h, v1.8h\n\
                         shrn v16.8b, v16.8h, #8\n\
                         add v7.8b, v7.8b, v16.8b"
                    )
                };
            }
            if TRANSLUCENT {
                alpha!(general);
            } else {
                alpha!(opaque);
            }
        }
    }
    // Fewer than eight pixels stay in the established packed scalar driver.
    packed_a8_row(context, &mut dst[blocks * 32..], &mask[blocks * 8..]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    fn check_row(src: [u8; 4], mask: &[u8], offset: usize) {
        let mut actual: Vec<u8> = (0..offset + mask.len() * 4 + 11)
            .map(|i| (i.wrapping_mul(73).wrapping_add(191) & 255) as u8)
            .collect();
        let mut expected = actual.clone();
        for (i, &coverage) in mask.iter().enumerate() {
            blend_premultiplied(&mut expected[offset + i * 4..][..4], src, coverage);
        }
        blend_a8_row(&mut actual[offset..], src, mask);
        assert_eq!(
            actual,
            expected,
            "src={src:?}, offset={offset}, len={}",
            mask.len()
        );
    }

    #[test]
    fn a8_row_matches_scalar_all_alpha_and_coverage() {
        let mask: Vec<u8> = (0..=255).collect();
        for alpha in 0..=255u8 {
            for src in [
                [0, 0, 0, alpha],
                [alpha, alpha, alpha, alpha],
                [alpha / 2, alpha / 3, alpha.saturating_sub(1), alpha],
            ] {
                for offset in 0..8 {
                    check_row(src, &mask, offset);
                }
            }
        }
    }

    #[test]
    fn a8_row_preserves_guards_tails_and_zero_coverage() {
        for len in 0..=40 {
            let mask: Vec<u8> = (0..len)
                .map(|i| [0, 255, 1, 127, 254, 7, 128, 0][i % 8])
                .collect();
            for src in [
                [0, 0, 0, 255],
                [17, 123, 255, 255],
                [37, 0, 81, 129],
                [0; 4],
            ] {
                for offset in 0..16 {
                    check_row(src, &mask, offset);
                    check_row(src, &vec![0; len], offset);
                }
            }
        }
    }

    #[test]
    fn a8_row_prepared_setup_reuses_across_clipped_rows() {
        for alpha in 0..=255u8 {
            for src in [
                [0, 0, 0, alpha],
                [alpha, alpha, alpha, alpha],
                [alpha / 2, alpha / 3, alpha.saturating_sub(1), alpha],
            ] {
                let prepared = PreparedA8Blitter::new(src);
                for len in 0..=40 {
                    let offset = len % 16;
                    let mask: Vec<u8> = (0..len)
                        .map(|i| [0, 255, 1, 127, 254, alpha, 128, 0][i % 8])
                        .collect();
                    let mut actual: Vec<u8> = (0..offset + len * 4 + 11)
                        .map(|i| (i.wrapping_mul(73).wrapping_add(len) & 255) as u8)
                        .collect();
                    let mut wrapped = actual.clone();
                    let mut expected = actual.clone();
                    for (i, &coverage) in mask.iter().enumerate() {
                        blend_premultiplied(&mut expected[offset + i * 4..][..4], src, coverage);
                    }
                    prepared.blend_row(&mut actual[offset..], &mask);
                    blend_a8_row(&mut wrapped[offset..], src, &mask);
                    assert_eq!(actual, expected, "src={src:?}, row_len={len}");
                    assert_eq!(actual, wrapped, "prepared/wrapper row_len={len}");
                }
            }
        }
    }

    #[test]
    fn a8_row_all_formats_match_alpha_coverage_and_unaligned_tails() {
        for format in [
            crate::PixelFormat::Rgba8888,
            crate::PixelFormat::Bgra8888,
            crate::PixelFormat::Bgrx8888,
        ] {
            for alpha in 0..=255_u8 {
                for src in [
                    [0, 0, 0, alpha],
                    [alpha, alpha, alpha, alpha],
                    [alpha / 2, alpha / 3, alpha.saturating_sub(1), alpha],
                ] {
                    let context = PreparedA8Blitter::new_with_format(src, format);
                    for offset in 0..8 {
                        for length in [0, 1, 7, 8, 9, 15, 16, 17, 31, 32, 33, 256] {
                            let mask: Vec<_> = (0..length).map(|i| i as u8).collect();
                            let mut actual: Vec<_> = (0..offset + length * 4 + 11)
                                .map(|i| (i * 73 + 191) as u8)
                                .collect();
                            let mut expected = actual.clone();
                            for (i, &coverage) in mask.iter().enumerate() {
                                let p = &mut expected[offset + i * 4..][..4];
                                blend_premultiplied(p, context.src, coverage);
                                if context.zero_unused && src[3] != 0 && coverage != 0 {
                                    p[3] = 0;
                                }
                            }
                            context.blend_row(&mut actual[offset..], &mask);
                            assert_eq!(
                                actual, expected,
                                "format={format:?} src={src:?} offset={offset} length={length}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64", target_os = "macos"))]
    #[test]
    fn a8_row_mask_and_pixels_do_not_cross_guard_pages() {
        use core::ffi::{c_int, c_void};
        unsafe extern "C" {
            fn mmap(
                address: *mut c_void,
                length: usize,
                protection: c_int,
                flags: c_int,
                fd: c_int,
                offset: i64,
            ) -> *mut c_void;
            fn mprotect(address: *mut c_void, length: usize, protection: c_int) -> c_int;
            fn munmap(address: *mut c_void, length: usize) -> c_int;
            fn getpagesize() -> c_int;
        }
        struct Guard {
            base: *mut u8,
            page: usize,
        }
        impl Guard {
            fn new() -> Self {
                unsafe {
                    let page = getpagesize() as usize;
                    let base = mmap(core::ptr::null_mut(), page * 2, 3, 0x1002, -1, 0);
                    assert_ne!(base as isize, -1);
                    assert_eq!(mprotect((base as *mut u8).add(page).cast(), page, 0), 0);
                    Self {
                        base: base.cast(),
                        page,
                    }
                }
            }
            fn tail(&mut self, length: usize) -> &mut [u8] {
                unsafe {
                    core::slice::from_raw_parts_mut(self.base.add(self.page - length), length)
                }
            }
        }
        impl Drop for Guard {
            fn drop(&mut self) {
                unsafe {
                    assert_eq!(munmap(self.base.cast(), self.page * 2), 0);
                }
            }
        }
        let mut masks = Guard::new();
        let mut pixels = Guard::new();
        for length in 0..=40 {
            for format in [
                crate::PixelFormat::Rgba8888,
                crate::PixelFormat::Bgra8888,
                crate::PixelFormat::Bgrx8888,
            ] {
                for src in [
                    [0, 0, 0, 255],
                    [17, 123, 255, 255],
                    [37, 0, 81, 129],
                    [0; 4],
                ] {
                    let context = PreparedA8Blitter::new_with_format(src, format);
                    let mask = masks.tail(length);
                    for (i, x) in mask.iter_mut().enumerate() {
                        *x = [0, 255, 1, 127, 254, 7, 128, 0][i % 8];
                    }
                    let dst = pixels.tail(length * 4);
                    for (i, x) in dst.iter_mut().enumerate() {
                        *x = (i * 73 + 191) as u8;
                    }
                    let mut expected = dst.to_vec();
                    for (p, &coverage) in expected.chunks_exact_mut(4).zip(mask.iter()) {
                        blend_premultiplied(p, context.src, coverage);
                        if context.zero_unused && src[3] != 0 && coverage != 0 {
                            p[3] = 0;
                        }
                    }
                    context.blend_row(dst, mask);
                    assert_eq!(
                        dst, expected,
                        "length={length},src={src:?},format={format:?}"
                    );
                }
            }
        }
    }

    #[cfg(all(target_arch = "aarch64", feature = "simd"))]
    #[test]
    #[ignore = "manual Debug timing; no performance assertion"]
    fn a8_row_debug_kernel_benchmark() {
        use std::time::Instant;
        for format in [crate::PixelFormat::Rgba8888, crate::PixelFormat::Bgrx8888] {
            for src in [[0, 0, 0, 255], [53, 123, 233, 255], [37, 0, 81, 129]] {
                let context = PreparedA8Blitter::new_with_format(src, format);
                for width in [8, 16, 32, 64, 256, 1024] {
                    let mask: Vec<_> = (0..width)
                        .map(|i| [0, 255, 7, 127, 254, 21, 0, 129][i % 8])
                        .collect();
                    let initial: Vec<_> = (0..width * 4).map(|i| (i * 73 + 191) as u8).collect();
                    let iterations = 1_048_576 / width;
                    let mut old_time = Vec::new();
                    let mut new_time = Vec::new();
                    for _ in 0..3 {
                        for old in [true, false] {
                            let mut dst = initial.clone();
                            let start = Instant::now();
                            for _ in 0..iterations {
                                unsafe {
                                    if old {
                                        packed_a8_row(
                                            &context,
                                            std::hint::black_box(&mut dst),
                                            std::hint::black_box(&mask),
                                        );
                                    } else {
                                        context.blend_row(
                                            std::hint::black_box(&mut dst),
                                            std::hint::black_box(&mask),
                                        );
                                    }
                                }
                            }
                            let ms = start.elapsed().as_secs_f64() * 1000.0;
                            std::hint::black_box(&dst);
                            if old {
                                old_time.push(ms);
                            } else {
                                new_time.push(ms);
                            }
                        }
                    }
                    old_time.sort_by(f64::total_cmp);
                    new_time.sort_by(f64::total_cmp);
                    eprintln!(
                        "A8_BENCH format={format:?} src={src:?} width={width} pixels={} packed_ms={:.3} direct_neon_ms={:.3} speedup={:.2}",
                        width * iterations,
                        old_time[1],
                        new_time[1],
                        old_time[1] / new_time[1]
                    );
                }
            }
        }
    }

    #[test]
    #[should_panic(expected = "A8 row exceeds destination")]
    fn a8_row_rejects_short_destination() {
        blend_a8_row(&mut [0; 31], [0, 0, 0, 255], &[255; 8]);
    }

    #[test]
    #[should_panic(expected = "A8 row exceeds destination")]
    fn a8_row_prepared_rejects_short_destination_even_when_transparent() {
        PreparedA8Blitter::new([0; 4]).blend_row(&mut [0; 31], &[0; 8]);
    }
}

// Equivalent packed-channel form of SkBlitMask_opts.h's NEON alpha+1/256
// arithmetic. Each pair of byte channels occupies independent 16-bit lanes;
// products fit those lanes and the source-over sums are bounded by 255.
// This Rust driver avoids per-intrinsic call overhead at opt-level=0.
#[cfg(all(target_arch = "aarch64", feature = "simd"))]
unsafe fn packed_a8_row(context: &PreparedA8Blitter, dst: &mut [u8], mask: &[u8]) {
    #[repr(C, packed)]
    struct Pixel {
        value: u32,
    }
    let src = u32::from_le_bytes(context.src);
    let rb = src & 0x00ff00ff;
    let ag = (src >> 8) & 0x00ff00ff;
    let mut at = 0;
    let ptr = dst.as_mut_ptr();
    while at < mask.len() {
        let cov = mask[at] as u32;
        if cov != 0 {
            let scale = cov + 1;
            let inverse = 256 - ((context.src[3] as u32 * scale) >> 8);
            let p = ptr.add(at * 4) as *mut Pixel;
            let d = u32::from_le((*p).value);
            let r = (((rb * scale) >> 8) & 0x00ff00ff)
                + ((((d & 0x00ff00ff) * inverse) >> 8) & 0x00ff00ff);
            let a = (((ag * scale) >> 8) & 0x00ff00ff)
                + (((((d >> 8) & 0x00ff00ff) * inverse) >> 8) & 0x00ff00ff);
            (*p).value = ((r | (a << 8))
                & if context.zero_unused {
                    0x00ffffff
                } else {
                    u32::MAX
                })
            .to_le();
        }
        at += 1;
    }
}
