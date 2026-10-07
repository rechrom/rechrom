//! Local replay adapters; these methods are not asserted as C++ SkCanvas API.
use crate::compat::commands::*;
use crate::compat::geometry::*;
use crate::cpu::hvgl::HvglTable;
use crate::cpu::image_sampling::{build_mip_image, draw_image_bitmap, mip_level_for_size};
use crate::raster::{
    ColorU8, FillRule, FilterQuality, GradientStop, IntSize, LineCap, LineJoin, LinearGradient,
    Mask, Paint, Path, PathBuilder, Pattern, Pixmap, Point, Rect, SpreadMode, Stroke, StrokeDash,
    Transform,
};
use crate::src::core::SkCanvas::{SavedFrame, SkCanvas};
use crate::src::core::SkMaskBlurFilter::{blur_alpha_mask, blur_alpha_mask_with_bounds};
use std::collections::HashMap;
use ttf_parser::{Face, GlyphId, OutlineBuilder, Tag};
// Preserve the original scan converter's AA runs. A vertical shader on a
// transparent device has one exact packed result per (device y, coverage) run.
struct VerticalMaskGradientBlitter<'a, 'b: 'a> {
    inner: crate::src::core::SkRasterPipelineBlitter::SkRasterPipelineBlitter<'a, 'b>,
    clip: crate::raster::geom::ScreenIntRect,
    mask: Option<&'a Mask>,
}
impl crate::raster::blitter::Blitter for VerticalMaskGradientBlitter<'_, '_> {
    fn blit_h(&mut self, x: u32, y: u32, width: core::num::NonZeroU32) {
        self.shade_run(x, y, width.get(), 255);
    }
    fn blit_anti_h(
        &mut self,
        mut x: u32,
        y: u32,
        aa: &mut [u8],
        runs: &mut [crate::raster::alpha_runs::AlphaRun],
    ) {
        let mut offset = 0;
        while let Some(run) = runs[offset] {
            self.shade_run(x, y, u32::from(run.get()), aa[offset]);
            offset += usize::from(run.get());
            x += u32::from(run.get());
        }
    }
    fn blit_rect(&mut self, rect: &crate::raster::geom::ScreenIntRect) {
        for y in rect.y()..rect.bottom() {
            self.shade_run(rect.x(), y, rect.width(), 255);
        }
    }
    fn blit_v(&mut self, x: u32, y: u32, height: core::num::NonZeroU32, alpha: u8) {
        use crate::raster::blitter::Blitter;
        self.inner.blit_v(x, y, height, alpha);
    }
    fn blit_anti_h2(&mut self, x: u32, y: u32, alpha0: u8, alpha1: u8) {
        use crate::raster::blitter::Blitter;
        self.inner.blit_anti_h2(x, y, alpha0, alpha1);
    }
    fn blit_anti_v2(&mut self, x: u32, y: u32, alpha0: u8, alpha1: u8) {
        use crate::raster::blitter::Blitter;
        self.inner.blit_anti_v2(x, y, alpha0, alpha1);
    }
    fn blit_mask(
        &mut self,
        mask: &crate::raster::blitter::Mask,
        clip: &crate::raster::geom::ScreenIntRect,
    ) {
        use crate::raster::blitter::Blitter;
        self.inner.blit_mask(mask, clip);
    }
}
impl VerticalMaskGradientBlitter<'_, '_> {
    fn shade_run(&mut self, x: u32, y: u32, width: u32, alpha: u8) {
        if y < self.clip.top() || y >= self.clip.bottom() || alpha == 0 {
            return;
        }
        let mut first = x.max(self.clip.left());
        let mut end = (x + width).min(self.clip.right());
        if let Some(mask) = self.mask {
            // Bound reads to the real packed AA storage; outside is zero.
            // Split scanner runs at every clip-alpha change. The inner
            // pipeline still reads that actual mask and applies its original
            // mask/scan-coverage stages; no AA value is promoted to BW.
            let bounds = mask.storage_bounds();
            if y < bounds.top() as u32 || y >= bounds.bottom() as u32 {
                return;
            }
            first = first.max(bounds.left() as u32);
            end = end.min(bounds.right() as u32);
            if first >= end {
                return;
            }
            let row = mask.row_range(y, first, end);
            let mut offset = 0;
            while offset < row.len() {
                let clip_alpha = row[offset];
                let next = crate::cpu::analytic_aa::equal_byte_run_end(row, offset, clip_alpha);
                if clip_alpha != 0 {
                    self.inner.blit_uniform_in_x_h(
                        first + offset as u32,
                        y,
                        core::num::NonZeroU32::new((next - offset) as u32)
                            .expect("nonempty equal-alpha run"),
                        alpha,
                    );
                }
                offset = next;
            }
            return;
        }
        if let Some(width) = end.checked_sub(first).and_then(core::num::NonZeroU32::new) {
            self.inner.blit_uniform_in_x_h(first, y, width, alpha);
        }
    }
}

// Restore a mask surface with DstIn inside a unit-coverage BW clip.
// Pixel formats share alpha at byte 3; all four destination bytes use exactly
// SkMulDiv255Round. Partial clip coverage retains the original scalar path.
fn mask_dst_in_rgba_span(dst: &mut [u8], mask: &[u8]) {
    debug_assert_eq!(dst.len(), mask.len());
    debug_assert_eq!(dst.len() % 4, 0);
    let mut done = 0;
    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    {
        let blocks = dst.len() / 16;
        if blocks != 0 {
            let indices = [3u8, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15];
            // Each vector covers four complete accessible RGBA pixels.
            // Widened byte products plus 128 are <= 65153; adding their
            // high byte stays <= 65407, so the u16 rounding cannot overflow.
            unsafe {
                core::arch::asm!(
                    "ldr q6, [{indices}]",
                    "movi v5.8h, #128",
                    "2:",
                    "ldr q0, [{source}], #16",
                    "tbl v1.16b, {{v0.16b}}, v6.16b",
                    "ldr q2, [{target}]",
                    "umull v3.8h, v2.8b, v1.8b",
                    "umull2 v4.8h, v2.16b, v1.16b",
                    "add v3.8h, v3.8h, v5.8h",
                    "add v4.8h, v4.8h, v5.8h",
                    "usra v3.8h, v3.8h, #8",
                    "usra v4.8h, v4.8h, #8",
                    "shrn v2.8b, v3.8h, #8",
                    "shrn2 v2.16b, v4.8h, #8",
                    "str q2, [{target}], #16",
                    "subs {count}, {count}, #1",
                    "b.ne 2b",
                    source = inout(reg) mask.as_ptr() => _,
                    target = inout(reg) dst.as_mut_ptr() => _,
                    count = inout(reg) blocks => _,
                    indices = in(reg) indices.as_ptr(),
                    out("v0") _, out("v1") _, out("v2") _,
                    out("v3") _, out("v4") _, out("v5") _, out("v6") _,
                    options(nostack),
                );
            }
            done = blocks * 16;
        }
    }
    for (pixel, source) in dst[done..]
        .chunks_exact_mut(4)
        .zip(mask[done..].chunks_exact(4))
    {
        for channel in pixel {
            *channel = crate::cpu::mask_blitter::mul_div_255_round(*channel, source[3]);
        }
    }
}

impl SkCanvas {
    // SkRasterClip limits intersect operations by the old clip bounds. The
    // analytic walker snaps fixed-point edges, so retain a conservative pixel
    // margin before deriving exact metadata from the resulting alpha bytes.
    fn analytic_clip_support(&self, rect: PaintRect) -> Option<crate::raster::IntRect> {
        let tr = self.state.transform;
        let x = rect.x as f32 * tr.sx + tr.tx;
        let y = rect.y as f32 * tr.sy + tr.ty;
        let right = x + rect.width as f32 * tr.sx;
        let bottom = y + rect.height as f32 * tr.sy;
        let width = self.pixmap.width();
        let height = self.pixmap.height();
        let left = (x.floor() - 1.0).max(0.0).min(width as f32) as i32;
        let top = (y.floor() - 1.0).max(0.0).min(height as f32) as i32;
        let right = (right.ceil() + 1.0).max(0.0).min(width as f32) as i32;
        let bottom = (bottom.ceil() + 1.0).max(0.0).min(height as f32) as i32;
        let support = crate::raster::IntRect::from_xywh(
            left,
            top,
            right.saturating_sub(left) as u32,
            bottom.saturating_sub(top) as u32,
        )?;
        let old = self.state.clip_summary.or_else(|| {
            self.state.clip.as_deref().and_then(|mask| {
                crate::src::core::SkRasterClip::SkRasterClip::from_mask_builder(
                    mask,
                    Some(mask.storage_bounds()),
                )
            })
        });
        match old {
            Some(old) => old
                .getBounds()
                .and_then(|bounds| support.intersect(&bounds)),
            None => Some(support),
        }
    }

    // The analytic raster path scales x/width separately, while the path
    // fallback transforms the already rounded local right/bottom. Retain the
    // union of both projections for rejection, including large coordinates.
    fn rect_clip_reject_support(&self, rect: PaintRect) -> Option<crate::raster::IntRect> {
        let tr = self.state.transform;
        let x = rect.x as f32 * tr.sx + tr.tx;
        let y = rect.y as f32 * tr.sy + tr.ty;
        let right = x + rect.width as f32 * tr.sx;
        let bottom = y + rect.height as f32 * tr.sy;
        let path_right = (rect.x as f32 + rect.width as f32) * tr.sx + tr.tx;
        let path_bottom = (rect.y as f32 + rect.height as f32) * tr.sy + tr.ty;
        if ![x, y, right, bottom, path_right, path_bottom]
            .into_iter()
            .all(f32::is_finite)
        {
            return crate::raster::IntRect::from_xywh(
                0,
                0,
                self.pixmap.width(),
                self.pixmap.height(),
            );
        }
        let width = self.pixmap.width() as f32;
        let height = self.pixmap.height() as f32;
        let left = (x.floor() - 1.0).max(0.0).min(width) as i32;
        let top = (y.floor() - 1.0).max(0.0).min(height) as i32;
        let right = (right.max(path_right).ceil() + 1.0).max(0.0).min(width) as i32;
        let bottom = (bottom.max(path_bottom).ceil() + 1.0).max(0.0).min(height) as i32;
        let bounds = crate::raster::IntRect::from_xywh(
            left,
            top,
            right.saturating_sub(left) as u32,
            bottom.saturating_sub(top) as u32,
        )?;
        match self.state.clip_summary {
            Some(old) => old.getBounds().and_then(|old| bounds.intersect(&old)),
            None => Some(bounds),
        }
    }

    fn install_analytic_clip(
        &mut self,
        mut coverage: crate::compat::analytic_masks::RoundedClipCoverage,
        support: Option<crate::raster::IntRect>,
        retain_runs: bool,
    ) {
        #[cfg(feature = "profiling")]
        let profile = std::env::var_os("SKIA_TRACE_CLIP")
            .is_some()
            .then(std::time::Instant::now);
        let width = coverage.mask.width() as usize;
        // The old opaque rectangular clip was already passed to setPath as
        // scan-conversion bounds; its intersection needs no alpha multiply.
        let rectangular = self.state.clip_summary.is_some_and(|clip| clip.isRect());
        if !rectangular {
            if let (Some(old), Some(bounds)) = (&self.state.clip, support) {
                for y in bounds.top() as usize..bounds.bottom() as usize {
                    let old_row =
                        old.row_range(y as u32, bounds.left() as u32, bounds.right() as u32);
                    let new_row = coverage.mask.row_range_mut(
                        y as u32,
                        bounds.left() as u32,
                        bounds.right() as u32,
                    );
                    let mut x = 0;
                    while x < old_row.len() {
                        let alpha = old_row[x];
                        let run_end =
                            crate::cpu::analytic_aa::equal_byte_run_end(old_row, x, alpha);
                        match alpha {
                            255 => {} // Intersect with full coverage is identity.
                            0 => new_row[x..run_end].fill(0),
                            _ => {
                                for a in &mut new_row[x..run_end] {
                                    *a = crate::cpu::mask_blitter::mul_div_255_round(*a, alpha);
                                }
                            }
                        }
                        x = run_end;
                    }
                }
            }
        }
        #[cfg(feature = "profiling")]
        let intersect_elapsed = profile.map(|p| p.elapsed());
        let summary = crate::src::core::SkRasterClip::SkRasterClip::from_mask_builder(
            &coverage.mask,
            support,
        )
        .expect("valid bounded raster clip");
        #[cfg(feature = "profiling")]
        let metadata_elapsed = profile.map(|p| p.elapsed());
        if retain_runs {
            // Keep equal adjacent SkAAClip runs, without cloning flags for
            // pixels outside the surviving clip. Bitmap shaders depend on
            // these restart points even when neighbouring alpha is identical.
            let mut runs = Vec::new();
            if let (Some(old), Some(bounds)) = (&self.state.clip_runs, summary.getBounds()) {
                // Keep only surviving run boundaries. A sorted index target
                // preserves equal adjacent AA runs without a device-sized flag
                // plane, matching SkAAClip's bounded run representation.
                let first = bounds.top() as usize * width + bounds.left() as usize;
                let last = (bounds.bottom() as usize - 1) * width + bounds.right() as usize;
                let start = old.partition_point(|&i| i < first);
                let end = old.partition_point(|&i| i < last);
                for &i in &old[start..end] {
                    let x = i % width;
                    if x >= bounds.left() as usize && x < bounds.right() as usize {
                        runs.push(i);
                    }
                }
            }
            runs.append(&mut coverage.run_starts);
            runs.sort_unstable();
            runs.dedup();
            #[cfg(feature = "profiling")]
            let sorted_elapsed = profile.map(|p| p.elapsed());
            // Analytic path scanning consumes the actual clip class and
            // these native run starts; its legacy tile-mode input is unused.
            self.state.clip_aa_tiles = None;
            #[cfg(feature = "profiling")]
            if let Some(profile) = profile {
                eprintln!(
                    "clip-run-tiles sort_ms={:.6} classify_ms={:.6} runs={}",
                    (sorted_elapsed.unwrap() - metadata_elapsed.unwrap()).as_secs_f64() * 1000.,
                    (profile.elapsed() - sorted_elapsed.unwrap()).as_secs_f64() * 1000.,
                    runs.len()
                );
            }
            self.state.clip_runs = Some(std::sync::Arc::new(runs));
        }
        self.state.clip = Some(std::sync::Arc::new(coverage.mask));
        self.state.clip_is_aa = true;
        self.state.clip_summary = Some(summary);
        #[cfg(feature = "profiling")]
        if let Some(profile) = profile {
            eprintln!("clip-install intersect_ms={:.6} metadata_ms={:.6} runs_tiles_ms={:.6} storage_bytes={} support={support:?}",intersect_elapsed.unwrap().as_secs_f64()*1000.,(metadata_elapsed.unwrap()-intersect_elapsed.unwrap()).as_secs_f64()*1000.,(profile.elapsed()-metadata_elapsed.unwrap()).as_secs_f64()*1000.,self.state.clip.as_ref().unwrap().allocated_bytes());
        }
    }

