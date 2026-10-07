//! Local mask/tile adapters around the official analytic scanline stages.
//! These return Rust masks and ordered blit metadata; SkScan::AAAFillPath writes a SkBlitter.
use crate::compat::commands::{PaintCornerRadii, PaintRect};
use crate::raster::{IntRect, Mask, Transform};
use crate::src::core::SkAnalyticEdge::{Pt, SkAnalyticEdge};
use crate::src::core::SkCubicClipper as cubic_clip;
use crate::src::core::SkEdgeBuilder::{add_conic, add_line, add_quad};
use crate::src::core::SkEdgeClipper::{clipped_quad, clipped_quad_with_culling, ClipBox};
use crate::src::core::SkGeometry as polynomial_roots;
use crate::src::core::SkGeometry::{chop_quad_axis, conic_to_quads};
use crate::src::core::SkPath::convexity as path_geometry;
use crate::src::core::SkScan_AAAPath::{
    aaa_walk_convex_edges, aaa_walk_convex_edges_for_mask, aaa_walk_convex_edges_with_rects,
    aaa_walk_edges, ConvexRect,
};
use crate::src::core::SkTSort as edge_sort;
// Local target metadata. Union only the rectangles whose accumulated alpha
// is copied; never rescan a framebuffer-sized alpha plane to recover support.
fn add_support(support: &mut Option<IntRect>, added: Option<IntRect>) {
    if let Some(added) = added {
        *support = Some(support.map_or(added, |old| {
            IntRect::from_ltrb(
                old.left().min(added.left()),
                old.top().min(added.top()),
                old.right().max(added.right()),
                old.bottom().max(added.bottom()),
            )
            .expect("support rectangles share representable device bounds")
        }));
    }
}

pub struct RoundedCoverage {
    /// Conservative support for accumulated mask bytes. Direct blits retain
    /// their independent ordered list and need no scan over this rectangle.
    pub support_bounds: Option<IntRect>,
    pub mask: Mask,
    pub pairs: Vec<bool>,
    pub blits: Vec<(usize, u8, bool)>,
    pub run_starts: Vec<usize>,
    rects: Vec<ConvexRect>,
    // Only tile-to-span adaptation uses unsnapped alpha and its direct flags.
    raw_direct: Option<Vec<bool>>,
}

// SkRRect scales neighbouring corner radii to fit their shared extent.
// Keep this device-space normalization shared by dense and real-blitter paths.
fn normalized_rrect_radii(w: f32, h: f32, radii: PaintCornerRadii) -> [(f32, f32); 4] {
    let mut r = [
        (radii.top_left.x as f32, radii.top_left.y as f32),
        (radii.top_right.x as f32, radii.top_right.y as f32),
        (radii.bottom_right.x as f32, radii.bottom_right.y as f32),
        (radii.bottom_left.x as f32, radii.bottom_left.y as f32),
    ];
    for (rx, ry) in &mut r {
        if *rx <= 0.0 || *ry <= 0.0 {
            *rx = 0.0;
            *ry = 0.0;
        }
    }
    let mut scale = 1.0f32;
    for (extent, total) in [
        (w, r[0].0 + r[1].0),
        (w, r[2].0 + r[3].0),
        (h, r[0].1 + r[3].1),
        (h, r[1].1 + r[2].1),
    ] {
        if total > extent {
            scale = scale.min(extent / total);
        }
    }
    for (rx, ry) in &mut r {
        *rx *= scale;
        *ry *= scale;
    }
    r
}

// SkDraw builds the device path and decomposes its conics before scan
// conversion. Keep those global f32 operations ahead of integer tile shifts.
// Rebuilding conics in tile-local coordinates changes their fixed coefficients.
struct RoundedGeometry {
    clip: ClipBox,
    horizontal_clip: bool,
    edges: Vec<SkAnalyticEdge>,
}
impl RoundedGeometry {
    fn new(bounds: PaintRect, radii: PaintCornerRadii, canvas: Transform, clip: ClipBox) -> Self {
        let x = bounds.x as f32 + canvas.tx;
        let y = bounds.y as f32 + canvas.ty;
        let w = bounds.width as f32;
        let h = bounds.height as f32;
        let r = normalized_rrect_radii(w, h, radii);
        let points = [
            (x + r[0].0, y),
            (x + w - r[1].0, y),
            (x + w, y + r[1].1),
            (x + w, y + h - r[2].1),
            (x + w - r[2].0, y + h),
            (x + r[3].0, y + h),
            (x, y + h - r[3].1),
            (x, y + r[0].1),
        ];
        let controls = [(x + w, y), (x + w, y + h), (x, y + h), (x, y)];
        let contained = x.floor() >= clip.left
            && y.floor() >= clip.top
            && (x + w).ceil() <= clip.right
            && (y + h).ceil() <= clip.bottom;
        let mut out = Vec::new();
        for i in 0..4 {
            let k = 2 * i;
            let mut a = points[k];
            let mut b = points[k + 1];
            if !contained {
                a.0 = a.0.clamp(clip.left, clip.right);
                b.0 = b.0.clamp(clip.left, clip.right);
                a.1 = a.1.clamp(clip.top, clip.bottom);
                b.1 = b.1.clamp(clip.top, clip.bottom);
            }
            add_line(a, b, &mut out);
            for quad in conic_to_quads(
                [points[k + 1], controls[i], points[(k + 2) % 8]],
                std::f32::consts::FRAC_1_SQRT_2,
            ) {
                if contained {
                    add_quad(quad, &mut out);
                } else {
                    clipped_quad(quad, clip, &mut out);
                }
            }
        }
        Self {
            clip,
            horizontal_clip: x.floor() < clip.left || (x + w).ceil() > clip.right,
            edges: out,
        }
    }
    fn tile_edges(&self, offset: (u32, u32)) -> Vec<SkAnalyticEdge> {
        // Tile boundaries only crop output. Re-chopping a global quad at a
        // virtual tile changes its fixed-point forward differences.
        let mut out = self.edges.clone();
        let tx = (offset.0 as i32) << 16;
        let ty = (offset.1 as i32) << 16;
        for edge in &mut out {
            edge.x -= tx;
            edge.upper_x -= tx;
            edge.y -= ty;
            edge.upper_y -= ty;
            edge.lower_y -= ty;
            if let Some(q) = edge.quad.as_mut() {
                q.x -= tx;
                q.last_x -= tx;
                q.snapped_x -= tx;
                q.y -= ty;
                q.last_y -= ty;
                q.snapped_y -= ty;
            }
            // RoundedGeometry consists only of lines and decomposed conics.
            debug_assert!(edge.cubic.is_none());
        }
        out
    }
}

/// RRect coverage only; no native rendering or cached reference pixels.
fn rounded_rect_tile(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    force_rle: bool,
    clip: ClipBox,
    defer_snap: bool,
) -> Option<RoundedCoverage> {
    rounded_rect_tile_impl::<false, true>(
        bounds, radii, canvas, width, height, force_rle, clip, defer_snap,
    )
}

fn rounded_rect_tile_impl<const ALPHA_ONLY: bool, const RECORD_RUNS: bool>(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    force_rle: bool,
    clip: ClipBox,
    defer_snap: bool,
) -> Option<RoundedCoverage> {
    rounded_rect_tile_with_geometry::<ALPHA_ONLY, RECORD_RUNS>(
        bounds,
        radii,
        canvas,
        width,
        height,
        force_rle,
        clip,
        defer_snap,
        None,
        (0, 0),
    )
}

fn rounded_rect_tile_with_geometry<const ALPHA_ONLY: bool, const RECORD_RUNS: bool>(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    force_rle: bool,
    clip: ClipBox,
    defer_snap: bool,
    geometry: Option<&RoundedGeometry>,
    offset: (u32, u32),
) -> Option<RoundedCoverage> {
    if canvas.sx != 1.0 || canvas.sy != 1.0 || canvas.kx != 0.0 || canvas.ky != 0.0 {
        return None;
    }
    let x = bounds.x as f32 + canvas.tx;
    let y = bounds.y as f32 + canvas.ty;
    let w = bounds.width as f32;
    let h = bounds.height as f32;
    if w <= 0.0
        || h <= 0.0
        || [x, y, x + w, y + h]
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 4096.0)
    {
        return None;
    }
    let r = normalized_rrect_radii(w, h, radii);
    let pts = [
        (x + r[0].0, y),
        (x + w - r[1].0, y),
        (x + w, y + r[1].1),
        (x + w, y + h - r[2].1),
        (x + w - r[2].0, y + h),
        (x + r[3].0, y + h),
        (x, y + h - r[3].1),
        (x, y + r[0].1),
    ];
    let controls = [(x + w, y), (x + w, y + h), (x, y + h), (x, y)];
    let contained = x.floor() >= clip.left
        && y.floor() >= clip.top
        && (x + w).ceil() <= clip.right
        && (y + h).ceil() <= clip.bottom;
    let edge_clip = (!contained).then_some(clip);
    let mut edges = if let Some(geometry) = geometry {
        geometry.tile_edges(offset)
    } else {
        Vec::new()
    };
    for i in if geometry.is_some() { 0..0 } else { 0..4 } {
        let k = 2 * i;
        let mut a = pts[k];
        let mut b = pts[k + 1];
        if let Some(c) = edge_clip {
            a.0 = a.0.clamp(c.left, c.right);
            b.0 = b.0.clamp(c.left, c.right);
            a.1 = a.1.clamp(c.top, c.bottom);
            b.1 = b.1.clamp(c.top, c.bottom);
        }
        add_line(a, b, &mut edges);
        add_conic(
            [pts[k + 1], controls[i], pts[(k + 2) % 8]],
            edge_clip,
            &mut edges,
        );
    }
    let bw = ((x + w).ceil() - x.floor()) as u32;
    let bh = ((y + h).ceil() - y.floor()) as u32;
    let small = !force_rle && bw <= 32 && bw.div_ceil(4) * 4 * bh <= 1024;
    // SkScan::try_blit_fat_anti_rect / SkBlitter::blitFatAntiRect.
    // Small rectangular stroke outlines use scalar coverage instead of
    // quarter-pixel edge stepping, with truncation and endpoint snapping.
    if small && r.iter().all(|&(x, y)| x == 0.0 && y == 0.0) {
        let left = x.max(clip.left);
        let top = y.max(clip.top);
        let right = (x + w).min(clip.right);
        let bottom = (y + h).min(clip.bottom);
        if right.ceil() - left.floor() >= 3.0 && bottom > top {
            let mut mask = Mask::new(width, height)?;
            for py in top.floor().max(0.0) as u32..bottom.ceil().min(height as f32).max(0.0) as u32
            {
                for px in
                    left.floor().max(0.0) as u32..right.ceil().min(width as f32).max(0.0) as u32
                {
                    let dx = right.min(px as f32 + 1.0) - left.max(px as f32);
                    let dy = bottom.min(py as f32 + 1.0) - top.max(py as f32);
                    let a = (dx * dy * 255.0) as u8;
                    mask.data_mut()[(py * width + px) as usize] = if a > 247 {
                        255
                    } else if a < 8 {
                        0
                    } else {
                        a
                    };
                }
            }
            return Some(RoundedCoverage {
                support_bounds: IntRect::from_xywh(0, 0, width, height),
                mask,
                pairs: if ALPHA_ONLY {
                    Vec::new()
                } else {
                    vec![false; (width * height) as usize]
                },
                blits: Vec::new(),
                run_starts: Vec::new(),
                rects: Vec::new(),
                raw_direct: None,
            });
        }
    }
    // A virtual tile is only a bounded output sink. Clamping an edge to its
    // horizontal boundary before stepping changes later AA intersections.
    let horizontal_clip = geometry.map_or(clip, |g| ClipBox {
        left: g.clip.left - offset.0 as f32,
        right: g.clip.right - offset.0 as f32,
        top: clip.top,
        bottom: clip.bottom,
    });
    let scan_clip = ClipBox {
        left: if small {
            horizontal_clip.left.max(x.floor())
        } else {
            horizontal_clip.left
        },
        right: if small {
            horizontal_clip.right.min((x + w).ceil())
        } else {
            horizontal_clip.right
        },
        top: clip.top.max(y.floor()),
        bottom: clip.bottom.min((y + h).ceil()),
    };
    let mut coverage = aaa_walk_convex_edges_for_mask::<ALPHA_ONLY, RECORD_RUNS>(
        edges, width, height, small, scan_clip, defer_snap,
    );
    // Keep the H2 mark on each surviving pixel when virtual tile ownership
    // crops a pair. Only the actual horizontal clip changes its arithmetic.
    // SkRectClipBlitter inherits the generic two-pixel entry point,
    // which forwards through blitAntiH instead of the color H2 kernel.
    if geometry.map_or(x.floor() < clip.left || (x + w).ceil() > clip.right, |g| {
        g.horizontal_clip
    }) {
        // Clearing this initialized bool provenance plane is a local representation
        // operation. Zero is false; bulk zeroing preserves every flag exactly.
        unsafe {
            core::ptr::write_bytes(coverage.pairs.as_mut_ptr(), 0, coverage.pairs.len());
        }
        for blit in &mut coverage.blits {
            blit.2 = false;
        }
    }
    if !small && !defer_snap {
        snap_alpha_runs(&mut coverage.pixels, &coverage.direct);
    }
    // The walker already owns the complete tile coverage. Transfer it into
    // the compatibility mask instead of allocating/zeroing/copying it again.
    let mask = Mask::from_vec(
        coverage.pixels,
        crate::path::IntSize::from_wh(width, height)?,
    )?;
    Some(RoundedCoverage {
        support_bounds: IntRect::from_xywh(0, 0, width, height),
        mask,
        pairs: coverage.pairs,
        blits: coverage.blits,
        run_starts: coverage.run_starts,
        rects: coverage.rects,
        raw_direct: (defer_snap && !small).then_some(coverage.direct),
    })
}

// RunBasedAdditiveBlitter::snapAlpha changes only 1..7 and 248..254.
// Zero/opaque interiors and mid-range AA runs are fixed points; preserving
// direct provenance requires scalar work only for runs that can actually snap.
fn snap_alpha_runs(pixels: &mut [u8], direct: &[bool]) {
    debug_assert_eq!(pixels.len(), direct.len());
    let mut i = 0;
    while i < pixels.len() {
        let alpha = pixels[i];
        let end = equal_byte_run_end(&pixels, i, alpha);
        if (alpha != 0 && alpha < 8) || (alpha > 247 && alpha != 255) {
            let snapped = if alpha < 8 { 0 } else { 255 };
            for x in i..end {
                if !direct[x] {
                    pixels[x] = snapped;
                }
            }
        }
        i = end;
    }
}

// Match skia_renderer.cc::tiles_for_axis, including one-pixel gutters.
pub fn rounded_rect_mask(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    force_rle: bool,
    clip: Option<&Mask>,
) -> Option<RoundedCoverage> {
    rounded_rect_mask_with_clip(bounds, radii, canvas, width, height, force_rle, clip, None)
}

pub(crate) fn rounded_rect_mask_with_clip(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    force_rle: bool,
    clip: Option<&Mask>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
) -> Option<RoundedCoverage> {
    rounded_rect_mask_with_clip_impl(
        bounds,
        radii,
        canvas,
        width,
        height,
        force_rle,
        clip,
        clip_summary,
        true,
    )
}

fn rounded_rect_mask_with_clip_impl(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    force_rle: bool,
    clip: Option<&Mask>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    compact: bool,
) -> Option<RoundedCoverage> {
    let mut result: Option<RoundedCoverage> = None;
    let mut failed = false;
    visit_rounded_rect_tiles(
        bounds,
        radii,
        canvas,
        width,
        height,
        force_rle,
        clip,
        clip_summary,
        false,
        compact,
        None,
        &mut |ox, nx, lx, oy, ny, ly, tw, tile| {
            if result.is_none() {
                let Some(mask) = Mask::new(width, height) else {
                    failed = true;
                    return;
                };
                result = Some(RoundedCoverage {
                    support_bounds: None,
                    mask,
                    pairs: vec![false; (width * height) as usize],
                    blits: Vec::new(),
                    run_starts: Vec::new(),
                    rects: Vec::new(),
                    raw_direct: None,
                });
            }
            let result = result.as_mut().unwrap();
            add_support(
                &mut result.support_bounds,
                IntRect::from_xywh(ox as i32, oy as i32, nx, ny),
            );
            for &i in &tile.run_starts {
                let x = i as u32 % tw;
                let y = i as u32 / tw;
                if x >= lx && x < lx + nx && y >= ly && y < ly + ny {
                    result
                        .run_starts
                        .push(((oy + y - ly) * width + ox + x - lx) as usize);
                }
            }
            for &(i, a, pair) in &tile.blits {
                let x = i as u32 % tw;
                let y = i as u32 / tw;
                if x >= lx && x < lx + nx && y >= ly && y < ly + ny {
                    result
                        .blits
                        .push((((oy + y - ly) * width + ox + x - lx) as usize, a, pair));
                }
            }
            for y in 0..ny {
                let dst = ((oy + y) * width + ox) as usize;
                let src = ((ly + y) * tw + lx) as usize;
                let n = nx as usize;
                result.mask.data_mut()[dst..dst + n]
                    .copy_from_slice(&tile.mask.data()[src..src + n]);
                result.pairs[dst..dst + n].copy_from_slice(&tile.pairs[src..src + n]);
            }
        },
    )?;
    if failed {
        return None;
    }
    if let Some(result) = result {
        Some(result)
    } else {
        Some(RoundedCoverage {
            support_bounds: None,
            mask: Mask::new(width, height)?,
            pairs: vec![false; (width * height) as usize],
            blits: Vec::new(),
            run_starts: Vec::new(),
            rects: Vec::new(),
            raw_direct: None,
        })
    }
}

/// Clip-only target for the same official analytic tile walker. Clipping
/// needs alpha and SkAAClip run boundaries, but never the color blitter's
/// per-pixel two-pixel provenance plane or ordered direct blit list.
/// The dense alpha plane is retained for existing mask consumers; scratch
/// scan conversion and output copies remain limited to intersecting tiles.
pub(crate) struct RoundedClipCoverage {
    pub mask: Mask,
    pub run_starts: Vec<usize>,
}

pub(crate) fn rounded_rect_clip_mask(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    clip: Option<&Mask>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
) -> Option<RoundedClipCoverage> {
    rounded_rect_alpha_target(
        bounds,
        radii,
        canvas,
        width,
        height,
        true,
        clip,
        clip_summary,
        true,
    )
}

/// SkDraw::DrawToMask-style target for shadow input: collect alpha from the
/// same analytic walker without color-blitter provenance or clip run storage.
pub(crate) fn rounded_rect_alpha_mask(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    force_rle: bool,
) -> Option<Mask> {
    rounded_rect_alpha_target(
        bounds, radii, canvas, width, height, force_rle, None, None, false,
    )
    .map(|output| output.mask)
}

