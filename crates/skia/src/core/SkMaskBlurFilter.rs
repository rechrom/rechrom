//! PlanGauss and A8 small/three-box blur algorithms from SkMaskBlurFilter.cpp.
//! Rust buffer scanning is adapted; arithmetic/order retain the existing port.
use crate::raster::{IntRect, Mask};
struct PlanGauss {
    pass_sizes: [usize; 3],
    border: usize,
    weight: u64,
}

impl PlanGauss {
    fn new(sigma: f32) -> Self {
        let possible_window = (f64::from(sigma) * 3.0 * (2.0 * std::f64::consts::PI).sqrt() / 4.0
            + 0.5)
            .floor() as usize;
        let window = possible_window.max(1);
        let pass_sizes = [
            window - 1,
            window - 1,
            window - 1 + usize::from(window % 2 == 0),
        ];
        let border = if window % 2 == 1 {
            3 * ((window - 1) / 2)
        } else {
            3 * (window / 2) - 1
        };
        let divisor = if window % 2 == 1 {
            window.pow(3)
        } else {
            window.pow(3) + window.pow(2)
        };
        let weight = ((1_u64 << 32) as f64 / divisor as f64).round() as u64;
        Self {
            pass_sizes,
            border,
            weight,
        }
    }

    #[cfg(test)]
    fn scan(&self, src: &[u8]) -> Vec<u8> {
        let mut output = vec![0; src.len() + 2 * self.border];
        Scan::new(self).blur(src, &mut output);
        output
    }
}

/// PlanGauss::Scan scratch is reused for all rows and columns. The three
/// integrators and cursor updates have the same order as the upstream Scan;
/// unsigned wrapping operations match C++ uint32_t arithmetic. The caller's
/// 135 sigma clamp keeps all legitimate A8 accumulator values representable.
struct Scan {
    weight: u64,
    border: usize,
    buffers: [Vec<u32>; 3],
}

impl Scan {
    fn new(plan: &PlanGauss) -> Self {
        Self {
            weight: plan.weight,
            border: plan.border,
            buffers: plan.pass_sizes.map(|size| vec![0_u32; size]),
        }
    }

    fn blur(&mut self, src: &[u8], dst: &mut [u8]) {
        // Validate the pointer walk's extent once; the hot loops below then
        // mirror upstream AlphaIter/dstStride=1 and ring cursor arithmetic.
        assert!(!src.is_empty());
        assert_eq!(dst.len(), src.len() + 2 * self.border);
        let [buffer0, buffer1, buffer2] = &mut self.buffers;
        let (length0, length1, length2) = (buffer0.len(), buffer1.len(), buffer2.len());
        let (base0, base1, base2) = (
            buffer0.as_mut_ptr(),
            buffer1.as_mut_ptr(),
            buffer2.as_mut_ptr(),
        );
        // SAFETY: each ring allocation remains fixed throughout this call.
        // Every cursor is either the start of an empty ring (never read) or
        // a valid element of its nonempty ring. Increment reaches at most its
        // one-past-end pointer and immediately resets to the ring start.
        // src/dst are nonaliasing initialized slices with the checked extents;
        // forward writes `max(src.len(), 2*border+1)` bytes, <= dst.len() because
        // src is nonempty. Reverse fills exactly the remaining suffix, whose
        // length is <= src.len(). No output or source pointer escapes.
        // Ring zeroing uses the same raw pointers, avoiding intervening mutable
        // slice reborrows while the cursors remain live.
        unsafe {
            let (end0, end1, end2) = (base0.add(length0), base1.add(length1), base2.add(length2));
            core::ptr::write_bytes(base0, 0, length0);
            core::ptr::write_bytes(base1, 0, length1);
            core::ptr::write_bytes(base2, 0, length2);
            let (mut cursor0, mut cursor1, mut cursor2) = (base0, base1, base2);
            let (mut sum0, mut sum1, mut sum2) = (0u32, 0u32, 0u32);
            // Three source phases share the literal upstream update sequence.
            // Expansion avoids a per-pixel closure or level-loop dispatch.
            macro_rules! advance {
                ($alpha:expr) => {{
                    let leading = u32::from($alpha);
                    sum0 = sum0.wrapping_add(leading);
                    sum1 = sum1.wrapping_add(sum0);
                    sum2 = sum2.wrapping_add(sum1);
                    let output = ((self.weight * u64::from(sum2) + (1u64 << 31)) >> 32) as u8;
                    if length2 != 0 {
                        sum2 = sum2.wrapping_sub(*cursor2);
                        *cursor2 = sum1;
                        cursor2 = cursor2.add(1);
                        if cursor2 == end2 {
                            cursor2 = base2;
                        }
                    }
                    if length1 != 0 {
                        sum1 = sum1.wrapping_sub(*cursor1);
                        *cursor1 = sum0;
                        cursor1 = cursor1.add(1);
                        if cursor1 == end1 {
                            cursor1 = base1;
                        }
                    }
                    if length0 != 0 {
                        sum0 = sum0.wrapping_sub(*cursor0);
                        *cursor0 = leading;
                        cursor0 = cursor0.add(1);
                        if cursor0 == end0 {
                            cursor0 = base0;
                        }
                    }
                    output
                }};
            }
            let source_begin = src.as_ptr();
            let source_end = source_begin.add(src.len());
            let mut source = source_begin;
            let mut output = dst.as_mut_ptr();
            let output_end = output.add(dst.len());
            while source != source_end {
                *output = advance!(*source);
                source = source.add(1);
                output = output.add(1);
            }
            let no_change_count = (2 * self.border + 1).saturating_sub(src.len());
            for _ in 0..no_change_count {
                *output = advance!(0u8);
                output = output.add(1);
            }
            core::ptr::write_bytes(base0, 0, length0);
            core::ptr::write_bytes(base1, 0, length1);
            core::ptr::write_bytes(base2, 0, length2);
            sum0 = 0;
            sum1 = 0;
            sum2 = 0;
            // Retaining the source's ring positions after zeroing is equivalent
            // to rotating an all-zero ring, including odd/even window sizes.
            let mut output_cursor = output_end;
            while output_cursor != output {
                output_cursor = output_cursor.sub(1);
                source = source.sub(1);
                *output_cursor = advance!(*source);
            }
        }
    }
}

// Local mask ownership adapter: source support is a conservative nonzero
// superset. Scan only those rows to recover exact alpha bounds for SkMask.
fn source_bounds(mask: &Mask, support: Option<IntRect>) -> Option<(usize, usize, usize, usize)> {
    let device = IntRect::from_xywh(0, 0, mask.width(), mask.height())?;
    let support = support?.intersect(&device)?;
    let stride = mask.width() as usize;
    let (mut left, mut top, mut right, mut bottom) = (
        support.right() as usize,
        support.bottom() as usize,
        0usize,
        0usize,
    );
    for y in support.top() as usize..support.bottom() as usize {
        let start = y * stride + support.left() as usize;
        let end = y * stride + support.right() as usize;
        let row = &mask.data()[start..end];
        if let Some(first) = row.iter().position(|&a| a != 0) {
            let last = row.iter().rposition(|&a| a != 0).unwrap();
            left = left.min(support.left() as usize + first);
            right = right.max(support.left() as usize + last);
            top = top.min(y);
            bottom = y;
        }
    }
    (right >= left && bottom >= top).then_some((left, top, right, bottom))
}

fn clear_source(mask: &mut Mask, bounds: (usize, usize, usize, usize)) {
    let (left, top, right, bottom) = bounds;
    let stride = mask.width() as usize;
    for y in top..=bottom {
        mask.data_mut()[y * stride + left..y * stride + right + 1].fill(0);
    }
}

pub(crate) fn blur_alpha_mask(mask: &mut Mask, sigma: f32) {
    blur_alpha_mask_with_bounds(
        mask,
        sigma,
        IntRect::from_xywh(0, 0, mask.width(), mask.height()),
    );
}

