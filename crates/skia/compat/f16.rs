//! Local F16 accumulator. Uses Skia raster-pipeline load/store and src-over math;
//! it is not an implementation of the C++ SkSurface_Raster object.

// CPU implementation moved from renderer/f16_surface.rs.

//! SkRasterPipeline load/store_f16, scale_u8 and srcover for alpha layers.
//! Copyright Google LLC. BSD-3-Clause; glyphs/SKIA_LICENSE.
use crate::compat::commands::Color;
use half::f16;

#[derive(Clone, Copy)]
enum Clip<'a> {
    Dense(&'a [u8]),
    Mask(&'a crate::raster::Mask),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Written {
    Empty,
    Range { start: usize, end: usize },
    Full,
}

pub(crate) struct Surface {
    // Keep raw storage private: every mutable escape must invalidate tracking.
    pixels: Vec<[f16; 4]>,
    written: Written,
}
impl Surface {
    pub fn new(count: usize) -> Self {
        let pixels = if count == 0 {
            Vec::new()
        } else {
            // SkMallocPixelRef::MakeAllocate uses calloc. Let the allocator
            // supply zero pages instead of malloc followed by a full clear.
            let layout = std::alloc::Layout::array::<[f16; 4]>(count)
                .expect("F16 device allocation overflow");
            // SAFETY: use the global allocator and the exact Vec element layout.
            // Every half is initialized to valid +0 bits. Vec owns and frees
            // this allocation with the same capacity, alignment and size.
            unsafe {
                let pointer = std::alloc::alloc_zeroed(layout).cast::<[f16; 4]>();
                if pointer.is_null() {
                    std::alloc::handle_alloc_error(layout);
                }
                Vec::from_raw_parts(pointer, count, count)
            }
        };
        Self {
            pixels,
            written: Written::Empty,
        }
    }
    /// Unknown pixel provenance must retain the original full-device scan.
    #[allow(dead_code)]
    pub(crate) fn from_pixels(pixels: Vec<[f16; 4]>) -> Self {
        Self {
            pixels,
            written: Written::Full,
        }
    }
    #[allow(dead_code)]
    pub(crate) fn pixels(&self) -> &[[f16; 4]] {
        &self.pixels
    }
    #[allow(dead_code)]
    pub(crate) fn pixels_mut(&mut self) -> &mut [[f16; 4]] {
        self.written = Written::Full;
        &mut self.pixels
    }
    fn mark_written(&mut self, start: usize, end: usize) {
        debug_assert!(start < end && end <= self.pixels.len());
        self.written = match self.written {
            Written::Empty => Written::Range { start, end },
            Written::Range {
                start: old_start,
                end: old_end,
            } => Written::Range {
                start: start.min(old_start),
                end: end.max(old_end),
            },
            Written::Full => Written::Full,
        };
    }
    pub fn blend(&mut self, index: usize, color: Color, coverage: u8) {
        let alpha = color.alpha.clamp(0.0, 1.0);
        let source = [
            color.red.clamp(0.0, 1.0) * alpha,
            color.green.clamp(0.0, 1.0) * alpha,
            color.blue.clamp(0.0, 1.0) * alpha,
            alpha,
        ];
        self.blend_premultiplied(index, source, coverage);
    }
    pub fn blend_premultiplied(&mut self, index: usize, source: [f32; 4], coverage: u8) {
        let mask = f32::from(coverage) * (1.0 / 255.0);
        let source = source.map(|v| v * mask);
        assert!(index < self.pixels.len());
        self.mark_written(index, index + 1);
        let destination = &mut self.pixels[index];
        for c in 0..4 {
            destination[c] =
                f16::from_f32((1.0 - source[3]).mul_add(destination[c].to_f32(), source[c]));
        }
    }
    /// Solid-color SkRasterPipeline span, retaining F16 storage rounding and
    /// the accompanying N32 view after every blend. Clip coverage is resolved
    /// by the canvas before entering this constant-coverage span.
    pub(crate) fn blend_color_span(
        &mut self,
        start: usize,
        bytes: &mut [u8],
        color: Color,
        coverage: u8,
        format: crate::raster::PixelFormat,
    ) {
        assert_eq!(bytes.len() % 4, 0);
        let count = bytes.len() / 4;
        if count == 0 {
            return;
        }
        let alpha = color.alpha.clamp(0.0, 1.0);
        let mask = f32::from(coverage) * (1.0 / 255.0);
        let source = [
            color.red.clamp(0.0, 1.0) * alpha,
            color.green.clamp(0.0, 1.0) * alpha,
            color.blue.clamp(0.0, 1.0) * alpha,
            alpha,
        ]
        .map(|v| v * mask);
        let _ = &self.pixels[start..start + count];
        self.mark_written(start, start + count);
        let destination = &mut self.pixels[start..start + count];
        #[cfg(all(target_arch = "aarch64", target_endian = "little", feature = "simd"))]
        {
            solid_f16_span(destination, bytes, source, format);
            return;
        }
        #[cfg(not(all(target_arch = "aarch64", target_endian = "little", feature = "simd")))]
        for (pixel, dst) in destination.iter_mut().zip(bytes.chunks_exact_mut(4)) {
            for c in 0..4 {
                pixel[c] = f16::from_f32((1.0 - source[3]).mul_add(pixel[c].to_f32(), source[c]));
            }
            let rgba =
                pixel.map(|v| (v.to_f32() * 255.0).clamp(0.0, 255.0).round_ties_even() as u8);
            dst.copy_from_slice(&format.encode(rgba));
        }
    }

    /// HIGHP N32 image -> opacity -> coverage -> SrcOver into an F16 device.
    /// `source` uses the child device's premultiplied RGBA storage; `bytes`
    /// mirrors the rounded F16 result in the parent device's encoding.
    pub(crate) fn composite_n32_span(
        &mut self,
        start: usize,
        source: &[u8],
        bytes: &mut [u8],
        opacity: f32,
        coverage: u8,
        format: crate::raster::PixelFormat,
    ) {
        self.composite_n32_span_impl(start, source, bytes, opacity, coverage, format, true);
    }
    fn composite_n32_span_impl(
        &mut self,
        start: usize,
        source: &[u8],
        bytes: &mut [u8],
        opacity: f32,
        coverage: u8,
        format: crate::raster::PixelFormat,
        use_simd: bool,
    ) {
        assert_eq!(source.len(), bytes.len());
        assert_eq!(source.len() % 4, 0);
        let count = source.len() / 4;
        let _ = &self.pixels[start..start + count];
        if count == 0 || coverage == 0 {
            return;
        }
        self.mark_written(start, start + count);
        let destination = &mut self.pixels[start..start + count];
        #[cfg(all(target_arch = "aarch64", target_endian = "little", feature = "simd"))]
        if use_simd {
            composite_n32_f16_row(source, destination, bytes, opacity, coverage, format);
            return;
        }
        #[cfg(not(all(target_arch = "aarch64", target_endian = "little", feature = "simd")))]
        let _ = use_simd;
        let mask = f32::from(coverage) * (1.0 / 255.0);
        for ((src, pixel), dst) in source
            .chunks_exact(4)
            .zip(destination)
            .zip(bytes.chunks_exact_mut(4))
        {
            if src == [0; 4] {
                continue;
            }
            let source: [f32; 4] =
                core::array::from_fn(|c| (f32::from(src[c]) * (1.0 / 255.0) * opacity) * mask);
            for c in 0..4 {
                pixel[c] = f16::from_f32((1.0 - source[3]).mul_add(pixel[c].to_f32(), source[c]));
            }
            let rgba =
                pixel.map(|v| (v.to_f32() * 255.0).clamp(0.0, 255.0).round_ties_even() as u8);
            dst.copy_from_slice(&format.encode(rgba));
        }
    }

    pub fn rgba8(&self, index: usize) -> [u8; 4] {
        self.pixels[index].map(|v| (v.to_f32() * 255.0).clamp(0.0, 255.0).round_ties_even() as u8)
    }
    pub fn composite_into(
        &self,
        bytes: &mut [u8],
        parent: Option<&mut Surface>,
        opacity: f32,
        clip: Option<&[u8]>,
        format: crate::raster::PixelFormat,
        child_width: usize,
        parent_width: usize,
        origin: (i32, i32),
    ) {
        self.composite_into_impl(
            bytes,
            parent,
            opacity,
            clip,
            format,
            child_width,
            parent_width,
            origin,
            true,
        );
    }
    fn composite_into_impl(
        &self,
        bytes: &mut [u8],
        parent: Option<&mut Surface>,
        opacity: f32,
        clip: Option<&[u8]>,
        format: crate::raster::PixelFormat,
        child_width: usize,
        parent_width: usize,
        origin: (i32, i32),
        use_row_pipeline: bool,
    ) {
        self.composite_into_rows(
            bytes,
            parent,
            opacity,
            clip.map(Clip::Dense),
            format,
            child_width,
            parent_width,
            origin,
            use_row_pipeline,
        );
    }

    /// Local packed SkMask row driver; F16 arithmetic and the bounded SIMD
    /// row pipeline are shared with the original dense compatibility API.
    pub(crate) fn composite_into_mask(
        &self,
        bytes: &mut [u8],
        parent: Option<&mut Surface>,
        opacity: f32,
        clip: Option<&crate::raster::Mask>,
        format: crate::raster::PixelFormat,
        child_width: usize,
        parent_width: usize,
        origin: (i32, i32),
    ) {
        self.composite_into_rows(
            bytes,
            parent,
            opacity,
            clip.map(Clip::Mask),
            format,
            child_width,
            parent_width,
            origin,
            true,
        );
    }

    fn composite_into_rows(
        &self,
        bytes: &mut [u8],
        mut parent: Option<&mut Surface>,
        opacity: f32,
        clip: Option<Clip<'_>>,
        format: crate::raster::PixelFormat,
        child_width: usize,
        parent_width: usize,
        origin: (i32, i32),
        use_row_pipeline: bool,
    ) {
        let child_height = self.pixels.len() / child_width;
        let parent_height = bytes.len() / 4 / parent_width;
        let left = (-(origin.0 as i64)).max(0).min(child_width as i64) as usize;
        let mut top = (-(origin.1 as i64)).max(0).min(child_height as i64) as usize;
        let right = (parent_width as i64 - origin.0 as i64)
            .max(0)
            .min(child_width as i64) as usize;
        let mut bottom = (parent_height as i64 - origin.1 as i64)
            .max(0)
            .min(child_height as i64) as usize;
        // Untouched new storage is positive zero, which the original row
        // pipeline always skips. Restrict only complete rows; preserve the
        // original width, origin, horizontal spans, clip and float operations.
        match self.written {
            Written::Empty => return,
            Written::Range { start, end } => {
                top = top.max(start / child_width);
                bottom = bottom.min((end - 1) / child_width + 1);
            }
            Written::Full => {}
        }
        if left >= right || top >= bottom {
            return;
        }
        // The raster device's intersection is invariant across the row. Read
        // the four initialized half bit patterns as one word to reject exact
        // signed-zero pixels, avoiding four f16 floating comparisons.
        // Most saved layer rows lie wholly inside the packed parent clip
        // and borrow its bytes below. Allocate a zero-extending scratch row
        // only when a real row crosses those storage bounds.
        let mut clip_scratch = Vec::new();
        for y in top..bottom {
            let parent_y = (y as i64 + origin.1 as i64) as usize;
            let mut child_index = y * child_width + left;
            let mut i = parent_y * parent_width + (left as i64 + origin.0 as i64) as usize;
            let row_start = i;
            let row_clip = clip.map(|coverage| match coverage {
                Clip::Dense(bytes) => &bytes[i..i + right - left],
                Clip::Mask(mask) => {
                    let x = (i % parent_width) as u32;
                    let bounds = mask.storage_bounds();
                    if parent_y >= bounds.top() as usize
                        && parent_y < bounds.bottom() as usize
                        && x >= bounds.left() as u32
                        && x + (right - left) as u32 <= bounds.right() as u32
                    {
                        mask.row_range(parent_y as u32, x, x + (right - left) as u32)
                    } else {
                        clip_scratch.resize(right - left, 0);
                        mask.copy_row_range(parent_y as u32, x, &mut clip_scratch);
                        clip_scratch.as_slice()
                    }
                }
            });
            #[cfg(all(target_arch = "aarch64", feature = "simd"))]
            if use_row_pipeline && parent.is_none() && left < right {
                let count = right - left;
                composite_f16_n32_row(
                    &self.pixels[child_index..child_index + count],
                    &mut bytes[i * 4..(i + count) * 4],
                    opacity,
                    row_clip,
                    format,
                );
                continue;
            }
            #[cfg(not(all(target_arch = "aarch64", feature = "simd")))]
            let _ = use_row_pipeline;
            let mut x = left;
            while x < right {
                if right - x >= 8 && zero_f16_block(&self.pixels[child_index..child_index + 8]) {
                    child_index += 8;
                    i += 8;
                    x += 8;
                    continue;
                }
                x += 1;
                let src = &self.pixels[child_index];
                child_index += 1;
                i += 1;
                // SAFETY: [f16;4] occupies exactly eight initialized bytes;
                // an unaligned integer load accepts its two-byte alignment.
                let bits = unsafe { core::ptr::read_unaligned(src.as_ptr().cast::<u64>()) };
                if bits & 0x7fff_7fff_7fff_7fff == 0 {
                    continue;
                }
                let i = i - 1;
                let dst = &mut bytes[i * 4..i * 4 + 4];
                let source = src.map(|v| v.to_f32() * opacity);
                let source = if let Some(clip) = row_clip {
                    let coverage = f32::from(clip[i - row_start]) * (1.0 / 255.0);
                    source.map(|v| v * coverage)
                } else {
                    source
                };
                // This nested restore writes directly into the parent's F16
                // storage. Zero clip/opacity cannot exclude NaN arithmetic.
                if let Some(p) = parent.as_mut() {
                    p.mark_written(i, i + 1);
                }
                for c in 0..4 {
                    let memory = format.channel(c);
                    if c == 3 && format == crate::raster::PixelFormat::Bgrx8888 {
                        dst[3] = 0;
                        continue;
                    }
                    if let Some(p) = parent.as_mut() {
                        let value = (1.0 - source[3]).mul_add(p.pixels[i][c].to_f32(), source[c]);
                        p.pixels[i][c] = f16::from_f32(value);
                        dst[memory] = (p.pixels[i][c].to_f32() * 255.0)
                            .clamp(0.0, 255.0)
                            .round_ties_even() as u8;
                    } else {
                        // HIGHP srcover_rgba_8888 keeps the destination in byte
                        // units. Normalizing and scaling back adds a rounding step.
                        let value =
                            (1.0 - source[3]).mul_add(f32::from(dst[memory]), source[c] * 255.0);
                        dst[memory] = value.clamp(0.0, 255.0).round_ties_even() as u8;
                    }
                }
            }
        }
    }
}

// HIGHP constant_color -> scale_u8 -> load_f16 -> srcover -> store_f16.
// Reload the rounded half value before computing the N32 view, exactly as
// Surface::blend followed by Surface::rgba8 does. No fused conversion or
// unrounded float result is substituted for that intermediate storage.
// Source: SkRasterPipeline_opts.h at cacf77bdba7ba7df8ea7236d7e14b08c658ff368:
// ARM64 mad/from_half/to_half, scale_u8, srcover, store_f16 and to_unorm.
// The interleaved RGBA lane layout and N32 mirror are local storage adapters.
#[cfg(all(target_arch = "aarch64", target_endian = "little", feature = "simd"))]
fn solid_f16_span(
    destination: &mut [[f16; 4]],
    bytes: &mut [u8],
    source: [f32; 4],
    format: crate::raster::PixelFormat,
) {
    let shuffle: [u8; 16] = if format == crate::raster::PixelFormat::Rgba8888 {
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    } else {
        [2, 1, 0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    };
    let alpha_mask = if format == crate::raster::PixelFormat::Bgrx8888 {
        0x00ff_ffffu32
    } else {
        u32::MAX
    };
    // SAFETY: the caller supplies nonempty equally sized initialized F16 and
    // N32 spans. Each iteration loads/stores one eight-byte half pixel and
    // stores one four-byte N32 pixel. NEON conversion is available on AArch64;
    // no padded load or access beyond these slices is performed.
    unsafe {
        core::arch::asm!(
            "ldr q10, [{source}]",
            "ldr q15, [{shuffle}]",
            "dup v11.4s, {inverse:w}",
            "dup v12.4s, {maximum:w}",
            "movi v13.4s, #0",
            "2:",
            "ldr d1, [{halves}]",
            "fcvtl v1.4s, v1.4h",
            "mov v0.16b, v10.16b",
            "fmla v0.4s, v1.4s, v11.4s",
            "fcvtn v0.4h, v0.4s",
            "str d0, [{halves}], #8",
            "fcvtl v0.4s, v0.4h",
            "fmul v0.4s, v0.4s, v12.4s",
            "fmaxnm v0.4s, v0.4s, v13.4s",
            "fminnm v0.4s, v0.4s, v12.4s",
            "fcvtnu v0.4s, v0.4s",
            "xtn v0.4h, v0.4s",
            "xtn v0.8b, v0.8h",
            "tbl v0.16b, {{v0.16b}}, v15.16b",
            "fmov {word:w}, s0",
            "and {word:w}, {word:w}, {alpha_mask:w}",
            "str {word:w}, [{bytes}], #4",
            "subs {count}, {count}, #1",
            "b.ne 2b",
            source=in(reg) source.as_ptr(), shuffle=in(reg) shuffle.as_ptr(),
            inverse=in(reg) (1.0-source[3]).to_bits(), maximum=in(reg) 255.0f32.to_bits(),
            alpha_mask=in(reg) alpha_mask,
            halves=inout(reg) destination.as_mut_ptr()=>_, bytes=inout(reg) bytes.as_mut_ptr()=>_,
            count=inout(reg) destination.len()=>_, word=out(reg) _,
            out("v0") _, out("v1") _, out("v10") _, out("v11") _,
            out("v12") _, out("v13") _, out("v15") _, options(nostack),
        );
    }
}

// SkRasterPipeline_opts.h ARM64 from_byte / scale_1_float / scale_u8 /
// srcover / store_f16. Retain each source multiplication, fused SrcOver,
// and half-storage rounding before updating the local N32 mirror.
#[cfg(all(target_arch = "aarch64", target_endian = "little", feature = "simd"))]
fn composite_n32_f16_row(
    source: &[u8],
    destination: &mut [[f16; 4]],
    bytes: &mut [u8],
    opacity: f32,
    coverage: u8,
    format: crate::raster::PixelFormat,
) {
    let shuffle: [u8; 16] = if format == crate::raster::PixelFormat::Rgba8888 {
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    } else {
        [2, 1, 0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    };
    let alpha_mask = if format == crate::raster::PixelFormat::Bgrx8888 {
        0x00ff_ffffu32
    } else {
        u32::MAX
    };
    // SAFETY: the caller checks equally sized complete N32/F16 spans and
    // nonempty count. Every iteration reads exactly one four-byte source,
    // reads/writes one eight-byte half pixel, and writes one four-byte mirror.
    // Transparent source is skipped before floating arithmetic, as in the
    // scalar restore; no padding, tail read, or nonzero-source omission occurs.
    unsafe {
        core::arch::asm!(
            "dup v10.4s, {normalizer:w}",
            "dup v11.4s, {opacity:w}",
            "dup v12.4s, {coverage:w}",
            "dup v13.4s, {one:w}",
            "dup v14.4s, {maximum:w}",
            "movi v16.4s, #0",
            "ldr q15, [{shuffle}]",
            "2:",
            "ldr {word:w}, [{source}], #4",
            "cbz {word:w}, 3f",
            "movi v0.16b, #0",
            "fmov s0, {word:w}",
            "uxtl v0.8h, v0.8b",
            "uxtl v0.4s, v0.4h",
            "ucvtf v0.4s, v0.4s",
            "fmul v0.4s, v0.4s, v10.4s",
            "fmul v0.4s, v0.4s, v11.4s",
            "fmul v0.4s, v0.4s, v12.4s",
            "dup v1.4s, v0.s[3]",
            "fsub v1.4s, v13.4s, v1.4s",
            "ldr d2, [{halves}]",
            "fcvtl v2.4s, v2.4h",
            "fmla v0.4s, v2.4s, v1.4s",
            "fcvtn v0.4h, v0.4s",
            "str d0, [{halves}]",
            "fcvtl v0.4s, v0.4h",
            "fmul v0.4s, v0.4s, v14.4s",
            "fmaxnm v0.4s, v0.4s, v16.4s",
            "fminnm v0.4s, v0.4s, v14.4s",
            "fcvtnu v0.4s, v0.4s",
            "xtn v0.4h, v0.4s",
            "xtn v0.8b, v0.8h",
            "tbl v0.16b, {{v0.16b}}, v15.16b",
            "fmov {word:w}, s0",
            "and {word:w}, {word:w}, {alpha_mask:w}",
            "str {word:w}, [{bytes}]",
            "3:",
            "add {halves}, {halves}, #8",
            "add {bytes}, {bytes}, #4",
            "subs {count}, {count}, #1",
            "b.ne 2b",
            normalizer=in(reg) (1.0f32/255.0).to_bits(),opacity=in(reg) opacity.to_bits(),
            coverage=in(reg) (f32::from(coverage)*(1.0/255.0)).to_bits(),
            one=in(reg) 1.0f32.to_bits(),maximum=in(reg) 255.0f32.to_bits(),
            alpha_mask=in(reg) alpha_mask,shuffle=in(reg) shuffle.as_ptr(),
            source=inout(reg) source.as_ptr()=>_,halves=inout(reg) destination.as_mut_ptr()=>_,
            bytes=inout(reg) bytes.as_mut_ptr()=>_,count=inout(reg) destination.len()=>_,word=out(reg) _,
            out("v0") _,out("v1") _,out("v2") _,out("v10") _,out("v11") _,out("v12") _,
            out("v13") _,out("v14") _,out("v15") _,out("v16") _,options(nostack),
        );
    }
}

// SkRasterPipeline_opts.h HIGHP load_f16 -> scale_1_float -> scale_u8
// -> srcover_rgba_8888 -> to_unorm. Keep every f32 rounding and the fused
// multiply-add, then convert with nearest-even rounding. The N32 destination
// remains in byte units; nested F16 destinations retain the scalar pipeline.
#[cfg(all(target_arch = "aarch64", feature = "simd"))]
fn composite_f16_n32_row(
    source: &[[f16; 4]],
    destination: &mut [u8],
    opacity: f32,
    clip: Option<&[u8]>,
    format: crate::raster::PixelFormat,
) {
    debug_assert_eq!(destination.len(), source.len() * 4);
    debug_assert!(clip.is_none_or(|mask| mask.len() == source.len()));
    if source.is_empty() {
        return;
    }
    let shuffle: [u8; 16] = if format == crate::raster::PixelFormat::Rgba8888 {
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    } else {
        [2, 1, 0, 3, 6, 5, 4, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    };
    let alpha_word_mask = if format == crate::raster::PixelFormat::Bgrx8888 {
        0x00ff_ffff_00ff_ffffu64
    } else {
        u64::MAX
    };
    // Two live pixels use independent four-channel f32 pipelines, retaining
    // every multiply, fused SrcOver and nearest-even conversion. A zero in
    // either pixel keeps the original single-pixel path; two initial zeros
    // may skip an exact eight-pixel signed-zero block without reading target.
    // SAFETY: count checks bound every 16/64-byte source load and eight-byte
    // target store. Clip bytes are read only when present. No padded loads,
    // row crossing, or discarded NaN/nonzero contribution is admitted.
    unsafe {
        core::arch::asm!(
            "ldr q15, [{shuffle}]",
            "dup v10.4s, {opacity:w}",
            "dup v11.4s, {max_value:w}",
            "dup v12.4s, {one:w}",
            "dup v14.4s, {mask_scale:w}",
            "movi v13.4s, #0",
            "2:",
            "cmp {count}, #2",
            "b.lo 7f",
            "ldr q0, [{src}]",
            "fmov {bits}, d0",
            "and {bits}, {bits}, {zero_mask}",
            "dup v3.2d, v0.d[1]",
            "fmov {bits2}, d3",
            "and {bits2}, {bits2}, {zero_mask}",
            "cbnz {bits}, 8f",
            "cbnz {bits2}, 7f",
            // Both first pixels are exact signed zero. Larger zero blocks
            // avoid the old per-pixel load, mask branch and pointer loop.
            "cmp {count}, #8",
            "b.lo 9f",
            "ld1 {{v0.8h, v1.8h, v2.8h, v3.8h}}, [{src}]",
            "orr v0.16b, v0.16b, v1.16b",
            "orr v2.16b, v2.16b, v3.16b",
            "orr v0.16b, v0.16b, v2.16b",
            "shl v0.8h, v0.8h, #1",
            "umaxv h0, v0.8h",
            "umov {bits:w}, v0.h[0]",
            "cbnz {bits:w}, 9f",
            "add {src}, {src}, #64",
            "add {dst}, {dst}, #32",
            "cbz {mask}, 12f",
            "add {mask}, {mask}, #8",
            "12:",
            "subs {count}, {count}, #8",
            "b.ne 2b",
            "b 14f",
            "8:",
            "cbz {bits2}, 7f",
            "fcvtl v0.4s, v0.4h",
            "fcvtl v3.4s, v3.4h",
            "fmul v0.4s, v0.4s, v10.4s",
            "fmul v3.4s, v3.4s, v10.4s",
            "cbz {mask}, 15f",
            "ldrb {bits:w}, [{mask}]",
            "ldrb {bits2:w}, [{mask}, #1]",
            "dup v2.4s, {bits:w}",
            "dup v4.4s, {bits2:w}",
            "ucvtf v2.4s, v2.4s",
            "ucvtf v4.4s, v4.4s",
            "fmul v2.4s, v2.4s, v14.4s",
            "fmul v4.4s, v4.4s, v14.4s",
            "fmul v0.4s, v0.4s, v2.4s",
            "fmul v3.4s, v3.4s, v4.4s",
            "15:",
            "dup v2.4s, v0.s[3]",
            "dup v4.4s, v3.s[3]",
            "fsub v2.4s, v12.4s, v2.4s",
            "fsub v4.4s, v12.4s, v4.4s",
            "fmul v0.4s, v0.4s, v11.4s",
            "fmul v3.4s, v3.4s, v11.4s",
            "ldr d1, [{dst}]",
            "tbl v1.16b, {{v1.16b}}, v15.16b",
            "ushll v1.8h, v1.8b, #0",
            "ushll2 v5.4s, v1.8h, #0",
            "ushll v1.4s, v1.4h, #0",
            "ucvtf v1.4s, v1.4s",
            "ucvtf v5.4s, v5.4s",
            "fmla v0.4s, v1.4s, v2.4s",
            "fmla v3.4s, v5.4s, v4.4s",
            "fmaxnm v0.4s, v0.4s, v13.4s",
            "fmaxnm v3.4s, v3.4s, v13.4s",
            "fminnm v0.4s, v0.4s, v11.4s",
            "fminnm v3.4s, v3.4s, v11.4s",
            "fcvtnu v0.4s, v0.4s",
            "fcvtnu v3.4s, v3.4s",
            "xtn v0.4h, v0.4s",
            "xtn2 v0.8h, v3.4s",
            "xtn v0.8b, v0.8h",
            "tbl v0.16b, {{v0.16b}}, v15.16b",
            "fmov {bits}, d0",
            "and {bits}, {bits}, {alpha_mask}",
            "str {bits}, [{dst}]",
            "9:",
            "add {src}, {src}, #16",
            "add {dst}, {dst}, #8",
            "cbz {mask}, 13f",
            "add {mask}, {mask}, #2",
            "13:",
            "subs {count}, {count}, #2",
            "b.ne 2b",
            "b 14f",
            // Preserve the original zero/live decisions and exact tail math.
            "7:",
            "ldr d0, [{src}], #8",
            "fmov {bits}, d0",
            "and {bits}, {bits}, {zero_mask}",
            "cbz {bits}, 5f",
            "fcvtl v0.4s, v0.4h",
            "fmul v0.4s, v0.4s, v10.4s",
            "cbz {mask}, 3f",
            "ldrb {bits:w}, [{mask}]",
            "dup v2.4s, {bits:w}",
            "ucvtf v2.4s, v2.4s",
            "fmul v2.4s, v2.4s, v14.4s",
            "fmul v0.4s, v0.4s, v2.4s",
            "3:",
            "dup v2.4s, v0.s[3]",
            "fsub v2.4s, v12.4s, v2.4s",
            "fmul v0.4s, v0.4s, v11.4s",
            "ldr s1, [{dst}]",
            "tbl v1.16b, {{v1.16b}}, v15.16b",
            "ushll v1.8h, v1.8b, #0",
            "ushll v1.4s, v1.4h, #0",
            "ucvtf v1.4s, v1.4s",
            "fmla v0.4s, v1.4s, v2.4s",
            "fmaxnm v0.4s, v0.4s, v13.4s",
            "fminnm v0.4s, v0.4s, v11.4s",
            "fcvtnu v0.4s, v0.4s",
            "xtn v0.4h, v0.4s",
            "xtn v0.8b, v0.8h",
            "tbl v0.16b, {{v0.16b}}, v15.16b",
            "fmov {bits:w}, s0",
            "and {bits:w}, {bits:w}, {alpha_mask:w}",
            "str {bits:w}, [{dst}]",
            "5:",
            "add {dst}, {dst}, #4",
            "cbz {mask}, 6f",
            "add {mask}, {mask}, #1",
            "6:",
            "subs {count}, {count}, #1",
            "b.ne 2b",
            "14:",
            src = inout(reg) source.as_ptr() => _,
            dst = inout(reg) destination.as_mut_ptr() => _,
            mask = inout(reg) clip.map_or(core::ptr::null(), |mask| mask.as_ptr()) => _,
            count = inout(reg) source.len() => _,
            shuffle = in(reg) shuffle.as_ptr(),
            opacity = in(reg) opacity.to_bits(),
            max_value = in(reg) 255.0f32.to_bits(),
            one = in(reg) 1.0f32.to_bits(),
            mask_scale = in(reg) (1.0f32 / 255.0).to_bits(),
            zero_mask = in(reg) 0x7fff_7fff_7fff_7fffu64,
            alpha_mask = in(reg) alpha_word_mask,
            bits = out(reg) _, bits2 = out(reg) _,
            out("v0") _, out("v1") _, out("v2") _,
            out("v3") _, out("v4") _, out("v5") _,
            out("v10") _, out("v11") _, out("v12") _,
            out("v13") _, out("v14") _, out("v15") _,
            options(nostack),
        );
    }
}

// Empty F16 raster-device runs carry no SrcOver contribution. Test eight
// complete initialized pixels as raw half patterns; signed zero compares
// equal to positive zero, while every nonzero/NaN pattern remains live.
fn zero_f16_block(pixels: &[[f16; 4]]) -> bool {
    debug_assert!(pixels.len() >= 8);
    #[cfg(all(target_arch = "aarch64", feature = "simd"))]
    unsafe {
        let live: u32;
        core::arch::asm!(
            "ld1 {{v0.8h, v1.8h, v2.8h, v3.8h}}, [{ptr}]",
            "orr v0.16b, v0.16b, v1.16b",
            "orr v2.16b, v2.16b, v3.16b",
            "orr v0.16b, v0.16b, v2.16b",
            "shl v0.8h, v0.8h, #1",
            "umaxv h0, v0.8h",
            "umov {live:w}, v0.h[0]",
            ptr = in(reg) pixels.as_ptr(), live = lateout(reg) live,
            out("v0") _, out("v1") _, out("v2") _, out("v3") _,
            options(nostack, readonly),
        );
        live == 0
    }
    #[cfg(not(all(target_arch = "aarch64", feature = "simd")))]
    {
        pixels[..8].iter().all(|pixel| *pixel == [f16::ZERO; 4])
    }
}

#[cfg(test)]
mod zero_block_tests {
    use super::*;
    #[test]
    fn zero_block_preserves_half_comparison_including_signed_zero_and_nan() {
        let mut pixels = [[f16::NEG_ZERO; 4]; 8];
        assert!(zero_f16_block(&pixels));
        for bits in [0, 0x8000, 1, 0x8001, 0x3c00, 0x7c00, 0xfc00, 0x7e00, 0x7c01] {
            for lane in 0..32 {
                pixels[lane / 4][lane % 4] = f16::from_bits(bits);
                assert_eq!(
                    zero_f16_block(&pixels),
                    pixels.iter().all(|p| *p == [f16::ZERO; 4])
                );
                pixels[lane / 4][lane % 4] = f16::NEG_ZERO;
            }
        }
    }
}

#[cfg(test)]
mod row_pipeline_tests {
    #[test]
    fn n32_f16_span_preserves_all_half_patterns_rounding_transparency_and_encoding() {
        let count = usize::from(u16::MAX) + 1;
        let initial: Vec<_> = (0..=u16::MAX)
            .map(|b| [b, b.rotate_left(3), b.rotate_left(7), b.rotate_left(11)].map(f16::from_bits))
            .collect();
        let mut source = vec![0; count * 4];
        for (i, p) in source.chunks_exact_mut(4).enumerate() {
            if i % 7 != 0 {
                p.copy_from_slice(&[
                    (i * 37) as u8,
                    (i * 53) as u8,
                    (i * 71) as u8,
                    (i * 91) as u8,
                ]);
            }
        }
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for opacity in [0.0, 0.37, 1.0] {
                for coverage in [0, 1, 91, 255] {
                    let mut expected = Surface::from_pixels(initial.clone());
                    let mut actual = Surface::from_pixels(initial.clone());
                    let mut expected_bytes: Vec<_> =
                        (0..count * 4).map(|i| (i * 101) as u8).collect();
                    let mut bytes = vec![173; count * 4 + 6];
                    bytes[3..3 + count * 4].copy_from_slice(&expected_bytes);
                    expected.composite_n32_span_impl(
                        0,
                        &source,
                        &mut expected_bytes,
                        opacity,
                        coverage,
                        format,
                        false,
                    );
                    actual.composite_n32_span(
                        0,
                        &source,
                        &mut bytes[3..3 + count * 4],
                        opacity,
                        coverage,
                        format,
                    );
                    assert_eq!(&bytes[..3], &[173; 3]);
                    assert_eq!(&bytes[3 + count * 4..], &[173; 3]);
                    assert_eq!(
                        &bytes[3..3 + count * 4],
                        expected_bytes,
                        "format={format:?} opacity={opacity} coverage={coverage}"
                    );
                    for (i, (a, b)) in actual.pixels().iter().zip(expected.pixels()).enumerate() {
                        for c in 0..4 {
                            assert_eq!(a[c].to_bits(),b[c].to_bits(),"pixel={i} channel={c} format={format:?} opacity={opacity} coverage={coverage}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn profile_google_n32_f16_nested_restore() {
        let (w, h) = (1438, 342);
        let mut source = vec![0; w * h * 4];
        for (i, p) in source.chunks_exact_mut(4).enumerate() {
            if i % w > 14 && i % w < w - 14 {
                p.copy_from_slice(&[37, 89, 113, 179]);
            }
        }
        for round in 0..5 {
            for fast in [false, true] {
                let mut surface = Surface::new(w * h);
                let mut bytes = vec![0; w * h * 4];
                let start = std::time::Instant::now();
                for y in 0..h {
                    surface.composite_n32_span_impl(
                        y * w,
                        &source[y * w * 4..(y + 1) * w * 4],
                        &mut bytes[y * w * 4..(y + 1) * w * 4],
                        0.73,
                        255,
                        PixelFormat::Bgra8888,
                        fast,
                    );
                }
                let ms = start.elapsed().as_secs_f64() * 1000.0;
                let hash = bytes
                    .iter()
                    .fold(0u64, |h, &v| h.wrapping_mul(31).wrapping_add(u64::from(v)));
                eprintln!("n32-f16-google round={round} fast={fast} ms={ms:.3} hash={hash}");
            }
        }
    }

    #[test]
    #[ignore]
    fn profile_solid_f16_google_span() {
        let count = 700 * 120;
        let color = Color {
            red: 0.17,
            green: 0.59,
            blue: 0.83,
            alpha: 0.73,
        };
        for round in 0..5 {
            for span in [false, true] {
                let mut surface = Surface::new(count);
                let mut bytes = vec![0; count * 4];
                let now = std::time::Instant::now();
                for coverage in [91, 255] {
                    if span {
                        for y in 0..120 {
                            surface.blend_color_span(
                                y * 700,
                                &mut bytes[y * 700 * 4..(y + 1) * 700 * 4],
                                color,
                                coverage,
                                PixelFormat::Bgra8888,
                            );
                        }
                    } else {
                        for (i, p) in bytes.chunks_exact_mut(4).enumerate() {
                            surface.blend(i, color, coverage);
                            p.copy_from_slice(&PixelFormat::Bgra8888.encode(surface.rgba8(i)));
                        }
                    }
                }
                let ms = now.elapsed().as_secs_f64() * 1000.0;
                let hash = bytes
                    .iter()
                    .fold(0u64, |a, &v| a.wrapping_mul(31).wrapping_add(v as u64));
                eprintln!("solid-f16 round={round} span={span} ms={ms:.3} hash={hash}");
            }
        }
    }

    #[test]
    fn solid_f16_span_matches_scalar_all_half_patterns() {
        let original: Vec<_> = (0..=u16::MAX)
            .map(|bits| {
                [
                    f16::from_bits(bits),
                    f16::from_bits(bits.rotate_left(3)),
                    f16::from_bits(bits.rotate_left(7)),
                    f16::from_bits(bits.rotate_left(11)),
                ]
            })
            .collect();
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for coverage in [0, 1, 91, 254, 255] {
                for alpha in [0.0, 0.37, 1.0] {
                    let color = Color {
                        red: 0.13,
                        green: 0.73,
                        blue: 0.91,
                        alpha,
                    };
                    let mut expected = Surface::from_pixels(original.clone());
                    let mut actual = Surface::from_pixels(original.clone());
                    let mut expected_bytes = vec![0; original.len() * 4];
                    let mut actual_bytes = vec![173; original.len() * 4 + 6];
                    for (i, pixel) in expected_bytes.chunks_exact_mut(4).enumerate() {
                        expected.blend(i, color, coverage);
                        pixel.copy_from_slice(&format.encode(expected.rgba8(i)));
                    }
                    actual.blend_color_span(
                        0,
                        &mut actual_bytes[3..3 + original.len() * 4],
                        color,
                        coverage,
                        format,
                    );
                    assert_eq!(&actual_bytes[..3], &[173; 3]);
                    assert_eq!(&actual_bytes[actual_bytes.len() - 3..], &[173; 3]);
                    assert_eq!(
                        &actual_bytes[3..actual_bytes.len() - 3],
                        expected_bytes,
                        "format={format:?} coverage={coverage} alpha={alpha}"
                    );
                    for (i, (a, b)) in actual.pixels().iter().zip(expected.pixels()).enumerate() {
                        for c in 0..4 {
                            assert_eq!(a[c].to_bits(), b[c].to_bits(), "half i={i} c={c} format={format:?} coverage={coverage} alpha={alpha}");
                        }
                    }
                }
            }
        }
    }

    use super::*;
    use crate::raster::PixelFormat;

    #[test]
    fn f16_n32_row_preserves_scalar_rounding_formats_clips_and_origins() {
        for width in [1usize, 2, 7, 17, 257] {
            let height = 9;
            let mut surface = Surface::new(width * height);
            for (i, pixel) in surface.pixels_mut().iter_mut().enumerate() {
                let a = (i * 73 % 256) as f32 / 255.0;
                *pixel = [a * 0.125, a * 0.75, a * 0.37, a].map(f16::from_f32);
                if i % 11 == 0 {
                    *pixel = [f16::NEG_ZERO; 4];
                }
            }
            for format in [
                PixelFormat::Rgba8888,
                PixelFormat::Bgra8888,
                PixelFormat::Bgrx8888,
            ] {
                for opacity in [0.0, 0.001, 0.5, 0.73, 1.0] {
                    for origin in [(0, 0), (-2, 3), (4, -3), (-300, 0), (0, 20)] {
                        let parent_width = width + 5;
                        let initial: Vec<u8> = (0..parent_width * (height + 3) * 4)
                            .map(|i| (i * 31 + 73) as u8)
                            .collect();
                        let mask: Vec<u8> = (0..initial.len() / 4)
                            .map(|i| [0, 1, 91, 127, 183, 254, 255][i % 7])
                            .collect();
                        for clip in [None, Some(mask.as_slice())] {
                            let mut expected = initial.clone();
                            let mut actual = initial.clone();
                            surface.composite_into_impl(
                                &mut expected,
                                None,
                                opacity,
                                clip,
                                format,
                                width,
                                parent_width,
                                origin,
                                false,
                            );
                            surface.composite_into(
                                &mut actual,
                                None,
                                opacity,
                                clip,
                                format,
                                width,
                                parent_width,
                                origin,
                            );
                            assert_eq!(
                                actual,
                                expected,
                                "width={width} {format:?} opacity={opacity} origin={origin:?} clip={}",
                                clip.is_some()
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn f16_n32_row_handles_every_half_bit_pattern() {
        for alpha_patterns in [false, true] {
            let surface = Surface::from_pixels(
                (0..=u16::MAX)
                    .map(|bits| {
                        [
                            f16::from_bits(bits),
                            f16::from_bits(bits.rotate_left(3)),
                            f16::from_bits(bits.rotate_left(7)),
                            if alpha_patterns {
                                f16::from_bits(bits)
                            } else {
                                f16::from_f32(0.375)
                            },
                        ]
                    })
                    .collect(),
            );
            for format in [
                PixelFormat::Rgba8888,
                PixelFormat::Bgra8888,
                PixelFormat::Bgrx8888,
            ] {
                let mut expected: Vec<u8> = (0..surface.pixels().len() * 4)
                    .map(|i| (i * 37) as u8)
                    .collect();
                let mut actual = expected.clone();
                surface.composite_into_impl(
                    &mut expected,
                    None,
                    0.73,
                    None,
                    format,
                    surface.pixels().len(),
                    surface.pixels().len(),
                    (0, 0),
                    false,
                );
                surface.composite_into(
                    &mut actual,
                    None,
                    0.73,
                    None,
                    format,
                    surface.pixels().len(),
                    surface.pixels().len(),
                    (0, 0),
                );
                assert_eq!(actual, expected, "{format:?}");
            }
        }
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64", target_os = "macos"))]
    #[test]
    fn f16_rows_do_not_cross_source_destination_or_clip_guard_pages() {
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
        let mut mask = GuardedBuffer::new();
        for count in 1..=33 {
            let input_bytes = source.tail(count * 8);
            // mmap and the eight-byte row length guarantee f16 alignment.
            let input: &mut [[f16; 4]] =
                unsafe { core::slice::from_raw_parts_mut(input_bytes.as_mut_ptr().cast(), count) };
            for (i, p) in input.iter_mut().enumerate() {
                *p = if i % 5 == 0 {
                    [f16::NEG_ZERO; 4]
                } else {
                    [0.125, 0.375, 0.25, 0.5].map(f16::from_f32)
                };
            }
            let clip = mask.tail(count);
            for (i, a) in clip.iter_mut().enumerate() {
                *a = (i * 37) as u8;
            }
            for format in [
                PixelFormat::Rgba8888,
                PixelFormat::Bgra8888,
                PixelFormat::Bgrx8888,
            ] {
                for coverage in [None, Some(&*clip)] {
                    let output = destination.tail(count * 4);
                    output.fill(181);
                    let scalar = Surface::from_pixels(input.to_vec());
                    let mut expected = output.to_vec();
                    scalar.composite_into_impl(
                        &mut expected,
                        None,
                        0.73,
                        coverage,
                        format,
                        count,
                        count,
                        (0, 0),
                        false,
                    );
                    composite_f16_n32_row(input, output, 0.73, coverage, format);
                    assert_eq!(output, expected, "count={count} {format:?}");
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn profile_f16_n32_restore_round3() {
        let (width, height) = (730usize, 410usize);
        let surface = Surface::from_pixels(
            (0..width * height)
                .map(|i| {
                    let a = if i % 17 == 0 {
                        0.0
                    } else {
                        (i * 73 % 256) as f32 / 255.0
                    };
                    [a * 0.125, a * 0.75, a * 0.37, a].map(f16::from_f32)
                })
                .collect(),
        );
        let initial = vec![181u8; width * height * 4];
        let mask: Vec<u8> = (0..width * height)
            .map(|i| if i % width < 3 { 91 } else { 255 })
            .collect();
        for clip in [None, Some(mask.as_slice())] {
            for use_rows in [false, true] {
                for round in 0..5 {
                    let mut bytes = initial.clone();
                    let now = std::time::Instant::now();
                    surface.composite_into_impl(
                        &mut bytes,
                        None,
                        0.73,
                        clip,
                        PixelFormat::Bgrx8888,
                        width,
                        width,
                        (0, 0),
                        use_rows,
                    );
                    let elapsed = now.elapsed().as_secs_f64() * 1000.0;
                    let hash = bytes
                        .iter()
                        .fold(0u64, |a, &v| a.wrapping_mul(31).wrapping_add(v as u64));
                    eprintln!(
                        "f16-round3 clip={} row={use_rows} round={round} ms={elapsed:.3} hash={hash}",
                        clip.is_some()
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod bounded_clip_restore_round6 {
    use super::*;
    use crate::raster::{IntRect, Mask, PixelFormat};
    #[test]
    fn packed_restore_keeps_bounds_and_matches_dense_for_formats_parents_and_origins() {
        let (width, height) = (37usize, 19usize);
        let bounds = IntRect::from_xywh(5, 4, 21, 9).unwrap();
        let mut mask = Mask::new_bounded(width as u32, height as u32, bounds).unwrap();
        for y in 4..13 {
            for (x, a) in mask.row_range_mut(y, 5, 26).iter_mut().enumerate() {
                *a = match x % 5 {
                    0 => 0,
                    1 => 91,
                    2 => 183,
                    _ => 255,
                };
            }
        }
        let allocated = mask.allocated_bytes();
        let mut dense = Mask::new(width as u32, height as u32).unwrap();
        for y in 4..13 {
            dense
                .row_range_mut(y, 5, 26)
                .copy_from_slice(mask.row_range(y, 5, 26));
        }
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for origin in [(0, 0), (-2, 3), (4, -3), (-300, 0), (0, 30)] {
                for nested in [false, true] {
                    let mut child = Surface::new(23 * 11);
                    for (i, p) in child.pixels_mut().iter_mut().enumerate() {
                        let alpha = (i * 37 % 256) as f32 / 255.0;
                        *p = [alpha * 0.19, alpha * 0.57, alpha * 0.83, alpha].map(f16::from_f32);
                    }
                    let initial: Vec<u8> = (0..width * height * 4)
                        .map(|i| (i * 31 + 73) as u8)
                        .collect();
                    let mut expected = initial.clone();
                    let mut actual = initial;
                    let mut parent_a = Surface::new(width * height);
                    let mut parent_b = Surface::new(width * height);
                    child.composite_into(
                        &mut expected,
                        nested.then_some(&mut parent_a),
                        0.73,
                        Some(dense.data()),
                        format,
                        23,
                        width,
                        origin,
                    );
                    child.composite_into_mask(
                        &mut actual,
                        nested.then_some(&mut parent_b),
                        0.73,
                        Some(&mask),
                        format,
                        23,
                        width,
                        origin,
                    );
                    assert_eq!(
                        actual, expected,
                        "format={format:?} origin={origin:?} nested={nested}"
                    );
                    for (a, b) in parent_a.pixels().iter().zip(parent_b.pixels()) {
                        assert_eq!(a.map(f16::to_bits), b.map(f16::to_bits));
                    }
                    assert_eq!(
                        mask.allocated_bytes(),
                        allocated,
                        "restore must not materialize a device mask"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod written_rows_tests {
    use super::*;
    use crate::raster::{IntRect, Mask, PixelFormat};

    #[test]
    fn written_rows_match_full_scan_for_sparse_nested_and_raw_writes() {
        const WIDTH: usize = 13;
        const HEIGHT: usize = 9;
        const PARENT_WIDTH: usize = 17;
        const PARENT_HEIGHT: usize = 11;
        let format = PixelFormat::Bgrx8888;
        let mut mask = Mask::new_bounded(
            PARENT_WIDTH as u32,
            PARENT_HEIGHT as u32,
            IntRect::from_xywh(3, 2, 11, 7).unwrap(),
        )
        .unwrap();
        for y in 2..9 {
            for (x, value) in mask.row_range_mut(y, 3, 14).iter_mut().enumerate() {
                *value = [0, 91, 255][x % 3];
            }
        }
        let check = |child: &Surface| {
            let full = Surface::from_pixels(child.pixels().to_vec());
            let initial: Vec<u8> = (0..PARENT_WIDTH * PARENT_HEIGHT * 4)
                .map(|i| (i * 37 + 73) as u8)
                .collect();
            let mut actual = initial.clone();
            let mut expected = initial.clone();
            child.composite_into_mask(
                &mut actual,
                None,
                0.73,
                Some(&mask),
                format,
                WIDTH,
                PARENT_WIDTH,
                (2, 1),
            );
            full.composite_into_mask(
                &mut expected,
                None,
                0.73,
                Some(&mask),
                format,
                WIDTH,
                PARENT_WIDTH,
                (2, 1),
            );
            assert_eq!(actual, expected, "N32 restore {:?}", child.written);

            let mut parent = Surface::new(PARENT_WIDTH * PARENT_HEIGHT);
            let mut parent_full = Surface::from_pixels(parent.pixels().to_vec());
            actual = initial.clone();
            expected = initial.clone();
            child.composite_into_mask(
                &mut actual,
                Some(&mut parent),
                0.73,
                Some(&mask),
                format,
                WIDTH,
                PARENT_WIDTH,
                (2, 1),
            );
            full.composite_into_mask(
                &mut expected,
                Some(&mut parent_full),
                0.73,
                Some(&mask),
                format,
                WIDTH,
                PARENT_WIDTH,
                (2, 1),
            );
            assert_eq!(actual, expected, "nested N32 view");
            assert_eq!(
                parent
                    .pixels()
                    .iter()
                    .map(|p| p.map(f16::to_bits))
                    .collect::<Vec<_>>(),
                parent_full
                    .pixels()
                    .iter()
                    .map(|p| p.map(f16::to_bits))
                    .collect::<Vec<_>>(),
                "nested F16 storage"
            );
            if child.written == Written::Empty {
                assert_eq!(parent.written, Written::Empty);
            } else {
                assert!(matches!(parent.written, Written::Range { .. }));
            }
            actual = initial.clone();
            expected = initial;
            parent.composite_into(
                &mut actual,
                None,
                0.91,
                None,
                format,
                PARENT_WIDTH,
                PARENT_WIDTH,
                (0, 0),
            );
            parent_full.composite_into(
                &mut expected,
                None,
                0.91,
                None,
                format,
                PARENT_WIDTH,
                PARENT_WIDTH,
                (0, 0),
            );
            assert_eq!(actual, expected, "tracked nested parent restore");
        };

        let mut surface = Surface::new(WIDTH * HEIGHT);
        assert_eq!(surface.written, Written::Empty);
        check(&surface);
        surface.blend_premultiplied(2 * WIDTH + 3, [0.11, 0.23, 0.37, 0.51], 183);
        assert_eq!(
            surface.written,
            Written::Range {
                start: 2 * WIDTH + 3,
                end: 2 * WIDTH + 4
            }
        );
        check(&surface);
        let color = Color {
            red: 0.19,
            green: 0.57,
            blue: 0.83,
            alpha: 0.73,
        };
        surface.blend_color_span(4 * WIDTH + 11, &mut [0; 16], color, 91, format);
        assert_eq!(
            surface.written,
            Written::Range {
                start: 2 * WIDTH + 3,
                end: 5 * WIDTH + 2
            }
        );
        check(&surface);
        surface.composite_n32_span(
            6 * WIDTH + 1,
            &[31, 63, 91, 127, 0, 0, 0, 0],
            &mut [0; 8],
            0.73,
            255,
            format,
        );
        assert_eq!(
            surface.written,
            Written::Range {
                start: 2 * WIDTH + 3,
                end: 6 * WIDTH + 3
            }
        );
        check(&surface);
        surface.pixels_mut()[7 * WIDTH + 4] = [
            f16::from_bits(0x7e01),
            f16::NEG_ZERO,
            f16::from_f32(0.3),
            f16::from_f32(0.7),
        ];
        assert_eq!(surface.written, Written::Full);
        check(&surface);
        assert_eq!(
            Surface::from_pixels(vec![[f16::NEG_ZERO; 4]; WIDTH * HEIGHT]).written,
            Written::Full
        );
    }
}