/// SkDraw::DrawToMask-style finite A8 storage. Region coordinates stay in
/// the original device: raster tile sizes, gutter ownership and edge clipping
/// still use full_width/full_height. Only final A8 storage has a local origin.
pub(crate) fn rounded_rect_alpha_mask_region(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    full_width: u32,
    full_height: u32,
    force_rle: bool,
    region: IntRect,
) -> Option<Mask> {
    if full_width == 0 || full_height == 0 {
        return None;
    }
    let mut output = Mask::new(region.width(), region.height())?;
    let mut rect_spans = Vec::new();
    let mut rect_target = |ox, nx, lx, oy, ny, ly, rect: &ConvexRect, full_alpha| {
        // Keep global span indices until the same real-blitter rectangle
        // events have been emitted. Storage clipping never alters edge math.
        append_convex_rect_spans(
            &mut rect_spans,
            rect,
            full_alpha,
            full_width,
            ox,
            nx,
            lx,
            oy,
            ny,
            ly,
        );
    };
    visit_rounded_rect_tiles_impl::<true, false>(
        bounds,
        radii,
        canvas,
        full_width,
        full_height,
        force_rle,
        None,
        None,
        false,
        true,
        Some(&mut rect_target),
        &mut |ox, nx, lx, oy, ny, ly, tw, tile| {
            let Some(copy) =
                IntRect::from_xywh(ox as i32, oy as i32, nx, ny).and_then(|b| b.intersect(&region))
            else {
                return;
            };
            let local_left = (i64::from(copy.left()) - i64::from(region.left())) as usize;
            let local_top = (i64::from(copy.top()) - i64::from(region.top())) as usize;
            let tile_left = lx as usize + (copy.left() as u32 - ox) as usize;
            let tile_top = ly as usize + (copy.top() as u32 - oy) as usize;
            let count = copy.width() as usize;
            for y in 0..copy.height() as usize {
                let dst = (local_top + y) * region.width() as usize + local_left;
                let src = (tile_top + y) * tw as usize + tile_left;
                output.data_mut()[dst..dst + count]
                    .copy_from_slice(&tile.mask.data()[src..src + count]);
            }
        },
        (0, 0),
    )?;
    for span in rect_spans {
        let y = (span.start / full_width as usize) as i64;
        if y < i64::from(region.top()) || y >= i64::from(region.bottom()) {
            continue;
        }
        let x = (span.start % full_width as usize) as i64;
        let left = x.max(i64::from(region.left()));
        let right = (x + span.count as i64).min(i64::from(region.right()));
        if left < right {
            let dst = (y - i64::from(region.top())) as usize * region.width() as usize
                + (left - i64::from(region.left())) as usize;
            output.data_mut()[dst..dst + (right - left) as usize].fill(span.coverage);
        }
    }
    Some(output)
}

fn rounded_rect_alpha_target(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    force_rle: bool,
    clip: Option<&Mask>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    collect_runs: bool,
) -> Option<RoundedClipCoverage> {
    if collect_runs {
        rounded_rect_alpha_target_with_sink::<true, true>(
            bounds,
            radii,
            canvas,
            width,
            height,
            force_rle,
            clip,
            clip_summary,
            collect_runs,
        )
    } else {
        rounded_rect_alpha_target_with_sink::<true, false>(
            bounds,
            radii,
            canvas,
            width,
            height,
            force_rle,
            clip,
            clip_summary,
            collect_runs,
        )
    }
}

fn rounded_rect_alpha_target_with_sink<const ALPHA_ONLY: bool, const RECORD_RUNS: bool>(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    force_rle: bool,
    clip: Option<&Mask>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    collect_runs: bool,
) -> Option<RoundedClipCoverage> {
    // Keep the original device tile grid and global run indices. Only the
    // A8 destination storage changes to SkMask's bounds/rowBytes model.
    let x = bounds.x as f32 * canvas.sx + canvas.tx;
    let y = bounds.y as f32 * canvas.sy + canvas.ty;
    let right = x + bounds.width as f32 * canvas.sx;
    let bottom = y + bounds.height as f32 * canvas.sy;
    if ![x, y, right, bottom].into_iter().all(f32::is_finite) {
        return None;
    }
    let device = crate::raster::IntRect::from_xywh(0, 0, width, height)?;
    let left = (x.floor() - 1.0).clamp(0.0, width as f32) as i32;
    let top = (y.floor() - 1.0).clamp(0.0, height as f32) as i32;
    let right = (right.ceil() + 1.0).clamp(0.0, width as f32) as i32;
    let bottom = (bottom.ceil() + 1.0).clamp(0.0, height as f32) as i32;
    let mut support = crate::raster::IntRect::from_xywh(
        left,
        top,
        right.saturating_sub(left) as u32,
        bottom.saturating_sub(top) as u32,
    );
    if let Some(summary) = clip_summary {
        support = support.and_then(|b| summary.getBounds().and_then(|c| b.intersect(&c)));
    } else if let Some(clip) = clip {
        support = support.and_then(|b| b.intersect(&clip.storage_bounds()));
    }
    // IntRect cannot represent empty bounds: one implicit-zero pixel is the
    // local carrier for an empty logical device mask.
    let storage = support.unwrap_or(crate::raster::IntRect::from_xywh(0, 0, 1, 1)?);
    debug_assert!(device.contains(&storage));
    let mut output = RoundedClipCoverage {
        mask: Mask::new_bounded(width, height, storage)?,
        run_starts: Vec::new(),
    };
    let mut rect_spans = Vec::new();
    let mut rect_target = |ox, nx, lx, oy, ny, ly, rect: &ConvexRect, full_alpha| {
        append_convex_rect_spans(
            &mut rect_spans,
            rect,
            full_alpha,
            width,
            ox,
            nx,
            lx,
            oy,
            ny,
            ly,
        );
    };
    let rect_target: Option<&mut dyn FnMut(u32, u32, u32, u32, u32, u32, &ConvexRect, u8)> =
        Some(&mut rect_target);
    visit_rounded_rect_tiles_impl::<ALPHA_ONLY, RECORD_RUNS>(
        bounds,
        radii,
        canvas,
        width,
        height,
        force_rle,
        clip,
        clip_summary,
        false,
        true,
        rect_target,
        &mut |ox, nx, lx, oy, ny, ly, tw, tile| {
            if collect_runs {
                for &i in &tile.run_starts {
                    let x = i as u32 % tw;
                    let y = i as u32 / tw;
                    if x >= lx && x < lx + nx && y >= ly && y < ly + ny {
                        output
                            .run_starts
                            .push(((oy + y - ly) * width + ox + x - lx) as usize);
                    }
                }
            }
            if let Some(bounds) = support {
                let left = ox.max(bounds.left() as u32);
                let right = (ox + nx).min(bounds.right() as u32);
                for y in oy.max(bounds.top() as u32)..(oy + ny).min(bounds.bottom() as u32) {
                    if left < right {
                        let src = ((ly + y - oy) * tw + lx + left - ox) as usize;
                        output
                            .mask
                            .row_range_mut(y, left, right)
                            .copy_from_slice(&tile.mask.data()[src..src + (right - left) as usize]);
                    }
                }
            }
        },
        (0, 0),
    )?;
    for span in rect_spans {
        let y = (span.start / width as usize) as u32;
        let x = (span.start % width as usize) as u32;
        if let Some(bounds) = support.filter(|b| y >= b.top() as u32 && y < b.bottom() as u32) {
            let left = x.max(bounds.left() as u32);
            let right = (x + span.count as u32).min(bounds.right() as u32);
            if left < right {
                output
                    .mask
                    .row_range_mut(y, left, right)
                    .fill(span.coverage);
            }
        }
    }
    Some(output)
}

// Local interval carrier for SkBlitter::blitAntiH-style output. The analytic
// walker still uses its existing small tile storage; this removes the final
// framebuffer-sized coverage image and its later interval reconstruction.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CoverageSpan {
    pub start: usize,
    pub count: usize,
    pub coverage: u8,
    pub pair: bool,
}
#[derive(Default)]
pub(crate) struct RoundedSpans {
    pub direct: Vec<CoverageSpan>,
    pub accumulated: Vec<CoverageSpan>,
}

// Dense-tile to run-list adapter, not an upstream RLE implementation.
// Read fixed-width words to skip equal coverage/flag groups. Every actual load
// covers eight initialized one-byte values within its slice; unaligned loads
// are supported. Rust bool is represented by a byte containing 0 or 1.
pub(crate) fn equal_byte_run_end(row: &[u8], start: usize, value: u8) -> usize {
    debug_assert!(start <= row.len());
    let word = u64::from_ne_bytes([value; 8]);
    let mut end = start;
    #[cfg(all(feature = "simd", target_arch = "aarch64", target_endian = "little"))]
    {
        let mut pointer = row.as_ptr().wrapping_add(start);
        let mut groups = (row.len() - start) / 8;
        // The local alpha-run target compares exactly the same initialized
        // eight-byte words as the scalar adapter. Keep that scan in registers
        // in debug builds; the first differing byte is found from the xor.
        // SAFETY: groups counts complete accessible words only. Read-only loads
        // never cross the slice end; the scalar tail covers remaining bytes.
        unsafe {
            core::arch::asm!(
                "cbz {groups}, 4f",
                "2:",
                "ldr {diff}, [{pointer}]",
                "eor {diff}, {diff}, {word}",
                "cbnz {diff}, 3f",
                "add {pointer}, {pointer}, #8",
                "subs {groups}, {groups}, #1",
                "b.ne 2b",
                "b 4f",
                "3:",
                "rbit {diff}, {diff}",
                "clz {diff}, {diff}",
                "lsr {diff}, {diff}, #3",
                "add {pointer}, {pointer}, {diff}",
                "4:",
                pointer = inout(reg) pointer,
                groups = inout(reg) groups,
                word = in(reg) word,
                diff = out(reg) _,
                options(nostack, readonly),
            );
        }
        end = pointer as usize - row.as_ptr() as usize;
        if groups != 0 {
            return end;
        }
    }
    #[cfg(not(all(feature = "simd", target_arch = "aarch64", target_endian = "little")))]
    while end + 8 <= row.len() {
        // SAFETY: each unaligned word is fully inside the initialized slice.
        let current = unsafe { core::ptr::read_unaligned(row.as_ptr().add(end).cast::<u64>()) };
        if current != word {
            break;
        }
        end += 8;
    }
    while end < row.len() && row[end] == value {
        end += 1;
    }
    end
}

fn coverage_run_end(row: &[u8], pairs: &[bool], direct: Option<&[bool]>, start: usize) -> usize {
    debug_assert_eq!(row.len(), pairs.len());
    debug_assert!(direct.is_none_or(|flags| flags.len() == row.len()));
    let raw = row[start];
    let pair = pairs[start];
    // Direct provenance is immaterial once alpha is already 255, as in the
    // existing adapter. For other alpha values it controls snapAlpha at flush.
    let direct = direct.filter(|_| raw != 255);
    let is_direct = direct.is_some_and(|flags| flags[start]);
    let alpha_word = u64::from_ne_bytes([raw; 8]);
    let pair_word = u64::from_ne_bytes([pair as u8; 8]);
    let direct_word = u64::from_ne_bytes([is_direct as u8; 8]);
    let mut end = start;
    while end + 8 <= row.len() {
        let matches = unsafe {
            core::ptr::read_unaligned(row.as_ptr().wrapping_add(end).cast::<u64>()) == alpha_word
                && core::ptr::read_unaligned(pairs.as_ptr().wrapping_add(end).cast::<u64>())
                    == pair_word
                && direct.is_none_or(|flags| {
                    core::ptr::read_unaligned(flags.as_ptr().wrapping_add(end).cast::<u64>())
                        == direct_word
                })
        };
        if !matches {
            break;
        }
        end += 8;
    }
    while end < row.len()
        && row[end] == raw
        && pairs[end] == pair
        && direct.is_none_or(|flags| flags[end] == is_direct)
    {
        end += 1;
    }
    end
}

// blitAntiRect's left coverage / opaque interior / right coverage, adapted
// to framebuffer-addressed intervals. Tile gutters never reach the target.
fn append_convex_rect_spans(
    output: &mut Vec<CoverageSpan>,
    rect: &ConvexRect,
    full_alpha: u8,
    width: u32,
    ox: u32,
    nx: u32,
    lx: u32,
    oy: u32,
    ny: u32,
    ly: u32,
) {
    let left = rect.x;
    let interior_left = left + 1;
    let interior_right = interior_left + rect.width;
    let top = rect.y.max(ly as i32);
    let bottom = (rect.y + rect.height).min((ly + ny) as i32);
    for y in top..bottom {
        let mut emit = |x: i32, count: i32, coverage: u8| {
            let end = (x + count).min((lx + nx) as i32);
            let x = x.max(lx as i32);
            if coverage != 0 && end > x {
                output.push(CoverageSpan {
                    start: ((oy + y as u32 - ly) * width + ox + x as u32 - lx) as usize,
                    count: (end - x) as usize,
                    coverage,
                    pair: false,
                });
            }
        };
        emit(left, 1, rect.left_alpha);
        emit(interior_left, rect.width, full_alpha);
        emit(interior_right, 1, rect.right_alpha);
    }
}

pub(crate) fn rounded_rect_spans(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    clip: Option<&Mask>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
) -> Option<RoundedSpans> {
    rounded_rect_spans_in_device(
        bounds,
        radii,
        canvas,
        width,
        height,
        clip,
        clip_summary,
        (0, 0),
    )
}

/// Local replay adapter: a bounded layer intersects the original renderer's
/// root tiles instead of starting a different tiling grid at its own origin.
/// SkCanvas::internalSaveLayer changes device origin, not page-space geometry.
pub(crate) fn rounded_rect_spans_in_device(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    clip: Option<&Mask>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    tile_origin: (i32, i32),
) -> Option<RoundedSpans> {
    let mut output = RoundedSpans::default();
    let mut rect_only = Vec::new();
    visit_rounded_rect_tiles_impl::<false, true>(
        bounds,
        radii,
        canvas,
        width,
        height,
        false,
        clip,
        clip_summary,
        true,
        true,
        Some(&mut |ox, nx, lx, oy, ny, ly, rect, full_alpha| {
            append_convex_rect_spans(
                &mut rect_only,
                rect,
                full_alpha,
                width,
                ox,
                nx,
                lx,
                oy,
                ny,
                ly,
            );
        }),
        &mut |ox, nx, lx, oy, ny, ly, tw, tile| {
            // Preserve direct-event order; accumulation is flushed afterward,
            // as in RunBasedAdditiveBlitter::flush. Gutter pixels are excluded.
            for &(i, coverage, pair) in &tile.blits {
                let x = i as u32 % tw;
                let y = i as u32 / tw;
                if coverage == 0 || x < lx || x >= lx + nx || y < ly || y >= ly + ny {
                    continue;
                }
                let start = ((oy + y - ly) * width + ox + x - lx) as usize;
                if let Some(last) = output.direct.last_mut().filter(|last| {
                    last.start + last.count == start
                        && last.start / width as usize == start / width as usize
                        && last.coverage == coverage
                        && last.pair == pair
                }) {
                    last.count += 1;
                } else {
                    output.direct.push(CoverageSpan {
                        start,
                        count: 1,
                        coverage,
                        pair,
                    });
                }
            }
            // These are the official convex walker's real-blitter
            // blitAntiRect events. Preserve edge alpha and crop tile gutters;
            // their full integer rows have no additive dense contributions.
            for rect in &tile.rects {
                append_convex_rect_spans(
                    &mut output.accumulated,
                    rect,
                    255,
                    width,
                    ox,
                    nx,
                    lx,
                    oy,
                    ny,
                    ly,
                );
            }
            for y in 0..ny {
                let tile_y = (ly + y) as i32;
                if tile
                    .rects
                    .iter()
                    .any(|r| r.y <= tile_y && tile_y < r.y + r.height)
                {
                    continue;
                }
                let row_start = ((ly + y) * tw + lx) as usize;
                let row = &tile.mask.data()[row_start..row_start + nx as usize];
                let pairs = &tile.pairs[row_start..row_start + nx as usize];
                let direct = tile
                    .raw_direct
                    .as_ref()
                    .map(|flags| &flags[row_start..row_start + nx as usize]);
                let mut x = 0;
                while x < nx as usize {
                    let raw = row[x];
                    if raw == 0 {
                        x = equal_byte_run_end(row, x, 0);
                        continue;
                    }
                    let pair = pairs[x];
                    let is_direct = direct.is_some_and(|flags| flags[x]);
                    let end = coverage_run_end(row, pairs, direct, x);
                    let coverage = if direct.is_some() && !is_direct {
                        if raw > 247 {
                            255
                        } else if raw < 8 {
                            0
                        } else {
                            raw
                        }
                    } else {
                        raw
                    };
                    if coverage != 0 {
                        output.accumulated.push(CoverageSpan {
                            start: ((oy + y) * width + ox) as usize + x,
                            count: end - x,
                            coverage,
                            pair,
                        });
                    }
                    x = end;
                }
            }
        },
        tile_origin,
    )?;
    output.accumulated.extend(rect_only);
    Some(output)
}

fn visit_rounded_rect_tiles(
    bounds: PaintRect,
    radii: PaintCornerRadii,
    canvas: Transform,
    width: u32,
    height: u32,
    force_rle: bool,
    clip: Option<&Mask>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    defer_snap: bool,
    compact: bool,
    emit_rect: Option<&mut dyn FnMut(u32, u32, u32, u32, u32, u32, &ConvexRect, u8)>,
    emit: &mut impl FnMut(u32, u32, u32, u32, u32, u32, u32, &RoundedCoverage),
) -> Option<()> {
    visit_rounded_rect_tiles_impl::<false, true>(
        bounds,
        radii,
        canvas,
        width,
        height,
        force_rle,
        clip,
        clip_summary,
        defer_snap,
        compact,
        emit_rect,
        emit,
        (0, 0),
    )
}