/// Source support is a known nonzero superset; None is a known empty mask.
/// Padding offsets belong to the caller's mask coordinates, not the canvas.
/// Thresholds, Gaussian factors, three-box plan and integer rounding retain
/// the established Skia implementation; only local buffer work is bounded.
pub(crate) fn blur_alpha_mask_with_bounds(
    mask: &mut Mask,
    sigma: f32,
    source_support: Option<IntRect>,
) {
    if sigma <= 0.0 || !sigma.is_finite() {
        return;
    }
    if sigma >= 2.0 {
        blur_alpha_mask_box(mask, sigma, source_support);
    } else if sigma >= 1.0 / 3.0 {
        blur_alpha_mask_small(mask, sigma, source_support);
    }
}

fn skia_small_gauss_factors(sigma: f32) -> Vec<u16> {
    let variance = f64::from(sigma).powi(2);
    let bessel0 = |t: f64| {
        let t_squared_over_four = t * t / 4.0;
        let mut sum = 1.0;
        let mut factor = 1.0;
        let mut k = 1;
        while factor > 1.0 / 1_000_000.0 {
            factor *= t_squared_over_four / f64::from(k * k);
            sum += factor;
            k += 1;
        }
        sum
    };
    let bessel1 = |t: f64| {
        let t_squared_over_four = t * t / 4.0;
        let mut sum = t / 2.0;
        let mut factor = sum;
        let mut k = 1;
        while factor > 1.0 / 1_000_000.0 {
            factor *= t_squared_over_four / f64::from(k * (k + 1));
            sum += factor;
            k += 1;
        }
        sum
    };
    let divisor = variance.exp();
    let mut bessel = [0.0_f64; 6];
    let mut factors = [0.0_f64; 6];
    bessel[0] = bessel0(variance);
    bessel[1] = bessel1(variance);
    factors[0] = bessel[0] / divisor;
    factors[1] = bessel[1] / divisor;
    let mut count = 1;
    while factors[count] > 0.01 {
        bessel[count + 1] = -(2.0 * count as f64 / variance) * bessel[count] + bessel[count - 1];
        factors[count + 1] = bessel[count + 1] / divisor;
        count += 1;
    }
    let mut sum = 0.0;
    for factor in factors[1..count].iter().rev() {
        sum += 2.0 * factor;
    }
    sum += factors[0];
    for factor in &mut factors[..count] {
        *factor /= sum;
    }
    let mut sum = 0.0;
    for factor in factors[1..count].iter().rev() {
        sum += 2.0 * factor;
    }
    factors[0] = 1.0 - sum;
    factors[..count]
        .iter()
        .map(|factor| (factor * 65536.0).round() as u16)
        .collect()
}

fn blur_alpha_mask_small(mask: &mut Mask, sigma: f32, support: Option<IntRect>) {
    let width = mask.width() as usize;
    let height = mask.height() as usize;
    let factors = skia_small_gauss_factors(sigma);
    let radius = factors.len() - 1;
    let Some((left, top, right, bottom)) = source_bounds(mask, support) else {
        return;
    };
    let src_width = right - left + 1;
    let src_height = bottom - top + 1;
    let blurred_height = src_height + 2 * radius;
    let blurred_width = src_width + 2 * radius;
    let mut vertical = vec![0_u8; src_width * blurred_height];
    let source = mask.data();
    for y in 0..blurred_height {
        for x in 0..src_width {
            let mut value = 128_u32;
            for offset in -(radius as isize)..=(radius as isize) {
                let source_y = y as isize - radius as isize + offset;
                if (0..src_height as isize).contains(&source_y) {
                    let alpha = u32::from(source[(top + source_y as usize) * width + left + x]);
                    let factor = u32::from(factors[offset.unsigned_abs()]);
                    value += ((alpha << 8) * factor) >> 16;
                }
            }
            vertical[y * src_width + x] = (value >> 8) as u8;
        }
    }
    clear_source(mask, (left, top, right, bottom));
    let output = mask.data_mut();
    for y in 0..blurred_height {
        let dest_y = top as isize + y as isize - radius as isize;
        if dest_y < 0 || dest_y >= height as isize {
            continue;
        }
        for x in 0..blurred_width {
            let dest_x = left as isize + x as isize - radius as isize;
            if dest_x < 0 || dest_x >= width as isize {
                continue;
            }
            let mut value = 128_u32;
            for offset in -(radius as isize)..=(radius as isize) {
                let source_x = x as isize - radius as isize + offset;
                if (0..src_width as isize).contains(&source_x) {
                    let alpha = u32::from(vertical[y * src_width + source_x as usize]);
                    let factor = u32::from(factors[offset.unsigned_abs()]);
                    value += ((alpha << 8) * factor) >> 16;
                }
            }
            output[dest_y as usize * width + dest_x as usize] = (value >> 8) as u8;
        }
    }
}

fn blur_alpha_mask_box(mask: &mut Mask, sigma: f32, support: Option<IntRect>) {
    let width = mask.width() as usize;
    let height = mask.height() as usize;
    let Some((left, top, right, bottom)) = source_bounds(mask, support) else {
        return;
    };
    let src_width = right - left + 1;
    let src_height = bottom - top + 1;
    let plan = PlanGauss::new(sigma.min(135.0));
    let blurred_width = src_width + 2 * plan.border;
    let blurred_height = src_height + 2 * plan.border;
    // Pad only independent SIMD lanes, never the logical source height.
    // The vertical pass still reads exactly src_height original rows.
    let horizontal_rows = if cfg!(all(
        feature = "simd",
        target_arch = "aarch64",
        target_endian = "little"
    )) {
        src_height.div_ceil(8) * 8
    } else {
        src_height
    };
    let mut horizontal = vec![0_u8; blurred_width * horizontal_rows];
    let mut scan = Scan::new(&plan);
    let source = mask.data();
    let mut y = 0;
    #[cfg(all(feature = "simd", target_arch = "aarch64", target_endian = "little"))]
    let mut vector_scan = neon_scan::Scan8::new(&plan);
    #[cfg(all(feature = "simd", target_arch = "aarch64", target_endian = "little"))]
    {
        // SkMaskBlurFilter transposes its horizontal output so the next scan
        // reads in memory order. Pack eight independent rows into contiguous
        // lanes with 8x8 transposes, preserving each row's original scan order.
        let mut packed_source = vec![0u8; src_width * 8];
        let mut packed_output = vec![0u8; blurred_width * 8];
        while y + 8 <= src_height {
            let start = (top + y) * width + left;
            unsafe {
                neon_scan::transpose_rows8(&source[start..], width, src_width, &mut packed_source);
                vector_scan.blur(&packed_source, 8, src_width, &mut packed_output);
                neon_scan::untranspose_rows8(
                    &packed_output,
                    blurred_width,
                    &mut horizontal[y * blurred_width..],
                    blurred_width,
                );
            }
            y += 8;
        }
        if y < src_height {
            // Standard SIMD tail lanes: missing rows are zero scratch, and
            // their independent output is never consumed by the vertical pass.
            let mut tail_source = vec![0u8; src_width * 8];
            for row in 0..src_height - y {
                let start = (top + y + row) * width + left;
                tail_source[row * src_width..(row + 1) * src_width]
                    .copy_from_slice(&source[start..start + src_width]);
            }
            unsafe {
                neon_scan::transpose_rows8(&tail_source, src_width, src_width, &mut packed_source);
                vector_scan.blur(&packed_source, 8, src_width, &mut packed_output);
                neon_scan::untranspose_rows8(
                    &packed_output,
                    blurred_width,
                    &mut horizontal[y * blurred_width..],
                    blurred_width,
                );
            }
            y = src_height;
        }
    }
    for y in y..src_height {
        let start = (top + y) * width + left;
        scan.blur(
            &source[start..start + src_width],
            &mut horizontal[y * blurred_width..(y + 1) * blurred_width],
        );
    }
    clear_source(mask, (left, top, right, bottom));
    let output = mask.data_mut();
    let mut column = vec![0_u8; src_height];
    let mut blurred = vec![0_u8; blurred_height];
    // The horizontal result still contains the complete filter expansion.
    // Independent vertical columns whose output lies beyond this mask need
    // no scan, and only their in-device row interval needs to be copied.
    let x_start = plan.border.saturating_sub(left);
    let x_end = blurred_width.min(width + plan.border - left);
    let y_start = plan.border.saturating_sub(top);
    let y_end = blurred_height.min(height + plan.border - top);
    let first_dest_y = top + y_start - plan.border;
    let mut x = x_start;
    #[cfg(all(feature = "simd", target_arch = "aarch64", target_endian = "little"))]
    {
        let mut vector_output = vec![0u8; blurred_height * 8];
        while x + 8 <= x_end {
            // Eight adjacent A8 columns are contiguous in each source row.
            // Each lane is an independent original PlanGauss::Scan.
            unsafe {
                vector_scan.blur(
                    &horizontal[x..],
                    blurred_width,
                    src_height,
                    &mut vector_output,
                );
            }
            let dest_x = left + x - plan.border;
            let dest = first_dest_y * width + dest_x;
            unsafe {
                neon_scan::copy_rows8(
                    &vector_output[y_start * 8..y_end * 8],
                    &mut output[dest..],
                    y_end - y_start,
                    width,
                );
            }
            x += 8;
        }
    }
    for x in x..x_end {
        for y in 0..src_height {
            column[y] = horizontal[y * blurred_width + x];
        }
        scan.blur(&column, &mut blurred);
        let dest_x = left + x - plan.border;
        for (row, &alpha) in blurred[y_start..y_end].iter().enumerate() {
            output[(first_dest_y + row) * width + dest_x] = alpha;
        }
    }
}

