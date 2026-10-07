// Copyright 2014 Google Inc.
// Copyright 2016 Google Inc.
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Contiguous RGBA8888 subset of upstream `SkConvertPixels.cpp`.
//!
//! Both alpha conversions use the same-color-space branches. Unpremul-to-premul
//! translates `SkSwizzler_opts.inc::premul_should_swapRB(false)` and its integer
//! `div255_round` kernel. The premul-to-unpremul private
//! implementation translates `SkSwizzler_opts.inc::common_rgbA_to_RGBA` with
//! `kFastUnpremul == false`. That branch deliberately preserves the floating-point
//! operations and nearest-even rounding of the AArch64 raster pipeline.
//! The scalar fallback preserves those same bytes on other architectures.
//! This API does not implement SkImageInfo, row strides, color-space conversion,
//! BGRA swizzling, or the other upstream pixel formats.

/// Restricted alpha metadata for this RGBA8888 conversion adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rgba8888AlphaType {
    Opaque,
    Premul,
    Unpremul,
}

/// Converts equal-size, tightly packed RGBA8888 buffers.
///
/// Like upstream `rect_memcpy`, matching alpha types are copied directly. Like
/// `SkColorSpaceXformSteps`, an opaque destination retains the source alpha type.
/// Incomplete pixels or unequal buffer sizes return false without writing.
#[allow(non_snake_case)]
pub fn SkConvertPixels(
    dst: &mut [u8],
    src: &[u8],
    src_alpha: Rgba8888AlphaType,
    dst_alpha: Rgba8888AlphaType,
) -> bool {
    if src.len() != dst.len() || src.len() % 4 != 0 {
        return false;
    }
    if src_alpha == dst_alpha
        || src_alpha == Rgba8888AlphaType::Opaque
        || dst_alpha == Rgba8888AlphaType::Opaque
    {
        dst.copy_from_slice(src);
        return true;
    }
    match (src_alpha, dst_alpha) {
        (Rgba8888AlphaType::Unpremul, Rgba8888AlphaType::Premul) => {
            #[cfg(all(feature = "simd", target_arch = "aarch64"))]
            unsafe {
                RGBA_to_rgbA_neon(dst, src);
            }
            #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
            RGBA_to_rgbA_portable(dst, src);
        }
        (Rgba8888AlphaType::Premul, Rgba8888AlphaType::Unpremul) => {
            #[cfg(all(feature = "simd", target_arch = "aarch64"))]
            {
                // NEON is mandatory on AArch64. Helpers access complete groups;
                // scalar tails cannot overread either buffer.
                unsafe { common_rgbA_to_RGBA_neon(dst, src) };
            }
            #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
            rgbA_to_RGBA_portable(dst, src);
        }
        _ => unreachable!("matching or opaque alpha types handled above"),
    }
    true
}

// SkSwizzler_opts.inc::RGBA_to_rgbA_portable: (channel * alpha + 127) / 255.
#[allow(non_snake_case)]
fn RGBA_to_rgbA_portable(dst: &mut [u8], src: &[u8]) {
    for (output, input) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
        let alpha = u16::from(input[3]);
        for channel in 0..3 {
            output[channel] = ((u16::from(input[channel]) * alpha + 127) / 255) as u8;
        }
        output[3] = input[3];
    }
}

