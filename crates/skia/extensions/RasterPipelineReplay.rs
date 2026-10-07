//! Local replay adapter combining Skia stages for browser gradient commands.
//! Tile scheduling and PaintShader input have no exact upstream Skia symbol.

// Selected SkRasterPipeline stages used by the software screenshot profile.
use crate::compat::commands::PaintRect;
use crate::compat::commands::{PaintShader, PaintSpreadMethod};
use crate::raster::{Mask, Pixmap, Transform};

// Screenshot replay uses 256px raster tiles with one-pixel gutters. Skia's
// dither stage reads tile-local device coordinates, not shader coordinates.
pub(crate) fn tile_shift(position: u32) -> u32 {
    if position < 255 {
        0
    } else {
        254 * (1 + (position - 255) / 254)
    }
}

// A bounded layer's child device retains the enclosing screenshot's tile
// origin. Translate the outer 255px schedule into child coordinates, then clip
// it to this child's own device origin, just like seed_shader's raster bounds.
#[inline]
fn gradient_tile_shift(position: u32, origin: i32, device_origin: u32) -> u32 {
    let global = (position as i64 + origin as i64).clamp(0, u32::MAX as i64) as u32;
    (tile_shift(global) as i64 - origin as i64)
        .max(device_origin as i64)
        .clamp(0, u32::MAX as i64) as u32
}

// SkRasterPipeline_opts.h::dither: the 8x8 ordered matrix, in fcebda order.
fn dither(x: u32, y: u32) -> f32 {
    let y = y ^ x;
    let m = ((y & 1) << 5)
        | ((x & 1) << 4)
        | ((y & 2) << 2)
        | ((x & 2) << 1)
        | ((y & 4) >> 1)
        | ((x & 4) >> 2);
    m as f32 * (2.0 / 128.0) - 63.0 / 128.0
}

/// Skia gradient stop lookup, interpolation, N32 packing, and device dithering
/// under scale+translate. Unsupported transforms use the existing renderer.
pub(crate) fn draw_linear_gradient(
    pixmap: &mut Pixmap,
    clip: Option<&Mask>,
    canvas: Transform,
    shader: &PaintShader,
    destination: PaintRect,
    tile: PaintRect,
    device_origin: (u32, u32),
    tile_origin: (i32, i32),
    direct: bool,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
) -> bool {
    draw_linear_gradient_impl(
        pixmap,
        clip,
        canvas,
        shader,
        destination,
        tile,
        device_origin,
        tile_origin,
        direct,
        clip_summary,
        true,
    )
}

fn draw_linear_gradient_impl(
    pixmap: &mut Pixmap,
    clip: Option<&Mask>,
    canvas: Transform,
    shader: &PaintShader,
    destination: PaintRect,
    tile: PaintRect,
    device_origin: (u32, u32),
    tile_origin: (i32, i32),
    direct: bool,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    use_segment_spans: bool,
) -> bool {
    draw_linear_gradient_output(
        pixmap,
        clip,
        canvas,
        shader,
        destination,
        tile,
        device_origin,
        tile_origin,
        direct,
        clip_summary,
        use_segment_spans,
        None,
    )
}

/// The same prepared gradient stages with an A8 destination. The caller owns
/// a single Alpha/Add mask on a transparent device, so RGB cannot contribute
/// to the result. Dither changes RGB alone (SkRasterPipeline_opts.h::dither).
pub(crate) fn draw_linear_gradient_alpha(
    pixmap: &mut Pixmap,
    clip: Option<&Mask>,
    canvas: Transform,
    shader: &PaintShader,
    destination: PaintRect,
    tile: PaintRect,
    device_origin: (u32, u32),
    tile_origin: (i32, i32),
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    alpha: &mut [u8],
) -> bool {
    if pixmap.format != crate::PixelFormat::Rgba8888
        || alpha.len() != pixmap.width() as usize * pixmap.height() as usize
    {
        return false;
    }
    draw_linear_gradient_output(
        pixmap,
        clip,
        canvas,
        shader,
        destination,
        tile,
        device_origin,
        tile_origin,
        false,
        clip_summary,
        true,
        Some(alpha),
    )
}

