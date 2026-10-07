// CPU implementation moved from renderer/analytic_aa.rs.

// Selected SkScan AAA stages. Fixed-point edge and trapezoid arithmetic
// follows Chromium Skia SkAnalyticEdge.cpp / SkScan_AAAPath.cpp.
// Copyright 2006, 2016 The Android Open Source Project (BSD-3-Clause).
// License: glyphs/SKIA_LICENSE. Native Skia is used only by differential tests.

use crate::src::core::SkTSort as edge_sort;

use crate::src::core::SkAnalyticEdge::{Pt, SkAnalyticEdge};
pub(crate) use crate::src::core::SkEdgeBuilder::{add_clipped_line, add_line};
pub(crate) use crate::src::core::SkEdgeClipper::ClipBox;

const ONE: i32 = 65536;
fn mul(a: i32, b: i32) -> i32 {
    (i64::from(a) * i64::from(b) >> 16) as i32
}
fn ceil(v: i32) -> i32 {
    (v + ONE - 1) >> 16
}

// SkEdgeClipper::clipMonoQuad and SkGeometry::SkFindUnitQuadRoots.
// The caller supplies monotonic rounded-rectangle quadratic segments.

// SkChopQuadAt{Y,X}Extrema also repairs a control point when an
// extremum ratio underflows or rounds to an endpoint. Conic subdivision
// can produce such nearly monotonic curves at fractional coordinates.

// SkConic::computeQuadPOW2 / chop / subdivide: use the same rational
// conic and 0.25px tolerance as SkAnalyticEdgeBuilder.

fn partial_triangle_to_alpha(a: i32, b: i32) -> u8 {
    (((i64::from(a >> 11) * (a >> 11) as i64 * (b >> 11) as i64) >> 8) & 255) as u8
}
fn trapezoid_to_alpha(a: i32, b: i32) -> u8 {
    (((a + b) / 2) >> 8) as u8
}
fn get_partial_alpha(a: u8, full: u8) -> u8 {
    ((u32::from(a) * u32::from(full)) >> 8) as u8
}
fn fixed_to_alpha(v: i32) -> u8 {
    ((255 * v + 32768) >> 16) as u8
}
fn compute_alpha_above_line(l: i32, r: i32, dy: i32, full: u8) -> Vec<u8> {
    let n = ceil(r) as usize;
    let mut a = vec![0; n];
    if n == 1 {
        a[0] = get_partial_alpha(((2 * ONE - l - r) >> 9) as u8, full);
    } else if n > 1 {
        let first = ONE - l;
        let last = r - ((n as i32 - 1) << 16);
        let h = mul(first, dy);
        a[0] = (mul(first, h) >> 9) as u8;
        let mut v = h.saturating_add(dy >> 1);
        for item in a.iter_mut().take(n - 1).skip(1) {
            *item = (v >> 8) as u8;
            v = v.saturating_add(dy);
        }
        a[n - 1] = full.wrapping_sub(partial_triangle_to_alpha(last, dy));
    }
    a
}
fn compute_alpha_below_line(l: i32, r: i32, dy: i32, full: u8) -> Vec<u8> {
    let n = ceil(r) as usize;
    let mut a = vec![0; n];
    if n == 1 {
        a[0] = get_partial_alpha(trapezoid_to_alpha(l, r), full);
    } else if n > 1 {
        let first = ONE - l;
        let last = r - ((n as i32 - 1) << 16);
        let h = mul(last, dy);
        a[n - 1] = (mul(last, h) >> 9) as u8;
        let mut v = h.saturating_add(dy >> 1);
        for i in (1..n - 1).rev() {
            a[i] = ((v >> 8) & 255) as u8;
            v = v.saturating_add(dy);
        }
        a[0] = full.wrapping_sub(partial_triangle_to_alpha(first, dy));
    }
    a
}
// Selected real-blitter rectangle output from aaa_walk_convex_edges. This
// event is a local representation adapter; it does not translate SkAlphaRuns
// or the full RunBasedAdditiveBlitter implementation.
#[derive(Debug)]
pub(crate) struct ConvexRect {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) left_alpha: u8,
    pub(crate) right_alpha: u8,
}

// blit_full_alpha sends an opaque full-height interior to the real blitter's
// blitH. This event preserves that selected upstream output branch; the
// ordering index adapts it to the local event stream, not SkAlphaRuns storage.
#[derive(Debug)]
pub(crate) struct DirectSpan {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) alpha: u8,
    pub(crate) pair: bool,
    // Emit this span before blits[blit_index]. Multiple spans at the same
    // position retain their order in spans; the index counts residual blits.
    pub(crate) blit_index: usize,
}

pub(crate) struct AdditiveBlitter<const ALPHA_ONLY: bool = false, const RECORD_RUNS: bool = true> {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Vec<u8>,
    pub(crate) small: bool,
    pub(crate) direct: Vec<bool>,
    pub(crate) emitting_direct: bool,
    pub(crate) pairs: Vec<bool>,
    pub(crate) emitting_pair: bool,
    pub(crate) suppress_real: bool,
    pub(crate) record_direct: bool,
    pub(crate) blits: Vec<(usize, u8, bool)>,
    pub(crate) run_starts: Vec<usize>,
    pub(crate) rects: Vec<ConvexRect>,
    pub(crate) spans: Vec<DirectSpan>,
    record_spans: bool,
    pub(crate) color_clip_stream: Option<crate::src::core::SkColorGlyphClip::MetadataDispatch>,
    #[cfg(any(test, feature = "profiling"))]
    partial_alpha_per_pixel: bool,
}

// SkScan_AAAPath.cpp::safely_add_alpha / blit_full_alpha(maskRow): add
// fullAlpha to a pre-clipped row span and saturate each byte at 255. NEON's
// unsigned saturating addition implements the same arithmetic; tails retain
// the scalar operation. The caller has already checked y and row bounds.
fn safely_add_alpha_span(pixels: &mut [u8], alpha: u8) {
    if alpha == 0 {
        return;
    }
    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    let offset = unsafe { safely_add_alpha_span_neon(pixels, alpha) };
    #[cfg(not(all(feature = "simd", target_arch = "aarch64")))]
    let offset = 0;
    for pixel in &mut pixels[offset..] {
        *pixel = pixel.saturating_add(alpha);
    }
}

#[cfg(all(feature = "simd", target_arch = "aarch64"))]
unsafe fn safely_add_alpha_span_neon(pixels: &mut [u8], alpha: u8) -> usize {
    use core::arch::aarch64::*;
    let delta = vdupq_n_u8(alpha);
    let mut offset = 0;
    while offset + 16 <= pixels.len() {
        // SAFETY: the complete 16-byte lane is inside the exclusive slice;
        // unaligned loads/stores are supported by the AArch64 instructions.
        let ptr = pixels.as_mut_ptr().add(offset);
        vst1q_u8(ptr, vqaddq_u8(vld1q_u8(ptr), delta));
        offset += 16;
    }
    offset
}

#[cfg(any(test, feature = "profiling"))]
fn partial_alpha_legacy_driver() -> bool {
    #[cfg(feature = "profiling")]
    {
        std::env::var_os("SKIA_PARTIAL_ALPHA_PIXEL").is_some()
    }
    #[cfg(not(feature = "profiling"))]
    {
        false
    }
}
// Local dense provenance storage; SkBlitter span flags are constant. Rust
// bool has valid byte patterns 0/1, so a byte span clear/set preserves every
// flag while avoiding a per-element generic fill loop in unoptimized builds.
fn fill_bool_span(flags: &mut [bool], value: bool) {
    // SAFETY: the exclusive initialized slice is filled with a valid bool byte.
    unsafe {
        core::ptr::write_bytes(flags.as_mut_ptr(), value as u8, flags.len());
    }
}