// SkBlurEngine.cpp::ThreeBoxApproxPass gangs the same three u32 integrators
// into vector lanes. This A8 backend retains SkMaskBlurFilter::PlanGauss's
// weight/rounding and two-ended Scan schedule instead of the engine's divider.
// It changes only the number of independent rows/columns processed together.
// The explicit byte/register packing is little-endian; other targets and
// builds without `simd` retain the scalar backend.
#[cfg(all(feature = "simd", target_arch = "aarch64", target_endian = "little"))]
mod neon_scan {
    use super::PlanGauss;

    // The three byte/halfword/word interleaves are an exact 8x8 transpose.
    // `src` and `dst` each admit eight rows of blocks*8 initialized/writable
    // bytes, with the supplied strides and complete-tile advances. Each
    // tile's loads precede its stores; wrappers check the whole extent and
    // use distinct allocations. No alignment greater than one is needed.
    #[inline(always)]
    unsafe fn transpose8_blocks(
        src: *const u8,
        src_stride: usize,
        dst: *mut u8,
        dst_stride: usize,
        blocks: usize,
        src_advance: usize,
        dst_advance: usize,
    ) {
        if blocks == 0 {
            return;
        }
        core::arch::asm!(
            "2:", "mov {read}, {src}", "mov {write}, {dst}",
            "ldr {a0:d}, [{read}]", "add {read}, {read}, {ss}",
            "ldr {a1:d}, [{read}]", "add {read}, {read}, {ss}",
            "ldr {a2:d}, [{read}]", "add {read}, {read}, {ss}",
            "ldr {a3:d}, [{read}]", "add {read}, {read}, {ss}",
            "ldr {a4:d}, [{read}]", "add {read}, {read}, {ss}",
            "ldr {a5:d}, [{read}]", "add {read}, {read}, {ss}",
            "ldr {a6:d}, [{read}]", "add {read}, {read}, {ss}",
            "ldr {a7:d}, [{read}]",
            "trn1 {b0:v}.8b, {a0:v}.8b, {a1:v}.8b",
            "trn2 {b1:v}.8b, {a0:v}.8b, {a1:v}.8b",
            "trn1 {b2:v}.8b, {a2:v}.8b, {a3:v}.8b",
            "trn2 {b3:v}.8b, {a2:v}.8b, {a3:v}.8b",
            "trn1 {b4:v}.8b, {a4:v}.8b, {a5:v}.8b",
            "trn2 {b5:v}.8b, {a4:v}.8b, {a5:v}.8b",
            "trn1 {b6:v}.8b, {a6:v}.8b, {a7:v}.8b",
            "trn2 {b7:v}.8b, {a6:v}.8b, {a7:v}.8b",
            "trn1 {a0:v}.4h, {b0:v}.4h, {b2:v}.4h",
            "trn1 {a1:v}.4h, {b1:v}.4h, {b3:v}.4h",
            "trn2 {a2:v}.4h, {b0:v}.4h, {b2:v}.4h",
            "trn2 {a3:v}.4h, {b1:v}.4h, {b3:v}.4h",
            "trn1 {a4:v}.4h, {b4:v}.4h, {b6:v}.4h",
            "trn1 {a5:v}.4h, {b5:v}.4h, {b7:v}.4h",
            "trn2 {a6:v}.4h, {b4:v}.4h, {b6:v}.4h",
            "trn2 {a7:v}.4h, {b5:v}.4h, {b7:v}.4h",
            "trn1 {b0:v}.2s, {a0:v}.2s, {a4:v}.2s",
            "trn1 {b1:v}.2s, {a1:v}.2s, {a5:v}.2s",
            "trn1 {b2:v}.2s, {a2:v}.2s, {a6:v}.2s",
            "trn1 {b3:v}.2s, {a3:v}.2s, {a7:v}.2s",
            "trn2 {b4:v}.2s, {a0:v}.2s, {a4:v}.2s",
            "trn2 {b5:v}.2s, {a1:v}.2s, {a5:v}.2s",
            "trn2 {b6:v}.2s, {a2:v}.2s, {a6:v}.2s",
            "trn2 {b7:v}.2s, {a3:v}.2s, {a7:v}.2s",
            "str {b0:d}, [{write}]", "add {write}, {write}, {ds}",
            "str {b1:d}, [{write}]", "add {write}, {write}, {ds}",
            "str {b2:d}, [{write}]", "add {write}, {write}, {ds}",
            "str {b3:d}, [{write}]", "add {write}, {write}, {ds}",
            "str {b4:d}, [{write}]", "add {write}, {write}, {ds}",
            "str {b5:d}, [{write}]", "add {write}, {write}, {ds}",
            "str {b6:d}, [{write}]", "add {write}, {write}, {ds}",
            "str {b7:d}, [{write}]",
            "add {src}, {src}, {sa}", "add {dst}, {dst}, {da}",
            "subs {blocks}, {blocks}, #1", "b.ne 2b",
            src = inout(reg) src => _, dst = inout(reg) dst => _,
            sa = in(reg) src_advance, da = in(reg) dst_advance,
            blocks = inout(reg) blocks => _, read = out(reg) _, write = out(reg) _,
            ss = in(reg) src_stride, ds = in(reg) dst_stride,
            a0 = out(vreg) _, a1 = out(vreg) _, a2 = out(vreg) _, a3 = out(vreg) _,
            a4 = out(vreg) _, a5 = out(vreg) _, a6 = out(vreg) _, a7 = out(vreg) _,
            b0 = out(vreg) _, b1 = out(vreg) _, b2 = out(vreg) _, b3 = out(vreg) _,
            b4 = out(vreg) _, b5 = out(vreg) _, b6 = out(vreg) _, b7 = out(vreg) _,
            options(nostack),
        );
    }

    pub(super) unsafe fn transpose_rows8(src: &[u8], stride: usize, width: usize, dst: &mut [u8]) {
        assert!(stride >= width && 7 * stride + width <= src.len());
        assert_eq!(dst.len(), width * 8);
        let complete = width / 8 * 8;
        // Admit all complete tiles once; retain row/block cursors in registers.
        transpose8_blocks(src.as_ptr(), stride, dst.as_mut_ptr(), 8, width / 8, 8, 64);
        for x in complete..width {
            for y in 0..8 {
                dst[x * 8 + y] = src[y * stride + x];
            }
        }
    }

    pub(super) unsafe fn untranspose_rows8(
        src: &[u8],
        width: usize,
        dst: &mut [u8],
        stride: usize,
    ) {
        assert_eq!(src.len(), width * 8);
        assert!(stride >= width && 7 * stride + width <= dst.len());
        let complete = width / 8 * 8;
        transpose8_blocks(src.as_ptr(), 8, dst.as_mut_ptr(), stride, width / 8, 64, 8);
        for x in complete..width {
            for y in 0..8 {
                dst[y * stride + x] = src[x * 8 + y];
            }
        }
    }