// SkSwizzler_opts.inc::premul_should_swapRB(false), scale, and div255_round.
#[cfg(all(feature = "simd", target_arch = "aarch64"))]
#[allow(non_snake_case)]
unsafe fn RGBA_to_rgbA_neon(dst: &mut [u8], src: &[u8]) {
    let groups = src.len() / 32;
    if groups != 0 {
        // Direct instruction translation of the upstream eight-pixel loop.
        // Generated std::arch intrinsics each cross a vector ABI at opt-level
        // zero; this local backend keeps the same intermediates in registers.
        // UMULL -> URSHR #8 -> RADDHN is precisely scale/div255_round; no
        // approximation or alternate division formula is used here.
        // Both pointers cover groups*32 bytes (equal slice lengths were checked
        // by SkConvertPixels). LD4/ST4 accept byte-unaligned addresses, advance
        // only after a complete group, and leave the scalar tail untouched.
        // NEON is mandatory on AArch64; the Rust slices cannot alias. Explicit
        // clobbers include every vector register written, and flags are local.
        core::arch::asm!(
            "2:",
            "ld4 {{v0.8b, v1.8b, v2.8b, v3.8b}}, [{src}], #32",
            // Upstream premultiplies blue, green, then red.
            "umull v4.8h, v2.8b, v3.8b",
            "urshr v5.8h, v4.8h, #8",
            "raddhn v2.8b, v4.8h, v5.8h",
            "umull v4.8h, v1.8b, v3.8b",
            "urshr v5.8h, v4.8h, #8",
            "raddhn v1.8b, v4.8h, v5.8h",
            "umull v4.8h, v0.8b, v3.8b",
            "urshr v5.8h, v4.8h, #8",
            "raddhn v0.8b, v4.8h, v5.8h",
            "st4 {{v0.8b, v1.8b, v2.8b, v3.8b}}, [{dst}], #32",
            "subs {groups}, {groups}, #1",
            "b.ne 2b",
            src = inout(reg) src.as_ptr() => _,
            dst = inout(reg) dst.as_mut_ptr() => _,
            groups = inout(reg) groups => _,
            out("v0") _, out("v1") _, out("v2") _, out("v3") _,
            out("v4") _, out("v5") _,
            options(nostack),
        );
    }
    // groups is a quotient of this slice length, so groups*32 cannot overflow.
    let offset = groups * 32;
    RGBA_to_rgbA_portable(&mut dst[offset..], &src[offset..]);
}

// SkSwizzler_opts.inc::reciprocal_alpha_portable, rgbA_to_CCCA,
// unpremul_simulating_RP, and pixel_round_as_RP (ARM64 branch).
#[allow(non_snake_case)]
fn rgbA_to_RGBA_portable(dst: &mut [u8], src: &[u8]) {
    for (output, input) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
        let normalized_a = f32::from(input[3]) * (1.0f32 / 255.0);
        let reciprocal_a = if normalized_a != 0.0 {
            1.0 / normalized_a
        } else {
            0.0
        };
        for channel in 0..3 {
            let normalized_c = f32::from(input[channel]) * (1.0f32 / 255.0);
            let divided = normalized_c * reciprocal_a;
            let denormalized = divided * 255.0;
            output[channel] = denormalized.min(255.0).round_ties_even() as u8;
        }
        output[3] = input[3];
    }
}