impl<const ALPHA_ONLY: bool, const RECORD_RUNS: bool> AdditiveBlitter<ALPHA_ONLY, RECORD_RUNS> {
    fn add(&mut self, x: i32, y: i32, a: u8) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            let i = y as usize * self.width as usize + x as usize;
            if self.record_direct && self.emitting_direct && !self.small {
                self.blits.push((i, a, self.emitting_pair));
                return;
            }
            self.pixels[i] = self.pixels[i].saturating_add(a);
            self.direct[i] |= self.emitting_direct;
            if !ALPHA_ONLY && !self.pairs.is_empty() {
                self.pairs[i] |= self.emitting_pair;
            }
        }
    }
    fn mark_run(&mut self, x: i32, y: i32) {
        // Encoded clip targets capture the actual SkBlitter stream separately.
        // Their local color-output restart list is never consumed.
        if RECORD_RUNS
            && self.color_clip_stream.is_none()
            && x >= 0
            && y >= 0
            && x < self.width as i32
            && y < self.height as i32
        {
            self.run_starts
                .push((y as u32 * self.width + x as u32) as usize);
        }
    }
    fn single(&mut self, x: i32, y: i32, a: u8, full: u8) {
        if self.emitting_direct && RECORD_RUNS {
            self.mark_run(x, y);
        }
        if let Some(stream) = &mut self.color_clip_stream {
            stream.single(x, y, a, full, self.suppress_real);
        }
        if self.small
            && full == 255
            && !self.suppress_real
            && x >= 0
            && y >= 0
            && x < self.width as i32
            && y < self.height as i32
        {
            self.pixels[y as usize * self.width as usize + x as usize] = a;
        } else {
            self.add(
                x,
                y,
                if full == 255 && !self.suppress_real {
                    a
                } else {
                    get_partial_alpha(a, full)
                },
            );
        }
    }
    fn aaa_row(
        &mut self,
        y: i32,
        ul: i32,
        ur: i32,
        ll: i32,
        lr: i32,
        ldy: i32,
        rdy: i32,
        full: u8,
    ) {
        self.emitting_direct = full == 255 && !self.suppress_real;
        let l = ul >> 16;
        let r = ceil(lr);
        let len = r - l;
        if len <= 0 {
            return;
        }
        if len == 1 {
            self.single(l, y, trapezoid_to_alpha(ur - ul, lr - ll), full);
            return;
        }
        let mut alphas = vec![full; len as usize];
        let ul_int = ul >> 16;
        let ll_int = ceil(ll);
        if ul_int + 2 == ll_int {
            let first = ((ul_int + 1) << 16) - ul;
            let second = ll - ul - first;
            alphas[0] =
                alphas[0].saturating_sub(full.wrapping_sub(partial_triangle_to_alpha(first, ldy)));
            alphas[1] = alphas[1].saturating_sub(partial_triangle_to_alpha(second, ldy));
        } else {
            let excluded =
                compute_alpha_below_line(ul - (ul_int << 16), ll - (ul_int << 16), ldy, full);
            for (i, &v) in excluded.iter().enumerate() {
                alphas[(ul_int - l) as usize + i] =
                    alphas[(ul_int - l) as usize + i].saturating_sub(v);
            }
        }
        let ur_int = ur >> 16;
        let lr_int = ceil(lr);
        if ur_int + 2 == lr_int {
            let first = ((ur_int + 1) << 16) - ur;
            let second = lr - ur - first;
            alphas[len as usize - 2] =
                alphas[len as usize - 2].saturating_sub(partial_triangle_to_alpha(first, rdy));
            alphas[len as usize - 1] = alphas[len as usize - 1]
                .saturating_sub(full.wrapping_sub(partial_triangle_to_alpha(second, rdy)));
        } else {
            let excluded =
                compute_alpha_above_line(ur - (ur_int << 16), lr - (ur_int << 16), rdy, full);
            for (i, &v) in excluded.iter().enumerate() {
                alphas[(ur_int - l) as usize + i] =
                    alphas[(ur_int - l) as usize + i].saturating_sub(v);
            }
        }
        if let Some(stream) = &mut self.color_clip_stream {
            stream.array(l, y, &alphas, full, self.suppress_real);
        }
        let mut previous = None;
        for (i, a) in alphas.into_iter().enumerate() {
            if RECORD_RUNS && self.emitting_direct && previous != Some(a) {
                self.mark_run(l + i as i32, y);
            }
            previous = Some(a);
            self.add(l + i as i32, y, a);
        }
    }
    // SkScan_AAAPath.cpp::aaa_walk_convex_edges, vertical-edge rectangle
    // branch: split partial left/right columns from the full interior. The
    // official blitAntiRect sends spans to a real blitter. This adapter retains
    // dense mask output, using a span write for the same full-coverage pixels.
    fn blit_vertical_row(&mut self, y: i32, left: i32, right: i32, height: i32) {
        if right <= left {
            return;
        }
        let full_left = ceil(left);
        let full_right = right >> 16;
        if full_right < full_left {
            // Both edges lie inside one pixel; do not add the edges twice.
            self.add(left >> 16, y, fixed_to_alpha(mul(right - left, height)));
            return;
        }
        let partial_left = (full_left << 16) - left;
        let partial_right = right - (full_right << 16);
        if partial_left > 0 {
            self.add(full_left - 1, y, fixed_to_alpha(mul(partial_left, height)));
        }
        let alpha = fixed_to_alpha(height);
        let start = full_left.max(0).min(self.width as i32) as usize;
        let end = full_right.max(0).min(self.width as i32) as usize;
        if y >= 0 && y < self.height as i32 && start < end {
            if alpha == 255 && !(self.record_direct && self.emitting_direct && !self.small) {
                let row = y as usize * self.width as usize;
                // Adding 255 saturates every existing alpha to 255. The flag
                // planes retain their old values unless the old add() OR set them.
                self.pixels[row + start..row + end].fill(255);
                if self.emitting_direct {
                    fill_bool_span(&mut self.direct[row + start..row + end], true);
                }
                if !ALPHA_ONLY && !self.pairs.is_empty() && self.emitting_pair {
                    fill_bool_span(&mut self.pairs[row + start..row + end], true);
                }
            } else {
                for x in start..end {
                    self.add(x as i32, y, alpha);
                }
            }
        }
        if partial_right > 0 {
            self.add(full_right, y, fixed_to_alpha(mul(partial_right, height)));
        }
    }

    fn trapezoid(
        &mut self,
        y: i32,
        mut ul: i32,
        mut ur: i32,
        mut ll: i32,
        mut lr: i32,
        ldy: i32,
        rdy: i32,
        full: u8,
    ) {
        self.emitting_direct = full == 255 && !self.suppress_real;
        if ul > ur {
            return;
        }
        if ll > lr {
            let l1 = ul.min(ll);
            let r1 = ul.max(ll);
            let l2 = ur.min(lr);
            let r2 = ur.max(lr);
            ll = (l1.max(l2) + r1.min(r2)) / 2;
            lr = ll;
        }
        if ul == ur && ll == lr {
            return;
        }
        if ul > ll {
            std::mem::swap(&mut ul, &mut ll);
        }
        if ur > lr {
            std::mem::swap(&mut ur, &mut lr);
        }
        let join_l = ceil(ll) << 16;
        let join_r = (ur >> 16) << 16;
        if join_l > join_r {
            self.aaa_row(y, ul, ur, ll, lr, ldy, rdy, full);
            return;
        }
        if ul < join_l {
            match ceil(join_l - ul) {
                1 => self.single(
                    ul >> 16,
                    y,
                    trapezoid_to_alpha(join_l - ul, join_l - ll),
                    full,
                ),
                2 => {
                    self.emitting_pair =
                        !ALPHA_ONLY && !self.small && full == 255 && !self.suppress_real;
                    let first = join_l - ONE - ul;
                    let second = ll - ul - first;
                    if self.emitting_direct && RECORD_RUNS {
                        self.mark_run(ul >> 16, y);
                        self.mark_run((ul >> 16) + 1, y);
                    }
                    if let Some(stream) = &mut self.color_clip_stream {
                        stream.two(
                            ul >> 16,
                            y,
                            partial_triangle_to_alpha(first, ldy),
                            full.wrapping_sub(partial_triangle_to_alpha(second, ldy)),
                            full,
                            self.suppress_real,
                        );
                    }
                    self.add(ul >> 16, y, partial_triangle_to_alpha(first, ldy));
                    self.add(
                        (ul >> 16) + 1,
                        y,
                        full.wrapping_sub(partial_triangle_to_alpha(second, ldy)),
                    );
                    self.emitting_pair = false;
                }
                _ => self.aaa_row(y, ul, join_l, ll, join_l, ldy, i32::MAX, full),
            }
        }
        if let Some(stream) = &mut self.color_clip_stream {
            stream.full(
                join_l >> 16,
                y,
                (join_r - join_l) >> 16,
                full,
                self.suppress_real,
            );
        }
        let start = (join_l >> 16).max(0);
        let end = (join_r >> 16).min(self.width as i32);
        #[cfg(any(test, feature = "profiling"))]
        let partial_span = !self.partial_alpha_per_pixel;
        #[cfg(not(any(test, feature = "profiling")))]
        let partial_span = true;
        if full == 255
            && self.record_spans
            && self.record_direct
            && self.emitting_direct
            && !self.small
        {
            // SkScan_AAAPath.cpp::blit_full_alpha: !maskRow, fullAlpha ==
            // 255, !noRealBlitter -> getRealBlitter()->blitH(x, y, len).
            // Keep the adapter's original 255-pixel run boundary positions,
            // including unclipped alignment and ordering among edge runs.
            for x in (join_l >> 16..join_r >> 16).step_by(255) {
                self.mark_run(x, y);
            }
            if y >= 0 && y < self.height as i32 && start < end {
                self.spans.push(DirectSpan {
                    x: start,
                    y,
                    width: end - start,
                    alpha: full,
                    pair: self.emitting_pair,
                    blit_index: self.blits.len(),
                });
            }
        } else if full == 255
            && !(self.record_direct && self.emitting_direct && !self.small)
            && y >= 0
            && y < self.height as i32
            && start < end
        {
            // SkScan emits an interior blitH span; saturation at 255 makes
            // this equivalent to add() without re-reading every pixel.
            let row = y as usize * self.width as usize;
            self.pixels[row + start as usize..row + end as usize].fill(255);
            if self.emitting_direct {
                fill_bool_span(
                    &mut self.direct[row + start as usize..row + end as usize],
                    true,
                );
            }
            if !ALPHA_ONLY && !self.pairs.is_empty() && self.emitting_pair {
                fill_bool_span(
                    &mut self.pairs[row + start as usize..row + end as usize],
                    true,
                );
            }
            if self.emitting_direct && RECORD_RUNS {
                for x in (join_l >> 16..join_r >> 16).step_by(255) {
                    self.mark_run(x, y);
                }
            }
        } else if full < 255
            && partial_span
            && !self.emitting_direct
            && !self.emitting_pair
            && y >= 0
            && y < self.height as i32
            && start < end
        {
            // Partial-height interior is the maskRow saturation branch.
            // add() ORs false into both flag planes and records no events or
            // runs here; leave those planes and ordered restart indices intact.
            let row = y as usize * self.width as usize;
            safely_add_alpha_span(
                &mut self.pixels[row + start as usize..row + end as usize],
                full,
            );
        } else {
            for x in join_l >> 16..join_r >> 16 {
                if RECORD_RUNS && self.emitting_direct && (x - (join_l >> 16)) % 255 == 0 {
                    self.mark_run(x, y);
                }
                self.add(x, y, full);
            }
        }
        if lr > join_r {
            match ceil(lr - join_r) {
                1 => self.single(
                    join_r >> 16,
                    y,
                    trapezoid_to_alpha(ur - join_r, lr - join_r),
                    full,
                ),
                2 => {
                    self.emitting_pair =
                        !ALPHA_ONLY && !self.small && full == 255 && !self.suppress_real;
                    let first = join_r + ONE - ur;
                    let second = lr - ur - first;
                    if self.emitting_direct && RECORD_RUNS {
                        self.mark_run(join_r >> 16, y);
                        self.mark_run((join_r >> 16) + 1, y);
                    }
                    if let Some(stream) = &mut self.color_clip_stream {
                        stream.two(
                            join_r >> 16,
                            y,
                            full.wrapping_sub(partial_triangle_to_alpha(first, rdy)),
                            partial_triangle_to_alpha(second, rdy),
                            full,
                            self.suppress_real,
                        );
                    }
                    self.add(
                        join_r >> 16,
                        y,
                        full.wrapping_sub(partial_triangle_to_alpha(first, rdy)),
                    );
                    self.add(
                        (join_r >> 16) + 1,
                        y,
                        partial_triangle_to_alpha(second, rdy),
                    );
                    self.emitting_pair = false;
                }
                _ => self.aaa_row(y, join_r, ur, join_r, lr, i32::MAX, rdy, full),
            }
        }
    }
}

