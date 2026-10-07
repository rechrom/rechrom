//! Skia raster image-filter blur passes over premultiplied N32 channels.
use crate::compat::pixel_view::PixelView as SkPixmap;

// Chromium Skia: src/core/SkBlurEngine.cpp ThreeBoxApproxPass
// and eval_blur_passes. Source image filters use X then Y, with transparent
// samples beyond the input bounds and byte storage between the two passes.
#[cfg(any(
    test,
    not(all(target_arch = "aarch64", target_endian = "little", feature = "simd"))
))]
fn scan(input: &[u8], sigma: f32) -> Vec<u8> {
    if sigma <= 0.03 || input.iter().all(|&v| v == 0) {
        return input.to_vec();
    }
    // The unchanged native N32 reference selects ThreeBoxApproxPass even
    // below sigma 2 (verified against sigma 0.5 and 1); preserve that profile.
    assert!(
        sigma <= 135.0,
        "large image-filter blur requires Skia rescaling"
    );
    let window = (f64::from(sigma * 3.0) * (2.0 * f64::from(std::f32::consts::PI)).sqrt() / 4.0
        + 0.5)
        .floor() as usize;
    if window <= 1 {
        return input.to_vec();
    }
    let sizes = [
        window - 1,
        window - 1,
        window - 1 + usize::from(window % 2 == 0),
    ];
    let border = if window % 2 == 1 {
        3 * ((window - 1) / 2)
    } else {
        3 * (window / 2) - 1
    };
    let divisor = window * window * (window + usize::from(window % 2 == 0));
    let factor = ((1_u64 << 32) as f64 / divisor as f64).round() as u64;
    let mut buffers = sizes.map(|n| vec![0_u32; n]);
    let mut cursors = [0; 3];
    let mut sums = [0, 0, ((divisor + 1) >> 1) as u32];
    let mut output = vec![0; input.len()];
    for x in 0..input.len() + border {
        let leading = u32::from(input.get(x).copied().unwrap_or(0));
        sums[0] += leading;
        sums[1] += sums[0];
        sums[2] += sums[1];
        if x >= border {
            output[x - border] = ((u64::from(sums[2]) * factor) >> 32) as u8;
        }
        for level in (0..3).rev() {
            sums[level] -= buffers[level][cursors[level]];
            buffers[level][cursors[level]] = if level == 0 { leading } else { sums[level - 1] };
            cursors[level] = (cursors[level] + 1) % sizes[level];
        }
    }
    output
}

// Pixel channels are independent. The vector pass evaluates the same integer
// ThreeBoxApprox recurrence in four lanes, retaining byte quantization between
// axes. It reads ahead of the in-place output by `border`, so no source sample
// is overwritten before it is consumed.
#[cfg(all(target_arch = "aarch64", target_endian = "little", feature = "simd"))]
mod packed {
    use std::arch::aarch64::*;

    // Find the admitted source support in packed rows. This avoids walking
    // every transparent column of a large saveLayer during the vertical pass.
    pub(super) fn bounds(pixels: &[u8], width: usize, height: usize) -> Option<[usize; 4]> {
        assert_eq!(pixels.len(), width * height * 4);
        let mut bounds = [width, height, 0, 0];
        for (y, row) in pixels.chunks_exact(width * 4).enumerate() {
            let mut left = 0;
            let mut right = width;
            // SAFETY: each vector/scalar load stays within this initialized
            // row. Unaligned N32 storage is supported by both load variants.
            unsafe {
                let base = row.as_ptr();
                while left + 4 <= width
                    && vmaxvq_u32(vld1q_u32(base.add(left * 4).cast::<u32>())) == 0
                {
                    left += 4;
                }
                while left < width
                    && core::ptr::read_unaligned(base.add(left * 4).cast::<u32>()) == 0
                {
                    left += 1;
                }
                if left == width {
                    continue;
                }
                while right >= left + 4
                    && vmaxvq_u32(vld1q_u32(base.add((right - 4) * 4).cast::<u32>())) == 0
                {
                    right -= 4;
                }
                while right > left
                    && core::ptr::read_unaligned(base.add((right - 1) * 4).cast::<u32>()) == 0
                {
                    right -= 1;
                }
            }
            bounds[0] = bounds[0].min(left);
            bounds[1] = bounds[1].min(y);
            bounds[2] = bounds[2].max(right);
            bounds[3] = y + 1;
        }
        (bounds[2] != 0).then_some(bounds)
    }