fn draw_linear_gradient_output(
    pixmap: &mut Pixmap,
    clip: Option<&Mask>,
    canvas: Transform,
    shader: &PaintShader,
    destination: PaintRect,
    tile: PaintRect,
    device_origin: (u32, u32),
    tile_origin: (i32, i32),
    direct: bool,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    use_segment_spans: bool,
    alpha_output: Option<&mut [u8]>,
) -> bool {
    let matrix = shader.transform.values;
    if canvas.sx <= 0.0
        || canvas.sy <= 0.0
        || ![canvas.sx, canvas.sy, canvas.tx, canvas.ty]
            .into_iter()
            .all(f32::is_finite)
        || canvas.kx != 0.0
        || canvas.ky != 0.0
        || matrix[0] != 1.0
        || matrix[5] != 1.0
        || matrix[1] != 0.0
        || matrix[4] != 0.0
        || matrix[3] != 0.0
        || matrix[7] != 0.0
        || matrix[15] != 1.0
        || shader.spread != PaintSpreadMethod::kPad
        || shader.stops.len() < 2
        || shader.stops.iter().any(|s| {
            [s.color.red, s.color.green, s.color.blue, s.color.alpha]
                .iter()
                .any(|c| !c.is_finite() || !(0.0..=1.0).contains(c))
        })
    {
        return false;
    }
    let gap_count = (shader.stops.len() - 1) as f32;
    if shader
        .stops
        .iter()
        .any(|s| !s.offset.is_finite() || !(0.0..=1.0).contains(&s.offset))
        || shader.stops.windows(2).any(|s| s[0].offset > s[1].offset)
    {
        return false;
    }
    let evenly_spaced = shader
        .stops
        .iter()
        .enumerate()
        .all(|(i, stop)| stop.offset as f32 == i as f32 / gap_count);
    let left = destination.x as f32 * canvas.sx + canvas.tx;
    let top = destination.y as f32 * canvas.sy + canvas.ty;
    let right = (destination.x as f32 + destination.width as f32) * canvas.sx + canvas.tx;
    let bottom = (destination.y as f32 + destination.height as f32) * canvas.sy + canvas.ty;
    if [left, top, right, bottom]
        .iter()
        .any(|v| !v.is_finite() || v.fract() != 0.0)
    {
        return false;
    }

    // SkLinearGradient.cpp::pts_to_unit_matrix, followed by the inverse local
    // translation. Preserve the reference's f32 normalization and stage order.
    let source_x = if direct { 0.0 } else { tile.x };
    let source_y = if direct { 0.0 } else { tile.y };
    let px = (shader.start.x - source_x) as f32;
    let py = (shader.start.y - source_y) as f32;
    let dx = (shader.end.x - source_x) as f32 - px;
    let dy = (shader.end.y - source_y) as f32 - py;
    let length = (dx * dx + dy * dy).sqrt();
    if !length.is_finite() || length <= 1.0 / 32768.0 {
        return false;
    }
    let inv = 1.0 / length;
    let nx = dx * inv;
    let ny = dy * inv;
    let a = nx * inv / canvas.sx;
    let b = ny * inv / canvas.sy;
    let base = ((-ny * py + (1.0 - nx) * px) - px) * inv;
    let tx = (matrix[12] + if direct { 0.0 } else { destination.x }) as f32 * canvas.sx + canvas.tx;
    let ty = (matrix[13] + if direct { 0.0 } else { destination.y }) as f32 * canvas.sy + canvas.ty;
    // SkGradientBaseShader.cpp::init_stop_evenly and add_const_color.
    let color = |color: crate::compat::commands::Color| {
        let alpha = color.alpha;
        let scale = if shader.interpolate_premultiplied {
            alpha
        } else {
            1.0
        };
        [
            color.red * scale,
            color.green * scale,
            color.blue * scale,
            alpha,
        ]
    };
    let mut coefficients = Vec::new();
    if !evenly_spaced {
        coefficients.push((f32::NEG_INFINITY, [0.0; 4], color(shader.stops[0].color)));
    }
    for (i, stop) in shader.stops.iter().enumerate() {
        let bias = color(stop.color);
        let start = stop.offset as f32;
        if let Some(next) = shader.stops.get(i + 1) {
            let gap = next.offset as f32 - start;
            if gap == 0.0 {
                continue;
            }
            let reciprocal = if evenly_spaced { gap_count } else { 1.0 / gap };
            let right = color(next.color);
            let factor = std::array::from_fn::<_, 4, _>(|c| (right[c] - bias[c]) * reciprocal);
            let intercept = std::array::from_fn::<_, 4, _>(|c| bias[c] - factor[c] * start);
            coefficients.push((start, factor, intercept));
        } else {
            coefficients.push((start, [0.0; 4], bias));
        }
    }
    let opaque = shader.stops.iter().all(|s| s.color.alpha == 1.0);
    let format = pixmap.format;
    // SkRasterClip bounds constrain the shader blitter. Rectangular BW clips
    // have unit coverage inside those bounds, so no dense coverage scan is
    // needed per span. Other masks retain the existing exact coverage path.
    let mut first_x = left.max(0.0) as u32;
    let mut end_x = right.min(pixmap.width() as f32).max(0.0) as u32;
    let mut first_y = top.max(0.0) as u32;
    let mut end_y = bottom.min(pixmap.height() as f32).max(0.0) as u32;
    let mut rectangular_clip = false;
    if let Some(summary) = clip_summary {
        let Some(bounds) = summary.getBounds() else {
            return true;
        };
        first_x = first_x.max(bounds.left() as u32);
        end_x = end_x.min(bounds.right() as u32);
        first_y = first_y.max(bounds.top() as u32);
        end_y = end_y.min(bounds.bottom() as u32);
        rectangular_clip = summary.isRect();
    }
    if let Some(alpha_output) = alpha_output {
        // Keep seed_shader's complete global tile schedule and the original
        // matrix_2x3 FMA ordering, including tile-boundary-dependent f32 bits.
        // Only a BW clip and exactly zero x slope admit constant-alpha spans;
        // all other coverage remains per pixel with the original mask scale.
        for y in first_y..end_y {
            let shift_y = gradient_tile_shift(y, tile_origin.1, device_origin.1);
            let mut x = first_x;
            while x < end_x {
                let shift_x = gradient_tile_shift(x, tile_origin.0, device_origin.0);
                let matrix_bias = a * (shift_x as f32 - tx) + b * (shift_y as f32 - ty) + base;
                let row_bias = (y as f32 - shift_y as f32 + 0.5).mul_add(b, matrix_bias);
                let t = (x as f32 - shift_x as f32 + 0.5)
                    .mul_add(a, row_bias)
                    .clamp(0.0, 1.0);
                let segment = if evenly_spaced {
                    ((t * gap_count) as usize).min(coefficients.len() - 1)
                } else {
                    coefficients
                        .partition_point(|(start, _, _)| *start <= t)
                        .saturating_sub(1)
                };
                let (_, factor, bias) = coefficients[segment];
                let alpha = t.mul_add(factor[3], bias[3]).clamp(0.0, 1.0);
                let coverage = if rectangular_clip {
                    255
                } else {
                    clip.map_or(255, |mask| mask.alpha_at(x, y))
                };
                let scale = coverage as f32 * (1.0 / 255.0);
                let covered_alpha = alpha * scale;
                let value = 0.0f32.mul_add(1.0 - covered_alpha, covered_alpha);
                let packed = (value * 255.0).round_ties_even().clamp(0.0, 255.0) as u8;
                let end = if rectangular_clip
                    && a == 0.0
                    && row_bias.is_finite()
                    && x >= shift_x
                    && y >= shift_y
                {
                    // Positive seed coordinates multiplied by exact +/-0 have
                    // the same signed-zero contribution throughout this tile.
                    let global_x = x as i64 + tile_origin.0 as i64;
                    let boundary = if global_x < 255 {
                        255
                    } else {
                        tile_shift(global_x as u32) as i64 + 255
                    };
                    end_x.min(
                        (boundary - tile_origin.0 as i64).clamp(x as i64 + 1, u32::MAX as i64)
                            as u32,
                    )
                } else {
                    x + 1
                };
                let start = y as usize * pixmap.width() as usize;
                alpha_output[start + x as usize..start + end as usize].fill(packed);
                x = end;
            }
        }
        return true;
    }
    #[cfg(target_arch = "aarch64")]
    let span_context = (evenly_spaced && shader.stops.len() == 2).then(|| {
        let mut context = TwoStopSpanContext::new(a, coefficients[0].1, coefficients[0].2);
        context.format = format;
        context
    });
    // Upstream gradient_lookup selects a factor/bias tuple per lane and
    // evaluates mad(t,factor,bias), exactly like the two-stop arithmetic after
    // lookup. Adjacent lanes inside one stop interval share that tuple. This
    // local driver resolves runs once, then executes those same vector stages;
    // it never substitutes a two-stop factor for a multi-stop factor.
    #[cfg(target_arch = "aarch64")]
    let segment_contexts =
        (use_segment_spans && cfg!(feature = "simd") && a != 0.0 && span_context.is_none()).then(
            || {
                coefficients
                    .iter()
                    .map(|(_, factor, bias)| {
                        let mut context = TwoStopSpanContext::new(a, *factor, *bias);
                        context.format = format;
                        context
                    })
                    .collect::<Vec<_>>()
            },
        );
    for y in first_y..end_y {
        #[cfg(target_arch = "aarch64")]
        let mut vertical_row = None;
        let shift_y = gradient_tile_shift(y, tile_origin.1, device_origin.1);
        #[cfg(target_arch = "aarch64")]
        let row_context = if a == 0.0 && span_context.is_none() {
            // General SkGradientBaseShader interpolation still uses the same
            // factor/bias stages. In a vertical gradient the stop segment is
            // constant across the entire row, so select it once, then run the
            // existing vector stages with its coefficients.
            let shift_x = gradient_tile_shift(first_x, tile_origin.0, device_origin.0);
            let bias = a * (shift_x as f32 - tx) + b * (shift_y as f32 - ty) + base;
            let t = (first_x as f32 - shift_x as f32 + 0.5)
                .mul_add(a, (y as f32 - shift_y as f32 + 0.5).mul_add(b, bias))
                .clamp(0.0, 1.0);
            let segment = if evenly_spaced {
                ((t * gap_count) as usize).min(coefficients.len() - 1)
            } else {
                coefficients
                    .partition_point(|(start, _, _)| *start <= t)
                    .saturating_sub(1)
            };
            let mut context =
                TwoStopSpanContext::new(a, coefficients[segment].1, coefficients[segment].2);
            context.format = format;
            Some(context)
        } else {
            None
        };
        #[cfg(target_arch = "aarch64")]
        let active_context = span_context.as_ref().or(row_context.as_ref());
        #[cfg(target_arch = "aarch64")]
        if rectangular_clip
            && opaque
            && a == 0.0
            && device_origin.0 == 0
            && tile_origin == (0, 0)
            && end_x.saturating_sub(first_x) >= 510
        {
            if let Some(context) = active_context {
                let row_bias = |shift_x: u32| {
                    let matrix_bias = a * (shift_x as f32 - tx) + b * (shift_y as f32 - ty) + base;
                    (y as f32 - shift_y as f32 + 0.5).mul_add(b, matrix_bias)
                };
                let bias = row_bias(tile_shift(first_x));
                // For zero x slope, only the sign of zero can differ across
                // tile translations. Equal finite endpoint bits establish the
                // same stage input for every intervening tile (including -0).
                if bias.is_finite() && bias.to_bits() == row_bias(tile_shift(end_x - 1)).to_bits() {
                    let start = (y as usize * pixmap.width() as usize + first_x as usize) * 4;
                    let bytes = (end_x - first_x) as usize * 4;
                    unsafe {
                        shade_vertical_tiled_row(
                            &mut pixmap.data_mut()[start..start + bytes],
                            first_x,
                            y - shift_y,
                            bias,
                            context,
                            shader.interpolate_premultiplied,
                        );
                    }
                    continue;
                }
            }
        }
        let mut x = first_x;
        let end = end_x;
        while x < end {
            let index = (y as usize * pixmap.width() as usize + x as usize) * 4;
            let coverage = if rectangular_clip {
                255
            } else {
                clip.map_or(255, |m| m.alpha_at(x as u32, y as u32))
            };
            if coverage == 0 {
                x += 1;
                continue;
            }
            let shift_x = gradient_tile_shift(x, tile_origin.0, device_origin.0);
            let matrix_bias = a * (shift_x as f32 - tx) + b * (shift_y as f32 - ty) + base;
            #[cfg(target_arch = "aarch64")]
            if (active_context.is_some() || segment_contexts.is_some())
                && coverage == 255
                && x >= shift_x
                && y >= shift_y
            {
                // Skia has a dedicated evenly_spaced_2_stop_gradient stage.
                // Emit its NEON lanes only within one tile and one fully
                // covered clip run; partial coverage retains the scalar path.
                let global_x = x as i64 + tile_origin.0 as i64;
                let global_boundary = if global_x < 255 {
                    255
                } else {
                    tile_shift(global_x as u32) as i64 + 255
                };
                let boundary =
                    (global_boundary - tile_origin.0 as i64).clamp(0, u32::MAX as i64) as u32;
                let mut span_end = end.min(boundary);
                if let Some(mask) = clip.filter(|_| !rectangular_clip) {
                    // Bounded A8 mask adapter to blitAntiH's constant-coverage
                    // spans. Share the existing exact wordwise run scan rather
                    // than invoke an iterator closure for every covered pixel.
                    let row = mask.row_range(y, x, span_end);
                    span_end = x + crate::cpu::analytic_aa::equal_byte_run_end(row, 0, 255) as u32;
                }
                let row_bias = (y as f32 - shift_y as f32 + 0.5).mul_add(b, matrix_bias);
                // Stop lookup is monotone within a fixed tile's finite matrix
                // stage. Find the first pixel outside this interval using the
                // exact original FMA expression, including >= at hard stops.
                // Solving the inverse matrix analytically would move boundaries
                // by a pixel when a float rounds at a stop, so do not do that.
                let context = if let Some(context) = active_context {
                    context
                } else if row_bias.is_finite() {
                    let t_at = |pixel: u32| {
                        (pixel as f32 - shift_x as f32 + 0.5)
                            .mul_add(a, row_bias)
                            .clamp(0.0, 1.0)
                    };
                    let segment_at = |pixel| {
                        let t = t_at(pixel);
                        if evenly_spaced {
                            ((t * gap_count) as usize).min(coefficients.len() - 1)
                        } else {
                            coefficients
                                .partition_point(|(start, _, _)| *start <= t)
                                .saturating_sub(1)
                        }
                    };
                    let segment = segment_at(x);
                    let mut lo = x + 1;
                    // Finite monotone matrix+stop lookup: equal endpoint
                    // segments prove this whole interval shares coefficients.
                    // The exact FMA test avoids an unnecessary binary search
                    // for the common case with no stop inside this clip run.
                    let mut hi = span_end;
                    if span_end > x && segment_at(span_end - 1) == segment {
                        lo = span_end;
                    }
                    while lo < hi {
                        let mid = lo + (hi - lo) / 2;
                        if segment_at(mid) == segment {
                            lo = mid + 1;
                        } else {
                            hi = mid;
                        }
                    }
                    span_end = lo;
                    &segment_contexts.as_ref().unwrap()[segment]
                } else {
                    // Invalid matrix inputs retain the prior scalar behavior.
                    span_end = x;
                    &segment_contexts.as_ref().unwrap()[0]
                };
                // start_pipeline patches the remaining lanes to scratch and
                // executes the same stages before copying back only active
                // lanes. Opaque spans need no load_dst; vertical spans already
                // have these tail values in their exact eight-pixel period.
                let pixels = if opaque && span_end.saturating_sub(shift_x) <= u32::MAX - 3 {
                    (span_end - x) as usize
                } else {
                    ((span_end - x) / 4 * 4) as usize
                };
                if pixels != 0 {
                    // AArch64 guarantees NEON. The slice holds exactly the
                    // complete four-pixel groups read/written by this kernel.
                    unsafe {
                        let dst = &mut pixmap.data_mut()[index..index + pixels * 4];
                        if opaque && context.constant_in_x {
                            shade_vertical_row_span(
                                dst,
                                x - shift_x,
                                y - shift_y,
                                row_bias,
                                context,
                                shader.interpolate_premultiplied,
                                &mut vertical_row,
                                true,
                            );
                        } else if opaque {
                            shade_opaque_span_with_tail(
                                dst,
                                x - shift_x,
                                y - shift_y,
                                row_bias,
                                context,
                            );
                        } else {
                            shade_two_stop_span_kernel::<false>(
                                dst,
                                x - shift_x,
                                y - shift_y,
                                row_bias,
                                context,
                                shader.interpolate_premultiplied,
                            );
                        }
                    }
                    x += pixels as u32;
                    continue;
                }
            }
            let t = (x as f32 - shift_x as f32 + 0.5)
                .mul_add(a, (y as f32 - shift_y as f32 + 0.5).mul_add(b, matrix_bias))
                .clamp(0.0, 1.0);
            let segment = if evenly_spaced {
                ((t * gap_count) as usize).min(coefficients.len() - 1)
            } else {
                coefficients
                    .partition_point(|(start, _, _)| *start <= t)
                    .saturating_sub(1)
            };
            let (_, factor, bias) = coefficients[segment];
            let noise = dither(
                (x as i64 - shift_x as i64) as u32,
                (y as i64 - shift_y as i64) as u32,
            );
            let scale = coverage as f32 * (1.0 / 255.0);
            let dst = &mut pixmap.data_mut()[index..index + 4];
            let alpha = t.mul_add(factor[3], bias[3]).clamp(0.0, 1.0);
            let covered_alpha = alpha * scale;
            for channel in 0..3 {
                let value = t.mul_add(factor[channel], bias[channel]);
                let premul = if shader.interpolate_premultiplied {
                    value
                } else {
                    value * alpha
                };
                let source = noise.mul_add(1.0 / 255.0, premul).clamp(0.0, alpha) * scale;
                // Dithered shaders use load_dst + src-over + store stages;
                // the fused byte-valued src-over shortcut excludes dithering.
                let value = (dst[format.channel(channel)] as f32 * (1.0 / 255.0))
                    .mul_add(1.0 - covered_alpha, source);
                dst[format.channel(channel)] =
                    (value * 255.0).round_ties_even().clamp(0.0, 255.0) as u8;
            }
            let value = (dst[3] as f32 * (1.0 / 255.0)).mul_add(1.0 - covered_alpha, covered_alpha);
            dst[3] = if format == crate::PixelFormat::Bgrx8888 {
                0
            } else {
                (value * 255.0).round_ties_even().clamp(0.0, 255.0) as u8
            };
            x += 1;
        }
    }
    true
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn pack_gradient_channel<const OPAQUE: bool>(
    value: std::arch::aarch64::float32x4_t,
    zero: std::arch::aarch64::float32x4_t,
    scale: std::arch::aarch64::float32x4_t,
) -> std::arch::aarch64::uint32x4_t {
    use std::arch::aarch64::*;
    if OPAQUE {
        // dither has already clamped these finite opaque channels to [0,1].
        // Multiplication by positive 255 preserves [0,255], so store_8888's
        // repeated clamp is an identity. Retain its nearest-even conversion.
        vcvtnq_u32_f32(vmulq_f32(value, scale))
    } else {
        to_unorm(value, zero, scale)
    }
}

// Local fused stage driver; individual arithmetic corresponds to the upstream stages.
// Selected native raster pipeline CPU stages. See ../../LICENSE.
// Port of SkRasterPipeline_opts.h's seed_shader, matrix_2x3,
// evenly_spaced_2_stop_gradient, dither, srcover and store_8888 stages.
// Same f32 fused operations and nearest-even packing, four pixels per group.
#[cfg(target_arch = "aarch64")]
/// # Safety
/// AArch64 NEON is mandatory on this target. No caller alignment is required.
/// `dst` exclusively owns its bytes for this call. Only `dst.len() / 16` complete
/// groups are touched; trailing bytes retain their original value. The pointer
/// loop below is bounded by that slice, including an empty slice. No pointer is
/// dereferenced outside a complete 16-byte group. NEON loads/stores support
/// unaligned access and the pointer is never dereferenced as a Rust `u32`.
/// The coordinates must describe the span without overflowing `u32`; this is
/// checked once before entering the kernel.
pub unsafe fn shade_two_stop_span(
    dst: &mut [u8],
    x: u32,
    y: u32,
    a: f32,
    row_bias: f32,
    factor: [f32; 4],
    bias: [f32; 4],
    premultiplied: bool,
    opaque: bool,
) {
    let context = TwoStopSpanContext::new(a, factor, bias);
    if opaque {
        shade_two_stop_span_kernel::<true>(dst, x, y, row_bias, &context, premultiplied);
    } else {
        shade_two_stop_span_kernel::<false>(dst, x, y, row_bias, &context, premultiplied);
    }
}

// Prepared SIMD representation of the upstream gradient/stage constants.
// This is local driver state: the official EvenlySpaced2StopGradientCtx stores
// scalar factors/biases and its compiled stages broadcast them. Prepare those
// broadcasts once per draw instead of once per clipped tile-row interval.
#[cfg(target_arch = "aarch64")]
struct TwoStopSpanContext {
    format: crate::PixelFormat,
    // The linear matrix's x coefficient, exactly zero for a vertical gradient.
    constant_in_x: bool,
    #[cfg(feature = "profiling")]
    verify_translucent: bool,
    factors: [std::arch::aarch64::float32x4_t; 4],
    biases: [std::arch::aarch64::float32x4_t; 4],
    matrix_scale: std::arch::aarch64::float32x4_t,
    zero: std::arch::aarch64::float32x4_t,
    one: std::arch::aarch64::float32x4_t,
    half: std::arch::aarch64::float32x4_t,
    lanes: std::arch::aarch64::uint32x4_t,
    step: std::arch::aarch64::uint32x4_t,
    dither_rate: std::arch::aarch64::float32x4_t,
    unorm_scale: std::arch::aarch64::float32x4_t,
}

#[cfg(target_arch = "aarch64")]
impl TwoStopSpanContext {
    fn new(a: f32, factor: [f32; 4], bias: [f32; 4]) -> Self {
        use std::arch::aarch64::*;
        unsafe {
            Self {
                format: crate::PixelFormat::Rgba8888,
                constant_in_x: a == 0.0,
                #[cfg(feature = "profiling")]
                verify_translucent: std::env::var_os("SKIA_VERIFY_TRANSLUCENT_SPANS").is_some(),
                // Explicit broadcasts also avoid the array map's general
                // partial-initialization/drop machinery in unoptimized builds.
                factors: [
                    vdupq_n_f32(factor[0]),
                    vdupq_n_f32(factor[1]),
                    vdupq_n_f32(factor[2]),
                    vdupq_n_f32(factor[3]),
                ],
                biases: [
                    vdupq_n_f32(bias[0]),
                    vdupq_n_f32(bias[1]),
                    vdupq_n_f32(bias[2]),
                    vdupq_n_f32(bias[3]),
                ],
                matrix_scale: vdupq_n_f32(a),
                zero: vdupq_n_f32(0.0),
                one: vdupq_n_f32(1.0),
                half: vdupq_n_f32(0.5),
                lanes: vld1q_u32([0, 1, 2, 3].as_ptr()),
                step: vdupq_n_u32(4),
                dither_rate: vdupq_n_f32(1.0 / 255.0),
                unorm_scale: vdupq_n_f32(255.0),
            }
        }
    }
}

// Eight output pixels of one opaque vertical row. This draw-local stage
// context is discarded on each row; no rendered frame or shader image is kept.
// matrix_2x3 is constant in x and dither uses only x/y mod 8, so adjacent
// clipped tile spans share the same eight values, possibly at another phase.
#[cfg(target_arch = "aarch64")]
struct VerticalSpanRow {
    row_bias: u32,
    y_phase: u32,
    pixels: [u8; 32],
}
#[cfg(target_arch = "aarch64")]
unsafe fn shade_vertical_row_span(
    dst: &mut [u8],
    x: u32,
    y: u32,
    row_bias: f32,
    context: &TwoStopSpanContext,
    premultiplied: bool,
    cached: &mut Option<VerticalSpanRow>,
    include_tail: bool,
) {
    let bytes = if include_tail {
        dst.len() / 4 * 4
    } else {
        dst.len() / 16 * 16
    };
    let pixels = bytes / 4;
    assert!(pixels == 0 || (pixels as u64 - 1) <= (u32::MAX - x) as u64);
    if bytes == 0 {
        return;
    }
    debug_assert!(context.constant_in_x);
    let y_phase = y & 7;
    if cached
        .as_ref()
        .is_none_or(|row| row.row_bias != row_bias.to_bits() || row.y_phase != y_phase)
    {
        let mut pixels = [0; 32];
        shade_two_stop_span_stages::<true>(
            &mut pixels,
            0,
            y_phase,
            row_bias,
            context,
            premultiplied,
        );
        *cached = Some(VerticalSpanRow {
            row_bias: row_bias.to_bits(),
            y_phase,
            pixels,
        });
    }
    let period = &cached.as_ref().unwrap().pixels;
    let phase = (x & 7) as usize * 4;
    let initial = bytes.min(32);
    let first = initial.min(32 - phase);
    dst[..first].copy_from_slice(&period[phase..phase + first]);
    dst[first..initial].copy_from_slice(&period[..initial - first]);
    let mut written = initial;
    while written < bytes {
        let count = written.min(bytes - written);
        dst.copy_within(..count, written);
        written += count;
    }
}

// Draw-local scheduling adapter for the same matrix/gradient/dither/store
// stages. This replay adapter has 256px tiles with gutters: x=0 is unique;
// x>=1 repeats local coordinates 1..=254, hence a 254-pixel output period.
// Fold that *exact* tile schedule into a contiguous row store when the shader
// is constant in x and the clip is a BW rect. No frame or shader image is kept.
#[cfg(target_arch = "aarch64")]
unsafe fn shade_vertical_tiled_row(
    dst: &mut [u8],
    mut x: u32,
    y: u32,
    row_bias: f32,
    context: &TwoStopSpanContext,
    premultiplied: bool,
) {
    debug_assert_eq!(dst.len() % 4, 0);
    let mut cached = None;
    let mut offset = 0;
    if x == 0 && !dst.is_empty() {
        shade_vertical_row_span(
            &mut dst[..4],
            0,
            y,
            row_bias,
            context,
            premultiplied,
            &mut cached,
            true,
        );
        offset = 4;
        x = 1;
    }
    let dst = &mut dst[offset..];
    if dst.is_empty() {
        return;
    }
    let phase = (x - 1) % 254;
    let first = (254 - phase) as usize * 4;
    let initial = dst.len().min(254 * 4);
    let prefix = initial.min(first);
    shade_vertical_row_span(
        &mut dst[..prefix],
        phase + 1,
        y,
        row_bias,
        context,
        premultiplied,
        &mut cached,
        true,
    );
    if prefix < initial {
        shade_vertical_row_span(
            &mut dst[prefix..initial],
            1,
            y,
            row_bias,
            context,
            premultiplied,
            &mut cached,
            true,
        );
    }
    let mut written = initial;
    while written < dst.len() {
        let count = written.min(dst.len() - written);
        dst.copy_within(..count, written);
        written += count;
    }
}

// Local driver specialization derived from the official matrix/gradient/
// dither/store stages. This is not a new upstream stage or a frame cache.
// For an opaque vertical shader, matrix_2x3 has no x contribution, and the
// ordered dither uses only the low three x bits. Therefore the *finished*
// eight-pixel group is invariant under x += 8 within this clipped tile span.
#[cfg(target_arch = "aarch64")]
unsafe fn shade_two_stop_span_kernel<const OPAQUE: bool>(
    dst: &mut [u8],
    x: u32,
    y: u32,
    row_bias: f32,
    context: &TwoStopSpanContext,
    premultiplied: bool,
) {
    let bytes = dst.len() / 16 * 16;
    let groups = bytes / 16;
    assert!(groups == 0 || (groups as u64 * 4 - 1) <= (u32::MAX - x) as u64);
    if OPAQUE && context.constant_in_x && bytes >= 32 {
        shade_two_stop_span_stages::<OPAQUE>(
            &mut dst[..32],
            x,
            y,
            row_bias,
            context,
            premultiplied,
        );
        // The first 32 bytes are an exact period. Doubling a written prefix
        // preserves that period; the final remainder is always a whole group.
        // Keep any incomplete four-pixel group untouched, like start_pipeline.
        let mut written = 32;
        while written < bytes {
            let count = written.min(bytes - written);
            dst.copy_within(..count, written);
            written += count;
        }
    } else if OPAQUE {
        shade_two_stop_span_stages::<OPAQUE>(dst, x, y, row_bias, context, premultiplied);
    } else {
        #[cfg(feature = "profiling")]
        let reference = context.verify_translucent.then(|| {
            let mut expected = dst.to_vec();
            shade_two_stop_span_stages::<false>(
                &mut expected,
                x,
                y,
                row_bias,
                context,
                premultiplied,
            );
            expected
        });
        if context.constant_in_x && dst.len() >= 128 {
            #[cfg(feature = "profiling")]
            if context.verify_translucent {
                static FIRST_UNIFORM_SPAN: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                if !FIRST_UNIFORM_SPAN.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    eprintln!(
                        "vertical-translucent-oracle admitted x={x} y={y} bytes={} format={:?}",
                        dst.len(),
                        context.format
                    );
                }
            }
            shade_vertical_translucent_span(dst, x, y, row_bias, context, premultiplied);
        } else {
            shade_translucent_span_neon(dst, x, y, row_bias, context, premultiplied);
        }
        #[cfg(feature = "profiling")]
        if let Some(reference) = reference {
            if let Some(at) = dst.iter().zip(&reference).position(|(a, b)| a != b) {
                panic!("translucent span differs from stages: x={x} y={y} row_bias={row_bias:?} premul={premultiplied} format={:?} byte={at} actual={} expected={}", context.format, dst[at], reference[at]);
            }
        }
    }
}