    pub fn replay_item(&mut self, item: &DrawCommand, list: &ResourceContext) {
        // SkCanvas::internalQuickReject and SkRasterClip::setEmpty: drawing
        // and intersect/difference clips cannot make an empty clip visible.
        // Preserve save/restore, transforms and layer/mask stack operations.
        if self
            .state
            .clip_summary
            .is_some_and(|clip| clip.getBounds().is_none())
            && !matches!(
                item.r#type,
                CommandKind::kSave
                    | CommandKind::kRestore
                    | CommandKind::kConcat
                    | CommandKind::kSaveLayer
                    | CommandKind::kSaveLayerAlpha
                    | CommandKind::kSaveLayerBlend
                    | CommandKind::kSaveLayerFilter
                    | CommandKind::kSaveLayerDstIn
                    | CommandKind::kBeginMask
                    | CommandKind::kEndMask
            )
        {
            return;
        }
        // An intersecting rect/rrect whose conservative support misses the
        // device or prior clip has an exact empty result. Represent it by
        // SkRasterClip bounds instead of allocating a device-sized A8 plane.
        if matches!(
            item.r#type,
            CommandKind::kClipRect | CommandKind::kClipRoundedRect
        ) {
            let tr = self.state.transform;
            let geometry_valid = item.rect.width > 0.0
                && item.rect.height > 0.0
                && [item.rect.x, item.rect.y, item.rect.width, item.rect.height]
                    .into_iter()
                    .all(f64::is_finite)
                && [
                    item.corner_radii.top_left,
                    item.corner_radii.top_right,
                    item.corner_radii.bottom_left,
                    item.corner_radii.bottom_right,
                ]
                .into_iter()
                .all(|r| r.x >= 0.0 && r.y >= 0.0 && r.x.is_finite() && r.y.is_finite());
            if geometry_valid
                && tr.kx == 0.0
                && tr.ky == 0.0
                && tr.sx > 0.0
                && tr.sy > 0.0
                && [tr.sx, tr.sy, tr.tx, tr.ty].into_iter().all(f32::is_finite)
                && [
                    item.rect.x as f32 * tr.sx + tr.tx,
                    item.rect.y as f32 * tr.sy + tr.ty,
                    item.rect.x as f32 * tr.sx + tr.tx + item.rect.width as f32 * tr.sx,
                    item.rect.y as f32 * tr.sy + tr.ty + item.rect.height as f32 * tr.sy,
                ]
                .into_iter()
                .all(f32::is_finite)
                && self.rect_clip_reject_support(item.rect).is_none()
            {
                self.state.clip_encoding.set_empty();
                self.state.clip = None;
                self.state.clip_summary = Some(Default::default());
                self.state.clip_is_aa = false;
                self.state.clip_runs = None;
                self.state.clip_aa_tiles = None;
                return;
            }
        }
        // These implemented drawing commands composite with SrcOver;
        // control operations preserve the current device's alpha. saveLayer
        // starts a transparent device and restores the parent's alpha metadata.
        self.known_opaque &= item.blend_mode == PaintBlendMode::kNormal
            && matches!(
                item.r#type,
                CommandKind::kSave
                    | CommandKind::kRestore
                    | CommandKind::kConcat
                    | CommandKind::kClipRect
                    | CommandKind::kClipRoundedRect
                    | CommandKind::kClipPath
                    | CommandKind::kClipOutRect
                    | CommandKind::kClipOutRoundedRect
                    | CommandKind::kDrawRect
                    | CommandKind::kDrawRoundedRect
                    | CommandKind::kDrawDoubleRoundedRect
                    | CommandKind::kDrawEllipse
                    | CommandKind::kDrawPath
                    | CommandKind::kStrokePath
                    | CommandKind::kStrokeRect
                    | CommandKind::kStrokeEllipse
                    | CommandKind::kStrokeLine
                    | CommandKind::kDrawGlyphRun
                    | CommandKind::kDrawGradientRect
                    | CommandKind::kDrawTiledGradient
                    | CommandKind::kDrawImageRect
                    | CommandKind::kDrawTiledImage
                    | CommandKind::kDrawBoxShadow
                    | CommandKind::kDrawMask
                    | CommandKind::kSaveLayerAlpha
                    | CommandKind::kSaveLayer
                    | CommandKind::kSaveLayerBlend
                    | CommandKind::kSaveLayerFilter
                    | CommandKind::kSaveLayerDstIn
                    | CommandKind::kDrawScrollbarTrack
                    | CommandKind::kDrawScrollbarThumb
                    | CommandKind::kDrawScrollbarButton
                    | CommandKind::kDrawScrollbarCorner
            );

        // SkCanvas::clipRRect delegates a rectangular RRect to onClipRect.
        // Keep BW rectangle clipping and its bounds fast path in that case.
        let rounded_clip_commands = matches!(
            item.r#type,
            CommandKind::kClipRoundedRect | CommandKind::kClipOutRoundedRect
        )
        .then(|| crate::src::core::SkColorGlyphClip::rounded_commands(item.rect, item.corner_radii))
        .flatten();
        let native_rect = rounded_clip_commands.as_ref().is_some_and(|commands| {
            commands.len() == 5
                && commands[1..4]
                    .iter()
                    .all(|c| c.verb == PaintPathVerb::kLineTo)
        });
        let kind = if !item.corner_radii.HasRadius() || native_rect {
            match item.r#type {
                CommandKind::kClipRoundedRect => CommandKind::kClipRect,
                CommandKind::kClipOutRoundedRect => CommandKind::kClipOutRect,
                other => other,
            }
        } else {
            item.r#type
        };
        match kind {
            CommandKind::kSave => self.save(),
            CommandKind::kRestore => self.restore(),
            CommandKind::kSaveLayerAlpha => self.save_layer_alpha(item),
            CommandKind::kSaveLayer => self.internalSaveLayer(item, false),
            CommandKind::kSaveLayerDstIn => {
                self.internalSaveLayer(item, false);
                self.stack.last_mut().expect("DstIn saved device").dst_in = true;
            }
            CommandKind::kSaveLayerBlend => {
                // CSS isolation also emits a blend layer for normal SrcOver.
                // It still needs a separate surface so descendants cannot
                // blend directly with the page backdrop.
                assert_eq!(item.blend_mode, PaintBlendMode::kNormal);
                self.internalSaveLayer(item, false);
            }
            // cpp: skia_renderer/skia_renderer.cc:1050-1073
            CommandKind::kSaveLayerFilter => self.save_layer_filter(item),
            CommandKind::kBeginMask => self.begin_mask(item),
            CommandKind::kEndMask => self.end_mask(list),
            CommandKind::kConcat => self.concat(item.transform.values),
            CommandKind::kClipRect => {
                let rect = item.rect;
                let mut aa = item.antialias;
                let tr = self.state.transform;
                if tr.sx > 0.0 && tr.sy > 0.0 && tr.kx == 0.0 && tr.ky == 0.0 {
                    let l = rect.x as f32 * tr.sx + tr.tx;
                    let t = rect.y as f32 * tr.sy + tr.ty;
                    let r = (rect.x as f32 + rect.width as f32) * tr.sx + tr.tx;
                    let b = (rect.y as f32 + rect.height as f32) * tr.sy + tr.ty;
                    use crate::src::core::SkColorGlyphClip::{
                        produce_path, CanvasClipOwner, FloatRect, ReceiverKind,
                    };
                    let was_native_bw = self.state.clip_encoding.is_bw();
                    let device_rect = FloatRect([l, t, r, b]);
                    // The owner calls the real rect receiver only when the
                    // native AA operation is not an identity or integer op.
                    let dims = (self.pixmap.width(), self.pixmap.height());
                    let mut rect_product = None;
                    let cache = &mut self.clip_product_cache;
                    self.state.clip_encoding.intersect_float_rect(
                        device_rect,
                        aa,
                        |rect, bounds| {
                            let Some(shape) = crate::raster::Rect::from_ltrb(
                                rect.0[0], rect.0[1], rect.0[2], rect.0[3],
                            ) else {
                                return None;
                            };
                            let product = crate::src::core::SkColorGlyphClip::produce_path_cached(
                                cache,
                                &PathBuilder::from_rect(shape),
                                None,
                                FillRule::Winding,
                                true,
                                Transform::identity(),
                                dims.0,
                                dims.1,
                                bounds,
                                ReceiverKind::AaClip,
                            );
                            let CanvasClipOwner::Aa(encoding) = product.owner else {
                                unreachable!("requested AA receiver");
                            };
                            rect_product = Some(product.mask);
                            encoding
                        },
                    );
                    // Empty intersection and containment identity both avoid
                    // scanning a rectangle. Distinguish them using the owner
                    // before interpreting rect_product=None as identity.
                    if self.state.clip_encoding.bounds().is_none() {
                        self.state.clip = None;
                        self.state.clip_summary = Some(Default::default());
                        self.refresh_encoded_clip_helpers();
                        return;
                    }
                    // SkRasterClip::op recognizes near-integer AA rectangles in BW clips.
                    let bw = was_native_bw;
                    let integral = |v: f32| {
                        let v = v + 0.125;
                        v - v.floor() < 0.25
                    };
                    if bw && aa && [l, t, r, b].iter().all(|&v| integral(v)) {
                        aa = false;
                    }
                    if !aa {
                        let nl = (l + 0.5).floor();
                        let nt = (t + 0.5).floor();
                        let nr = (r + 0.5).floor();
                        let nb = (b + 0.5).floor();
                        // SkRasterClip::op rounds in device space, including
                        // scale+translate matrices, and retains a BW clip.
                        let width = self.pixmap.width() as usize;
                        let height = self.pixmap.height() as usize;
                        let clamp_x = |v: f32| v.max(0.0).min(width as f32) as usize;
                        let clamp_y = |v: f32| v.max(0.0).min(height as f32) as usize;
                        let (left, right, top, bottom) =
                            (clamp_x(nl), clamp_x(nr), clamp_y(nt), clamp_y(nb));
                        // SkRasterClip already starts at device bounds. An
                        // intersecting BW rect covering those bounds is an
                        // identity operation; avoid materializing an all-255
                        // alpha plane just to represent the device clip.
                        if left == 0 && top == 0 && right == width && bottom == height {
                            if self.state.clip_summary.is_none() && self.state.clip.is_none() {
                                let mut summary =
                                    crate::src::core::SkRasterClip::SkRasterClip::default();
                                summary.setRect(crate::raster::IntRect::from_xywh(
                                    0,
                                    0,
                                    width as u32,
                                    height as u32,
                                ));
                                self.state.clip_summary = Some(summary);
                            }
                            self.refresh_encoded_clip_helpers();
                            return;
                        }
                        let new_clip = self.state.clip.is_none();
                        let old_summary = self.state.clip_summary.or_else(|| {
                            self.state.clip.as_deref().and_then(|mask| {
                                crate::src::core::SkRasterClip::SkRasterClip::from_mask_builder(
                                    mask,
                                    Some(mask.storage_bounds()),
                                )
                            })
                        });
                        let requested = crate::raster::IntRect::from_xywh(
                            left as i32,
                            top as i32,
                            right.saturating_sub(left) as u32,
                            bottom.saturating_sub(top) as u32,
                        );
                        let support = match old_summary {
                            Some(old) => requested
                                .and_then(|b| old.getBounds().and_then(|old| b.intersect(&old))),
                            None => requested,
                        };
                        // SkRasterClip's BW rectangle is represented by its
                        // region bounds, not a device-sized A8 plane. Backends
                        // which still require alpha bytes expand it on demand.
                        if self.state.clip.is_none()
                            || old_summary.is_some_and(|c| c.isRect() || c.getBounds().is_none())
                        {
                            self.state.clip = None;
                            let mut summary =
                                crate::src::core::SkRasterClip::SkRasterClip::default();
                            summary.setRect(support);
                            self.state.clip_summary = Some(summary);
                            self.refresh_encoded_clip_helpers();
                            return;
                        }
                        let mask = self.state.clip.get_or_insert_with(|| {
                            std::sync::Arc::new(
                                Mask::new(width as u32, height as u32)
                                    .expect("valid raster mask size"),
                            )
                        });
                        let mask = std::sync::Arc::make_mut(mask);
                        if new_clip {
                            if let Some(bounds) = support {
                                for y in bounds.top() as usize..bounds.bottom() as usize {
                                    mask.data_mut()[y * width + bounds.left() as usize
                                        ..y * width + bounds.right() as usize]
                                        .fill(255);
                                }
                            }
                        } else if let Some(old_bounds) =
                            old_summary.and_then(|clip| clip.getBounds())
                        {
                            for y in old_bounds.top() as usize..old_bounds.bottom() as usize {
                                if support.is_some_and(|b| {
                                    y >= b.top() as usize && y < b.bottom() as usize
                                }) {
                                    let bounds = support.unwrap();
                                    mask.row_range_mut(
                                        y as u32,
                                        old_bounds.left() as u32,
                                        bounds.left() as u32,
                                    )
                                    .fill(0);
                                    mask.row_range_mut(
                                        y as u32,
                                        bounds.right() as u32,
                                        old_bounds.right() as u32,
                                    )
                                    .fill(0);
                                } else {
                                    mask.row_range_mut(
                                        y as u32,
                                        old_bounds.left() as u32,
                                        old_bounds.right() as u32,
                                    )
                                    .fill(0);
                                }
                            }
                        }
                        let mut summary = crate::src::core::SkRasterClip::SkRasterClip::default();
                        if new_clip
                            || old_summary
                                .is_some_and(|old| old.isRect() || old.getBounds().is_none())
                        {
                            summary.setRect(support);
                        } else {
                            summary =
                                crate::src::core::SkRasterClip::SkRasterClip::from_mask_builder(
                                    mask, support,
                                )
                                .expect("valid bounded raster clip");
                        }
                        self.state.clip_summary = Some(summary);
                        self.refresh_encoded_clip_helpers();
                        return;
                    }
                    if aa {
                        if let Some(mask) = rect_product {
                            // Clip owner is already the completed encoded op;
                            // the old pixel intersection and sparse metadata
                            // remain in their existing install adapter.
                            let support = self.analytic_clip_support(rect);
                            self.install_analytic_clip(
                                crate::compat::analytic_masks::RoundedClipCoverage {
                                    mask,
                                    run_starts: Vec::new(),
                                },
                                support,
                                false,
                            );
                        }
                        self.refresh_encoded_clip_helpers();
                        return; // native float identity or completed AA op
                    }
                }
                if let Some(bounds) = crate::compat::geometry::rect(rect) {
                    self.clipPath(&PathBuilder::from_rect(bounds), aa);
                }
            }

            CommandKind::kClipPath => {
                let Some(path) = path_from_commands(&item.path) else {
                    self.state.clip_encoding.set_empty();
                    self.state.clip = None;
                    self.state.clip_summary = Some(Default::default());
                    self.state.clip_is_aa = false;
                    self.state.clip_runs = None;
                    self.state.clip_aa_tiles = None;
                    return;
                };
                // SkAAClip uses an RLE alpha target. Retain the analytic path
                // scanner and force its RLE receiver before intersecting the
                // previous clip once. SkRasterClip.cpp:188-206 first rasterizes
                // the new path within the current clip bounds, even for an AA
                // old clip; coverage intersection follows afterward.
                // The old path handles unsupported curves.
                let rule = if item.even_odd {
                    FillRule::EvenOdd
                } else {
                    FillRule::Winding
                };
                let prepared = self.state.clip_encoding.bounds().map(|bounds| {
                    let kind = self.state.clip_encoding.path_receiver_kind(item.antialias);
                    crate::src::core::SkColorGlyphClip::produce_path_cached(
                        &mut self.clip_product_cache,
                        &path,
                        Some(&item.path),
                        rule,
                        item.antialias,
                        self.state.transform,
                        self.pixmap.width(),
                        self.pixmap.height(),
                        bounds,
                        kind,
                    )
                });
                self.clip_path_with_mask(
                    &path,
                    item.antialias,
                    if item.even_odd {
                        FillRule::EvenOdd
                    } else {
                        FillRule::Winding
                    },
                    prepared,
                );
            }

            CommandKind::kClipRoundedRect => {
                if let Some(commands) = rounded_clip_commands {
                    if let Some(path) = path_from_commands(&commands) {
                        let prepared = self.state.clip_encoding.bounds().map(|bounds| {
                            let kind = self.state.clip_encoding.path_receiver_kind(item.antialias);
                            crate::src::core::SkColorGlyphClip::produce_path_cached(
                                &mut self.clip_product_cache,
                                &path,
                                Some(&commands),
                                FillRule::Winding,
                                item.antialias,
                                self.state.transform,
                                self.pixmap.width(),
                                self.pixmap.height(),
                                bounds,
                                kind,
                            )
                        });
                        self.clip_path_with_mask(
                            &path,
                            item.antialias,
                            FillRule::Winding,
                            prepared,
                        );
                    }
                }
            }
            CommandKind::kClipOutRoundedRect | CommandKind::kClipOutRect => {
                use crate::src::core::SkColorGlyphClip::{
                    Bounds, CanvasClipOwner, FloatRect, ReceiverKind,
                };
                let Some(old_bounds) = self.state.clip_encoding.bounds() else {
                    return;
                };
                let mut mask = None;
                let mut bw_rect = None;
                let tr = self.state.transform;
                if kind == CommandKind::kClipOutRect && tr.kx == 0.0 && tr.ky == 0.0 {
                    let (l, t) = (item.rect.x as f32, item.rect.y as f32);
                    let (r, b) = (l + item.rect.width as f32, t + item.rect.height as f32);
                    let (x0, x1) = (l * tr.sx + tr.tx, r * tr.sx + tr.tx);
                    let (y0, y1) = (t * tr.sy + tr.ty, b * tr.sy + tr.ty);
                    let rect = FloatRect([x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1)]);
                    if !rect.0.into_iter().all(f32::is_finite)
                        || rect.0[0] >= rect.0[2]
                        || rect.0[1] >= rect.0[3]
                    {
                        return;
                    }
                    let near = rect.0.into_iter().all(|x| {
                        let x = x + 0.125;
                        x - x.floor() < 0.25
                    });
                    let effective_aa =
                        item.antialias && !(self.state.clip_encoding.is_bw() && near);
                    if !effective_aa {
                        bw_rect = Some(Bounds(rect.0.map(|x| (x + 0.5).floor() as i32)));
                    }
                    let cache = &mut self.clip_product_cache;
                    let dims = (self.pixmap.width(), self.pixmap.height());
                    self.state.clip_encoding.difference_float_rect(
                        rect,
                        item.antialias,
                        |rect, bounds| {
                            let shape = crate::raster::Rect::from_ltrb(
                                rect.0[0], rect.0[1], rect.0[2], rect.0[3],
                            )?;
                            let product = crate::src::core::SkColorGlyphClip::produce_path_cached(
                                cache,
                                &PathBuilder::from_rect(shape),
                                None,
                                FillRule::Winding,
                                true,
                                Transform::identity(),
                                dims.0,
                                dims.1,
                                bounds,
                                ReceiverKind::AaClip,
                            );
                            mask = Some(product.mask);
                            let CanvasClipOwner::Aa(encoding) = product.owner else {
                                unreachable!("requested AA rect receiver");
                            };
                            encoding
                        },
                    );
                } else {
                    let commands = if kind == CommandKind::kClipOutRoundedRect {
                        rounded_clip_commands
                    } else {
                        rect_commands(item.rect)
                    };
                    let Some(commands) = commands else {
                        return;
                    };
                    let Some(path) = path_from_commands(&commands) else {
                        return;
                    };
                    let cache = &mut self.clip_product_cache;
                    let dims = (self.pixmap.width(), self.pixmap.height());
                    self.state
                        .clip_encoding
                        .difference_path(item.antialias, |bounds, receiver| {
                            let product = crate::src::core::SkColorGlyphClip::produce_path_cached(
                                cache,
                                &path,
                                Some(&commands),
                                FillRule::Winding,
                                item.antialias,
                                tr,
                                dims.0,
                                dims.1,
                                bounds,
                                receiver,
                            );
                            mask = Some(product.mask);
                            product.owner
                        });
                }
                self.install_difference_clip(old_bounds, mask, bw_rect);
            }
            // cpp: skia_renderer/skia_renderer.cc:1100-1116
            CommandKind::kDrawRect
            | CommandKind::kDrawScrollbarTrack
            | CommandKind::kDrawScrollbarThumb
            | CommandKind::kDrawScrollbarButton
            | CommandKind::kDrawScrollbarCorner => {
                assert_eq!(item.blend_mode, PaintBlendMode::kNormal);
                assert!(item.paint_shader.is_none());
                self.drawRect(item.rect, item.color, item.antialias);
            }
            CommandKind::kDrawRoundedRect => self.drawRRect(item),
            CommandKind::kDrawDoubleRoundedRect => self.drawDRRect(item),
            CommandKind::kDrawEllipse => {
                if let Some(commands) = oval_commands(item.rect) {
                    if item.antialias {
                        if let Some(mask) = crate::cpu::analytic_aa::path_spans_with_clip(
                            &commands,
                            false,
                            self.state.transform,
                            self.pixmap.width(),
                            self.pixmap.height(),
                            self.state.clip.as_deref(),
                            self.state.device_origin,
                            self.state.clip_is_aa,
                            self.state.clip_aa_tiles.as_deref().map(|v| v.as_slice()),
                            self.state.clip_summary.as_ref(),
                        ) {
                            self.blend_rounded_spans(&mask, item.color);
                            return;
                        }
                    }
                    if let Some(path) = path_from_commands(&commands) {
                        self.drawPath(&path, item.color, item.antialias, FillRule::Winding);
                    }
                }
            }
            CommandKind::kStrokeEllipse => {
                if let Some(commands) = oval_commands(item.rect) {
                    if let Some(path) = path_from_commands(&commands) {
                        let mut oval = item.clone();
                        oval.path = commands;
                        self.stroke_path(&path, &oval);
                    }
                }
            }
            CommandKind::kStrokeRect => {
                if !item.corner_radii.HasRadius()
                    && item.paint_shader.is_none()
                    && item.blend_mode == PaintBlendMode::kNormal
                    && item.blur_radius == 0.0
                    && item.dash_intervals.is_empty()
                    && item.svg_line_join == SvgStrokeLineJoin::kMiter
                    && item.miter_limit as f32 >= std::f32::consts::SQRT_2
                {
                    if let Some(blits) = crate::cpu::rect_stroke::coverage(
                        item.rect,
                        self.state.transform,
                        item.stroke_width as f32,
                        item.antialias,
                        self.pixmap.width(),
                        self.pixmap.height(),
                    ) {
                        for (i, coverage) in blits {
                            self.blend_coverage(i, item.color, coverage, false);
                        }
                        return;
                    }
                }
                assert_eq!(
                    item.dash_fit_thickness, 0.0,
                    "source fitted RRect dashes require path measurement translation"
                );
                let dashed = matches!(
                    item.line_style,
                    crate::compat::commands::BorderLineStyle::kDashed
                        | crate::compat::commands::BorderLineStyle::kDotted
                );
                let commands = if item.corner_radii.HasRadius() {
                    rounded_rect_commands(item.rect, item.corner_radii, if dashed { 0 } else { 6 })
                } else {
                    rect_commands(item.rect)
                };
                if let Some(commands) = commands {
                    if item.antialias
                        && item.paint_shader.is_none()
                        && item.blend_mode == PaintBlendMode::kNormal
                        && item.blur_radius == 0.0
                        && item.dash_intervals.is_empty()
                        && !item.round_cap
                        && item.svg_line_cap == SvgStrokeLineCap::kButt
                        && crate::cpu::hairline::device_hairline_width(
                            self.state.transform,
                            item.stroke_width as f32,
                        )
                        .is_some()
                        && crate::cpu::hairline::draw_with_clip(
                            &mut self.pixmap,
                            self.f16_surface.as_mut(),
                            self.state.clip.as_deref(),
                            self.state.clip_is_aa,
                            self.state.clip_summary.as_ref(),
                            self.state.transform,
                            &commands,
                            item.color,
                            item.stroke_width as f32,
                        )
                    {
                        return;
                    }
                    if let Some(path) = path_from_commands(&commands) {
                        let mut stroke = item.clone();
                        stroke.path = commands;
                        self.stroke_path(&path, &stroke);
                    }
                }
            }
            CommandKind::kDrawPath => {
                if item.inverse_winding {
                    self.ensure_clip_mask();
                    self.draw_inverse_path(item);
                    return;
                }
                if item.antialias
                    && !item.inverse_winding
                    && item.paint_shader.is_none()
                    && item.blend_mode == PaintBlendMode::kNormal
                {
                    if let Some(mask) = crate::cpu::analytic_aa::path_spans_with_clip(
                        &item.path,
                        item.even_odd,
                        self.state.transform,
                        self.pixmap.width(),
                        self.pixmap.height(),
                        self.state.clip.as_deref(),
                        self.state.device_origin,
                        self.state.clip_is_aa,
                        self.state.clip_aa_tiles.as_deref().map(|v| v.as_slice()),
                        self.state.clip_summary.as_ref(),
                    ) {
                        self.blend_rounded_spans(&mask, item.color);
                        return;
                    }
                }
                if let Some(path) = path_from_commands(&item.path) {
                    self.drawPath(
                        &path,
                        item.color,
                        item.antialias,
                        if item.even_odd {
                            FillRule::EvenOdd
                        } else {
                            FillRule::Winding
                        },
                    );
                }
            }
            CommandKind::kStrokePath => {
                if self.draw_axis_stroke(item) {
                    return;
                }
                if item.antialias
                    && item.paint_shader.is_none()
                    && item.blend_mode == PaintBlendMode::kNormal
                    && item.blur_radius == 0.0
                    && item.dash_intervals.is_empty()
                    && crate::cpu::hairline::device_hairline_width(
                        self.state.transform,
                        item.stroke_width as f32,
                    )
                    .is_some()
                    && crate::cpu::hairline::draw_stroke_with_clip(
                        &mut self.pixmap,
                        self.f16_surface.as_mut(),
                        self.state.clip.as_deref(),
                        self.state.clip_is_aa,
                        self.state.clip_summary.as_ref(),
                        self.state.transform,
                        &item.path,
                        item.color,
                        item.stroke_width as f32,
                        if item.round_cap {
                            SvgStrokeLineCap::kRound
                        } else {
                            item.svg_line_cap
                        },
                    )
                {
                    return;
                }
                if let Some(path) = path_from_commands(&item.path) {
                    self.stroke_path(&path, item);
                }
            }
            CommandKind::kStrokeLine => self.stroke_line(item),
            CommandKind::kDrawGradientRect | CommandKind::kDrawTiledGradient => {
                self.draw_gradient(item)
            }
            CommandKind::kDrawImageRect => self.drawImageRect(item, item.rect, list),
            CommandKind::kDrawTiledImage => self.draw_tiled_image(item, list),
            CommandKind::kDrawGlyphRun => self.draw_glyph_run(item, list),
            CommandKind::kDrawBoxShadow => self.draw_box_shadow(item),
            CommandKind::kDrawMask => self.draw_mask_layers(&item.mask_layers, list),
            other => panic!("pure Rust raster replay for {other:?} is not yet implemented"),
        }
    }

    fn draw_glyph_run(&mut self, item: &DrawCommand, list: &ResourceContext) {
        if item.blur_radius <= 0.0 {
            self.drawGlyphRunList(item, list);
            return;
        }
        // SkMaskFilter::MakeBlur receives sigma = CSS blur radius / 2. A
        // single glyph run filtered through a transparent saveLayer is
        // equivalent to applying that normal blur mask to its SkPaint, and
        // reuses the compositor's bounded blur implementation for tile-safe
        // input expansion.
        let mut layer = DrawCommand::default();
        layer.filters.push(PaintFilterOperation {
            r#type: PaintFilterType::kBlur,
            amount: item.blur_radius * 0.5,
            offset: Offset::default(),
            blur_radius: 0.0,
            color: Color::default(),
        });
        self.save_layer_filter(&layer);
        let mut glyphs = item.clone();
        glyphs.blur_radius = 0.0;
        self.drawGlyphRunList(&glyphs, list);
        self.restore();
    }

    fn save_layer_alpha(&mut self, item: &DrawCommand) {
        // cc::SaveLayerAlphaOp supplies no kF16ColorType flag. SkCanvas's
        // default layer color type follows its prior device: N32 stays N32,
        // while an existing F16 parent retains its actual precision.
        let use_f16 = self.f16_surface.is_some();
        self.internalSaveLayer(item, use_f16);
    }

    fn begin_mask(&mut self, item: &DrawCommand) {
        self.mask_stack.push(item.mask_layers.clone());
        self.internalSaveLayer(&DrawCommand::default(), false);
    }

    fn end_mask(&mut self, list: &ResourceContext) {
        self.end_mask_impl(list, true);
    }

    fn end_mask_impl(&mut self, list: &ResourceContext, fast_mask: bool) {
        let Some(layers) = self.mask_stack.pop() else {
            return;
        };
        self.internalSaveLayer(&DrawCommand::default(), false);
        self.draw_mask_layers(&layers, list);
        self.restore_mask_layer(fast_mask);
        self.restore();
    }

    // BoxPainterBase::PaintMaskImages draws the mask source into its own
    // PaintRecord. The owning effect supplies DstIn separately at restore.
    fn draw_mask_layers(&mut self, layers: &[MaskLayer], list: &ResourceContext) {
        for layer in layers.iter().cloned() {
            self.save();
            self.replay_item(
                &DrawCommand {
                    r#type: if layer.clip_radii.HasRadius() {
                        CommandKind::kClipRoundedRect
                    } else {
                        CommandKind::kClipRect
                    },
                    rect: layer.clip_rect,
                    corner_radii: layer.clip_radii,
                    antialias: layer.clip_radii.HasRadius(),
                    ..Default::default()
                },
                list,
            );
            if let Some(shader) = layer.paint_shader {
                // CSS mask images use the same positioned/repeating image
                // geometry as backgrounds. Preserve the tile phase instead
                // of stretching one gradient across the whole mask clip.
                self.draw_gradient(&DrawCommand {
                    r#type: CommandKind::kDrawTiledGradient,
                    rect: layer.clip_rect,
                    tile_rect: layer.tile_rect,
                    repeat_x: layer.repeat_x,
                    repeat_y: layer.repeat_y,
                    tile_spacing: layer.tile_spacing,
                    paint_shader: Some(shader),
                    ..Default::default()
                });
            } else {
                self.drawRect(
                    layer.clip_rect,
                    Color {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                    false,
                );
            }
            self.restore();
        }
    }

    pub(crate) fn restore_mask_layer(&mut self, fast_mask: bool) {
        // The source restores the mask surface with DstIn, then restores the
        // masked content with SrcOver. Keep these as two separate operations.
        let frame = self.stack.pop().expect("mask surface save");
        let (mut content, _, previous) = frame.layer.expect("mask surface layer");
        debug_assert!(previous.is_none());
        let mask_width = self.pixmap.width() as usize;
        let content_width = content.width() as usize;
        let (left, top) = frame.layer_origin;
        // DstIn affects only the saved parent clip. Outside the cropped mask
        // its source is transparent; parent pixels outside the clip survive.
        if let Some(bounds) = frame
            .state
            .clip_summary
            .as_ref()
            .and_then(|c| c.getBounds())
        {
            if fast_mask && frame.state.clip_summary.is_some_and(|c| c.isRect()) {
                let mask_height = self.pixmap.height() as usize;
                let mask_left = left as usize;
                let mask_top = top as usize;
                let row_left = bounds.left() as usize;
                let row_right = bounds.right() as usize;
                for y in bounds.top() as usize..bounds.bottom() as usize {
                    let start = (y * content_width + row_left) * 4;
                    let row = &mut content.data_mut()[start..start + (row_right - row_left) * 4];
                    if y < mask_top || y >= mask_top + mask_height {
                        row.fill(0);
                        continue;
                    }
                    let first = row_left.max(mask_left).min(row_right);
                    let end = row_right.min(mask_left + mask_width).max(first);
                    row[..(first - row_left) * 4].fill(0);
                    row[(end - row_left) * 4..].fill(0);
                    if first < end {
                        let mask_start = ((y - mask_top) * mask_width + first - mask_left) * 4;
                        mask_dst_in_rgba_span(
                            &mut row[(first - row_left) * 4..(end - row_left) * 4],
                            &self.pixmap.data()[mask_start..mask_start + (end - first) * 4],
                        );
                    }
                }
            } else {
                for y in bounds.top() as usize..bounds.bottom() as usize {
                    for x in bounds.left() as usize..bounds.right() as usize {
                        let i = y * content_width + x;
                        let clip = frame
                            .state
                            .clip
                            .as_deref()
                            .map_or(255, |m| m.alpha_at(x as u32, y as u32));
                        let mask_alpha = if x >= left as usize
                            && y >= top as usize
                            && x < left as usize + mask_width
                            && y < top as usize + self.pixmap.height() as usize
                        {
                            self.pixmap.data()
                                [((y - top as usize) * mask_width + x - left as usize) * 4 + 3]
                        } else {
                            0
                        };
                        let alpha = 255
                            - crate::cpu::mask_blitter::mul_div_255_round(255 - mask_alpha, clip);
                        for channel in &mut content.data_mut()[i * 4..i * 4 + 4] {
                            *channel = crate::cpu::mask_blitter::mul_div_255_round(*channel, alpha);
                        }
                    }
                }
            }
        }
        self.pixmap = content;
        self.state = frame.state;
        self.known_opaque = false;
    }

    fn draw_inverse_path(&mut self, item: &DrawCommand) {
        assert!(item.paint_shader.is_none());
        assert_eq!(item.blend_mode, PaintBlendMode::kNormal);
        let width = self.pixmap.width();
        let height = self.pixmap.height();
        // Rasterize the finite interior without the clip, invert its coverage,
        // then intersect the inverse fill with the current clip exactly once.
        let mut interior = if item.antialias {
            crate::cpu::analytic_aa::path_mask(
                &item.path,
                item.even_odd,
                self.state.transform,
                width,
                height,
                None,
                self.state.device_origin,
                false,
                None,
            )
            .map(|coverage| {
                let mut mask = coverage.mask;
                for (i, alpha, _) in coverage.blits {
                    mask.data_mut()[i] = 255
                        - crate::cpu::mask_blitter::mul_div_255_round(
                            255 - mask
                                .alpha_at((i % width as usize) as u32, (i / width as usize) as u32),
                            255 - alpha,
                        );
                }
                mask
            })
        } else {
            None
        };
        if interior.is_none() {
            let mut mask = Mask::new(width, height).expect("valid inverse fill mask");
            if let Some(path) = path_from_commands(&item.path) {
                mask.fill_path(
                    &path,
                    if item.even_odd {
                        FillRule::EvenOdd
                    } else {
                        FillRule::Winding
                    },
                    item.antialias,
                    self.state.transform,
                );
            }
            interior = Some(mask);
        }
        for (i, &alpha) in interior.unwrap().data().iter().enumerate() {
            if alpha != 255 {
                self.blend_coverage(i, item.color, 255 - alpha, false);
            }
        }
    }

    fn stroke_line(&mut self, item: &DrawCommand) {
        if item.is_text_decoration
            && item.rect.height == 0.0
            && matches!(
                item.decoration_style,
                TextDecorationStyle::kSolid | TextDecorationStyle::kDouble
            )
        {
            let thickness = item.stroke_width.floor().max(1.0);
            let first_y = (item.rect.y + 0.5).floor();
            self.drawRect(
                PaintRect {
                    y: first_y,
                    height: thickness,
                    ..item.rect
                },
                item.color,
                item.antialias,
            );
            if item.decoration_style == TextDecorationStyle::kDouble {
                self.drawRect(
                    PaintRect {
                        y: (item.rect.y + (item.stroke_width + 1.0).floor() + 0.5).floor(),
                        height: thickness,
                        ..item.rect
                    },
                    item.color,
                    item.antialias,
                );
            }
            return;
        }
        let mut builder = PathBuilder::new();
        builder.move_to(item.rect.x as f32, item.rect.y as f32);
        builder.line_to(
            (item.rect.x + item.rect.width) as f32,
            (item.rect.y + item.rect.height) as f32,
        );
        let Some(path) = builder.finish() else {
            return;
        };
        let mut stroke = Stroke {
            width: item.stroke_width as f32,
            line_cap: if item.round_cap || item.svg_line_cap == SvgStrokeLineCap::kRound {
                LineCap::Round
            } else {
                LineCap::Butt
            },
            ..Stroke::default()
        };
        if !item.dash_intervals.is_empty() {
            stroke.dash = StrokeDash::new(
                item.dash_intervals
                    .iter()
                    .map(|value| *value as f32)
                    .collect(),
                item.dash_offset as f32,
            );
        }
        self.ensure_clip_mask();
        let paint = solid_paint(item.color, item.antialias);
        self.pixmap.stroke_path(
            &path,
            &paint,
            &stroke,
            self.state.transform,
            self.state.clip.as_deref(),
        );
    }

    fn draw_axis_stroke(&mut self, item: &DrawCommand) -> bool {
        if !item.antialias
            || item.stroke_width <= 1.0
            || item.blur_radius != 0.0
            || !item.dash_intervals.is_empty()
            || item.paint_shader.is_some()
            || item.blend_mode != PaintBlendMode::kNormal
            || item.path.len() != 2
            || item.path[0].verb != PaintPathVerb::kMoveTo
            || item.path[1].verb != PaintPathVerb::kLineTo
        {
            return false;
        }
        let t = self.state.transform;
        if t.sx != 1.0 || t.sy != 1.0 || t.kx != 0.0 || t.ky != 0.0 {
            return false;
        }
        let a = item.path[0].point;
        let b = item.path[1].point;
        let ax = a.x as f32;
        let ay = a.y as f32;
        let bx = b.x as f32;
        let by = b.y as f32;
        if ax != bx && ay != by {
            return false;
        }
        let r = item.stroke_width as f32 * 0.5;
        let round = item.round_cap || item.svg_line_cap == SvgStrokeLineCap::kRound;
        let extend = round || item.svg_line_cap == SvgStrokeLineCap::kSquare;
        let ex = if ax == bx || extend { r } else { 0.0 };
        let ey = if ay == by || extend { r } else { 0.0 };
        let l = ax.min(bx) - ex;
        let top = ay.min(by) - ey;
        let right = ax.max(bx) + ex;
        let bottom = ay.max(by) + ey;
        if ![l, top, right, bottom].iter().all(|v| v.is_finite()) {
            return false;
        }
        let radius = crate::compat::commands::PaintCornerRadius {
            x: if round { r as f64 } else { 0.0 },
            y: if round { r as f64 } else { 0.0 },
        };
        let shape = DrawCommand {
            rect: PaintRect {
                x: l as f64,
                y: top as f64,
                width: (right - l) as f64,
                height: (bottom - top) as f64,
            },
            corner_radii: PaintCornerRadii {
                top_left: radius,
                top_right: radius,
                bottom_left: radius,
                bottom_right: radius,
            },
            ..item.clone()
        };
        let Some(mask) = crate::cpu::analytic_aa::rounded_rect_mask(
            shape.rect,
            shape.corner_radii,
            t,
            self.pixmap.width(),
            self.pixmap.height(),
            false,
            self.state.clip.as_deref(),
        ) else {
            return false;
        };
        self.blend_analytic_mask(&mask, shape.color);
        true
    }

    /// Finish one Alpha/Add mask gradient on an initially transparent device.
    /// This receiver consumes only coverage. Unsupported shader/tile/CTM inputs
    /// replay the complete existing command before reading the surface alpha.
    pub fn finish_transparent_mask_alpha(
        mut self,
        item: &DrawCommand,
        resources: &ResourceContext,
    ) -> Vec<u8> {
        let same = |a: f64, b: f64| (a - b).abs() <= 1e-6;
        let single_tile = same(item.rect.x, item.tile_rect.x)
            && same(item.rect.y, item.tile_rect.y)
            && same(item.rect.width, item.tile_rect.width)
            && same(item.rect.height, item.tile_rect.height);
        if item.r#type == CommandKind::kDrawTiledGradient
            && single_tile
            && item.blend_mode == PaintBlendMode::kNormal
        {
            if let Some(shader) = item
                .paint_shader
                .as_ref()
                .filter(|shader| shader.kind == PaintShaderKind::kLinearGradient)
            {
                let mut alpha =
                    vec![0; self.pixmap.width() as usize * self.pixmap.height() as usize];
                if crate::cpu::raster_pipeline::draw_linear_gradient_alpha(
                    &mut self.pixmap,
                    self.state.clip.as_deref(),
                    self.state.transform,
                    shader,
                    item.rect,
                    item.tile_rect,
                    self.state.device_origin,
                    self.state.tile_origin,
                    self.state.clip_summary.as_ref(),
                    &mut alpha,
                ) {
                    return alpha;
                }
            }
        }
        if item.r#type == CommandKind::kDrawTiledGradient
            && item.blend_mode == PaintBlendMode::kNormal
            && item
                .paint_shader
                .as_ref()
                .is_some_and(|shader| shader.kind == PaintShaderKind::kLinearGradient)
        {
            self.draw_gradient_on_empty_mask(item);
        } else {
            self.replay_item(item, resources);
        }
        self.finish_direct()
            .into_vec()
            .chunks_exact(4)
            .map(|pixel| pixel[3])
            .collect()
    }

    fn draw_gradient_tile_impl(
        &mut self,
        item: &DrawCommand,
        destination: PaintRect,
        direct: bool,
        mask_surface: bool,
    ) {
        let shader = item
            .paint_shader
            .as_ref()
            .expect("gradient item requires a paint shader");
        assert_eq!(shader.kind, PaintShaderKind::kLinearGradient);
        assert_eq!(item.blend_mode, PaintBlendMode::kNormal);
        let Some(destination_rect) = rect(destination) else {
            return;
        };
        let tr = self.state.transform;
        // Repeated mask tiles can lie entirely outside the clipped device.
        // Use the existing conservative analytic/path support union before
        // constructing their shader, stops, scan path and pipeline blitter.
        if mask_surface
            && tr.kx == 0.0
            && tr.ky == 0.0
            && tr.sx > 0.0
            && tr.sy > 0.0
            && [tr.sx, tr.ky, tr.kx, tr.sy, tr.tx, tr.ty]
                .iter()
                .all(|v| v.is_finite())
            && self.rect_clip_reject_support(destination).is_none()
        {
            return;
        }
        let same = |a: f64, b: f64| (a - b).abs() <= 1e-6;
        let same_tile = same(item.rect.x, item.tile_rect.x)
            && same(item.rect.y, item.tile_rect.y)
            && same(item.rect.width, item.tile_rect.width)
            && same(item.rect.height, item.tile_rect.height);
        let dither = item.r#type != CommandKind::kDrawTiledGradient || same_tile;
        if dither
            && crate::cpu::raster_pipeline::draw_linear_gradient(
                &mut self.pixmap,
                self.state.clip.as_deref(),
                self.state.transform,
                shader,
                destination,
                item.tile_rect,
                self.state.device_origin,
                self.state.tile_origin,
                direct,
                self.state.clip_summary.as_ref(),
            )
        {
            return;
        }
        let mode = match shader.spread {
            PaintSpreadMethod::kPad => SpreadMode::Pad,
            PaintSpreadMethod::kReflect => SpreadMode::Reflect,
            PaintSpreadMethod::kRepeat => SpreadMode::Repeat,
        };
        let transform = Transform::from_row(
            shader.transform.values[0] as f32,
            shader.transform.values[1] as f32,
            shader.transform.values[4] as f32,
            shader.transform.values[5] as f32,
            (shader.transform.values[12] + if direct { 0.0 } else { destination.x }) as f32,
            (shader.transform.values[13] + if direct { 0.0 } else { destination.y }) as f32,
        );
        let start = Point::from_xy(
            (shader.start.x - if direct { 0.0 } else { item.tile_rect.x }) as f32,
            (shader.start.y - if direct { 0.0 } else { item.tile_rect.y }) as f32,
        );
        let end = Point::from_xy(
            (shader.end.x - if direct { 0.0 } else { item.tile_rect.x }) as f32,
            (shader.end.y - if direct { 0.0 } else { item.tile_rect.y }) as f32,
        );
        let stops = shader
            .stops
            .iter()
            .map(|stop| GradientStop::new(stop.offset as f32, color(stop.color)))
            .collect();
        let mut paint = Paint::default();
        let Some(gradient) = LinearGradient::new(start, end, stops, mode, transform) else {
            return;
        };
        paint.shader = gradient;
        paint.anti_alias = item.antialias;
        let uniform_run_drawn = mask_surface
            && start.x == end.x
            && transform.kx == 0.0
            && transform.ky == 0.0
            && self.draw_vertical_mask_gradient_fallback(destination_rect, &paint);
        if mask_surface && std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
            static FIRST_MASK_FALLBACK: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !FIRST_MASK_FALLBACK.swap(true, std::sync::atomic::Ordering::Relaxed) {
                let step_x = item.tile_rect.width + item.tile_spacing.width;
                let step_y = item.tile_rect.height + item.tile_spacing.height;
                let first_x = if item.repeat_x {
                    item.tile_rect.x - ((item.tile_rect.x - item.rect.x) / step_x).ceil() * step_x
                } else {
                    item.tile_rect.x
                };
                let first_y = if item.repeat_y {
                    item.tile_rect.y - ((item.tile_rect.y - item.rect.y) / step_y).ceil() * step_y
                } else {
                    item.tile_rect.y
                };
                let next_x = first_x
                    + if item.repeat_x {
                        step_x
                    } else {
                        item.rect.width + step_x
                    };
                let next_y = first_y
                    + if item.repeat_y {
                        step_y
                    } else {
                        item.rect.height + step_y
                    };
                eprintln!("css-mask-fallback-diagnostic admitted={uniform_run_drawn} empty={mask_surface} clip_rect={} actual_mask={} vertical={} shader_axis={} repeat=({}, {}) same_tile={same_tile} first=({first_x},{first_y}) next=({next_x},{next_y}) end=({},{}) clip={:?} tile={:?} destination={destination:?} ctm={:?} shader_transform={transform:?} device=({}, {})",
                    self.state.clip_summary.is_some_and(|clip| clip.isRect()), self.state.clip.is_some(), start.x == end.x,
                    transform.kx == 0.0 && transform.ky == 0.0, item.repeat_x, item.repeat_y,
                    item.rect.x + item.rect.width, item.rect.y + item.rect.height,
                    item.rect, item.tile_rect, self.state.transform, self.pixmap.width(), self.pixmap.height());
            }
        }
        if uniform_run_drawn {
            return;
        }
        self.ensure_clip_mask();
        self.pixmap.fill_rect(
            destination_rect,
            &paint,
            self.state.transform,
            self.state.clip.as_deref(),
        );
    }

    // A freshly saved mask device is transparent. Under scale+translate a
    // vertical gradient is constant in x. Retain the original geometry scanner,
    // AA run coverage, pipeline recipes, mask stage and device coordinates;
    // shade one pixel per original constant-coverage run, then repeat its bytes.
    fn draw_vertical_mask_gradient_fallback(&mut self, destination: Rect, paint: &Paint) -> bool {
        use crate::raster::geom::{IntRectExt, IntSizeExt};
        let tr = self.state.transform;
        let Some(summary) = self.state.clip_summary else {
            return false;
        };
        if !summary.isRect() && self.state.clip.is_none() {
            return false;
        }
        if tr.kx != 0.0
            || tr.ky != 0.0
            || tr.sx <= 0.0
            || tr.sy <= 0.0
            || ![tr.sx, tr.sy, tr.tx, tr.ty].into_iter().all(f32::is_finite)
            // Larger devices use the fallback's separate shader tile transform.
            || self.pixmap.width() > 4096
            || self.pixmap.height() > 4096
        {
            return false;
        }
        let Some(clip) = summary.getBounds().and_then(|b| b.to_screen_int_rect()) else {
            return true;
        };
        let path = if tr.is_identity() {
            None
        } else {
            let Some(path) = PathBuilder::from_rect(destination).transform(tr) else {
                return false;
            };
            Some(path)
        };
        let mut device_paint = paint.clone();
        if !tr.is_identity() {
            device_paint.shader.transform(tr);
        }
        self.ensure_clip_mask();
        // The full device scan bounds match the original fill_rect/fill_path;
        // the BW clip is applied to its resulting runs, as by the mask stage.
        let device_bounds = self.pixmap.size().to_screen_int_rect(0, 0);
        let mut destination_pixels = self.pixmap.as_mut();
        let mut subpixmap = destination_pixels.as_subpixmap();
        let Some(inner) = crate::raster::pipeline::RasterPipelineBlitter::new(
            &device_paint,
            self.state.clip.as_deref().map(|mask| mask.as_submask()),
            &mut subpixmap,
        ) else {
            return true;
        };
        let mut blitter = VerticalMaskGradientBlitter {
            inner,
            clip,
            mask: self.state.clip.as_deref(),
        };
        match (path.as_ref(), paint.anti_alias) {
            (None, false) => {
                crate::raster::scan::fill_rect(&destination, &device_bounds, &mut blitter)
            }
            (None, true) => {
                crate::raster::scan::fill_rect_aa(&destination, &device_bounds, &mut blitter)
            }
            (Some(path), false) => crate::raster::scan::path::fill_path(
                path,
                FillRule::Winding,
                &device_bounds,
                &mut blitter,
            ),
            (Some(path), true) => crate::raster::scan::path_aa::fill_path(
                path,
                FillRule::Winding,
                &device_bounds,
                &mut blitter,
            ),
        }
        true
    }

    fn draw_gradient(&mut self, item: &DrawCommand) {
        self.draw_gradient_impl(item, false);
    }

    fn draw_gradient_on_empty_mask(&mut self, item: &DrawCommand) {
        self.draw_gradient_impl(item, true);
    }

    fn draw_gradient_impl(&mut self, item: &DrawCommand, mask_surface: bool) {
        let same = |left: f64, right: f64| (left - right).abs() <= 1e-6;
        let single_tile = same(item.rect.x, item.tile_rect.x)
            && same(item.rect.y, item.tile_rect.y)
            && same(item.rect.width, item.tile_rect.width)
            && same(item.rect.height, item.tile_rect.height);
        if item.r#type != CommandKind::kDrawTiledGradient || single_tile {
            self.draw_gradient_tile_impl(item, item.rect, false, mask_surface);
            return;
        }
        let step_x = item.tile_rect.width + item.tile_spacing.width;
        let step_y = item.tile_rect.height + item.tile_spacing.height;
        if step_x <= 0.0 || step_y <= 0.0 {
            return;
        }
        let first_x = if item.repeat_x {
            item.tile_rect.x - ((item.tile_rect.x - item.rect.x) / step_x).ceil() * step_x
        } else {
            item.tile_rect.x
        };
        let first_y = if item.repeat_y {
            item.tile_rect.y - ((item.tile_rect.y - item.rect.y) / step_y).ceil() * step_y
        } else {
            item.tile_rect.y
        };
        // Preserve every original tile iteration. The uniform blitter checks
        // each destination RGBA run as well as actual clip coverage, so a
        // repeated tile blends with earlier pixels through the same pipeline.
        self.save();
        if let Some(bounds) = rect(item.rect) {
            self.clipPath(&PathBuilder::from_rect(bounds), false);
        }
        let mut y = first_y;
        let mut rows = 0;
        while y < item.rect.y + item.rect.height && rows < 10_000 {
            rows += 1;
            let mut x = first_x;
            let mut columns = 0;
            while x < item.rect.x + item.rect.width && columns < 10_000 {
                columns += 1;
                self.draw_gradient_tile_impl(
                    item,
                    PaintRect {
                        x,
                        y,
                        width: item.tile_rect.width,
                        height: item.tile_rect.height,
                    },
                    false,
                    mask_surface,
                );
                x += if item.repeat_x {
                    step_x
                } else {
                    item.rect.width + step_x
                };
            }
            y += if item.repeat_y {
                step_y
            } else {
                item.rect.height + step_y
            };
        }
        self.restore();
    }

    fn draw_tiled_image(&mut self, item: &DrawCommand, list: &ResourceContext) {
        let step_x = item.tile_rect.width + item.tile_spacing.width;
        let step_y = item.tile_rect.height + item.tile_spacing.height;
        if step_x <= 0.0 || step_y <= 0.0 {
            return;
        }
        let first_x = if item.repeat_x {
            item.tile_rect.x - ((item.tile_rect.x - item.rect.x) / step_x).ceil() * step_x
        } else {
            item.tile_rect.x
        };
        let first_y = if item.repeat_y {
            item.tile_rect.y - ((item.tile_rect.y - item.rect.y) / step_y).ceil() * step_y
        } else {
            item.tile_rect.y
        };
        let mut y = first_y;
        let mut rows = 0;
        while y < item.rect.y + item.rect.height && rows < 10_000 {
            rows += 1;
            let mut x = first_x;
            let mut columns = 0;
            while x < item.rect.x + item.rect.width && columns < 10_000 {
                columns += 1;
                self.drawImageRect(
                    item,
                    PaintRect {
                        x,
                        y,
                        width: item.tile_rect.width,
                        height: item.tile_rect.height,
                    },
                    list,
                );
                x += if item.repeat_x {
                    step_x
                } else {
                    item.rect.width + step_x
                };
            }
            y += if item.repeat_y {
                step_y
            } else {
                item.rect.height + step_y
            };
        }
    }

    fn draw_box_shadow(&mut self, item: &DrawCommand) {
        if item.inset {
            if item.blur_radius == 0.0 && self.draw_integer_rectangular_inset_shadow(item) {
                return;
            }
            if item.blur_radius == 0.0 && self.draw_unblurred_inset_shadow_region(item) {
                return;
            }
            self.ensure_clip_mask();
            self.draw_inset_shadow(item);
            return;
        }
        let shadow_box = PaintRect {
            x: item.rect.x - item.spread,
            y: item.rect.y - item.spread,
            width: item.rect.width + item.spread * 2.0,
            height: item.rect.height + item.spread * 2.0,
        };
        let Some(path) = rounded_rect_path(shadow_box, item.corner_radii) else {
            return;
        };
        let mut transform = self.state.transform.pre_concat(Transform::from_translate(
            item.shadow_offset.x as f32,
            item.shadow_offset.y as f32,
        ));
        let sigma = xformed_shadow_sigma((item.blur_radius * 0.5) as f32, transform);
        let outside = rect(shadow_box)
            .and_then(|r| r.transform(transform))
            .is_some_and(|r| {
                r.left() < 0.0
                    || r.top() < 0.0
                    || r.right() > self.pixmap.width() as f32
                    || r.bottom() > self.pixmap.height() as f32
            });
        // SkMaskFilter's output is finite: source bounds plus blur support,
        // intersected with the destination clip. Keep the original tile phase
        // while restricting subsequent A8 work to this conservative support.
        let support = rect(shadow_box)
            .and_then(|r| r.transform(transform))
            .and_then(|r| {
                let border = if sigma.is_finite() && sigma > 0.0 {
                    (sigma * 3.0).ceil() + 2.0
                } else {
                    1.0
                };
                crate::raster::IntRect::from_ltrb(
                    (r.left() - border).floor().max(0.0) as i32,
                    (r.top() - border).floor().max(0.0) as i32,
                    (r.right() + border).ceil().min(self.pixmap.width() as f32) as i32,
                    (r.bottom() + border)
                        .ceil()
                        .min(self.pixmap.height() as f32) as i32,
                )
            });
        let support = if self.state.clip_summary.is_some() || self.state.clip.is_some() {
            support.and_then(|b| {
                self.state
                    .clip_summary
                    .as_ref()
                    .and_then(|c| c.getBounds())
                    .and_then(|c| b.intersect(&c))
            })
        } else {
            support
        };
        let Some(support) = support else {
            return;
        };
        // SkMaskFilter rasterizes the source within the clip expanded by the
        // blur support. Geometry outside the destination still contributes.
        let padding = if outside && sigma > 0.0 && sigma.is_finite() {
            (sigma * 3.0).ceil() as u32 + 1
        } else {
            0
        };
        transform.tx += padding as f32;
        transform.ty += padding as f32;
        let width = self.pixmap.width() + padding * 2;
        let height = self.pixmap.height() + padding * 2;
        if (item.antialias || item.blur_radius > 0.0)
            && item.corner_radii.HasRadius()
            && self.draw_analytic_shadow_region(
                item, shadow_box, transform, sigma, padding, width, height, support,
            )
        {
            return;
        }
        let analytic =
            if (item.antialias || item.blur_radius > 0.0) && item.corner_radii.HasRadius() {
                crate::cpu::analytic_aa::rounded_rect_alpha_mask(
                    shadow_box,
                    item.corner_radii,
                    transform,
                    width,
                    height,
                    false,
                )
            } else {
                None
            };
        let mut mask = analytic.unwrap_or_else(|| {
            let mut mask = Mask::new(width, height).expect("valid shadow mask size");
            mask.fill_path(
                &path,
                FillRule::Winding,
                item.antialias || (item.blur_radius > 0.0 && item.corner_radii.HasRadius()),
                transform,
            );
            mask
        });
        let source_support = rect(shadow_box)
            .and_then(|r| r.transform(transform))
            .and_then(|r| {
                crate::raster::IntRect::from_ltrb(
                    (r.left().floor() - 1.0).max(0.0).min(mask.width() as f32) as i32,
                    (r.top().floor() - 1.0).max(0.0).min(mask.height() as f32) as i32,
                    (r.right().ceil() + 1.0).max(0.0).min(mask.width() as f32) as i32,
                    (r.bottom().ceil() + 1.0).max(0.0).min(mask.height() as f32) as i32,
                )
            });
        blur_alpha_mask_with_bounds(&mut mask, sigma, source_support);
        if padding != 0 {
            let mut cropped = Mask::new(self.pixmap.width(), self.pixmap.height())
                .expect("valid destination shadow mask");
            for y in 0..self.pixmap.height() as usize {
                let start = (y + padding as usize) * mask.width() as usize + padding as usize;
                let width = self.pixmap.width() as usize;
                cropped.data_mut()[y * width..(y + 1) * width]
                    .copy_from_slice(&mask.data()[start..start + width]);
            }
            mask = cropped;
        }
        if !item.inset {
            let mut hole = item.rect;
            let mut radii = item.corner_radii;
            if item.shadow_has_opaque_background {
                hole.x += 1.0;
                hole.y += 1.0;
                hole.width = (hole.width - 2.0).max(0.0);
                hole.height = (hole.height - 2.0).max(0.0);
                for radius in [
                    &mut radii.top_left,
                    &mut radii.top_right,
                    &mut radii.bottom_right,
                    &mut radii.bottom_left,
                ] {
                    radius.x = (radius.x - 1.0).max(0.0);
                    radius.y = (radius.y - 1.0).max(0.0);
                }
            }
            if let Some(path) = rounded_rect_path(hole, radii) {
                let hole_mask = crate::cpu::analytic_aa::rounded_rect_alpha_mask(
                    hole,
                    radii,
                    self.state.transform,
                    mask.width(),
                    mask.height(),
                    true,
                )
                .unwrap_or_else(|| {
                    let mut hole_mask = Mask::new(mask.width(), mask.height())
                        .expect("valid shadow hole mask size");
                    hole_mask.fill_path(&path, FillRule::Winding, true, self.state.transform);
                    hole_mask
                });

                let width = mask.width() as usize;
                for y in support.top() as usize..support.bottom() as usize {
                    let start = y * width + support.left() as usize;
                    let end = y * width + support.right() as usize;
                    intersect_alpha_row(
                        &mut mask.data_mut()[start..end],
                        &hole_mask.data()[start..end],
                        true,
                    );
                }
            }
        }
        // SkBlitMask_opts.h: keep coverage until the A8-to-N32 blit. An
        // intermediate RGBA pixmap sends this through a different src-over
        // rounding path, particularly when several translucent shadows overlap.
        let premultiplied = skia_shadow_premultiplied_color(item.color);
        self.blit_shadow_mask(&mut mask, premultiplied, support);
    }

    // SkDraw::DrawToMask stores the mask's finite bounds independently from
    // the device. Keep scan conversion in the original global tile space;
    // only the A8 storage and its row addresses use a bounded origin.
    fn draw_analytic_shadow_region(
        &mut self,
        item: &DrawCommand,
        shadow_box: PaintRect,
        transform: Transform,
        sigma: f32,
        padding: u32,
        width: u32,
        height: u32,
        support: crate::raster::IntRect,
    ) -> bool {
        let Some(source) = rect(shadow_box)
            .and_then(|r| r.transform(transform))
            .and_then(|r| {
                crate::raster::IntRect::from_ltrb(
                    (r.left().floor() - 1.0).max(0.0).min(width as f32) as i32,
                    (r.top().floor() - 1.0).max(0.0).min(height as f32) as i32,
                    (r.right().ceil() + 1.0).max(0.0).min(width as f32) as i32,
                    (r.bottom().ceil() + 1.0).max(0.0).min(height as f32) as i32,
                )
            })
        else {
            return false;
        };
        // This conservative expansion contains the original PlanGauss support
        // (sigma is clamped to 135 inside the filter), including small Gauss.
        let border = if sigma.is_finite() && sigma > 0.0 {
            (sigma.min(135.0) * 3.0).ceil() as i64 + 2
        } else {
            1
        };
        let Some(region) = crate::raster::IntRect::from_ltrb(
            (i64::from(source.left()) - border).max(0) as i32,
            (i64::from(source.top()) - border).max(0) as i32,
            (i64::from(source.right()) + border).min(i64::from(width)) as i32,
            (i64::from(source.bottom()) + border).min(i64::from(height)) as i32,
        ) else {
            return false;
        };
        let Some(mut mask) = crate::cpu::analytic_aa::rounded_rect_alpha_mask_region(
            shadow_box,
            item.corner_radii,
            transform,
            width,
            height,
            false,
            region,
        ) else {
            return false;
        };
        let local_source = crate::raster::IntRect::from_ltrb(
            source.left() - region.left(),
            source.top() - region.top(),
            source.right() - region.left(),
            source.bottom() - region.top(),
        );
        blur_alpha_mask_with_bounds(&mut mask, sigma, local_source);
        let origin = (
            region.left() - padding as i32,
            region.top() - padding as i32,
        );
        let Some(mask_bounds) = crate::raster::IntRect::from_ltrb(
            origin.0,
            origin.1,
            origin.0 + mask.width() as i32,
            origin.1 + mask.height() as i32,
        ) else {
            return false;
        };
        let Some(bounds) = support.intersect(&mask_bounds) else {
            return true;
        };
        let mut hole = item.rect;
        let mut radii = item.corner_radii;
        if item.shadow_has_opaque_background {
            hole.x += 1.0;
            hole.y += 1.0;
            hole.width = (hole.width - 2.0).max(0.0);
            hole.height = (hole.height - 2.0).max(0.0);
            for radius in [
                &mut radii.top_left,
                &mut radii.top_right,
                &mut radii.bottom_right,
                &mut radii.bottom_left,
            ] {
                radius.x = (radius.x - 1.0).max(0.0);
                radius.y = (radius.y - 1.0).max(0.0);
            }
        }
        // A degenerate hole contributes no clipping, just as the original
        // rounded_rect_path(None) branch. Unsupported geometry uses the old
        // full-mask backend before any destination pixels are touched.
        if rounded_rect_path(hole, radii).is_some() {
            let Some(hole_mask) = crate::cpu::analytic_aa::rounded_rect_alpha_mask_region(
                hole,
                radii,
                self.state.transform,
                self.pixmap.width(),
                self.pixmap.height(),
                true,
                bounds,
            ) else {
                return false;
            };
            let mw = mask.width() as usize;
            let hw = hole_mask.width() as usize;
            for y in bounds.top()..bounds.bottom() {
                let start = (y - origin.1) as usize * mw + (bounds.left() - origin.0) as usize;
                let hs = (y - bounds.top()) as usize * hw;
                intersect_alpha_row(
                    &mut mask.data_mut()[start..start + hw],
                    &hole_mask.data()[hs..hs + hw],
                    true,
                );
            }
        }
        self.blit_shadow_mask_at(
            &mut mask,
            skia_shadow_premultiplied_color(item.color),
            bounds,
            origin,
        );
        true
    }

    fn blit_shadow_mask_at(
        &mut self,
        mask: &mut Mask,
        color: [u8; 4],
        bounds: crate::raster::IntRect,
        origin: (i32, i32),
    ) {
        let width = self.pixmap.width() as usize;
        let mask_width = mask.width() as usize;
        let count = bounds.width() as usize;
        let blitter = crate::src::opts::SkBlitMask_opts::PreparedA8Blitter::new_with_format(
            color,
            self.pixmap.format,
        );
        for y in bounds.top()..bounds.bottom() {
            let start = y as usize * width + bounds.left() as usize;
            let ms = (y - origin.1) as usize * mask_width + (bounds.left() - origin.0) as usize;
            let row = &mut mask.data_mut()[ms..ms + count];
            if let Some(clip) = self
                .state
                .clip
                .as_deref()
                .filter(|_| !self.state.clip_summary.is_some_and(|c| c.isRect()))
            {
                intersect_alpha_row(
                    row,
                    clip.row_range(y as u32, bounds.left() as u32, bounds.right() as u32),
                    false,
                );
            }
            let left = crate::cpu::analytic_aa::equal_byte_run_end(row, 0, 0);
            if left == row.len() {
                continue;
            }
            let mut right = row.len();
            while right >= left + 8 {
                // SAFETY: the complete initialized eight-byte word is inside
                // this bounded row. Zero coverage contributes no color.
                if unsafe { core::ptr::read_unaligned(row.as_ptr().add(right - 8).cast::<u64>()) }
                    != 0
                {
                    break;
                }
                right -= 8;
            }
            while right > left && row[right - 1] == 0 {
                right -= 1;
            }
            blitter.blend_row(
                &mut self.pixmap.data_mut()[(start + left) * 4..(start + right) * 4],
                &row[left..right],
            );
        }
    }

    fn blit_shadow_mask(
        &mut self,
        mask: &mut Mask,
        color: [u8; 4],
        bounds: crate::raster::IntRect,
    ) {
        self.blit_shadow_mask_at(mask, color, bounds, (0, 0));
    }

    // Blink clips the complement of the inset hole to the original rounded
    // box. At sigma zero its support is exactly that box. SkDraw::DrawToMask
    // keeps finite A8 bounds; preserve the original global analytic tile phase
    // and rounded /255 coverage while avoiding device-sized mask storage.
    // Blink PaintInsetBoxShadow clips a translated casting-hole complement.
    // For N32, zero blur/radii, nonnegative spread and exact integer device
    // edges, both masks are binary rectangles. Shade only their difference,
    // preserving the original source color/blend and all unsupported fallbacks.
    fn draw_integer_rectangular_inset_shadow(&mut self, item: &DrawCommand) -> bool {
        let tr = self.state.transform;
        if !item.inset
            || item.blur_radius != 0.0
            || item.corner_radii.HasRadius()
            || self.f16_surface.is_some()
            || tr.kx != 0.0
            || tr.ky != 0.0
            || tr.sx <= 0.0
            || tr.sy <= 0.0
            || ![tr.sx, tr.sy, tr.tx, tr.ty].into_iter().all(f32::is_finite)
            || !item.spread.is_finite()
            || item.spread < 0.0
            || !item.shadow_offset.x.is_finite()
            || !item.shadow_offset.y.is_finite()
            || self.state.clip_summary.is_some_and(|c| !c.isRect())
            || (self.state.clip.is_some() && self.state.clip_summary.is_none())
        {
            return false;
        }
        // Use the original casting-path construction and point transform, avoiding
        // f32 reassociation of right/bottom or shadow-offset addition.
        let integer_bounds = |box_rect: PaintRect, ts: Transform| {
            let p = rounded_rect_path(box_rect, PaintCornerRadii::default())?.transform(ts)?;
            let b = p.bounds();
            let v = [b.left(), b.top(), b.right(), b.bottom()];
            if v.iter().any(|&x| {
                !x.is_finite()
                    || x.fract() != 0.0
                    || f64::from(x) < f64::from(i32::MIN)
                    || f64::from(x) > f64::from(i32::MAX)
            }) {
                return None;
            }
            crate::raster::IntRect::from_ltrb(v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32)
        };
        let Some(outer) = integer_bounds(item.rect, tr) else {
            return false;
        };
        let hole_rect = PaintRect {
            x: item.rect.x + item.spread,
            y: item.rect.y + item.spread,
            width: (item.rect.width - 2.0 * item.spread).max(0.0),
            height: (item.rect.height - 2.0 * item.spread).max(0.0),
        };
        let hole = if hole_rect.width > 0.0 && hole_rect.height > 0.0 {
            let hole_tr = tr.pre_concat(Transform::from_translate(
                item.shadow_offset.x as f32,
                item.shadow_offset.y as f32,
            ));
            let Some(hole) = integer_bounds(hole_rect, hole_tr) else {
                return false;
            };
            Some(hole)
        } else {
            None
        };
        let Some(device) =
            crate::raster::IntRect::from_xywh(0, 0, self.pixmap.width(), self.pixmap.height())
        else {
            return false;
        };
        let region = if let Some(summary) = self.state.clip_summary {
            summary
                .getBounds()
                .and_then(|c| c.intersect(&device))
                .and_then(|c| c.intersect(&outer))
        } else {
            outer.intersect(&device)
        };
        let Some(region) = region else {
            return true;
        };
        let hole = hole.and_then(|h| h.intersect(&region));
        let color = skia_shadow_premultiplied_color(item.color);
        let format = self.pixmap.format;
        let stride = self.pixmap.width() as usize;
        let mut shade = |l: i32, t: i32, r: i32, b: i32| {
            if l >= r || t >= b {
                return;
            }
            for y in t..b {
                let start = (y as usize * stride + l as usize) * 4;
                let end = start + (r - l) as usize * 4;
                crate::src::core::SkBlitRow_D32::blend_span_format(
                    &mut self.pixmap.data_mut()[start..end],
                    color,
                    255,
                    false,
                    format,
                );
            }
        };
        if let Some(hole) = hole {
            shade(region.left(), region.top(), region.right(), hole.top());
            shade(
                region.left(),
                hole.bottom(),
                region.right(),
                region.bottom(),
            );
            shade(region.left(), hole.top(), hole.left(), hole.bottom());
            shade(hole.right(), hole.top(), region.right(), hole.bottom());
        } else {
            shade(region.left(), region.top(), region.right(), region.bottom());
        }
        true
    }

    fn draw_unblurred_inset_shadow_region(&mut self, item: &DrawCommand) -> bool {
        let transform = self.state.transform;
        if transform.kx != 0.0
            || transform.ky != 0.0
            || !transform.sx.is_finite()
            || !transform.sy.is_finite()
            || transform.sx <= 0.0
            || transform.sy <= 0.0
            || !transform.tx.is_finite()
            || !transform.ty.is_finite()
        {
            return false;
        }
        // A rectangular summary needs no materialized full-device A8 clip.
        // Other summaries keep the established fallback if coverage is absent.
        if self.state.clip.is_none() && self.state.clip_summary.is_some_and(|c| !c.isRect()) {
            return false;
        }
        let Some(region) = self.analytic_clip_support(item.rect) else {
            return true;
        };
        let width = self.pixmap.width();
        let height = self.pixmap.height();
        let Some(clip) = crate::cpu::analytic_aa::rounded_rect_alpha_mask_region(
            item.rect,
            item.corner_radii,
            transform,
            width,
            height,
            true,
            region,
        ) else {
            return false;
        };
        let hole = PaintRect {
            x: item.rect.x + item.spread,
            y: item.rect.y + item.spread,
            width: (item.rect.width - 2.0 * item.spread).max(0.0),
            height: (item.rect.height - 2.0 * item.spread).max(0.0),
        };
        let mut radii = item.corner_radii;
        for radius in [
            &mut radii.top_left,
            &mut radii.top_right,
            &mut radii.bottom_right,
            &mut radii.bottom_left,
        ] {
            radius.x = (radius.x - item.spread).max(0.0);
            radius.y = (radius.y - item.spread).max(0.0);
        }
        let interior = if hole.width > 0.0 && hole.height > 0.0 {
            let hole_transform = transform.pre_concat(Transform::from_translate(
                item.shadow_offset.x as f32,
                item.shadow_offset.y as f32,
            ));
            let Some(mask) = crate::cpu::analytic_aa::rounded_rect_alpha_mask_region(
                hole,
                radii,
                hole_transform,
                width,
                height,
                true,
                region,
            ) else {
                return false;
            };
            Some(mask)
        } else {
            None
        };
        // All fallible scan conversion finishes before any destination write.
        let color = skia_shadow_premultiplied_color(item.color);
        let format = self.pixmap.format;
        let row_width = region.width() as usize;
        for y in region.top()..region.bottom() {
            let local_start = (y - region.top()) as usize * row_width;
            let device_start = y as usize * width as usize + region.left() as usize;
            for x in 0..row_width {
                let alpha = clip.data()[local_start + x];
                if alpha == 0 {
                    continue;
                }
                let source = interior
                    .as_ref()
                    .map_or(255, |mask| 255 - mask.data()[local_start + x]);
                let mut coverage = crate::cpu::mask_blitter::mul_div_255_round(source, alpha);
                let i = device_start + x;
                if let Some(mask) = &self.state.clip {
                    coverage = crate::cpu::mask_blitter::mul_div_255_round(
                        coverage,
                        mask.alpha_at((i % width as usize) as u32, (i / width as usize) as u32),
                    );
                }
                crate::cpu::mask_blitter::blend_prepared_format(
                    &mut self.pixmap.data_mut()[i * 4..i * 4 + 4],
                    color,
                    coverage,
                    false,
                    format,
                );
            }
        }
        true
    }
    fn draw_inset_shadow(&mut self, item: &DrawCommand) {
        // Blink BoxPainterBase::PaintInsetBoxShadow casts the complement of
        // an inset hole, then clips the result to the original rounded box.
        let hole = PaintRect {
            x: item.rect.x + item.spread,
            y: item.rect.y + item.spread,
            width: (item.rect.width - 2.0 * item.spread).max(0.0),
            height: (item.rect.height - 2.0 * item.spread).max(0.0),
        };
        let mut radii = item.corner_radii;
        for radius in [
            &mut radii.top_left,
            &mut radii.top_right,
            &mut radii.bottom_right,
            &mut radii.bottom_left,
        ] {
            radius.x = (radius.x - item.spread).max(0.0);
            radius.y = (radius.y - item.spread).max(0.0);
        }
        let width = self.pixmap.width();
        let height = self.pixmap.height();
        let sigma = xformed_shadow_sigma((item.blur_radius * 0.5) as f32, self.state.transform);
        let padding = if sigma > 0.0 {
            (sigma * 3.0).ceil() as u32 + 1
        } else {
            0
        };
        let mut transform = self.state.transform.pre_concat(Transform::from_translate(
            item.shadow_offset.x as f32,
            item.shadow_offset.y as f32,
        ));
        transform.tx += padding as f32;
        transform.ty += padding as f32;
        let mut interior =
            Mask::new(width + 2 * padding, height + 2 * padding).expect("valid inset shadow mask");
        if hole.width > 0.0 && hole.height > 0.0 {
            if let Some(coverage) = crate::cpu::analytic_aa::rounded_rect_mask(
                hole,
                radii,
                transform,
                interior.width(),
                interior.height(),
                true,
                None,
            ) {
                interior = coverage.mask;
            } else if let Some(path) = rounded_rect_path(hole, radii) {
                interior.fill_path(&path, FillRule::Winding, true, transform);
            }
        }
        for alpha in interior.data_mut() {
            *alpha = 255 - *alpha;
        }
        blur_alpha_mask(&mut interior, sigma);
        let mut clip = Mask::new(width, height).expect("valid inset shadow clip");
        if let Some(coverage) = crate::cpu::analytic_aa::rounded_rect_mask(
            item.rect,
            item.corner_radii,
            self.state.transform,
            width,
            height,
            true,
            None,
        ) {
            clip = coverage.mask;
        } else if let Some(path) = rounded_rect_path(item.rect, item.corner_radii) {
            clip.fill_path(&path, FillRule::Winding, true, self.state.transform);
        }
        let color = skia_shadow_premultiplied_color(item.color);
        let format = self.pixmap.format;
        for (i, &alpha) in clip.data().iter().enumerate() {
            if alpha == 0 {
                continue;
            }
            let x = i % width as usize;
            let y = i / width as usize;
            let source = interior.data()
                [(y + padding as usize) * interior.width() as usize + x + padding as usize];
            let mut coverage = crate::cpu::mask_blitter::mul_div_255_round(source, alpha);
            if let Some(clip) = &self.state.clip {
                coverage = crate::cpu::mask_blitter::mul_div_255_round(
                    coverage,
                    clip.alpha_at(x as u32, y as u32),
                );
            }
            crate::cpu::mask_blitter::blend_prepared_format(
                &mut self.pixmap.data_mut()[i * 4..i * 4 + 4],
                color,
                coverage,
                false,
                format,
            );
        }
    }
}
// Alpha intersection from SkAAClip/SkAlphaRuns: constant empty/full runs
// reduce to zero/identity. Word loads only skip those exact integer cases;
// fractional coverage retains the original rounded division by 255.
fn intersect_alpha_row(dst: &mut [u8], mask: &[u8], invert: bool) {
    assert_eq!(dst.len(), mask.len());
    let (identity, empty) = if invert {
        (0u64, u64::MAX)
    } else {
        (u64::MAX, 0u64)
    };
    let mut i = 0;
    while i + 8 <= dst.len() {
        // SAFETY: eight initialized mask bytes lie in the checked row.
        let word = unsafe { core::ptr::read_unaligned(mask.as_ptr().add(i).cast::<u64>()) };
        if word == empty || word == identity {
            let end = crate::cpu::analytic_aa::equal_byte_run_end(mask, i, mask[i]);
            if word == empty {
                dst[i..end].fill(0);
            }
            i = end;
            continue;
        } else {
            for j in i..i + 8 {
                let coverage = if invert { 255 - mask[j] } else { mask[j] };
                dst[j] = ((u16::from(dst[j]) * u16::from(coverage) + 127) / 255) as u8;
            }
        }
        i += 8;
    }
    while i < dst.len() {
        let coverage = if invert { 255 - mask[i] } else { mask[i] };
        dst[i] = ((u16::from(dst[i]) * u16::from(coverage) + 127) / 255) as u8;
        i += 1;
    }
}