    pub(super) fn axis(
        pixels: &mut [u8],
        width: usize,
        height: usize,
        sigma: f32,
        vertical: bool,
        bounds: &mut [usize; 4],
    ) {
        if sigma <= 0.03 {
            return;
        }
        // The scalar path admits a completely transparent image even when
        // the nontransparent large-sigma rescaling path is unsupported.
        if !(sigma <= 135.0) && pixels.iter().all(|&v| v == 0) {
            return;
        }
        assert!(
            sigma <= 135.0,
            "large image-filter blur requires Skia rescaling"
        );
        let window = (f64::from(sigma * 3.0) * (2.0 * f64::from(std::f32::consts::PI)).sqrt() / 4.0
            + 0.5)
            .floor() as usize;
        if window <= 1 {
            return;
        }
        let sizes = [
            window - 1,
            window - 1,
            window - 1 + usize::from(window % 2 == 0),
        ];
        let border = if window % 2 == 1 {
            3 * ((window - 1) / 2)
        } else {
            3 * (window / 2) - 1
        };
        let divisor = window * window * (window + usize::from(window % 2 == 0));
        let factor = ((1_u64 << 32) as f64 / divisor as f64).round() as u32;
        let (length, stride, line_step, line_start, line_end, source_start, source_end) =
            if vertical {
                (
                    height,
                    width * 4,
                    4,
                    bounds[0],
                    bounds[2],
                    bounds[1],
                    bounds[3],
                )
            } else {
                (
                    width,
                    4,
                    width * 4,
                    bounds[1],
                    bounds[3],
                    bounds[0],
                    bounds[2],
                )
            };
        assert_eq!(pixels.len(), width * height * 4);
        // SAFETY: AArch64 has NEON. Each line contains `length` complete N32
        // pixels with the checked strides above. Ring indices wrap at their
        // nonzero lengths; stores are restricted to [0,length). Intermediates
        // fit u32 for the admitted sigma <=135 just as in the scalar reference.
        unsafe {
            let zero = vdupq_n_u32(0);
            let mut buffers = sizes.map(|n| vec![zero; n]);
            for line in line_start..line_end {
                let base = pixels.as_mut_ptr().add(line * line_step);
                let mut first = source_start;
                if stride == 4 {
                    while first + 4 <= source_end
                        && vmaxvq_u32(vld1q_u32(base.add(first * 4).cast::<u32>())) == 0
                    {
                        first += 4;
                    }
                }
                while first < source_end
                    && core::ptr::read_unaligned(base.add(first * stride).cast::<u32>()) == 0
                {
                    first += 1;
                }
                if first == source_end {
                    continue;
                }
                let mut end = source_end;
                if stride == 4 {
                    while end >= first + 4
                        && vmaxvq_u32(vld1q_u32(base.add((end - 4) * 4).cast::<u32>())) == 0
                    {
                        end -= 4;
                    }
                }
                while end > first
                    && core::ptr::read_unaligned(base.add((end - 1) * stride).cast::<u32>()) == 0
                {
                    end -= 1;
                }
                for b in &mut buffers {
                    b.fill(zero);
                }
                let mut cursors = [0usize; 3];
                let mut s0 = zero;
                let mut s1 = zero;
                let mut s2 = vdupq_n_u32(((divisor + 1) >> 1) as u32);
                let b0 = buffers[0].as_mut_ptr();
                let b1 = buffers[1].as_mut_ptr();
                let b2 = buffers[2].as_mut_ptr();
                // Skipping transparent prefixes starts with the same zero
                // delay lines. The finite support ends 2*border input steps
                // after the last nonzero source; pixels beyond stay zero.
                for x in first..(end + 2 * border).min(length + border) {
                    let leading = if x < length {
                        let word = core::ptr::read_unaligned(base.add(x * stride).cast::<u32>());
                        vmovl_u16(vget_low_u16(vmovl_u8(vcreate_u8(u64::from(word)))))
                    } else {
                        zero
                    };
                    s0 = vaddq_u32(s0, leading);
                    s1 = vaddq_u32(s1, s0);
                    s2 = vaddq_u32(s2, s1);
                    if x >= border {
                        let lo = vshrn_n_u64::<32>(vmull_n_u32(vget_low_u32(s2), factor));
                        let hi = vshrn_n_u64::<32>(vmull_n_u32(vget_high_u32(s2), factor));
                        let halves = vmovn_u32(vcombine_u32(lo, hi));
                        let bytes = vmovn_u16(vcombine_u16(halves, vdup_n_u16(0)));
                        let word = vget_lane_u32::<0>(vreinterpret_u32_u8(bytes));
                        core::ptr::write_unaligned(
                            base.add((x - border) * stride).cast::<u32>(),
                            word,
                        );
                    }
                    s2 = vsubq_u32(s2, *b2.add(cursors[2]));
                    *b2.add(cursors[2]) = s1;
                    s1 = vsubq_u32(s1, *b1.add(cursors[1]));
                    *b1.add(cursors[1]) = s0;
                    s0 = vsubq_u32(s0, *b0.add(cursors[0]));
                    *b0.add(cursors[0]) = leading;
                    cursors[0] += 1;
                    if cursors[0] == sizes[0] {
                        cursors[0] = 0;
                    }
                    cursors[1] += 1;
                    if cursors[1] == sizes[1] {
                        cursors[1] = 0;
                    }
                    cursors[2] += 1;
                    if cursors[2] == sizes[2] {
                        cursors[2] = 0;
                    }
                }
            }
        }
        if vertical {
            bounds[1] = bounds[1].saturating_sub(border);
            bounds[3] = (bounds[3] + border).min(height);
        } else {
            bounds[0] = bounds[0].saturating_sub(border);
            bounds[2] = (bounds[2] + border).min(width);
        }
    }
}