pub(crate) fn aaa_walk_convex_edges(
    edges: Vec<SkAnalyticEdge>,
    width: u32,
    height: u32,
    small: bool,
    clip: ClipBox,
) -> AdditiveBlitter {
    aaa_walk_convex_edges_impl::<false, false, false, true>(edges, width, height, small, clip)
}

pub(crate) fn aaa_walk_convex_edges_with_rects(
    edges: Vec<SkAnalyticEdge>,
    width: u32,
    height: u32,
    small: bool,
    clip: ClipBox,
) -> AdditiveBlitter {
    aaa_walk_convex_edges_impl::<true, false, false, true>(edges, width, height, small, clip)
}

pub(crate) fn aaa_walk_convex_edges_with_spans(
    edges: Vec<SkAnalyticEdge>,
    width: u32,
    height: u32,
    small: bool,
    clip: ClipBox,
) -> AdditiveBlitter {
    aaa_walk_convex_edges_impl::<true, true, false, true>(edges, width, height, small, clip)
}

// SkDraw::DrawToMask chooses an A8 real blitter, not the N32 two-pixel
// provenance target. The direct plane is still needed by the local RLE
// snap adapter. A8 clip targets still record the original run boundaries;
// shadow input targets omit that unused run list. Neither needs color pairs.
pub(crate) fn aaa_walk_convex_edges_for_mask<const ALPHA_ONLY: bool, const RECORD_RUNS: bool>(
    edges: Vec<SkAnalyticEdge>,
    width: u32,
    height: u32,
    small: bool,
    clip: ClipBox,
    real_rects: bool,
) -> AdditiveBlitter<ALPHA_ONLY, RECORD_RUNS> {
    if real_rects {
        aaa_walk_convex_edges_impl::<true, false, ALPHA_ONLY, RECORD_RUNS>(
            edges, width, height, small, clip,
        )
    } else {
        aaa_walk_convex_edges_impl::<false, false, ALPHA_ONLY, RECORD_RUNS>(
            edges, width, height, small, clip,
        )
    }
}

fn aaa_walk_convex_edges_impl<
    const REAL_RECTS: bool,
    const DIRECT_SPANS: bool,
    const ALPHA_ONLY: bool,
    const RECORD_RUNS: bool,
>(
    edges: Vec<SkAnalyticEdge>,
    width: u32,
    height: u32,
    small: bool,
    clip: ClipBox,
) -> AdditiveBlitter<ALPHA_ONLY, RECORD_RUNS> {
    aaa_walk_convex_edges_encoded_impl::<REAL_RECTS, DIRECT_SPANS, ALPHA_ONLY, RECORD_RUNS>(
        edges, width, height, small, clip, None,
    )
}
// Clip-only entry point. Stream coordinates are the same shifted coordinates
// as these edges; use snug path/actual clip bounds, not dense margin bounds.
pub(crate) fn aaa_walk_convex_edges_with_encoding(
    edges: Vec<SkAnalyticEdge>,
    width: u32,
    height: u32,
    clip: ClipBox,
    stream: crate::src::core::SkColorGlyphClip::MetadataDispatch,
) -> AdditiveBlitter {
    aaa_walk_convex_edges_encoded_impl::<false, false, false, true>(
        edges,
        width,
        height,
        false,
        clip,
        Some(stream),
    )
}
fn aaa_walk_convex_edges_encoded_impl<
    const REAL_RECTS: bool,
    const DIRECT_SPANS: bool,
    const ALPHA_ONLY: bool,
    const RECORD_RUNS: bool,