#[cfg(all(feature = "simd", target_arch = "aarch64"))]
#[allow(non_snake_case)]
unsafe fn common_rgbA_to_RGBA_neon(dst: &mut [u8], src: &[u8]) {
    use core::arch::aarch64::*;

    #[inline(always)]
    unsafe fn to_normalized(value: uint16x4_t) -> float32x4_t {
        vmulq_n_f32(vcvtq_f32_u32(vmovl_u16(value)), 1.0f32 / 255.0)
    }
    #[inline(always)]
    unsafe fn reciprocal(alpha: float32x4_t) -> float32x4_t {
        let mask = vcgtq_f32(alpha, vdupq_n_f32(0.0));
        // Use actual division, as in upstream. Approximate reciprocal changes
        // half-way quantization results for some legal premultiplied pixels.
        let value = vdivq_f32(vdupq_n_f32(1.0), alpha);
        vreinterpretq_f32_u32(vandq_u32(mask, vreinterpretq_u32_f32(value)))
    }
    #[inline(always)]
    unsafe fn unpremul_half(inverse: float32x4_t, value: uint16x4_t) -> uint16x4_t {
        let normalized = to_normalized(value);
        let divided = vmulq_f32(inverse, normalized);
        let denormalized = vmulq_n_f32(divided, 255.0);
        vqmovn_u32(vcvtnq_u32_f32(denormalized))
    }
    #[inline(always)]
    unsafe fn unpremul(value: uint8x8_t, low: float32x4_t, high: float32x4_t) -> uint8x8_t {
        let expanded = vmovl_u8(value);
        let low = unpremul_half(low, vget_low_u16(expanded));
        let high = unpremul_half(high, vget_high_u16(expanded));
        vqmovn_u16(vcombine_u16(low, high))
    }

    let mut offset = 0;
    while offset + 32 <= src.len() {
        let input = vld4_u8(src.as_ptr().add(offset));
        let alpha = vmovl_u8(input.3);
        let inverse_low = reciprocal(to_normalized(vget_low_u16(alpha)));
        let inverse_high = reciprocal(to_normalized(vget_high_u16(alpha)));
        let output = uint8x8x4_t(
            unpremul(input.0, inverse_low, inverse_high),
            unpremul(input.1, inverse_low, inverse_high),
            unpremul(input.2, inverse_low, inverse_high),
            input.3,
        );
        vst4_u8(dst.as_mut_ptr().add(offset), output);
        offset += 32;
    }
    rgbA_to_RGBA_portable(&mut dst[offset..], &src[offset..]);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Independent copy of the previous browser readback formula, used only as
    // a compatibility oracle rather than implementing the new conversion.
    fn previous_readback(input: &[u8]) -> Vec<u8> {
        let mut output = input.to_vec();
        for pixel in output.chunks_exact_mut(4) {
            let alpha = f32::from(pixel[3]) * (1.0f32 / 255.0);
            let inverse = if alpha > 0.0 { 1.0 / alpha } else { 0.0 };
            for channel in &mut pixel[..3] {
                let loaded = f32::from(*channel) * (1.0f32 / 255.0);
                let straight = loaded * inverse;
                *channel = (straight * 255.0).clamp(0.0, 255.0).round_ties_even() as u8;
            }
        }
        output
    }
    fn convert(input: &[u8]) -> Vec<u8> {
        let mut output = vec![0; input.len()];
        assert!(SkConvertPixels(
            &mut output,
            input,
            Rgba8888AlphaType::Premul,
            Rgba8888AlphaType::Unpremul,
        ));
        output
    }

    #[test]
    fn every_alpha_and_legal_premultiplied_channel_matches_previous_readback() {
        let mut input = Vec::new();
        for alpha in 0..=255u8 {
            for channel in 0..=alpha {
                input.extend_from_slice(&[channel, alpha - channel, channel / 2, alpha]);
            }
        }
        assert_eq!(convert(&input), previous_readback(&input));
        let mut scalar = vec![0; input.len()];
        rgbA_to_RGBA_portable(&mut scalar, &input);
        assert_eq!(scalar, previous_readback(&input));
    }

    #[test]
    fn every_alpha_and_channel_including_out_of_range_matches_clamped_readback() {
        let mut input = Vec::new();
        for alpha in 0..=255u8 {
            for channel in 0..=255u8 {
                input.extend_from_slice(&[channel, 255 - channel, channel, alpha]);
            }
        }
        assert_eq!(convert(&input), previous_readback(&input));
    }

    #[test]
    fn four_and_eight_pixel_tails_and_unaligned_buffers_preserve_sentinels() {
        for count in 0..=35usize {
            for source_offset in 0..=3 {
                for destination_offset in 0..=3 {
                    let mut source = vec![0x73; source_offset + 4 * count + 7];
                    let input = &mut source[source_offset..source_offset + count * 4];
                    for (index, pixel) in input.chunks_exact_mut(4).enumerate() {
                        let alpha = [0, 1, 6, 10, 42, 128, 254, 255][index % 8];
                        pixel.copy_from_slice(&[alpha / 2, alpha / 3, alpha, alpha]);
                    }
                    let expected = previous_readback(input);
                    let mut destination = vec![0xa5; destination_offset + count * 4 + 11];
                    assert!(SkConvertPixels(
                        &mut destination[destination_offset..destination_offset + count * 4],
                        input,
                        Rgba8888AlphaType::Premul,
                        Rgba8888AlphaType::Unpremul,
                    ));
                    assert_eq!(
                        &destination[destination_offset..destination_offset + count * 4],
                        expected
                    );
                    assert!(destination[..destination_offset].iter().all(|&v| v == 0xa5));
                    assert!(destination[destination_offset + count * 4..]
                        .iter()
                        .all(|&v| v == 0xa5));
                }
            }
        }
    }

    #[test]
    fn matching_and_opaque_metadata_copy_and_invalid_requests_do_not_write() {
        use Rgba8888AlphaType::*;
        let source = [1, 2, 3, 255, 17, 18, 19, 127];
        for (source_alpha, destination_alpha) in [
            (Premul, Premul),
            (Unpremul, Unpremul),
            (Opaque, Premul),
            (Opaque, Unpremul),
            (Premul, Opaque),
            (Unpremul, Opaque),
        ] {
            let mut destination = [0; 8];
            assert!(SkConvertPixels(
                &mut destination,
                &source,
                source_alpha,
                destination_alpha
            ));
            assert_eq!(destination, source);
        }
        let mut destination = [0x77; 8];
        assert!(!SkConvertPixels(
            &mut destination[..7],
            &source[..7],
            Premul,
            Unpremul
        ));
        assert!(!SkConvertPixels(
            &mut destination[..4],
            &source,
            Premul,
            Unpremul
        ));
        assert_eq!(destination, [0x77; 8]);
    }

    /// A dedicated test for a captured premultiplied canvas. The integration
    /// runner sets SKIA_CONVERT_PIXELS_FIXTURE to a raw RGBA8888 file.
    #[test]
    #[ignore = "requires captured premultiplied canvas via SKIA_CONVERT_PIXELS_FIXTURE"]
    fn captured_canvas_matches_previous_readback() {
        let path = std::env::var("SKIA_CONVERT_PIXELS_FIXTURE").expect("fixture path");
        let input = std::fs::read(path).expect("premultiplied canvas fixture");
        assert_eq!(convert(&input), previous_readback(&input));
    }
    fn previous_premultiply(input: &[u8]) -> Vec<u8> {
        let mut output = Vec::with_capacity(input.len());
        for pixel in input.chunks_exact(4) {
            let color =
                crate::ColorU8::from_rgba(pixel[0], pixel[1], pixel[2], pixel[3]).premultiply();
            output.extend_from_slice(&[color.red(), color.green(), color.blue(), color.alpha()]);
        }
        output
    }

    #[test]
    fn premultiply_every_alpha_and_channel_matches_color_u8_and_upstream_scalar() {
        let mut input = Vec::new();
        for alpha in 0..=255u8 {
            for channel in 0..=255u8 {
                input.extend_from_slice(&[channel, 255 - channel, channel / 2, alpha]);
            }
        }
        let expected = previous_premultiply(&input);
        let mut scalar = vec![0; input.len()];
        RGBA_to_rgbA_portable(&mut scalar, &input);
        assert_eq!(scalar, expected);
        let mut dispatched = vec![0; input.len()];
        assert!(SkConvertPixels(
            &mut dispatched,
            &input,
            Rgba8888AlphaType::Unpremul,
            Rgba8888AlphaType::Premul,
        ));
        assert_eq!(dispatched, expected);
    }

    #[test]
    fn premultiply_unaligned_buffers_and_all_eight_pixel_tails_preserve_sentinels() {
        for count in 0..=35usize {
            for source_offset in 0..=7 {
                for destination_offset in 0..=7 {
                    let mut source = vec![0x73; source_offset + 4 * count + 7];
                    let input = &mut source[source_offset..source_offset + count * 4];
                    for (index, pixel) in input.chunks_exact_mut(4).enumerate() {
                        let alpha = [0, 1, 6, 10, 42, 128, 254, 255][index % 8];
                        pixel.copy_from_slice(&[255, (index * 29) as u8, 73, alpha]);
                    }
                    let expected = previous_premultiply(input);
                    let mut destination = vec![0xa5; destination_offset + count * 4 + 11];
                    assert!(SkConvertPixels(
                        &mut destination[destination_offset..destination_offset + count * 4],
                        input,
                        Rgba8888AlphaType::Unpremul,
                        Rgba8888AlphaType::Premul,
                    ));
                    assert_eq!(
                        &destination[destination_offset..destination_offset + count * 4],
                        expected,
                    );
                    assert!(destination[..destination_offset].iter().all(|&v| v == 0xa5));
                    assert!(destination[destination_offset + count * 4..]
                        .iter()
                        .all(|&v| v == 0xa5));
                }
            }
        }
        let mut destination = [0x77; 8];
        for (input, output) in [(7, 7), (8, 4), (4, 8)] {
            assert!(!SkConvertPixels(
                &mut destination[..output],
                &[0x21; 8][..input],
                Rgba8888AlphaType::Unpremul,
                Rgba8888AlphaType::Premul,
            ));
            assert_eq!(destination, [0x77; 8]);
        }
    }
    // macOS guard pages turn any load or store past the last complete pixel
    // into a fault. These OS calls are confined to this test; production has
    // no foreign-code dependency for conversion.
    #[cfg(target_os = "macos")]
    #[test]
    fn premultiply_vector_groups_and_tails_do_not_cross_guard_pages() {
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
        struct GuardedBuffer {
            base: *mut u8,
            page: usize,
        }
        impl GuardedBuffer {
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
            fn tail(&mut self, len: usize) -> &mut [u8] {
                assert!(len <= self.page);
                unsafe { core::slice::from_raw_parts_mut(self.base.add(self.page - len), len) }
            }
        }
        impl Drop for GuardedBuffer {
            fn drop(&mut self) {
                unsafe {
                    assert_eq!(munmap(self.base.cast(), self.page * 2), 0);
                }
            }
        }
        let mut source = GuardedBuffer::new();
        let mut destination = GuardedBuffer::new();
        for count in 0..=35 {
            let input = source.tail(count * 4);
            for (index, pixel) in input.chunks_exact_mut(4).enumerate() {
                pixel.copy_from_slice(&[255, (index * 37) as u8, 13, (index * 19) as u8]);
            }
            let expected = previous_premultiply(input);
            let output = destination.tail(count * 4);
            assert!(SkConvertPixels(
                output,
                input,
                Rgba8888AlphaType::Unpremul,
                Rgba8888AlphaType::Premul
            ));
            assert_eq!(output, expected);
        }
    }
    #[test]
    #[ignore = "isolated debug RGBA-to-premul timing; optional captured RGBA fixture"]
    fn rgba_to_premul_kernel_profile() {
        let input = if let Ok(path) = std::env::var("SKIA_PREMUL_FIXTURE") {
            std::fs::read(path).expect("RGBA fixture")
        } else {
            // A 1600x1200 high-DPI page with all alpha/channel combinations.
            let mut input = vec![0; 1600 * 1200 * 4];
            for (i, pixel) in input.chunks_exact_mut(4).enumerate() {
                pixel.copy_from_slice(&[i as u8, (i >> 8) as u8, (i >> 3) as u8, (i >> 4) as u8]);
            }
            input
        };
        let mut output = vec![0; input.len()];
        let mut elapsed = Vec::new();
        for _ in 0..12 {
            let start = std::time::Instant::now();
            assert!(SkConvertPixels(
                &mut output,
                &input,
                Rgba8888AlphaType::Unpremul,
                Rgba8888AlphaType::Premul
            ));
            elapsed.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        elapsed.sort_by(f64::total_cmp);
        eprintln!(
            "RGBA_PREMUL_KERNEL_MS bytes={} median={:.6} samples={elapsed:?}",
            input.len(),
            elapsed[elapsed.len() / 2]
        );
        assert_eq!(output, previous_premultiply(&input));
        if let Ok(path) = std::env::var("SKIA_PREMUL_OUTPUT") {
            std::fs::write(path, output).expect("premul output");
        }
    }
}