pub fn blur(pixmap: &mut SkPixmap<'_>, sigma_x: f32, sigma_y: f32) {
    let width = pixmap.width() as usize;
    let height = pixmap.height() as usize;
    #[cfg(all(target_arch = "aarch64", target_endian = "little", feature = "simd"))]
    {
        if sigma_x <= 0.03 && sigma_y <= 0.03 {
            return;
        }
        let Some(mut bounds) = packed::bounds(pixmap.data(), width, height) else {
            return;
        };
        packed::axis(
            pixmap.data_mut(),
            width,
            height,
            sigma_x,
            false,
            &mut bounds,
        );
        packed::axis(pixmap.data_mut(), width, height, sigma_y, true, &mut bounds);
    }
    #[cfg(not(all(target_arch = "aarch64", target_endian = "little", feature = "simd")))]
    scalar_blur(pixmap, width, height, sigma_x, sigma_y);
}

#[cfg(any(
    test,
    not(all(target_arch = "aarch64", target_endian = "little", feature = "simd"))
))]
fn scalar_blur(pixmap: &mut SkPixmap<'_>, width: usize, height: usize, sigma_x: f32, sigma_y: f32) {
    let mut line = vec![0; width.max(height)];
    for channel in 0..4 {
        if !(sigma_x <= 0.03) {
            for y in 0..height {
                for x in 0..width {
                    line[x] = pixmap.data()[(y * width + x) * 4 + channel];
                }
                let result = scan(&line[..width], sigma_x);
                for x in 0..width {
                    pixmap.data_mut()[(y * width + x) * 4 + channel] = result[x];
                }
            }
        }
        if !(sigma_y <= 0.03) {
            for x in 0..width {
                for y in 0..height {
                    line[y] = pixmap.data()[(y * width + x) * 4 + channel];
                }
                let result = scan(&line[..height], sigma_y);
                for y in 0..height {
                    pixmap.data_mut()[(y * width + x) * 4 + channel] = result[y];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packed_blur_matches_scalar_transparent_edges_and_asymmetric_axes() {
        for (w, h) in [(1, 1), (2, 7), (19, 13), (97, 43)] {
            for (sx, sy) in [
                (0., 0.),
                (0.5, 0.),
                (0., 1.),
                (1., 2.),
                (3., 7.),
                (12., 4.),
                (135., 0.5),
            ] {
                let mut source = vec![0; w * h * 4];
                for (i, p) in source.chunks_exact_mut(4).enumerate() {
                    if i % 5 == 0 {
                        continue;
                    }
                    let a = ((i * 73 + 29) % 256) as u8;
                    p.copy_from_slice(&[a / 3, a / 2, a, a]);
                }
                let mut expected = source.clone();
                scalar_blur(
                    &mut SkPixmap::from_rgba(&mut expected, w as u32, h as u32).unwrap(),
                    w,
                    h,
                    sx,
                    sy,
                );
                blur(
                    &mut SkPixmap::from_rgba(&mut source, w as u32, h as u32).unwrap(),
                    sx,
                    sy,
                );
                assert_eq!(source, expected, "size={w}x{h} sigma={sx},{sy}");
            }
        }
    }
    #[test]
    fn packed_blur_sparse_support_and_unaligned_storage_match_scalar() {
        let (w, h) = (97, 43);
        for anchor in [None, Some((0, 0)), Some((48, 21)), Some((96, 42))] {
            for (sx, sy) in [(0.5, 1.), (4., 7.), (35., 0.), (0., 135.)] {
                let mut guarded = vec![0xabu8; w * h * 4 + 11];
                let source = &mut guarded[3..3 + w * h * 4];
                source.fill(0);
                if let Some((x, y)) = anchor {
                    source[(y * w + x) * 4..(y * w + x) * 4 + 4]
                        .copy_from_slice(&[11, 53, 71, 131]);
                }
                let mut expected = source.to_vec();
                scalar_blur(
                    &mut SkPixmap::from_rgba(&mut expected, w as u32, h as u32).unwrap(),
                    w,
                    h,
                    sx,
                    sy,
                );
                blur(
                    &mut SkPixmap::from_rgba(source, w as u32, h as u32).unwrap(),
                    sx,
                    sy,
                );
                assert_eq!(source, expected, "anchor={anchor:?} sigma={sx},{sy}");
                assert!(guarded[..3]
                    .iter()
                    .chain(&guarded[3 + w * h * 4..])
                    .all(|&b| b == 0xab));
            }
        }
    }
    #[test]
    #[ignore = "manual paired Debug blur timing"]
    fn google_filter_blur_timing() {
        let (w, h) = (700, 120);
        let source: Vec<u8> = (0..w * h * 4)
            .map(|i| ((i * 17 + i / 7) % 256) as u8)
            .collect();
        for sigma in [(0., 0.), (4., 4.), (12., 3.)] {
            let mut old = source.clone();
            let t = std::time::Instant::now();
            scalar_blur(
                &mut SkPixmap::from_rgba(&mut old, w as u32, h as u32).unwrap(),
                w,
                h,
                sigma.0,
                sigma.1,
            );
            let before = t.elapsed();
            let mut new = source.clone();
            let t = std::time::Instant::now();
            blur(
                &mut SkPixmap::from_rgba(&mut new, w as u32, h as u32).unwrap(),
                sigma.0,
                sigma.1,
            );
            let after = t.elapsed();
            assert_eq!(old, new);
            println!(
                "blur-paired sigma={sigma:?} scalar_ms={:.3} packed_ms={:.3}",
                before.as_secs_f64() * 1000.,
                after.as_secs_f64() * 1000.
            );
        }
    }
}