>(
    mut edges: Vec<SkAnalyticEdge>,
    width: u32,
    height: u32,
    small: bool,
    clip: ClipBox,
    stream: Option<crate::src::core::SkColorGlyphClip::MetadataDispatch>,
) -> AdditiveBlitter<ALPHA_ONLY, RECORD_RUNS> {
    let mut mask = AdditiveBlitter {
        width,
        height,
        pixels: vec![0; (width * height) as usize],
        small,
        direct: vec![false; (width * height) as usize],
        emitting_direct: false,
        // H2 provenance is color-output metadata only. Encoded clip callers
        // consume alpha bytes and MetadataDispatch, never this second plane.
        pairs: if ALPHA_ONLY || stream.is_some() {
            Vec::new()
        } else {
            vec![false; (width * height) as usize]
        },
        emitting_pair: false,
        suppress_real: false,
        record_direct: DIRECT_SPANS,
        blits: Vec::new(),
        run_starts: Vec::new(),
        rects: Vec::new(),
        spans: Vec::new(),
        record_spans: DIRECT_SPANS,
        color_clip_stream: stream,
        #[cfg(any(test, feature = "profiling"))]
        partial_alpha_per_pixel: partial_alpha_legacy_driver(),
    };
    edge_sort::sort(&mut edges, |a, b| {
        (a.upper_y, a.x, a.dx) < (b.upper_y, b.x, b.dx)
    });
    if edges.len() < 2 {
        return mask;
    }
    let mut left = 0;
    let mut right = 1;
    let mut next = 2;
    let stop = (clip.bottom as i32) << 16;
    let left_bound = (clip.left as i32) << 16;
    let right_bound = (clip.right as i32) << 16;
    let mut y = edges[left]
        .upper_y
        .max(edges[right].upper_y)
        .max((clip.top as i32) << 16);
    loop {
        while edges[left].lower_y <= y {
            if !edges[left].update() {
                if next == edges.len() || edges[next].upper_y >= stop {
                    return mask;
                }
                left = next;
                next += 1;
            }
        }
        while edges[right].lower_y <= y {
            if !edges[right].update() {
                if next == edges.len() || edges[next].upper_y >= stop {
                    return mask;
                }
                right = next;
                next += 1;
            }
        }
        if y >= stop {
            return mask;
        }
        edges[left].go_y(y);
        edges[right].go_y(y);
        if (edges[left].x, edges[left].dx) > (edges[right].x, edges[right].dx) {
            std::mem::swap(&mut left, &mut right);
        }
        let le = &edges[left];
        let re = &edges[right];
        let mut bottom = le.lower_y.min(re.lower_y);
        let smooth = if next < edges.len() && edges[next].upper_y < stop {
            if le.lower_y + ONE < re.lower_y {
                le.smooth(&edges[next])
            } else if le.lower_y > re.lower_y + ONE {
                re.smooth(&edges[next])
            } else if next + 1 < edges.len() && edges[next + 1].upper_y < stop {
                let (a, b) = if edges[next].upper_x <= edges[next + 1].upper_x {
                    (next, next + 1)
                } else {
                    (next + 1, next)
                };
                le.smooth(&edges[a]) && re.smooth(&edges[b])
            } else {
                false
            }
        } else {
            false
        };
        if smooth {
            bottom = ceil(bottom) << 16;
        }
        bottom = bottom.min(stop);
        let mut lx = le.x.max(left_bound);
        let mut rx = re.x.min(right_bound);
        let dl = le.dx;
        let dr = re.dx;
        let ldy = le.dy;
        let rdy = re.dy;
        if dl == 0 && dr == 0 {
            if let Some(stream) = &mut mask.color_clip_stream {
                stream.vertical_band(lx, rx, y, bottom);
            }
            let full_left = ceil(lx);
            let full_right = rx >> 16;
            let partial_left = (full_left << 16) - lx;
            let partial_right = rx - (full_right << 16);
            // Zero-alpha nonzero edge fractions are retained in the dense
            // path so its direct flag plane remains byte-identical as well.
            let representable_edges = (partial_left == 0 || fixed_to_alpha(partial_left) > 0)
                && (partial_right == 0 || fixed_to_alpha(partial_right) > 0);
            if REAL_RECTS && rx > lx && full_right >= full_left && representable_edges {
                // SkScan_AAAPath.cpp::aaa_walk_convex_edges vertical branch:
                // partialTop / real blitAntiRect / partialBot. Partial rows
                // retain the same fixed-point products as the dense walker.
                let full_top = ceil(y);
                let full_bot = bottom >> 16;
                let mut partial_top = (full_top << 16) - y;
                let mut partial_bot = bottom - (full_bot << 16);
                if full_top > full_bot {
                    partial_top -= ONE - partial_bot;
                    partial_bot = 0;
                }
                mask.emitting_direct = false;
                if partial_top > 0 {
                    mask.blit_vertical_row(full_top - 1, lx, rx, partial_top);
                }
                if full_bot > full_top
                    && (full_right > full_left
                        || fixed_to_alpha(partial_left) > 0
                        || fixed_to_alpha(partial_right) > 0)
                {
                    mask.rects.push(ConvexRect {
                        x: full_left - 1,
                        y: full_top,
                        width: full_right - full_left,
                        height: full_bot - full_top,
                        left_alpha: fixed_to_alpha(partial_left),
                        right_alpha: fixed_to_alpha(partial_right),
                    });
                }
                if partial_bot > 0 {
                    mask.blit_vertical_row(full_bot, lx, rx, partial_bot);
                }
                y = bottom;
            } else {
                while y < bottom {
                    let ny = bottom.min((ceil(y + 1)) << 16);
                    let h = ny - y;
                    if rx > lx {
                        mask.emitting_direct = h == ONE;
                        mask.blit_vertical_row(y >> 16, lx, rx, h);
                    }
                    y = ny;
                }
            }
        } else {
            lx += 2048;
            rx += 2048;
            while y < bottom {
                let ny = bottom.min((ceil(y + 1)) << 16);
                let h = ny - y;
                let nl = if ny == bottom {
                    (lx + mul(dl, h)).max(left_bound + 2048)
                } else {
                    lx + mul(dl, h)
                };
                let nr = if ny == bottom {
                    (rx + mul(dr, h)).min(right_bound + 2048)
                } else {
                    rx + mul(dr, h)
                };
                let alpha = if h == ONE && ny < bottom {
                    255
                } else {
                    fixed_to_alpha(h)
                };
                mask.trapezoid(
                    y >> 16,
                    lx & !4095,
                    rx & !4095,
                    nl & !4095,
                    nr & !4095,
                    ldy,
                    rdy,
                    alpha,
                );
                if let Some(stream) = &mut mask.color_clip_stream {
                    stream.flush_if_y_changed(y, ny);
                }
                lx = nl;
                rx = nr;
                y = ny;
            }
            lx -= 2048;
            rx -= 2048;
        }
        edges[left].x = lx;
        edges[right].x = rx;
        edges[left].y = y;
        edges[right].y = y;
    }
}

// SkScan_AAAPath::aaa_walk_edges: active edges, winding spans, quarter
// intersections and keepContinuous updates. Array indices retain removed
// edges so a left span can finish while its starting edge is being updated.
pub(crate) fn aaa_walk_edges(
    edges: Vec<SkAnalyticEdge>,
    width: u32,
    height: u32,
    small: bool,
    clip: ClipBox,
    even_odd: bool,
    force_rle: bool,
    skip_intersect: bool,
) -> AdditiveBlitter {
    aaa_walk_edges_impl::<false>(
        edges,
        width,
        height,
        small,
        clip,
        even_odd,
        force_rle,
        skip_intersect,
    )
}

pub(crate) fn aaa_walk_edges_with_spans(
    edges: Vec<SkAnalyticEdge>,
    width: u32,
    height: u32,
    small: bool,
    clip: ClipBox,
    even_odd: bool,
    force_rle: bool,
    skip_intersect: bool,
) -> AdditiveBlitter {
    aaa_walk_edges_impl::<true>(
        edges,
        width,
        height,
        small,
        clip,
        even_odd,
        force_rle,
        skip_intersect,
    )
}