// LOCAL_OPTIMIZATION: same HIGHP matrix, gradient, premul, dither,
// load_dst/SrcOver, clamp and nearest-even N32 store as the stages oracle.
// Four pixels remain in registers through the complete span. The dither's
// eight-pixel period is draw-local setup; destination pixels are still loaded
// and composited individually. Incomplete groups remain untouched for the
// existing scalar tail. No rendering or compiler optimization mode changes.
#[cfg(target_arch = "aarch64")]
unsafe fn shade_translucent_span_neon(
    dst: &mut [u8],
    x: u32,
    y: u32,
    row_bias: f32,
    context: &TwoStopSpanContext,
    premultiplied: bool,
) {
    use std::arch::aarch64::*;
    let groups = dst.len() / 16;
    assert!(groups == 0 || (groups as u64 * 4 - 1) <= (u32::MAX - x) as u64);
    if groups == 0 {
        return;
    }
    let constants = [
        context.matrix_scale,
        context.half,
        vdupq_n_f32(row_bias),
        context.dither_rate,
        context.unorm_scale,
        context.zero,
        context.one,
        vreinterpretq_f32_u32(context.step),
    ];
    let coeff = [
        context.factors[0],
        context.biases[0],
        context.factors[1],
        context.biases[1],
        context.factors[2],
        context.biases[2],
        context.factors[3],
        context.biases[3],
    ];
    let coords = [x, x + 1, x + 2, x + 3];
    let noise: [f32; 8] = std::array::from_fn(|i| {
        // Low three coordinate bits alone select ordered dither. Wrapping the
        // inactive second half is valid even at the u32 coordinate boundary.
        dither(x.wrapping_add(i as u32), y)
    });
    let swap = context.format != crate::PixelFormat::Rgba8888;
    // SAFETY: every LDR/STR touches one complete 16-byte group inside dst;
    // all local vector arrays hold initialized fixed-size constants. Source
    // and destination are the same premultiplied byte layout as the oracle.
    core::arch::asm!(
        "ldp q4, q2, [{constants}]",
        "ldp q3, q7, [{constants}, #32]",
        "ldp q8, q5, [{constants}, #64]",
        "ldp q6, q1, [{constants}, #96]",
        "ldr q0, [{coords}]",
        "ldp q9, q10, [{noise}]",
        "movi v15.4s, #255",
        "dup v24.4s, {red_shift:w}",
        "movi v25.4s, #8",
        "neg v25.4s, v25.4s",
        "dup v26.4s, {blue_shift:w}",
        "neg v27.4s, v24.4s",
        "neg v28.4s, v26.4s",
        "2:",
        "ucvtf v16.4s, v0.4s",
        "fadd v16.4s, v16.4s, v2.4s",
        "mov v11.16b, v3.16b",
        "fmla v11.4s, v16.4s, v4.4s",
        "fmax v11.4s, v5.4s, v11.4s",
        "fmin v11.4s, v6.4s, v11.4s",
        "ldp q21, q22, [{coeff}, #96]",
        "mov v12.16b, v22.16b",
        "fmla v12.4s, v11.4s, v21.4s",
        "fmax v12.4s, v5.4s, v12.4s",
        "fmin v12.4s, v6.4s, v12.4s",
        "fsub v13.4s, v6.4s, v12.4s",
        "ldr q14, [{dst}]",
        "ldp q21, q22, [{coeff}, #0]",
        "mov v17.16b, v22.16b",
        "fmla v17.4s, v11.4s, v21.4s",
        "cbnz {premul:w}, 4f",
        "fmul v17.4s, v17.4s, v12.4s",
        "4:",
        "fmla v17.4s, v9.4s, v7.4s",
        "fmax v17.4s, v5.4s, v17.4s",
        "fmin v17.4s, v12.4s, v17.4s",
        "ushl v23.4s, v14.4s, v24.4s",
        "and v23.16b, v23.16b, v15.16b",
        "ucvtf v23.4s, v23.4s",
        "fmul v23.4s, v23.4s, v7.4s",
        "fmla v17.4s, v23.4s, v13.4s",
        "fmul v17.4s, v17.4s, v8.4s",
        "fmax v17.4s, v5.4s, v17.4s",
        "fmin v17.4s, v8.4s, v17.4s",
        "fcvtnu v17.4s, v17.4s",
        "ldp q21, q22, [{coeff}, #32]",
        "mov v18.16b, v22.16b",
        "fmla v18.4s, v11.4s, v21.4s",
        "cbnz {premul:w}, 5f",
        "fmul v18.4s, v18.4s, v12.4s",
        "5:",
        "fmla v18.4s, v9.4s, v7.4s",
        "fmax v18.4s, v5.4s, v18.4s",
        "fmin v18.4s, v12.4s, v18.4s",
        "ushl v23.4s, v14.4s, v25.4s",
        "and v23.16b, v23.16b, v15.16b",
        "ucvtf v23.4s, v23.4s",
        "fmul v23.4s, v23.4s, v7.4s",
        "fmla v18.4s, v23.4s, v13.4s",
        "fmul v18.4s, v18.4s, v8.4s",
        "fmax v18.4s, v5.4s, v18.4s",
        "fmin v18.4s, v8.4s, v18.4s",
        "fcvtnu v18.4s, v18.4s",
        "ldp q21, q22, [{coeff}, #64]",
        "mov v19.16b, v22.16b",
        "fmla v19.4s, v11.4s, v21.4s",
        "cbnz {premul:w}, 6f",
        "fmul v19.4s, v19.4s, v12.4s",
        "6:",
        "fmla v19.4s, v9.4s, v7.4s",
        "fmax v19.4s, v5.4s, v19.4s",
        "fmin v19.4s, v12.4s, v19.4s",
        "ushl v23.4s, v14.4s, v26.4s",
        "and v23.16b, v23.16b, v15.16b",
        "ucvtf v23.4s, v23.4s",
        "fmul v23.4s, v23.4s, v7.4s",
        "fmla v19.4s, v23.4s, v13.4s",
        "fmul v19.4s, v19.4s, v8.4s",
        "fmax v19.4s, v5.4s, v19.4s",
        "fmin v19.4s, v8.4s, v19.4s",
        "fcvtnu v19.4s, v19.4s",
        "ushr v23.4s, v14.4s, #24",
        "ucvtf v23.4s, v23.4s",
        "fmul v23.4s, v23.4s, v7.4s",
        "mov v20.16b, v12.16b",
        "fmla v20.4s, v23.4s, v13.4s",
        "fmul v20.4s, v20.4s, v8.4s",
        "fmax v20.4s, v5.4s, v20.4s",
        "fmin v20.4s, v8.4s, v20.4s",
        "fcvtnu v20.4s, v20.4s",
        "cbz {zero_x:w}, 7f",
        "movi v20.4s, #0",
        "7:",
        "ushl v17.4s, v17.4s, v27.4s",
        "shl v18.4s, v18.4s, #8",
        "ushl v19.4s, v19.4s, v28.4s",
        "shl v20.4s, v20.4s, #24",
        "orr v17.16b, v17.16b, v18.16b",
        "orr v19.16b, v19.16b, v20.16b",
        "orr v17.16b, v17.16b, v19.16b",
        "str q17, [{dst}], #16",
        "add v0.4s, v0.4s, v1.4s",
        "mov v16.16b, v9.16b",
        "mov v9.16b, v10.16b",
        "mov v10.16b, v16.16b",
        "subs {groups}, {groups}, #1",
        "b.ne 2b",
        dst = inout(reg) dst.as_mut_ptr() => _, groups = inout(reg) groups => _,
        constants = in(reg) constants.as_ptr(), coeff = in(reg) coeff.as_ptr(),
        coords = in(reg) coords.as_ptr(), noise = in(reg) noise.as_ptr(),
        premul = in(reg) premultiplied as u32,
        zero_x = in(reg) (context.format == crate::PixelFormat::Bgrx8888) as u32,
        red_shift = in(reg) if swap { -16i32 } else { 0i32 },
        blue_shift = in(reg) if swap { 0i32 } else { -16i32 },
        out("v0") _,
        out("v1") _,
        out("v2") _,
        out("v3") _,
        out("v4") _,
        out("v5") _,
        out("v6") _,
        out("v7") _,
        out("v8") _,
        out("v9") _,
        out("v10") _,
        out("v11") _,
        out("v12") _,
        out("v13") _,
        out("v14") _,
        out("v15") _,
        out("v16") _,
        out("v17") _,
        out("v18") _,
        out("v19") _,
        out("v20") _,
        out("v21") _,
        out("v22") _,
        out("v23") _,
        out("v24") _,
        out("v25") _,
        out("v26") _,
        out("v27") _,
        out("v28") _,
        options(nostack),
    );
}

