//! Rust replay adapter combining several official kernel responsibilities.
//! Function-level algorithm correspondences are recorded in translation_map.json.

// CPU implementation moved from renderer/image_sampling.rs.

//! Skia N32 premultiplied image minification, implemented in Rust.
//! The source resource stays RGBA; the filters act independently on each
//! premultiplied channel, so Skia's native BGRA byte order is immaterial.

use crate::compat::commands::PaintRect;
use crate::raster::Pixmap;
use crate::raster::{Mask, Transform};

/// SkBitmapProcState's four-bit bilerp, strict source subset, and N32
/// src-over under scale+translation. Like SkScan::FillRect, image geometry
/// uses rounded integer device bounds; the default drawImageRect paint has no AA.
pub(crate) fn draw_image_bitmap(
    output: &mut Pixmap,
    clip: Option<&Mask>,
    canvas: Transform,
    image: &Pixmap,
    destination: PaintRect,
    source: PaintRect,
    device_origin: (u32, u32),
    tile_origin: (i32, i32),
    clip_runs: Option<&[usize]>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
) -> bool {
    draw_image_bitmap_opaque(
        output,
        clip,
        canvas,
        image,
        destination,
        source,
        device_origin,
        tile_origin,
        clip_runs,
        clip_summary,
        false,
    )
}

// An authenticated all-255 alpha source remains opaque after strict subset
// sampling and the mip filters: each complete filter's weights sum exactly to
// its power-of-two denominator. Full coverage therefore needs only a store.
pub(crate) fn draw_image_bitmap_opaque(
    output: &mut Pixmap,
    clip: Option<&Mask>,
    canvas: Transform,
    image: &Pixmap,
    destination: PaintRect,
    source: PaintRect,
    device_origin: (u32, u32),
    tile_origin: (i32, i32),
    clip_runs: Option<&[usize]>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    known_opaque: bool,
) -> bool {
    if canvas.sx <= 0.0
        || canvas.sy <= 0.0
        || !canvas.sx.is_finite()
        || !canvas.sy.is_finite()
        || canvas.kx != 0.0
        || canvas.ky != 0.0
    {
        return false;
    }
    // SkBitmapProcState's scale+translate matrix is expressed in device
    // pixels; a high-DPI canvas still uses the same four-bit bilerp stages.
    let left = destination.x as f32 * canvas.sx + canvas.tx;
    let top = destination.y as f32 * canvas.sy + canvas.ty;
    let right = left + destination.width as f32 * canvas.sx;
    let bottom = top + destination.height as f32 * canvas.sy;
    if destination.width <= 0.0
        || destination.height <= 0.0
        || [left, top, right, bottom].iter().any(|v| !v.is_finite())
    {
        return false;
    }
    // SkScan::FillRect (BW image paint) calls SkRect::round, not roundOut.
    // Retain raw mapped bounds for the shader; round only raster geometry.
    let round = |v: f32| (v + 0.5).floor();
    let mut x_start = round(left).max(0.0) as u32;
    let mut x_end = round(right).min(output.width() as f32).max(0.0) as u32;
    let mut y_start = round(top).max(0.0) as u32;
    let mut y_end = round(bottom).min(output.height() as f32).max(0.0) as u32;
    if let Some(summary) = clip_summary {
        let Some(bounds) = summary.getBounds() else {
            return true;
        };
        x_start = x_start.max(bounds.left() as u32);
        x_end = x_end.min(bounds.right() as u32);
        y_start = y_start.max(bounds.top() as u32);
        y_end = y_end.min(bounds.bottom() as u32);
    }
    if let Some(mask) = clip {
        let bounds = mask.storage_bounds();
        x_start = x_start.max(bounds.left() as u32);
        x_end = x_end.min(bounds.right() as u32);
        y_start = y_start.max(bounds.top() as u32);
        y_end = y_end.min(bounds.bottom() as u32);
    }
    let clip = clip.filter(|_| !clip_summary.is_some_and(|c| c.isRect()));
    // SkBitmapDevice::drawImageRect extracts the rounded-out source subset
    // before building its clamp shader (kStrict_SrcRectConstraint).
    let src_left = source.x as f32;
    let src_top = source.y as f32;
    let src_right = src_left + source.width as f32;
    let src_bottom = src_top + source.height as f32;
    if src_left < 0.0
        || src_top < 0.0
        || src_right > image.width() as f32
        || src_bottom > image.height() as f32
        || ![src_left, src_top, src_right, src_bottom]
            .iter()
            .all(|v| v.is_finite())
        || src_right <= src_left
        || src_bottom <= src_top
    {
        return false;
    }
    if x_start >= x_end || y_start >= y_end {
        return true;
    }
    let ox = src_left.floor() as u32;
    let oy = src_top.floor() as u32;
    // SkBitmap::extractSubset shares its pixel ref and retains the parent
    // rowBytes. Borrow the same rounded-out ROI rather than allocating and
    // copying it for each draw. Shader coordinates remain subset-local.
    let image = BitmapSubset {
        data: image.data(),
        stride: image.width() as usize,
        left: ox,
        top: oy,
        width: src_right.ceil() as u32 - ox,
        height: src_bottom.ceil() as u32 - oy,
    };
    let local_scale_x = ((destination.x as f32 + destination.width as f32) - destination.x as f32)
        / (src_right - src_left);
    let local_scale_y = ((destination.y as f32 + destination.height as f32) - destination.y as f32)
        / (src_bottom - src_top);
    let scale_x = local_scale_x * canvas.sx;
    let scale_y = local_scale_y * canvas.sy;
    let inv_x = 1.0 / scale_x;
    let inv_y = 1.0 / scale_y;
    let shader_tx = (destination.x as f32 - src_left * local_scale_x + ox as f32 * local_scale_x)
        * canvas.sx
        + canvas.tx;
    let shader_ty = (destination.y as f32 - src_top * local_scale_y + oy as f32 * local_scale_y)
        * canvas.sy
        + canvas.ty;
    if scale_x == 1.0
        && scale_y == 1.0
        && shader_tx.is_finite()
        && shader_ty.is_finite()
        && shader_tx.fract() == 0.0
        && shader_ty.fract() == 0.0
        && draw_image_integer_translate(
            output,
            clip,
            &image,
            (x_start, y_start, x_end, y_end),
            (shader_tx, shader_ty),
            tile_origin,
            clip_runs,
            known_opaque,
        )
    {
        return true;
    }
    let fixed = |v: f32| (f64::from(v) * 4_294_967_296.0) as i64;
    let step = fixed(inv_x);
    let format = output.format;
    let output_width = output.width() as usize;
    let output_data = output.data_mut();
    let rows = BitmapDrawRows {
        image,
        clip,
        clip_runs,
        x_start,
        x_end,
        output_width,
        format,
        inv_x,
        inv_y,
        shader_tx,
        shader_ty,
        step,
        device_origin,
        tile_origin,
        known_opaque,
    };
    let area = u64::from(x_end - x_start) * u64::from(y_end - y_start);
    let workers = if area >= 512 * 1024 && y_end - y_start >= 64 {
        static PARALLELISM: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        let available = *PARALLELISM
            .get_or_init(|| std::thread::available_parallelism().map_or(1, |count| count.get()));
        available.min(if area >= 2 * 1024 * 1024 { 4 } else { 2 })
    } else {
        1
    };
    rows.draw_partitioned(output_data, y_start, y_end, workers);
    true
}

#[derive(Clone, Copy)]
struct BitmapDrawRows<'a> {
    image: BitmapSubset<'a>,
    clip: Option<&'a Mask>,
    clip_runs: Option<&'a [usize]>,
    x_start: u32,
    x_end: u32,
    output_width: usize,
    format: crate::raster::PixelFormat,
    inv_x: f32,
    inv_y: f32,
    shader_tx: f32,
    shader_ty: f32,
    step: i64,
    device_origin: (u32, u32),
    tile_origin: (i32, i32),
    known_opaque: bool,
}
impl BitmapDrawRows<'_> {
    fn draw_partitioned(&self, output: &mut [u8], y_start: u32, y_end: u32, workers: usize) {
        let height = (y_end - y_start) as usize;
        if height == 0 {
            return;
        }
        let row_bytes = self.output_width * 4;
        let output = &mut output[y_start as usize * row_bytes..y_end as usize * row_bytes];
        let workers = workers.max(1).min(height);
        if workers == 1 {
            self.draw(output, y_start, y_end);
            return;
        }
        let rows_per_worker = height.div_ceil(workers);
        // Scoped borrows partition complete, disjoint destination rows. Every
        // worker retains global coordinates and the original per-row shader
        // restarts. Spawn/panic errors propagate; never reblend partial output.
        std::thread::scope(|scope| {
            let mut chunks = output.chunks_mut(rows_per_worker * row_bytes);
            let first = chunks.next().expect("nonempty image rows");
            for (index, chunk) in chunks.enumerate() {
                let start = y_start + ((index + 1) * rows_per_worker) as u32;
                let end = start + (chunk.len() / row_bytes) as u32;
                scope.spawn(move || self.draw(chunk, start, end));
            }
            let first_end = y_start + (first.len() / row_bytes) as u32;
            self.draw(first, y_start, first_end);
        });
    }
    fn draw(&self, output_data: &mut [u8], y_start: u32, y_end: u32) {
        let Self {
            image,
            clip,
            clip_runs,
            x_start,
            x_end,
            output_width,
            format,
            inv_x,
            inv_y,
            shader_tx,
            shader_ty,
            step,
            device_origin,
            tile_origin,
            known_opaque,
        } = *self;
        let row_base = y_start as usize * output_width;
        let fixed = |v: f32| (f64::from(v) * 4_294_967_296.0) as i64;
        // SkBitmapProcState uses a bounded coordinate buffer and shades whole
        // homogeneous clip runs. Keep the existing 127-pixel/tile schedule and
        // four-bit sampling exactly, then feed the byte-domain SrcOver row kernel.
        let mut sampled = [0_u8; 127 * 4];
        let mut run_cursor = ImageClipRunCursor::new("bilerp");
        for y in y_start..y_end {
            let clip_row = clip.map(|mask| mask.row_range(y, x_start, x_end));
            let shift_y =
                ((crate::cpu::raster_pipeline::tile_shift((y as i32 + tile_origin.1).max(0) as u32)
                    as i32
                    - tile_origin.1)
                    .max(device_origin.1 as i32)) as f32;
            let fy = fixed((y as f32 - shift_y + 0.5) * inv_y - (shader_ty - shift_y) * inv_y)
                - (1_i64 << 31);
            let source_rows = BitmapSampleRows::new_subset(&image, fy);
            let mut x = x_start;
            while x < x_end {
                let index = y as usize * output_width + x as usize;
                let output_index = index - row_base;
                let coverage = clip_row.map_or(255, |m| m[(x - x_start) as usize]);
                if coverage == 0 {
                    x += 1;
                    continue;
                }
                let shift_x = ((crate::cpu::raster_pipeline::tile_shift(
                    (x as i32 + tile_origin.0).max(0) as u32,
                ) as i32
                    - tile_origin.0)
                    .max(device_origin.0 as i32));
                let fx = fixed(
                    (x as f32 - shift_x as f32 + 0.5) * inv_x
                        - (shader_tx - shift_x as f32) * inv_x,
                ) - (1_i64 << 31);
                let run_end = clip_runs
                    .and_then(|runs| run_cursor.next_after(runs, index))
                    .unwrap_or(usize::MAX);
                // SkBitmapProcState shades homogeneous spans with a bounded
                // coordinate buffer. Derive the existing tile/run restart points
                // once, rather than asking the same tile/clip question per pixel.
                // The device-origin clamp can merge several global tiles; the next
                // restart is the first 254px tile shift strictly above that clamp.
                let global_shift = i64::from(shift_x) + i64::from(tile_origin.0);
                let next_tile =
                    (global_shift.div_euclid(254) + 1) * 254 + 1 - i64::from(tile_origin.0);
                let mut count = (x_end - x).min(127) as usize;
                count = count.min((next_tile - i64::from(x)).max(1) as usize);
                count = count.min(run_end - index);
                if let Some(mask) = clip_row {
                    let start = (x - x_start) as usize;
                    count = crate::cpu::analytic_aa::equal_byte_run_end(
                        &mask[start..start + count],
                        0,
                        coverage,
                    );
                }
                x += count as u32;
                // Full-coverage authenticated opaque shading needs no destination
                // blend. Write the unchanged 4-bit filter result directly in the
                // device layout instead of storing and rereading the scratch row.
                if known_opaque
                    && coverage == 255
                    && source_rows.try_sample_opaque_row(
                        fx,
                        step,
                        &mut output_data[output_index * 4..(output_index + count) * 4],
                        format,
                    )
                {
                    continue;
                }
                source_rows.sample_row(fx, step, &mut sampled[..count * 4]);
                image_src_over_row_with_opacity(
                    &mut output_data[output_index * 4..(output_index + count) * 4],
                    &sampled[..count * 4],
                    coverage,
                    format,
                    known_opaque,
                );
            }
        }
    }
}

// SkBitmapProcState::init disables bilerp for integral_translate_only;
// chooseShaderProc32 selects Clamp_S32_D32_nofilter_trans_shaderproc. For this
// exact subset the mapper advances by one integer source pixel regardless of
// run/tile boundaries, so complete source rows can feed the SrcOver row kernel.
fn draw_image_integer_translate(
    output: &mut Pixmap,
    clip: Option<&Mask>,
    image: &BitmapSubset<'_>,
    bounds: (u32, u32, u32, u32),
    translation: (f32, f32),
    tile_origin: (i32, i32),
    clip_runs: Option<&[usize]>,
    known_opaque: bool,
) -> bool {
    let (x0, y0, x1, y1) = bounds;
    if x1 <= x0 || y1 <= y0 {
        return true;
    }
    let sx = i64::from(x0) - translation.0 as i64;
    let sy = i64::from(y0) - translation.1 as i64;
    let width = (x1 - x0) as usize;
    let height = (y1 - y0) as usize;
    // The strict source subset must contain every pixel in these runs. Leave
    // uncommon clamped source extensions to the existing mapper without writes.
    if sx < 0
        || sy < 0
        || sx + width as i64 > i64::from(image.width)
        || sy + height as i64 > i64::from(image.height)
    {
        return false;
    }
    let output_width = output.width() as usize;
    let format = output.format;
    let pixels = output.data_mut();
    let mut run_cursor = ImageClipRunCursor::new("integer");
    for row in 0..height {
        let clip_row = clip.map(|mask| mask.row_range(y0 + row as u32, x0, x0 + width as u32));
        let destination_index = (y0 as usize + row) * output_width + x0 as usize;
        let source = image.row(sy as u32 + row as u32);
        let source_index = sx as usize;
        let dst = &mut pixels[destination_index * 4..(destination_index + width) * 4];
        let src = &source[source_index * 4..(source_index + width) * 4];
        let mut start = 0;
        while start < width {
            let coverage = clip_row.map_or(255, |mask| mask[start]);
            let global_x = (i64::from(x0) + start as i64 + i64::from(tile_origin.0)).max(0) as u32;
            let tile_end = crate::cpu::raster_pipeline::tile_shift(global_x) + 255;
            let next_run = clip_runs
                .and_then(|runs| run_cursor.next_after(runs, destination_index + start))
                .map_or(width, |i| (i - destination_index).min(width));
            let limit = width
                .min(start + 127)
                .min(start + (tile_end - global_x) as usize)
                .min(next_run);
            let mut end = start + 1;
            if clip.is_some() {
                while end < limit && clip_row.is_none_or(|mask| mask[end] == coverage) {
                    end += 1;
                }
            } else {
                end = limit;
            }
            if coverage != 0 {
                image_src_over_row_with_opacity(
                    &mut dst[start * 4..end * 4],
                    &src[start * 4..end * 4],
                    coverage,
                    format,
                    known_opaque,
                );
            }
            start = end;
        }
    }
    true
}