fn aaa_walk_edges_impl<const DIRECT_SPANS: bool>(
    edges: Vec<SkAnalyticEdge>,
    width: u32,
    height: u32,
    small: bool,
    clip: ClipBox,
    even_odd: bool,
    force_rle: bool,
    skip_intersect: bool,
) -> AdditiveBlitter {
    aaa_walk_edges_encoded_impl::<DIRECT_SPANS>(
        edges,
        width,
        height,
        small,
        clip,
        even_odd,
        force_rle,
        skip_intersect,
        None,
    )
}
pub(crate) fn aaa_walk_edges_with_encoding(
    edges: Vec<SkAnalyticEdge>,
    width: u32,
    height: u32,
    clip: ClipBox,
    even_odd: bool,
    skip_intersect: bool,
    stream: crate::src::core::SkColorGlyphClip::MetadataDispatch,
) -> AdditiveBlitter {
    aaa_walk_edges_encoded_impl::<false>(
        edges,
        width,
        height,
        false,
        clip,
        even_odd,
        true,
        skip_intersect,
        Some(stream),
    )
}
fn aaa_walk_edges_encoded_impl<const DIRECT_SPANS: bool>(
    mut edges: Vec<SkAnalyticEdge>,
    width: u32,
    height: u32,
    small: bool,
    clip: ClipBox,
    even_odd: bool,
    force_rle: bool,
    skip_intersect: bool,
    stream: Option<crate::src::core::SkColorGlyphClip::MetadataDispatch>,
) -> AdditiveBlitter {
    let mut out = AdditiveBlitter {
        width,
        height,
        pixels: vec![0; (width * height) as usize],
        small,
        direct: vec![false; (width * height) as usize],
        emitting_direct: false,
        pairs: if stream.is_some() {
            Vec::new()
        } else {
            vec![false; (width * height) as usize]
        },
        emitting_pair: false,
        suppress_real: false,
        record_direct: true,
        blits: Vec::new(),
        run_starts: Vec::new(),
        rects: Vec::new(),
        spans: Vec::new(),
        record_spans: DIRECT_SPANS,
        color_clip_stream: stream,
        #[cfg(any(test, feature = "profiling"))]
        partial_alpha_per_pixel: partial_alpha_legacy_driver(),
    };
    edge_sort::sort(&mut edges, |a, b| {
        (a.upper_y, a.x, a.dx) < (b.upper_y, b.x, b.dx)
    });
    if edges.is_empty() {
        return out;
    }
    // SkScan_AAAPath.cpp::aaa_walk_edges keeps the active and pending
    // edges in one linked list. Updating a curve can ripple its node backward
    // while its neighbors are still at the previous scan line.
    let count = edges.len();
    let head = count;
    let tail = count + 1;
    let sentinel = |x, y| SkAnalyticEdge {
        x,
        dx: 0,
        upper_x: x,
        y,
        upper_y: y,
        lower_y: y,
        dy: i32::MAX,
        quad: None,
        cubic: None,
        winding: 0,
    };
    let lc = (clip.left as i32) << 16;
    let rc = (clip.right as i32) << 16;
    edges.push(sentinel(lc, i32::MIN));
    edges.push(sentinel(rc, i32::MAX));
    let mut next = vec![tail; count + 2];
    let mut prev = vec![head; count + 2];
    next[head] = 0;
    prev[tail] = count - 1;
    for i in 0..count {
        next[i] = if i + 1 == count { tail } else { i + 1 };
        prev[i] = if i == 0 { head } else { i - 1 };
    }
    let remove = |i: usize, next: &mut Vec<usize>, prev: &mut Vec<usize>| {
        next[prev[i]] = next[i];
        prev[next[i]] = prev[i];
    };
    let insert = |i: usize, after: usize, next: &mut Vec<usize>, prev: &mut Vec<usize>| {
        let following = next[after];
        next[after] = i;
        prev[i] = after;
        next[i] = following;
        prev[following] = i;
    };
    let update_next = |v: i32, y: i32, n: &mut i32| {
        if v > y && v < *n {
            *n = v;
        }
    };
    let mut y = edges[0].upper_y.max((clip.top as i32) << 16);
    let stop = (clip.bottom as i32) << 16;
    let mut next_next = i32::MAX;
    let mut current = next[head];
    while edges[current].upper_y <= y {
        edges[current].go_y(y);
        update_next(edges[current].lower_y, y, &mut next_next);
        current = next[current];
    }
    update_next(edges[current].upper_y, y, &mut next_next);
    while y < stop {
        let mut ny = next_next.min(ceil(y + 1) << 16);
        next_next = i32::MAX;
        let shift = if (ny - y) & 16384 != 0 {
            ny = y + 16384;
            2
        } else if (ny - y) & 32768 != 0 {
            1
        } else {
            0
        };
        if ny <= y {
            return out;
        }
        let alpha = fixed_to_alpha(ny - y);
        let mut winding = 0;
        let mut inside = false;
        let mut left = lc;
        let mut left_dy = 0;
        let mut left_edge = head;
        let mut previous_x = edges[head].x;
        let mut previous_right = lc >> 16;
        current = next[head];
        while edges[current].upper_y <= y {
            let was_inside = inside;
            winding += i32::from(edges[current].winding);
            inside = if even_odd {
                winding & 1 != 0
            } else {
                winding != 0
            };
            let is_left = inside && !was_inside;
            let is_right = !inside && was_inside;
            let old_x = edges[current].x;
            if is_left {
                left = old_x.max(lc);
                left_dy = edges[current].dy;
                left_edge = current;
            }
            edges[current].x += edges[current].dx >> shift;
            edges[current].y = ny;
            if is_right {
                let right = old_x.min(rc);
                let next_left = edges[left_edge].x.max(lc);
                let next_right = edges[current].x.min(rc);
                let neighbor = next[current];
                let close = edges[neighbor].upper_y < ny
                    && edges[current].x.saturating_add(ONE)
                        >= edges[neighbor].x.saturating_sub(edges[neighbor].dx.abs());
                out.suppress_real = force_rle
                    || (alpha == 255
                        && (previous_right > left >> 16
                            || previous_right > edges[left_edge].x >> 16
                            || close));
                out.trapezoid(
                    y >> 16,
                    left,
                    right,
                    next_left,
                    next_right,
                    left_dy,
                    edges[current].dy,
                    alpha,
                );
                previous_right = ceil(right.max(edges[current].x));
            }
            let following = next[current];
            while edges[current].lower_y <= ny {
                let e = &mut edges[current];
                if let Some(q) = e.quad.as_mut().filter(|q| q.count > 0) {
                    q.snapped_x = e.x;
                    q.snapped_y = e.y;
                }
                if let Some(c) = e.cubic.as_mut().filter(|c| c.count < 0) {
                    c.keep_continuous(e.x, e.y);
                }
                if !e.update() {
                    break;
                }
            }
            if edges[current].lower_y <= ny {
                remove(current, &mut next, &mut prev);
            } else {
                update_next(edges[current].lower_y, ny, &mut next_next);
                let x = edges[current].x;
                if x < previous_x {
                    let mut before = prev[current];
                    while before != head && edges[before].x > x {
                        before = prev[before];
                    }
                    remove(current, &mut next, &mut prev);
                    insert(current, before, &mut next, &mut prev);
                } else {
                    previous_x = x;
                }
                let before = prev[current];
                if !skip_intersect
                    && before != head
                    && edges[before].x.saturating_add(edges[before].dx)
                        > edges[current].x.saturating_add(edges[current].dx)
                {
                    next_next = ny + 16384;
                }
            }
            current = following;
        }
        if inside {
            let before = prev[left_edge];
            let close = edges[left_edge].upper_y < ny
                && edges[before].x.saturating_add(ONE)
                    >= edges[left_edge].x.saturating_sub(edges[left_edge].dx.abs());
            out.suppress_real = force_rle || (alpha == 255 && close);
            out.trapezoid(
                y >> 16,
                left,
                rc,
                edges[left_edge].x.max(lc),
                rc,
                left_dy,
                0,
                alpha,
            );
        }
        if let Some(stream) = &mut out.color_clip_stream {
            stream.flush_if_y_changed(y, ny);
        }
        y = ny;
        if y >= stop {
            break;
        }
        if edges[current].upper_y > y {
            update_next(edges[current].upper_y, y, &mut next_next);
            continue;
        }
        let mut before = prev[current];
        let reorders = edges[before].x > edges[current].x;
        if reorders {
            while before != head && edges[before].x > edges[current].x {
                before = prev[before];
            }
        }
        while edges[current].upper_y <= y {
            let following = next[current];
            if reorders {
                while next[before] != current && edges[next[before]].x < edges[current].x {
                    before = next[before];
                }
                if next[before] != current {
                    remove(current, &mut next, &mut prev);
                    insert(current, before, &mut next, &mut prev);
                }
            }
            let p = prev[current];
            if p != head
                && edges[p].x.saturating_add(edges[p].dx)
                    > edges[current].x.saturating_add(edges[current].dx)
            {
                next_next = y + 16384;
            }
            let n = next[current];
            if reorders
                && n != tail
                && edges[current].x.saturating_add(edges[current].dx)
                    > edges[n].x.saturating_add(edges[n].dx)
            {
                next_next = y + 16384;
            }
            update_next(edges[current].lower_y, y, &mut next_next);
            before = current;
            current = following;
        }
        update_next(edges[current].upper_y, y, &mut next_next);
    }
    out
}