// LOCAL_OPTIMIZATION candidate: constant-in-x shader values repeat every
// eight dither samples. Retain float RGBA values before load_dst; destination
// pixels are still loaded, SrcOver-composited, rounded and stored individually.
// The palette is fresh per clipped span, never reused across rows or frames.
#[cfg(target_arch = "aarch64")]
unsafe fn shade_vertical_translucent_span(
    dst: &mut [u8],
    x: u32,
    y: u32,
    row_bias: f32,
    context: &TwoStopSpanContext,
    premultiplied: bool,
) {
    use std::arch::aarch64::*;
    debug_assert!(context.constant_in_x);
    let groups = dst.len() / 16;
    assert!(groups >= 2 && (groups as u64 * 4 - 1) <= (u32::MAX - x) as u64);
    let constants = [
        context.matrix_scale,
        context.half,
        vdupq_n_f32(row_bias),
        context.dither_rate,
        context.unorm_scale,
        context.zero,
        context.one,
        vreinterpretq_f32_u32(context.step),
    ];
    let coeff = [
        context.factors[0],
        context.biases[0],
        context.factors[1],
        context.biases[1],
        context.factors[2],
        context.biases[2],
        context.factors[3],
        context.biases[3],
    ];
    let coords = [x, x + 1, x + 2, x + 3];
    let noise: [f32; 8] = std::array::from_fn(|i| dither(x + i as u32, y));
    let mut shader = [vdupq_n_f32(0.); 8];
    // Two complete float groups, preserving the original shader arithmetic.
    core::arch::asm!(
        "ldp q4, q2, [{constants}]",
        "ldp q3, q7, [{constants}, #32]",
        "ldp q8, q5, [{constants}, #64]",
        "ldp q6, q1, [{constants}, #96]",
        "ldr q0, [{coords}]",
        "ldp q9, q10, [{noise}]",
        "mov {count}, #2",
        "2:",
        "ucvtf v16.4s, v0.4s",
        "fadd v16.4s, v16.4s, v2.4s",
        "mov v11.16b, v3.16b",
        "fmla v11.4s, v16.4s, v4.4s",
        "fmax v11.4s, v5.4s, v11.4s",
        "fmin v11.4s, v6.4s, v11.4s",
        "ldp q21, q22, [{coeff}, #96]",
        "mov v12.16b, v22.16b",
        "fmla v12.4s, v11.4s, v21.4s",
        "fmax v12.4s, v5.4s, v12.4s",
        "fmin v12.4s, v6.4s, v12.4s",
        "fsub v13.4s, v6.4s, v12.4s",
        "ldp q21, q22, [{coeff}, #0]",
        "mov v17.16b, v22.16b",
        "fmla v17.4s, v11.4s, v21.4s",
        "cbnz {premul:w}, 4f",
        "fmul v17.4s, v17.4s, v12.4s",
        "4:",
        "fmla v17.4s, v9.4s, v7.4s",
        "fmax v17.4s, v5.4s, v17.4s",
        "fmin v17.4s, v12.4s, v17.4s",
        "ldp q21, q22, [{coeff}, #32]",
        "mov v18.16b, v22.16b",
        "fmla v18.4s, v11.4s, v21.4s",
        "cbnz {premul:w}, 5f",
        "fmul v18.4s, v18.4s, v12.4s",
        "5:",
        "fmla v18.4s, v9.4s, v7.4s",
        "fmax v18.4s, v5.4s, v18.4s",
        "fmin v18.4s, v12.4s, v18.4s",
        "ldp q21, q22, [{coeff}, #64]",
        "mov v19.16b, v22.16b",
        "fmla v19.4s, v11.4s, v21.4s",
        "cbnz {premul:w}, 6f",
        "fmul v19.4s, v19.4s, v12.4s",
        "6:",
        "fmla v19.4s, v9.4s, v7.4s",
        "fmax v19.4s, v5.4s, v19.4s",
        "fmin v19.4s, v12.4s, v19.4s",
        "stp q17, q18, [{shader}], #32",
        "stp q19, q12, [{shader}], #32",
        "add v0.4s, v0.4s, v1.4s",
        "mov v16.16b, v9.16b",
        "mov v9.16b, v10.16b",
        "mov v10.16b, v16.16b",
        "subs {count}, {count}, #1",
        "b.ne 2b",
        shader=inout(reg) shader.as_mut_ptr()=>_, count=out(reg) _,
        constants=in(reg) constants.as_ptr(),coords=in(reg) coords.as_ptr(),
        coeff=in(reg) coeff.as_ptr(),noise=in(reg) noise.as_ptr(),
        premul=in(reg) premultiplied as u32,
        out("v0") _,
        out("v1") _,
        out("v2") _,
        out("v3") _,
        out("v4") _,
        out("v5") _,
        out("v6") _,
        out("v7") _,
        out("v8") _,
        out("v9") _,
        out("v10") _,
        out("v11") _,
        out("v12") _,
        out("v13") _,
        out("v16") _,
        out("v17") _,
        out("v18") _,
        out("v19") _,
        out("v21") _,
        out("v22") _,
        options(nostack),
    );
    let output_constants = [1.0f32, 1.0 / 255.0, 255.0, 0.0];
    let swap = context.format != crate::PixelFormat::Rgba8888;
    // Every store covers a complete accessible four-pixel destination group.
    // A partial group is untouched for the original caller's scalar tail.
    core::arch::asm!(
        "ldp q0, q1, [{shader}]",
        "ldp q2, q3, [{shader}, #32]",
        "ldp q4, q5, [{shader}, #64]",
        "ldp q6, q7, [{shader}, #96]",
        "ld1r {{v10.4s}}, [{constants}], #4",
        "ld1r {{v11.4s}}, [{constants}], #4",
        "ld1r {{v12.4s}}, [{constants}], #4",
        "ld1r {{v14.4s}}, [{constants}]",
        "movi v13.4s, #255",
        "dup v15.4s, {red_shift:w}",
        "dup v16.4s, {blue_shift:w}",
        "neg v17.4s, v15.4s",
        "neg v18.4s, v16.4s",
        "2:",
        "fsub v8.4s, v10.4s, v3.4s",
        "ldr q9, [{dst}]",
        "ushl v23.4s, v9.4s, v15.4s",
        "and v23.16b, v23.16b, v13.16b",
        "ucvtf v23.4s, v23.4s",
        "fmul v23.4s, v23.4s, v11.4s",
        "mov v19.16b, v0.16b",
        "fmla v19.4s, v23.4s, v8.4s",
        "fmul v19.4s, v19.4s, v12.4s",
        "fmax v19.4s, v14.4s, v19.4s",
        "fmin v19.4s, v12.4s, v19.4s",
        "fcvtnu v19.4s, v19.4s",
        "ushr v23.4s, v9.4s, #8",
        "and v23.16b, v23.16b, v13.16b",
        "ucvtf v23.4s, v23.4s",
        "fmul v23.4s, v23.4s, v11.4s",
        "mov v20.16b, v1.16b",
        "fmla v20.4s, v23.4s, v8.4s",
        "fmul v20.4s, v20.4s, v12.4s",
        "fmax v20.4s, v14.4s, v20.4s",
        "fmin v20.4s, v12.4s, v20.4s",
        "fcvtnu v20.4s, v20.4s",
        "ushl v23.4s, v9.4s, v16.4s",
        "and v23.16b, v23.16b, v13.16b",
        "ucvtf v23.4s, v23.4s",
        "fmul v23.4s, v23.4s, v11.4s",
        "mov v21.16b, v2.16b",
        "fmla v21.4s, v23.4s, v8.4s",
        "fmul v21.4s, v21.4s, v12.4s",
        "fmax v21.4s, v14.4s, v21.4s",
        "fmin v21.4s, v12.4s, v21.4s",
        "fcvtnu v21.4s, v21.4s",
        "ushr v23.4s, v9.4s, #24",
        "and v23.16b, v23.16b, v13.16b",
        "ucvtf v23.4s, v23.4s",
        "fmul v23.4s, v23.4s, v11.4s",
        "mov v22.16b, v3.16b",
        "fmla v22.4s, v23.4s, v8.4s",
        "fmul v22.4s, v22.4s, v12.4s",
        "fmax v22.4s, v14.4s, v22.4s",
        "fmin v22.4s, v12.4s, v22.4s",
        "fcvtnu v22.4s, v22.4s",
        "cbz {zero_x:w}, 7f",
        "movi v22.4s, #0",
        "7:",
        "ushl v19.4s, v19.4s, v17.4s",
        "shl v20.4s, v20.4s, #8",
        "ushl v21.4s, v21.4s, v18.4s",
        "shl v22.4s, v22.4s, #24",
        "orr v19.16b, v19.16b, v20.16b",
        "orr v21.16b, v21.16b, v22.16b",
        "orr v19.16b, v19.16b, v21.16b",
        "str q19, [{dst}], #16",
        "mov v24.16b, v0.16b",
        "mov v0.16b, v4.16b",
        "mov v4.16b, v24.16b",
        "mov v24.16b, v1.16b",
        "mov v1.16b, v5.16b",
        "mov v5.16b, v24.16b",
        "mov v24.16b, v2.16b",
        "mov v2.16b, v6.16b",
        "mov v6.16b, v24.16b",
        "mov v24.16b, v3.16b",
        "mov v3.16b, v7.16b",
        "mov v7.16b, v24.16b",
        "subs {groups}, {groups}, #1",
        "b.ne 2b",
        shader=in(reg) shader.as_ptr(),
        constants=inout(reg) output_constants.as_ptr()=>_,
        dst=inout(reg) dst.as_mut_ptr()=>_,groups=inout(reg) groups=>_,
        red_shift=in(reg) if swap {-16i32} else {0i32},
        blue_shift=in(reg) if swap {0i32} else {-16i32},
        zero_x=in(reg) (context.format==crate::PixelFormat::Bgrx8888) as u32,
        out("v0") _,
        out("v1") _,
        out("v2") _,
        out("v3") _,
        out("v4") _,
        out("v5") _,
        out("v6") _,
        out("v7") _,
        out("v8") _,
        out("v9") _,
        out("v10") _,
        out("v11") _,
        out("v12") _,
        out("v13") _,
        out("v14") _,
        out("v15") _,
        out("v16") _,
        out("v17") _,
        out("v18") _,
        out("v19") _,
        out("v20") _,
        out("v21") _,
        out("v22") _,
        out("v23") _,
        out("v24") _,
        options(nostack),
    );
}