    // Scan8 output is contiguous; the destination consists of admitted
    // eight-byte row spans. Match SkMaskBlurFilter's dstStride scatter without
    // O0 iterator/slice calls per row. Only these eight bytes are written.
    // Last-row progression creates only an unused machine address marker;
    // all loads/stores satisfy the checked extent and no cursor escapes asm.
    pub(super) unsafe fn copy_rows8(src: &[u8], dst: &mut [u8], rows: usize, stride: usize) {
        if rows == 0 {
            return;
        }
        assert!(rows * 8 <= src.len());
        assert!(stride >= 8 && (rows - 1) * stride + 8 <= dst.len());
        core::arch::asm!(
            "2:",
            "ldr {value}, [{source}], #8",
            "str {value}, [{dest}]",
            "add {dest}, {dest}, {stride}",
            "subs {rows}, {rows}, #1",
            "b.ne 2b",
            source = inout(reg) src.as_ptr() => _,
            dest = inout(reg) dst.as_mut_ptr() => _,
            stride = in(reg) stride, rows = inout(reg) rows => _,
            value = out(reg) _, options(nostack),
        );
    }

    pub(super) struct Scan8 {
        weight: u32,
        border: usize,
        buffers: [Vec<[u32; 8]>; 3],
    }

    impl Scan8 {
        pub(super) fn new(plan: &PlanGauss) -> Self {
            // PlanGauss's box path starts at sigma >= 2 (window >= 4).
            // Its 32.32 weight therefore fits a u32. Window=1 stays scalar.
            assert!(plan.weight <= u64::from(u32::MAX));
            assert!(plan.pass_sizes.iter().all(|&size| size != 0));
            Self {
                weight: plan.weight as u32,
                border: plan.border,
                buffers: plan.pass_sizes.map(|size| vec![[0; 8]; size]),
            }
        }