// Local index driver for sorted SkAAClip restart boundaries. Every shaded
// span advances in device order; restart the lower-bound search only when a
// query moves backwards. This changes lookup alone, not AA / 127px / tile
// boundaries, mapper phase or four-bit interpolation.
struct ImageClipRunCursor {
    next: usize,
    last: Option<usize>,
    #[cfg(feature = "profiling")]
    binary_search: bool,
    #[cfg(feature = "profiling")]
    trace: Option<(std::time::Instant, &'static str, usize, usize, usize)>,
}
impl ImageClipRunCursor {
    fn new(mode: &'static str) -> Self {
        #[cfg(not(feature = "profiling"))]
        let _ = mode;
        Self {
            next: 0,
            last: None,
            #[cfg(feature = "profiling")]
            binary_search: std::env::var_os("SKIA_IMAGE_BINARY_SEARCH").is_some(),
            #[cfg(feature = "profiling")]
            trace: std::env::var_os("SKIA_TRACE_IMAGE")
                .is_some()
                .then(|| (std::time::Instant::now(), mode, 0, 0, 0)),
        }
    }
    fn next_after(&mut self, runs: &[usize], index: usize) -> Option<usize> {
        #[cfg(feature = "profiling")]
        {
            if let Some((_, _, queries, _, _)) = &mut self.trace {
                *queries += 1;
            }
            if self.binary_search {
                if let Some((_, _, _, searches, _)) = &mut self.trace {
                    *searches += 1;
                }
                return runs.get(runs.partition_point(|&i| i <= index)).copied();
            }
        }
        if self.last.is_none_or(|last| index < last) {
            self.next = runs.partition_point(|&i| i <= index);
            #[cfg(feature = "profiling")]
            if let Some((_, _, _, searches, _)) = &mut self.trace {
                *searches += 1;
            }
        } else {
            while self.next < runs.len() && runs[self.next] <= index {
                self.next += 1;
                #[cfg(feature = "profiling")]
                if let Some((_, _, _, _, advanced)) = &mut self.trace {
                    *advanced += 1;
                }
            }
        }
        self.last = Some(index);
        runs.get(self.next).copied()
    }
}
#[cfg(feature = "profiling")]
impl Drop for ImageClipRunCursor {
    fn drop(&mut self) {
        if let Some((start, mode, queries, searches, advanced)) = self.trace {
            eprintln!("image-sampling mode={mode} queries={queries} searches={searches} advanced={advanced} sample_ms={:.6}", start.elapsed().as_secs_f64()*1000.);
        }
    }
}

// Opaque shader proc: a full-coverage SrcOver operation has zero destination
// coefficient. Leave partial coverage and unknown-alpha images on the original
// arithmetic path. This stores newly sampled pixels, not cached frame pixels.
fn image_src_over_row_with_opacity(
    dst: &mut [u8],
    src: &[u8],
    coverage: u8,
    format: crate::raster::PixelFormat,
    known_opaque: bool,
) {
    if !known_opaque || coverage != 255 {
        image_src_over_row(dst, src, coverage, format);
        return;
    }
    debug_assert_eq!(dst.len(), src.len());
    debug_assert_eq!(src.len() % 4, 0);
    if format == crate::raster::PixelFormat::Rgba8888 {
        dst.copy_from_slice(src);
        return;
    }
    let mut offset = 0;
    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    {
        let blocks = src.len() / 16;
        if blocks != 0 {
            let shuffle = if format == crate::raster::PixelFormat::Bgrx8888 {
                [2u8, 1, 0, 16, 6, 5, 4, 16, 10, 9, 8, 16, 14, 13, 12, 16]
            } else {
                [2u8, 1, 0, 3, 6, 5, 4, 7, 10, 9, 8, 11, 14, 13, 12, 15]
            };
            // Four complete sampled pixels per group. Out-of-table indices
            // write zero only to BGRX's unused byte; no destination read occurs.
            unsafe {
                core::arch::asm!(
                    "ldr q1, [{shuffle}]",
                    "2:",
                    "ldr q0, [{src}], #16",
                    "tbl v0.16b, {{v0.16b}}, v1.16b",
                    "str q0, [{dst}], #16",
                    "subs {blocks}, {blocks}, #1",
                    "b.ne 2b",
                    shuffle = in(reg) shuffle.as_ptr(),
                    src = inout(reg) src.as_ptr() => _,
                    dst = inout(reg) dst.as_mut_ptr() => _,
                    blocks = inout(reg) blocks => _,
                    out("v0") _, out("v1") _, options(nostack),
                );
            }
            offset = blocks * 16;
        }
    }
    for (d, s) in dst[offset..]
        .chunks_exact_mut(4)
        .zip(src[offset..].chunks_exact(4))
    {
        d.copy_from_slice(&format.encode(s.try_into().unwrap()));
    }
}

// The current image shader's exact SrcOver row arithmetic. Full coverage
// translates LOWP_STAGE_PP(srcover_rgba_8888) / NEON div255_round; partial AA
// translates the combined source/global-alpha products of SkBlitRow's NEON
// bitmap shader blitter. Color layout is applied only at the store adapter.
fn image_src_over_row(
    dst: &mut [u8],
    src: &[u8],
    coverage: u8,
    format: crate::raster::PixelFormat,
) {
    debug_assert_eq!(dst.len(), src.len());
    debug_assert_eq!(src.len() % 4, 0);
    if coverage == 0 {
        return;
    }
    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    let offset = unsafe { image_src_over_row_neon(dst, src, coverage, format) };
    #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
    let offset = 0;
    if offset == src.len() {
        return;
    }
    for (d, s) in dst[offset..]
        .chunks_exact_mut(4)
        .zip(src[offset..].chunks_exact(4))
    {
        let s = format.swizzle(s.try_into().unwrap());
        if coverage == 255 {
            for c in 0..4 {
                let product = u32::from(d[c]) * (255 - u32::from(s[3]));
                let p = product + 128;
                d[c] = (u32::from(s[c]) + ((p + (p >> 8)) >> 8)) as u8;
            }
        } else {
            let scale = u32::from(coverage) + 1;
            let inverse = if s[3] == 255 {
                256 - scale
            } else {
                let p = 65535 - u32::from(s[3]) * scale;
                (p + (p >> 8)) >> 8
            };
            for c in 0..4 {
                d[c] = ((u32::from(s[c]) * scale + u32::from(d[c]) * inverse) >> 8).min(255) as u8;
            }
        }
        if format == crate::raster::PixelFormat::Bgrx8888 {
            d[3] = 0;
        }
    }
}

// Direct instruction translation of the preceding row's NEON arithmetic.
// A bounded eight-pixel loop avoids Debug ABI calls through stdarch wrappers;
// sampling coordinates, clip runs, and source ROI remain in the caller.
#[cfg(all(feature = "simd", target_arch = "aarch64"))]
unsafe fn image_src_over_row_neon(
    dst: &mut [u8],
    src: &[u8],
    coverage: u8,
    format: crate::raster::PixelFormat,
) -> usize {
    let blocks = src.len() / 32;
    if blocks == 0 {
        image_src_over_tail_neon(dst, src, coverage, format);
        return src.len();
    }
    macro_rules! run {
        ($setup:literal, $swizzle:literal, $inverse:literal, $colors:literal, $alpha:literal) => {
            core::arch::asm!(
                "// scale {scale:w}",
                $setup,
                "2:",
                "ld4 {{v0.8b, v1.8b, v2.8b, v3.8b}}, [{src}], #32",
                "ld4 {{v4.8b, v5.8b, v6.8b, v7.8b}}, [{dst}]",
                $swizzle,
                $inverse,
                $colors,
                $alpha,
                "st4 {{v4.8b, v5.8b, v6.8b, v7.8b}}, [{dst}], #32",
                "subs {blocks}, {blocks}, #1",
                "b.ne 2b",
                src = inout(reg) src.as_ptr() => _,
                dst = inout(reg) dst.as_mut_ptr() => _,
                blocks = inout(reg) blocks => _,
                scale = in(reg) u32::from(coverage) + 1,
                out("v0") _, out("v1") _, out("v2") _, out("v3") _,
                out("v4") _, out("v5") _, out("v6") _, out("v7") _,
                out("v16") _, out("v17") _, out("v18") _, out("v19") _,
                out("v20") _, out("v21") _, out("v22") _, out("v23") _,
                options(nostack),
            );
        };
    }
    macro_rules! full {
        ($swizzle:literal, $alpha:literal) => {
            run!(
                "movi v20.8b, #255",
                $swizzle,
                "sub v17.8b, v20.8b, v3.8b",
                "umull v16.8h, v4.8b, v17.8b\n\
                 urshr v18.8h, v16.8h, #8\n\
                 raddhn v4.8b, v16.8h, v18.8h\n\
                 add v4.8b, v4.8b, v0.8b\n\
                 umull v16.8h, v5.8b, v17.8b\n\
                 urshr v18.8h, v16.8h, #8\n\
                 raddhn v5.8b, v16.8h, v18.8h\n\
                 add v5.8b, v5.8b, v1.8b\n\
                 umull v16.8h, v6.8b, v17.8b\n\
                 urshr v18.8h, v16.8h, #8\n\
                 raddhn v6.8b, v16.8h, v18.8h\n\
                 add v6.8b, v6.8b, v2.8b",
                $alpha
            );
        };
    }
    macro_rules! partial {
        ($swizzle:literal, $alpha:literal) => {
            run!(
                "dup v20.8h, {scale:w}\n\
                 movi v21.8h, #255\n\
                 movi v22.8h, #1, lsl #8\n\
                 movi v23.8h, #1\n\
                 orr v23.8h, #1, lsl #8",
                $swizzle,
                "ushll v17.8h, v3.8b, #0\n\
                 mul v18.8h, v17.8h, v20.8h\n\
                 ushr v19.8h, v18.8h, #8\n\
                 add v19.8h, v19.8h, v18.8h\n\
                 add v19.8h, v19.8h, v23.8h\n\
                 ushr v19.8h, v19.8h, #8\n\
                 sub v19.8h, v23.8h, v19.8h\n\
                 cmeq v17.8h, v17.8h, v21.8h\n\
                 sub v18.8h, v22.8h, v20.8h\n\
                 bsl v17.16b, v18.16b, v19.16b",
                "ushll v18.8h, v0.8b, #0\n\
                 mul v18.8h, v18.8h, v20.8h\n\
                 ushll v19.8h, v4.8b, #0\n\
                 mla v18.8h, v19.8h, v17.8h\n\
                 shrn v4.8b, v18.8h, #8\n\
                 ushll v18.8h, v1.8b, #0\n\
                 mul v18.8h, v18.8h, v20.8h\n\
                 ushll v19.8h, v5.8b, #0\n\
                 mla v18.8h, v19.8h, v17.8h\n\
                 shrn v5.8b, v18.8h, #8\n\
                 ushll v18.8h, v2.8b, #0\n\
                 mul v18.8h, v18.8h, v20.8h\n\
                 ushll v19.8h, v6.8b, #0\n\
                 mla v18.8h, v19.8h, v17.8h\n\
                 shrn v6.8b, v18.8h, #8",
                $alpha
            );
        };
    }
    macro_rules! alpha {
        ($swizzle:literal, $zero:literal) => {
            if coverage == 255 {
                if $zero {
                    full!($swizzle, "movi v7.8b, #0");
                } else {
                    full!(
                        $swizzle,
                        "umull v16.8h, v7.8b, v17.8b\n\
                         urshr v18.8h, v16.8h, #8\n\
                         raddhn v7.8b, v16.8h, v18.8h\n\
                         add v7.8b, v7.8b, v3.8b"
                    );
                }
            } else if $zero {
                partial!($swizzle, "movi v7.8b, #0");
            } else {
                partial!(
                    $swizzle,
                    "ushll v18.8h, v3.8b, #0\n\
                     mul v18.8h, v18.8h, v20.8h\n\
                     ushll v19.8h, v7.8b, #0\n\
                     mla v18.8h, v19.8h, v17.8h\n\
                     shrn v7.8b, v18.8h, #8"
                );
            }
        };
    }
    if format == crate::raster::PixelFormat::Rgba8888 {
        alpha!("", false);
    } else if format == crate::raster::PixelFormat::Bgrx8888 {
        alpha!(
            "mov v16.8b, v0.8b\nmov v0.8b, v2.8b\nmov v2.8b, v16.8b",
            true
        );
    } else {
        alpha!(
            "mov v16.8b, v0.8b\nmov v0.8b, v2.8b\nmov v2.8b, v16.8b",
            false
        );
    }
    image_src_over_tail_neon(
        &mut dst[blocks * 32..],
        &src[blocks * 32..],
        coverage,
        format,
    );
    src.len()
}

// The same channel-wise kernel for the final one to seven pixels. Lane loads
// and stores touch exactly one complete 32-bit pixel; they never read beyond
// a strict source subset or a clipped destination row.
#[cfg(all(feature = "simd", target_arch = "aarch64"))]
unsafe fn image_src_over_tail_neon(
    dst: &mut [u8],
    src: &[u8],
    coverage: u8,
    format: crate::raster::PixelFormat,
) {
    let count = src.len() / 4;
    if count == 0 {
        return;
    }
    macro_rules! run {
        ($setup:literal, $swizzle:literal, $blend:literal, $unused:literal) => {
            core::arch::asm!(
                "// scale {scale:w} alpha {alpha:w}",
                $setup,
                "2:",
                "ld1 {{v0.s}}[0], [{src}], #4",
                "dup v0.2s, v0.s[0]",
                "ld1 {{v4.s}}[0], [{dst}]",
                $swizzle,
                $blend,
                $unused,
                "st1 {{v4.s}}[0], [{dst}], #4",
                "subs {count}, {count}, #1",
                "b.ne 2b",
                src = inout(reg) src.as_ptr() => _,
                dst = inout(reg) dst.as_mut_ptr() => _,
                count = inout(reg) count => _,
                scale = in(reg) u32::from(coverage) + 1,
                alpha = out(reg) _,
                out("v0") _, out("v4") _, out("v16") _, out("v17") _,
                out("v18") _, out("v19") _, out("v20") _,
                out("v21") _, out("v22") _, out("v23") _,
                options(nostack),
            );
        };
    }
    macro_rules! blend {
        ($swizzle:literal, $unused:literal) => {
            if coverage == 255 {
                run!(
                    "movi v20.8b, #255",
                    $swizzle,
                    "dup v17.8b, v0.b[3]\n\
                     sub v17.8b, v20.8b, v17.8b\n\
                     umull v16.8h, v4.8b, v17.8b\n\
                     urshr v18.8h, v16.8h, #8\n\
                     raddhn v4.8b, v16.8h, v18.8h\n\
                     add v4.8b, v4.8b, v0.8b",
                    $unused
                );
            } else {
                run!(
                    "dup v20.8h, {scale:w}\n\
                     movi v21.8h, #255\n\
                     movi v22.8h, #1, lsl #8\n\
                     movi v23.8h, #1\n\
                     orr v23.8h, #1, lsl #8",
                    $swizzle,
                    "umov {alpha:w}, v0.b[3]\n\
                     dup v17.8h, {alpha:w}\n\
                     mul v18.8h, v17.8h, v20.8h\n\
                     ushr v19.8h, v18.8h, #8\n\
                     add v19.8h, v19.8h, v18.8h\n\
                     add v19.8h, v19.8h, v23.8h\n\
                     ushr v19.8h, v19.8h, #8\n\
                     sub v19.8h, v23.8h, v19.8h\n\
                     cmeq v17.8h, v17.8h, v21.8h\n\
                     sub v18.8h, v22.8h, v20.8h\n\
                     bsl v17.16b, v18.16b, v19.16b\n\
                     ushll v18.8h, v0.8b, #0\n\
                     mul v18.8h, v18.8h, v20.8h\n\
                     ushll v19.8h, v4.8b, #0\n\
                     mla v18.8h, v19.8h, v17.8h\n\
                     shrn v4.8b, v18.8h, #8",
                    $unused
                );
            }
        };
    }
    if format == crate::raster::PixelFormat::Rgba8888 {
        blend!("", "");
    } else if format == crate::raster::PixelFormat::Bgrx8888 {
        blend!(
            "rev32 v0.8b, v0.8b\next v0.8b, v0.8b, v0.8b, #1",
            "bic v4.2s, #255, lsl #24"
        );
    } else {
        blend!("rev32 v0.8b, v0.8b\next v0.8b, v0.8b, v0.8b, #1", "");
    }
}

#[cfg(all(test, feature = "simd", target_arch = "aarch64"))]
unsafe fn image_src_over_row_intrinsics(
    dst: &mut [u8],
    src: &[u8],
    coverage: u8,
    format: crate::raster::PixelFormat,
) -> usize {
    use core::arch::aarch64::*;
    #[inline(always)]
    unsafe fn full(s: uint8x8_t, d: uint8x8_t, inverse: uint8x8_t) -> uint8x8_t {
        let p = vmull_u8(d, inverse);
        vadd_u8(s, vraddhn_u16(p, vrshrq_n_u16::<8>(p)))
    }
    #[inline(always)]
    unsafe fn partial(
        s: uint8x8_t,
        d: uint8x8_t,
        scale: uint16x8_t,
        inverse: uint16x8_t,
    ) -> uint8x8_t {
        let p = vmulq_u16(vmovl_u8(s), scale);
        vshrn_n_u16::<8>(vmlaq_u16(p, vmovl_u8(d), inverse))
    }
    let mut offset = 0;
    let scale = vdupq_n_u16(u16::from(coverage) + 1);
    while offset + 32 <= src.len() {
        let s = vld4_u8(src.as_ptr().add(offset));
        let d = vld4_u8(dst.as_ptr().add(offset));
        let (s0, s2) = if format == crate::raster::PixelFormat::Rgba8888 {
            (s.0, s.2)
        } else {
            (s.2, s.0)
        };
        let result = if coverage == 255 {
            let inverse = vsub_u8(vdup_n_u8(255), s.3);
            uint8x8x4_t(
                full(s0, d.0, inverse),
                full(s.1, d.1, inverse),
                full(s2, d.2, inverse),
                full(s.3, d.3, inverse),
            )
        } else {
            let alpha = vmovl_u8(s.3);
            let t = vmulq_u16(alpha, scale);
            // Algebraically (65535-t + ((65535-t)>>8))>>8. All nonopaque
            // lanes fit u16 after +257; opaque lanes use 256-scale explicitly.
            let generic = vsubq_u16(
                vdupq_n_u16(257),
                vshrq_n_u16::<8>(vaddq_u16(
                    vaddq_u16(t, vshrq_n_u16::<8>(t)),
                    vdupq_n_u16(257),
                )),
            );
            let inverse = vbslq_u16(
                vceqq_u16(alpha, vdupq_n_u16(255)),
                vsubq_u16(vdupq_n_u16(256), scale),
                generic,
            );
            uint8x8x4_t(
                partial(s0, d.0, scale, inverse),
                partial(s.1, d.1, scale, inverse),
                partial(s2, d.2, scale, inverse),
                partial(s.3, d.3, scale, inverse),
            )
        };
        let mut result = result;
        if format == crate::raster::PixelFormat::Bgrx8888 {
            result.3 = vdup_n_u8(0);
        }
        vst4_u8(dst.as_mut_ptr().add(offset), result);
        offset += 32;
    }
    offset
}

// cpp: skia_renderer/skia_renderer.cc:342-357
pub(crate) fn mip_level_for_size(width: u32, height: u32, target_w: u32, target_h: u32) -> u32 {
    let mut level = 0;
    loop {
        let divisor = 1_u64 << (level + 1);
        let next_w = u64::from(width).div_ceil(divisor).max(1);
        let next_h = u64::from(height).div_ceil(divisor).max(1);
        if next_w < u64::from(target_w)
            || next_h < u64::from(target_h)
            || (width == 1 && height == 1)
        {
            return level;
        }
        // The source loop does not terminate once a larger image reaches 1x1.
        // Stop at that fixed point, as the existing Rust source replay does.
        if (next_w == 1 && next_h == 1) || level == 30 {
            return level + 1;
        }
        level += 1;
    }
}

// Chromium Skia src/core/SkMipmapHQDownSampler.cpp:178-360,393-445.
// Match HQDownSampler's once-per-level choice of the eight anisotropic /
// isotropic row filters. ColorTypeFilter_8888 widens all four channels to u16;
// only the complete two-dimensional weighted sum is shifted and narrowed.
fn downsample(source: &Pixmap) -> Pixmap {
    let w = source.width();
    let h = source.height();
    let mut output = Pixmap::new((w / 2).max(1), (h / 2).max(1)).unwrap();
    let axis = |size| match size {
        1 => 1,
        size if size % 2 == 0 => 2,
        _ => 3,
    };
    match (axis(w), axis(h)) {
        (1, 1) => output.data_mut().copy_from_slice(source.data()),
        (1, 2) => downsample_rows::<1, 2, 1>(source, &mut output),
        (1, 3) => downsample_rows::<1, 3, 2>(source, &mut output),
        (2, 1) => downsample_rows::<2, 1, 1>(source, &mut output),
        (2, 2) => downsample_rows::<2, 2, 2>(source, &mut output),
        (2, 3) => downsample_rows::<2, 3, 3>(source, &mut output),
        (3, 1) => downsample_rows::<3, 1, 2>(source, &mut output),
        (3, 2) => downsample_rows::<3, 2, 3>(source, &mut output),
        (3, 3) => downsample_rows::<3, 3, 4>(source, &mut output),
        _ => unreachable!(),
    }
    output
}

fn downsample_rows<const NX: usize, const NY: usize, const SHIFT: i32>(
    source: &Pixmap,
    output: &mut Pixmap,
) {
    let input = source.data();
    let row_bytes = source.width() as usize * 4;
    let output_row_bytes = output.width() as usize * 4;
    for (y, row) in output
        .data_mut()
        .chunks_exact_mut(output_row_bytes)
        .enumerate()
    {
        let input = &input[y * 2 * row_bytes..];
        #[cfg(all(feature = "simd", target_arch = "aarch64"))]
        unsafe {
            downsample_row_neon::<NX, NY, SHIFT>(input, row_bytes, row);
        }
        #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
        downsample_row_portable::<NX, NY, SHIFT>(input, row_bytes, row, 0);
    }
}

// Portable ColorTypeFilter_8888 Expand/add_121/Compact counterparts.
fn downsample_row_portable<const NX: usize, const NY: usize, const SHIFT: i32>(
    input: &[u8],
    row_bytes: usize,
    output: &mut [u8],
    start: usize,
) {
    for (x, pixel) in output.chunks_exact_mut(4).enumerate().skip(start) {
        let offset = x * 8;
        for c in 0..4 {
            let horizontal = |row: usize| {
                let i = row * row_bytes + offset + c;
                let a = u16::from(input[i]);
                if NX == 1 {
                    a
                } else if NX == 2 {
                    a + u16::from(input[i + 4])
                } else {
                    a + 2 * u16::from(input[i + 4]) + u16::from(input[i + 8])
                }
            };
            let a = horizontal(0);
            let sum = if NY == 1 {
                a
            } else if NY == 2 {
                a + horizontal(1)
            } else {
                a + 2 * horizontal(1) + horizontal(2)
            };
            pixel[c] = (sum >> SHIFT) as u8;
        }
    }
}

// Four output pixels per group, translating ColorTypeFilter_8888's widened
// vector sum. There is no intermediate rounding between the horizontal and
// vertical filters. Loads are unaligned and never extend past the final tap.
#[cfg(all(feature = "simd", target_arch = "aarch64"))]
unsafe fn downsample_row_neon<const NX: usize, const NY: usize, const SHIFT: i32>(
    input: &[u8],
    row_bytes: usize,
    output: &mut [u8],
) {
    let groups = if NX == 1 { 0 } else { output.len() / 16 };
    let x = groups * 4;
    if groups != 0 {
        let p0 = input.as_ptr();
        let p1 = if NY >= 2 { p0.add(row_bytes) } else { p0 };
        let p2 = if NY == 3 { p0.add(2 * row_bytes) } else { p0 };
        let flags =
            usize::from(NX == 3) | (usize::from(NY >= 2) << 1) | (usize::from(NY == 3) << 2);
        // SAFETY: each group has four complete destination pixels. NX=2
        // reads exactly eight source pixels; NX=3 reads those plus one u32
        // third tap, never another interleaved vector past the row boundary.
        // Only NY admitted source rows are addressed. Widened u16 sums cannot
        // overflow (maximum 16*255); narrowing truncates exactly like Skia's
        // ColorTypeFilter_8888::Compact(shift_right(...)). The checked caller
        // supplies disjoint initialized source and destination slices. Keeping
        // the complete row kernel in one asm block avoids opt-level=0 SIMD ABI
        // temporaries without changing taps, precision, tails or allocations.
        core::arch::asm!(
            "2:",
            "ld2 {{v0.4s, v1.4s}}, [{p0}], #32",
            "uaddl v4.8h, v0.8b, v1.8b",
            "uaddl2 v5.8h, v0.16b, v1.16b",
            "tbz {flags}, #0, 3f",
            "ldr s2, [{p0}]",
            "ext v2.16b, v0.16b, v2.16b, #4",
            "uaddw v4.8h, v4.8h, v1.8b",
            "uaddw2 v5.8h, v5.8h, v1.16b",
            "uaddw v4.8h, v4.8h, v2.8b",
            "uaddw2 v5.8h, v5.8h, v2.16b",
            "3:",
            "tbz {flags}, #1, 7f",
            "ld2 {{v0.4s, v1.4s}}, [{p1}], #32",
            "uaddl v6.8h, v0.8b, v1.8b",
            "uaddl2 v7.8h, v0.16b, v1.16b",
            "tbz {flags}, #0, 4f",
            "ldr s2, [{p1}]",
            "ext v2.16b, v0.16b, v2.16b, #4",
            "uaddw v6.8h, v6.8h, v1.8b",
            "uaddw2 v7.8h, v7.8h, v1.16b",
            "uaddw v6.8h, v6.8h, v2.8b",
            "uaddw2 v7.8h, v7.8h, v2.16b",
            "4:",
            "tbz {flags}, #2, 5f",
            "shl v6.8h, v6.8h, #1",
            "shl v7.8h, v7.8h, #1",
            "5:",
            "add v4.8h, v4.8h, v6.8h",
            "add v5.8h, v5.8h, v7.8h",
            "tbz {flags}, #2, 7f",
            "ld2 {{v0.4s, v1.4s}}, [{p2}], #32",
            "uaddl v6.8h, v0.8b, v1.8b",
            "uaddl2 v7.8h, v0.16b, v1.16b",
            "tbz {flags}, #0, 6f",
            "ldr s2, [{p2}]",
            "ext v2.16b, v0.16b, v2.16b, #4",
            "uaddw v6.8h, v6.8h, v1.8b",
            "uaddw2 v7.8h, v7.8h, v1.16b",
            "uaddw v6.8h, v6.8h, v2.8b",
            "uaddw2 v7.8h, v7.8h, v2.16b",
            "6:",
            "add v4.8h, v4.8h, v6.8h",
            "add v5.8h, v5.8h, v7.8h",
            "7:",
            "shrn v0.8b, v4.8h, #{shift}",
            "shrn2 v0.16b, v5.8h, #{shift}",
            "str q0, [{dst}], #16",
            "subs {groups}, {groups}, #1",
            "b.ne 2b",
            p0 = inout(reg) p0 => _, p1 = inout(reg) p1 => _, p2 = inout(reg) p2 => _,
            dst = inout(reg) output.as_mut_ptr() => _,
            groups = inout(reg) groups => _, flags = in(reg) flags, shift = const SHIFT,
            out("v0") _, out("v1") _, out("v2") _,
            out("v4") _, out("v5") _, out("v6") _, out("v7") _,
            options(nostack),
        );
    }
    downsample_row_portable::<NX, NY, SHIFT>(input, row_bytes, output, x);
}

// SkBitmapProcStateAutoMapper / S32_alpha_D32_filter_DX use a 32.32
// center coordinate with a half-pixel bias and four fractional filter bits.
fn sample_bilinear_bitmap(source: &Pixmap, qx: i64, qy: i64) -> [u8; 4] {
    BitmapSampleRows::new(source, qy).sample(qx)
}

// S32_alpha_D32_filter_DX decodes its constant Y coordinate once, then samples
// every X coordinate from the same two source rows and four-bit Y weight.
// Borrowed row slices are per-draw inputs, not a retained image/frame cache.
// Immutable strict-source ROI. Each row slice excludes pixels outside the
// rounded subset, so the existing clamp and SIMD loads cannot cross its edge.
#[derive(Clone, Copy)]
struct BitmapSubset<'a> {
    data: &'a [u8],
    stride: usize,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
}
impl<'a> BitmapSubset<'a> {
    fn row(&self, y: u32) -> &'a [u8] {
        debug_assert!(y < self.height);
        let start = ((self.top as usize + y as usize) * self.stride + self.left as usize) * 4;
        &self.data[start..start + self.width as usize * 4]
    }
}