// start_pipeline patches a final N-lane group's memory context to scratch,
// executes the same stage program, and copies back only its active lanes.
// This opaque subset has no load_dst stage, so scratch needs no source pixels.
// Adjacent inactive lanes may cross a stop; their results are never committed.
#[cfg(target_arch = "aarch64")]
unsafe fn shade_opaque_span_with_tail(
    dst: &mut [u8],
    x: u32,
    y: u32,
    row_bias: f32,
    context: &TwoStopSpanContext,
) {
    let bytes = dst.len() / 16 * 16;
    shade_opaque_span_neon(dst, x, y, row_bias, context);
    if bytes != dst.len() {
        let mut scratch = [0u8; 16];
        shade_opaque_span_neon(&mut scratch, x + (bytes / 4) as u32, y, row_bias, context);
        let tail = dst.len() - bytes;
        dst[bytes..].copy_from_slice(&scratch[..tail]);
    }
}

// AArch64 implementation of the same opaque HIGHP stage sequence below.
// At opt-level=0 std::arch's generated NEON intrinsics each use an out-of-line
// vector ABI. Keep this one complete span in registers instead. This is a local
// fused driver, not an additional Skia stage or an alternative color formula.
#[cfg(target_arch = "aarch64")]
unsafe fn shade_opaque_span_neon(
    dst: &mut [u8],
    x: u32,
    y: u32,
    row_bias: f32,
    context: &TwoStopSpanContext,
) {
    use std::arch::aarch64::*;
    let groups = dst.len() / 16;
    assert!(groups == 0 || (groups as u64 * 4 - 1) <= (u32::MAX - x) as u64);
    if groups == 0 {
        return;
    }
    let xx = vaddq_u32(vdupq_n_u32(x), context.lanes);
    let yy = vdupq_n_u32(y);
    // Dispatch byte order once per span. Opaque input alpha is exactly one;
    // premultiplication by one is an identity for our finite validated colors.
    let (first, last) = if context.format == crate::PixelFormat::Rgba8888 {
        (0, 2)
    } else {
        (2, 0)
    };
    let alpha = vdupq_n_u32(if context.format == crate::PixelFormat::Bgrx8888 {
        0
    } else {
        0xff00_0000
    });
    // vfmaq_f32 maps to FMLA (accumulator + lhs*rhs). FMAX/FMIN preserve
    // the intrinsic's clamp semantics; FCVTNU uses its nearest-even packing.
    // The two noise vectors are the exact existing dither stage results.
    // Each STR writes one unaligned 16-byte group inside dst; groups excludes
    // incomplete lanes, and the coordinate extent was checked above. NEON is
    // mandatory on AArch64. No memory is read and no flags escape the block.
    core::arch::asm!(
        // dither's integer fcebda construction and exact final mad, once for
        // each half of its eight-pixel period, before the span's stage loop.
        "eor {seed:v}.16b, {xx:v}.16b, {yy:v}.16b",
        "movi {t:v}.4s, #1",
        "and {noise0:v}.16b, {seed:v}.16b, {t:v}.16b",
        "shl {noise0:v}.4s, {noise0:v}.4s, #5",
        "and {c0:v}.16b, {xx:v}.16b, {t:v}.16b",
        "shl {c0:v}.4s, {c0:v}.4s, #4",
        "orr {noise0:v}.16b, {noise0:v}.16b, {c0:v}.16b",
        "movi {t:v}.4s, #2",
        "and {c0:v}.16b, {seed:v}.16b, {t:v}.16b",
        "shl {c0:v}.4s, {c0:v}.4s, #2",
        "orr {noise0:v}.16b, {noise0:v}.16b, {c0:v}.16b",
        "and {c0:v}.16b, {xx:v}.16b, {t:v}.16b",
        "shl {c0:v}.4s, {c0:v}.4s, #1",
        "orr {noise0:v}.16b, {noise0:v}.16b, {c0:v}.16b",
        "movi {t:v}.4s, #4",
        "and {c0:v}.16b, {seed:v}.16b, {t:v}.16b",
        "ushr {c0:v}.4s, {c0:v}.4s, #1",
        "orr {noise0:v}.16b, {noise0:v}.16b, {c0:v}.16b",
        "and {c0:v}.16b, {xx:v}.16b, {t:v}.16b",
        "ushr {c0:v}.4s, {c0:v}.4s, #2",
        "orr {noise0:v}.16b, {noise0:v}.16b, {c0:v}.16b",
        "ucvtf {noise0:v}.4s, {noise0:v}.4s",
        "dup {c1:v}.4s, {noise_scale:w}",
        "dup {t:v}.4s, {noise_bias:w}",
        "fmla {t:v}.4s, {noise0:v}.4s, {c1:v}.4s",
        "mov {noise0:v}.16b, {t:v}.16b",
        "add {c2:v}.4s, {xx:v}.4s, {step:v}.4s",
        "eor {seed:v}.16b, {c2:v}.16b, {yy:v}.16b",
        "movi {t:v}.4s, #1",
        "and {noise1:v}.16b, {seed:v}.16b, {t:v}.16b",
        "shl {noise1:v}.4s, {noise1:v}.4s, #5",
        "and {c0:v}.16b, {c2:v}.16b, {t:v}.16b",
        "shl {c0:v}.4s, {c0:v}.4s, #4",
        "orr {noise1:v}.16b, {noise1:v}.16b, {c0:v}.16b",
        "movi {t:v}.4s, #2",
        "and {c0:v}.16b, {seed:v}.16b, {t:v}.16b",
        "shl {c0:v}.4s, {c0:v}.4s, #2",
        "orr {noise1:v}.16b, {noise1:v}.16b, {c0:v}.16b",
        "and {c0:v}.16b, {c2:v}.16b, {t:v}.16b",
        "shl {c0:v}.4s, {c0:v}.4s, #1",
        "orr {noise1:v}.16b, {noise1:v}.16b, {c0:v}.16b",
        "movi {t:v}.4s, #4",
        "and {c0:v}.16b, {seed:v}.16b, {t:v}.16b",
        "ushr {c0:v}.4s, {c0:v}.4s, #1",
        "orr {noise1:v}.16b, {noise1:v}.16b, {c0:v}.16b",
        "and {c0:v}.16b, {c2:v}.16b, {t:v}.16b",
        "ushr {c0:v}.4s, {c0:v}.4s, #2",
        "orr {noise1:v}.16b, {noise1:v}.16b, {c0:v}.16b",
        "ucvtf {noise1:v}.4s, {noise1:v}.4s",
        "dup {c1:v}.4s, {noise_scale:w}",
        "dup {t:v}.4s, {noise_bias:w}",
        "fmla {t:v}.4s, {noise1:v}.4s, {c1:v}.4s",
        "mov {noise1:v}.16b, {t:v}.16b",
        "2:",
        "ucvtf {seed:v}.4s, {xx:v}.4s",
        "fadd {seed:v}.4s, {seed:v}.4s, {half:v}.4s",
        "mov {t:v}.16b, {row:v}.16b",
        "fmla {t:v}.4s, {seed:v}.4s, {scale:v}.4s",
        "fmax {t:v}.4s, {zero:v}.4s, {t:v}.4s",
        "fmin {t:v}.4s, {one:v}.4s, {t:v}.4s",
        "mov {c0:v}.16b, {b0:v}.16b",
        "fmla {c0:v}.4s, {t:v}.4s, {f0:v}.4s",
        "fmla {c0:v}.4s, {noise0:v}.4s, {rate:v}.4s",
        "fmax {c0:v}.4s, {zero:v}.4s, {c0:v}.4s",
        "fmin {c0:v}.4s, {one:v}.4s, {c0:v}.4s",
        "mov {c1:v}.16b, {b1:v}.16b",
        "fmla {c1:v}.4s, {t:v}.4s, {f1:v}.4s",
        "fmla {c1:v}.4s, {noise0:v}.4s, {rate:v}.4s",
        "fmax {c1:v}.4s, {zero:v}.4s, {c1:v}.4s",
        "fmin {c1:v}.4s, {one:v}.4s, {c1:v}.4s",
        "mov {c2:v}.16b, {b2:v}.16b",
        "fmla {c2:v}.4s, {t:v}.4s, {f2:v}.4s",
        "fmla {c2:v}.4s, {noise0:v}.4s, {rate:v}.4s",
        "fmax {c2:v}.4s, {zero:v}.4s, {c2:v}.4s",
        "fmin {c2:v}.4s, {one:v}.4s, {c2:v}.4s",
        "fmul {c0:v}.4s, {c0:v}.4s, {unorm:v}.4s",
        "fcvtnu {c0:v}.4s, {c0:v}.4s",
        "fmul {c1:v}.4s, {c1:v}.4s, {unorm:v}.4s",
        "fcvtnu {c1:v}.4s, {c1:v}.4s",
        "fmul {c2:v}.4s, {c2:v}.4s, {unorm:v}.4s",
        "fcvtnu {c2:v}.4s, {c2:v}.4s",
        "shl {c1:v}.4s, {c1:v}.4s, #8",
        "shl {c2:v}.4s, {c2:v}.4s, #16",
        "orr {c0:v}.16b, {c0:v}.16b, {c1:v}.16b",
        "orr {c2:v}.16b, {c2:v}.16b, {alpha:v}.16b",
        "orr {c0:v}.16b, {c0:v}.16b, {c2:v}.16b",
        "str {c0:q}, [{ptr}], #16",
        "add {xx:v}.4s, {xx:v}.4s, {step:v}.4s",
        "mov {seed:v}.16b, {noise0:v}.16b",
        "mov {noise0:v}.16b, {noise1:v}.16b",
        "mov {noise1:v}.16b, {seed:v}.16b",
        "subs {groups}, {groups}, #1",
        "b.ne 2b",
        ptr = inout(reg) dst.as_mut_ptr() => _,
        groups = inout(reg) groups => _,
        xx = inout(vreg) xx => _,
        noise0 = out(vreg) _,
        noise1 = out(vreg) _,
        yy = in(vreg) yy,
        noise_scale = in(reg) (2.0f32 / 128.0).to_bits(),
        noise_bias = in(reg) (-63.0f32 / 128.0).to_bits(),
        half = in(vreg) context.half,
        row = in(vreg) vdupq_n_f32(row_bias),
        scale = in(vreg) context.matrix_scale,
        zero = in(vreg) context.zero,
        one = in(vreg) context.one,
        f0 = in(vreg) context.factors[first],
        f1 = in(vreg) context.factors[1],
        f2 = in(vreg) context.factors[last],
        b0 = in(vreg) context.biases[first],
        b1 = in(vreg) context.biases[1],
        b2 = in(vreg) context.biases[last],
        rate = in(vreg) context.dither_rate,
        unorm = in(vreg) context.unorm_scale,
        alpha = in(vreg) alpha,
        step = in(vreg) context.step,
        seed = out(vreg) _,
        t = out(vreg) _,
        c0 = out(vreg) _,
        c1 = out(vreg) _,
        c2 = out(vreg) _,
        options(nostack),
    );
}