        pub(super) unsafe fn blur(
            &mut self,
            src: &[u8],
            stride: usize,
            length: usize,
            dst: &mut [u8],
        ) {
            assert!(length != 0 && stride >= 8);
            assert!((length - 1) * stride + 8 <= src.len());
            assert_eq!(dst.len(), (length + 2 * self.border) * 8);
            let [ring0, ring1, ring2] = &mut self.buffers;
            let (n0, n1, n2) = (ring0.len(), ring1.len(), ring2.len());
            let (base0, base1, base2) = (
                ring0.as_mut_ptr().cast::<u32>(),
                ring1.as_mut_ptr().cast::<u32>(),
                ring2.as_mut_ptr().cast::<u32>(),
            );
            // SAFETY: the three fixed allocations have n*8 initialized u32s.
            // Nonempty cursors point to a complete eight-lane entry; advancing
            // reaches only one-past-end before resetting. The factory proves
            // all three rings nonempty. NEON permits unaligned accesses.
            // The source check admits exactly eight bytes per strided row.
            // Forward writes max(length, 2*border+1) complete entries; reverse
            // fills the remaining suffix using <= length source entries.
            // Every dereference stays inside its distinct original allocation.
            let (end0, end1, end2) = (base0.add(n0 * 8), base1.add(n1 * 8), base2.add(n2 * 8));
            let no_change = (2 * self.border + 1).saturating_sub(length);
            let reverse = dst.len() / 8 - length - no_change;
            // Keep the whole literal Scan, including cursors and six sums,
            // in registers. O0 cannot otherwise keep states live across a
            // Rust loop containing individual inline-asm advances.
            // Source/destination memory is admitted by the checks above.
            // Strided source progression may create an unused exterior
            // address marker after its final load; only the checked forward
            // and reverse entries are ever read, and no marker escapes asm.
            // As in official Scan, reverse keeps the rotated ring positions
            // after zeroing. No phases or integer operations are omitted.
            let dest_pointer = dst.as_mut_ptr();
            core::arch::asm!(
                "movi {tmp0:v}.4s, #0",
                "mov {tmpint}, {base0}",
                "2:",
                "stp {tmp0:q}, {tmp0:q}, [{tmpint}], #32",
                "cmp {tmpint}, {end0}",
                "b.lo 2b",
                "mov {tmpint}, {base1}",
                "3:",
                "stp {tmp0:q}, {tmp0:q}, [{tmpint}], #32",
                "cmp {tmpint}, {end1}",
                "b.lo 3b",
                "mov {tmpint}, {base2}",
                "4:",
                "stp {tmp0:q}, {tmp0:q}, [{tmpint}], #32",
                "cmp {tmpint}, {end2}",
                "b.lo 4b",
                "movi {s0a:v}.4s, #0",
                "movi {s0b:v}.4s, #0",
                "movi {s1a:v}.4s, #0",
                "movi {s1b:v}.4s, #0",
                "movi {s2a:v}.4s, #0",
                "movi {s2b:v}.4s, #0",
                "dup {weight:v}.4s, {weightvalue:w}",
                "mov {tmpint}, #0x80000000",
                "dup {half:v}.2d, {tmpint}",
                "mov {c0}, {base0}",
                "mov {c1}, {base1}",
                "mov {c2}, {base2}",
                "mov {count}, {length}",
                "5:",
                "ldr {leading_a:d}, [{source}]",
                "ushll {leading_a:v}.8h, {leading_a:v}.8b, #0",
                "ushll2 {leading_b:v}.4s, {leading_a:v}.8h, #0",
                "ushll {leading_a:v}.4s, {leading_a:v}.4h, #0",
                "add {s0a:v}.4s, {s0a:v}.4s, {leading_a:v}.4s",
                "add {s0b:v}.4s, {s0b:v}.4s, {leading_b:v}.4s",
                "add {s1a:v}.4s, {s1a:v}.4s, {s0a:v}.4s",
                "add {s1b:v}.4s, {s1b:v}.4s, {s0b:v}.4s",
                "add {s2a:v}.4s, {s2a:v}.4s, {s1a:v}.4s",
                "add {s2b:v}.4s, {s2b:v}.4s, {s1b:v}.4s",
                "umull {tmp0:v}.2d, {s2a:v}.2s, {weight:v}.2s",
                "umull2 {tmp1:v}.2d, {s2a:v}.4s, {weight:v}.4s",
                "add {tmp0:v}.2d, {tmp0:v}.2d, {half:v}.2d",
                "add {tmp1:v}.2d, {tmp1:v}.2d, {half:v}.2d",
                "shrn {tmp0:v}.2s, {tmp0:v}.2d, #32",
                "shrn2 {tmp0:v}.4s, {tmp1:v}.2d, #32",
                "umull {tmp1:v}.2d, {s2b:v}.2s, {weight:v}.2s",
                "umull2 {tmp2:v}.2d, {s2b:v}.4s, {weight:v}.4s",
                "add {tmp1:v}.2d, {tmp1:v}.2d, {half:v}.2d",
                "add {tmp2:v}.2d, {tmp2:v}.2d, {half:v}.2d",
                "shrn {tmp1:v}.2s, {tmp1:v}.2d, #32",
                "shrn2 {tmp1:v}.4s, {tmp2:v}.2d, #32",
                "xtn {tmp0:v}.4h, {tmp0:v}.4s",
                "xtn2 {tmp0:v}.8h, {tmp1:v}.4s",
                "xtn {tmp0:v}.8b, {tmp0:v}.8h",
                "str {tmp0:d}, [{output}]",
                "ldp {tmp1:q}, {tmp2:q}, [{c2}]",
                "sub {s2a:v}.4s, {s2a:v}.4s, {tmp1:v}.4s",
                "sub {s2b:v}.4s, {s2b:v}.4s, {tmp2:v}.4s",
                "stp {s1a:q}, {s1b:q}, [{c2}]",
                "ldp {tmp1:q}, {tmp2:q}, [{c1}]",
                "sub {s1a:v}.4s, {s1a:v}.4s, {tmp1:v}.4s",
                "sub {s1b:v}.4s, {s1b:v}.4s, {tmp2:v}.4s",
                "stp {s0a:q}, {s0b:q}, [{c1}]",
                "ldp {tmp1:q}, {tmp2:q}, [{c0}]",
                "sub {s0a:v}.4s, {s0a:v}.4s, {tmp1:v}.4s",
                "sub {s0b:v}.4s, {s0b:v}.4s, {tmp2:v}.4s",
                "stp {leading_a:q}, {leading_b:q}, [{c0}]",
                "add {c2}, {c2}, #32",
                "cmp {c2}, {end2}",
                "csel {c2}, {c2}, {base2}, lo",
                "add {c1}, {c1}, #32",
                "cmp {c1}, {end1}",
                "csel {c1}, {c1}, {base1}, lo",
                "add {c0}, {c0}, #32",
                "cmp {c0}, {end0}",
                "csel {c0}, {c0}, {base0}, lo",
                "add {source}, {source}, {stride}",
                "add {output}, {output}, #8",
                "subs {count}, {count}, #1",
                "b.ne 5b",
                "mov {count}, {nochange}",
                "cbz {count}, 7f",
                "6:",
                "movi {leading_a:v}.8b, #0",
                "ushll {leading_a:v}.8h, {leading_a:v}.8b, #0",
                "ushll2 {leading_b:v}.4s, {leading_a:v}.8h, #0",
                "ushll {leading_a:v}.4s, {leading_a:v}.4h, #0",
                "add {s0a:v}.4s, {s0a:v}.4s, {leading_a:v}.4s",
                "add {s0b:v}.4s, {s0b:v}.4s, {leading_b:v}.4s",
                "add {s1a:v}.4s, {s1a:v}.4s, {s0a:v}.4s",
                "add {s1b:v}.4s, {s1b:v}.4s, {s0b:v}.4s",
                "add {s2a:v}.4s, {s2a:v}.4s, {s1a:v}.4s",
                "add {s2b:v}.4s, {s2b:v}.4s, {s1b:v}.4s",
                "umull {tmp0:v}.2d, {s2a:v}.2s, {weight:v}.2s",
                "umull2 {tmp1:v}.2d, {s2a:v}.4s, {weight:v}.4s",
                "add {tmp0:v}.2d, {tmp0:v}.2d, {half:v}.2d",
                "add {tmp1:v}.2d, {tmp1:v}.2d, {half:v}.2d",
                "shrn {tmp0:v}.2s, {tmp0:v}.2d, #32",
                "shrn2 {tmp0:v}.4s, {tmp1:v}.2d, #32",
                "umull {tmp1:v}.2d, {s2b:v}.2s, {weight:v}.2s",
                "umull2 {tmp2:v}.2d, {s2b:v}.4s, {weight:v}.4s",
                "add {tmp1:v}.2d, {tmp1:v}.2d, {half:v}.2d",
                "add {tmp2:v}.2d, {tmp2:v}.2d, {half:v}.2d",
                "shrn {tmp1:v}.2s, {tmp1:v}.2d, #32",
                "shrn2 {tmp1:v}.4s, {tmp2:v}.2d, #32",
                "xtn {tmp0:v}.4h, {tmp0:v}.4s",
                "xtn2 {tmp0:v}.8h, {tmp1:v}.4s",
                "xtn {tmp0:v}.8b, {tmp0:v}.8h",
                "str {tmp0:d}, [{output}]",
                "ldp {tmp1:q}, {tmp2:q}, [{c2}]",
                "sub {s2a:v}.4s, {s2a:v}.4s, {tmp1:v}.4s",
                "sub {s2b:v}.4s, {s2b:v}.4s, {tmp2:v}.4s",
                "stp {s1a:q}, {s1b:q}, [{c2}]",
                "ldp {tmp1:q}, {tmp2:q}, [{c1}]",
                "sub {s1a:v}.4s, {s1a:v}.4s, {tmp1:v}.4s",
                "sub {s1b:v}.4s, {s1b:v}.4s, {tmp2:v}.4s",
                "stp {s0a:q}, {s0b:q}, [{c1}]",
                "ldp {tmp1:q}, {tmp2:q}, [{c0}]",
                "sub {s0a:v}.4s, {s0a:v}.4s, {tmp1:v}.4s",
                "sub {s0b:v}.4s, {s0b:v}.4s, {tmp2:v}.4s",
                "stp {leading_a:q}, {leading_b:q}, [{c0}]",
                "add {c2}, {c2}, #32",
                "cmp {c2}, {end2}",
                "csel {c2}, {c2}, {base2}, lo",
                "add {c1}, {c1}, #32",
                "cmp {c1}, {end1}",
                "csel {c1}, {c1}, {base1}, lo",
                "add {c0}, {c0}, #32",
                "cmp {c0}, {end0}",
                "csel {c0}, {c0}, {base0}, lo",
                "add {output}, {output}, #8",
                "subs {count}, {count}, #1",
                "b.ne 6b",
                "7:",
                "movi {tmp0:v}.4s, #0",
                "mov {tmpint}, {base0}",
                "8:",
                "stp {tmp0:q}, {tmp0:q}, [{tmpint}], #32",
                "cmp {tmpint}, {end0}",
                "b.lo 8b",
                "mov {tmpint}, {base1}",
                "9:",
                "stp {tmp0:q}, {tmp0:q}, [{tmpint}], #32",
                "cmp {tmpint}, {end1}",
                "b.lo 9b",
                "mov {tmpint}, {base2}",
                "12:",
                "stp {tmp0:q}, {tmp0:q}, [{tmpint}], #32",
                "cmp {tmpint}, {end2}",
                "b.lo 12b",
                "movi {s0a:v}.4s, #0",
                "movi {s0b:v}.4s, #0",
                "movi {s1a:v}.4s, #0",
                "movi {s1b:v}.4s, #0",
                "movi {s2a:v}.4s, #0",
                "movi {s2b:v}.4s, #0",
                "mov {count}, {reverse}",
                "cbz {count}, 14f",
                "mov {output}, {outputend}",
                "13:",
                "sub {source}, {source}, {stride}",
                "sub {output}, {output}, #8",
                "ldr {leading_a:d}, [{source}]",
                "ushll {leading_a:v}.8h, {leading_a:v}.8b, #0",
                "ushll2 {leading_b:v}.4s, {leading_a:v}.8h, #0",
                "ushll {leading_a:v}.4s, {leading_a:v}.4h, #0",
                "add {s0a:v}.4s, {s0a:v}.4s, {leading_a:v}.4s",
                "add {s0b:v}.4s, {s0b:v}.4s, {leading_b:v}.4s",
                "add {s1a:v}.4s, {s1a:v}.4s, {s0a:v}.4s",
                "add {s1b:v}.4s, {s1b:v}.4s, {s0b:v}.4s",
                "add {s2a:v}.4s, {s2a:v}.4s, {s1a:v}.4s",
                "add {s2b:v}.4s, {s2b:v}.4s, {s1b:v}.4s",
                "umull {tmp0:v}.2d, {s2a:v}.2s, {weight:v}.2s",
                "umull2 {tmp1:v}.2d, {s2a:v}.4s, {weight:v}.4s",
                "add {tmp0:v}.2d, {tmp0:v}.2d, {half:v}.2d",
                "add {tmp1:v}.2d, {tmp1:v}.2d, {half:v}.2d",
                "shrn {tmp0:v}.2s, {tmp0:v}.2d, #32",
                "shrn2 {tmp0:v}.4s, {tmp1:v}.2d, #32",
                "umull {tmp1:v}.2d, {s2b:v}.2s, {weight:v}.2s",
                "umull2 {tmp2:v}.2d, {s2b:v}.4s, {weight:v}.4s",
                "add {tmp1:v}.2d, {tmp1:v}.2d, {half:v}.2d",
                "add {tmp2:v}.2d, {tmp2:v}.2d, {half:v}.2d",
                "shrn {tmp1:v}.2s, {tmp1:v}.2d, #32",
                "shrn2 {tmp1:v}.4s, {tmp2:v}.2d, #32",
                "xtn {tmp0:v}.4h, {tmp0:v}.4s",
                "xtn2 {tmp0:v}.8h, {tmp1:v}.4s",
                "xtn {tmp0:v}.8b, {tmp0:v}.8h",
                "str {tmp0:d}, [{output}]",
                "ldp {tmp1:q}, {tmp2:q}, [{c2}]",
                "sub {s2a:v}.4s, {s2a:v}.4s, {tmp1:v}.4s",
                "sub {s2b:v}.4s, {s2b:v}.4s, {tmp2:v}.4s",
                "stp {s1a:q}, {s1b:q}, [{c2}]",
                "ldp {tmp1:q}, {tmp2:q}, [{c1}]",
                "sub {s1a:v}.4s, {s1a:v}.4s, {tmp1:v}.4s",
                "sub {s1b:v}.4s, {s1b:v}.4s, {tmp2:v}.4s",
                "stp {s0a:q}, {s0b:q}, [{c1}]",
                "ldp {tmp1:q}, {tmp2:q}, [{c0}]",
                "sub {s0a:v}.4s, {s0a:v}.4s, {tmp1:v}.4s",
                "sub {s0b:v}.4s, {s0b:v}.4s, {tmp2:v}.4s",
                "stp {leading_a:q}, {leading_b:q}, [{c0}]",
                "add {c2}, {c2}, #32",
                "cmp {c2}, {end2}",
                "csel {c2}, {c2}, {base2}, lo",
                "add {c1}, {c1}, #32",
                "cmp {c1}, {end1}",
                "csel {c1}, {c1}, {base1}, lo",
                "add {c0}, {c0}, #32",
                "cmp {c0}, {end0}",
                "csel {c0}, {c0}, {base0}, lo",
                "subs {count}, {count}, #1",
                "b.ne 13b",
                "14:",
                source = inout(reg) src.as_ptr() => _,
                output = inout(reg) dest_pointer => _,
                outputend = in(reg) dest_pointer.add(dst.len()),
                base0 = in(reg) base0, end0 = in(reg) end0,
                base1 = in(reg) base1, end1 = in(reg) end1,
                base2 = in(reg) base2, end2 = in(reg) end2,
                stride = in(reg) stride, length = in(reg) length,
                nochange = in(reg) no_change, reverse = in(reg) reverse,
                weightvalue = in(reg) self.weight,
                c0 = out(reg) _, c1 = out(reg) _, c2 = out(reg) _,
                count = out(reg) _, tmpint = out(reg) _,
                weight = out(vreg) _, half = out(vreg) _,
                s0a = out(vreg) _, s0b = out(vreg) _,
                s1a = out(vreg) _, s1b = out(vreg) _,
                s2a = out(vreg) _, s2b = out(vreg) _,
                leading_a = out(vreg) _, leading_b = out(vreg) _,
                tmp0 = out(vreg) _, tmp1 = out(vreg) _, tmp2 = out(vreg) _,
                options(nostack),
            );
        }
    }
}
#[cfg(test)]
mod legacy_oracle {
    use super::*;
    struct LegacyPlanGauss {
        pass_sizes: [usize; 3],
        border: usize,
        weight: u64,
    }