// SkBlurMaskFilterImpl::computeXformedSigma uses SkMatrix::mapRadius,
// the geometric mean of the mapped axis-vector lengths, capped at 128.
// The mask and its blur support must both use this device-space sigma.
fn xformed_shadow_sigma(sigma: f32, transform: Transform) -> f32 {
    if sigma <= 0.0 || !sigma.is_finite() {
        return sigma;
    }
    let d0 = Point::from_xy(transform.sx * sigma, transform.ky * sigma).length();
    let d1 = Point::from_xy(transform.kx * sigma, transform.sy * sigma).length();
    let mapped = (d0 * d1).sqrt();
    if mapped.is_nan() {
        mapped
    } else {
        mapped.min(128.0)
    }
}

pub(crate) fn source_rgba_readback(premultiplied: &[u8]) -> Vec<u8> {
    use crate::src::core::SkConvertPixels::{Rgba8888AlphaType, SkConvertPixels};
    let mut output = vec![0; premultiplied.len()];
    assert!(SkConvertPixels(
        &mut output,
        premultiplied,
        Rgba8888AlphaType::Premul,
        Rgba8888AlphaType::Unpremul
    ));
    output
}
#[cfg(feature = "profiling")]
pub fn ProfileConstantMaskBlit(
    color: Color,
    width: u32,
    height: u32,
) -> Vec<(&'static str, std::time::Duration, bool)> {
    use std::time::Instant;
    assert_eq!(color.alpha, 1.0);
    let mut baseline = vec![255; (width * height) as usize * 4];
    let started = Instant::now();
    for pixel in baseline.chunks_exact_mut(4) {
        crate::cpu::mask_blitter::blend_mask(pixel, color, 255);
    }
    let baseline_time = started.elapsed();
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8;
    let source = [byte(color.red), byte(color.green), byte(color.blue), 255];
    let mut prepared = vec![255; baseline.len()];
    let started = Instant::now();
    for pixel in prepared.chunks_exact_mut(4) {
        crate::cpu::mask_blitter::blend_premultiplied(pixel, source, 255);
    }
    let prepared_time = started.elapsed();
    let mut row = Vec::with_capacity(width as usize * 4);
    for _ in 0..width {
        row.extend_from_slice(&source);
    }
    let mut spans = vec![255; baseline.len()];
    let started = Instant::now();
    for destination in spans.chunks_exact_mut(row.len()) {
        destination.copy_from_slice(&row);
    }
    let span_time = started.elapsed();
    vec![
        ("original_per_pixel", baseline_time, true),
        ("prepared_color", prepared_time, prepared == baseline),
        ("opaque_row_copy", span_time, spans == baseline),
    ]
}
#[cfg(feature = "profiling")]
pub fn ProfileRoundedRectAttempt(
    rect: PaintRect,
    radii: crate::compat::commands::PaintCornerRadii,
    width: u32,
    height: u32,
    scale: f64,
) -> (std::time::Duration, bool) {
    let mut clip = crate::raster::Mask::new(width, height).unwrap();
    clip.data_mut().fill(255);
    let started = std::time::Instant::now();
    let result = crate::cpu::analytic_aa::rounded_rect_mask(
        rect,
        radii,
        Transform::from_scale(scale as f32, scale as f32),
        width,
        height,
        false,
        Some(&clip),
    );
    (started.elapsed(), result.is_none())
}