fn visit_rounded_rect_tiles_impl<const ALPHA_ONLY: bool, const RECORD_RUNS: bool>(
    mut bounds: PaintRect,
    mut radii: PaintCornerRadii,
    mut canvas: Transform,
    width: u32,
    height: u32,
    force_rle: bool,
    clip: Option<&Mask>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    defer_snap: bool,
    compact: bool,
    mut emit_rect: Option<&mut dyn FnMut(u32, u32, u32, u32, u32, u32, &ConvexRect, u8)>,
    emit: &mut impl FnMut(u32, u32, u32, u32, u32, u32, u32, &RoundedCoverage),
    tile_origin: (i32, i32),
) -> Option<()> {
    // SkDraw transforms geometry before SkScan emits device-space spans.
    // Reject unsupported matrices before clip scanning or mask allocation.
    if canvas.kx != 0.0
        || canvas.ky != 0.0
        || canvas.sx <= 0.0
        || canvas.sy <= 0.0
        || ![canvas.sx, canvas.sy, canvas.tx, canvas.ty]
            .into_iter()
            .all(f32::is_finite)
    {
        return None;
    }
    if canvas.sx != 1.0 || canvas.sy != 1.0 {
        bounds.x = (bounds.x as f32 * canvas.sx) as f64;
        bounds.y = (bounds.y as f32 * canvas.sy) as f64;
        bounds.width = (bounds.width as f32 * canvas.sx) as f64;
        bounds.height = (bounds.height as f32 * canvas.sy) as f64;
        for r in [
            &mut radii.top_left,
            &mut radii.top_right,
            &mut radii.bottom_left,
            &mut radii.bottom_right,
        ] {
            r.x = (r.x as f32 * canvas.sx) as f64;
            r.y = (r.y as f32 * canvas.sy) as f64;
        }
        canvas.sx = 1.0;
        canvas.sy = 1.0;
    }
    // Every replay tile uses the same device geometry, with only a translation
    // offset. Validate its extent once, before allocating the full-sized mask.
    let x = bounds.x as f32 + canvas.tx;
    let y = bounds.y as f32 + canvas.ty;
    let right = x + bounds.width as f32;
    let bottom = y + bounds.height as f32;
    if bounds.width <= 0.0
        || bounds.height <= 0.0
        || ![x, y, right, bottom].into_iter().all(f32::is_finite)
    {
        return None;
    }
    // The fixed-point scan converter supports coordinates within 4096 of
    // each tile's origin. Check all possible tile translations up front.
    let last_origin = |extent: u32| {
        if extent <= 255 {
            0.0
        } else {
            (254 * (1 + (extent - 256) / 254)) as f32
        }
    };
    if x.abs()
        .max(right.abs())
        .max((x - last_origin(width)).abs())
        .max((right - last_origin(width)).abs())
        > 4096.0
        || y.abs()
            .max(bottom.abs())
            .max((y - last_origin(height)).abs())
            .max((bottom - last_origin(height)).abs())
            > 4096.0
    {
        return None;
    }
    let mut bbox = ClipBox {
        left: 0.0,
        top: 0.0,
        right: width as f32,
        bottom: height as f32,
    };
    if let Some(summary) = clip_summary {
        bbox = summary.getBounds().map_or(
            ClipBox {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            |b| ClipBox {
                left: b.left() as f32,
                top: b.top() as f32,
                right: b.right() as f32,
                bottom: b.bottom() as f32,
            },
        );
    } else if let Some(m) = clip {
        let mut l = width;
        let mut t = height;
        let mut r = 0;
        let mut b = 0;
        for (y, row) in m.data().chunks_exact(width as usize).enumerate() {
            if let Some(left) = row.iter().position(|&a| a != 0) {
                let right = row.iter().rposition(|&a| a != 0).unwrap() + 1;
                l = l.min(left as u32);
                t = t.min(y as u32);
                r = r.max(right as u32);
                b = b.max(y as u32 + 1);
            }
        }
        bbox = ClipBox {
            left: l as f32,
            top: t as f32,
            right: r as f32,
            bottom: b as f32,
        };
    }
    if ALPHA_ONLY && RECORD_RUNS {
        // SkAAClip::setPath constructs its Builder with pathBounds.roundOut
        // intersected with the prior clip. That is a real scan bound: fixed
        // curve stepping may overshoot the mathematical edge by a subpixel.
        // The snug right bound changes the last interior coverage too, so
        // cropping a completed full-device mask is not equivalent. Draw and
        // pixel-only shadow targets retain their original scan bounds.
        bbox.left = bbox.left.max(x.floor());
        bbox.top = bbox.top.max(y.floor());
        bbox.right = bbox.right.min(right.ceil());
        bbox.bottom = bbox.bottom.min(bottom.ceil());
    }
    if bbox.left >= bbox.right || bbox.top >= bbox.bottom {
        return Some(());
    }
    if tile_origin.0 < 0 || tile_origin.1 < 0 {
        return None;
    }
    let global_geometry = RoundedGeometry::new(bounds, radii, canvas, bbox);
    let tiles = |extent: u32, device_origin: i32| {
        let mut tiles = Vec::new();
        let base = i64::from(device_origin);
        let end = base + i64::from(extent);
        let mut owned_left = base;
        while owned_left < end {
            let global_left = if owned_left < 255 {
                0
            } else {
                255 + (owned_left - 255) / 254 * 254
            };
            let global_right = global_left + if global_left == 0 { 255 } else { 254 };
            let owned_right = global_right.min(end);
            let raster_left = (global_left - i64::from(global_left != 0)).max(base);
            let raster_right = (global_right + 1).min(end);
            tiles.push((
                (owned_left - base) as u32,
                (owned_right - owned_left) as u32,
                (owned_left - raster_left) as u32,
                (raster_right - raster_left) as u32,
            ));
            owned_left = owned_right;
        }
        tiles
    };
    for (oy, ny, ly, th) in tiles(height, tile_origin.1) {
        for (ox, nx, lx, tw) in tiles(width, tile_origin.0) {
            // quickReject before scan conversion and tile-sized allocations.
            if ox as f32 >= right.ceil()
                || oy as f32 >= bottom.ceil()
                || (ox + nx) as f32 <= x.floor()
                || (oy + ny) as f32 <= y.floor()
            {
                continue;
            }
            // SkScan::AAAFillPath uses the path / clip intersection for
            // temporary scan storage. Retain this adapter's original tile
            // origin and leading gutter, which keep floating-point device
            // edge construction unchanged. Trim only unused trailing rows /
            // columns. An extra conservative coverage row / column preserves
            // terminal fixed-point samples before the one-pixel trailing gutter.
            let (nx, tw, ny, th) = if compact {
                let right = (ox + nx)
                    .min((right.ceil().max(0.0) as u32).saturating_add(1))
                    .min(bbox.right.max(0.0) as u32);
                let bottom = (oy + ny)
                    .min((bottom.ceil().max(0.0) as u32).saturating_add(1))
                    .min(bbox.bottom.max(0.0) as u32);
                if ox >= right || oy >= bottom {
                    continue;
                }
                let nx = right - ox;
                let ny = bottom - oy;
                (
                    nx,
                    lx + nx + u32::from(right < width),
                    ny,
                    ly + ny + u32::from(bottom < height),
                )
            } else {
                (nx, tw, ny, th)
            };
            let sx = ox - lx;
            let sy = oy - ly;
            let c = ClipBox {
                left: (bbox.left - sx as f32).max(0.0),
                top: (bbox.top - sy as f32).max(0.0),
                right: (bbox.right - sx as f32).min(tw as f32),
                bottom: (bbox.bottom - sy as f32).min(th as f32),
            };
            if c.left >= c.right || c.top >= c.bottom {
                continue;
            }
            if let Some(emit_rect) = emit_rect.as_mut() {
                let r = normalized_rrect_radii(bounds.width as f32, bounds.height as f32, radii);
                let straight_top = (y + r[0].1.max(r[1].1)).ceil() + 1.0;
                let straight_bottom = (bottom - r[2].1.max(r[3].1)).floor() - 1.0;
                let device_top = c.top + sy as f32;
                let device_bottom = c.bottom + sy as f32;
                if r.iter().all(|&(rx, ry)| rx.is_finite() && ry.is_finite())
                    && r.iter().any(|&(rx, ry)| rx != 0.0 || ry != 0.0)
                    && device_top >= straight_top
                    && device_bottom <= straight_bottom
                {
                    // Selected SkScan_AAAPath::aaa_walk_convex_edges vertical
                    // branch: both slopes are zero and all clipped rows have
                    // integer height, so only getRealBlitter()->blitAntiRect
                    // is emitted. The conservative one-row guard excludes
                    // rounded smooth curve/line joins. This local target
                    // adapter can replay that event without dense mask planes.
                    let local_x = bounds.x as f32 + (canvas.tx - sx as f32);
                    let local_right = local_x + bounds.width as f32;
                    let left = ((local_x.max(c.left) * 256.0) as i32) << 8;
                    let right = ((local_right.min(c.right) * 256.0) as i32) << 8;
                    let full_left = (left + 65535) >> 16;
                    let full_right = right >> 16;
                    if right > left && full_right >= full_left {
                        let alpha = |v: i32| ((255 * v + 32768) >> 16) as u8;
                        let rect = ConvexRect {
                            x: full_left - 1,
                            y: c.top as i32,
                            width: full_right - full_left,
                            height: (c.bottom - c.top) as i32,
                            left_alpha: alpha((full_left << 16) - left),
                            right_alpha: alpha(right - (full_right << 16)),
                        };
                        emit_rect(ox, nx, lx, oy, ny, ly, &rect, 255);
                        continue;
                    }
                }
            }
            // The output tile's opaque center does not imply vertical
            // global edges. A clip target must retain actual curved-row H
            // restart positions (SkAAClip::Builder::AppendRun caps at255),
            // even when every surviving output alpha is255. The source
            // walker below records them before the bounded sink crops.
            // Pixel-only shadow/RGB targets do not consume that metadata.
            if let Some(emit_rect) = emit_rect.as_mut().filter(|_| !(ALPHA_ONLY && RECORD_RUNS)) {
                let r = normalized_rrect_radii(bounds.width as f32, bounds.height as f32, radii);
                let straight_left = (x + r[0].0.max(r[3].0)).ceil() + 1.0;
                let straight_right = (right - r[1].0.max(r[2].0)).floor() - 1.0;
                if r.iter().all(|&(rx, ry)| rx.is_finite() && ry.is_finite())
                    && r.iter().any(|&(rx, ry)| rx != 0.0 || ry != 0.0)
                    && c.left + sx as f32 >= straight_left
                    && c.right + sx as f32 <= straight_right
                    && [c.left, c.top, c.right, c.bottom]
                        .iter()
                        .all(|v| v.fract() == 0.0)
                {
                    // Both corner contours lie outside this horizontal clip.
                    // SkEdgeClipper emits vertical boundary lines, preserving
                    // SkAnalyticEdge's quarter-pixel Y snapping. Replay the
                    // same vertical aaa_walk_convex_edges branch: partial rows
                    // go through blitAntiH; full rows go through blitAntiRect.
                    let local_y = bounds.y as f32 + (canvas.ty - sy as f32);
                    let local_bottom = local_y + bounds.height as f32;
                    let top =
                        crate::src::core::SkAnalyticEdge::snap_y(((local_y * 256.0) as i32) << 8)
                            .max((c.top as i32) << 16);
                    let bottom = crate::src::core::SkAnalyticEdge::snap_y(
                        ((local_bottom * 256.0) as i32) << 8,
                    )
                    .min((c.bottom as i32) << 16);
                    // Virtual cropping cannot substitute vertical-edge
                    // partial-row coverage for the real global curved edges.
                    // Quarter-row accumulation can yield192 where a single
                    // fixed_to_alpha(3/4) event yields191. Only complete rows
                    // are opaque independently of that event sequence.
                    if top < bottom && top & 65535 == 0 && bottom & 65535 == 0 {
                        let rect = ConvexRect {
                            x: c.left as i32 - 1,
                            y: top >> 16,
                            width: (c.right - c.left) as i32,
                            height: (bottom - top) >> 16,
                            left_alpha: 0,
                            right_alpha: 0,
                        };
                        emit_rect(ox, nx, lx, oy, ny, ly, &rect, 255);
                        continue;
                    }
                }
            }
            let mut transform = canvas;
            transform.tx -= sx as f32;
            transform.ty -= sy as f32;
            let tile = rounded_rect_tile_with_geometry::<ALPHA_ONLY, RECORD_RUNS>(
                bounds,
                radii,
                transform,
                tw,
                th,
                force_rle,
                c,
                defer_snap,
                Some(&global_geometry),
                (sx, sy),
            )?;
            emit(ox, nx, lx, oy, ny, ly, tw, &tile);
        }
    }
    Some(())
}

/// Analytic path stages: conservative convex contours and general multi-contour
/// winding/even-odd fills. Curves requiring clip chopping use the existing backend.
pub(crate) fn path_mask(
    commands: &[crate::compat::commands::PaintPathCommand],
    even_odd: bool,
    canvas: Transform,
    width: u32,
    height: u32,
    clip: Option<&Mask>,
    device_origin: (u32, u32),
    clip_is_aa: bool,
    clip_aa_tiles: Option<&[bool]>,
) -> Option<RoundedCoverage> {
    path_mask_with_clip(
        commands,
        even_odd,
        canvas,
        width,
        height,
        clip,
        device_origin,
        clip_is_aa,
        clip_aa_tiles,
        None,
    )
}

pub(crate) fn path_mask_with_clip(
    commands: &[crate::compat::commands::PaintPathCommand],
    even_odd: bool,
    canvas: Transform,
    width: u32,
    height: u32,
    clip: Option<&Mask>,
    device_origin: (u32, u32),
    clip_is_aa: bool,
    clip_aa_tiles: Option<&[bool]>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
) -> Option<RoundedCoverage> {
    path_mask_with_clip_impl(
        commands,
        even_odd,
        canvas,
        width,
        height,
        clip,
        device_origin,
        clip_is_aa,
        clip_aa_tiles,
        clip_summary,
        true,
    )
}

// Dense-reference output mode shares the unchanged scan converter, while
// retaining its former full-tile copies to validate the bounded target.
fn path_mask_with_clip_impl(
    commands: &[crate::compat::commands::PaintPathCommand],
    even_odd: bool,
    canvas: Transform,
    width: u32,
    height: u32,
    clip: Option<&Mask>,
    device_origin: (u32, u32),
    clip_is_aa: bool,
    clip_aa_tiles: Option<&[bool]>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    compact_output: bool,
) -> Option<RoundedCoverage> {
    match path_coverage_impl(
        commands,
        even_odd,
        canvas,
        width,
        height,
        clip,
        device_origin,
        clip_is_aa,
        clip_aa_tiles,
        clip_summary,
        compact_output,
        false,
    )? {
        PathCoverage::Mask(mask) => Some(mask),
        PathCoverage::Spans(_) => unreachable!("dense target requested"),
    }
}

pub(crate) fn path_spans_with_clip(
    commands: &[crate::compat::commands::PaintPathCommand],
    even_odd: bool,
    canvas: Transform,
    width: u32,
    height: u32,
    clip: Option<&Mask>,
    device_origin: (u32, u32),
    clip_is_aa: bool,
    clip_aa_tiles: Option<&[bool]>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
) -> Option<RoundedSpans> {
    match path_coverage_impl(
        commands,
        even_odd,
        canvas,
        width,
        height,
        clip,
        device_origin,
        clip_is_aa,
        clip_aa_tiles,
        clip_summary,
        true,
        true,
    )? {
        PathCoverage::Spans(spans) => Some(spans),
        PathCoverage::Mask(_) => unreachable!("span target requested"),
    }
}

enum PathCoverage {
    Mask(RoundedCoverage),
    Spans(RoundedSpans),
}

// A local SkBlitter target adapter. The analytic scanner, clip choice,
// tile origins, direct-event order and flush arithmetic are shared with
// the dense mask target; only the receiving storage differs.
struct PathCoverageTarget {
    dense: Option<RoundedCoverage>,
    spans: RoundedSpans,
    width: usize,
}
impl PathCoverageTarget {
    fn direct(&mut self, start: usize, coverage: u8, pair: bool) {
        if let Some(dense) = &mut self.dense {
            dense.blits.push((start, coverage, pair));
        } else if coverage != 0 {
            if let Some(last) = self.spans.direct.last_mut().filter(|last| {
                last.start + last.count == start
                    && last.start / self.width == start / self.width
                    && last.coverage == coverage
                    && last.pair == pair
            }) {
                last.count += 1;
            } else {
                self.spans.direct.push(CoverageSpan {
                    start,
                    count: 1,
                    coverage,
                    pair,
                });
            }
        }
    }
    fn row(&mut self, start: usize, pixels: &[u8], pairs: &[bool]) {
        if let Some(dense) = &mut self.dense {
            let y = (start / self.width) as u32;
            let x = (start % self.width) as u32;
            dense
                .mask
                .row_range_mut(y, x, x + pixels.len() as u32)
                .copy_from_slice(pixels);
            // Encoded clip production consumes coverage + its real receiver,
            // never the color-blitter's H2 flags. Ordinary draw sinks retain
            // the complete pair plane and its exact indexing.
            if !dense.pairs.is_empty() {
                dense.pairs[start..start + pixels.len()].copy_from_slice(pairs);
            }
        } else {
            let mut x = 0;
            while x < pixels.len() {
                let end = coverage_run_end(pixels, pairs, None, x);
                if pixels[x] != 0 {
                    self.spans.accumulated.push(CoverageSpan {
                        start: start + x,
                        count: end - x,
                        coverage: pixels[x],
                        pair: pairs[x],
                    });
                }
                x = end;
            }
        }
    }
    fn finish(self) -> PathCoverage {
        match self.dense {
            Some(mask) => PathCoverage::Mask(mask),
            None => PathCoverage::Spans(self.spans),
        }
    }
}

// Independent AA clip source route. Outer None means an unsupported path,
// not a missing encoding accepted by the caller; completed empty AA is inner
// None. This cannot be published until every fallback supplies its real stream.
pub(crate) fn path_clip_mask_with_encoding(
    commands: &[crate::compat::commands::PaintPathCommand],
    even_odd: bool,
    canvas: Transform,
    width: u32,
    height: u32,
    device_origin: (u32, u32),
    native_clip_bounds: crate::src::core::SkColorGlyphClip::Bounds,
) -> Option<(
    RoundedCoverage,
    Option<crate::src::core::SkColorGlyphClip::Encoding>,
)> {
    let [l, t, r, b] = native_clip_bounds.0;
    let mut summary = crate::src::core::SkRasterClip::SkRasterClip::default();
    summary.setRect(IntRect::from_ltrb(l, t, r, b));
    let mut encoded = None;
    let mask = match path_coverage_encoded_impl(
        commands,
        even_odd,
        canvas,
        width,
        height,
        None,
        device_origin,
        true,
        None,
        Some(&summary),
        true,
        false,
        Some(&mut encoded),
    )? {
        PathCoverage::Mask(mask) => mask,
        PathCoverage::Spans(_) => unreachable!("dense AA clip requested"),
    };
    Some((mask, encoded))
}

fn path_coverage_impl(
    commands: &[crate::compat::commands::PaintPathCommand],
    even_odd: bool,
    canvas: Transform,
    width: u32,
    height: u32,
    clip: Option<&Mask>,
    device_origin: (u32, u32),
    clip_is_aa: bool,
    _clip_aa_tiles: Option<&[bool]>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    compact_output: bool,
    span_output: bool,
) -> Option<PathCoverage> {
    path_coverage_encoded_impl(
        commands,
        even_odd,
        canvas,
        width,
        height,
        clip,
        device_origin,
        clip_is_aa,
        _clip_aa_tiles,
        clip_summary,
        compact_output,
        span_output,
        None,
    )
}

fn path_coverage_encoded_impl(
    commands: &[crate::compat::commands::PaintPathCommand],
    even_odd: bool,
    canvas: Transform,
    width: u32,
    height: u32,
    clip: Option<&Mask>,
    device_origin: (u32, u32),
    clip_is_aa: bool,
    _clip_aa_tiles: Option<&[bool]>,
    clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
    compact_output: bool,
    span_output: bool,
    clip_encoding: Option<&mut Option<crate::src::core::SkColorGlyphClip::Encoding>>,
) -> Option<PathCoverage> {
    use crate::compat::commands::PaintPathVerb as V;
    let device_len = (width as usize).checked_mul(height as usize)?;
    if device_len > isize::MAX as usize {
        return None;
    }
    if commands.first()?.verb != V::kMoveTo {
        return None;
    }
    let mut polygon = Vec::new();
    let single_contour = commands.iter().filter(|c| c.verb == V::kMoveTo).count() == 1;
    for c in commands {
        if c.verb == V::kConicTo && (!(c.conic_weight as f32).is_finite() || c.conic_weight < 0.0) {
            return None;
        }
        if matches!(c.verb, V::kQuadraticTo | V::kConicTo | V::kCubicTo) {
            polygon.push((c.control1.x as f32, c.control1.y as f32));
        }
        if c.verb == V::kCubicTo {
            polygon.push((c.control2.x as f32, c.control2.y as f32));
        }
        if c.verb != V::kClose {
            polygon.push((c.point.x as f32, c.point.y as f32));
        }
    }
    polygon.dedup();
    if polygon.first() == polygon.last() {
        polygon.pop();
    }
    if polygon.len() < 3
        || !canvas.tx.is_finite()
        || !canvas.ty.is_finite()
        || polygon.iter().any(|p| {
            !p.0.is_finite()
                || !p.1.is_finite()
                || (clip_encoding.is_none()
                    && ((p.0 + canvas.tx).abs() > 4096.0 || (p.1 + canvas.ty).abs() > 4096.0))
        })
    {
        return None;
    }
    let is_rect = single_contour
        && polygon.len() == 4
        && commands
            .iter()
            .all(|c| matches!(c.verb, V::kMoveTo | V::kLineTo | V::kClose))
        && (0..4).all(|i| {
            let a = polygon[i];
            let b = polygon[(i + 1) % 4];
            (a.0 == b.0) != (a.1 == b.1)
        });
    let mut bbox = ClipBox {
        left: 0.0,
        top: 0.0,
        right: width as f32,
        bottom: height as f32,
    };
    if let Some(summary) = clip_summary {
        bbox = summary.getBounds().map_or(
            ClipBox {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            |b| ClipBox {
                left: b.left() as f32,
                top: b.top() as f32,
                right: b.right() as f32,
                bottom: b.bottom() as f32,
            },
        );
    } else if let Some(m) = clip {
        let mut l = width;
        let mut t = height;
        let mut r = 0;
        let mut b = 0;
        let storage = m.storage_bounds();
        for yy in storage.top() as u32..storage.bottom() as u32 {
            for (xx, &a) in m
                .row_range(yy, storage.left() as u32, storage.right() as u32)
                .iter()
                .enumerate()
            {
                let i = yy as usize * width as usize + storage.left() as usize + xx;
                if a != 0 {
                    let x = i as u32 % width;
                    let y = i as u32 / width;
                    l = l.min(x);
                    t = t.min(y);
                    r = r.max(x + 1);
                    b = b.max(y + 1);
                }
            }
        }
        bbox = ClipBox {
            left: l as f32,
            top: t as f32,
            right: r as f32,
            bottom: b as f32,
        };
    }
    let mut result = PathCoverageTarget {
        dense: if span_output {
            None
        } else {
            Some(RoundedCoverage {
                support_bounds: None,
                mask: if compact_output {
                    Mask::new_bounded(width, height, IntRect::from_xywh(0, 0, 1, 1)?)?
                } else {
                    Mask::new(width, height)?
                },
                // The sole encoded-clip caller returns only mask bytes and
                // the captured receiver. Do not clear a device-sized H2 plane
                // that it immediately discards. Scanner-local flags stay intact.
                pairs: if clip_encoding.is_some() {
                    Vec::new()
                } else {
                    vec![false; device_len]
                },
                blits: Vec::new(),
                run_starts: Vec::new(),
                rects: Vec::new(),
                raw_direct: None,
            })
        },
        spans: RoundedSpans::default(),
        width: width as usize,
    };
    // SkScan's bitmap/RLE choice belongs to the actual clip. A virtual
    // tile's AA classification must not choose a different small-path route.
    // Read packed mask rows directly if an older caller supplied a partial
    // clip without its root-AA flag; this never promotes it to a full plane.
    let has_partial_clip = !clip_is_aa
        && !clip_summary.is_some_and(|summary| summary.isRect())
        && clip.is_some_and(|mask| {
            let storage = mask.storage_bounds();
            let top = (bbox.top.max(storage.top() as f32).max(0.0)) as u32;
            let bottom = (bbox
                .bottom
                .min(storage.bottom() as f32)
                .min(height as f32)
                .max(0.0)) as u32;
            let left = (bbox.left.max(storage.left() as f32).max(0.0)) as u32;
            let right = (bbox
                .right
                .min(storage.right() as f32)
                .min(width as f32)
                .max(0.0)) as u32;
            left < right
                && (top..bottom).any(|y| {
                    mask.row_range(y, left, right)
                        .iter()
                        .any(|&alpha| alpha != 0 && alpha != 255)
                })
        });
    // SkDraw transforms one complete device path before the analytic builder.
    // Internal bounded output tiles must not change float curve decomposition,
    // edge clipping, fixed coefficients, or the real blitter's H2 arithmetic.
    let map_xy = |x: f32, y: f32| {
        (
            (x * canvas.sx + y * canvas.kx) + canvas.tx,
            (y * canvas.sy + x * canvas.ky) + canvas.ty,
        )
    };
    let map = |p: crate::compat::commands::Offset| map_xy(p.x as f32, p.y as f32);
    let device_polygon: Vec<_> = polygon.iter().map(|p| map_xy(p.0, p.1)).collect();
    if device_polygon.iter().any(|&(x, y)| {
        !x.is_finite()
            || !y.is_finite()
            || (clip_encoding.is_none() && (x.abs() > 4096.0 || y.abs() > 4096.0))
    }) {
        return None;
    }
    let convex = path_geometry::is_convex(&device_polygon, single_contour);
    let global_left = device_polygon
        .iter()
        .map(|p| p.0)
        .fold(f32::INFINITY, f32::min)
        .floor();
    let global_right = device_polygon
        .iter()
        .map(|p| p.0)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil();
    let global_top = device_polygon
        .iter()
        .map(|p| p.1)
        .fold(f32::INFINITY, f32::min)
        .floor();
    let global_bottom = device_polygon
        .iter()
        .map(|p| p.1)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil();
    if global_left >= bbox.right
        || global_right <= bbox.left
        || global_top >= bbox.bottom
        || global_bottom <= bbox.top
    {
        return Some(result.finish());
    }
    if clip_encoding.is_some() {
        // Original Builder::blitPath installs its snug fBounds as the real
        // scan clip before building edges. Preserve fixed-edge overshoot gates.
        bbox.left = bbox.left.max(global_left);
        bbox.top = bbox.top.max(global_top);
        bbox.right = bbox.right.min(global_right);
        bbox.bottom = bbox.bottom.min(global_bottom);
    }
    let horizontal_clip = global_left < bbox.left || global_right > bbox.right;
    let needs_clip = global_left < bbox.left
        || global_right > bbox.right
        || global_top < bbox.top
        || global_bottom > bbox.bottom;
    let line = |a: Pt, b: Pt, out: &mut Vec<SkAnalyticEdge>| {
        if needs_clip {
            cubic_clip::line([a, b], bbox, !convex, out);
        } else {
            add_line(a, b, out);
        }
    };
    let mut global_edges = Vec::new();
    let mut start = map(commands[0].point);
    let mut prev = start;
    for command in &commands[1..] {
        let p = map(command.point);
        match command.verb {
            V::kMoveTo => {
                line(prev, start, &mut global_edges);
                start = p;
                prev = p;
                continue;
            }
            V::kLineTo => line(prev, p, &mut global_edges),
            V::kQuadraticTo => {
                let ctrl = map(command.control1);
                if needs_clip {
                    clipped_quad_with_culling([prev, ctrl, p], bbox, !convex, &mut global_edges);
                } else {
                    // SkAnalyticEdgeBuilder::addQuad uses monotonic
                    // segments from SkChopQuadAtYExtrema.
                    for segment in chop_quad_axis([prev, ctrl, p], 1) {
                        add_quad(segment, &mut global_edges);
                    }
                }
            }
            V::kConicTo => {
                // SkAnalyticEdgeBuilder converts the device-space
                // rational conic to quads at 0.25px tolerance before
                // extrema chopping and edge clipping.
                for q in conic_to_quads(
                    [prev, map(command.control1), p],
                    command.conic_weight as f32,
                ) {
                    if needs_clip {
                        clipped_quad_with_culling(q, bbox, !convex, &mut global_edges);
                    } else {
                        for segment in chop_quad_axis(q, 1) {
                            add_quad(segment, &mut global_edges);
                        }
                    }
                }
            }
            V::kCubicTo => {
                let a = map(command.control1);
                let b = map(command.control2);
                if needs_clip {
                    cubic_clip::clipped([prev, a, b, p], bbox, !convex, &mut global_edges);
                } else {
                    for segment in crate::src::core::SkGeometry::chop_y_extrema([prev, a, b, p]) {
                        if let Some(e) = SkAnalyticEdge::setCubic(segment) {
                            global_edges.push(e);
                        }
                    }
                }
            }
            V::kClose => {
                line(prev, start, &mut global_edges);
                prev = start;
                continue;
            }
        }
        prev = p;
    }
    line(prev, start, &mut global_edges);
    let points: usize = commands
        .iter()
        .map(|c| match c.verb {
            V::kMoveTo | V::kLineTo => 1,
            V::kQuadraticTo | V::kConicTo => 2,
            V::kCubicTo => 3,
            V::kClose => 0,
        })
        .sum();
    // Match the actual path scan range for Skia's intersection heuristic.
    let scan_height = (global_bottom.min(bbox.bottom) - global_top.max(bbox.top)).max(0.0);
    let skip_intersect = points > scan_height as usize * 2;
    let rect_geometry = if is_rect && canvas.kx == 0.0 && canvas.ky == 0.0 {
        let x = device_polygon
            .iter()
            .map(|p| p.0)
            .fold(f32::INFINITY, f32::min);
        let y = device_polygon
            .iter()
            .map(|p| p.1)
            .fold(f32::INFINITY, f32::min);
        let right = device_polygon
            .iter()
            .map(|p| p.0)
            .fold(f32::NEG_INFINITY, f32::max);
        let bottom = device_polygon
            .iter()
            .map(|p| p.1)
            .fold(f32::NEG_INFINITY, f32::max);
        let rect = PaintRect {
            x: f64::from(x),
            y: f64::from(y),
            width: f64::from(right - x),
            height: f64::from(bottom - y),
        };
        Some((
            rect,
            RoundedGeometry::new(
                rect,
                PaintCornerRadii::default(),
                Transform::identity(),
                bbox,
            ),
        ))
    } else {
        None
    };
    // Match SkMask's bounded A8 address space without a device-sized plane.
    // This is fresh per-draw scratch; the complete original scan runs once.
    // A one-pixel conservative margin covers analytic fixed-point edge drift.
    let left = (global_left - 1.0)
        .max(bbox.left)
        .max(device_origin.0 as f32)
        .max(0.0) as u32;
    let top = (global_top - 1.0)
        .max(bbox.top)
        .max(device_origin.1 as f32)
        .max(0.0) as u32;
    let right = (global_right + 1.0)
        .min(bbox.right)
        .min(width as f32)
        .max(0.0) as u32;
    let bottom = (global_bottom + 1.0)
        .min(bbox.bottom)
        .min(height as f32)
        .max(0.0) as u32;
    if left >= right || top >= bottom {
        return Some(result.finish());
    }
    let support = IntRect::from_ltrb(left as i32, top as i32, right as i32, bottom as i32)?;
    let (scratch_width, scratch_height) = (right - left, bottom - top);
    if let Some(dense) = &mut result.dense {
        dense.support_bounds = Some(support);
        if compact_output {
            dense.mask = Mask::new_bounded(width, height, support)?;
        }
    }
    let shift_x = left as f32;
    let shift_y = top as f32;
    let actual_clip = ClipBox {
        left: bbox.left - shift_x,
        top: bbox.top - shift_y,
        right: bbox.right - shift_x,
        bottom: bbox.bottom - shift_y,
    };
    // The actual clip's scan mode applies to this one original device walk.
    // Clip metadata is only consulted; its external storage/indexing is unchanged.
    let force_rle = clip_is_aa || has_partial_clip;
    let bw = (global_right - global_left) as u32;
    let bh = (global_bottom - global_top) as u32;
    let small = !force_rle && bw <= 32 && bw.div_ceil(4) * 4 * bh <= 1024;
    if clip_encoding.is_none() {
        if let Some((rect, geometry)) = &rect_geometry {
            let coverage = rounded_rect_tile_with_geometry::<false, true>(
                *rect,
                PaintCornerRadii::default(),
                Transform::from_translate(-shift_x, -shift_y),
                scratch_width,
                scratch_height,
                force_rle,
                actual_clip,
                false,
                Some(geometry),
                (left, top),
            )?;
            for &(i, alpha, pair) in &coverage.blits {
                let x = i as u32 % scratch_width;
                let y = i as u32 / scratch_width;
                result.direct(
                    (top + y) as usize * width as usize + left as usize + x as usize,
                    alpha,
                    pair,
                );
            }
            for y in 0..scratch_height {
                let start = (y * scratch_width) as usize;
                let end = start + scratch_width as usize;
                result.row(
                    (top + y) as usize * width as usize + left as usize,
                    &coverage.mask.data()[start..end],
                    &coverage.pairs[start..end],
                );
            }
            return Some(result.finish());
        }
    }
    let offset_x = -((left as i32) << 16);
    let offset_y = -((top as i32) << 16);
    for edge in &mut global_edges {
        edge.translate_fixed(offset_x, offset_y);
    }
    let scan = ClipBox {
        left: if small {
            actual_clip.left.max(global_left - shift_x)
        } else {
            actual_clip.left
        },
        right: if small {
            actual_clip.right.min(global_right - shift_x)
        } else {
            actual_clip.right
        },
        top: (global_top - shift_y).max(actual_clip.top),
        bottom: (global_bottom - shift_y).min(actual_clip.bottom),
    };
    let mut coverage = if clip_encoding.is_some() {
        let bounds = crate::src::core::SkColorGlyphClip::Bounds([
            bbox.left as i32 - left as i32,
            bbox.top as i32 - top as i32,
            bbox.right as i32 - left as i32,
            bbox.bottom as i32 - top as i32,
        ]);
        let stream = crate::src::core::SkColorGlyphClip::MetadataDispatch::new(bounds, !convex);
        if convex {
            crate::src::core::SkScan_AAAPath::aaa_walk_convex_edges_with_encoding(
                global_edges,
                scratch_width,
                scratch_height,
                scan,
                stream,
            )
        } else {
            crate::src::core::SkScan_AAAPath::aaa_walk_edges_with_encoding(
                global_edges,
                scratch_width,
                scratch_height,
                scan,
                even_odd,
                skip_intersect,
                stream,
            )
        }
    } else if convex {
        aaa_walk_convex_edges(global_edges, scratch_width, scratch_height, small, scan)
    } else {
        aaa_walk_edges(
            global_edges,
            scratch_width,
            scratch_height,
            small,
            scan,
            even_odd,
            force_rle,
            skip_intersect,
        )
    };
    if let Some(out) = clip_encoding {
        // RunBasedAdditiveBlitter destructor flush precedes Builder.finish.
        // finish() copies owned rows; no borrowed alpha-run buffers escape.
        *out = coverage
            .color_clip_stream
            .take()
            .and_then(|stream| stream.finish())
            .and_then(|encoding| encoding.translated(left as i32, top as i32));
    }
    // Only an actual SkRectClipBlitter changes H2 into generic AntiH. Packed
    // output ownership never changes a surviving pixel's color arithmetic.
    if horizontal_clip {
        clear_bool_flags(&mut coverage.pairs);
        for blit in &mut coverage.blits {
            blit.2 = false;
        }
    }
    if !small {
        snap_alpha_runs(&mut coverage.pixels, &coverage.direct);
    }
    for &(i, alpha, pair) in &coverage.blits {
        let x = i as u32 % scratch_width;
        let y = i as u32 / scratch_width;
        result.direct(
            (top + y) as usize * width as usize + left as usize + x as usize,
            alpha,
            pair,
        );
    }
    for y in 0..scratch_height {
        let start = (y * scratch_width) as usize;
        let end = start + scratch_width as usize;
        result.row(
            (top + y) as usize * width as usize + left as usize,
            &coverage.pixels[start..end],
            if coverage.pairs.is_empty() {
                &[]
            } else {
                &coverage.pairs[start..end]
            },
        );
    }
    Some(result.finish())
}

fn clear_bool_flags(flags: &mut [bool]) {
    // SAFETY: false is represented by an all-zero byte; every initialized
    // bool in this exclusive slice is replaced with that valid value.
    unsafe {
        core::ptr::write_bytes(flags.as_mut_ptr(), 0, flags.len());
    }
}

fn tile_number(origin: u32) -> usize {
    if origin == 0 {
        0
    } else {
        (1 + (origin - 255) / 254) as usize
    }
}
fn tile_count(size: u32) -> usize {
    if size <= 255 {
        1
    } else {
        (1 + (size - 255).div_ceil(254)) as usize
    }
}
/// SkRasterClip::updateCacheAndReturnResult runs on each raster device.
/// Identical opaque rows become a BW clip. Keep equal adjacent AA runs:
/// SkAAClip::Builder compresses rows by their run structure, not coverage bytes.
pub(crate) fn clip_aa_tiles(
    mask: &Mask,
    run_starts: &[bool],
    device_origin: (u32, u32),
) -> Vec<bool> {
    clip_aa_tiles_with_bounds(
        mask,
        run_starts,
        device_origin,
        crate::raster::IntRect::from_xywh(0, 0, mask.width(), mask.height()),
    )
}

/// Same device tile indices and AA/RLE rules as clip_aa_tiles. The support
/// rectangle is the exact nonzero bounds supplied by SkRasterClip metadata;
/// tiles outside it are empty BW clips, and zero margins need no pixel scan.
pub(crate) fn clip_aa_tiles_with_bounds(
    mask: &Mask,
    run_starts: &[bool],
    device_origin: (u32, u32),
    support: Option<crate::raster::IntRect>,
) -> Vec<bool> {
    clip_aa_tiles_with_run_eq(mask, device_origin, support, |a, b, len| {
        run_starts[a..a + len] == run_starts[b..b + len]
    })
}

pub(crate) fn clip_aa_tiles_with_sparse_runs(
    mask: &Mask,
    run_starts: &[usize],
    device_origin: (u32, u32),
    support: Option<crate::raster::IntRect>,
) -> Vec<bool> {
    let mut ranges = SparseClipRunRanges::default();
    // Retain the original diagnostic driver for isolated same-binary A/B
    // measurements while unrelated raster stages may be edited elsewhere.
    #[cfg(feature = "profiling")]
    let binary_search = std::env::var_os("SKIA_CLIP_BINARY_SEARCH").is_some();
    clip_aa_tiles_with_run_eq(mask, device_origin, support, |a, b, len| {
        #[cfg(feature = "profiling")]
        if binary_search {
            let a0 = run_starts.partition_point(|&i| i < a);
            let a1 = run_starts.partition_point(|&i| i < a + len);
            let b0 = run_starts.partition_point(|&i| i < b);
            let b1 = run_starts.partition_point(|&i| i < b + len);
            if a1 - a0 != b1 - b0 {
                return false;
            }
            for j in 0..a1 - a0 {
                if run_starts[a0 + j] - a != run_starts[b0 + j] - b {
                    return false;
                }
            }
            return true;
        }
        let (a0, a1, b0, b1) = ranges.find(run_starts, a, b, len);
        if a1 - a0 != b1 - b0 {
            return false;
        }
        for j in 0..a1 - a0 {
            if run_starts[a0 + j] - a != run_starts[b0 + j] - b {
                return false;
            }
        }
        true
    })
}

// Local sorted sparse-run driver for SkAAClip's row-structure comparison.
// Within a tile, queried rows advance monotonically and the reference row is
// unchanged. Traverse those restart indices once instead of doing four global
// binary searches per row. Tile changes may move backwards: retain the exact
// lower-bound lookup in that case. Coverage and equal adjacent AA boundaries
// remain independent; this does not merge, omit or synthesize runs.
#[derive(Default)]
struct SparseClipRunRanges {
    last_start: usize,
    row_begin: usize,
    reference: Option<(usize, usize, usize, usize)>,
}
impl SparseClipRunRanges {
    fn find(
        &mut self,
        runs: &[usize],
        a: usize,
        b: usize,
        len: usize,
    ) -> (usize, usize, usize, usize) {
        let mut a0 = if a >= self.last_start {
            self.row_begin
        } else {
            runs.partition_point(|&i| i < a)
        };
        while a0 < runs.len() && runs[a0] < a {
            a0 += 1;
        }
        let mut a1 = a0;
        while a1 < runs.len() && runs[a1] < a + len {
            a1 += 1;
        }
        self.last_start = a;
        self.row_begin = a0;
        let (b0, b1) = if let Some((start, count, first, last)) = self
            .reference
            .filter(|(start, count, _, _)| *start == b && *count == len)
        {
            debug_assert_eq!((start, count), (b, len));
            (first, last)
        } else {
            let first = runs.partition_point(|&i| i < b);
            let last = runs.partition_point(|&i| i < b + len);
            self.reference = Some((b, len, first, last));
            (first, last)
        };
        (a0, a1, b0, b1)
    }
}

fn clip_aa_tiles_with_run_eq(
    mask: &Mask,
    device_origin: (u32, u32),
    support: Option<crate::raster::IntRect>,
    mut same_runs: impl FnMut(usize, usize, usize) -> bool,
) -> Vec<bool> {
    let width = mask.width();
    let height = mask.height();
    let mut modes = Vec::new();
    let mut oy = 0;
    while oy < height {
        let sy = oy - u32::from(oy != 0);
        let ey = (oy + 255 - u32::from(oy != 0) + 1).min(height);
        let mut ox = 0;
        while ox < width {
            let sx = ox - u32::from(ox != 0);
            let ex = (ox + 255 - u32::from(ox != 0) + 1).min(width);
            let (mut l, mut t, mut r, mut b) = (ex, ey, sx, sy);
            let mut aa = false;
            let active = support.and_then(|bounds| {
                crate::raster::IntRect::from_xywh(sx as i32, sy as i32, ex - sx, ey - sy)
                    .and_then(|tile| bounds.intersect(&tile))
            });
            let Some(active) = active else {
                modes.push(false);
                ox += if ox == 0 { 255 } else { 254 };
                continue;
            };
            'classify: for y in active.top() as u32..active.bottom() as u32 {
                let row = mask.row_range(y, active.left() as u32, active.right() as u32);
                let mut x = 0;
                while x < row.len() {
                    let alpha = row[x];
                    let end = equal_byte_run_end(row, x, alpha);
                    if alpha != 0 {
                        // A partial-alpha byte is conclusive. Likewise the
                        // source's relative-Y vs absolute-bottom isRect rule
                        // rejects a clip whose first live row is below this
                        // tile/device origin. Neither needs a remaining scan.
                        if alpha != 255 || (t == ey && y != sy.max(device_origin.1)) {
                            aa = true;
                            break 'classify;
                        }
                        l = l.min(active.left() as u32 + x as u32);
                        t = t.min(y);
                        r = r.max(active.left() as u32 + end as u32);
                        b = y + 1;
                    }
                    x = end;
                }
            }
            if !aa && l < r && t < b {
                let reference_start = (t * width + l + 1) as usize;
                let reference_end = (t * width + r) as usize;
                for y in t..b {
                    let start = (y * width + l) as usize;
                    let row = mask.row_range(y, l, r);
                    if equal_byte_run_end(row, 0, 255) != row.len()
                        || (y > t
                            && !same_runs(
                                start + 1,
                                reference_start,
                                reference_end - reference_start,
                            ))
                    {
                        aa = true;
                        break;
                    }
                }
            }
            modes.push(aa);
            ox += if ox == 0 { 255 } else { 254 };
        }
        oy += if oy == 0 { 255 } else { 254 };
    }
    modes
}

#[cfg(test)]
mod rounded_span_tests {
    use super::*;
    use crate::src::core::SkRasterClip::SkRasterClip;

    #[test]
    fn virtual_output_crop_preserves_each_h2_pixel_and_bounded_storage() {
        use crate::compat::commands::PaintCornerRadius as Radius;
        let bounds = PaintRect {
            x: 160.0,
            y: 18.0,
            width: 170.0,
            height: 80.0,
        };
        let radius = Radius { x: 40.0, y: 40.0 };
        let radii = PaintCornerRadii {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        };
        let clip = ClipBox {
            left: 0.0,
            top: 0.0,
            right: 512.0,
            bottom: 120.0,
        };
        let geometry = RoundedGeometry::new(bounds, radii, Transform::identity(), clip);
        let whole = rounded_rect_tile_with_geometry::<false, true>(
            bounds,
            radii,
            Transform::identity(),
            512,
            120,
            true,
            clip,
            false,
            Some(&geometry),
            (0, 0),
        )
        .unwrap();
        let pair = whole
            .pairs
            .iter()
            .enumerate()
            .find(|&(i, p)| *p && whole.mask.data()[i] > 8 && whole.mask.data()[i] < 247)
            .unwrap()
            .0;
        let px = pair % 512;
        let py = pair / 512;
        assert!(px > 0);
        // Put that exact H2 pixel at the first and last accessible column.
        // The other member lies outside the sink in one of these cases.
        for (offset, width, local_x) in [(px as u32, 1, 0), (px as u32 - 1, 2, 1)] {
            let transform = Transform::from_translate(-(offset as f32), 0.0);
            let tile_clip = ClipBox {
                left: 0.0,
                top: 0.0,
                right: width as f32,
                bottom: 120.0,
            };
            let tile = rounded_rect_tile_with_geometry::<false, true>(
                bounds,
                radii,
                transform,
                width,
                120,
                true,
                tile_clip,
                false,
                Some(&geometry),
                (offset, 0),
            )
            .unwrap();
            assert_eq!(tile.mask.data().len(), width as usize * 120);
            assert_eq!(tile.pairs.len(), width as usize * 120);
            let i = py * width as usize + local_x;
            assert_eq!(tile.mask.data()[i], whole.mask.data()[pair]);
            assert!(
                tile.pairs[i],
                "a virtual crop must keep the surviving pixel's H2 arithmetic"
            );
            assert!(tile.blits.iter().all(|&(i, _, _)| i < width as usize * 120));
        }
    }
    #[test]
    fn sparse_clip_cursor_ranges_match_binary_search_with_backward_queries() {
        for runs in [
            vec![],
            vec![0, 0, 1, 254, 255, 255, 256, 511, 1023, 1024],
            (0..4096).filter(|i| i % 31 == 0 || i % 255 == 0).collect(),
        ] {
            let mut cursor = SparseClipRunRanges::default();
            for a in (0..4200).step_by(17).chain((0..4200).rev().step_by(13)) {
                for len in [0, 1, 2, 254, 255, 256, 511] {
                    for b in [0, 255, 1024, a, a / 2] {
                        let expected = (
                            runs.partition_point(|&i| i < a),
                            runs.partition_point(|&i| i < a + len),
                            runs.partition_point(|&i| i < b),
                            runs.partition_point(|&i| i < b + len),
                        );
                        assert_eq!(
                            cursor.find(&runs, a, b, len),
                            expected,
                            "a={a} b={b} len={len}"
                        );
                        assert_eq!(cursor.find(&runs, a, b, len), expected, "cached reference");
                    }
                }
            }
        }
    }

    #[test]
    fn alpha_word_scan_matches_scalar_at_every_byte_and_tail() {
        for value in 0..=255u8 {
            for len in [0usize, 1, 2, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65] {
                for offset in [0usize, 1, 7] {
                    let mut bytes = vec![value.wrapping_add(1); offset + len + 8];
                    bytes[offset..offset + len].fill(value);
                    for mismatch in 0..=len {
                        if mismatch < len {
                            bytes[offset + mismatch] = value.wrapping_add(1);
                        }
                        for start in [0usize, len / 2, len] {
                            let row = &bytes[offset..offset + len];
                            let expected = start
                                + row[start..]
                                    .iter()
                                    .position(|&v| v != value)
                                    .unwrap_or(len - start);
                            assert_eq!(equal_byte_run_end(row, start, value), expected,
                                "value={value} len={len} offset={offset} mismatch={mismatch} start={start}");
                        }
                        if mismatch < len {
                            bytes[offset + mismatch] = value;
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn sparse_clip_run_indices_preserve_dense_tile_classification() {
        for (width, height) in [(17, 13), (520, 360)] {
            for seed in 0..16usize {
                let mut mask = Mask::new(width, height).unwrap();
                let mut flags = vec![false; (width * height) as usize];
                let left = seed % 7;
                let top = seed % 5;
                let right = width as usize - seed % 3;
                let bottom = height as usize - seed % 4;
                for y in top..bottom {
                    let start = y * width as usize + left;
                    let end = y * width as usize + right;
                    mask.data_mut()[start..end].fill(255);
                    for x in left..right {
                        flags[y * width as usize + x] = (x + (y % (seed + 1))) % 11 == 0;
                    }
                }
                if seed % 3 == 0 {
                    mask.data_mut()[(top + 1) * width as usize + left + 1] = 127;
                }
                let indices: Vec<usize> = flags
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &v)| v.then_some(i))
                    .collect();
                let support = SkRasterClip::from_mask(width, height, mask.data())
                    .unwrap()
                    .getBounds();
                for origin in [(0, 0), (left as u32, top as u32), (254, 254), (511, 1024)] {
                    assert_eq!(
                        clip_aa_tiles_with_sparse_runs(&mask, &indices, origin, support),
                        clip_aa_tiles_with_bounds(&mask, &flags, origin, support),
                        "size={width}x{height} seed={seed} origin={origin:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn sparse_intervals_preserve_dense_coverage_pairs_and_direct_event_order() {
        let (width, height) = (520, 360);
        let r = crate::compat::commands::PaintCornerRadius { x: 4.0, y: 6.0 };
        let radii = PaintCornerRadii {
            top_left: r,
            top_right: r,
            bottom_left: r,
            bottom_right: r,
        };
        for (x, y) in [
            (-8.5, 5.125),
            (252.25, 249.5),
            (508.0, 252.875),
            (12.125, 9.75),
        ] {
            for (w, h) in [(12.0, 17.0), (160.0, 44.0)] {
                for scale in [1.0, 1.5] {
                    for clipped in [false, true] {
                        let mut clip = Mask::new(width, height).unwrap();
                        for row in 7..340 {
                            clip.data_mut()
                                [(row * width + 9) as usize..(row * width + 510) as usize]
                                .fill(255);
                        }
                        // Irregular holes and partial alpha exercise conservative
                        // bounds instead of the rectangular fast dispatch.
                        clip.data_mut()[(20 * width + 17) as usize] = 0;
                        clip.data_mut()[(21 * width + 18) as usize] = 127;
                        let clip = clipped.then_some(&clip);
                        let summary =
                            clip.map(|m| SkRasterClip::from_mask(width, height, m.data()).unwrap());
                        let bounds = PaintRect {
                            x,
                            y,
                            width: w,
                            height: h,
                        };
                        let transform = Transform::from_scale(scale, scale);
                        let dense = rounded_rect_mask_with_clip(
                            bounds,
                            radii,
                            transform,
                            width,
                            height,
                            false,
                            clip,
                            summary.as_ref(),
                        )
                        .unwrap();
                        let sparse = rounded_rect_spans(
                            bounds,
                            radii,
                            transform,
                            width,
                            height,
                            clip,
                            summary.as_ref(),
                        )
                        .unwrap();
                        let mut values = vec![0; (width * height) as usize];
                        let mut pairs = vec![false; values.len()];
                        for span in sparse.accumulated {
                            assert_eq!(
                                span.start / width as usize,
                                (span.start + span.count - 1) / width as usize
                            );
                            values[span.start..span.start + span.count].fill(span.coverage);
                            pairs[span.start..span.start + span.count].fill(span.pair);
                        }
                        assert_eq!(values, dense.mask.data());
                        for i in 0..values.len() {
                            if values[i] != 0 {
                                assert_eq!(pairs[i], dense.pairs[i]);
                            }
                        }
                        let direct: Vec<_> = sparse
                            .direct
                            .into_iter()
                            .flat_map(|s| {
                                (s.start..s.start + s.count).map(move |i| (i, s.coverage, s.pair))
                            })
                            .collect();
                        let old: Vec<_> = dense
                            .blits
                            .into_iter()
                            .filter(|&(_, a, _)| a != 0)
                            .collect();
                        assert_eq!(
                            direct, old,
                            "x={x},y={y},size={w}x{h},scale={scale},clip={clipped}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn compact_rrect_tiles_preserve_previous_full_tile_planes_and_events() {
        use crate::compat::commands::PaintCornerRadius as Radius;
        let (width, height) = (520, 360);
        let radii = PaintCornerRadii {
            top_left: Radius { x: 4.0, y: 6.0 },
            top_right: Radius { x: 13.5, y: 3.25 },
            bottom_right: Radius { x: 0.0, y: 0.0 },
            bottom_left: Radius { x: 8.5, y: 17.0 },
        };
        let mut bw = Mask::new(width, height).unwrap();
        for y in 7..340 {
            bw.data_mut()[(y * width + 9) as usize..(y * width + 510) as usize].fill(255);
        }
        let mut aa = bw.clone();
        for y in 7..340 {
            for x in 9..510 {
                if (x + y) % 23 == 0 {
                    aa.data_mut()[(y * width + x) as usize] = if y % 2 == 0 { 0 } else { 127 };
                }
            }
        }
        let empty = Mask::new(width, height).unwrap();
        for (x, y) in [
            (-8.5, 5.125),
            (252.25, 249.5),
            (508.0, 252.875),
            (12.125, 9.75),
        ] {
            for (w, h) in [(12.0, 17.0), (160.0, 44.0), (505.0, 277.0)] {
                for scale in [0.5, 1.0, 1.5, 2.0] {
                    let mut transform = Transform::from_scale(scale, scale);
                    transform.tx = 0.125;
                    transform.ty = 0.375;
                    for clip in [None, Some(&bw), Some(&aa), Some(&empty)] {
                        let summary = clip.map(|mask| {
                            SkRasterClip::from_mask(width, height, mask.data()).unwrap()
                        });
                        let bounds = PaintRect {
                            x,
                            y,
                            width: w,
                            height: h,
                        };
                        let old = rounded_rect_mask_with_clip_impl(
                            bounds,
                            radii,
                            transform,
                            width,
                            height,
                            false,
                            clip,
                            summary.as_ref(),
                            false,
                        )
                        .unwrap();
                        let new = rounded_rect_mask_with_clip_impl(
                            bounds,
                            radii,
                            transform,
                            width,
                            height,
                            false,
                            clip,
                            summary.as_ref(),
                            true,
                        )
                        .unwrap();
                        let differences = new
                            .mask
                            .data()
                            .iter()
                            .zip(old.mask.data())
                            .enumerate()
                            .filter(|(_, (a, b))| a != b)
                            .take(20)
                            .map(|(i, (a, b))| (i % width as usize, i / width as usize, *a, *b))
                            .collect::<Vec<_>>();
                        assert!(
                            differences.is_empty(),
                            "x={x},y={y},size={w}x{h},scale={scale},clip={},diff={differences:?}",
                            clip.is_some()
                        );
                        assert_eq!(new.pairs, old.pairs);
                        assert_eq!(new.blits, old.blits);
                        assert_eq!(new.run_starts, old.run_starts);
                    }
                }
            }
        }
    }

    #[test]
    fn straight_rrect_replay_preserves_three_formats_and_complex_aa_clips() {
        use crate::compat::commands::PaintCornerRadius as Radius;
        use crate::raster::PixelFormat;
        let (width, height) = (520, 360);
        let radii = PaintCornerRadii {
            top_left: Radius { x: 16.0, y: 16.0 },
            top_right: Radius { x: 9.5, y: 17.25 },
            bottom_right: Radius { x: 3.25, y: 8.5 },
            bottom_left: Radius { x: 13.5, y: 4.0 },
        };
        let mut clip = Mask::new(width, height).unwrap();
        for y in 0..height {
            for x in 9..510 {
                clip.data_mut()[(y * width + x) as usize] = match (x + y) % 7 {
                    0 => 0,
                    1 => 1,
                    2 => 127,
                    3 => 254,
                    _ => 255,
                };
            }
        }
        let summary = SkRasterClip::from_mask(width, height, clip.data()).unwrap();
        for bounds in [
            PaintRect {
                x: 32.5,
                y: -31.5,
                width: 1215.0,
                height: 276.0,
            },
            PaintRect {
                x: -3.125,
                y: -100.25,
                width: 270.875,
                height: 800.125,
            },
            PaintRect {
                x: 253.25,
                y: -80.75,
                width: 260.5,
                height: 700.125,
            },
            PaintRect {
                x: 12.125,
                y: -40.75,
                width: 50.5,
                height: 800.25,
            },
        ] {
            for scale in [1.0, 2.0] {
                let transform = Transform::from_scale(scale, scale);
                for clipped in [false, true] {
                    let clip = clipped.then_some(&clip);
                    let summary = clipped.then_some(&summary);
                    let dense = rounded_rect_mask_with_clip_impl(
                        bounds, radii, transform, width, height, false, clip, summary, false,
                    )
                    .unwrap();
                    let sparse =
                        rounded_rect_spans(bounds, radii, transform, width, height, clip, summary)
                            .unwrap();
                    let mut reconstructed = vec![0; (width * height) as usize];
                    let mut pairs = vec![false; reconstructed.len()];
                    for span in &sparse.accumulated {
                        reconstructed[span.start..span.start + span.count].fill(span.coverage);
                        pairs[span.start..span.start + span.count].fill(span.pair);
                    }
                    let diff = reconstructed
                        .iter()
                        .zip(dense.mask.data())
                        .enumerate()
                        .find(|(_, (a, b))| a != b)
                        .map(|(i, (a, b))| (i % width as usize, i / width as usize, *a, *b));
                    assert!(
                        diff.is_none(),
                        "bounds={bounds:?},scale={scale},clip={clipped},diff={diff:?}"
                    );
                    for i in 0..pairs.len() {
                        if reconstructed[i] != 0 {
                            assert_eq!(pairs[i], dense.pairs[i]);
                        }
                    }
                    for format in [
                        PixelFormat::Rgba8888,
                        PixelFormat::Bgra8888,
                        PixelFormat::Bgrx8888,
                    ] {
                        for source in [[255, 255, 255, 255], [63, 37, 24, 127]] {
                            let mut old = Vec::with_capacity(reconstructed.len() * 4);
                            for i in 0..reconstructed.len() {
                                old.extend_from_slice(&format.encode([
                                    i as u8 / 2,
                                    (i / 17) as u8 / 2,
                                    (i / 37) as u8 / 2,
                                    255,
                                ]));
                            }
                            let mut new = old.clone();
                            let apply = |bytes: &mut [u8],
                                         start: usize,
                                         count: usize,
                                         alpha: u8,
                                         pair: bool| {
                                if let Some(clip) = clip {
                                    for i in start..start + count {
                                        let a = crate::cpu::mask_blitter::mul_div_255_round(
                                            alpha,
                                            clip.data()[i],
                                        );
                                        crate::src::core::SkBlitRow_D32::blend_span_format(
                                            &mut bytes[i * 4..i * 4 + 4],
                                            source,
                                            a,
                                            false,
                                            format,
                                        );
                                    }
                                } else {
                                    crate::src::core::SkBlitRow_D32::blend_span_format(
                                        &mut bytes[start * 4..(start + count) * 4],
                                        source,
                                        alpha,
                                        pair,
                                        format,
                                    );
                                }
                            };
                            for &(i, a, pair) in &dense.blits {
                                apply(&mut old, i, 1, a, pair);
                            }
                            for (i, &a) in dense.mask.data().iter().enumerate() {
                                if a != 0 {
                                    apply(&mut old, i, 1, a, dense.pairs[i]);
                                }
                            }
                            for span in sparse.direct.iter().chain(&sparse.accumulated) {
                                apply(&mut new, span.start, span.count, span.coverage, span.pair);
                            }
                            assert!(new == old, "bounds={bounds:?},scale={scale},clip={clipped},format={format:?},source={source:?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn straight_replay_skips_scratch_planes_but_keeps_fat_rect_rounding() {
        use crate::compat::commands::PaintCornerRadius as Radius;
        let r = Radius { x: 16.0, y: 16.0 };
        let radii = PaintCornerRadii {
            top_left: r,
            top_right: r,
            bottom_left: r,
            bottom_right: r,
        };
        let mut rect_count = 0;
        let mut dense_count = 0;
        visit_rounded_rect_tiles(
            PaintRect {
                x: 32.5,
                y: -31.5,
                width: 1215.0,
                height: 276.0,
            },
            radii,
            Transform::from_scale(2.0, 2.0),
            2560,
            1542,
            false,
            None,
            None,
            true,
            true,
            Some(&mut |_, _, _, _, _, _, _, _| rect_count += 1),
            &mut |_, _, _, _, _, _, _, _| dense_count += 1,
        )
        .unwrap();
        assert!(
            rect_count >= 8,
            "central straight tiles must bypass scratch allocation"
        );
        assert!(
            dense_count > 0,
            "curved edge tiles retain the analytic walker"
        );
        let zero = Radius { x: 0.0, y: 0.0 };
        let radii = PaintCornerRadii {
            top_left: zero,
            top_right: zero,
            bottom_left: zero,
            bottom_right: zero,
        };
        let bounds = PaintRect {
            x: 0.5,
            y: -4.0,
            width: 19.0,
            height: 22.0,
        };
        let dense = rounded_rect_mask_with_clip_impl(
            bounds,
            radii,
            Transform::identity(),
            255,
            9,
            false,
            None,
            None,
            false,
        )
        .unwrap();
        let sparse =
            rounded_rect_spans(bounds, radii, Transform::identity(), 255, 9, None, None).unwrap();
        let mut reconstructed = vec![0; 255 * 9];
        for span in sparse.accumulated {
            reconstructed[span.start..span.start + span.count].fill(span.coverage);
        }
        assert_eq!(reconstructed, dense.mask.data());
        assert_eq!(reconstructed[0], 127); // SkBlitter::blitFatAntiRect truncates.
    }

    #[test]
    fn horizontal_central_tiles_preserve_quarter_pixel_y_and_short_rectangles() {
        use crate::compat::commands::PaintCornerRadius as Radius;
        let r = Radius { x: 16.0, y: 16.0 };
        let radii = PaintCornerRadii {
            top_left: r,
            top_right: r,
            bottom_left: r,
            bottom_right: r,
        };
        for y in [-0.125, 0.125, 17.375, 254.875, 255.125] {
            for height in [0.125, 0.375, 0.75, 1.625, 40.125] {
                let bounds = PaintRect {
                    x: -100.25,
                    y,
                    width: 1000.0,
                    height,
                };
                let dense = rounded_rect_mask_with_clip_impl(
                    bounds,
                    radii,
                    Transform::identity(),
                    520,
                    360,
                    false,
                    None,
                    None,
                    false,
                )
                .unwrap();
                let sparse =
                    rounded_rect_spans(bounds, radii, Transform::identity(), 520, 360, None, None)
                        .unwrap();
                let mut coverage = vec![0; 520 * 360];
                let mut pairs = vec![false; coverage.len()];
                assert!(sparse.direct.is_empty());
                for span in sparse.accumulated {
                    coverage[span.start..span.start + span.count].fill(span.coverage);
                    pairs[span.start..span.start + span.count].fill(span.pair);
                }
                let differences = coverage
                    .iter()
                    .zip(dense.mask.data())
                    .enumerate()
                    .filter(|(_, (a, b))| a != b)
                    .take(8)
                    .map(|(i, (a, b))| (i % 520, i / 520, *a, *b))
                    .collect::<Vec<_>>();
                assert!(
                    differences.is_empty(),
                    "y={y},height={height},diff={differences:?}"
                );
                for i in 0..coverage.len() {
                    if coverage[i] != 0 {
                        assert_eq!(pairs[i], dense.pairs[i]);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod run_adapter_tests {
    use super::{coverage_run_end, equal_byte_run_end};

    #[test]
    fn word_run_search_preserves_every_tail_and_flag_boundary() {
        // The former scalar predicate is an independent oracle for all three
        // planes. Move each kind of boundary through every word/tail alignment.
        for length in 1..=73 {
            for start in 0..length {
                for boundary in start..=length {
                    for raw in [0u8, 1, 7, 8, 247, 248, 254, 255] {
                        let mut row = vec![raw; length];
                        let mut pairs = vec![false; length];
                        let mut direct = vec![true; length];
                        for changed_plane in 0..3 {
                            if boundary < length {
                                match changed_plane {
                                    0 => row[boundary] = raw.wrapping_add(1),
                                    1 => pairs[boundary] = true,
                                    _ => direct[boundary] = false,
                                }
                            }
                            for flags in [None, Some(direct.as_slice())] {
                                let current = row[start];
                                let pair = pairs[start];
                                let provenance = flags.is_some_and(|f| f[start]);
                                let mut expected = start + 1;
                                while expected < length
                                    && row[expected] == current
                                    && pairs[expected] == pair
                                    && (current == 255
                                        || flags.is_none_or(|f| f[expected] == provenance))
                                {
                                    expected += 1;
                                }
                                assert_eq!(coverage_run_end(&row, &pairs, flags, start), expected);
                            }
                            let expected_byte = row[start..]
                                .iter()
                                .position(|&v| v != row[start])
                                .map_or(length, |i| start + i);
                            assert_eq!(equal_byte_run_end(&row, start, row[start]), expected_byte);
                            if boundary < length {
                                row[boundary] = raw;
                                pairs[boundary] = false;
                                direct[boundary] = true;
                            }
                        }
                    }
                }
            }
        }
    }
}

// Test oracle for SkAAClip::setPath's actual scan domain. The old dense
// color/provenance walker remains unchanged; it receives the same native
// path.roundOut intersect prior-clip bounds instead of a full-device domain.
#[cfg(test)]
fn source_snug_clip_summary(
    bounds: PaintRect,
    canvas: Transform,
    width: u32,
    height: u32,
    prior: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
) -> crate::src::core::SkRasterClip::SkRasterClip {
    assert!(canvas.kx == 0.0 && canvas.ky == 0.0 && canvas.sx > 0.0 && canvas.sy > 0.0);
    let x = bounds.x as f32 * canvas.sx + canvas.tx;
    let y = bounds.y as f32 * canvas.sy + canvas.ty;
    // Map the path's pre-transform float right/bottom; do not reassociate
    // its arithmetic into mapped-left plus mapped-width.
    let right = (bounds.x as f32 + bounds.width as f32) * canvas.sx + canvas.tx;
    let bottom = (bounds.y as f32 + bounds.height as f32) * canvas.sy + canvas.ty;
    let device = IntRect::from_xywh(0, 0, width, height).unwrap();
    let snug = IntRect::from_ltrb(
        x.floor() as i32,
        y.floor() as i32,
        right.ceil() as i32,
        bottom.ceil() as i32,
    )
    .and_then(|s| s.intersect(&device))
    .and_then(|s| match prior {
        Some(p) => p.getBounds().and_then(|p| s.intersect(&p)),
        None => Some(s),
    });
    let mut summary = crate::src::core::SkRasterClip::SkRasterClip::default();
    summary.setRect(snug);
    summary
}

#[cfg(test)]
mod bounded_clip_tests {
    use super::*;
    use crate::raster::IntRect;
    use crate::src::core::SkRasterClip::SkRasterClip;

    #[test]
    fn clip_only_target_preserves_dense_alpha_and_rle_boundaries() {
        let (width, height) = (520, 360);
        let radius = crate::compat::commands::PaintCornerRadius { x: 11.25, y: 7.5 };
        let radii = PaintCornerRadii {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        };
        let mut bw = Mask::new(width, height).unwrap();
        for y in 11..331 {
            bw.data_mut()[(y * width + 5) as usize..(y * width + 512) as usize].fill(255);
        }
        let mut aa = bw.clone();
        for y in 11..331 {
            for x in 5..512 {
                if (x + y) % 7 == 0 {
                    aa.data_mut()[(y * width + x) as usize] = 127;
                }
            }
        }
        let empty = Mask::new(width, height).unwrap();
        for rect in [
            PaintRect {
                x: -9.125,
                y: 2.75,
                width: 48.25,
                height: 33.5,
            },
            PaintRect {
                x: 253.75,
                y: 252.125,
                width: 55.5,
                height: 19.75,
            },
            PaintRect {
                x: 512.875,
                y: 353.125,
                width: 42.5,
                height: 31.5,
            },
        ] {
            for scale in [0.5, 1.0, 1.5, 2.0] {
                let mut transform = Transform::from_scale(scale, scale);
                transform.tx = 0.375;
                transform.ty = 0.125;
                for clip in [None, Some(&bw), Some(&aa), Some(&empty)] {
                    let summary =
                        clip.map(|m| SkRasterClip::from_mask(width, height, m.data()).unwrap());
                    let dense = rounded_rect_mask_with_clip(
                        rect,
                        radii,
                        transform,
                        width,
                        height,
                        true,
                        clip,
                        summary.as_ref(),
                    )
                    .unwrap();
                    let only = rounded_rect_clip_mask(
                        rect,
                        radii,
                        transform,
                        width,
                        height,
                        clip,
                        summary.as_ref(),
                    )
                    .unwrap();
                    let source_scan =
                        source_snug_clip_summary(rect, transform, width, height, summary.as_ref());
                    let source_dense = rounded_rect_mask_with_clip(
                        rect,
                        radii,
                        transform,
                        width,
                        height,
                        true,
                        clip,
                        Some(&source_scan),
                    )
                    .unwrap();
                    assert_eq!(only.mask.data(), source_dense.mask.data());
                    assert_eq!(only.run_starts, source_dense.run_starts);
                    if rect.x == -9.125 && scale == 0.5 && clip.is_none() {
                        // Independent PUBLIC original SkCanvas clipRRect,
                        // whole 520x360 A8: native240/0, unbounded248/7.
                        // Keep this original failed geometry as a negative
                        // control for the former full-device clip oracle.
                        assert_eq!(dense.mask.data()[5 * 520 + 19], 248);
                        assert_eq!(dense.mask.data()[5 * 520 + 20], 7);
                        assert_eq!(only.mask.data()[5 * 520 + 19], 240);
                        assert_eq!(only.mask.data()[5 * 520 + 20], 0);
                        assert_ne!(only.mask.data(), dense.mask.data());
                    }
                }
            }
        }
    }

    #[test]
    fn bounded_tile_cache_preserves_origin_run_structure_and_empty_tile_indices() {
        let (width, height) = (520, 360);
        for shape in 0..5 {
            let mut mask = Mask::new(width, height).unwrap();
            let mut runs = vec![false; (width * height) as usize];
            for y in 247..280 {
                for x in 253..286 {
                    let i = (y * width + x) as usize;
                    mask.data_mut()[i] = match shape {
                        1 if (x + y) % 7 == 0 => 127,
                        2 if (x + y) % 7 == 0 => 0,
                        4 => 0,
                        _ => 255,
                    };
                    runs[i] = x == 253 || (shape == 3 && x == 260 && y % 2 == 0);
                }
            }
            let summary = SkRasterClip::from_mask(width, height, mask.data()).unwrap();
            for origin in [(0, 0), (17, 247), (200, 260)] {
                assert_eq!(
                    clip_aa_tiles_with_bounds(&mask, &runs, origin, summary.getBounds()),
                    clip_aa_tiles(&mask, &runs, origin),
                );
            }
        }
    }

    #[test]
    #[ignore = "reproducible debug microbenchmark; validates outputs before reporting timings"]
    fn small_clip_metadata_debug_timing() {
        use std::time::Instant;
        let (width, height) = (2560, 1542);
        let bounds = IntRect::from_xywh(1219, 899, 44, 44).unwrap();
        let mut mask = Mask::new(width, height).unwrap();
        let mut runs = vec![false; mask.data().len()];
        for y in 900..941 {
            for x in 1220..1261 {
                let i = (y * width + x) as usize;
                mask.data_mut()[i] = if x == 1220 || x == 1260 { 127 } else { 255 };
                runs[i] = x == 1220 || x == 1240;
            }
        }
        let old_summary = SkRasterClip::from_mask(width, height, mask.data()).unwrap();
        let old_tiles = clip_aa_tiles(&mask, &runs, (0, 0));
        assert_eq!(
            SkRasterClip::from_mask_with_bounds(width, height, mask.data(), Some(bounds)).unwrap(),
            old_summary
        );
        assert_eq!(
            clip_aa_tiles_with_bounds(&mask, &runs, (0, 0), Some(bounds)),
            old_tiles
        );
        let count = 10;
        let now = Instant::now();
        for _ in 0..count {
            std::hint::black_box(SkRasterClip::from_mask(
                width,
                height,
                std::hint::black_box(mask.data()),
            ));
            std::hint::black_box(clip_aa_tiles(&mask, &runs, (0, 0)));
        }
        let full = now.elapsed();
        let now = Instant::now();
        for _ in 0..count {
            std::hint::black_box(SkRasterClip::from_mask_with_bounds(
                width,
                height,
                std::hint::black_box(mask.data()),
                Some(bounds),
            ));
            std::hint::black_box(clip_aa_tiles_with_bounds(
                &mask,
                &runs,
                (0, 0),
                Some(bounds),
            ));
        }
        let bounded = now.elapsed();
        eprintln!(
            "small clip metadata debug: full {:.3} ms/op, bounded {:.3} ms/op; {:.1}x",
            full.as_secs_f64() * 1000.0 / count as f64,
            bounded.as_secs_f64() * 1000.0 / count as f64,
            full.as_secs_f64() / bounded.as_secs_f64()
        );
    }
}

#[cfg(test)]
mod bounded_path_tests {
    use super::*;
    use crate::compat::commands::{Offset, PaintPathCommand as Cmd, PaintPathVerb as V};
    use crate::src::core::SkRasterClip::SkRasterClip;

    fn contours(x: f64, y: f64, kind: usize) -> Vec<Cmd> {
        let point = |dx, dy| Offset {
            x: x + dx,
            y: y + dy,
        };
        let mut commands = vec![Cmd {
            verb: V::kMoveTo,
            point: point(0.0, 1.125),
            ..Default::default()
        }];
        match kind {
            0 => commands.extend([
                Cmd {
                    verb: V::kLineTo,
                    point: point(17.75, 1.125),
                    ..Default::default()
                },
                Cmd {
                    verb: V::kLineTo,
                    point: point(17.75, 14.625),
                    ..Default::default()
                },
                Cmd {
                    verb: V::kLineTo,
                    point: point(0.0, 14.625),
                    ..Default::default()
                },
            ]),
            1 => commands.extend([
                Cmd {
                    verb: V::kQuadraticTo,
                    control1: point(14.0, -12.875),
                    point: point(27.125, 7.75),
                    ..Default::default()
                },
                Cmd {
                    verb: V::kCubicTo,
                    control1: point(10.25, 32.125),
                    control2: point(-13.75, 7.75),
                    point: point(0.0, 1.125),
                    ..Default::default()
                },
            ]),
            2 => commands.extend([
                Cmd {
                    verb: V::kConicTo,
                    control1: point(24.875, -4.5),
                    point: point(24.875, 22.5),
                    conic_weight: std::f64::consts::FRAC_1_SQRT_2,
                    ..Default::default()
                },
                Cmd {
                    verb: V::kLineTo,
                    point: point(7.125, 27.875),
                    ..Default::default()
                },
            ]),
            _ => commands.extend([
                Cmd {
                    verb: V::kLineTo,
                    point: point(32.125, 1.125),
                    ..Default::default()
                },
                Cmd {
                    verb: V::kLineTo,
                    point: point(16.25, 8.125),
                    ..Default::default()
                },
                Cmd {
                    verb: V::kLineTo,
                    point: point(31.5, 31.875),
                    ..Default::default()
                },
                Cmd {
                    verb: V::kLineTo,
                    point: point(-1.875, 15.375),
                    ..Default::default()
                },
            ]),
        }
        commands.push(Cmd {
            verb: V::kClose,
            ..Default::default()
        });
        if kind == 3 {
            commands.extend([
                Cmd {
                    verb: V::kMoveTo,
                    point: point(7.125, 9.5),
                    ..Default::default()
                },
                Cmd {
                    verb: V::kLineTo,
                    point: point(10.75, 20.25),
                    ..Default::default()
                },
                Cmd {
                    verb: V::kLineTo,
                    point: point(14.5, 9.5),
                    ..Default::default()
                },
                Cmd {
                    verb: V::kClose,
                    ..Default::default()
                },
            ]);
        }
        commands
    }

    #[test]
    #[ignore = "reproducible debug path-target benchmark"]
    fn small_path_target_debug_timing() {
        let (width, height) = (2560, 1542);
        let commands = contours(1221.25, 901.875, 1);
        let mut clip = Mask::new(width, height).unwrap();
        for y in 896..944 {
            clip.data_mut()[(y * width + 1215) as usize..(y * width + 1262) as usize].fill(255);
        }
        let summary = SkRasterClip::from_mask(width, height, clip.data()).unwrap();
        let old = path_mask_with_clip_impl(
            &commands,
            false,
            Transform::identity(),
            width,
            height,
            Some(&clip),
            (0, 0),
            false,
            None,
            None,
            false,
        )
        .unwrap();
        let new = path_mask_with_clip(
            &commands,
            false,
            Transform::identity(),
            width,
            height,
            Some(&clip),
            (0, 0),
            false,
            None,
            Some(&summary),
        )
        .unwrap();
        assert_eq!(old.mask.data(), new.mask.data());
        assert_eq!(old.pairs, new.pairs);
        assert_eq!(old.blits, new.blits);
        let support = new.support_bounds.unwrap();
        assert!(support.width() * support.height() < 4096);
        let count = 10;
        let now = std::time::Instant::now();
        for _ in 0..count {
            std::hint::black_box(path_mask_with_clip_impl(
                std::hint::black_box(&commands),
                false,
                Transform::identity(),
                width,
                height,
                Some(&clip),
                (0, 0),
                false,
                None,
                None,
                false,
            ));
        }
        let old_time = now.elapsed();
        let now = std::time::Instant::now();
        for _ in 0..count {
            std::hint::black_box(path_mask_with_clip(
                std::hint::black_box(&commands),
                false,
                Transform::identity(),
                width,
                height,
                Some(&clip),
                (0, 0),
                false,
                None,
                Some(&summary),
            ));
        }
        let new_time = now.elapsed();
        eprintln!("small path debug: full clip scan/tile output {:.3}ms/op, metadata/bounded output {:.3}ms/op, {:.1}x; new blend support={}x{}", old_time.as_secs_f64()*1000.0/count as f64, new_time.as_secs_f64()*1000.0/count as f64, old_time.as_secs_f64()/new_time.as_secs_f64(), support.width(), support.height());
    }

    #[test]
    fn one_path_walk_packs_actual_support_and_preserves_global_h2_spans() {
        let (width, height) = (2560, 1440);
        let modes = vec![true; tile_count(width) * tile_count(height)];
        for kind in 0..4 {
            for transform in [Transform::identity(), Transform::from_scale(0.5, 0.5)] {
                let commands = contours(1221.25, 831.75, kind);
                let dense = path_mask_with_clip(
                    &commands,
                    false,
                    transform,
                    width,
                    height,
                    None,
                    (0, 0),
                    false,
                    None,
                    None,
                )
                .unwrap();
                let support = dense.support_bounds.unwrap();
                assert_eq!(dense.mask.storage_bounds(), support);
                assert!(
                    dense.mask.allocated_bytes() < 4096,
                    "small actual support cannot allocate a device plane"
                );
                let spans = path_spans_with_clip(
                    &commands,
                    false,
                    transform,
                    width,
                    height,
                    None,
                    (0, 0),
                    false,
                    Some(&modes),
                    None,
                )
                .unwrap();
                let direct = spans
                    .direct
                    .iter()
                    .flat_map(|span| {
                        (span.start..span.start + span.count)
                            .map(move |i| (i, span.coverage, span.pair))
                    })
                    .collect::<Vec<_>>();
                // The existing span carrier omits zero-coverage direct events:
                // they have no destination effect in either AA color kernel.
                // Compare every emitted nonzero event and its per-pixel H2 flag.
                let drawable = dense
                    .blits
                    .iter()
                    .copied()
                    .filter(|&(_, alpha, _)| alpha != 0)
                    .collect::<Vec<_>>();
                assert_eq!(
                    direct, drawable,
                    "virtual AA tile modes cannot change the actual clip route"
                );
                let area = (support.width() * support.height()) as usize;
                let mut alpha = vec![0; area];
                let mut pairs = vec![false; area];
                for span in &spans.accumulated {
                    let x = (span.start % width as usize) as u32;
                    let y = (span.start / width as usize) as u32;
                    assert!(
                        x >= support.left() as u32
                            && x + span.count as u32 <= support.right() as u32
                    );
                    assert!(y >= support.top() as u32 && y < support.bottom() as u32);
                    let start = ((y - support.top() as u32) * support.width() + x
                        - support.left() as u32) as usize;
                    alpha[start..start + span.count].fill(span.coverage);
                    pairs[start..start + span.count].fill(span.pair);
                }
                for y in support.top() as u32..support.bottom() as u32 {
                    let local = ((y - support.top() as u32) * support.width()) as usize;
                    let global = (y * width + support.left() as u32) as usize;
                    assert_eq!(
                        &alpha[local..local + support.width() as usize],
                        dense
                            .mask
                            .row_range(y, support.left() as u32, support.right() as u32)
                    );
                    for x in 0..support.width() as usize {
                        if alpha[local + x] != 0 {
                            assert_eq!(pairs[local + x], dense.pairs[global + x]);
                        }
                    }
                }
                assert!(
                    dense.mask.allocated_bytes() < 4096,
                    "row reads must keep packed ownership"
                );
            }
        }
    }

    #[test]
    fn metadata_path_target_preserves_dense_pixels_pairs_and_direct_order() {
        let (width, height) = (520, 360);
        let mut bw = Mask::new(width, height).unwrap();
        for y in 2..352 {
            bw.data_mut()[(y * width + 2) as usize..(y * width + 518) as usize].fill(255);
        }
        let mut aa = bw.clone();
        for y in 2..352 {
            for x in 2..518 {
                if (x + y) % 17 == 0 {
                    aa.data_mut()[(y * width + x) as usize] = 127;
                }
            }
        }
        let empty = Mask::new(width, height).unwrap();
        for position in [(-7.125, 3.875), (251.875, 251.125), (501.25, 345.875)] {
            for kind in 0..4 {
                let commands = contours(position.0, position.1, kind);
                for transform in [
                    Transform::identity(),
                    Transform::from_scale(0.75, 0.5),
                    Transform::from_row(1.0, 0.125, -0.25, 1.0, 0.375, 0.125),
                ] {
                    for clip in [None, Some(&bw), Some(&aa), Some(&empty)] {
                        let summary = clip.map(|mask| {
                            SkRasterClip::from_mask(width, height, mask.data()).unwrap()
                        });
                        for origin in [(0, 0), (7, 11)] {
                            for even_odd in [false, true] {
                                let old = path_mask_with_clip_impl(
                                    &commands, even_odd, transform, width, height, clip, origin,
                                    false, None, None, false,
                                )
                                .unwrap();
                                let new = path_mask_with_clip(
                                    &commands,
                                    even_odd,
                                    transform,
                                    width,
                                    height,
                                    clip,
                                    origin,
                                    false,
                                    None,
                                    summary.as_ref(),
                                )
                                .unwrap();
                                assert_eq!(
                                    new.mask.data(),
                                    old.mask.data(),
                                    "position={position:?},kind={kind},origin={origin:?}"
                                );
                                assert_eq!(new.pairs, old.pairs);
                                assert_eq!(new.blits, old.blits);
                                for (i, &alpha) in new.mask.data().iter().enumerate() {
                                    if alpha != 0 {
                                        let bounds = new
                                            .support_bounds
                                            .expect("nonzero accumulated alpha has support");
                                        let (x, y) = (
                                            (i % width as usize) as i32,
                                            (i / width as usize) as i32,
                                        );
                                        assert!(
                                            x >= bounds.left()
                                                && x < bounds.right()
                                                && y >= bounds.top()
                                                && y < bounds.bottom()
                                        );
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

#[cfg(test)]
mod highdpi_clip_profile_tests {
    use super::*;
    use crate::src::core::SkRasterClip::SkRasterClip;

    #[test]
    #[ignore = "debug production large clip stage profile"]
    fn highdpi_clip_stage_debug_timing() {
        let (width, height) = (2048, 1536);
        let rect = PaintRect {
            x: 112.0,
            y: 185.0,
            width: 800.0,
            height: 99.0,
        };
        let r = crate::compat::commands::PaintCornerRadius { x: 19.0, y: 19.0 };
        let radii = PaintCornerRadii {
            top_left: r,
            top_right: r,
            bottom_left: r,
            bottom_right: r,
        };
        for scale in [1.0, 2.0] {
            let now = std::time::Instant::now();
            let clip = rounded_rect_clip_mask(
                rect,
                radii,
                Transform::from_scale(scale, scale),
                width,
                height,
                None,
                None,
            )
            .expect("positive axis scale is already admitted");
            let build = now.elapsed();
            let mut runs = vec![false; clip.mask.data().len()];
            for &i in &clip.run_starts {
                runs[i] = true;
            }
            let support = IntRect::from_ltrb(
                (112.0 * scale) as i32 - 1,
                (185.0 * scale) as i32 - 1,
                (912.0 * scale) as i32 + 1,
                (284.0 * scale) as i32 + 1,
            );
            let now = std::time::Instant::now();
            let summary =
                SkRasterClip::from_mask_with_bounds(width, height, clip.mask.data(), support)
                    .unwrap();
            let metadata = now.elapsed();
            let now = std::time::Instant::now();
            std::hint::black_box(clip_aa_tiles_with_bounds(
                &clip.mask,
                &runs,
                (0, 0),
                summary.getBounds(),
            ));
            let tiles = now.elapsed();
            eprintln!("highdpi clip scale={scale} analytic=Some build={:.3} metadata={:.3} aa_tiles={:.3}ms",build.as_secs_f64()*1000.0,metadata.as_secs_f64()*1000.0,tiles.as_secs_f64()*1000.0);
        }
    }
}

#[cfg(test)]
fn aa_tiles_scalar_reference(
    mask: &Mask,
    run_starts: &[bool],
    device_origin: (u32, u32),
    support: Option<crate::raster::IntRect>,
) -> Vec<bool> {
    let width = mask.width();
    let height = mask.height();
    let mut modes = Vec::new();
    let mut oy = 0;
    while oy < height {
        let sy = oy - u32::from(oy != 0);
        let ey = (oy + 255 - u32::from(oy != 0) + 1).min(height);
        let mut ox = 0;
        while ox < width {
            let sx = ox - u32::from(ox != 0);
            let ex = (ox + 255 - u32::from(ox != 0) + 1).min(width);
            let (mut l, mut t, mut r, mut b) = (ex, ey, sx, sy);
            let mut aa = false;
            let active = support.and_then(|bounds| {
                crate::raster::IntRect::from_xywh(sx as i32, sy as i32, ex - sx, ey - sy)
                    .and_then(|tile| bounds.intersect(&tile))
            });
            let Some(active) = active else {
                modes.push(false);
                ox += if ox == 0 { 255 } else { 254 };
                continue;
            };
            for y in active.top() as u32..active.bottom() as u32 {
                for x in active.left() as u32..active.right() as u32 {
                    let a = mask.data()[(y * width + x) as usize];
                    if a != 0 && a != 255 {
                        aa = true;
                    }
                    if a != 0 {
                        l = l.min(x);
                        t = t.min(y);
                        r = r.max(x + 1);
                        b = b.max(y + 1);
                    }
                }
            }
            // SkAAClip::isRect compares relative row Y with absolute bottom.
            // Preserve the source behavior when the clip begins compute_alpha_below_line the
            // device origin, even if every surviving coverage byte is 255.
            if l < r && t < b && t != sy.max(device_origin.1) {
                aa = true;
            }
            if !aa && l < r && t < b {
                for y in t..b {
                    for x in l..r {
                        let i = (y * width + x) as usize;
                        if mask.data()[i] != 255
                            || (y > t
                                && x > l
                                && run_starts[i] != run_starts[(t * width + x) as usize])
                        {
                            aa = true;
                        }
                    }
                }
            }
            modes.push(aa);
            ox += if ox == 0 { 255 } else { 254 };
        }
        oy += if oy == 0 { 255 } else { 254 };
    }
    modes
}

#[cfg(test)]
mod clip_run_classification_tests {
    use super::*;
    use crate::src::core::SkRasterClip::SkRasterClip;
    #[test]
    fn run_alpha_snap_preserves_old_scalar_provenance_at_all_byte_thresholds() {
        for alpha in 0..=255u8 {
            for length in 1..=33 {
                let mut actual = vec![alpha; length];
                let direct: Vec<_> = (0..length).map(|x| (x * 7 + length) % 3 == 0).collect();
                let mut expected = actual.clone();
                for (x, value) in expected.iter_mut().enumerate() {
                    if !direct[x] {
                        *value = if *value > 247 {
                            255
                        } else if *value < 8 {
                            0
                        } else {
                            *value
                        };
                    }
                }
                snap_alpha_runs(&mut actual, &direct);
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn alpha_run_tile_classification_matches_old_scalar_mask_and_run_rules() {
        let (width, height) = (520, 360);
        for (x, y, w, h) in [
            (0, 0, 520, 360),
            (9, 7, 13, 17),
            (253, 247, 37, 51),
            (511, 350, 9, 10),
        ] {
            for pattern in 0..7 {
                let mut mask = Mask::new(width, height).unwrap();
                let mut flags = vec![false; mask.data().len()];
                for yy in y..y + h {
                    for xx in x..x + w {
                        let i = (yy * width + xx) as usize;
                        mask.data_mut()[i] = match pattern {
                            0 => 255,
                            1 => 127,
                            2 if (xx + yy) % 23 == 0 => 0,
                            3 if (xx + yy) % 23 == 0 => 127,
                            6 => 0,
                            _ => 255,
                        };
                        flags[i] = xx == x
                            || (pattern == 4 && xx % 17 == 0)
                            || (pattern == 5 && (xx + yy) % 17 == 0);
                    }
                }
                let summary = SkRasterClip::from_mask(width, height, mask.data()).unwrap();
                for origin in [(0, 0), (7, 9), (253, 247)] {
                    assert_eq!(
                        clip_aa_tiles_with_bounds(&mask, &flags, origin, summary.getBounds()),
                        aa_tiles_scalar_reference(&mask, &flags, origin, summary.getBounds()),
                        "xy={x},{y},pattern={pattern},origin={origin:?}"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod alpha_only_mask_sink_tests {
    use super::*;
    fn radii(x: f64, y: f64) -> PaintCornerRadii {
        let radius = crate::compat::commands::PaintCornerRadius { x, y };
        PaintCornerRadii {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
    fn compare(
        bounds: PaintRect,
        radii: PaintCornerRadii,
        transform: Transform,
        w: u32,
        h: u32,
        force_rle: bool,
    ) {
        let old = rounded_rect_alpha_target_with_sink::<false, true>(
            bounds, radii, transform, w, h, force_rle, None, None, false,
        )
        .unwrap();
        let new = rounded_rect_alpha_target_with_sink::<true, false>(
            bounds, radii, transform, w, h, force_rle, None, None, false,
        )
        .unwrap();
        assert_eq!(new.mask.data(),old.mask.data(),
            "bounds={bounds:?},radii={radii:?},transform={transform:?},size={w}x{h},force_rle={force_rle}");
        assert!(new.run_starts.is_empty());
    }
    #[test]
    fn alpha_only_sink_keeps_full_masks_and_rle_snap_under_scale_clip_and_tile_gutters() {
        for (w, h) in [(47, 39), (520, 360)] {
            for x in [-13.25, -2.0, 0.0, 7.75, 254.0, 256.2] {
                for scale in [0.8, 1.0, 1.2, 1.5, 2.0] {
                    let transform = Transform::from_row(scale, 0.0, 0.0, scale, 0.125, -0.375);
                    for r in [
                        radii(0.0, 0.0),
                        radii(0.125, 0.25),
                        radii(6.0, 9.0),
                        radii(18.5, 17.25),
                        radii(61.0, 49.0),
                        PaintCornerRadii {
                            top_left: crate::compat::commands::PaintCornerRadius {
                                x: 1.5,
                                y: 23.25,
                            },
                            top_right: crate::compat::commands::PaintCornerRadius {
                                x: 19.75,
                                y: 2.5,
                            },
                            bottom_right: crate::compat::commands::PaintCornerRadius {
                                x: 13.0,
                                y: 4.0,
                            },
                            bottom_left: crate::compat::commands::PaintCornerRadius {
                                x: 0.0,
                                y: 0.0,
                            },
                        },
                    ] {
                        for force_rle in [false, true] {
                            compare(
                                PaintRect {
                                    x,
                                    y: x / 2.0,
                                    width: 73.25,
                                    height: 49.5,
                                },
                                r,
                                transform,
                                w,
                                h,
                                force_rle,
                            );
                        }
                    }
                }
            }
        }
        // Real Baidu source/hole and high-DPI device sizes preserve the
        // original tile schedule, with no storage-origin transformation.
        for (x, y, width, height) in [
            (120.0, 203.0, 784.0, 83.0),
            (127.0, 213.0, 770.0, 69.0),
            (112.0, 185.0, 800.0, 99.0),
        ] {
            for force_rle in [false, true] {
                compare(
                    PaintRect {
                        x,
                        y,
                        width,
                        height,
                    },
                    radii(19.0, 19.0),
                    Transform::from_scale(2.0, 2.0),
                    2048,
                    1536,
                    force_rle,
                );
            }
        }
        // Tiny rectangle path has a separate mask blitter shortcut.
        for force_rle in [false, true] {
            compare(
                PaintRect {
                    x: 0.25,
                    y: 0.125,
                    width: 9.5,
                    height: 7.75,
                },
                radii(0.0, 0.0),
                Transform::identity(),
                47,
                39,
                force_rle,
            );
        }
    }
    #[test]
    fn alpha_region_storage_matches_full_mask_for_random_regions_and_reconstruction() {
        let mut state = 0x13579bdf_u32;
        let mut random = || {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            state
        };
        for (w, h) in [(47, 39), (520, 360), (2048, 1536)] {
            for scale in [0.8, 1.0, 1.2, 1.5, 2.0] {
                let transform = Transform::from_row(scale, 0.0, 0.0, scale, 0.125, -0.375);
                for force_rle in [false, true] {
                    let bounds = PaintRect {
                        x: if w > 520 { 112.0 } else { -3.25 },
                        y: if w > 520 { 185.0 } else { 2.5 },
                        width: if w > 520 { 800.0 } else { 291.5 },
                        height: if w > 520 { 99.0 } else { 179.25 },
                    };
                    let r = radii(19.0, 17.25);
                    let full = rounded_rect_alpha_target_with_sink::<false, true>(
                        bounds, r, transform, w, h, force_rle, None, None, false,
                    )
                    .unwrap()
                    .mask;
                    let mut regions = vec![
                        IntRect::from_xywh(-2, -3, 11, 9).unwrap(),
                        IntRect::from_xywh(0, 0, w, h).unwrap(),
                        IntRect::from_xywh(w as i32 - 1, h as i32 - 1, 9, 7).unwrap(),
                    ];
                    for _ in 0..16 {
                        regions.push(
                            IntRect::from_xywh(
                                (random() % (w + 20)) as i32 - 10,
                                (random() % (h + 20)) as i32 - 10,
                                1 + random() % (w + 13),
                                1 + random() % (h + 7),
                            )
                            .unwrap(),
                        );
                    }
                    for region in regions {
                        let cropped = rounded_rect_alpha_mask_region(
                            bounds, r, transform, w, h, force_rle, region,
                        )
                        .unwrap();
                        for y in 0..region.height() as usize {
                            for x in 0..region.width() as usize {
                                let gx = i64::from(region.left()) + x as i64;
                                let gy = i64::from(region.top()) + y as i64;
                                let expected =
                                    if gx >= 0 && gy >= 0 && gx < i64::from(w) && gy < i64::from(h)
                                    {
                                        full.data()[gy as usize * w as usize + gx as usize]
                                    } else {
                                        0
                                    };
                                assert_eq!(cropped.data()[y*region.width() as usize+x],expected,
                                    "size={w}x{h},scale={scale},rle={force_rle},region={region:?},pixel={x},{y}");
                            }
                        }
                    }
                    // Random partitions reconstruct the complete frame; the
                    // 254/255 tile-gutter cuts also appear in the middle case.
                    let cx = if w == 520 {
                        255
                    } else {
                        1 + random() % (w - 1)
                    };
                    let cy = 1 + random() % (h - 1);
                    let mut restored = vec![0; full.data().len()];
                    for (left, right) in [(0, cx), (cx, w)] {
                        for (top, bottom) in [(0, cy), (cy, h)] {
                            let region = IntRect::from_ltrb(
                                left as i32,
                                top as i32,
                                right as i32,
                                bottom as i32,
                            )
                            .unwrap();
                            let cropped = rounded_rect_alpha_mask_region(
                                bounds, r, transform, w, h, force_rle, region,
                            )
                            .unwrap();
                            for y in top..bottom {
                                let dst = (y * w + left) as usize;
                                let src = ((y - top) * region.width()) as usize;
                                restored[dst..dst + region.width() as usize].copy_from_slice(
                                    &cropped.data()[src..src + region.width() as usize],
                                );
                            }
                        }
                    }
                    assert_eq!(restored, full.data());
                }
            }
        }
    }

    #[test]
    #[ignore = "manual Debug timing; no performance assertion"]
    fn alpha_only_sink_debug_baidu_source_and_hole_benchmark() {
        use std::{hint::black_box, time::Instant};
        for (kind, x, y, width, height, force_rle) in [
            ("source40", 120.0, 203.0, 784.0, 83.0, false),
            ("source18", 127.0, 213.0, 770.0, 69.0, false),
            ("hole", 112.0, 185.0, 800.0, 99.0, true),
        ] {
            let bounds = PaintRect {
                x,
                y,
                width,
                height,
            };
            let r = radii(19.0, 19.0);
            let transform = Transform::from_scale(2.0, 2.0);
            let mut previous = Vec::new();
            let mut alpha_only = Vec::new();
            let mut bounded = Vec::new();
            let region = IntRect::from_ltrb(
                (x * 2.0) as i32 - 1,
                (y * 2.0) as i32 - 1,
                ((x + width) * 2.0) as i32 + 1,
                ((y + height) * 2.0) as i32 + 1,
            )
            .unwrap();
            for _ in 0..9 {
                let start = Instant::now();
                let old = rounded_rect_alpha_target_with_sink::<false, true>(
                    black_box(bounds),
                    r,
                    transform,
                    2048,
                    1536,
                    force_rle,
                    None,
                    None,
                    false,
                )
                .unwrap();
                previous.push(start.elapsed().as_secs_f64() * 1000.0);
                let start = Instant::now();
                let new = rounded_rect_alpha_target_with_sink::<true, false>(
                    black_box(bounds),
                    r,
                    transform,
                    2048,
                    1536,
                    force_rle,
                    None,
                    None,
                    false,
                )
                .unwrap();
                alpha_only.push(start.elapsed().as_secs_f64() * 1000.0);
                assert_eq!(new.mask.data(), old.mask.data());
                black_box(&new);
                let start = Instant::now();
                let cropped = rounded_rect_alpha_mask_region(
                    bounds, r, transform, 2048, 1536, force_rle, region,
                )
                .unwrap();
                bounded.push(start.elapsed().as_secs_f64() * 1000.0);
                for y in 0..region.height() as usize {
                    let dst = y * region.width() as usize;
                    let src = (y + region.top() as usize) * 2048 + region.left() as usize;
                    assert_eq!(
                        &cropped.data()[dst..dst + region.width() as usize],
                        &old.mask.data()[src..src + region.width() as usize]
                    );
                }
                black_box(&cropped);
            }
            previous.sort_by(f64::total_cmp);
            alpha_only.sort_by(f64::total_cmp);
            bounded.sort_by(f64::total_cmp);
            eprintln!("ALPHA_ONLY_MASK_BENCH kind={kind} previous_ms={previous:?} alpha_only_ms={alpha_only:?} region_ms={bounded:?} speedup={:.3}",previous[4]/alpha_only[4]);
        }
    }
}

#[cfg(test)]
mod alpha_clip_sink_tests {
    use super::*;
    use crate::compat::commands::PaintCornerRadius as Radius;
    use crate::src::core::SkRasterClip::SkRasterClip;
    fn radii(x: f64, y: f64) -> PaintCornerRadii {
        let r = Radius { x, y };
        PaintCornerRadii {
            top_left: r,
            top_right: r,
            bottom_left: r,
            bottom_right: r,
        }
    }
    #[test]
    fn clip_alpha_only_mask_and_run_order_match_old_provenance_target() {
        let mut cases = 0;
        for (w, h) in [(47, 39), (520, 360), (2560, 1542)] {
            let mut bw = Mask::new(w, h).unwrap();
            for y in 1..h - 1 {
                bw.data_mut()[(y * w + 2) as usize..((y + 1) * w - 2) as usize].fill(255);
            }
            let mut aa = bw.clone();
            for (i, value) in aa.data_mut().iter_mut().enumerate() {
                if *value != 0 {
                    *value = [0, 1, 6, 7, 8, 127, 247, 248, 254, 255][(i * 3 + i / 13) % 10];
                }
            }
            let empty = Mask::new(w, h).unwrap();
            for (x, y, width, height) in [
                (-2.125, 0.375, 18.75, 13.5),
                (253.875, 254.125, 49.5, 21.75),
                (w as f64 - 8.25, h as f64 - 6.5, 31.25, 27.75),
                (112.0, 185.0, 800.0, 99.0),
            ] {
                for scale in [0.8, 1.0, 1.2, 1.5, 2.0] {
                    let t = Transform::from_row(scale, 0.0, 0.0, scale, 0.125, -0.375);
                    for r in [radii(0.0, 0.0), radii(0.125, 0.25), radii(19.0, 17.25)] {
                        for clip in [None, Some(&bw), Some(&aa), Some(&empty)] {
                            let summary =
                                clip.map(|m| SkRasterClip::from_mask(w, h, m.data()).unwrap());
                            let bounds = PaintRect {
                                x,
                                y,
                                width,
                                height,
                            };
                            let old = rounded_rect_alpha_target_with_sink::<false, true>(
                                bounds,
                                r,
                                t,
                                w,
                                h,
                                true,
                                clip,
                                summary.as_ref(),
                                true,
                            );
                            let shadow = rounded_rect_alpha_target_with_sink::<true, false>(
                                bounds,
                                r,
                                t,
                                w,
                                h,
                                true,
                                clip,
                                summary.as_ref(),
                                false,
                            );
                            // Full-device shadow coverage keeps the old target
                            // contract. Only an AA clip uses the native snug
                            // scan domain (SkAAClip.cpp1403-1427).
                            match (&old,&shadow) {
                                (Some(old),Some(shadow))=>assert_eq!(shadow.mask.data(),old.mask.data()),
                                (None,None)=>{},
                                _=>panic!("shadow support differs: size={w}x{h},bounds={bounds:?},scale={scale}"),
                            }
                            let source_scan =
                                source_snug_clip_summary(bounds, t, w, h, summary.as_ref());
                            let source_old = rounded_rect_mask_with_clip(
                                bounds,
                                r,
                                t,
                                w,
                                h,
                                true,
                                clip,
                                Some(&source_scan),
                            );
                            let new =
                                rounded_rect_clip_mask(bounds, r, t, w, h, clip, summary.as_ref());
                            match (source_old,new) {
                                (Some(source_old),Some(new)) => {
                                    assert_eq!(new.mask.data(),source_old.mask.data(),"mask size={w}x{h},bounds={bounds:?},scale={scale},r={r:?}");
                                    assert_eq!(new.run_starts,source_old.run_starts,"runs size={w}x{h},bounds={bounds:?},scale={scale},r={r:?}");
                                    if w==47 && h==39 && bounds.x==-2.125 && scale==1.0
                                        && r.top_left.x==0.125 && clip.is_none() {
                                        // Independent original SkCanvas full
                                        // A8 fixture native208/0 vs old219/4.
                                        let old=old.as_ref().unwrap();
                                        assert_eq!(old.mask.data()[16],219);
                                        assert_eq!(old.mask.data()[17],4);
                                        assert_eq!(new.mask.data()[16],208);
                                        assert_eq!(new.mask.data()[17],0);
                                        assert_ne!(old.mask.data(),new.mask.data());
                                    }
                                }
                                (None,None) => {}, // Original 4096 fixed-point domain fallback.
                                _ => panic!("target support differs: size={w}x{h},bounds={bounds:?},scale={scale}"),
                            }
                            cases += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(cases, 720);
    }
    #[test]
    #[ignore = "manual Debug timing; no performance assertion"]
    fn clip_alpha_only_debug_actual_window_benchmark() {
        use std::{hint::black_box, time::Instant};
        let (w, h) = (2560, 1542);
        for (kind, bounds, r) in [
            (
                "small",
                PaintRect {
                    x: 254.75,
                    y: 125.5,
                    width: 17.5,
                    height: 13.75,
                },
                radii(4.125, 2.875),
            ),
            (
                "search",
                PaintRect {
                    x: 112.0,
                    y: 185.0,
                    width: 800.0,
                    height: 99.0,
                },
                radii(19.0, 19.0),
            ),
            (
                "large",
                PaintRect {
                    x: 0.125,
                    y: 0.375,
                    width: 1279.0,
                    height: 760.0,
                },
                radii(12.0, 12.0),
            ),
        ] {
            let t = Transform::from_scale(2.0, 2.0);
            let mut old_ms = Vec::new();
            let mut new_ms = Vec::new();
            for _ in 0..9 {
                let start = Instant::now();
                let old = rounded_rect_alpha_target_with_sink::<false, true>(
                    black_box(bounds),
                    r,
                    t,
                    w,
                    h,
                    true,
                    None,
                    None,
                    true,
                )
                .unwrap();
                old_ms.push(start.elapsed().as_secs_f64() * 1000.0);
                let start = Instant::now();
                let new =
                    rounded_rect_clip_mask(black_box(bounds), r, t, w, h, None, None).unwrap();
                new_ms.push(start.elapsed().as_secs_f64() * 1000.0);
                assert_eq!(new.mask.data(), old.mask.data());
                assert_eq!(new.run_starts, old.run_starts);
                black_box(new);
            }
            old_ms.sort_by(f64::total_cmp);
            new_ms.sort_by(f64::total_cmp);
            eprintln!("ALPHA_CLIP_BENCH kind={kind} previous_ms={old_ms:?} alpha_only_ms={new_ms:?} speedup={:.3}",old_ms[4]/new_ms[4]);
        }
    }
}

#[cfg(test)]
mod bounded_storage_integration_round6 {
    use super::*;
    use crate::src::core::SkRasterClip::SkRasterClip;
    fn dense_reference<const ALPHA_ONLY: bool, const RECORD_RUNS: bool>(
        bounds: PaintRect,
        radii: PaintCornerRadii,
        canvas: Transform,
        width: u32,
        height: u32,
        force_rle: bool,
        clip: Option<&Mask>,
        clip_summary: Option<&crate::src::core::SkRasterClip::SkRasterClip>,
        collect_runs: bool,
    ) -> Option<RoundedClipCoverage> {
        let mut output = RoundedClipCoverage {
            mask: Mask::new(width, height)?,
            run_starts: Vec::new(),
        };
        let mut rect_spans = Vec::new();
        let mut rect_target = |ox, nx, lx, oy, ny, ly, rect: &ConvexRect, full_alpha| {
            append_convex_rect_spans(
                &mut rect_spans,
                rect,
                full_alpha,
                width,
                ox,
                nx,
                lx,
                oy,
                ny,
                ly,
            );
        };
        let rect_target: Option<&mut dyn FnMut(u32, u32, u32, u32, u32, u32, &ConvexRect, u8)> =
            Some(&mut rect_target);
        visit_rounded_rect_tiles_impl::<ALPHA_ONLY, RECORD_RUNS>(
            bounds,
            radii,
            canvas,
            width,
            height,
            force_rle,
            clip,
            clip_summary,
            false,
            true,
            rect_target,
            &mut |ox, nx, lx, oy, ny, ly, tw, tile| {
                if collect_runs {
                    for &i in &tile.run_starts {
                        let x = i as u32 % tw;
                        let y = i as u32 / tw;
                        if x >= lx && x < lx + nx && y >= ly && y < ly + ny {
                            output
                                .run_starts
                                .push(((oy + y - ly) * width + ox + x - lx) as usize);
                        }
                    }
                }
                for y in 0..ny {
                    let dst = ((oy + y) * width + ox) as usize;
                    let src = ((ly + y) * tw + lx) as usize;
                    output.mask.data_mut()[dst..dst + nx as usize]
                        .copy_from_slice(&tile.mask.data()[src..src + nx as usize]);
                }
            },
            (0, 0),
        )?;
        for span in rect_spans {
            output.mask.data_mut()[span.start..span.start + span.count].fill(span.coverage);
        }
        Some(output)
    }

    fn shape() -> (PaintRect, PaintCornerRadii) {
        let r = crate::compat::commands::PaintCornerRadius { x: 28.8, y: 28.8 };
        (
            PaintRect {
                x: 247.25,
                y: 245.5,
                width: 610.5,
                height: 130.75,
            },
            PaintCornerRadii {
                top_left: r,
                top_right: r,
                bottom_left: r,
                bottom_right: r,
            },
        )
    }
    #[test]
    fn bounded_storage_metadata_and_runs_match_dense_without_materialization() {
        let (rect, radii) = shape();
        let (width, height) = (1216, 704);
        for scale in [0.5, 1.0, 2.0] {
            let transform = Transform::from_scale(scale, scale);
            let packed =
                rounded_rect_clip_mask(rect, radii, transform, width, height, None, None).unwrap();
            let allocated = packed.mask.allocated_bytes();
            assert!(allocated < width as usize * height as usize / 2);
            let summary =
                SkRasterClip::from_mask_builder(&packed.mask, Some(packed.mask.storage_bounds()))
                    .unwrap();
            let tiles = clip_aa_tiles_with_sparse_runs(
                &packed.mask,
                &packed.run_starts,
                (0, 0),
                summary.getBounds(),
            );
            assert_eq!(
                packed.mask.allocated_bytes(),
                allocated,
                "metadata/tile scan must keep packed storage"
            );
            let dense = dense_reference::<true, true>(
                rect, radii, transform, width, height, true, None, None, true,
            )
            .unwrap();
            let expected = SkRasterClip::from_mask(width, height, dense.mask.data()).unwrap();
            assert_eq!(summary, expected);
            assert_eq!(packed.run_starts, dense.run_starts);
            assert_eq!(
                tiles,
                clip_aa_tiles_with_sparse_runs(
                    &dense.mask,
                    &dense.run_starts,
                    (0, 0),
                    expected.getBounds()
                )
            );
            assert_eq!(packed.mask.data(), dense.mask.data());
        }
    }
    #[test]
    #[ignore = "manual same-geometry Debug measurement; no frame-rate claim"]
    fn bounded_storage_debug_benchmark() {
        let (rect, radii) = shape();
        let (width, height) = (2432, 1408);
        let transform = Transform::from_scale(2.0, 2.0);
        for sample in 0..5 {
            for bounded in [false, true] {
                let mut checksum = 0usize;
                let mut bytes = 0;
                let start = std::time::Instant::now();
                for _ in 0..20 {
                    let clip = if bounded {
                        rounded_rect_clip_mask(rect, radii, transform, width, height, None, None)
                            .unwrap()
                    } else {
                        dense_reference::<true, true>(
                            rect, radii, transform, width, height, true, None, None, true,
                        )
                        .unwrap()
                    };
                    let summary = SkRasterClip::from_mask_builder(
                        &clip.mask,
                        Some(clip.mask.storage_bounds()),
                    )
                    .unwrap();
                    checksum ^= clip_aa_tiles_with_sparse_runs(
                        &clip.mask,
                        &clip.run_starts,
                        (0, 0),
                        summary.getBounds(),
                    )
                    .len();
                    bytes = clip.mask.allocated_bytes();
                    checksum ^= bytes;
                    std::hint::black_box(&clip);
                }
                eprintln!("bounded-mask-debug sample={sample} bounded={bounded} mean_ms={:.6} bytes={bytes} checksum={checksum}",start.elapsed().as_secs_f64()*1000.0/20.0);
            }
        }
    }
}