    impl LegacyPlanGauss {
        fn new(sigma: f32) -> Self {
            let possible_window =
                (f64::from(sigma) * 3.0 * (2.0 * std::f64::consts::PI).sqrt() / 4.0 + 0.5).floor()
                    as usize;
            let window = possible_window.max(1);
            let pass_sizes = [
                window - 1,
                window - 1,
                window - 1 + usize::from(window % 2 == 0),
            ];
            let border = if window % 2 == 1 {
                3 * ((window - 1) / 2)
            } else {
                3 * (window / 2) - 1
            };
            let divisor = if window % 2 == 1 {
                window.pow(3)
            } else {
                window.pow(3) + window.pow(2)
            };
            let weight = ((1_u64 << 32) as f64 / divisor as f64).round() as u64;
            Self {
                pass_sizes,
                border,
                weight,
            }
        }

        fn scan(&self, src: &[u8]) -> Vec<u8> {
            let mut dst = vec![0; src.len() + 2 * self.border];
            let mut buffers = [
                vec![0_u32; self.pass_sizes[0]],
                vec![0_u32; self.pass_sizes[1]],
                vec![0_u32; self.pass_sizes[2]],
            ];
            let mut cursors = [0_usize; 3];
            let mut sums = [0_u32; 3];
            let mut advance = |leading: u8| -> u8 {
                sums[0] += u32::from(leading);
                sums[1] += sums[0];
                sums[2] += sums[1];
                let result = ((self.weight * u64::from(sums[2]) + (1_u64 << 31)) >> 32) as u8;
                for level in (0..3).rev() {
                    if !buffers[level].is_empty() {
                        sums[level] -= buffers[level][cursors[level]];
                        buffers[level][cursors[level]] = if level == 0 {
                            u32::from(leading)
                        } else {
                            sums[level - 1]
                        };
                        cursors[level] = (cursors[level] + 1) % buffers[level].len();
                    }
                }
                result
            };
            for (index, &alpha) in src.iter().enumerate() {
                dst[index] = advance(alpha);
            }
            let sliding_window = 2 * self.border + 1;
            let no_change_count = sliding_window.saturating_sub(src.len());
            for index in src.len()..src.len() + no_change_count {
                dst[index] = advance(0);
            }
            drop(advance);
            buffers.iter_mut().for_each(|buffer| buffer.fill(0));
            cursors = [0; 3];
            sums = [0; 3];
            for (index, &alpha) in src.iter().rev().enumerate() {
                let leading = u32::from(alpha);
                sums[0] += leading;
                sums[1] += sums[0];
                sums[2] += sums[1];
                let output_index = dst.len() - 1 - index;
                if output_index < src.len() + no_change_count {
                    break;
                }
                dst[output_index] =
                    ((self.weight * u64::from(sums[2]) + (1_u64 << 31)) >> 32) as u8;
                for level in (0..3).rev() {
                    if !buffers[level].is_empty() {
                        sums[level] -= buffers[level][cursors[level]];
                        buffers[level][cursors[level]] =
                            if level == 0 { leading } else { sums[level - 1] };
                        cursors[level] = (cursors[level] + 1) % buffers[level].len();
                    }
                }
            }
            dst
        }
    }

    fn legacy_blur_alpha_mask(mask: &mut Mask, sigma: f32) {
        if sigma <= 0.0 || !sigma.is_finite() {
            return;
        }
        if sigma >= 2.0 {
            legacy_blur_alpha_mask_box(mask, sigma);
        } else if sigma >= 1.0 / 3.0 {
            legacy_blur_alpha_mask_small(mask, sigma);
        }
    }

    fn legacy_skia_small_gauss_factors(sigma: f32) -> Vec<u16> {
        let variance = f64::from(sigma).powi(2);
        let bessel0 = |t: f64| {
            let t_squared_over_four = t * t / 4.0;
            let mut sum = 1.0;
            let mut factor = 1.0;
            let mut k = 1;
            while factor > 1.0 / 1_000_000.0 {
                factor *= t_squared_over_four / f64::from(k * k);
                sum += factor;
                k += 1;
            }
            sum
        };
        let bessel1 = |t: f64| {
            let t_squared_over_four = t * t / 4.0;
            let mut sum = t / 2.0;
            let mut factor = sum;
            let mut k = 1;
            while factor > 1.0 / 1_000_000.0 {
                factor *= t_squared_over_four / f64::from(k * (k + 1));
                sum += factor;
                k += 1;
            }
            sum
        };
        let divisor = variance.exp();
        let mut bessel = [0.0_f64; 6];
        let mut factors = [0.0_f64; 6];
        bessel[0] = bessel0(variance);
        bessel[1] = bessel1(variance);
        factors[0] = bessel[0] / divisor;
        factors[1] = bessel[1] / divisor;
        let mut count = 1;
        while factors[count] > 0.01 {
            bessel[count + 1] =
                -(2.0 * count as f64 / variance) * bessel[count] + bessel[count - 1];
            factors[count + 1] = bessel[count + 1] / divisor;
            count += 1;
        }
        let mut sum = 0.0;
        for factor in factors[1..count].iter().rev() {
            sum += 2.0 * factor;
        }
        sum += factors[0];
        for factor in &mut factors[..count] {
            *factor /= sum;
        }
        let mut sum = 0.0;
        for factor in factors[1..count].iter().rev() {
            sum += 2.0 * factor;
        }
        factors[0] = 1.0 - sum;
        factors[..count]
            .iter()
            .map(|factor| (factor * 65536.0).round() as u16)
            .collect()
    }