#[cfg(feature = "profiling")]
pub fn outline_scan_cache_profile() -> (usize, usize, u64, u64, u64) {
    crate::compat::glyph_rasters::profile_stats()
}

#[cfg(test)]
mod bounded_clip_replay_tests {
    use super::*;
    use crate::src::core::SkRasterClip::SkRasterClip;

    #[test]
    fn nested_clip_replay_preserves_dense_reference_and_exact_bounds() {
        let (width, height) = (520, 360);
        let resources = ResourceContext::default();
        let radius = PaintCornerRadius { x: 9.25, y: 13.5 };
        for offset in [-7.125, 0.0, 0.125, 0.875, 250.125, 254.875, 510.75] {
            let mut canvas = SkCanvas::new(&resources, width, height);
            let mut previous: Option<Mask> = None;
            let commands = [
                DrawCommand {
                    r#type: CommandKind::kClipRoundedRect,
                    rect: PaintRect {
                        x: offset,
                        y: 247.125,
                        width: 61.25,
                        height: 37.75,
                    },
                    corner_radii: PaintCornerRadii {
                        top_left: radius,
                        top_right: radius,
                        bottom_left: radius,
                        bottom_right: radius,
                    },
                    antialias: true,
                    ..Default::default()
                },
                DrawCommand {
                    r#type: CommandKind::kClipRoundedRect,
                    rect: PaintRect {
                        x: offset + 2.75,
                        y: 241.75,
                        width: 52.875,
                        height: 42.0,
                    },
                    corner_radii: PaintCornerRadii {
                        top_left: radius,
                        top_right: radius,
                        bottom_left: radius,
                        bottom_right: radius,
                    },
                    antialias: true,
                    ..Default::default()
                },
                DrawCommand {
                    r#type: CommandKind::kClipRect,
                    rect: PaintRect {
                        x: offset + 4.0,
                        y: 253.0,
                        width: 41.0,
                        height: 60.0,
                    },
                    antialias: false,
                    ..Default::default()
                },
                DrawCommand {
                    r#type: CommandKind::kClipRect,
                    rect: PaintRect {
                        x: offset + 9.25,
                        y: 251.25,
                        width: 39.75,
                        height: 20.125,
                    },
                    antialias: true,
                    ..Default::default()
                },
            ];
            for command in commands {
                let mut reference = if command.antialias {
                    crate::cpu::analytic_aa::rounded_rect_mask(
                        command.rect,
                        command.corner_radii,
                        Transform::identity(),
                        width,
                        height,
                        true,
                        previous.as_ref(),
                    )
                    .unwrap()
                    .mask
                } else {
                    let mut result = previous.clone().unwrap_or_else(|| {
                        let mut mask = Mask::new(width, height).unwrap();
                        mask.data_mut().fill(255);
                        mask
                    });
                    let left = (command.rect.x + 0.5).floor().max(0.0) as u32;
                    let top = (command.rect.y + 0.5).floor().max(0.0) as u32;
                    let right = (command.rect.x + command.rect.width + 0.5).floor().max(0.0) as u32;
                    let bottom = (command.rect.y + command.rect.height + 0.5)
                        .floor()
                        .max(0.0) as u32;
                    for y in 0..height {
                        for x in 0..width {
                            if x < left || x >= right || y < top || y >= bottom {
                                result.data_mut()[(y * width + x) as usize] = 0;
                            }
                        }
                    }
                    result
                };
                if command.antialias {
                    if let Some(old) = &previous {
                        for (a, &b) in reference.data_mut().iter_mut().zip(old.data()) {
                            *a = crate::cpu::mask_blitter::mul_div_255_round(*a, b);
                        }
                    }
                }
                canvas.replay_item(&command, &resources);
                canvas.ensure_clip_mask();
                let actual = canvas.state.clip.as_deref().unwrap();
                assert_eq!(
                    actual.data(),
                    reference.data(),
                    "offset={offset}, type={:?}",
                    command.r#type
                );
                assert_eq!(
                    canvas.state.clip_summary,
                    SkRasterClip::from_mask(width, height, actual.data())
                );
                previous = Some(reference);
            }
        }
    }
}