#[cfg(target_arch = "aarch64")]
unsafe fn shade_two_stop_span_stages<const OPAQUE: bool>(
    dst: &mut [u8],
    x: u32,
    y: u32,
    row_bias: f32,
    context: &TwoStopSpanContext,
    premultiplied: bool,
) {
    if OPAQUE {
        shade_opaque_span_neon(dst, x, y, row_bias, context);
        return;
    }
    use std::arch::aarch64::*;
    let zero = context.zero;
    let one = context.one;
    let lanes = context.lanes;
    let factors = context.factors;
    let biases = context.biases;
    let ff = vdupq_n_u32(255);
    let groups = dst.len() / 16;
    // A slice's byte length is bounded by isize::MAX, so groups * 16 cannot
    // overflow usize. Check the coordinate extent once, outside the hot loop.
    assert!(groups == 0 || (groups as u64 * 4 - 1) <= (u32::MAX - x) as u64);
    let mut ptr = dst.as_mut_ptr();
    let end = ptr.wrapping_add(groups * 16);
    let mut xx = vaddq_u32(vdupq_n_u32(x), lanes);
    let step = context.step;
    let yy_row = vdupq_n_u32(y);
    let half = context.half;
    let row_bias = vdupq_n_f32(row_bias);
    let matrix_scale = context.matrix_scale;
    let bit1 = vdupq_n_u32(1);
    let bit2 = vdupq_n_u32(2);
    let bit4 = vdupq_n_u32(4);
    let noise_bias = vdupq_n_f32(-63.0 / 128.0);
    let noise_scale = vdupq_n_f32(2.0 / 128.0);
    let dither_rate = context.dither_rate;
    let unorm_scale = context.unorm_scale;
    let opaque_alpha = vdupq_n_u32(0xff00_0000);
    // Official dither reads only the low three coordinate bits. On this fixed
    // row it repeats every eight x coordinates: two four-lane groups. Compute
    // the exact stage's FMA once for each group; swap the vectors after a store.
    // This is context setup for our fused driver, not a new upstream stage.
    let mut span_noise = if OPAQUE {
        dither_four_lanes(xx, yy_row)
    } else {
        zero
    };
    let mut next_noise = if OPAQUE {
        dither_four_lanes(vaddq_u32(xx, step), yy_row)
    } else {
        zero
    };
    // SkRasterPipeline_opts.h::start_pipeline visits complete N-lane groups;
    // MemoryCtx/store_8888 access consecutive pixels through an unaligned
    // pointer. This local fused driver follows that traversal without creating
    // and splitting a new Rust subslice for every group. Constants are shader/
    // row context, not per-group work. The caller handles incomplete lanes.
    while ptr != end {
        let seed = vaddq_f32(vcvtq_f32_u32(xx), half);
        let t = vminq_f32(
            one,
            vmaxq_f32(zero, vfmaq_f32(row_bias, seed, matrix_scale)),
        );
        let noise = if OPAQUE {
            span_noise
        } else {
            let yy = veorq_u32(yy_row, xx);
            let bits = vorrq_u32(
                vorrq_u32(
                    vshlq_n_u32::<5>(vandq_u32(yy, bit1)),
                    vshlq_n_u32::<4>(vandq_u32(xx, bit1)),
                ),
                vorrq_u32(
                    vorrq_u32(
                        vshlq_n_u32::<2>(vandq_u32(yy, bit2)),
                        vshlq_n_u32::<1>(vandq_u32(xx, bit2)),
                    ),
                    vorrq_u32(
                        vshrq_n_u32::<1>(vandq_u32(yy, bit4)),
                        vshrq_n_u32::<2>(vandq_u32(xx, bit4)),
                    ),
                ),
            );
            vfmaq_f32(noise_bias, vcvtq_f32_u32(bits), noise_scale)
        };
        let alpha = if OPAQUE {
            one
        } else {
            vminq_f32(one, vmaxq_f32(zero, vfmaq_f32(biases[3], t, factors[3])))
        };
        let mut r = gradient_channel::<OPAQUE>(
            t,
            factors[0],
            biases[0],
            noise,
            alpha,
            premultiplied,
            zero,
            dither_rate,
        );
        let mut g = gradient_channel::<OPAQUE>(
            t,
            factors[1],
            biases[1],
            noise,
            alpha,
            premultiplied,
            zero,
            dither_rate,
        );
        let mut b = gradient_channel::<OPAQUE>(
            t,
            factors[2],
            biases[2],
            noise,
            alpha,
            premultiplied,
            zero,
            dither_rate,
        );
        let mut output_alpha = alpha;
        if !OPAQUE {
            let inverse = vsubq_f32(one, alpha);
            let old = vld1q_u32(ptr.cast());
            let old_red = if context.format == crate::PixelFormat::Rgba8888 {
                vandq_u32(old, ff)
            } else {
                vandq_u32(vshrq_n_u32::<16>(old), ff)
            };
            let old_blue = if context.format == crate::PixelFormat::Rgba8888 {
                vandq_u32(vshrq_n_u32::<16>(old), ff)
            } else {
                vandq_u32(old, ff)
            };
            r = src_over(r, old_red, inverse, dither_rate);
            g = src_over(
                g,
                vandq_u32(vshrq_n_u32::<8>(old), ff),
                inverse,
                dither_rate,
            );
            b = src_over(b, old_blue, inverse, dither_rate);
            output_alpha = src_over(alpha, vshrq_n_u32::<24>(old), inverse, dither_rate);
        }
        // The pipeline is dispatched once knowing opacity. For alpha=1,
        // multiplying an unpremultiplied finite color by one and quantizing
        // alpha to 255 are identities; match the official opaque stage setup.
        let packed_alpha = if context.format == crate::PixelFormat::Bgrx8888 {
            vdupq_n_u32(0)
        } else if OPAQUE {
            opaque_alpha
        } else {
            vshlq_n_u32::<24>(to_unorm(output_alpha, zero, unorm_scale))
        };
        let (r, b) = if context.format == crate::PixelFormat::Rgba8888 {
            (r, b)
        } else {
            (b, r)
        };
        let pixel = vorrq_u32(
            vorrq_u32(
                pack_gradient_channel::<OPAQUE>(r, zero, unorm_scale),
                vshlq_n_u32::<8>(pack_gradient_channel::<OPAQUE>(g, zero, unorm_scale)),
            ),
            vorrq_u32(
                vshlq_n_u32::<16>(pack_gradient_channel::<OPAQUE>(b, zero, unorm_scale)),
                packed_alpha,
            ),
        );
        vst1q_u32(ptr.cast(), pixel);
        // `ptr` remains inside this slice or exactly one-past the last complete
        // group. wrapping_add avoids per-group debug pointer precondition
        // checks; the once-established slice bound proves every actual access.
        ptr = ptr.wrapping_add(16);
        xx = vaddq_u32(xx, step);
        if OPAQUE {
            let previous_noise = span_noise;
            span_noise = next_noise;
            next_noise = previous_noise;
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn dither_four_lanes(
    xx: std::arch::aarch64::uint32x4_t,
    yy_row: std::arch::aarch64::uint32x4_t,
) -> std::arch::aarch64::float32x4_t {
    use std::arch::aarch64::*;
    let yy = veorq_u32(yy_row, xx);
    let bit1 = vdupq_n_u32(1);
    let bit2 = vdupq_n_u32(2);
    let bit4 = vdupq_n_u32(4);
    let noise_bias = vdupq_n_f32(-63.0 / 128.0);
    let noise_scale = vdupq_n_f32(2.0 / 128.0);
    let bits = vorrq_u32(
        vorrq_u32(
            vshlq_n_u32::<5>(vandq_u32(yy, bit1)),
            vshlq_n_u32::<4>(vandq_u32(xx, bit1)),
        ),
        vorrq_u32(
            vorrq_u32(
                vshlq_n_u32::<2>(vandq_u32(yy, bit2)),
                vshlq_n_u32::<1>(vandq_u32(xx, bit2)),
            ),
            vorrq_u32(
                vshrq_n_u32::<1>(vandq_u32(yy, bit4)),
                vshrq_n_u32::<2>(vandq_u32(xx, bit4)),
            ),
        ),
    );
    vfmaq_f32(noise_bias, vcvtq_f32_u32(bits), noise_scale)
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn gradient_channel<const OPAQUE: bool>(
    t: std::arch::aarch64::float32x4_t,
    factor: std::arch::aarch64::float32x4_t,
    bias: std::arch::aarch64::float32x4_t,
    noise: std::arch::aarch64::float32x4_t,
    alpha: std::arch::aarch64::float32x4_t,
    premultiplied: bool,
    zero: std::arch::aarch64::float32x4_t,
    dither_rate: std::arch::aarch64::float32x4_t,
) -> std::arch::aarch64::float32x4_t {
    use std::arch::aarch64::*;
    let value = vfmaq_f32(bias, t, factor);
    let source = if OPAQUE || premultiplied {
        value
    } else {
        vmulq_f32(value, alpha)
    };
    vminq_f32(
        alpha,
        vmaxq_f32(zero, vfmaq_f32(source, noise, dither_rate)),
    )
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn src_over(
    source: std::arch::aarch64::float32x4_t,
    old: std::arch::aarch64::uint32x4_t,
    inverse: std::arch::aarch64::float32x4_t,
    unorm_reciprocal: std::arch::aarch64::float32x4_t,
) -> std::arch::aarch64::float32x4_t {
    use std::arch::aarch64::*;
    vfmaq_f32(
        source,
        vmulq_f32(vcvtq_f32_u32(old), unorm_reciprocal),
        inverse,
    )
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn to_unorm(
    value: std::arch::aarch64::float32x4_t,
    zero: std::arch::aarch64::float32x4_t,
    scale: std::arch::aarch64::float32x4_t,
) -> std::arch::aarch64::uint32x4_t {
    use std::arch::aarch64::*;
    vcvtnq_u32_f32(vminq_f32(scale, vmaxq_f32(zero, vmulq_f32(value, scale))))
}

// Complete previous driver and helper snapshot. This independent byte oracle
// deliberately retains per-group dither and opaque alpha quantization.
#[cfg(all(test, target_arch = "aarch64"))]
mod span_regression_tests {
    use super::{
        shade_two_stop_span, shade_two_stop_span_stages, shade_vertical_row_span,
        TwoStopSpanContext,
    };
    use super::{shade_vertical_tiled_row, tile_shift};
    unsafe fn old_loop_oracle(
        dst: &mut [u8],
        x: u32,
        y: u32,
        a: f32,
        row_bias: f32,
        factor: [f32; 4],
        bias: [f32; 4],
        premultiplied: bool,
        opaque: bool,
    ) {
        use std::arch::aarch64::*;
        let zero = vdupq_n_f32(0.0);
        let one = vdupq_n_f32(1.0);
        let lanes = vld1q_u32([0, 1, 2, 3].as_ptr());
        let ff = vdupq_n_u32(255);
        let factors = factor.map(|v| vdupq_n_f32(v));
        let biases = bias.map(|v| vdupq_n_f32(v));
        let groups = dst.len() / 16;
        // A slice's byte length is bounded by isize::MAX, so groups * 16 cannot
        // overflow usize. Check the coordinate extent once, outside the hot loop.
        assert!(groups == 0 || (groups as u64 * 4 - 1) <= (u32::MAX - x) as u64);
        let mut ptr = dst.as_mut_ptr();
        let end = ptr.wrapping_add(groups * 16);
        let mut xx = vaddq_u32(vdupq_n_u32(x), lanes);
        let step = vdupq_n_u32(4);
        let yy_row = vdupq_n_u32(y);
        let half = vdupq_n_f32(0.5);
        let row_bias = vdupq_n_f32(row_bias);
        let matrix_scale = vdupq_n_f32(a);
        let bit1 = vdupq_n_u32(1);
        let bit2 = vdupq_n_u32(2);
        let bit4 = vdupq_n_u32(4);
        let noise_bias = vdupq_n_f32(-63.0 / 128.0);
        let noise_scale = vdupq_n_f32(2.0 / 128.0);
        let dither_rate = vdupq_n_f32(1.0 / 255.0);
        let unorm_scale = vdupq_n_f32(255.0);
        // SkRasterPipeline_opts.h::start_pipeline visits complete N-lane groups;
        // MemoryCtx/store_8888 access consecutive pixels through an unaligned
        // pointer. This local fused driver follows that traversal without creating
        // and splitting a new Rust subslice for every group. Constants are shader/
        // row context, not per-group work. The caller handles incomplete lanes.
        while ptr != end {
            let yy = veorq_u32(yy_row, xx);
            let seed = vaddq_f32(vcvtq_f32_u32(xx), half);
            let t = vminq_f32(
                one,
                vmaxq_f32(zero, vfmaq_f32(row_bias, seed, matrix_scale)),
            );
            let bits = vorrq_u32(
                vorrq_u32(
                    vshlq_n_u32::<5>(vandq_u32(yy, bit1)),
                    vshlq_n_u32::<4>(vandq_u32(xx, bit1)),
                ),
                vorrq_u32(
                    vorrq_u32(
                        vshlq_n_u32::<2>(vandq_u32(yy, bit2)),
                        vshlq_n_u32::<1>(vandq_u32(xx, bit2)),
                    ),
                    vorrq_u32(
                        vshrq_n_u32::<1>(vandq_u32(yy, bit4)),
                        vshrq_n_u32::<2>(vandq_u32(xx, bit4)),
                    ),
                ),
            );
            let noise = vfmaq_f32(noise_bias, vcvtq_f32_u32(bits), noise_scale);
            let alpha = if opaque {
                one
            } else {
                vminq_f32(one, vmaxq_f32(zero, vfmaq_f32(biases[3], t, factors[3])))
            };
            let mut r = gradient_channel(
                t,
                factors[0],
                biases[0],
                noise,
                alpha,
                premultiplied,
                zero,
                dither_rate,
            );
            let mut g = gradient_channel(
                t,
                factors[1],
                biases[1],
                noise,
                alpha,
                premultiplied,
                zero,
                dither_rate,
            );
            let mut b = gradient_channel(
                t,
                factors[2],
                biases[2],
                noise,
                alpha,
                premultiplied,
                zero,
                dither_rate,
            );
            let mut output_alpha = alpha;
            if !opaque {
                let inverse = vsubq_f32(one, alpha);
                let old = vld1q_u32(ptr.cast());
                r = src_over(r, vandq_u32(old, ff), inverse, dither_rate);
                g = src_over(
                    g,
                    vandq_u32(vshrq_n_u32::<8>(old), ff),
                    inverse,
                    dither_rate,
                );
                b = src_over(
                    b,
                    vandq_u32(vshrq_n_u32::<16>(old), ff),
                    inverse,
                    dither_rate,
                );
                output_alpha = src_over(alpha, vshrq_n_u32::<24>(old), inverse, dither_rate);
            }
            let pixel = vorrq_u32(
                vorrq_u32(
                    to_unorm(r, zero, unorm_scale),
                    vshlq_n_u32::<8>(to_unorm(g, zero, unorm_scale)),
                ),
                vorrq_u32(
                    vshlq_n_u32::<16>(to_unorm(b, zero, unorm_scale)),
                    vshlq_n_u32::<24>(to_unorm(output_alpha, zero, unorm_scale)),
                ),
            );
            vst1q_u32(ptr.cast(), pixel);
            // `ptr` remains inside this slice or exactly one-past the last complete
            // group. wrapping_add avoids per-group debug pointer precondition
            // checks; the once-established slice bound proves every actual access.
            ptr = ptr.wrapping_add(16);
            xx = vaddq_u32(xx, step);
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    unsafe fn gradient_channel(
        t: std::arch::aarch64::float32x4_t,
        factor: std::arch::aarch64::float32x4_t,
        bias: std::arch::aarch64::float32x4_t,
        noise: std::arch::aarch64::float32x4_t,
        alpha: std::arch::aarch64::float32x4_t,
        premultiplied: bool,
        zero: std::arch::aarch64::float32x4_t,
        dither_rate: std::arch::aarch64::float32x4_t,
    ) -> std::arch::aarch64::float32x4_t {
        use std::arch::aarch64::*;
        let value = vfmaq_f32(bias, t, factor);
        let source = if premultiplied {
            value
        } else {
            vmulq_f32(value, alpha)
        };
        vminq_f32(
            alpha,
            vmaxq_f32(zero, vfmaq_f32(source, noise, dither_rate)),
        )
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    unsafe fn src_over(
        source: std::arch::aarch64::float32x4_t,
        old: std::arch::aarch64::uint32x4_t,
        inverse: std::arch::aarch64::float32x4_t,
        unorm_reciprocal: std::arch::aarch64::float32x4_t,
    ) -> std::arch::aarch64::float32x4_t {
        use std::arch::aarch64::*;
        vfmaq_f32(
            source,
            vmulq_f32(vcvtq_f32_u32(old), unorm_reciprocal),
            inverse,
        )
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    unsafe fn to_unorm(
        value: std::arch::aarch64::float32x4_t,
        zero: std::arch::aarch64::float32x4_t,
        scale: std::arch::aarch64::float32x4_t,
    ) -> std::arch::aarch64::uint32x4_t {
        use std::arch::aarch64::*;
        vcvtnq_u32_f32(vminq_f32(scale, vmaxq_f32(zero, vmulq_f32(value, scale))))
    }

    #[test]
    fn span_specialization_matches_old_loop_bytes() {
        let palettes = [
            ([1.0, -1.0, 1.0, 0.0], [0.0, 1.0, 0.0, 1.0]),
            ([-1.0, 1.0, -0.999, 0.0], [1.0, 0.0, 0.999, 1.0]),
            (
                [f32::EPSILON, -f32::EPSILON, 0.0, 0.0],
                [0.0, 1.0, 0.5, 1.0],
            ),
            ([0.333, -0.721, 0.615, -0.5], [0.123, 0.9, 0.001, 0.75]),
            ([0.0; 4], [0.0; 4]),
        ];
        for x in 0..8 {
            for y in 0..8 {
                for length in [0, 1, 15, 16, 17, 31, 32, 33, 48, 64, 255, 1027] {
                    for alignment in [0, 1, 3] {
                        for (factor, bias) in palettes {
                            for (a, row_bias) in [
                                (0.0, -1.0),
                                (0.0, 0.0),
                                (0.0, 1.0),
                                (0.0, 2.0),
                                (0.013, -0.1),
                                (-0.125, 1.5),
                            ] {
                                for premultiplied in [false, true] {
                                    for opaque in [false, true] {
                                        let initial: Vec<u8> = (0..length + alignment + 7)
                                            .map(|i| (i as u32 * 73 + x * 11 + y * 19) as u8)
                                            .collect();
                                        let mut expected = initial.clone();
                                        let mut actual = initial;
                                        unsafe {
                                            old_loop_oracle(
                                                &mut expected[alignment..alignment + length],
                                                x,
                                                y,
                                                a,
                                                row_bias,
                                                factor,
                                                bias,
                                                premultiplied,
                                                opaque,
                                            );
                                            shade_two_stop_span(
                                                &mut actual[alignment..alignment + length],
                                                x,
                                                y,
                                                a,
                                                row_bias,
                                                factor,
                                                bias,
                                                premultiplied,
                                                opaque,
                                            );
                                        }
                                        assert_eq!(actual, expected, "x={x} y={y} length={length} alignment={alignment} factor={factor:?} bias={bias:?} a={a} row_bias={row_bias} premultiplied={premultiplied} opaque={opaque}");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn shared_vertical_row_matches_stages_for_tile_phase_and_every_tail() {
        for a in [0.0, -0.0] {
            let context =
                TwoStopSpanContext::new(a, [0.321, -0.754, 0.619, 0.0], [0.179, 0.823, 0.123, 1.0]);
            for y in [0u32, 1, 7, 8, 254, u32::MAX] {
                let mut cached = None;
                for row_bias in [-0.0f32, 0.0, 0.333, 1.0, f32::NAN] {
                    for x in [
                        0u32,
                        1,
                        2,
                        3,
                        4,
                        5,
                        6,
                        7,
                        254,
                        255,
                        256,
                        1 << 24,
                        u32::MAX - 2048,
                    ] {
                        for length in [0usize, 1, 15, 16, 17, 31, 32, 33, 48, 63, 64, 255, 1027] {
                            let mut expected = vec![173; length + 12];
                            let mut actual = expected.clone();
                            unsafe {
                                shade_two_stop_span_stages::<true>(
                                    &mut expected[3..3 + length],
                                    x,
                                    y,
                                    row_bias,
                                    &context,
                                    false,
                                );
                                shade_vertical_row_span(
                                    &mut actual[3..3 + length],
                                    x,
                                    y,
                                    row_bias,
                                    &context,
                                    false,
                                    &mut cached,
                                    false,
                                );
                            }
                            assert_eq!(
                                actual, expected,
                                "x={x},y={y},a={a},bias={row_bias},length={length}"
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn vertical_tail_matches_masked_final_pipeline_lanes() {
        let context =
            TwoStopSpanContext::new(0.0, [0.321, -0.754, 0.619, 0.0], [0.179, 0.823, 0.123, 1.0]);
        for y in 0..8 {
            let mut cached = None;
            for row_bias in [-0.0, 0.0, 0.333, 1.0] {
                for x in 0..8 {
                    for pixels in 0usize..=259 {
                        let len = pixels * 4;
                        let mut padded = vec![137; pixels.div_ceil(4) * 16];
                        let mut actual = vec![137; len + 13];
                        unsafe {
                            shade_two_stop_span_stages::<true>(
                                &mut padded,
                                x,
                                y,
                                row_bias,
                                &context,
                                false,
                            );
                            shade_vertical_row_span(
                                &mut actual[3..3 + len],
                                x,
                                y,
                                row_bias,
                                &context,
                                false,
                                &mut cached,
                                true,
                            );
                        }
                        assert_eq!(&actual[3..3 + len], &padded[..len]);
                        assert!(actual[..3]
                            .iter()
                            .chain(actual[3 + len..].iter())
                            .all(|&b| b == 137));
                    }
                }
            }
        }
    }
    #[test]
    fn vertical_period_preserves_signed_zero_large_coordinates_and_slice_guards() {
        for x in [
            0u32,
            7,
            254,
            255,
            256,
            (1 << 23) - 1,
            1 << 24,
            u32::MAX - 2048,
        ] {
            for y in [0u32, 1, 7, 8, 254, u32::MAX] {
                for length in [31usize, 32, 33, 48, 63, 64, 255, 1027, 2047] {
                    for alignment in [0, 1, 7] {
                        for a in [0.0f32, -0.0] {
                            for row_bias in [-0.0f32, 0.0, 0.333, 1.0, f32::NAN] {
                                let initial: Vec<u8> = (0..length + alignment + 11)
                                    .map(|i| (i as u32).wrapping_mul(73).wrapping_add(x) as u8)
                                    .collect();
                                let mut expected = initial.clone();
                                let mut actual = initial;
                                let factor = [0.321, -0.754, 0.619, 0.0];
                                let bias = [0.179, 0.823, 0.123, 1.0];
                                unsafe {
                                    old_loop_oracle(
                                        &mut expected[alignment..alignment + length],
                                        x,
                                        y,
                                        a,
                                        row_bias,
                                        factor,
                                        bias,
                                        false,
                                        true,
                                    );
                                    shade_two_stop_span(
                                        &mut actual[alignment..alignment + length],
                                        x,
                                        y,
                                        a,
                                        row_bias,
                                        factor,
                                        bias,
                                        false,
                                        true,
                                    );
                                }
                                assert_eq!(
                                    actual, expected,
                                    "x={x} y={y} a={a} bias={row_bias} length={length}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn vertical_tiled_row_matches_original_stage_schedule_and_guards() {
        for format in [
            crate::raster::PixelFormat::Rgba8888,
            crate::raster::PixelFormat::Bgra8888,
            crate::raster::PixelFormat::Bgrx8888,
        ] {
            let mut context = TwoStopSpanContext::new(
                0.0,
                [0.321, -0.754, 0.619, 0.0],
                [0.179, 0.823, 0.123, 1.0],
            );
            context.format = format;
            for bias in [-0.0, 0.0, 0.0001, 0.333, 0.9999, 1.0, 1.25] {
                for y in 0..8 {
                    for premultiplied in [false, true] {
                        for first in [0u32, 1, 7, 253, 254, 255, 256, 508, 509, 999, 2049] {
                            for count in
                                [0usize, 1, 7, 31, 253, 254, 255, 257, 509, 510, 1021, 2560]
                            {
                                let mut expected = vec![173; count * 4 + 11];
                                let mut actual = expected.clone();
                                let mut x = first;
                                let end = first + count as u32;
                                let mut cached = None;
                                while x < end {
                                    let shift = tile_shift(x);
                                    let boundary = if x < 255 { 255 } else { shift + 255 };
                                    let next = end.min(boundary);
                                    let at = (x - first) as usize * 4;
                                    unsafe {
                                        shade_vertical_row_span(
                                            &mut expected[3 + at..3 + at + (next - x) as usize * 4],
                                            x - shift,
                                            y,
                                            bias,
                                            &context,
                                            premultiplied,
                                            &mut cached,
                                            true,
                                        );
                                    }
                                    x = next;
                                }
                                unsafe {
                                    shade_vertical_tiled_row(
                                        &mut actual[3..3 + count * 4],
                                        first,
                                        y,
                                        bias,
                                        &context,
                                        premultiplied,
                                    );
                                }
                                assert_eq!(actual,expected,"format={format:?} first={first} count={count} bias={bias} y={y} premul={premultiplied}");
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod multistop_row_profile {
    use super::*;
    use crate::compat::commands::{Color, Offset, PaintColorStop};

    #[test]
    #[ignore]
    fn baidu_multistop_diagonal_kernel_profile() {
        let rect = PaintRect {
            x: 112.0,
            y: 185.0,
            width: 800.0,
            height: 99.0,
        };
        let shader = PaintShader {
            start: Offset {
                x: 449.2998962402344,
                y: 62.23286437988281,
            },
            end: Offset {
                x: 574.7001037597656,
                y: 406.7671356201172,
            },
            stops: [
                (51.0, 119.0, 254.0),
                (76.0, 111.0, 255.0),
                (131.0, 112.0, 255.0),
                (186.0, 89.0, 255.0),
            ]
            .into_iter()
            .enumerate()
            .map(|(i, (r, g, b))| PaintColorStop {
                offset: i as f64 / 3.0,
                color: Color {
                    red: r / 255.0,
                    green: g / 255.0,
                    blue: b / 255.0,
                    alpha: 1.0,
                },
                ..Default::default()
            })
            .collect(),
            interpolate_premultiplied: true,
            ..Default::default()
        };
        let (rect, shader) = match std::env::var("GRADIENT_KERNEL_CASE").as_deref() {
            Ok("vertical") => (
                PaintRect {
                    x: 113.0,
                    y: 186.0,
                    width: 798.0,
                    height: 97.0,
                },
                PaintShader {
                    start: Offset { x: 512.0, y: 186.0 },
                    end: Offset { x: 512.0, y: 283.0 },
                    stops: [0.0, 1.0]
                        .into_iter()
                        .map(|offset| PaintColorStop {
                            offset,
                            color: Color {
                                red: 1.0,
                                green: 1.0,
                                blue: 1.0,
                                alpha: 1.0,
                            },
                            ..Default::default()
                        })
                        .collect(),
                    interpolate_premultiplied: true,
                    ..Default::default()
                },
            ),
            Ok("small") => (
                PaintRect {
                    x: 794.0,
                    y: 230.0,
                    width: 108.0,
                    height: 44.0,
                },
                PaintShader {
                    start: Offset {
                        x: 810.9489860534668,
                        y: 213.63255310058594,
                    },
                    end: Offset {
                        x: 885.0510063171387,
                        y: 290.36744689941406,
                    },
                    stops: [
                        (40.0, 106.0, 255.0),
                        (78.0, 110.0, 242.0),
                        (114.0, 116.0, 249.0),
                        (159.0, 102.0, 255.0),
                    ]
                    .into_iter()
                    .enumerate()
                    .map(|(i, (r, g, b))| PaintColorStop {
                        offset: i as f64 / 3.0,
                        color: Color {
                            red: r / 255.0,
                            green: g / 255.0,
                            blue: b / 255.0,
                            alpha: 1.0,
                        },
                        ..Default::default()
                    })
                    .collect(),
                    interpolate_premultiplied: true,
                    ..Default::default()
                },
            ),
            _ => (rect, shader),
        };
        let mut pixmap = Pixmap::new(2048, 600).unwrap();
        let mask = std::env::var_os("GRADIENT_KERNEL_MASK").map(|_| {
            let mut mask = Mask::new(2048, 600).unwrap();
            mask.data_mut().fill(255);
            mask
        });
        let mut elapsed = Vec::new();
        for _ in 0..std::env::var("GRADIENT_KERNEL_ITERATIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(12)
        {
            let start = std::time::Instant::now();
            assert!(draw_linear_gradient(
                &mut pixmap,
                mask.as_ref(),
                Transform::from_scale(2.0, 2.0),
                &shader,
                rect,
                rect,
                (0, 0),
                (0, 0),
                true,
                None
            ));
            elapsed.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        let mut scalar = Pixmap::new(2048, 600).unwrap();
        assert!(draw_linear_gradient_impl(
            &mut scalar,
            mask.as_ref(),
            Transform::from_scale(2.0, 2.0),
            &shader,
            rect,
            rect,
            (0, 0),
            (0, 0),
            true,
            None,
            false,
        ));
        assert_eq!(pixmap.data(), scalar.data());
        elapsed.sort_by(f64::total_cmp);
        eprintln!(
            "BAIDU_DIAGONAL_KERNEL_MS median={:.3} samples={elapsed:?}",
            elapsed[elapsed.len() / 2]
        );
        if let Ok(path) = std::env::var("GRADIENT_KERNEL_PIXELS") {
            std::fs::write(path, pixmap.data()).unwrap();
        }
    }
}

#[cfg(test)]
mod multistop_segment_regressions {
    use super::*;
    use crate::compat::commands::{Color, Offset, PaintColorStop};

    #[test]
    fn segmented_rows_match_original_scalar_lookup_and_stage_bytes() {
        for format in [
            crate::PixelFormat::Rgba8888,
            crate::PixelFormat::Bgra8888,
            crate::PixelFormat::Bgrx8888,
        ] {
            for scale in [1.0, 2.0] {
                for opaque in [false, true] {
                    for offsets in [
                        [0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0],
                        [0.0, 0.23, 0.66, 1.0],
                        [0.0, 0.37, 0.37, 1.0],
                    ] {
                        for premultiplied in [false, true] {
                            for origin in [(0, 0), (247, 245), (-17, -11)] {
                                for direct in [true, false] {
                                    for palette in [
                                        [
                                            (0.19, 0.47, 0.97, 0.27),
                                            (0.79, 0.24, 0.68, 0.81),
                                            (0.34, 0.81, 0.26, 0.43),
                                            (0.68, 0.37, 0.89, 0.66),
                                        ],
                                        [
                                            (
                                                51.0 / 255.0,
                                                119.0 / 255.0,
                                                254.0 / 255.0,
                                                69.0 / 255.0,
                                            ),
                                            (
                                                76.0 / 255.0,
                                                111.0 / 255.0,
                                                255.0 / 255.0,
                                                207.0 / 255.0,
                                            ),
                                            (
                                                131.0 / 255.0,
                                                112.0 / 255.0,
                                                255.0 / 255.0,
                                                109.0 / 255.0,
                                            ),
                                            (
                                                186.0 / 255.0,
                                                89.0 / 255.0,
                                                255.0 / 255.0,
                                                168.0 / 255.0,
                                            ),
                                        ],
                                    ] {
                                        for (dx, dy) in [(58.0, 51.0), (-57.0, 51.0), (58.0, -51.0)]
                                        {
                                            for masked in [false, true] {
                                                let rect = PaintRect {
                                                    x: 247.0,
                                                    y: 245.0,
                                                    width: 65.0,
                                                    height: 57.0,
                                                };
                                                let shader=PaintShader {
                                        start:Offset { x:if dx>0.0 {251.0} else {308.0},y:if dy>0.0 {248.0} else {299.0} },
                                        end:Offset { x:if dx>0.0 {309.0} else {251.0},y:if dy>0.0 {299.0} else {248.0} },
                                        interpolate_premultiplied:premultiplied,
                                        stops:palette.into_iter().zip(offsets).map(|((red,green,blue,alpha),offset)|PaintColorStop {
                                            offset,color:Color {red,green,blue,alpha:if opaque {1.0} else {alpha}},..Default::default()
                                        }).collect(),..Default::default()
                                    };
                                                let dim = (320.0 * scale) as u32;
                                                let mut actual = Pixmap::new(dim, dim).unwrap();
                                                actual.format = format;
                                                actual.data_mut().fill(117);
                                                let mut expected = actual.clone();
                                                let mask = masked.then(|| {
                                                    let mut mask = Mask::new(dim, dim).unwrap();
                                                    for (index, c) in
                                                        mask.data_mut().iter_mut().enumerate()
                                                    {
                                                        let x = index % dim as usize;
                                                        *c = if x % 37 == 0 {
                                                            0
                                                        } else if x % 19 == 0 {
                                                            149
                                                        } else {
                                                            255
                                                        };
                                                    }
                                                    mask
                                                });
                                                let canvas = Transform::from_scale(
                                                    scale as f32,
                                                    scale as f32,
                                                )
                                                .post_translate(-origin.0 as f32, -origin.1 as f32);
                                                assert!(draw_linear_gradient_impl(
                                                    &mut actual,
                                                    mask.as_ref(),
                                                    canvas,
                                                    &shader,
                                                    rect,
                                                    rect,
                                                    (0, 0),
                                                    origin,
                                                    direct,
                                                    None,
                                                    true
                                                ));
                                                assert!(draw_linear_gradient_impl(
                                                    &mut expected,
                                                    mask.as_ref(),
                                                    canvas,
                                                    &shader,
                                                    rect,
                                                    rect,
                                                    (0, 0),
                                                    origin,
                                                    direct,
                                                    None,
                                                    false
                                                ));
                                                let first = actual
                                                    .data()
                                                    .iter()
                                                    .zip(expected.data())
                                                    .position(|(a, b)| a != b);
                                                assert!(first.is_none(),"first={first:?} format={format:?}, scale={scale}, opaque={opaque}, offsets={offsets:?}, premul={premultiplied}, direction={dx},{dy}, masked={masked}, origin={origin:?}, direct={direct}");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