    fn legacy_blur_alpha_mask_small(mask: &mut Mask, sigma: f32) {
        let width = mask.width() as usize;
        let height = mask.height() as usize;
        let factors = legacy_skia_small_gauss_factors(sigma);
        let radius = factors.len() - 1;
        let mut bounds = None::<(usize, usize, usize, usize)>;
        for (index, &alpha) in mask.data().iter().enumerate() {
            if alpha != 0 {
                let x = index % width;
                let y = index / width;
                bounds = Some(match bounds {
                    Some((left, top, right, bottom)) => {
                        (left.min(x), top.min(y), right.max(x), bottom.max(y))
                    }
                    None => (x, y, x, y),
                });
            }
        }
        let Some((left, top, right, bottom)) = bounds else {
            return;
        };
        let src_width = right - left + 1;
        let src_height = bottom - top + 1;
        let blurred_height = src_height + 2 * radius;
        let blurred_width = src_width + 2 * radius;
        let mut vertical = vec![0_u8; src_width * blurred_height];
        for y in 0..blurred_height {
            for x in 0..src_width {
                let mut value = 128_u32;
                for offset in -(radius as isize)..=(radius as isize) {
                    let source_y = y as isize - radius as isize + offset;
                    if (0..src_height as isize).contains(&source_y) {
                        let alpha =
                            u32::from(mask.data()[(top + source_y as usize) * width + left + x]);
                        let factor = u32::from(factors[offset.unsigned_abs()]);
                        value += ((alpha << 8) * factor) >> 16;
                    }
                }
                vertical[y * src_width + x] = (value >> 8) as u8;
            }
        }
        let mut output = vec![0_u8; width * height];
        for y in 0..blurred_height {
            let dest_y = top as isize + y as isize - radius as isize;
            if dest_y < 0 || dest_y >= height as isize {
                continue;
            }
            for x in 0..blurred_width {
                let dest_x = left as isize + x as isize - radius as isize;
                if dest_x < 0 || dest_x >= width as isize {
                    continue;
                }
                let mut value = 128_u32;
                for offset in -(radius as isize)..=(radius as isize) {
                    let source_x = x as isize - radius as isize + offset;
                    if (0..src_width as isize).contains(&source_x) {
                        let alpha = u32::from(vertical[y * src_width + source_x as usize]);
                        let factor = u32::from(factors[offset.unsigned_abs()]);
                        value += ((alpha << 8) * factor) >> 16;
                    }
                }
                output[dest_y as usize * width + dest_x as usize] = (value >> 8) as u8;
            }
        }
        mask.data_mut().copy_from_slice(&output);
    }

    fn legacy_blur_alpha_mask_box(mask: &mut Mask, sigma: f32) {
        let width = mask.width() as usize;
        let height = mask.height() as usize;
        let mut bounds = None::<(usize, usize, usize, usize)>;
        for (index, &alpha) in mask.data().iter().enumerate() {
            if alpha != 0 {
                let x = index % width;
                let y = index / width;
                bounds = Some(match bounds {
                    Some((left, top, right, bottom)) => {
                        (left.min(x), top.min(y), right.max(x), bottom.max(y))
                    }
                    None => (x, y, x, y),
                });
            }
        }
        let Some((left, top, right, bottom)) = bounds else {
            return;
        };
        let src_width = right - left + 1;
        let src_height = bottom - top + 1;
        let plan = LegacyPlanGauss::new(sigma.min(135.0));
        let blurred_width = src_width + 2 * plan.border;
        let blurred_height = src_height + 2 * plan.border;
        let mut horizontal = vec![0_u8; blurred_width * src_height];
        for y in 0..src_height {
            let start = (top + y) * width + left;
            let row = plan.scan(&mask.data()[start..start + src_width]);
            horizontal[y * blurred_width..(y + 1) * blurred_width].copy_from_slice(&row);
        }
        let mut output = vec![0_u8; width * height];
        let mut column = vec![0_u8; src_height];
        for x in 0..blurred_width {
            for y in 0..src_height {
                column[y] = horizontal[y * blurred_width + x];
            }
            let blurred = plan.scan(&column);
            let dest_x = left as isize + x as isize - plan.border as isize;
            if dest_x < 0 || dest_x >= width as isize {
                continue;
            }
            for (y, &alpha) in blurred.iter().enumerate().take(blurred_height) {
                let dest_y = top as isize + y as isize - plan.border as isize;
                if dest_y >= 0 && dest_y < height as isize {
                    output[dest_y as usize * width + dest_x as usize] = alpha;
                }
            }
        }
        mask.data_mut().copy_from_slice(&output);
    }