#[cfg(test)]
mod inset_shadow_round2_tests {
    use super::*;
    fn shadow() -> DrawCommand {
        let radius = PaintCornerRadius {
            x: 14.40000021,
            y: 14.40000021,
        };
        DrawCommand {
            r#type: CommandKind::kDrawBoxShadow,
            rect: PaintRect {
                x: 411.0,
                y: 19.0,
                width: 304.0,
                height: 31.0,
            },
            corner_radii: PaintCornerRadii {
                top_left: radius,
                top_right: radius,
                bottom_left: radius,
                bottom_right: radius,
            },
            inset: true,
            spread: 0.16000000238,
            color: Color {
                alpha: 0.35,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    #[test]
    fn bounded_inset_shadow_matches_full_device_coverage() {
        let resources = ResourceContext::default();
        for format in [
            crate::PixelFormat::Rgba8888,
            crate::PixelFormat::Bgra8888,
            crate::PixelFormat::Bgrx8888,
        ] {
            for scale in [0.8, 1.0, 2.0] {
                for (x, y) in [(-13.25, -7.5), (17.3, 9.7), (113.25, 77.5)] {
                    for spread in [-3.25, 0.16000000238, 4.0, 40.0] {
                        for offset in [Offset::default(), Offset { x: -2.3, y: 1.7 }] {
                            for clip_kind in 0..4 {
                                let mut command = shadow();
                                command.rect = PaintRect {
                                    x,
                                    y,
                                    width: 53.7,
                                    height: 27.25,
                                };
                                command.spread = spread;
                                command.shadow_offset = offset;
                                let render = |bounded: bool| {
                                    let mut canvas = SkCanvas::new(&resources, 137, 93);
                                    canvas.pixmap.format = format;
                                    if format == crate::PixelFormat::Bgrx8888 {
                                        for pixel in canvas.pixmap.data_mut().chunks_exact_mut(4) {
                                            pixel[3] = 0;
                                        }
                                    }
                                    canvas.set_scale(scale);
                                    if clip_kind != 0 {
                                        canvas.replay_item(
                                            &DrawCommand {
                                                r#type: if clip_kind == 3 {
                                                    CommandKind::kClipRoundedRect
                                                } else {
                                                    CommandKind::kClipRect
                                                },
                                                rect: PaintRect {
                                                    x: 5.25,
                                                    y: 3.75,
                                                    width: 70.5,
                                                    height: 63.25,
                                                },
                                                corner_radii: command.corner_radii,
                                                antialias: clip_kind >= 2,
                                                ..Default::default()
                                            },
                                            &resources,
                                        );
                                    }
                                    if bounded {
                                        canvas.draw_box_shadow(&command);
                                    } else {
                                        canvas.ensure_clip_mask();
                                        canvas.draw_inset_shadow(&command);
                                    }
                                    canvas.pixmap.data().to_vec()
                                };
                                assert_eq!(render(true), render(false), "format={format:?} scale={scale} x={x} y={y} spread={spread} offset={offset:?} clip={clip_kind}");
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    #[ignore = "manual Debug inset shadow timing, no speed assertion"]
    fn profile_inset_shadow_round2() {
        let resources = ResourceContext::default();
        for iteration in 0..5 {
            let mut canvas = SkCanvas::new(&resources, 2560, 1446);
            canvas.set_scale(2.0);
            let start = std::time::Instant::now();
            canvas.draw_box_shadow(&shadow());
            let elapsed = start.elapsed();
            let hash = canvas
                .pixmap
                .data()
                .iter()
                .fold(0xcbf29ce484222325u64, |h, &b| {
                    (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
                });
            eprintln!(
                "inset-round2 iteration={iteration} ms={:.3} hash={hash:016x}",
                elapsed.as_secs_f64() * 1000.0
            );
            std::hint::black_box(canvas);
        }
    }
}

#[cfg(test)]
mod empty_clip_round4_tests {
    use super::*;
    use crate::raster::PixelFormat;

    fn dense_clip(canvas: &mut SkCanvas, command: &DrawCommand) {
        let coverage = crate::cpu::analytic_aa::rounded_rect_clip_mask(
            command.rect,
            command.corner_radii,
            canvas.state.transform,
            canvas.pixmap.width(),
            canvas.pixmap.height(),
            canvas.state.clip.as_deref(),
            canvas.state.clip_summary.as_ref(),
        )
        .unwrap();
        let support = canvas.analytic_clip_support(command.rect);
        canvas.install_analytic_clip(coverage, support, true);
    }
    fn clip(x: f64, y: f64) -> DrawCommand {
        let radius = PaintCornerRadius { x: 5.5, y: 3.75 };
        DrawCommand {
            r#type: CommandKind::kClipRoundedRect,
            rect: PaintRect {
                x,
                y,
                width: 37.25,
                height: 27.5,
            },
            corner_radii: PaintCornerRadii {
                top_left: radius,
                top_right: radius,
                bottom_left: radius,
                bottom_right: radius,
            },
            antialias: true,
            ..Default::default()
        }
    }
    #[test]
    fn empty_rect_clip_matches_dense_coverage_and_survives_layer_restore() {
        let list = ResourceContext::default();
        for scale in [0.75, 1.0, 2.0] {
            for x in [-700.0, -11.75, 201.25, 1000.0] {
                for y in [-500.0, -7.5, 121.5, 1000.0] {
                    for format in [
                        PixelFormat::Rgba8888,
                        PixelFormat::Bgra8888,
                        PixelFormat::Bgrx8888,
                    ] {
                        let mut old = SkCanvas::new(&list, 520, 360);
                        let mut new = SkCanvas::new(&list, 520, 360);
                        for canvas in [&mut old, &mut new] {
                            canvas.set_scale(scale);
                            canvas.pixmap.format = format;
                            for p in canvas.pixmap.data_mut().chunks_exact_mut(4) {
                                p.copy_from_slice(&format.encode([49, 81, 113, 255]));
                            }
                            canvas.save();
                        }
                        let command = clip(x, y);
                        dense_clip(&mut old, &command);
                        new.replay_item(&command, &list);
                        assert_eq!(
                            new.state.clip_summary.unwrap().getBounds(),
                            old.state.clip_summary.unwrap().getBounds()
                        );
                        if old.state.clip_summary.unwrap().getBounds().is_none() {
                            assert!(new.state.clip.is_none(), "empty clip must not allocate A8");
                            for canvas in [&mut old, &mut new] {
                                canvas.internalSaveLayer(
                                    &DrawCommand {
                                        opacity: 0.73,
                                        ..Default::default()
                                    },
                                    true,
                                );
                                canvas.replay_item(
                                    &DrawCommand {
                                        r#type: CommandKind::kDrawRoundedRect,
                                        rect: PaintRect {
                                            x: 0.0,
                                            y: 0.0,
                                            width: 500.0,
                                            height: 350.0,
                                        },
                                        color: Color {
                                            red: 1.0,
                                            green: 0.0,
                                            blue: 0.0,
                                            alpha: 1.0,
                                        },
                                        antialias: true,
                                        corner_radii: command.corner_radii,
                                        ..Default::default()
                                    },
                                    &list,
                                );
                                canvas.restore();
                            }
                        }
                        new.ensure_clip_mask();
                        assert_eq!(
                            new.state.clip.as_ref().unwrap().data(),
                            old.state.clip.as_ref().unwrap().data(),
                            "scale={scale} x={x} y={y}"
                        );
                        old.restore();
                        new.restore();
                        for canvas in [&mut old, &mut new] {
                            canvas.replay_item(
                                &DrawCommand {
                                    r#type: CommandKind::kDrawRect,
                                    rect: PaintRect {
                                        x: 3.0,
                                        y: 4.0,
                                        width: 5.0,
                                        height: 6.0,
                                    },
                                    color: Color {
                                        red: 0.0,
                                        green: 0.0,
                                        blue: 1.0,
                                        alpha: 1.0,
                                    },
                                    ..Default::default()
                                },
                                &list,
                            );
                        }
                        assert_eq!(old.pixmap.data(), new.pixmap.data());
                    }
                }
            }
        }
    }
    #[test]
    fn disjoint_clip_inside_device_preserves_parent_and_empty_draws() {
        let resources = ResourceContext::default();
        for aa in [false, true] {
            for rounded in [false, true] {
                for format in [
                    PixelFormat::Rgba8888,
                    PixelFormat::Bgra8888,
                    PixelFormat::Bgrx8888,
                ] {
                    let mut reference = SkCanvas::new(&resources, 128, 96);
                    let mut actual = SkCanvas::new(&resources, 128, 96);
                    let background = format.encode([49, 81, 113, 255]);
                    for canvas in [&mut reference, &mut actual] {
                        canvas.pixmap.format = format;
                        for pixel in canvas.pixmap.data_mut().chunks_exact_mut(4) {
                            pixel.copy_from_slice(&background);
                        }
                        canvas.replay_item(
                            &DrawCommand {
                                r#type: if rounded {
                                    CommandKind::kClipRoundedRect
                                } else {
                                    CommandKind::kClipRect
                                },
                                antialias: aa,
                                ..clip(5.25, 5.75)
                            },
                            &resources,
                        );
                        canvas.save();
                    }
                    let disjoint = clip(80.25, 60.75);
                    dense_clip(&mut reference, &disjoint);
                    actual.replay_item(&disjoint, &resources);
                    assert!(reference.state.clip_summary.unwrap().getBounds().is_none());
                    assert!(actual.state.clip_summary.unwrap().getBounds().is_none());
                    assert!(actual.state.clip.is_none());
                    for canvas in [&mut reference, &mut actual] {
                        canvas.replay_item(
                            &DrawCommand {
                                r#type: CommandKind::kSaveLayerAlpha,
                                opacity: 0.73,
                                ..Default::default()
                            },
                            &resources,
                        );
                        canvas.replay_item(
                            &DrawCommand {
                                r#type: CommandKind::kDrawRect,
                                rect: PaintRect {
                                    x: 0.0,
                                    y: 0.0,
                                    width: 128.0,
                                    height: 96.0,
                                },
                                color: Color {
                                    red: 1.0,
                                    alpha: 1.0,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            &resources,
                        );
                        canvas.replay_item(
                            &DrawCommand {
                                r#type: CommandKind::kRestore,
                                ..Default::default()
                            },
                            &resources,
                        );
                        assert!(canvas
                            .pixmap
                            .data()
                            .chunks_exact(4)
                            .all(|pixel| pixel == background));
                        canvas.restore();
                        canvas.replay_item(
                            &DrawCommand {
                                r#type: CommandKind::kDrawRect,
                                rect: PaintRect {
                                    x: 0.0,
                                    y: 0.0,
                                    width: 128.0,
                                    height: 96.0,
                                },
                                color: Color {
                                    blue: 1.0,
                                    alpha: 1.0,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            &resources,
                        );
                    }
                    assert_eq!(reference.pixmap.data(), actual.pixmap.data());
                    assert!(actual
                        .pixmap
                        .data()
                        .chunks_exact(4)
                        .any(|pixel| pixel != background));
                }
            }
        }
    }
    #[test]
    fn large_local_coordinates_do_not_reject_visible_path_fallback() {
        let resources = ResourceContext::default();
        let mut actual = SkCanvas::new(&resources, 128, 96);
        let mut reference = SkCanvas::new(&resources, 128, 96);
        let radius = PaintCornerRadius { x: 1.0, y: 1.0 };
        let command = DrawCommand {
            r#type: CommandKind::kClipRoundedRect,
            rect: PaintRect {
                x: 100000000.0,
                y: 0.0,
                width: 5.0,
                height: 20.0,
            },
            corner_radii: PaintCornerRadii {
                top_left: radius,
                top_right: radius,
                bottom_left: radius,
                bottom_right: radius,
            },
            antialias: false,
            ..Default::default()
        };
        for canvas in [&mut actual, &mut reference] {
            canvas.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kClipRect,
                    rect: PaintRect {
                        x: 20.0,
                        y: 2.0,
                        width: 2.0,
                        height: 10.0,
                    },
                    ..Default::default()
                },
                &resources,
            );
            canvas.state.transform = Transform::from_row(3.0, 0.0, 0.0, 1.0, -300000000.0, 0.0);
        }
        reference.clipPath(
            &rounded_rect_path(command.rect, command.corner_radii).unwrap(),
            false,
        );
        actual.replay_item(&command, &resources);
        assert!(reference.state.clip_summary.unwrap().getBounds().is_some());
        actual.ensure_clip_mask();
        reference.ensure_clip_mask();
        assert_eq!(
            actual.state.clip.as_ref().unwrap().data(),
            reference.state.clip.as_ref().unwrap().data()
        );
    }
    #[test]
    fn unrepresentable_local_rrect_preserves_existing_fallback_behavior() {
        let resources = ResourceContext::default();
        for antialias in [false, true] {
            let mut canvas = SkCanvas::new(&resources, 128, 96);
            let mut command = clip(1.0e100, 0.0);
            command.rect.width = 1.0;
            command.antialias = antialias;
            assert!(rounded_rect_path(command.rect, command.corner_radii).is_none());
            canvas.replay_item(&command, &resources);
            canvas.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kDrawRect,
                    rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 128.0,
                        height: 96.0,
                    },
                    color: Color {
                        blue: 1.0,
                        alpha: 1.0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                &resources,
            );
            assert!(canvas
                .pixmap
                .data()
                .chunks_exact(4)
                .all(|pixel| pixel == [0, 0, 255, 255]));
        }
    }
    #[test]
    #[ignore = "manual Debug timing, no speed assertion"]
    fn profile_empty_rounded_clips_round4() {
        let list = ResourceContext::default();
        let command = clip(80.0, 1900.0);
        for dense in [true, false] {
            for iteration in 0..5 {
                let mut canvas = SkCanvas::new(&list, 2432, 1408);
                canvas.set_scale(2.0);
                let started = std::time::Instant::now();
                for _ in 0..40 {
                    canvas.save();
                    if dense {
                        dense_clip(&mut canvas, &command);
                    } else {
                        canvas.replay_item(&command, &list);
                    }
                    canvas.restore();
                }
                let elapsed = started.elapsed().as_secs_f64() * 1000.0;
                let hash = canvas
                    .pixmap
                    .data()
                    .iter()
                    .fold(0u64, |h, &b| h.wrapping_mul(31).wrapping_add(b as u64));
                eprintln!("empty-clip-round4 dense={dense} iteration={iteration} ms={elapsed:.3} hash={hash}");
            }
        }
    }
}

#[cfg(test)]
mod bilibili_mask_tests {
    use super::*;

    fn mask_gradient(spread: SpreadMode, stop_kind: u32, reverse: bool) -> Paint<'static> {
        let stops = match stop_kind {
            0 => vec![
                GradientStop::new(
                    0.0,
                    crate::raster::Color::from_rgba(0.0, 0.0, 0.0, 0.0).unwrap(),
                ),
                GradientStop::new(
                    1.0,
                    crate::raster::Color::from_rgba(0.0, 0.0, 0.0, 1.0).unwrap(),
                ),
            ],
            1 => vec![
                GradientStop::new(
                    0.0,
                    crate::raster::Color::from_rgba(0.17, 0.81, 0.24, 0.0).unwrap(),
                ),
                GradientStop::new(
                    0.31,
                    crate::raster::Color::from_rgba(0.73, 0.25, 0.91, 0.41).unwrap(),
                ),
                GradientStop::new(
                    0.77,
                    crate::raster::Color::from_rgba(0.91, 0.19, 0.47, 0.93).unwrap(),
                ),
                GradientStop::new(
                    1.0,
                    crate::raster::Color::from_rgba(0.29, 0.69, 0.77, 0.62).unwrap(),
                ),
            ],
            _ => vec![
                GradientStop::new(0.0, crate::raster::Color::BLACK),
                GradientStop::new(0.5, crate::raster::Color::BLACK),
                GradientStop::new(0.5, crate::raster::Color::TRANSPARENT),
                GradientStop::new(1.0, crate::raster::Color::WHITE),
            ],
        };
        let (start, end) = if reverse { (83.7, -7.3) } else { (-7.3, 83.7) };
        let mut paint = Paint::default();
        paint.anti_alias = false;
        paint.shader = LinearGradient::new(
            Point::from_xy(17.5, start),
            Point::from_xy(17.5, end),
            stops,
            spread,
            Transform::from_row(1.25, 0.0, 0.0, 0.75, 3.7, -2.9),
        )
        .unwrap();
        paint
    }

    #[test]
    fn vertical_fallback_matches_original_pipeline_at_fractional_scan_edges() {
        let resources = ResourceContext::default();
        for spread in [SpreadMode::Pad, SpreadMode::Reflect, SpreadMode::Repeat] {
            for stop_kind in 0..3 {
                for reverse in [false, true] {
                    for antialias in [false, true] {
                        let mut paint = mask_gradient(spread, stop_kind, reverse);
                        paint.anti_alias = antialias;
                        for scale in [0.75, 1.0, 2.0] {
                            for fraction in [-13.7, 0.25, 0.4921875, 0.5, 0.5078125, 17.3] {
                                for format in
                                    [crate::PixelFormat::Rgba8888, crate::PixelFormat::Bgra8888]
                                {
                                    let destination = PaintRect {
                                        x: fraction,
                                        y: fraction + 1.3,
                                        width: 79.7,
                                        height: 60.5,
                                    };
                                    let mut actual = SkCanvas::new(&resources, 137, 93);
                                    let mut reference = SkCanvas::new(&resources, 137, 93);
                                    for canvas in [&mut actual, &mut reference] {
                                        canvas.pixmap.format = format;
                                        canvas.pixmap.data_mut().fill(0);
                                        canvas.state.transform = Transform::from_row(
                                            scale, 0.0, 0.0, scale, -3.25, 2.375,
                                        );
                                        canvas.replay_item(
                                            &DrawCommand {
                                                r#type: CommandKind::kClipRect,
                                                rect: PaintRect {
                                                    x: 3.5,
                                                    y: 1.2,
                                                    width: 100.3,
                                                    height: 79.7,
                                                },
                                                antialias: false,
                                                ..Default::default()
                                            },
                                            &resources,
                                        );
                                        canvas.replay_item(
                                            &DrawCommand {
                                                r#type: CommandKind::kClipRect,
                                                rect: destination,
                                                antialias: false,
                                                ..Default::default()
                                            },
                                            &resources,
                                        );
                                    }
                                    let destination_rect = rect(destination).unwrap();
                                    assert!(actual.draw_vertical_mask_gradient_fallback(destination_rect, &paint), "rejected spread={spread:?} stops={stop_kind} reverse={reverse} scale={scale} fraction={fraction} format={format:?} clip={:?} transform={:?}", actual.state.clip_summary, actual.state.transform);
                                    reference.ensure_clip_mask();
                                    reference.pixmap.fill_rect(
                                        destination_rect,
                                        &paint,
                                        reference.state.transform,
                                        reference.state.clip.as_deref(),
                                    );
                                    let first = actual
                                        .pixmap
                                        .data()
                                        .iter()
                                        .zip(reference.pixmap.data())
                                        .position(|(a, b)| a != b);
                                    assert!(first.is_none(), "first={first:?} spread={spread:?} stops={stop_kind} reverse={reverse} scale={scale} fraction={fraction} format={format:?}");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn vertical_mask_with_fractional_aa_clip_matches_original_fallback() {
        let resources = ResourceContext::default();
        let paint = mask_gradient(SpreadMode::Pad, 0, false);
        for scale in [0.75, 1.0, 2.0] {
            for format in [crate::PixelFormat::Rgba8888, crate::PixelFormat::Bgra8888] {
                let mut actual = SkCanvas::new(&resources, 137, 93);
                let mut reference = SkCanvas::new(&resources, 137, 93);
                let destination = PaintRect {
                    x: -13.7,
                    y: -12.4,
                    width: 79.7,
                    height: 60.5,
                };
                for canvas in [&mut actual, &mut reference] {
                    canvas.pixmap.format = format;
                    canvas.pixmap.data_mut().fill(0);
                    canvas.state.transform =
                        Transform::from_row(scale, 0.0, 0.0, scale, -3.25, 2.375);
                    for clip_rect in [
                        PaintRect {
                            x: 3.5,
                            y: 1.2,
                            width: 100.3,
                            height: 79.7,
                        },
                        destination,
                    ] {
                        canvas.replay_item(
                            &DrawCommand {
                                r#type: CommandKind::kClipRect,
                                rect: clip_rect,
                                antialias: true,
                                ..Default::default()
                            },
                            &resources,
                        );
                    }
                }
                assert!(!actual.state.clip_summary.unwrap().isRect());
                let command = DrawCommand {
                    r#type: CommandKind::kDrawGradientRect,
                    rect: destination,
                    paint_shader: Some(shader()),
                    antialias: false,
                    ..Default::default()
                };
                // Exercise the actual dispatcher: admitted first mask versus
                // the explicitly retained old path under the same real AA clip.
                actual.draw_gradient_tile_impl(&command, destination, true, true);
                reference.draw_gradient_tile_impl(&command, destination, true, false);
                let first = actual
                    .pixmap
                    .data()
                    .iter()
                    .zip(reference.pixmap.data())
                    .position(|(a, b)| a != b);
                assert!(
                    first.is_none(),
                    "first={first:?} scale={scale} format={format:?}"
                );
                assert!(actual
                    .pixmap
                    .data()
                    .chunks_exact(4)
                    .any(|pixel| pixel[3] != 0));
            }
        }
    }

    #[test]
    fn dst_in_span_matches_byte_domain_oracle_and_preserves_guards() {
        for alpha in 0..=255u8 {
            let mut actual: Vec<u8> = (0..=255u8)
                .flat_map(|c| [c, 255 - c, c.wrapping_mul(37), c.wrapping_mul(71)])
                .collect();
            let mask: Vec<u8> = (0..256).flat_map(|_| [1, 2, 3, alpha]).collect();
            let expected: Vec<u8> = actual
                .iter()
                .map(|&c| ((u32::from(c) * u32::from(alpha) + 127) / 255) as u8)
                .collect();
            mask_dst_in_rgba_span(&mut actual, &mask);
            assert_eq!(actual, expected, "alpha={alpha}");
        }
        for offset in 0..32 {
            for pixels in 0..35 {
                let mut actual = vec![0xA7; offset + pixels * 4 + 31];
                let mut expected = actual.clone();
                let mask: Vec<u8> = (0..pixels)
                    .flat_map(|p| [0, 0, 0, (p * 79 + pixels * 11) as u8])
                    .collect();
                for (dst, src) in expected[offset..offset + pixels * 4]
                    .chunks_exact_mut(4)
                    .zip(mask.chunks_exact(4))
                {
                    for channel in dst {
                        *channel = ((u32::from(*channel) * u32::from(src[3]) + 127) / 255) as u8;
                    }
                }
                mask_dst_in_rgba_span(&mut actual[offset..offset + pixels * 4], &mask);
                assert_eq!(actual, expected, "offset={offset} pixels={pixels}");
            }
        }
    }

    fn shader() -> PaintShader {
        let mut shader = PaintShader::default();
        shader.kind = PaintShaderKind::kLinearGradient;
        shader.start = Offset { x: 0.0, y: -3.3 };
        shader.end = Offset { x: 0.0, y: 73.7 };
        shader.stops = vec![
            PaintColorStop {
                offset: 0.0,
                color: Color {
                    alpha: 0.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            PaintColorStop {
                offset: 0.49,
                color: Color {
                    alpha: 0.63,
                    ..Default::default()
                },
                ..Default::default()
            },
            PaintColorStop {
                offset: 1.0,
                color: Color {
                    alpha: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ];
        shader
    }

    #[test]
    fn complete_masks_match_original_with_layers_crops_and_aa_parent_clips() {
        let resources = ResourceContext::default();
        for scale in [0.75, 1.0, 2.0] {
            for rounded in [false, true] {
                for layers in 0..=3 {
                    let render = |fast: bool| {
                        let mut canvas = SkCanvas::new(&resources, 137, 93);
                        canvas.set_scale(scale);
                        let radius = PaintCornerRadius { x: 7.25, y: 5.75 };
                        canvas.replay_item(
                            &DrawCommand {
                                r#type: if rounded {
                                    CommandKind::kClipRoundedRect
                                } else {
                                    CommandKind::kClipRect
                                },
                                rect: PaintRect {
                                    x: 3.7,
                                    y: 2.2,
                                    width: 83.7,
                                    height: 60.5,
                                },
                                antialias: rounded,
                                corner_radii: PaintCornerRadii {
                                    top_left: radius,
                                    top_right: radius,
                                    bottom_left: radius,
                                    bottom_right: radius,
                                },
                                ..Default::default()
                            },
                            &resources,
                        );
                        canvas.begin_mask(&DrawCommand {
                            mask_layers: (0..layers)
                                .map(|i| MaskLayer {
                                    paint_shader: Some(shader()),
                                    clip_rect: PaintRect {
                                        x: 7.3 + i as f64 * 11.7,
                                        y: 3.7 + i as f64 * 13.3,
                                        width: 67.5,
                                        height: 48.5,
                                    },
                                    ..Default::default()
                                })
                                .collect(),
                            ..Default::default()
                        });
                        canvas.replay_item(
                            &DrawCommand {
                                r#type: CommandKind::kDrawRect,
                                rect: PaintRect {
                                    x: 1.25,
                                    y: 1.75,
                                    width: 98.7,
                                    height: 79.5,
                                },
                                color: Color {
                                    red: 0.17,
                                    green: 0.63,
                                    blue: 0.89,
                                    alpha: 0.71,
                                },
                                ..Default::default()
                            },
                            &resources,
                        );
                        canvas.end_mask_impl(&resources, fast);
                        canvas.pixmap.data().to_vec()
                    };
                    let actual = render(true);
                    let reference = render(false);
                    let first = actual.iter().zip(&reference).position(|(a, b)| a != b);
                    assert!(
                        first.is_none(),
                        "first={first:?} scale={scale} rounded={rounded} layers={layers}"
                    );
                }
            }
        }
    }

    #[test]
    fn masked_content_with_aa_clip_and_nonvertical_or_overlapping_masks_matches_original() {
        let resources = ResourceContext::default();
        for direction in 0..3 {
            for scale in [0.75, 1.0, 2.0] {
                for format in [
                    crate::PixelFormat::Rgba8888,
                    crate::PixelFormat::Bgra8888,
                    crate::PixelFormat::Bgrx8888,
                ] {
                    let render = |fast: bool| {
                        let mut canvas = SkCanvas::new(&resources, 137, 93);
                        canvas.pixmap.format = format;
                        canvas.set_scale(scale);
                        let mut gradient = shader();
                        if direction == 1 {
                            gradient.end.x = 81.7;
                        }
                        if direction == 2 {
                            gradient.end.x = 81.7;
                            gradient.end.y = gradient.start.y;
                        }
                        canvas.begin_mask(&DrawCommand {
                            mask_layers: (0..2)
                                .map(|i| MaskLayer {
                                    paint_shader: Some(gradient.clone()),
                                    clip_rect: PaintRect {
                                        x: 7.3 + i as f64 * 11.7,
                                        y: 3.7 + i as f64 * 13.3,
                                        width: 67.5,
                                        height: 48.5,
                                    },
                                    ..Default::default()
                                })
                                .collect(),
                            ..Default::default()
                        });
                        canvas.replay_item(
                            &DrawCommand {
                                r#type: CommandKind::kDrawRect,
                                rect: PaintRect {
                                    x: 0.0,
                                    y: 0.0,
                                    width: 137.0,
                                    height: 93.0,
                                },
                                color: Color {
                                    red: 0.17,
                                    green: 0.63,
                                    blue: 0.89,
                                    alpha: 0.71,
                                },
                                ..Default::default()
                            },
                            &resources,
                        );
                        let radius = PaintCornerRadius { x: 7.25, y: 5.75 };
                        canvas.replay_item(
                            &DrawCommand {
                                r#type: CommandKind::kClipRoundedRect,
                                rect: PaintRect {
                                    x: 3.7,
                                    y: 2.2,
                                    width: 83.7,
                                    height: 60.5,
                                },
                                antialias: true,
                                corner_radii: PaintCornerRadii {
                                    top_left: radius,
                                    top_right: radius,
                                    bottom_left: radius,
                                    bottom_right: radius,
                                },
                                ..Default::default()
                            },
                            &resources,
                        );
                        canvas.end_mask_impl(&resources, fast);
                        canvas.pixmap.data().to_vec()
                    };
                    let actual = render(true);
                    let reference = render(false);
                    let first = actual.iter().zip(&reference).position(|(a, b)| a != b);
                    assert!(
                        first.is_none(),
                        "first={first:?} direction={direction} scale={scale} format={format:?}"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod bilibili_mask_profile {
    use super::*;
    use std::hash::Hasher;

    #[test]
    #[ignore = "manual Debug mask timing, no speed assertion"]
    fn profile_bilibili_vertical_mask() {
        let resources = ResourceContext::default();
        for fast in [false, true] {
            for iteration in 0..3 {
                let mut canvas = SkCanvas::new(&resources, 2560, 1598);
                canvas.set_scale(2.0);
                let mut shader = PaintShader::default();
                shader.kind = PaintShaderKind::kLinearGradient;
                shader.start = Offset { x: 0.0, y: 0.0 };
                shader.end = Offset { x: 0.0, y: 799.0 };
                shader.stops = vec![
                    PaintColorStop {
                        offset: 0.0,
                        color: Color {
                            alpha: 0.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    PaintColorStop {
                        offset: 0.49,
                        color: Color {
                            alpha: 0.63,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    PaintColorStop {
                        offset: 1.0,
                        color: Color {
                            alpha: 1.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ];
                canvas.begin_mask(&DrawCommand {
                    mask_layers: vec![MaskLayer {
                        paint_shader: Some(shader),
                        clip_rect: PaintRect {
                            x: 0.3,
                            y: 0.7,
                            width: 1279.5,
                            height: 798.3,
                        },
                        ..Default::default()
                    }],
                    ..Default::default()
                });
                canvas.replay_item(
                    &DrawCommand {
                        r#type: CommandKind::kDrawRect,
                        rect: PaintRect {
                            x: 0.0,
                            y: 0.0,
                            width: 1280.0,
                            height: 799.0,
                        },
                        color: Color {
                            red: 0.17,
                            green: 0.63,
                            blue: 0.89,
                            alpha: 0.71,
                        },
                        ..Default::default()
                    },
                    &resources,
                );
                let start = std::time::Instant::now();
                canvas.end_mask_impl(&resources, fast);
                let ms = start.elapsed().as_secs_f64() * 1000.0;
                let mut hash = std::collections::hash_map::DefaultHasher::new();
                hash.write(canvas.pixmap.data());
                eprintln!("bilibili-mask fast={fast} iteration={iteration} end_mask_ms={ms:.3} hash={:016x}", hash.finish());
            }
        }
    }
}

// Artifact only. Append to compat/canvas.rs after promoting the separate helper.
#[cfg(test)]
mod integer_inset_shadow_oracle_tests {
    use super::*;
    fn initialized(width: u32, height: u32, format: crate::PixelFormat, scale: f32) -> SkCanvas {
        let mut c = SkCanvas::new(&ResourceContext::default(), width, height);
        c.pixmap.format = format;
        c.state.transform = Transform::from_row(scale, 0., 0., scale, 0., -4.);
        for (i, p) in c.pixmap.data_mut().chunks_exact_mut(4).enumerate() {
            let alpha = if format == crate::PixelFormat::Bgrx8888 {
                255
            } else {
                [63, 127, 255][i % 3]
            };
            let rgba = [
                (i * 73 % (alpha as usize + 1)) as u8,
                (i * 17 % (alpha as usize + 1)) as u8,
                (i * 37 % (alpha as usize + 1)) as u8,
                alpha,
            ];
            p.copy_from_slice(&format.encode(rgba));
            if format == crate::PixelFormat::Bgrx8888 {
                p[3] = (i * 11) as u8;
            }
        }
        c
    }
    fn same(actual: &SkCanvas, expected: &SkCanvas, label: &str) {
        let first = actual
            .pixmap
            .data()
            .iter()
            .zip(expected.pixmap.data())
            .position(|(a, b)| a != b);
        assert!(first.is_none(), "{label}: firstdifferent={first:?}");
    }
    #[test]
    fn integer_rectangle_difference_equals_bounded_and_full_original_masks() {
        for format in [
            crate::PixelFormat::Rgba8888,
            crate::PixelFormat::Bgra8888,
            crate::PixelFormat::Bgrx8888,
        ] {
            for scale in [0.5, 1., 1.25, 1.5, 2.] {
                for offset in [(0., -4.), (-4., 0.), (4., 4.)] {
                    for spread in [-4., 0., 20.] {
                        for clipped in [false, true] {
                            for alpha in [0.35, 1.] {
                                let item = DrawCommand {
                                    r#type: CommandKind::kDrawBoxShadow,
                                    rect: PaintRect {
                                        x: 4.,
                                        y: 8.,
                                        width: 48.,
                                        height: 32.,
                                    },
                                    inset: true,
                                    spread,
                                    shadow_offset: Offset {
                                        x: offset.0,
                                        y: offset.1,
                                    },
                                    color: Color {
                                        red: 0.27,
                                        green: 0.61,
                                        blue: 0.43,
                                        alpha,
                                    },
                                    ..Default::default()
                                };
                                let mut actual = initialized(97, 83, format, scale);
                                let mut bounded = initialized(97, 83, format, scale);
                                let mut full = initialized(97, 83, format, scale);
                                if clipped {
                                    let clip = DrawCommand {
                                        r#type: CommandKind::kClipRect,
                                        rect: PaintRect {
                                            x: 8.,
                                            y: 12.,
                                            width: 32.,
                                            height: 24.,
                                        },
                                        ..Default::default()
                                    };
                                    for c in [&mut actual, &mut bounded, &mut full] {
                                        c.replay_item(&clip, &ResourceContext::default());
                                    }
                                }
                                let label=format!("format={format:?} scale={scale} offset={offset:?} spread={spread} clipped={clipped} alpha={alpha}");
                                let before = actual.pixmap.data().to_vec();
                                let admitted = actual.draw_integer_rectangular_inset_shadow(&item);
                                assert_eq!(admitted, spread >= 0., "{label}");
                                if !admitted {
                                    assert_eq!(actual.pixmap.data(), before);
                                    assert!(actual.draw_unblurred_inset_shadow_region(&item));
                                }
                                assert!(
                                    bounded.draw_unblurred_inset_shadow_region(&item),
                                    "{label}"
                                );
                                full.ensure_clip_mask();
                                full.draw_inset_shadow(&item);
                                same(&actual, &bounded, &label);
                                same(&actual, &full, &label);
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn actual_header_changes_only_original_bottom_rows_in_every_format() {
        for format in [
            crate::PixelFormat::Rgba8888,
            crate::PixelFormat::Bgra8888,
            crate::PixelFormat::Bgrx8888,
        ] {
            for scale in [1.0_f32, 2.0] {
                let (width, height) = ((1280.0 * scale) as u32, (713.0 * scale) as u32);
                let item = DrawCommand {
                    r#type: CommandKind::kDrawBoxShadow,
                    rect: PaintRect {
                        x: 0.,
                        y: 0.,
                        width: 1328.,
                        height: 64.,
                    },
                    inset: true,
                    shadow_offset: Offset { x: 0., y: -1. },
                    color: Color {
                        red: 0.27,
                        green: 0.61,
                        blue: 0.43,
                        alpha: 0.35,
                    },
                    ..Default::default()
                };
                let mut actual = initialized(width, height, format, scale);
                let mut bounded = initialized(width, height, format, scale);
                let mut full = initialized(width, height, format, scale);
                let mut production = initialized(width, height, format, scale);
                for c in [&mut actual, &mut bounded, &mut full, &mut production] {
                    c.state.transform.ty = 0.;
                }
                let before = actual.pixmap.data().to_vec();
                assert!(actual.draw_integer_rectangular_inset_shadow(&item));
                assert!(bounded.draw_unblurred_inset_shadow_region(&item));
                full.ensure_clip_mask();
                full.draw_inset_shadow(&item);
                production.draw_box_shadow(&item);
                let label = format!("actual header format={format:?} scale={scale}");
                same(&actual, &bounded, &label);
                same(&actual, &full, &label);
                same(&actual, &production, &label);
                let first_row = (63.0 * scale) as usize;
                let last_row = (64.0 * scale) as usize;
                let begin = first_row * width as usize * 4;
                let end = last_row * width as usize * 4;
                assert_eq!(
                    &actual.pixmap.data()[..begin],
                    &before[..begin],
                    "{label}: above band"
                );
                assert_eq!(
                    &actual.pixmap.data()[end..],
                    &before[end..],
                    "{label}: below band"
                );
                assert_ne!(
                    &actual.pixmap.data()[begin..end],
                    &before[begin..end],
                    "{label}: must actually draw visible pixels"
                );
            }
        }
    }
    #[test]
    fn fractional_geometry_rejects_without_writing_any_bytes() {
        for field in 0..7 {
            let mut c = initialized(97, 83, crate::PixelFormat::Rgba8888, 1.);
            let mut item = DrawCommand {
                r#type: CommandKind::kDrawBoxShadow,
                rect: PaintRect {
                    x: 4.,
                    y: 8.,
                    width: 48.,
                    height: 32.,
                },
                inset: true,
                shadow_offset: Offset { x: 0., y: -1. },
                color: Color {
                    alpha: 1.,
                    ..Default::default()
                },
                ..Default::default()
            };
            match field {
                0 => item.rect.y += 0.25,
                1 => item.shadow_offset.y += 0.25,
                2 => item.spread = 0.25,
                3 => item.blur_radius = 1.,
                4 => item.corner_radii.top_left = PaintCornerRadius { x: 2., y: 2. },
                5 => c.state.transform.kx = 0.25,
                _ => c.f16_surface = Some(crate::cpu::f16_surface::Surface::new(97 * 83)),
            }
            let before = c.pixmap.data().to_vec();
            assert!(
                !c.draw_integer_rectangular_inset_shadow(&item),
                "field={field}"
            );
            assert_eq!(c.pixmap.data(), before, "field={field}");
        }
    }

    // Artifact only; add inside integer_inset_shadow_oracle_tests. Root owns CPU slot.
    // The independent full-device old source supplies the complete output oracle.
    #[test]
    #[ignore = "isolated old/new Debug CPU benchmark; root serial only"]
    fn header_old_bounded_candidate_abba_with_every_frame_complete_pixels() {
        for format in [
            crate::PixelFormat::Rgba8888,
            crate::PixelFormat::Bgra8888,
            crate::PixelFormat::Bgrx8888,
        ] {
            for scale in [1.0_f32, 2.0] {
                let (width, height) = ((1280.0 * scale) as u32, (713.0 * scale) as u32);
                let item = DrawCommand {
                    r#type: CommandKind::kDrawBoxShadow,
                    rect: PaintRect {
                        x: 0.,
                        y: 0.,
                        width: 1328.,
                        height: 64.,
                    },
                    inset: true,
                    shadow_offset: Offset { x: 0., y: -1. },
                    color: Color {
                        red: 0.27,
                        green: 0.61,
                        blue: 0.43,
                        alpha: 0.35,
                    },
                    ..Default::default()
                };
                let mut golden = initialized(width, height, format, scale);
                golden.state.transform.ty = 0.;
                let initial = golden.pixmap.data().to_vec();
                golden.ensure_clip_mask();
                golden.draw_inset_shadow(&item);
                let expected = golden.pixmap.data().to_vec();
                assert_ne!(
                    expected, initial,
                    "must render actual changed header pixels"
                );
                let changed_bytes = expected
                    .iter()
                    .zip(&initial)
                    .filter(|(a, b)| a != b)
                    .count();
                eprintln!("integer-inset-header initialized format={format:?} scale={scale} changed_bytes={changed_bytes}");
                let mut old = initialized(width, height, format, scale);
                let mut candidate = initialized(width, height, format, scale);
                old.state.transform.ty = 0.;
                candidate.state.transform.ty = 0.;
                let mut old_ns = Vec::new();
                let mut candidate_ns = Vec::new();
                // Every draw begins with the full original initialized target.
                // Restore, allocation and whole-output comparison remain outside
                // the draw interval, equally for A and B; neither path caches output.
                for round in 0..8 {
                    for is_candidate in [false, true, true, false] {
                        let canvas = if is_candidate {
                            &mut candidate
                        } else {
                            &mut old
                        };
                        canvas.pixmap.data_mut().copy_from_slice(&initial);
                        let start = std::time::Instant::now();
                        let admitted = if is_candidate {
                            canvas.draw_integer_rectangular_inset_shadow(&item)
                        } else {
                            canvas.draw_unblurred_inset_shadow_region(&item)
                        };
                        let ns = start.elapsed().as_nanos();
                        assert!(admitted,"format={format:?} scale={scale} round={round} candidate={is_candidate}");
                        let first = canvas
                            .pixmap
                            .data()
                            .iter()
                            .zip(&expected)
                            .position(|(a, b)| a != b);
                        assert!(first.is_none(),"format={format:?} scale={scale} round={round} candidate={is_candidate} firstdifferent={first:?}");
                        if is_candidate {
                            candidate_ns.push(ns);
                        } else {
                            old_ns.push(ns);
                        }
                    }
                }
                old_ns.sort_unstable();
                candidate_ns.sort_unstable();
                eprintln!("integer-inset-header ABBA format={format:?} scale={scale} old_ns={old_ns:?} candidate_ns={candidate_ns:?} old_median_ns={} candidate_median_ns={} complete_frame_checks={}",
                old_ns[old_ns.len()/2],candidate_ns[candidate_ns.len()/2],old_ns.len()+candidate_ns.len());
            }
        }
    }
}