#[cfg(test)]
mod partial_alpha_span_tests {
    use super::*;

    #[test]
    fn partial_alpha_span_matches_every_byte_delta_and_guarded_tail() {
        for alpha in 0..=255u8 {
            for len in [
                0, 1, 7, 8, 15, 16, 17, 31, 32, 33, 63, 127, 254, 255, 256, 511,
            ] {
                for left in [0, 1, 7, 15] {
                    let mut actual: Vec<u8> =
                        (0..left + len + 19).map(|i| (i % 256) as u8).collect();
                    let mut expected = actual.clone();
                    for p in &mut expected[left..left + len] {
                        *p = (u16::from(*p) + u16::from(alpha)).min(255) as u8;
                    }
                    safely_add_alpha_span(&mut actual[left..left + len], alpha);
                    assert_eq!(actual, expected, "alpha={alpha} len={len} left={left}");
                }
            }
        }
    }

    fn buffer<const A: bool, const R: bool>(
        old: bool,
        small: bool,
        spans: bool,
    ) -> AdditiveBlitter<A, R> {
        let count = 517 * 4;
        AdditiveBlitter {
            width: 517,
            height: 4,
            pixels: (0..count).map(|i| (i % 256) as u8).collect(),
            small,
            direct: (0..count).map(|i| i % 17 == 0).collect(),
            emitting_direct: false,
            pairs: if A {
                Vec::new()
            } else {
                (0..count).map(|i| i % 19 == 0).collect()
            },
            emitting_pair: false,
            suppress_real: false,
            record_direct: spans,
            blits: vec![(5, 7, false)],
            run_starts: vec![5, 17],
            rects: Vec::new(),
            spans: Vec::new(),
            record_spans: spans,
            color_clip_stream: None,
            partial_alpha_per_pixel: old,
        }
    }