    #[test]
    fn scan_cursor_and_reused_scratch_match_old_gauss_arithmetic() {
        for sigma in [0.1, 0.5, 0.8, 1.2, 1.6, 2.0, 2.25, 3.5, 9.0, 20.0, 135.0] {
            let old = LegacyPlanGauss::new(sigma);
            let new = super::PlanGauss::new(sigma);
            let mut reusable = super::Scan::new(&new);
            for size in [1, 2, 3, 11, 17, 40, 255, 256, 520] {
                if sigma < 0.6 && size > 17 {
                    continue;
                }
                for pattern in 0..4 {
                    let src: Vec<_> = (0..size)
                        .map(|i| match pattern {
                            0 => 0,
                            1 => 255,
                            2 => {
                                if i % 2 == 0 {
                                    0
                                } else {
                                    255
                                }
                            }
                            _ => ((i * 37 + i * i * 13 + 71) % 256) as u8,
                        })
                        .collect();
                    let expected = old.scan(&src);
                    assert_eq!(new.scan(&src), expected);
                    let mut actual = vec![0xa5; expected.len()];
                    reusable.blur(&src, &mut actual);
                    assert_eq!(
                        actual, expected,
                        "sigma={sigma},size={size},pattern={pattern}"
                    );
                }
            }
        }
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64", target_endian = "little"))]
    #[test]
    fn eight_lane_scan_matches_independent_old_scans_with_stride_and_reused_rings() {
        for sigma in [0.8, 1.2, 1.6, 2.0, 2.25, 3.5, 9.0, 18.0, 20.0, 40.0, 135.0] {
            let old = LegacyPlanGauss::new(sigma);
            let plan = super::PlanGauss::new(sigma);
            let mut scan = super::neon_scan::Scan8::new(&plan);
            for length in [1, 2, 3, 11, 17, 40, 255, 256, 520] {
                for stride in [8, 13] {
                    for pattern in 0..4 {
                        let mut src = vec![0xa5; length * stride];
                        for y in 0..length {
                            for lane in 0..8 {
                                src[y * stride + lane] = match pattern {
                                    0 => 0,
                                    1 => 255,
                                    2 => {
                                        if (y + lane) % 2 == 0 {
                                            255
                                        } else {
                                            0
                                        }
                                    }
                                    _ => ((y * y * 13 + y * 37 + lane * 71 + 43) % 256) as u8,
                                };
                            }
                        }
                        let mut expected = vec![0u8; (length + 2 * plan.border) * 8];
                        for lane in 0..8 {
                            let column: Vec<_> =
                                (0..length).map(|y| src[y * stride + lane]).collect();
                            for (y, alpha) in old.scan(&column).into_iter().enumerate() {
                                expected[y * 8 + lane] = alpha;
                            }
                        }
                        let mut actual = vec![0xa5; expected.len()];
                        unsafe {
                            scan.blur(&src, stride, length, &mut actual);
                        }
                        assert_eq!(
                            actual, expected,
                            "sigma={sigma},length={length},stride={stride},pattern={pattern}"
                        );
                    }
                }
            }
        }
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64", target_endian = "little"))]
    #[test]
    fn eight_row_transpose_preserves_lane_order_tail_and_unaligned_strides() {
        for width in [1, 7, 8, 9, 15, 16, 17, 33, 1600] {
            let stride = width + 5;
            let mut source = vec![0xa5; 3 + stride * 8];
            for y in 0..8 {
                for x in 0..width {
                    source[3 + y * stride + x] = ((y * 71 + x * 37 + 43) % 256) as u8;
                }
            }
            let mut packed = vec![0xa5; width * 8];
            unsafe {
                super::neon_scan::transpose_rows8(&source[3..], stride, width, &mut packed);
            }
            for x in 0..width {
                for y in 0..8 {
                    assert_eq!(packed[x * 8 + y], source[3 + y * stride + x]);
                }
            }
            let mut restored = vec![0xa5; 5 + stride * 8];
            unsafe {
                super::neon_scan::untranspose_rows8(&packed, width, &mut restored[5..], stride);
            }
            assert!(restored[..5].iter().all(|&a| a == 0xa5));
            for y in 0..8 {
                assert_eq!(
                    &restored[5 + y * stride..5 + y * stride + width],
                    &source[3 + y * stride..3 + y * stride + width]
                );
                assert!(restored[5 + y * stride + width..5 + (y + 1) * stride]
                    .iter()
                    .all(|&a| a == 0xa5));
            }
        }
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64", target_endian = "little"))]
    #[test]
    fn strided_eight_byte_copy_preserves_unaligned_spans_and_padding() {
        for rows in [0, 1, 2, 3, 17, 198, 424] {
            for stride in [8, 9, 13, 520] {
                for prefix in [0, 1, 7] {
                    let src: Vec<_> = (0..rows * 8).map(|i| ((i * 37 + 43) % 256) as u8).collect();
                    let mut actual = vec![0xa5; prefix + rows * stride + 11];
                    let mut expected = actual.clone();
                    for row in 0..rows {
                        expected[prefix + row * stride..prefix + row * stride + 8]
                            .copy_from_slice(&src[row * 8..row * 8 + 8]);
                    }
                    unsafe {
                        super::neon_scan::copy_rows8(&src, &mut actual[prefix..], rows, stride);
                    }
                    assert_eq!(actual, expected);
                }
            }
        }
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64", target_endian = "little"))]
    #[test]
    fn padded_row_lanes_never_change_logical_blur_height_or_tail_pixels() {
        for height in 1..=15 {
            for width in [7, 8, 9, 17] {
                let support = IntRect::from_xywh(3, 2, width, height).unwrap();
                let mut input = Mask::new(96, 80).unwrap();
                for y in 2..2 + height {
                    for x in 3..3 + width {
                        input.data_mut()[(y * 96 + x) as usize] =
                            ((x * 37 + y * 71) % 255 + 1) as u8;
                    }
                }
                for sigma in [2.0, 18.0, 40.0] {
                    let mut expected = input.clone();
                    legacy_blur_alpha_mask(&mut expected, sigma);
                    let mut actual = input.clone();
                    super::blur_alpha_mask_with_bounds(&mut actual, sigma, Some(support));
                    assert_eq!(
                        actual.data(),
                        expected.data(),
                        "size={width}x{height},sigma={sigma}"
                    );
                }
            }
        }
    }

    #[test]
    fn bounded_shadow_blur_matches_old_dense_algorithm_at_edges_and_thresholds() {
        for (width, height) in [(7, 5), (61, 43), (520, 360)] {
            for (x, y, w, h) in [(0, 0, 1, 1), (3, 2, 13, 9), (width - 3, height - 2, 9, 5)] {
                let support = IntRect::from_xywh(x as i32, y as i32, w, h).unwrap();
                for pattern in 0..3 {
                    let mut input = Mask::new(width, height).unwrap();
                    for yy in y..(y + h).min(height) {
                        for xx in x..(x + w).min(width) {
                            input.data_mut()[(yy * width + xx) as usize] = match pattern {
                                0 => 255,
                                1 if (xx + yy) % 3 == 0 => 0,
                                _ => ((xx * 37 + yy * 13 + 71) % 256) as u8,
                            };
                        }
                    }
                    for sigma in [
                        0.0,
                        1.0 / 3.0 - 0.001,
                        1.0 / 3.0,
                        0.5,
                        1.0,
                        1.5,
                        1.99,
                        2.0,
                        2.5,
                        9.0,
                        20.0,
                        135.0,
                    ] {
                        let mut expected = input.clone();
                        legacy_blur_alpha_mask(&mut expected, sigma);
                        let mut bounded = input.clone();
                        super::blur_alpha_mask_with_bounds(&mut bounded, sigma, Some(support));
                        assert_eq!(
                            bounded.data(),
                            expected.data(),
                            "size={width}x{height},sigma={sigma},pattern={pattern},xy={x},{y}"
                        );
                        let mut whole = input.clone();
                        super::blur_alpha_mask(&mut whole, sigma);
                        assert_eq!(whole.data(), expected.data());
                    }
                }
            }
        }
        let mut empty = Mask::new(7, 5).unwrap();
        super::blur_alpha_mask_with_bounds(&mut empty, 20.0, None);
        assert_eq!(empty.data(), &[0; 35]);
    }

    #[test]
    #[ignore = "reproducible debug large-shadow blur benchmark"]
    fn bounded_shadow_debug_timing() {
        let (width, height) = (2560, 1542);
        let support = IntRect::from_xywh(281, 311, 1215, 79).unwrap();
        let mut input = Mask::new(width, height).unwrap();
        for y in support.top()..support.bottom() {
            input.data_mut()[y as usize * width as usize + support.left() as usize
                ..y as usize * width as usize + support.right() as usize]
                .fill(255);
        }
        for sigma in [9.0, 20.0] {
            let mut expected = input.clone();
            let before = std::time::Instant::now();
            legacy_blur_alpha_mask(&mut expected, sigma);
            let old_time = before.elapsed();
            let mut new = input.clone();
            let now = std::time::Instant::now();
            super::blur_alpha_mask_with_bounds(&mut new, sigma, Some(support));
            let new_time = now.elapsed();
            assert_eq!(new.data(), expected.data());
            eprintln!(
                "debug shadow sigma={sigma}: old {:.3}ms, bounded/reused Scan {:.3}ms, {:.1}x",
                old_time.as_secs_f64() * 1000.0,
                new_time.as_secs_f64() * 1000.0,
                old_time.as_secs_f64() / new_time.as_secs_f64()
            );
        }
    }

    #[test]
    #[ignore = "reproducible debug high-DPI shadow pointer benchmark"]
    fn highdpi_shadow_pointer_debug_timing() {
        let (width, height) = (2048, 1536);
        let support = IntRect::from_xywh(224, 370, 1600, 198).unwrap();
        let mut input = Mask::new(width, height).unwrap();
        for y in support.top()..support.bottom() {
            input.data_mut()[y as usize * width as usize + support.left() as usize
                ..y as usize * width as usize + support.right() as usize]
                .fill(255);
        }
        for sigma in [18.0, 40.0] {
            let mut expected = input.clone();
            legacy_blur_alpha_mask(&mut expected, sigma);
            let mut samples = Vec::new();
            let repetitions = std::env::var("SKIA_BLUR_BENCH_REPEAT")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(7)
                .max(1);
            for _ in 0..repetitions {
                let mut actual = input.clone();
                let start = std::time::Instant::now();
                super::blur_alpha_mask_with_bounds(&mut actual, sigma, Some(support));
                samples.push(start.elapsed().as_secs_f64() * 1000.0);
                assert_eq!(actual.data(), expected.data());
                std::hint::black_box(&actual);
            }
            samples.sort_by(f64::total_cmp);
            eprintln!("highdpi shadow device={width}x{height},source=1600x198,sigma={sigma}: min={:.3} median={:.3} max={:.3}ms", samples[0], samples[repetitions / 2], samples[repetitions - 1]);
        }
    }
}