struct BitmapSampleRows<'a> {
    top: &'a [u8],
    bottom: &'a [u8],
    width: u32,
    weight_y: u32,
}
impl<'a> BitmapSampleRows<'a> {
    fn new(source: &'a Pixmap, qy: i64) -> Self {
        Self::new_subset(
            &BitmapSubset {
                data: source.data(),
                stride: source.width() as usize,
                left: 0,
                top: 0,
                width: source.width(),
                height: source.height(),
            },
            qy,
        )
    }
    fn new_subset(source: &BitmapSubset<'a>, qy: i64) -> Self {
        let integer = qy >> 32;
        let y0 = integer.clamp(0, i64::from(source.height) - 1) as u32;
        let y1 = (integer + 1).clamp(0, i64::from(source.height) - 1) as u32;
        Self {
            top: source.row(y0),
            bottom: source.row(y1),
            width: source.width,
            weight_y: ((qy >> 28) & 15) as u32,
        }
    }
    // SkBitmapProcState shades a bounded row rather than calling a pixel
    // sampler through the Debug ABI for every destination pixel. Coordinates
    // retain the existing 32.32 mapper and four-bit weights exactly.
    fn sample_row(&self, qx: i64, step: i64, dst: &mut [u8]) {
        assert_eq!(dst.len() % 4, 0);
        let count = dst.len() / 4;
        if count == 0 {
            return;
        }
        #[cfg(all(feature = "simd", target_arch = "aarch64"))]
        if step
            .checked_mul(count as i64)
            .and_then(|d| qx.checked_add(d))
            .is_some()
        {
            // SAFETY: both source rows have exactly width*4 initialized bytes,
            // width is nonzero, and each loaded index is clamped to [0,width).
            // dst has count complete pixels, disjoint from the source rows.
            // The checked progression cannot overflow and no pointer escapes.
            unsafe {
                self.sample_row_neon(qx, step, dst);
            }
            return;
        }
        let mut q = qx;
        for pixel in dst.chunks_exact_mut(4) {
            pixel.copy_from_slice(&self.sample(q));
            q += step;
        }
    }