    fn check<const A: bool, const R: bool>() {
        for (left, right) in [
            (-2 * ONE, 3 * ONE),
            (-ONE / 4, 518 * ONE),
            (0, 517 * ONE),
            (ONE + ONE / 4, 17 * ONE),
            (254 * ONE + ONE / 4, 256 * ONE),
            (510 * ONE + 3 * ONE / 4, 520 * ONE),
        ] {
            for shift in [-ONE / 2, 0, ONE / 4, 3 * ONE / 2] {
                for y in [-1, 0, 3, 4] {
                    for small in [false, true] {
                        for spans in [false, true] {
                            let mut old = buffer::<A, R>(true, small, spans);
                            let mut new = buffer::<A, R>(false, small, spans);
                            for full in [0, 1, 63, 127, 128, 254, 255] {
                                old.trapezoid(
                                    y,
                                    left,
                                    right,
                                    left + shift,
                                    right + shift,
                                    2 * ONE,
                                    2 * ONE,
                                    full,
                                );
                                new.trapezoid(
                                    y,
                                    left,
                                    right,
                                    left + shift,
                                    right + shift,
                                    2 * ONE,
                                    2 * ONE,
                                    full,
                                );
                                assert_eq!(new.pixels, old.pixels, "A={A} R={R} full={full}");
                                assert_eq!(new.direct, old.direct);
                                assert_eq!(new.pairs, old.pairs);
                                assert_eq!(new.run_starts, old.run_starts);
                                assert_eq!(new.blits, old.blits);
                                let events = |m: &AdditiveBlitter<A, R>| {
                                    m.spans
                                        .iter()
                                        .map(|s| (s.x, s.y, s.width, s.alpha, s.pair, s.blit_index))
                                        .collect::<Vec<_>>()
                                };
                                assert_eq!(events(&new), events(&old));
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn partial_alpha_span_preserves_legacy_clipped_planes_events_and_runs() {
        check::<false, false>();
        check::<false, true>();
        check::<true, false>();
        check::<true, true>();
        // An inherited pair flag must keep the per-pixel fallback, even for
        // partial fullAlpha; no flag plane may be cleared by the row kernel.
        let mut old = buffer::<false, true>(true, false, false);
        let mut new = buffer::<false, true>(false, false, false);
        old.emitting_pair = true;
        new.emitting_pair = true;
        old.trapezoid(0, ONE, 17 * ONE, ONE, 17 * ONE, 2 * ONE, 2 * ONE, 127);
        new.trapezoid(0, ONE, 17 * ONE, ONE, 17 * ONE, 2 * ONE, 2 * ONE, 127);
        assert_eq!(new.pixels, old.pixels);
        assert_eq!(new.pairs, old.pairs);
        assert_eq!(new.direct, old.direct);
        assert_eq!(new.run_starts, old.run_starts);
    }
}

#[cfg(test)]
mod vertical_span_tests {
    use super::*;

    fn buffer(small: bool, record_direct: bool) -> AdditiveBlitter {
        let count = 35 * 4;
        AdditiveBlitter {
            width: 35,
            height: 4,
            pixels: (0..count).map(|i| (i % 251) as u8).collect(),
            small,
            direct: vec![false; count],
            emitting_direct: false,
            pairs: vec![false; count],
            emitting_pair: false,
            suppress_real: false,
            record_direct,
            blits: Vec::new(),
            run_starts: Vec::new(),
            rects: Vec::new(),
            spans: Vec::new(),
            record_spans: false,
            color_clip_stream: None,
            #[cfg(any(test, feature = "profiling"))]
            partial_alpha_per_pixel: partial_alpha_legacy_driver(),
        }
    }

    #[test]
    fn vertical_spans_preserve_previous_per_pixel_scan_output() {
        // Previous scan loop is an independent regression oracle for all mask
        // planes and ordered direct events, including pre-existing coverage.
        let coordinates = [
            -2 * ONE - ONE / 2,
            -ONE,
            -1,
            0,
            1,
            ONE / 4,
            ONE / 2,
            ONE - 1,
            ONE,
            ONE + 1,
            ONE + ONE / 2,
            3 * ONE,
            34 * ONE + ONE / 2,
            35 * ONE,
            36 * ONE,
        ];
        for &left in &coordinates {
            for &right in &coordinates {
                for height in [1, ONE / 4, ONE / 2, ONE - 1, ONE] {
                    for y in [-1, 0, 3, 4] {
                        for (small, record, pair) in [
                            (false, false, false),
                            (true, false, false),
                            (false, true, false),
                            (false, true, true),
                        ] {
                            let mut actual = buffer(small, record);
                            let mut expected = buffer(small, record);
                            actual.emitting_direct = height == ONE;
                            expected.emitting_direct = height == ONE;
                            actual.emitting_pair = pair;
                            expected.emitting_pair = pair;
                            actual.blit_vertical_row(y, left, right, height);
                            if right > left {
                                for x in left >> 16..ceil(right) {
                                    let width = right.min((x + 1) << 16) - left.max(x << 16);
                                    if width > 0 {
                                        expected.add(x, y, fixed_to_alpha(mul(width, height)));
                                    }
                                }
                            }
                            assert_eq!(
                                actual.pixels, expected.pixels,
                                "left={left} right={right} height={height} y={y}"
                            );
                            assert_eq!(actual.direct, expected.direct);
                            assert_eq!(actual.pairs, expected.pairs);
                            assert_eq!(actual.blits, expected.blits);
                            assert_eq!(actual.run_starts, expected.run_starts);
                        }
                    }
                }
            }
        }
    }

    fn compare_rectangle_events(edges: Vec<SkAnalyticEdge>, small: bool, clip: ClipBox) -> usize {
        let expected = aaa_walk_convex_edges(edges.clone(), 37, 29, small, clip);
        let mut actual = aaa_walk_convex_edges_with_rects(edges, 37, 29, small, clip);
        assert!(expected.rects.is_empty());
        let rects = std::mem::take(&mut actual.rects);
        for rect in &rects {
            assert!(rect.height > 0);
            assert!(rect.width >= 0);
            for y in rect.y..rect.y + rect.height {
                if y >= 0 && y < actual.height as i32 {
                    let row = y as usize * actual.width as usize;
                    // Full integer rows are emitted once through the real
                    // blitter; they must not also carry additive mask data.
                    assert!(actual.pixels[row..row + actual.width as usize]
                        .iter()
                        .all(|&a| a == 0));
                    assert!(actual.direct[row..row + actual.width as usize]
                        .iter()
                        .all(|&a| !a));
                    assert!(actual.pairs[row..row + actual.width as usize]
                        .iter()
                        .all(|&a| !a));
                }
            }
        }
        // Decode blitAntiRect's left column, full interior and right column
        // independently from the new top/bottom partitioning. Compare against
        // the previous per-row walker, including its direct/pair planes.
        actual.emitting_direct = true;
        for rect in &rects {
            for y in rect.y..rect.y + rect.height {
                for x in rect.x..=rect.x + rect.width + 1 {
                    let alpha = if x == rect.x {
                        rect.left_alpha
                    } else if x == rect.x + rect.width + 1 {
                        rect.right_alpha
                    } else {
                        255
                    };
                    if alpha != 0 {
                        actual.add(x, y, alpha);
                    }
                }
            }
        }
        assert_eq!(actual.pixels, expected.pixels);
        assert_eq!(actual.direct, expected.direct);
        assert_eq!(actual.pairs, expected.pairs);
        assert_eq!(actual.blits, expected.blits);
        assert_eq!(actual.run_starts, expected.run_starts);
        rects.len()
    }

    fn rectangle_edges(left: f32, right: f32, top: f32, bottom: f32) -> Vec<SkAnalyticEdge> {
        [
            SkAnalyticEdge::setLine((left, top), (left, bottom)),
            SkAnalyticEdge::setLine((right, bottom), (right, top)),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    #[test]
    fn convex_real_rects_preserve_dense_planes_with_fractional_bounds_and_clips() {
        let clips = [
            ClipBox {
                left: 0.0,
                top: 0.0,
                right: 37.0,
                bottom: 29.0,
            },
            ClipBox {
                left: 1.0,
                top: 2.0,
                right: 35.0,
                bottom: 27.0,
            },
            ClipBox {
                left: 10.0,
                top: 8.0,
                right: 20.0,
                bottom: 19.0,
            },
        ];
        let mut events = 0;
        // Include single-pixel widths/heights, zero-width full interiors,
        // edges outside all four clip sides, and tiny quantized edge alpha.
        for left in [-3.25, 0.0, 0.00390625, 0.25, 0.75, 10.5, 35.75, 38.0] {
            for width in [0.25, 0.5, 1.0, 1.25, 2.75, 18.0, 39.0] {
                for top in [-2.25, 0.0, 0.25, 0.75, 8.5, 27.75, 30.0] {
                    for height in [0.25, 0.5, 1.0, 1.25, 3.75, 24.0, 31.0] {
                        let edges = rectangle_edges(left, left + width, top, top + height);
                        for clip in &clips {
                            for small in [false, true] {
                                events += compare_rectangle_events(edges.clone(), small, *clip);
                            }
                        }
                    }
                }
            }
        }
        assert!(events > 1000);
        // Exercise unquantized fixed-point clients: a nonzero edge fraction
        // that rounds to zero retains the dense path and direct provenance.
        let mut edges = rectangle_edges(0.0, 8.0, 0.0, 12.0);
        edges[0].x = ONE - 1;
        edges[0].upper_x = ONE - 1;
        assert_eq!(compare_rectangle_events(edges, false, clips[0]), 0);
    }

    #[test]
    fn convex_real_rects_preserve_curve_to_vertical_connections() {
        let mut events = 0;
        for left in [-2.25, 0.0, 0.25, 1.75, 11.5] {
            for top in [-1.25, 0.0, 0.25, 2.75, 9.5] {
                for radius in [0.25, 0.75, 1.5, 3.75] {
                    let right = left + 20.5;
                    let bottom = top + 17.25;
                    let edges = [
                        SkAnalyticEdge::setQuadratic([
                            (left + radius, top),
                            (left, top),
                            (left, top + radius),
                        ]),
                        SkAnalyticEdge::setLine((left, top + radius), (left, bottom - radius)),
                        SkAnalyticEdge::setQuadratic([
                            (left, bottom - radius),
                            (left, bottom),
                            (left + radius, bottom),
                        ]),
                        SkAnalyticEdge::setQuadratic([
                            (right - radius, top),
                            (right, top),
                            (right, top + radius),
                        ]),
                        SkAnalyticEdge::setLine((right, top + radius), (right, bottom - radius)),
                        SkAnalyticEdge::setQuadratic([
                            (right, bottom - radius),
                            (right, bottom),
                            (right - radius, bottom),
                        ]),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>();
                    for clip in [
                        ClipBox {
                            left: 0.0,
                            top: 0.0,
                            right: 37.0,
                            bottom: 29.0,
                        },
                        ClipBox {
                            left: 2.0,
                            top: 3.0,
                            right: 31.0,
                            bottom: 23.0,
                        },
                        ClipBox {
                            left: 10.0,
                            top: 8.0,
                            right: 20.0,
                            bottom: 16.0,
                        },
                    ] {
                        for small in [false, true] {
                            events += compare_rectangle_events(edges.clone(), small, clip);
                        }
                    }
                }
            }
        }
        assert!(events > 100);
    }
}

#[cfg(test)]
mod direct_span_tests {
    use super::*;

    fn expanded_blits(mask: &AdditiveBlitter) -> Vec<(usize, u8, bool)> {
        let mut expanded = Vec::new();
        let mut spans = mask.spans.iter().peekable();
        for at in 0..=mask.blits.len() {
            while spans.peek().is_some_and(|span| span.blit_index == at) {
                let span = spans.next().unwrap();
                assert_eq!(span.alpha, 255);
                assert!(span.width > 0);
                assert!(span.x >= 0 && span.x + span.width <= mask.width as i32);
                assert!(span.y >= 0 && span.y < mask.height as i32);
                for x in span.x..span.x + span.width {
                    expanded.push((
                        span.y as usize * mask.width as usize + x as usize,
                        span.alpha,
                        span.pair,
                    ));
                }
            }
            if at < mask.blits.len() {
                expanded.push(mask.blits[at]);
            }
        }
        assert!(
            spans.next().is_none(),
            "span index must preserve event order"
        );
        expanded
    }

    fn buffer(small: bool, suppress: bool, spans: bool) -> AdditiveBlitter {
        let count = 517 * 8;
        AdditiveBlitter {
            width: 517,
            height: 8,
            pixels: (0..count).map(|i| (i % 251) as u8).collect(),
            small,
            direct: (0..count).map(|i| i % 17 == 0).collect(),
            emitting_direct: false,
            pairs: (0..count).map(|i| i % 19 == 0).collect(),
            emitting_pair: false,
            suppress_real: suppress,
            record_direct: true,
            blits: vec![(5, 7, false), (17, 13, true)],
            run_starts: vec![5, 17],
            rects: Vec::new(),
            spans: Vec::new(),
            record_spans: spans,
            color_clip_stream: None,
            #[cfg(any(test, feature = "profiling"))]
            partial_alpha_per_pixel: partial_alpha_legacy_driver(),
        }
    }

    #[test]
    fn direct_horizontal_spans_preserve_trapezoid_event_order_and_every_plane() {
        let mut event_count = 0;
        let bounds = [
            -2 * ONE - ONE / 2,
            0,
            ONE / 4,
            ONE,
            254 * ONE + 3 * ONE / 4,
            255 * ONE,
            256 * ONE + ONE / 4,
            510 * ONE + 3 * ONE / 4,
            518 * ONE,
        ];
        for &left in &bounds {
            for &right in &bounds {
                if right < left {
                    continue;
                }
                for delta in [-ONE / 2, 0, ONE / 4, 3 * ONE / 2] {
                    for y in [-1, 0, 7, 8] {
                        for (small, suppress) in
                            [(false, false), (false, true), (true, false), (true, true)]
                        {
                            let mut expected = buffer(small, suppress, false);
                            let mut actual = buffer(small, suppress, true);
                            for full in [0, 63, 128, 254, 255] {
                                // Several calls share each stream so span insertion must
                                // preserve order among both earlier and later edge pixels.
                                expected.trapezoid(
                                    y,
                                    left,
                                    right,
                                    left + delta,
                                    right + delta,
                                    2 * ONE,
                                    2 * ONE,
                                    full,
                                );
                                actual.trapezoid(
                                    y,
                                    left,
                                    right,
                                    left + delta,
                                    right + delta,
                                    2 * ONE,
                                    2 * ONE,
                                    full,
                                );
                            }
                            assert_eq!(actual.pixels, expected.pixels);
                            assert_eq!(actual.direct, expected.direct);
                            assert_eq!(actual.pairs, expected.pairs);
                            assert_eq!(actual.run_starts, expected.run_starts);
                            assert_eq!(expanded_blits(&actual), expected.blits);
                            if small || suppress {
                                assert!(actual.spans.is_empty());
                            }
                            event_count += actual.spans.len();
                        }
                    }
                }
            }
        }
        assert!(event_count > 100);
    }

    fn polygon_edges(points: &[Pt]) -> Vec<SkAnalyticEdge> {
        points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(points.len())
            .filter_map(|(&a, &b)| SkAnalyticEdge::setLine(a, b))
            .collect()
    }

    #[test]
    fn general_walk_spans_preserve_winding_clipping_partial_rows_and_runs() {
        let shapes = [
            vec![(0.25, 0.25), (35.75, 0.25), (35.75, 27.75), (0.25, 27.75)],
            vec![(-2.25, -1.25), (38.75, 1.75), (31.25, 26.25), (2.75, 30.75)],
            vec![
                (0.25, 0.25),
                (35.75, 0.25),
                (35.75, 25.75),
                (23.25, 25.75),
                (23.25, 5.25),
                (11.75, 5.25),
                (11.75, 25.75),
                (0.25, 25.75),
            ],
            vec![(1.25, 1.25), (35.25, 25.75), (1.25, 25.75), (35.25, 1.25)],
        ];
        let mut event_count = 0;
        for shape in &shapes {
            for offset in [0.0, 0.25, 0.5, 0.75] {
                let shifted = shape
                    .iter()
                    .map(|&(x, y)| (x + offset, y + offset))
                    .collect::<Vec<_>>();
                let edges = polygon_edges(&shifted);
                for clip in [
                    ClipBox {
                        left: 0.0,
                        top: 0.0,
                        right: 37.0,
                        bottom: 29.0,
                    },
                    ClipBox {
                        left: 3.0,
                        top: 2.0,
                        right: 34.0,
                        bottom: 27.0,
                    },
                    ClipBox {
                        left: 13.0,
                        top: 9.0,
                        right: 23.0,
                        bottom: 19.0,
                    },
                ] {
                    for small in [false, true] {
                        for even_odd in [false, true] {
                            for force_rle in [false, true] {
                                for skip_intersect in [false, true] {
                                    let expected = aaa_walk_edges(
                                        edges.clone(),
                                        37,
                                        29,
                                        small,
                                        clip,
                                        even_odd,
                                        force_rle,
                                        skip_intersect,
                                    );
                                    let actual = aaa_walk_edges_with_spans(
                                        edges.clone(),
                                        37,
                                        29,
                                        small,
                                        clip,
                                        even_odd,
                                        force_rle,
                                        skip_intersect,
                                    );
                                    assert!(expected.spans.is_empty());
                                    assert_eq!(actual.pixels, expected.pixels);
                                    assert_eq!(actual.direct, expected.direct);
                                    assert_eq!(actual.pairs, expected.pairs);
                                    assert_eq!(actual.run_starts, expected.run_starts);
                                    assert_eq!(expanded_blits(&actual), expected.blits);
                                    if small || force_rle {
                                        assert!(actual.spans.is_empty());
                                    }
                                    event_count += actual.spans.len();
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(event_count > 100);
    }

    #[test]
    fn convex_walk_spans_and_rects_reconstruct_previous_dense_output() {
        let mut event_count = 0;
        for left in [-2.25, 0.0, 0.25, 1.75, 11.5] {
            for top in [-1.25, 0.0, 0.25, 2.75, 9.5] {
                for radius in [0.25, 0.75, 1.5, 3.75] {
                    let right = left + 20.5;
                    let bottom = top + 17.25;
                    let edges = [
                        SkAnalyticEdge::setQuadratic([
                            (left + radius, top),
                            (left, top),
                            (left, top + radius),
                        ]),
                        SkAnalyticEdge::setLine((left, top + radius), (left, bottom - radius)),
                        SkAnalyticEdge::setQuadratic([
                            (left, bottom - radius),
                            (left, bottom),
                            (left + radius, bottom),
                        ]),
                        SkAnalyticEdge::setQuadratic([
                            (right - radius, top),
                            (right, top),
                            (right, top + radius),
                        ]),
                        SkAnalyticEdge::setLine((right, top + radius), (right, bottom - radius)),
                        SkAnalyticEdge::setQuadratic([
                            (right, bottom - radius),
                            (right, bottom),
                            (right - radius, bottom),
                        ]),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>();
                    for clip in [
                        ClipBox {
                            left: 0.0,
                            top: 0.0,
                            right: 37.0,
                            bottom: 29.0,
                        },
                        ClipBox {
                            left: 2.0,
                            top: 3.0,
                            right: 31.0,
                            bottom: 23.0,
                        },
                        ClipBox {
                            left: 10.0,
                            top: 8.0,
                            right: 20.0,
                            bottom: 16.0,
                        },
                    ] {
                        for small in [false, true] {
                            let expected =
                                aaa_walk_convex_edges(edges.clone(), 37, 29, small, clip);
                            let legacy = aaa_walk_convex_edges_with_rects(
                                edges.clone(),
                                37,
                                29,
                                small,
                                clip,
                            );
                            assert!(legacy.spans.is_empty());
                            let mut actual = aaa_walk_convex_edges_with_spans(
                                edges.clone(),
                                37,
                                29,
                                small,
                                clip,
                            );
                            let blits = expanded_blits(&actual);
                            event_count += actual.spans.len();
                            actual.record_direct = false;
                            actual.emitting_direct = true;
                            for (index, alpha, pair) in blits {
                                actual.emitting_pair = pair;
                                actual.add((index % 37) as i32, (index / 37) as i32, alpha);
                            }
                            actual.emitting_pair = false;
                            let rects = std::mem::take(&mut actual.rects);
                            for rect in &rects {
                                for y in rect.y..rect.y + rect.height {
                                    for x in rect.x..=rect.x + rect.width + 1 {
                                        let alpha = if x == rect.x {
                                            rect.left_alpha
                                        } else if x == rect.x + rect.width + 1 {
                                            rect.right_alpha
                                        } else {
                                            255
                                        };
                                        if alpha > 0 {
                                            actual.add(x, y, alpha);
                                        }
                                    }
                                }
                            }
                            assert_eq!(actual.pixels, expected.pixels);
                            assert_eq!(actual.direct, expected.direct);
                            assert_eq!(actual.pairs, expected.pairs);
                            assert_eq!(actual.run_starts, expected.run_starts);
                            if small {
                                assert!(actual.spans.is_empty());
                            }
                        }
                    }
                }
            }
        }
        assert!(event_count > 100);
    }
}