    // Preserve the original checked mapper and strict interior proof. Edge
    // runs use the existing clamped sampler and row store without any writes
    // here; partial/unknown-alpha callers never enter this path.
    fn try_sample_opaque_row(
        &self,
        qx: i64,
        step: i64,
        dst: &mut [u8],
        format: crate::raster::PixelFormat,
    ) -> bool {
        #[cfg(all(feature = "simd", target_arch = "aarch64"))]
        {
            debug_assert_eq!(dst.len() % 4, 0);
            let count = dst.len() / 4;
            if count == 0 {
                return true;
            }
            if step
                .checked_mul(count as i64)
                .and_then(|d| qx.checked_add(d))
                .is_none()
            {
                return false;
            }
            let last = qx + step * (count as i64 - 1);
            if (qx.min(last) >> 32) < 0 || (qx.max(last) >> 32) >= i64::from(self.width) - 1 {
                return false;
            }
            let paired_pixels = count & !1;
            if paired_pixels != 0 {
                // SAFETY: the identical endpoint proof used by sample_row_neon
                // bounds all adjacent tap loads; dst holds complete pairs.
                unsafe {
                    if format == crate::raster::PixelFormat::Rgba8888 {
                        self.sample_interior_pairs_neon::<false>(
                            qx,
                            step,
                            &mut dst[..paired_pixels * 4],
                            format,
                        );
                    } else {
                        self.sample_interior_pairs_neon::<true>(
                            qx,
                            step,
                            &mut dst[..paired_pixels * 4],
                            format,
                        );
                    }
                }
            }
            if paired_pixels != count {
                let pixel = self.sample(qx + step * paired_pixels as i64);
                dst[paired_pixels * 4..].copy_from_slice(&format.encode(pixel));
            }
            true
        }
        #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
        {
            let _ = (qx, step, dst, format);
            false
        }
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    unsafe fn sample_row_neon(&self, mut qx: i64, step: i64, mut dst: &mut [u8]) {
        // Prove both taps are inside the strict subset once per bounded span.
        // The checked linear progression in sample_row is monotone, so its
        // endpoints contain every intermediate integer coordinate. Interior
        // samples can load each adjacent pair together without per-pixel clamp
        // branches; edge spans keep the original four separately clamped taps.
        let last = qx + step * (dst.len() as i64 / 4 - 1);
        let interior =
            (qx.min(last) >> 32) >= 0 && (qx.max(last) >> 32) < i64::from(self.width) - 1;
        macro_rules! shade {
            ($loads:literal) => {
                core::arch::asm!(
                    "dup v17.8b, {wy:w}",
                    "movi v16.8b, #16",
                    "sub v16.8b, v16.8b, v17.8b",
                    "movi v18.4h, #16",
                    "2:",
                    "asr {ix}, {qx}, #32",
                    $loads,
                    "ubfx {scratch}, {qx}, #28, #4",
                    "dup v6.4h, {scratch:w}",
                    "sub v7.4h, v18.4h, v6.4h",
                    "umull v4.8h, v2.8b, v16.8b",
                    "umull v5.8h, v3.8b, v17.8b",
                    "ext v2.16b, v4.16b, v4.16b, #8",
                    "ext v3.16b, v5.16b, v5.16b, #8",
                    "mul v1.4h, v2.4h, v6.4h",
                    "mla v1.4h, v3.4h, v6.4h",
                    "mla v1.4h, v4.4h, v7.4h",
                    "mla v1.4h, v5.4h, v7.4h",
                    "shrn v0.8b, v1.8h, #8",
                    "st1 {{v0.s}}[0], [{dst}], #4",
                    "add {qx}, {qx}, {step}",
                    "subs {count}, {count}, #1",
                    "b.ne 2b",
                    // ix1/max are used only by the edge kernel, but keep the
                    // register contract identical for the two instantiations.
                    "// {ix1} {max}",
                    top = in(reg) self.top.as_ptr(), bottom = in(reg) self.bottom.as_ptr(),
                    max = in(reg) i64::from(self.width) - 1,
                    wy = in(reg) self.weight_y, step = in(reg) step,
                    qx = inout(reg) qx => _, dst = inout(reg) dst.as_mut_ptr() => _,
                    count = inout(reg) dst.len() / 4 => _,
                    ix = out(reg) _, ix1 = out(reg) _, scratch = out(reg) _,
                    out("v0") _, out("v1") _, out("v2") _, out("v3") _,
                    out("v4") _, out("v5") _, out("v6") _, out("v7") _,
                    out("v16") _, out("v17") _, out("v18") _,
                    options(nostack),
                );
            };
        }
        if interior {
            let paired_pixels = (dst.len() / 8) * 2;
            if paired_pixels != 0 {
                self.sample_interior_pairs_neon::<false>(
                    qx,
                    step,
                    &mut dst[..paired_pixels * 4],
                    crate::raster::PixelFormat::Rgba8888,
                );
                qx += step * paired_pixels as i64;
                dst = &mut dst[paired_pixels * 4..];
                if dst.is_empty() {
                    return;
                }
            }
            // Each eight-byte load ends at x+1 inside the borrowed ROI row;
            // unaligned pair loads preserve the original RGBA lane ordering.
            shade!(
                "add {scratch}, {top}, {ix}, lsl #2\n\
                    ldr d2, [{scratch}]\n\
                    add {scratch}, {bottom}, {ix}, lsl #2\n\
                    ldr d3, [{scratch}]"
            );
        } else {
            shade!(
                "add {ix1}, {ix}, #1\n\
                    cmp {ix}, #0\n\
                    csel {ix}, xzr, {ix}, lt\n\
                    cmp {ix}, {max}\n\
                    csel {ix}, {max}, {ix}, gt\n\
                    cmp {ix1}, #0\n\
                    csel {ix1}, xzr, {ix1}, lt\n\
                    cmp {ix1}, {max}\n\
                    csel {ix1}, {max}, {ix1}, gt\n\
                    ldr s2, [{top}, {ix}, lsl #2]\n\
                    ldr {scratch:w}, [{top}, {ix1}, lsl #2]\n\
                    ins v2.s[1], {scratch:w}\n\
                    ldr s3, [{bottom}, {ix}, lsl #2]\n\
                    ldr {scratch:w}, [{bottom}, {ix1}, lsl #2]\n\
                    ins v3.s[1], {scratch:w}"
            );
        }
    }

    // S32_alpha_D32_filter_DX, two outputs per SIMD group. Gather the exact
    // four taps for each coordinate, widen before any arithmetic and narrow
    // once after the complete 4-bit two-dimensional weighted sum. Unlike a
    // separable byte filter, this never rounds between the X and Y stages.
    // Only the caller's proven interior span uses adjacent 8-byte row loads.
    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    #[inline(always)]
    unsafe fn sample_interior_pairs_neon<const SWIZZLE: bool>(
        &self,
        qx: i64,
        step: i64,
        dst: &mut [u8],
        format: crate::raster::PixelFormat,
    ) {
        debug_assert!(dst.len() >= 8 && dst.len() % 8 == 0);
        let shuffle = if format == crate::raster::PixelFormat::Bgrx8888 {
            [2u8, 1, 0, 16, 6, 5, 4, 16, 10, 9, 8, 16, 14, 13, 12, 16]
        } else {
            [2u8, 1, 0, 3, 6, 5, 4, 7, 10, 9, 8, 11, 14, 13, 12, 15]
        };
        macro_rules! shade {
            ($setup:literal, $store:literal) => {
                core::arch::asm!(
            $setup,
            "// {shuffle}",
            "dup v17.16b, {wy:w}",
            "movi v16.16b, #16",
            "sub v16.16b, v16.16b, v17.16b",
            "movi v18.8h, #16",
            "2:",
            "add {q1}, {qx}, {step}",
            "asr {ix}, {qx}, #32",
            "asr {ix1}, {q1}, #32",
            "add {scratch}, {top}, {ix}, lsl #2",
            "ldr d2, [{scratch}]",
            "add {scratch}, {top}, {ix1}, lsl #2",
            "ldr {scratch}, [{scratch}]",
            "ins v2.d[1], {scratch}",
            "add {scratch}, {bottom}, {ix}, lsl #2",
            "ldr d3, [{scratch}]",
            "add {scratch}, {bottom}, {ix1}, lsl #2",
            "ldr {scratch}, [{scratch}]",
            "ins v3.d[1], {scratch}",
            // Low halves contain top A/B; high halves contain bottom A/B.
            "uzp1 v4.4s, v2.4s, v3.4s",
            "uzp2 v5.4s, v2.4s, v3.4s",
            "ubfx {scratch}, {qx}, #28, #4",
            "dup v6.4h, {scratch:w}",
            "ubfx {scratch}, {q1}, #28, #4",
            "dup v7.4h, {scratch:w}",
            "ins v6.d[1], v7.d[0]",
            "sub v7.8h, v18.8h, v6.8h",
            "umull v0.8h, v4.8b, v16.8b",
            "umull2 v1.8h, v4.16b, v17.16b",
            "add v0.8h, v0.8h, v1.8h",
            "umull v2.8h, v5.8b, v16.8b",
            "umull2 v3.8h, v5.16b, v17.16b",
            "add v2.8h, v2.8h, v3.8h",
            "mul v0.8h, v0.8h, v7.8h",
            "mla v0.8h, v2.8h, v6.8h",
            "shrn v0.8b, v0.8h, #8",
            $store,
            "add {qx}, {q1}, {step}",
            "subs {count}, {count}, #1",
            "b.ne 2b",
            top = in(reg) self.top.as_ptr(), bottom = in(reg) self.bottom.as_ptr(),
            wy = in(reg) self.weight_y, step = in(reg) step,
            shuffle = in(reg) if SWIZZLE { shuffle.as_ptr() } else { core::ptr::null() },
            qx = inout(reg) qx => _, dst = inout(reg) dst.as_mut_ptr() => _,
            count = inout(reg) dst.len() / 8 => _,
            q1 = out(reg) _, ix = out(reg) _, ix1 = out(reg) _, scratch = out(reg) _,
            out("v0") _, out("v1") _, out("v2") _, out("v3") _,
            out("v4") _, out("v5") _, out("v6") _, out("v7") _,
            out("v16") _, out("v17") _, out("v18") _, out("v19") _,
            options(nostack),
                );
            };
        }
        if SWIZZLE {
            shade!(
                "ldr q19, [{shuffle}]",
                "tbl v0.16b, {{v0.16b}}, v19.16b\nstr d0, [{dst}], #8"
            );
        } else {
            shade!("", "str d0, [{dst}], #8");
        }
    }

    fn sample(&self, qx: i64) -> [u8; 4] {
        let integer = qx >> 32;
        let x0 = integer.clamp(0, i64::from(self.width) - 1) as usize;
        let x1 = (integer + 1).clamp(0, i64::from(self.width) - 1) as usize;
        let wx = ((qx >> 28) & 15) as u32;
        let load = |row: &[u8], x: usize| {
            // The clamp and complete row slice guarantee exactly four readable
            // bytes, including at the bottom-right or in a one-pixel image.
            unsafe {
                u32::from_le(core::ptr::read_unaligned(
                    row.as_ptr().add(x * 4).cast::<u32>(),
                ))
            }
        };
        let pixels = [
            load(self.top, x0),
            load(self.top, x1),
            load(self.bottom, x0),
            load(self.bottom, x1),
        ];
        #[cfg(all(feature = "simd", target_arch = "aarch64"))]
        let output = unsafe { filter_and_scale_by_alpha_neon(wx, self.weight_y, pixels) };
        #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
        let output = filter_and_scale_by_alpha_portable(wx, self.weight_y, pixels);
        output.to_le_bytes()
    }
}

// SkBitmapProcState_opts.h::filter_and_scale_by_alpha, alphaScale == 256.
// Treat alternating byte channels as two independent 16-bit lanes. Since all
// four weights add to 256, each lane sum is <= 255*256 and cannot carry into
// its neighbor. Shift only after the complete two-dimensional weighted sum.
#[cfg_attr(all(feature = "simd", target_arch = "aarch64"), allow(dead_code))]
fn filter_and_scale_by_alpha_portable(x: u32, y: u32, [a00, a01, a10, a11]: [u32; 4]) -> u32 {
    let xy = x * y;
    let mask = 0x00ff00ff;
    let scale = (256 - 16 * y as i32 - 16 * x as i32 + xy as i32) as u32;
    let mut lo = (a00 & mask) * scale;
    let mut hi = ((a00 >> 8) & mask) * scale;
    let scale = 16 * x - xy;
    lo += (a01 & mask) * scale;
    hi += ((a01 >> 8) & mask) * scale;
    let scale = 16 * y - xy;
    lo += (a10 & mask) * scale;
    hi += ((a10 >> 8) & mask) * scale;
    lo += (a11 & mask) * xy;
    hi += ((a11 >> 8) & mask) * xy;
    ((lo >> 8) & mask) | (hi & !mask)
}

// Direct NEON counterpart of SkBitmapProcState_opts.h's scalar-output helper.
// Source pixels have already been loaded individually; vectorizing the four
// color channels never reads additional neighbors outside the source subset.
#[cfg(all(feature = "simd", target_arch = "aarch64"))]
unsafe fn filter_and_scale_by_alpha_neon(x: u32, y: u32, [a00, a01, a10, a11]: [u32; 4]) -> u32 {
    let output: u32;
    // The caller decodes four-bit weights, so x/y are in 0..16. Each
    // widened byte sum is at most 255*256 = 65280, fitting u16. This is
    // exactly the official NEON helper with alphaScale=256: no intermediate
    // rounding, then one non-saturating right shift by eight. A single block
    // keeps the vector intermediates in registers in unoptimized builds.
    // No memory is accessed: each complete source pixel was loaded by the
    // clamped row adapter, including one-pixel images and final-row tails.
    core::arch::asm!(
        "dup v1.8b, {y:w}",
        "movi v0.8b, #16",
        "sub v0.8b, v0.8b, v1.8b",
        "fmov s2, {a00:w}",
        "ins v2.s[1], {a01:w}",
        "fmov s3, {a10:w}",
        "ins v3.s[1], {a11:w}",
        "umull v4.8h, v2.8b, v0.8b",
        "umull v5.8h, v3.8b, v1.8b",
        "dup v6.4h, {x:w}",
        "movi v7.4h, #16",
        "sub v7.4h, v7.4h, v6.4h",
        "ext v2.16b, v4.16b, v4.16b, #8",
        "ext v3.16b, v5.16b, v5.16b, #8",
        "mul v1.4h, v2.4h, v6.4h",
        "mla v1.4h, v3.4h, v6.4h",
        "mla v1.4h, v4.4h, v7.4h",
        "mla v1.4h, v5.4h, v7.4h",
        "shrn v0.8b, v1.8h, #8",
        "umov {output:w}, v0.s[0]",
        x = in(reg) x,
        y = in(reg) y,
        a00 = in(reg) a00,
        a01 = in(reg) a01,
        a10 = in(reg) a10,
        a11 = in(reg) a11,
        output = lateout(reg) output,
        out("v0") _, out("v1") _, out("v2") _, out("v3") _,
        out("v4") _, out("v5") _, out("v6") _, out("v7") _,
        options(nomem, nostack, preserves_flags),
    );
    output
}

// SkRasterPipeline_opts.h: LOWP_STAGE_GP(bilerp_clamp_8888). Sample
// coordinates are quantized to 16.16, then two signed Q15 interpolations keep
// extra precision between axes. The rounded multiply matches ARM's vqrdmulh.
pub(crate) fn sample_bilinear_lowp(source: &Pixmap, cx: f32, cy: f32) -> [u8; 4] {
    let coords = |center: f32, size: u32| {
        let q = (65536.0 * center + 0.5).floor() as i32 - 32768;
        let integer = q >> 16;
        let left = integer.clamp(0, size as i32 - 1) as usize;
        let right = (integer + 1).clamp(0, size as i32 - 1) as usize;
        (left, right, (q ^ 0x8000) as i16)
    };
    let (x0, x1, tx) = coords(cx, source.width());
    let (y0, y1, ty) = coords(cy, source.height());
    let offsets = [
        (y0 * source.width() as usize + x0) * 4,
        (y0 * source.width() as usize + x1) * 4,
        (y1 * source.width() as usize + x0) * 4,
        (y1 * source.width() as usize + x1) * 4,
    ];
    let scaled = |a: i16, b: i32| {
        ((i64::from(a) * i64::from(b) + (1 << 14)) >> 15).clamp(-32768, 32767) as i32
    };
    std::array::from_fn(|c| {
        let horizontal = |l: usize, r: usize| {
            let l = i32::from(source.data()[l + c]);
            let r = i32::from(source.data()[r + c]);
            (scaled(tx, (r - l) << 7) + ((r + l) << 7) + 1) >> 1
        };
        let top = horizontal(offsets[0], offsets[1]);
        let bottom = horizontal(offsets[2], offsets[3]);
        ((scaled(ty, bottom - top) + bottom + top + 128) >> 8) as u8
    })
}

// Restricted lowp bilerp_clamp_8888 row kernel. Coordinates retain the
// scalePixels matrix's f32 operations; only X quantization is shared across
// rows. Each tap loads a single clamped pixel, including one-pixel images.
#[cfg(all(feature = "simd", target_arch = "aarch64"))]
unsafe fn sample_lowp_row_neon(
    top: &[u8],
    bottom: &[u8],
    xs: &[[i32; 3]],
    ty: i16,
    output: &mut [u8],
) {
    debug_assert_eq!(output.len(), xs.len() * 4);
    if xs.is_empty() {
        return;
    }
    // SAFETY: the private resize driver supplies one complete pair of source
    // rows and clamps every X index to that row. xs contains exactly one
    // initialized [left,right,Q15 weight] triple per output pixel. The source,
    // coordinate and output slices are disjoint. No vector load exceeds a
    // four-byte pixel. Q15 products use saturating rounded doubling multiply,
    // exactly SkRasterPipeline lowp q_mult; middle sums are constrained to
    // u16 by the same lerpX/lerpY ranges and are narrowed only at the end.
    core::arch::asm!(
        "dup v17.4h, {ty:w}",
        "movi v18.4h, #1",
        "movi v19.4h, #128",
        "2:",
        "ldp {left:w}, {right:w}, [{coords}]",
        "ldr {weight:w}, [{coords}, #8]",
        "add {coords}, {coords}, #12",
        "dup v16.4h, {weight:w}",
        "ldr s0, [{top}, {left}, lsl #2]",
        "ldr s1, [{top}, {right}, lsl #2]",
        "uxtl v0.8h, v0.8b",
        "uxtl v1.8h, v1.8b",
        "shl v0.4h, v0.4h, #7",
        "shl v1.4h, v1.4h, #7",
        "sub v2.4h, v1.4h, v0.4h",
        "add v3.4h, v1.4h, v0.4h",
        "sqrdmulh v2.4h, v2.4h, v16.4h",
        "add v2.4h, v2.4h, v3.4h",
        "add v2.4h, v2.4h, v18.4h",
        "ushr v4.4h, v2.4h, #1",
        "ldr s0, [{bottom}, {left}, lsl #2]",
        "ldr s1, [{bottom}, {right}, lsl #2]",
        "uxtl v0.8h, v0.8b",
        "uxtl v1.8h, v1.8b",
        "shl v0.4h, v0.4h, #7",
        "shl v1.4h, v1.4h, #7",
        "sub v2.4h, v1.4h, v0.4h",
        "add v3.4h, v1.4h, v0.4h",
        "sqrdmulh v2.4h, v2.4h, v16.4h",
        "add v2.4h, v2.4h, v3.4h",
        "add v2.4h, v2.4h, v18.4h",
        "ushr v5.4h, v2.4h, #1",
        "sub v6.4h, v5.4h, v4.4h",
        "sqrdmulh v6.4h, v6.4h, v17.4h",
        "add v6.4h, v6.4h, v4.4h",
        "add v6.4h, v6.4h, v5.4h",
        "add v6.4h, v6.4h, v19.4h",
        "ushr v6.4h, v6.4h, #8",
        "xtn v0.8b, v6.8h",
        "st1 {{v0.s}}[0], [{dst}], #4",
        "subs {count}, {count}, #1",
        "b.ne 2b",
        top = in(reg) top.as_ptr(), bottom = in(reg) bottom.as_ptr(), ty = in(reg) i32::from(ty),
        coords = inout(reg) xs.as_ptr() => _, dst = inout(reg) output.as_mut_ptr() => _,
        count = inout(reg) xs.len() => _, left = out(reg) _, right = out(reg) _, weight = out(reg) _,
        out("v0") _, out("v1") _, out("v2") _, out("v3") _, out("v4") _, out("v5") _, out("v6") _,
        out("v16") _, out("v17") _, out("v18") _, out("v19") _, options(nostack),
    );
}

fn resize_bilinear(source: &Pixmap, width: u32, height: u32, base_size: (u32, u32)) -> Pixmap {
    if source.width() == width && source.height() == height {
        return source.clone();
    }
    let mut output = Pixmap::new(width, height).unwrap();
    // SkPixmap::scalePixels creates a forward Rect2Rect from the ORIGINAL
    // image to the target. MatrixRec::apply inverts it, then left-concatenates
    // SkMipmapAccessor's selected-mip/original-image scale. Keep each f32
    // operation: reducing this to selected-mip/target changes sample rounding.
    let inv_x =
        (source.width() as f32 / base_size.0 as f32) * (1.0 / (width as f32 / base_size.0 as f32));
    let inv_y = (source.height() as f32 / base_size.1 as f32)
        * (1.0 / (height as f32 / base_size.1 as f32));
    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    {
        let coords = |center: f32, size: u32| {
            let q = (65536.0 * center + 0.5).floor() as i32 - 32768;
            let integer = q >> 16;
            [
                integer.clamp(0, size as i32 - 1),
                (integer + 1).clamp(0, size as i32 - 1),
                i32::from((q ^ 0x8000) as i16),
            ]
        };
        let xs: Vec<[i32; 3]> = (0..width)
            .map(|x| coords((x as f32 + 0.5) * inv_x, source.width()))
            .collect();
        let row_bytes = source.width() as usize * 4;
        for (y, row) in output
            .data_mut()
            .chunks_exact_mut(width as usize * 4)
            .enumerate()
        {
            let [y0, y1, ty] = coords((y as f32 + 0.5) * inv_y, source.height());
            let top = &source.data()[y0 as usize * row_bytes..(y0 as usize + 1) * row_bytes];
            let bottom = &source.data()[y1 as usize * row_bytes..(y1 as usize + 1) * row_bytes];
            unsafe {
                sample_lowp_row_neon(top, bottom, &xs, ty as i16, row);
            }
        }
    }
    #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
    {
        for y in 0..height {
            for x in 0..width {
                let sample = sample_bilinear_lowp(
                    source,
                    (x as f32 + 0.5) * inv_x,
                    (y as f32 + 0.5) * inv_y,
                );
                let offset = (y as usize * width as usize + x as usize) * 4;
                output.data_mut()[offset..offset + 4].copy_from_slice(&sample);
            }
        }
    }
    output
}

// cpp: skia_renderer/skia_renderer.cc:646-669
pub(crate) fn build_mip_image(source: &Pixmap, level: u32) -> Pixmap {
    let divisor = 1_u64 << level;
    let width = u64::from(source.width()).div_ceil(divisor).max(1) as u32;
    let height = u64::from(source.height()).div_ceil(divisor).max(1) as u32;
    // scalePixels(kLinear,kNearest) internally selects a floor-sized SkMipmap
    // level, then bilinearly scales it to the ceil-sized image above.
    // SkMipmap::ComputeLevel has a -0.5 bias before nearest-level rounding.
    let scale = (width as f32 / source.width() as f32).min(height as f32 / source.height() as f32);
    let selected = (-scale.log2() - 0.5).max(0.0).round() as u32;
    // SkMipmap::Build starts from a borrowed source view, not a pixel copy.
    let mut mip = None;
    for _ in 0..selected {
        let current = mip.as_ref().unwrap_or(source);
        if current.width() == 1 && current.height() == 1 {
            break;
        }
        mip = Some(downsample(current));
    }
    resize_bilinear(
        mip.as_ref().unwrap_or(source),
        width,
        height,
        (source.width(), source.height()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster::{ColorU8, IntSize};

    fn previous_sample_bilinear_bitmap(source: &Pixmap, qx: i64, qy: i64) -> [u8; 4] {
        let coordinates = |q: i64, size: u32| {
            let integer = q >> 32;
            (
                integer.clamp(0, i64::from(size) - 1) as usize,
                (integer + 1).clamp(0, i64::from(size) - 1) as usize,
                ((q >> 28) & 15) as u32,
            )
        };
        let (x0, x1, wx) = coordinates(qx, source.width());
        let (y0, y1, wy) = coordinates(qy, source.height());
        let row_bytes = source.width() as usize * 4;
        let data = source.data();
        let load = |offset: usize| {
            // Both coordinates are clamped to a valid pixel. Pixmap stores width
            // * height complete four-byte pixels, so each load has exactly four
            // accessible bytes even at the bottom-right corner or on a thin image.
            unsafe {
                u32::from_le(core::ptr::read_unaligned(
                    data.as_ptr().add(offset).cast::<u32>(),
                ))
            }
        };
        let pixels = [
            load(y0 * row_bytes + x0 * 4),
            load(y0 * row_bytes + x1 * 4),
            load(y1 * row_bytes + x0 * 4),
            load(y1 * row_bytes + x1 * 4),
        ];
        #[cfg(all(feature = "simd", target_arch = "aarch64"))]
        let output = unsafe { filter_and_scale_by_alpha_neon(wx, wy, pixels) };
        #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
        let output = filter_and_scale_by_alpha_portable(wx, wy, pixels);
        output.to_le_bytes()
    }

    fn previous_draw_image_bitmap(
        output: &mut Pixmap,
        clip: Option<&Mask>,
        canvas: Transform,
        image: &Pixmap,
        destination: PaintRect,
        source: PaintRect,
        device_origin: (u32, u32),
        tile_origin: (i32, i32),
        clip_runs: Option<&[bool]>,
    ) -> bool {
        if canvas.sx <= 0.0
            || canvas.sy <= 0.0
            || !canvas.sx.is_finite()
            || !canvas.sy.is_finite()
            || canvas.kx != 0.0
            || canvas.ky != 0.0
        {
            return false;
        }
        // SkBitmapProcState's scale+translate matrix is expressed in device
        // pixels; a high-DPI canvas still uses the same four-bit bilerp stages.
        let left = destination.x as f32 * canvas.sx + canvas.tx;
        let top = destination.y as f32 * canvas.sy + canvas.ty;
        let right = left + destination.width as f32 * canvas.sx;
        let bottom = top + destination.height as f32 * canvas.sy;
        if destination.width <= 0.0
            || destination.height <= 0.0
            || [left, top, right, bottom].iter().any(|v| !v.is_finite())
        {
            return false;
        }
        // SkScan::FillRect (BW image paint) calls SkRect::round, not roundOut.
        // Retain raw mapped bounds for the shader; round only raster geometry.
        let round = |v: f32| (v + 0.5).floor();
        let x_start = round(left).max(0.0) as u32;
        let x_end = round(right).min(output.width() as f32).max(0.0) as u32;
        let y_start = round(top).max(0.0) as u32;
        let y_end = round(bottom).min(output.height() as f32).max(0.0) as u32;
        // SkBitmapDevice::drawImageRect extracts the rounded-out source subset
        // before building its clamp shader (kStrict_SrcRectConstraint).
        let src_left = source.x as f32;
        let src_top = source.y as f32;
        let src_right = src_left + source.width as f32;
        let src_bottom = src_top + source.height as f32;
        if src_left < 0.0
            || src_top < 0.0
            || src_right > image.width() as f32
            || src_bottom > image.height() as f32
            || ![src_left, src_top, src_right, src_bottom]
                .iter()
                .all(|v| v.is_finite())
            || src_right <= src_left
            || src_bottom <= src_top
        {
            return false;
        }
        let ox = src_left.floor() as u32;
        let oy = src_top.floor() as u32;
        let subset;
        let image = if ox != 0
            || oy != 0
            || src_right.ceil() as u32 != image.width()
            || src_bottom.ceil() as u32 != image.height()
        {
            subset = image
                .clone_rect(
                    crate::raster::IntRect::from_ltrb(
                        ox as i32,
                        oy as i32,
                        src_right.ceil() as i32,
                        src_bottom.ceil() as i32,
                    )
                    .unwrap(),
                )
                .unwrap();
            &subset
        } else {
            image
        };
        let local_scale_x = ((destination.x as f32 + destination.width as f32)
            - destination.x as f32)
            / (src_right - src_left);
        let local_scale_y = ((destination.y as f32 + destination.height as f32)
            - destination.y as f32)
            / (src_bottom - src_top);
        let scale_x = local_scale_x * canvas.sx;
        let scale_y = local_scale_y * canvas.sy;
        let inv_x = 1.0 / scale_x;
        let inv_y = 1.0 / scale_y;
        let shader_tx = (destination.x as f32 - src_left * local_scale_x
            + ox as f32 * local_scale_x)
            * canvas.sx
            + canvas.tx;
        let shader_ty = (destination.y as f32 - src_top * local_scale_y
            + oy as f32 * local_scale_y)
            * canvas.sy
            + canvas.ty;
        let fixed = |v: f32| (f64::from(v) * 4_294_967_296.0) as i64;
        let step = fixed(inv_x);
        let format = output.format;
        for y in y_start..y_end {
            let shift_y =
                ((crate::cpu::raster_pipeline::tile_shift((y as i32 + tile_origin.1).max(0) as u32)
                    as i32
                    - tile_origin.1)
                    .max(device_origin.1 as i32)) as f32;
            let fy = fixed((y as f32 - shift_y + 0.5) * inv_y - (shader_ty - shift_y) * inv_y)
                - (1_i64 << 31);
            let mut span = None;
            let mut fx = 0;
            let mut span_count = 0;
            let mut previous_coverage = 0;
            for x in x_start..x_end {
                let index = (y as usize * output.width() as usize + x as usize) * 4;
                if clip.is_some_and(|m| m.alpha_at(x as u32, y as u32) == 0) {
                    span = None;
                    continue;
                }
                let shift_x = ((crate::cpu::raster_pipeline::tile_shift(
                    (x as i32 + tile_origin.0).max(0) as u32,
                ) as i32
                    - tile_origin.0)
                    .max(device_origin.0 as i32));
                let coverage = clip.map_or(255, |m| m.alpha_at(x as u32, y as u32));
                if span != Some(shift_x)
                    || span_count == 127
                    || coverage != previous_coverage
                    || clip_runs.is_some_and(|runs| runs[index / 4])
                {
                    fx = fixed(
                        (x as f32 - shift_x as f32 + 0.5) * inv_x
                            - (shader_tx - shift_x as f32) * inv_x,
                    ) - (1_i64 << 31);
                    span = Some(shift_x);
                    span_count = 0;
                }
                let src = format.swizzle(previous_sample_bilinear_bitmap(image, fx, fy));
                fx += step;
                span_count += 1;
                previous_coverage = coverage;
                if src == [0; 4] {
                    if format == crate::raster::PixelFormat::Bgrx8888 {
                        output.data_mut()[index + 3] = 0;
                    }
                    continue;
                }
                let dst = &mut output.data_mut()[index..index + 4];
                if coverage != 255 {
                    // SkARGB32_Shader_Blitter::blitAntiH delegates partial
                    // coverage to SkBlitRow::Factory32 (NEON combined products).
                    let scale = u32::from(coverage) + 1;
                    let inverse = if src[3] == 255 {
                        256 - scale
                    } else {
                        let p = 65535 - u32::from(src[3]) * scale;
                        (p + (p >> 8)) >> 8
                    };
                    for c in 0..4 {
                        dst[c] = ((u32::from(src[c]) * scale + u32::from(dst[c]) * inverse) >> 8)
                            .min(255) as u8;
                    }
                    if format == crate::raster::PixelFormat::Bgrx8888 {
                        dst[3] = 0;
                    }
                    continue;
                }
                for c in 0..4 {
                    // LOWP_STAGE_PP(srcover_rgba_8888), NEON div255.
                    let product = u32::from(dst[c]) * (255 - u32::from(src[3]));
                    let v = product + 128;
                    dst[c] = (u32::from(src[c]) + ((v + (v >> 8)) >> 8)) as u8;
                }
                if format == crate::raster::PixelFormat::Bgrx8888 {
                    dst[3] = 0;
                }
            }
        }
        true
    }

    #[test]
    fn integer_image_dispatch_matches_previous_mapper_with_clips_and_formats() {
        let rgba: Vec<u8> = (0..37 * 19)
            .flat_map(|i| {
                let a = (i * 43 + 29) as u8;
                [a / 2, a / 3, a, a]
            })
            .collect();
        let image = Pixmap::from_vec(rgba, IntSize::from_wh(37, 19).unwrap()).unwrap();
        for format in [
            crate::raster::PixelFormat::Rgba8888,
            crate::raster::PixelFormat::Bgra8888,
            crate::raster::PixelFormat::Bgrx8888,
        ] {
            for scale in [1.0, 2.0] {
                for (x, y) in [(2.0, 3.0), (128.0, 127.0), (-3.0, -2.0)] {
                    for ratio in [0.8, 1.0, 1.2, 1.5] {
                        for source in [
                            PaintRect {
                                x: 0.0,
                                y: 0.0,
                                width: 37.0,
                                height: 19.0,
                            },
                            PaintRect {
                                x: 4.0,
                                y: 6.0,
                                width: 29.0,
                                height: 11.0,
                            },
                        ] {
                            let bounds = PaintRect {
                                x,
                                y,
                                width: source.width * ratio / f64::from(scale),
                                height: source.height * ratio / f64::from(scale),
                            };
                            let mut mask = Mask::new(320, 300).unwrap();
                            for (i, v) in mask.data_mut().iter_mut().enumerate() {
                                *v = [0, 1, 127, 128, 254, 255][i % 6];
                            }
                            let runs: Vec<bool> = (0..320 * 300).map(|i| i % 11 == 0).collect();
                            let sparse: Vec<usize> = runs
                                .iter()
                                .enumerate()
                                .filter_map(|(i, &v)| v.then_some(i))
                                .collect();
                            for clipped in [false, true] {
                                let mut actual = Pixmap::from_vec(
                                    vec![127; 320 * 300 * 4],
                                    IntSize::from_wh(320, 300).unwrap(),
                                )
                                .unwrap();
                                actual.format = format;
                                let mut expected = actual.clone();
                                let transform = Transform::from_scale(scale, scale);
                                let clip = if clipped { Some(&mask) } else { None };
                                assert!(previous_draw_image_bitmap(
                                    &mut expected,
                                    clip,
                                    transform,
                                    &image,
                                    bounds,
                                    source,
                                    (0, 0),
                                    (0, 0),
                                    Some(&runs)
                                ));
                                assert!(draw_image_bitmap(
                                    &mut actual,
                                    clip,
                                    transform,
                                    &image,
                                    bounds,
                                    source,
                                    (0, 0),
                                    (0, 0),
                                    Some(&sparse),
                                    None,
                                ));
                                assert_eq!(actual.data(),expected.data(),"format={format:?},scale={scale},source={source:?},bounds={bounds:?},clip={clipped}");
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn bitmap_span_restarts_match_scalar_at_tile_layer_and_aa_run_boundaries() {
        let image = Pixmap::from_vec(
            (0..37 * 19)
                .flat_map(|i| {
                    let a = (i * 43 + 29) as u8;
                    [a / 2, a / 3, a, a]
                })
                .collect(),
            IntSize::from_wh(37, 19).unwrap(),
        )
        .unwrap();
        let size = IntSize::from_wh(1037, 39).unwrap();
        let mut mask = Mask::new(1037, 39).unwrap();
        for (i, alpha) in mask.data_mut().iter_mut().enumerate() {
            *alpha = [0, 1, 127, 128, 254, 255][(i / 139) % 6];
        }
        let runs: Vec<bool> = (0..1037 * 39).map(|i| i % 83 == 0).collect();
        let sparse: Vec<usize> = runs
            .iter()
            .enumerate()
            .filter_map(|(i, &v)| v.then_some(i))
            .collect();
        for origin in [(-600, -300), (-13, -7), (0, 0), (253, 255), (999, 17)] {
            for device in [(0, 0), (33, 7), (300, 17), (700, 300)] {
                for clip in [None, Some(&mask)] {
                    for format in [
                        crate::PixelFormat::Rgba8888,
                        crate::PixelFormat::Bgra8888,
                        crate::PixelFormat::Bgrx8888,
                    ] {
                        let mut actual = Pixmap::from_vec(vec![127; 1037 * 39 * 4], size).unwrap();
                        actual.format = format;
                        let mut expected = actual.clone();
                        let destination = PaintRect {
                            x: -3.25,
                            y: 1.3,
                            width: 1039.7,
                            height: 35.2,
                        };
                        let source = PaintRect {
                            x: 0.0,
                            y: 0.0,
                            width: 37.0,
                            height: 19.0,
                        };
                        assert!(previous_draw_image_bitmap(
                            &mut expected,
                            clip,
                            Transform::identity(),
                            &image,
                            destination,
                            source,
                            device,
                            origin,
                            Some(&runs)
                        ));
                        assert!(draw_image_bitmap(
                            &mut actual,
                            clip,
                            Transform::identity(),
                            &image,
                            destination,
                            source,
                            device,
                            origin,
                            Some(&sparse),
                            None
                        ));
                        assert_eq!(
                            actual.data(),
                            expected.data(),
                            "origin={origin:?} device={device:?} clip={} format={format:?}",
                            clip.is_some()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn bitmap_bw_region_matches_dense_clip_with_same_mapper_phase() {
        let image = Pixmap::from_vec(
            (0..37 * 19)
                .flat_map(|i| {
                    let a = (i * 43 + 29) as u8;
                    [a / 2, a / 3, a, a]
                })
                .collect(),
            IntSize::from_wh(37, 19).unwrap(),
        )
        .unwrap();
        for format in [
            crate::PixelFormat::Rgba8888,
            crate::PixelFormat::Bgra8888,
            crate::PixelFormat::Bgrx8888,
        ] {
            for scale in [1.0, 1.5, 2.0] {
                for (left, top, width, height) in
                    [(0, 0, 320, 300), (12, 11, 23, 15), (258, 249, 19, 17)]
                {
                    let mut dense = Mask::new(320, 300).unwrap();
                    for y in top..top + height {
                        dense.data_mut()[y * 320 + left..y * 320 + left + width].fill(255);
                    }
                    let summary = crate::src::core::SkRasterClip::SkRasterClip::from_mask(
                        320,
                        300,
                        dense.data(),
                    )
                    .unwrap();
                    for (x, y) in [(-3.125, -1.75), (7.5, 9.125), (245.0, 240.0)] {
                        let destination = PaintRect {
                            x,
                            y,
                            width: 37.0 * 1.3 / scale,
                            height: 19.0 * 1.3 / scale,
                        };
                        let source = PaintRect {
                            x: 0.0,
                            y: 0.0,
                            width: 37.0,
                            height: 19.0,
                        };
                        let mut expected = Pixmap::from_vec(
                            vec![127; 320 * 300 * 4],
                            IntSize::from_wh(320, 300).unwrap(),
                        )
                        .unwrap();
                        expected.format = format;
                        let mut actual = expected.clone();
                        let transform = Transform::from_scale(scale as f32, scale as f32);
                        assert!(previous_draw_image_bitmap(
                            &mut expected,
                            Some(&dense),
                            transform,
                            &image,
                            destination,
                            source,
                            (0, 0),
                            (0, 0),
                            None
                        ));
                        assert!(draw_image_bitmap(
                            &mut actual,
                            None,
                            transform,
                            &image,
                            destination,
                            source,
                            (0, 0),
                            (0, 0),
                            None,
                            Some(&summary)
                        ));
                        assert_eq!(actual.data(), expected.data(), "format={format:?} scale={scale} region={left},{top},{width},{height} destination={destination:?}");
                    }
                }
            }
        }
    }

    fn previous_image_src_over(
        dst: &mut [u8],
        src: &[u8],
        coverage: u8,
        format: crate::raster::PixelFormat,
    ) {
        if coverage == 0 {
            return;
        }
        for (d, s) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
            let s = format.swizzle(s.try_into().unwrap());
            if coverage == 255 {
                for c in 0..4 {
                    let p = u32::from(d[c]) * (255 - u32::from(s[3])) + 128;
                    d[c] = (u32::from(s[c]) + ((p + (p >> 8)) >> 8)) as u8;
                }
            } else {
                let scale = u32::from(coverage) + 1;
                let inverse = if s[3] == 255 {
                    256 - scale
                } else {
                    let p = 65535 - u32::from(s[3]) * scale;
                    (p + (p >> 8)) >> 8
                };
                for c in 0..4 {
                    d[c] =
                        ((u32::from(s[c]) * scale + u32::from(d[c]) * inverse) >> 8).min(255) as u8;
                }
            }
            if format == crate::raster::PixelFormat::Bgrx8888 {
                d[3] = 0;
            }
        }
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    #[test]
    #[ignore = "manual Debug timing; no performance assertion"]
    fn image_row_debug_kernel_benchmark() {
        use std::time::Instant;
        for format in [
            crate::raster::PixelFormat::Rgba8888,
            crate::raster::PixelFormat::Bgrx8888,
        ] {
            for coverage in [127, 255] {
                for width in [16, 64, 127, 540, 1024] {
                    let source: Vec<_> = (0..width)
                        .flat_map(|i| {
                            let a = (i * 43 + 29) as u8;
                            [a / 2, a / 3, a, a]
                        })
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
                                if old {
                                    let offset = unsafe {
                                        image_src_over_row_intrinsics(
                                            std::hint::black_box(&mut dst),
                                            std::hint::black_box(&source),
                                            coverage,
                                            format,
                                        )
                                    };
                                    previous_image_src_over(
                                        &mut dst[offset..],
                                        &source[offset..],
                                        coverage,
                                        format,
                                    );
                                } else {
                                    image_src_over_row(
                                        std::hint::black_box(&mut dst),
                                        std::hint::black_box(&source),
                                        coverage,
                                        format,
                                    );
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
                    eprintln!("IMAGE_ROW_BENCH format={format:?} coverage={coverage} width={width} pixels={} before_intrinsics_ms={:.3} after_direct_ms={:.3} speedup={:.2}",width*iterations,old_time[1],new_time[1],old_time[1]/new_time[1]);
                }
            }
        }
    }

    #[test]
    fn image_integer_row_all_alpha_coverage_and_layouts_match_previous_pixels() {
        for format in [
            crate::raster::PixelFormat::Rgba8888,
            crate::raster::PixelFormat::Bgra8888,
            crate::raster::PixelFormat::Bgrx8888,
        ] {
            for alpha in 0..=255u8 {
                let src: Vec<u8> = (0..17)
                    .flat_map(|i| {
                        let a = if i == 0 {
                            255
                        } else if i == 1 {
                            0
                        } else {
                            alpha
                        };
                        [a / 2, a, a / 3, a]
                    })
                    .collect();
                for coverage in 0..=255u8 {
                    let mut row: Vec<u8> = (0..17)
                        .flat_map(|i| {
                            [
                                (i * 13) as u8,
                                (i * 37) as u8,
                                (i * 61) as u8,
                                (i * 73) as u8,
                            ]
                        })
                        .collect();
                    let mut expected = row.clone();
                    previous_image_src_over(&mut expected, &src, coverage, format);
                    image_src_over_row(&mut row, &src, coverage, format);
                    assert_eq!(
                        row, expected,
                        "alpha={alpha},coverage={coverage},format={format:?}"
                    );
                }
            }
        }
        for offset in 0..8 {
            for count in 0..=35 {
                let mut source = vec![0xa7; offset + count * 4 + 7];
                for (i, p) in source[offset..offset + count * 4]
                    .chunks_exact_mut(4)
                    .enumerate()
                {
                    let a = (i * 19) as u8;
                    p.copy_from_slice(&[a, a / 2, a / 3, a]);
                }
                let mut dest = vec![0xb1; offset + count * 4 + 9];
                let mut expected = dest.clone();
                previous_image_src_over(
                    &mut expected[offset..offset + count * 4],
                    &source[offset..offset + count * 4],
                    127,
                    crate::raster::PixelFormat::Bgra8888,
                );
                image_src_over_row(
                    &mut dest[offset..offset + count * 4],
                    &source[offset..offset + count * 4],
                    127,
                    crate::raster::PixelFormat::Bgra8888,
                );
                assert_eq!(dest, expected, "offset={offset},count={count}");
            }
        }
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64", target_os = "macos"))]
    #[test]
    fn image_integer_rows_do_not_cross_guard_pages() {
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
            let src = source.tail(count * 4);
            for (i, p) in src.chunks_exact_mut(4).enumerate() {
                let a = [0, 1, 6, 128, 254, 255][i % 6];
                p.copy_from_slice(&[a / 2, a / 3, a, a]);
            }
            for coverage in [0, 1, 2, 127, 128, 254, 255] {
                for format in [
                    crate::raster::PixelFormat::Rgba8888,
                    crate::raster::PixelFormat::Bgra8888,
                    crate::raster::PixelFormat::Bgrx8888,
                ] {
                    let dst = destination.tail(count * 4);
                    dst.fill(0x73);
                    let mut expected = dst.to_vec();
                    previous_image_src_over(&mut expected, src, coverage, format);
                    image_src_over_row(dst, src, coverage, format);
                    assert_eq!(
                        dst, expected,
                        "count={count},coverage={coverage},format={format:?}"
                    );
                }
            }
        }
    }

    // Independent pre-translation intrinsic helper retained only as an oracle.
    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    unsafe fn bitmap_filter_previous_intrinsics(
        x: u32,
        y: u32,
        [a00, a01, a10, a11]: [u32; 4],
    ) -> u32 {
        use core::arch::aarch64::*;
        let vy = vdup_n_u8(y as u8);
        let inverse_y = vsub_u8(vdup_n_u8(16), vy);
        let top = vset_lane_u32::<1>(a01, vdup_n_u32(a00));
        let bottom = vset_lane_u32::<1>(a11, vdup_n_u32(a10));
        let weighted_top = vmull_u8(vreinterpret_u8_u32(top), inverse_y);
        let weighted_bottom = vmull_u8(vreinterpret_u8_u32(bottom), vy);
        let vx = vdup_n_u16(x as u16);
        let inverse_x = vsub_u16(vdup_n_u16(16), vx);
        let sum = vmul_u16(vget_high_u16(weighted_top), vx);
        let sum = vmla_u16(sum, vget_high_u16(weighted_bottom), vx);
        let sum = vmla_u16(sum, vget_low_u16(weighted_top), inverse_x);
        let sum = vmla_u16(sum, vget_low_u16(weighted_bottom), inverse_x);
        let bytes = vshrn_n_u16::<8>(vcombine_u16(sum, vdup_n_u16(0)));
        vget_lane_u32::<0>(vreinterpret_u32_u8(bytes))
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    #[test]
    fn bitmap_filter_every_weight_channel_and_neighbor_match_previous_intrinsics() {
        for y in 0..16 {
            for x in 0..16 {
                for channel in 0..4 {
                    for corner in 0..4 {
                        for value in 0..=255 {
                            let mut pixels = [0xff7f0100u32, 0x0180feff, 0xfeff007f, 0x8001fffe];
                            let shift = channel * 8;
                            pixels[corner] = (pixels[corner] & !(255 << shift)) | (value << shift);
                            let expected = filter_and_scale_by_alpha_portable(x, y, pixels);
                            unsafe {
                                assert_eq!(
                                    bitmap_filter_previous_intrinsics(x, y, pixels),
                                    expected
                                );
                                assert_eq!(
                                    filter_and_scale_by_alpha_neon(x, y, pixels),
                                    expected,
                                    "x={x},y={y},channel={channel},corner={corner},value={value}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    #[test]
    #[ignore = "manual Debug timing; no performance assertion"]
    fn bitmap_filter_debug_million_samples_benchmark() {
        use std::{hint::black_box, time::Instant};
        let cases: Vec<_> = (0..1_048_576u32)
            .map(|i| {
                let x = i & 15;
                let y = (i >> 4) & 15;
                let pixels = [
                    i.wrapping_mul(0x13713543),
                    i.wrapping_mul(0x01032753),
                    i.wrapping_mul(0x73611937).wrapping_add(0x801020ff),
                    i.wrapping_mul(0x37711961).wrapping_add(0x00ff807f),
                ];
                (x, y, pixels)
            })
            .collect();
        let mut previous = Vec::new();
        let mut direct = Vec::new();
        for _ in 0..7 {
            let start = Instant::now();
            let mut checksum = 0u32;
            for &(x, y, pixels) in black_box(&cases) {
                checksum ^= unsafe { bitmap_filter_previous_intrinsics(x, y, pixels) };
            }
            let old_ms = start.elapsed().as_secs_f64() * 1000.0;
            let expected = black_box(checksum);
            let start = Instant::now();
            let mut checksum = 0u32;
            for &(x, y, pixels) in black_box(&cases) {
                checksum ^= unsafe { filter_and_scale_by_alpha_neon(x, y, pixels) };
            }
            let new_ms = start.elapsed().as_secs_f64() * 1000.0;
            assert_eq!(black_box(checksum), expected);
            previous.push(old_ms);
            direct.push(new_ms);
        }
        previous.sort_by(f64::total_cmp);
        direct.sort_by(f64::total_cmp);
        eprintln!("BITMAP_FILTER_BENCH samples={} previous_intrinsics_ms={previous:?} direct_register_ms={direct:?} speedup={:.3}",cases.len(),previous[3]/direct[3]);
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64", target_os = "macos"))]
    #[test]
    fn bitmap_filter_clamped_source_rows_do_not_cross_guard_pages() {
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

        let mut top_allocation = GuardedBuffer::new();
        let mut bottom_allocation = GuardedBuffer::new();
        for width in [1, 2, 3, 15, 16, 17, 126, 127, 128] {
            let top = top_allocation.tail(width * 4);
            let bottom = bottom_allocation.tail(width * 4);
            for (i, p) in top.chunks_exact_mut(4).enumerate() {
                let alpha = (i * 73 + 127) as u8;
                p.copy_from_slice(&[alpha / 3, alpha / 2, alpha, alpha]);
            }
            for (i, p) in bottom.chunks_exact_mut(4).enumerate() {
                let alpha = (i * 43 + 255) as u8;
                p.copy_from_slice(&[alpha, alpha / 2, alpha / 3, alpha]);
            }
            for wy in 0..16 {
                let rows = BitmapSampleRows {
                    top,
                    bottom,
                    width: width as u32,
                    weight_y: wy,
                };
                for integer in [
                    -2,
                    -1,
                    0,
                    width as i64 - 2,
                    width as i64 - 1,
                    width as i64,
                    width as i64 + 1,
                ] {
                    for wx in 0..16 {
                        let qx = (integer << 32) | (i64::from(wx) << 28);
                        let x0 = integer.clamp(0, width as i64 - 1) as usize;
                        let x1 = (integer + 1).clamp(0, width as i64 - 1) as usize;
                        let pixel = |row: &[u8], x| {
                            u32::from_le_bytes(row[x * 4..x * 4 + 4].try_into().unwrap())
                        };
                        let expected = filter_and_scale_by_alpha_portable(
                            wx,
                            wy,
                            [
                                pixel(top, x0),
                                pixel(top, x1),
                                pixel(bottom, x0),
                                pixel(bottom, x1),
                            ],
                        );
                        assert_eq!(
                            rows.sample(qx),
                            expected.to_le_bytes(),
                            "width={width},integer={integer},wx={wx},wy={wy}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn bitmap_filter_all_fractional_weights_match_previous_four_neighbor_sum() {
        for y in 0..16 {
            for x in 0..16 {
                for seed in 0..=255u32 {
                    let pixels = [
                        seed.wrapping_mul(0x01032753),
                        seed.wrapping_mul(0x13613543).wrapping_add(0xffffffff),
                        seed.wrapping_mul(0x73611937).wrapping_add(0x801020ff),
                        seed.wrapping_mul(0x37711961).wrapping_add(0x00ff807f),
                    ];
                    let weights = [(16 - x) * (16 - y), x * (16 - y), (16 - x) * y, x * y];
                    let expected: [u8; 4] = std::array::from_fn(|channel| {
                        ((0..4)
                            .map(|i| u32::from(pixels[i].to_le_bytes()[channel]) * weights[i])
                            .sum::<u32>()
                            >> 8) as u8
                    });
                    assert_eq!(
                        filter_and_scale_by_alpha_portable(x, y, pixels).to_le_bytes(),
                        expected
                    );
                    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
                    unsafe {
                        assert_eq!(
                            filter_and_scale_by_alpha_neon(x, y, pixels).to_le_bytes(),
                            expected
                        );
                    }
                }
            }
        }
        // All four reads at a one-pixel source repeat exactly that pixel.
        let source =
            Pixmap::from_vec(vec![11, 23, 37, 59], IntSize::from_wh(1, 1).unwrap()).unwrap();
        for x in [-1_i64 << 32, 0, 1_i64 << 32, 100_i64 << 32] {
            assert_eq!(sample_bilinear_bitmap(&source, x, x), [11, 23, 37, 59]);
        }
    }

    // Independent previous nested-tap algorithm checks all SIMD kernels/tails.
    fn downsample_reference(source: &Pixmap) -> Pixmap {
        let w = source.width();
        let h = source.height();
        let mut output = Pixmap::new((w / 2).max(1), (h / 2).max(1)).unwrap();
        let axis = |size| match size {
            1 => (&[1_u32][..], 0),
            size if size % 2 == 0 => (&[1_u32, 1][..], 1),
            _ => (&[1_u32, 2, 1][..], 2),
        };
        let (wx, shift_x) = axis(w);
        let (wy, shift_y) = axis(h);
        let row_bytes = w as usize * 4;
        let output_row_bytes = output.width() as usize * 4;
        for y in 0..output.height() as usize {
            for x in 0..output.width() as usize {
                let mut sum = [0_u32; 4];
                for (dy, weight_y) in wy.iter().enumerate() {
                    for (dx, weight_x) in wx.iter().enumerate() {
                        let offset = (y * 2 + dy) * row_bytes + (x * 2 + dx) * 4;
                        for channel in 0..4 {
                            sum[channel] +=
                                u32::from(source.data()[offset + channel]) * weight_x * weight_y;
                        }
                    }
                }
                let offset = y * output_row_bytes + x * 4;
                for channel in 0..4 {
                    output.data_mut()[offset + channel] =
                        (sum[channel] >> (shift_x + shift_y)) as u8;
                }
            }
        }
        output
    }

    #[test]
    fn mip_downsample_all_kernels_and_vector_tails_match_previous_bytes() {
        let dimensions = [
            1, 2, 3, 4, 7, 8, 9, 10, 15, 16, 17, 31, 32, 33, 65, 66, 67, 129,
        ];
        for width in dimensions {
            for height in dimensions {
                let source: Vec<u8> = (0..width * height)
                    .flat_map(|i| {
                        [
                            (i * 37) as u8,
                            (i * 53 + 71) as u8,
                            (i * 29 + 19) as u8,
                            (i * 61) as u8,
                        ]
                    })
                    .collect();
                let source =
                    Pixmap::from_vec(source, IntSize::from_wh(width, height).unwrap()).unwrap();
                let expected = downsample_reference(&source);
                let actual = downsample(&source);
                assert_eq!(actual.data(), expected.data(), "source {width}x{height}");
            }
        }
    }

    #[test]
    fn mip_resize_keeps_skia_forward_inverse_matrix_rounding() {
        let mut bytes = Vec::new();
        for y in 0..61 {
            for x in 0..73 {
                let c = ColorU8::from_rgba(
                    (x * 37 + y * 17) as u8,
                    (x * 13 + y * 53) as u8,
                    (x * 73 + y * 29) as u8,
                    ((x * 43 + y * 61 + 29) % 256) as u8,
                )
                .premultiply();
                bytes.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
            }
        }
        let source = Pixmap::from_vec(bytes, IntSize::from_wh(73, 61).unwrap()).unwrap();
        let actual = build_mip_image(&source, 1);
        assert_eq!((actual.width(), actual.height()), (37, 31));
        // Native SkPixmap::scalePixels(kLinear,kNearest), generated by
        // artifacts/baidu-bounded-raster-20261002/image-device/mip73-oracle.cc.
        // This requested half-size has nearest mip level zero due to Skia's
        // -0.5 bias. Reciprocal(37/73) differs from 73/37 by one float ULP,
        // changing x34's 16.16 quantization and its red channel by one byte.
        assert_eq!(&actual.data()[2060..2064], &[88, 136, 41, 172]);
        let hash = actual.data().iter().fold(0xcbf29ce484222325_u64, |h, &b| {
            (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
        });
        assert_eq!(hash, 0xf04642f8ec53d270);
    }

    #[test]
    fn mipmaps_match_source_skia_for_odd_even_thin_and_transparent_images() {
        let mut fixtures = include_bytes!("../tests/fixtures/image_mipmap.rgba").as_slice();
        let mut cases = 0;
        let mut differing_cases = 0;
        let mut first_difference = None;
        while !fixtures.is_empty() {
            let fields: Vec<_> = fixtures[..24]
                .chunks_exact(4)
                .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
                .collect();
            fixtures = &fixtures[24..];
            let [w, h, alpha, level, target_w, target_h] = fields.try_into().unwrap();
            let length = (target_w * target_h * 4) as usize;
            let expected = &fixtures[..length];
            fixtures = &fixtures[length..];
            let mut input = Vec::new();
            for y in 0..h {
                for x in 0..w {
                    let color = ColorU8::from_rgba(
                        ((x * 37 + y * 17 + 11) % 256) as u8,
                        ((x * 13 + y * 53 + 71) % 256) as u8,
                        ((x * 73 + y * 29 + 19) % 256) as u8,
                        if alpha != 0 {
                            ((x * 43 + y * 61 + 29) % 256) as u8
                        } else {
                            255
                        },
                    )
                    .premultiply();
                    input.extend_from_slice(&[
                        color.red(),
                        color.green(),
                        color.blue(),
                        color.alpha(),
                    ]);
                }
            }
            let source = Pixmap::from_vec(input, IntSize::from_wh(w, h).unwrap()).unwrap();
            let actual = build_mip_image(&source, level);
            assert_eq!((actual.width(), actual.height()), (target_w, target_h));
            if actual.data() != expected {
                differing_cases += 1;
                first_difference.get_or_insert_with(|| {
                    format!(
                        "{w}x{h} alpha={alpha} level={level}: {:?} != {:?}",
                        actual.data(),
                        expected
                    )
                });
            }
            cases += 1;
        }
        assert_eq!(cases, 3_610);
        assert_eq!(
            differing_cases,
            0,
            "{}",
            first_difference.unwrap_or_default()
        );
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64", target_os = "macos"))]
    #[test]
    fn mip_neon_rows_and_tails_do_not_cross_guard_pages() {
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

        fn check<const NX: usize, const NY: usize, const SHIFT: i32>() {
            let mut source = GuardedBuffer::new();
            let mut destination = GuardedBuffer::new();
            for count in 1..=21 {
                if NX == 1 && count != 1 {
                    continue;
                }
                let width = if NX == 1 {
                    1
                } else {
                    count * 2 + usize::from(NX == 3)
                };
                let row_bytes = width * 4;
                let input = source.tail(row_bytes * NY);
                for (index, byte) in input.iter_mut().enumerate() {
                    *byte = (index * 37 + 71) as u8;
                }
                let mut expected = vec![0; count * 4];
                downsample_row_portable::<NX, NY, SHIFT>(input, row_bytes, &mut expected, 0);
                let output = destination.tail(count * 4);
                unsafe {
                    downsample_row_neon::<NX, NY, SHIFT>(input, row_bytes, output);
                }
                assert_eq!(output, expected, "{NX}x{NY} taps count={count}");
            }
        }
        check::<1, 2, 1>();
        check::<1, 3, 2>();
        check::<2, 1, 1>();
        check::<2, 2, 2>();
        check::<2, 3, 3>();
        check::<3, 1, 2>();
        check::<3, 2, 3>();
        check::<3, 3, 4>();
        // The lowp gather row also ends directly against PROT_NONE: singleton
        // sources, clamped left/right taps and unaligned destination tails.
        let mut source = GuardedBuffer::new();
        let mut destination = GuardedBuffer::new();
        for width in [1, 2, 7, 21] {
            let row_bytes = width * 4;
            let input = source.tail(row_bytes * 2);
            for (index, byte) in input.iter_mut().enumerate() {
                *byte = (index * 37 + 71) as u8;
            }
            let oracle = Pixmap::from_vec(
                input.to_vec(),
                crate::raster::IntSize::from_wh(width as u32, 2).unwrap(),
            )
            .unwrap();
            for fraction in [0, 1, 32767, 32768, 65535] {
                let cy = 0.5 + fraction as f32 / 65536.0;
                let qy = (65536.0 * cy + 0.5).floor() as i32 - 32768;
                let ty = (qy ^ 0x8000) as i16;
                for count in 1..=21 {
                    let centers: Vec<f32> = (0..count)
                        .map(|x| x as f32 - 2.5 + fraction as f32 / 65536.0)
                        .collect();
                    let xs: Vec<[i32; 3]> = centers
                        .iter()
                        .map(|&cx| {
                            let q = (65536.0 * cx + 0.5).floor() as i32 - 32768;
                            let i = q >> 16;
                            [
                                i.clamp(0, width as i32 - 1),
                                (i + 1).clamp(0, width as i32 - 1),
                                i32::from((q ^ 0x8000) as i16),
                            ]
                        })
                        .collect();
                    let output = destination.tail(count * 4);
                    unsafe {
                        sample_lowp_row_neon(
                            &input[..row_bytes],
                            &input[row_bytes..],
                            &xs,
                            ty,
                            output,
                        );
                    }
                    for (x, &cx) in centers.iter().enumerate() {
                        assert_eq!(
                            &output[x * 4..x * 4 + 4],
                            sample_bilinear_lowp(&oracle, cx, cy)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn mip_selection_terminates_at_one_pixel_and_preserves_both_axes() {
        assert_eq!(mip_level_for_size(1, 1, 1, 1), 0);
        assert_eq!(mip_level_for_size(1, 8, 1, 1), 3);
        assert_eq!(mip_level_for_size(9, 1, 1, 1), 4);
        assert_eq!(mip_level_for_size(16, 16, 7, 7), 1);
        assert_eq!(mip_level_for_size(17, 17, 5, 5), 2);
        assert_eq!(mip_level_for_size(32, 8, 8, 8), 0);
    }
}

#[cfg(test)]
mod bitmap_row_tests {
    use super::*;
    #[test]
    fn bounded_row_matches_scalar_filter_for_fraction_clamp_stride_and_guards() {
        for width in [1, 2, 7, 31, 256] {
            let size = crate::raster::IntSize::from_wh(width, 3).unwrap();
            let bytes: Vec<u8> = (0..width as usize * 3 * 4)
                .map(|i| (i * 31 + i / 71) as u8)
                .collect();
            let image = Pixmap::from_vec(bytes, size).unwrap();
            for wy in 0..16 {
                for wx in 0..16 {
                    for start in [-3, 0, width as i64 - 1, width as i64 + 2] {
                        for step in [0, 1 << 28, 3 << 30, 1 << 32, 7 << 32] {
                            let qx = (start << 32) + (wx << 28);
                            let rows = BitmapSampleRows::new(&image, (1 << 32) + (wy << 28));
                            for (count, offset) in
                                [(0, 0), (1, 1), (7, 3), (8, 7), (15, 15), (127, 5)]
                            {
                                let mut target = vec![173u8; offset + count * 4 + 17];
                                rows.sample_row(qx, step, &mut target[offset..offset + count * 4]);
                                let mut q = qx;
                                for pixel in target[offset..offset + count * 4].chunks_exact(4) {
                                    let integer = q >> 32;
                                    let x0 = integer.clamp(0, width as i64 - 1) as usize;
                                    let x1 = (integer + 1).clamp(0, width as i64 - 1) as usize;
                                    let load = |row: &[u8], x: usize| {
                                        u32::from_le_bytes(
                                            row[x * 4..x * 4 + 4].try_into().unwrap(),
                                        )
                                    };
                                    let expected = filter_and_scale_by_alpha_portable(
                                        ((q >> 28) & 15) as u32,
                                        wy as u32,
                                        [
                                            load(rows.top, x0),
                                            load(rows.top, x1),
                                            load(rows.bottom, x0),
                                            load(rows.bottom, x1),
                                        ],
                                    )
                                    .to_le_bytes();
                                    assert_eq!(pixel,expected,"width={width}, wy={wy}, wx={wx}, start={start}, count={count}");
                                    q += step;
                                }
                                assert!(target[..offset]
                                    .iter()
                                    .chain(target[offset + count * 4..].iter())
                                    .all(|&v| v == 173));
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod mip_timing_round2 {
    use super::*;
    use crate::raster::IntSize;
    #[test]
    fn lowp_resize_rows_match_scalar_matrix_rounding_and_clamped_edges() {
        for (w, h) in [(1, 1), (1, 13), (13, 1), (3, 7), (17, 31), (64, 33)] {
            let data: Vec<u8> = (0..w * h * 4).map(|i| (i * 37 + i / 23) as u8).collect();
            let source = Pixmap::from_vec(data, IntSize::from_wh(w, h).unwrap()).unwrap();
            for (tw, th) in [(1, 1), (2, 3), (7, 9), (19, 33), (65, 37)] {
                for base in [(w, h), (w * 4 + 1, h * 4 + 3)] {
                    let output = resize_bilinear(&source, tw, th, base);
                    let inv_x = (w as f32 / base.0 as f32) * (1.0 / (tw as f32 / base.0 as f32));
                    let inv_y = (h as f32 / base.1 as f32) * (1.0 / (th as f32 / base.1 as f32));
                    for y in 0..th {
                        for x in 0..tw {
                            let expected = sample_bilinear_lowp(
                                &source,
                                (x as f32 + 0.5) * inv_x,
                                (y as f32 + 0.5) * inv_y,
                            );
                            let i = (y * tw + x) as usize * 4;
                            assert_eq!(
                                &output.data()[i..i + 4],
                                expected,
                                "source={w}x{h} target={tw}x{th} base={base:?} xy={x},{y}"
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    #[ignore = "manual Debug bitmap run timing, no speed assertion"]
    fn profile_bitmap_runs_round2() {
        let image = Pixmap::from_vec(
            (0..1025u32 * 577 * 4)
                .map(|i| (i.wrapping_mul(43) % 256) as u8)
                .collect(),
            IntSize::from_wh(1025, 577).unwrap(),
        )
        .unwrap();
        let mut clip = Mask::new(2560, 1446).unwrap();
        clip.data_mut().fill(255);
        for iteration in 0..5 {
            let mut output = Pixmap::new(2560, 1446).unwrap();
            let start = std::time::Instant::now();
            assert!(draw_image_bitmap(
                &mut output,
                Some(&clip),
                Transform::from_scale(2.0, 2.0),
                &image,
                PaintRect {
                    x: 64.0,
                    y: 438.0,
                    width: 365.0,
                    height: 205.0
                },
                PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 1025.0,
                    height: 577.0
                },
                (0, 0),
                (0, 0),
                None,
                None
            ));
            let elapsed = start.elapsed();
            let hash = output.data().iter().fold(0xcbf29ce484222325u64, |h, &b| {
                (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
            });
            eprintln!(
                "bitmap-runs-round2 iteration={iteration} ms={:.3} hash={hash:016x}",
                elapsed.as_secs_f64() * 1000.0
            );
            std::hint::black_box(output);
        }
    }
    #[test]
    #[ignore = "manual Debug mip timing, no speed assertion"]
    fn profile_mip_generation_round2() {
        for (w, h, level) in [(1024u32, 576u32, 2), (1025, 577, 2), (3840, 2160, 3)] {
            let data: Vec<u8> = (0..w * h * 4)
                .map(|i| (i.wrapping_mul(43) % 256) as u8)
                .collect();
            let source = Pixmap::from_vec(data, IntSize::from_wh(w, h).unwrap()).unwrap();
            for iteration in 0..5 {
                let start = std::time::Instant::now();
                let result = build_mip_image(&source, level);
                let elapsed = start.elapsed();
                let hash = result.data().iter().fold(0xcbf29ce484222325u64, |h, &b| {
                    (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
                });
                eprintln!("mip-round2 size={w}x{h} level={level} iteration={iteration} ms={:.3} hash={hash:016x}",elapsed.as_secs_f64()*1000.0);
                std::hint::black_box(result);
            }
        }
    }
}

#[cfg(test)]
mod bounded_image_clip_round6 {
    use super::*;
    use crate::src::core::SkRasterClip::SkRasterClip;
    #[test]
    fn image_clip_cursor_matches_partition_with_duplicate_and_backward_queries() {
        for runs in [
            vec![],
            vec![0, 0, 1, 127, 254, 255, 255, 256, 511, 1024],
            (0..20000).filter(|i| i % 31 == 0 || i % 255 == 0).collect(),
        ] {
            let mut cursor = ImageClipRunCursor::new("oracle");
            for index in (0..21000).step_by(17).chain((0..21000).rev().step_by(13)) {
                let expected = runs.get(runs.partition_point(|&i| i <= index)).copied();
                assert_eq!(cursor.next_after(&runs, index), expected, "index={index}");
                assert_eq!(cursor.next_after(&runs, index), expected, "repeated query");
            }
        }
    }
    #[test]
    fn bounded_bitmap_clip_retains_storage_and_exact_sampling_runs() {
        let bounds = crate::raster::IntRect::from_xywh(23, 31, 72, 49).unwrap();
        let mut mask = Mask::new_bounded(320, 300, bounds).unwrap();
        for y in 31..80 {
            for (x, a) in mask.row_range_mut(y, 23, 95).iter_mut().enumerate() {
                *a = match x % 7 {
                    0 => 0,
                    1 => 91,
                    2 => 183,
                    _ => 255,
                };
            }
        }
        let bytes = mask.allocated_bytes();
        let dense = Mask::from_vec(
            mask.clone().take(),
            crate::raster::IntSize::from_wh(320, 300).unwrap(),
        )
        .unwrap();
        let summary = SkRasterClip::from_mask_builder(&mask, Some(bounds)).unwrap();
        let mut image = Pixmap::new(128, 128).unwrap();
        for (i, p) in image.data_mut().chunks_exact_mut(4).enumerate() {
            p.copy_from_slice(&[
                (i * 13 % 180) as u8,
                (i * 31 % 180) as u8,
                (i * 73 % 180) as u8,
                191,
            ]);
        }
        for format in [
            crate::PixelFormat::Rgba8888,
            crate::PixelFormat::Bgra8888,
            crate::PixelFormat::Bgrx8888,
        ] {
            for size in [128.0, 217.25] {
                let rect = PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: size,
                    height: size,
                };
                let source = PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 128.0,
                    height: 128.0,
                };
                let mut actual = Pixmap::new(320, 300).unwrap();
                actual.format = format;
                let mut expected = actual.clone();
                assert!(draw_image_bitmap(
                    &mut actual,
                    Some(&mask),
                    Transform::identity(),
                    &image,
                    rect,
                    source,
                    (0, 0),
                    (0, 0),
                    None,
                    Some(&summary)
                ));
                assert!(draw_image_bitmap(
                    &mut expected,
                    Some(&dense),
                    Transform::identity(),
                    &image,
                    rect,
                    source,
                    (0, 0),
                    (0, 0),
                    None,
                    Some(&summary)
                ));
                assert_eq!(actual.data(), expected.data(), "{format:?} size={size}");
                assert_eq!(
                    mask.allocated_bytes(),
                    bytes,
                    "bitmap shader must keep packed clip"
                );
            }
        }
    }
}

#[cfg(test)]
mod parallel_bitmap_rows_tests {
    use super::*;
    #[test]
    fn parallel_bitmap_rows_match_serial_with_alpha_clips_runs_tiles_and_opaque_store() {
        let (width, height) = (280usize, 121usize);
        let mut clip = Mask::new_bounded(
            width as u32,
            height as u32,
            crate::raster::IntRect::from_xywh(37, 21, 242, 78).unwrap(),
        )
        .unwrap();
        for y in 21..99 {
            for (x, coverage) in clip.row_range_mut(y, 37, 279).iter_mut().enumerate() {
                *coverage = [0, 1, 91, 183, 254, 255][(x / 11 + y as usize) % 6];
            }
        }
        let runs: Vec<usize> = (0..width * height).filter(|i| i % 19 == 0).collect();
        for opaque in [false, true] {
            let source: Vec<u8> = (0..83 * 61)
                .flat_map(|i| {
                    let alpha = if opaque { 255 } else { (i * 43 + 29) as u8 };
                    [alpha / 2, alpha / 3, alpha, alpha]
                })
                .collect();
            let rows = BitmapDrawRows {
                // A borrowed ROI retains the full original 83-pixel stride.
                image: BitmapSubset {
                    data: &source,
                    stride: 83,
                    left: 5,
                    top: 7,
                    width: 71,
                    height: 45,
                },
                clip: (!opaque).then_some(&clip),
                clip_runs: Some(&runs),
                x_start: 37,
                x_end: 279,
                output_width: width,
                format: crate::raster::PixelFormat::Bgrx8888,
                inv_x: 0.25,
                inv_y: 0.37,
                shader_tx: -3.25,
                shader_ty: 2.375,
                step: (0.25f64 * 4_294_967_296.0) as i64,
                device_origin: (11, 13),
                tile_origin: (19, 23),
                known_opaque: opaque,
            };
            let initial: Vec<u8> = (0..width * height * 4)
                .map(|i| (i * 37 + 73) as u8)
                .collect();
            let mut serial = initial.clone();
            let mut parallel = initial.clone();
            rows.draw_partitioned(&mut serial, 21, 99, 1);
            rows.draw_partitioned(&mut parallel, 21, 99, 2);
            assert_eq!(parallel, serial, "opaque={opaque}");
            assert_eq!(&parallel[..21 * width * 4], &initial[..21 * width * 4]);
            assert_eq!(&parallel[99 * width * 4..], &initial[99 * width * 4..]);
        }
    }
}
