// Source: components/viz/service/display/software_renderer.h:1-140
// Source: components/viz/service/display/software_renderer.cc:1-1119
//! SoftwareRenderer execution through the source-aligned `skia` Canvas.
//!
//! DirectRenderer continues to own traversal, pass allocation requirements and
//! damage. This backend owns only pass pixels and synchronous drawing. Missing
//! translated materials fail the frame; execution never retries with legacy
//! Skia after mutating a pass.
#![allow(non_snake_case)]

use crate::{native::RenderTarget, translated_adapter::*};
use compositor_core::{
    adapter::{
        geometry::{PointF, Rect, RectF, Size},
        property_types::BlendMode,
    },
    runtime::frame::{QuadMaterial, RenderPassId},
};
use renderer::{RenderPassRequirements, RenderUpdate, RendererBackend, RendererSettings};
use skia::{
    include::core::{
        SkClipOp::SkClipOp,
        SkPathTypes::SkPathFillType,
        SkSamplingOptions::{SkFilterMode, SkSamplingOptions},
        SkTileMode::SkTileMode,
    },
    raster_target::{N32BlitMode, PixelFormat, RasterTarget},
    src::{
        core::{
            SkBlendMode::SkBlendMode,
            SkCanvas::SrcRectConstraint,
            SkM44::SkM44,
            SkMatrix::SkMatrix,
            SkPaint::{SkPaint, Style},
            SkPath::SkPath,
            SkPathBuilder::SkPathBuilder,
            SkPoint::SkPoint,
            SkRect::SkRect,
        },
        image::SkImage::SkImage,
    },
};
use std::{collections::BTreeMap, io, sync::Arc};
use viz::{
    common::quads::aggregated_render_pass::{NativeDrawQuad, NativeSharedQuadState},
    native::{AggregatedRenderPass, VizOutput},
};

struct PendingDraw {
    index: usize,
    draw_region: Option<[PointF; 4]>,
    scissor_rect: Option<Rect>,
}

/// Filter bounds consume only SkMatrix::mapVector. This adapter keeps the
/// translated SkMatrix concrete type inside the translated renderer while the
/// Paint contract carries only that semantic operation.
struct TranslatedFilterMapTransform<'a>(&'a SkMatrix);

impl paint::cc::paint::filter_operation::FilterMapTransform for TranslatedFilterMapTransform<'_> {
    fn MapVector(&self, x: f32, y: f32) -> (f32, f32) {
        let mapped = self.0.mapVector(SkPoint::make(x, y));
        (mapped.x(), mapped.y())
    }
}

#[derive(Default)]
pub struct TranslatedSoftwareRenderer {
    render_pass_bitmaps: BTreeMap<RenderPassId, PassBacking>,
    settings: RendererSettings,
    update: RenderUpdate,
    current_pass: Option<RenderPassId>,
    current_rect: Rect,
    scissor_rect: Option<Rect>,
    linear_sampling: bool,
    profile_input: bool,
    n32_tile_blit: bool,
    framebuffer_damage: BTreeMap<RenderPassId, Rect>,
    drawn_damage: BTreeMap<RenderPassId, Rect>,
    pending_draws: Vec<PendingDraw>,
    pending_clear: Option<(Rect, bool)>,
    frame_target: Option<RenderTarget>,
    direct_root_pass: Option<RenderPassId>,
}

impl TranslatedSoftwareRenderer {
    fn CanPartialSwap(&self) -> bool {
        true
    }

    fn BeginDrawingFrame(&mut self, input: &VizOutput) -> io::Result<()> {
        self.update = RenderUpdate::default();
        self.linear_sampling = false;
        self.profile_input = std::env::var_os("BROWSER_PROFILE_INPUT").is_some();
        self.n32_tile_blit = std::env::var_os("BROWSER_DISABLE_N32_TILE_BLIT").is_none();
        self.framebuffer_damage.clear();
        self.drawn_damage.clear();
        for pass in input.Frame().RenderPasses() {
            if input.RootDestination(pass.RenderPassId()).is_some()
                && self
                    .render_pass_bitmaps
                    .get(&pass.RenderPassId())
                    .is_none_or(|backing| backing.drawn_rect != pass.OutputRect())
            {
                self.framebuffer_damage
                    .insert(pass.RenderPassId(), pass.OutputRect());
            }
        }
        Ok(())
    }

    fn FinishDrawingFrame(&mut self) -> io::Result<()> {
        if self.current_pass.is_some() || !self.pending_draws.is_empty() {
            return Err(io::Error::other(
                "translated renderer ended a frame with an unfinished render pass",
            ));
        }
        Ok(())
    }

    fn BindFrameTarget(&mut self, input: &VizOutput, mut target: RenderTarget) -> io::Result<()> {
        if self.frame_target.is_some() {
            return Err(io::Error::other(
                "translated renderer already has a bound frame target",
            ));
        }
        self.direct_root_pass = None;
        let full_bounds = Rect::new(0, 0, target.width as i32, target.height as i32);
        let mut roots = input.Frame().RenderPasses().iter().filter_map(|pass| {
            input
                .RootDestination(pass.RenderPassId())
                .map(|destination| (pass, destination))
        });
        let direct = roots.next().and_then(|(pass, destination)| {
            if roots.next().is_some()
                || target.format != crate::native::PixelFormat::Bgra8888
                || target.row_stride != target.width as usize
                || destination != full_bounds
                || pass.OutputRect() != full_bounds
            {
                return None;
            }
            Some(pass.RenderPassId())
        });
        if let Some(id) = direct {
            let backing = self
                .render_pass_bitmaps
                .get_mut(&id)
                .ok_or_else(|| io::Error::other("translated root backing was not allocated"))?;
            if backing.rect.size() == full_bounds.size()
                && backing.pixels.len() == target.pixels.len()
            {
                std::mem::swap(&mut backing.pixels, &mut target.pixels);
                if !target.retained {
                    self.framebuffer_damage.insert(id, full_bounds);
                    backing.drawn_rect = Rect::default();
                }
                self.direct_root_pass = Some(id);
                self.update.direct_root_target = true;
            }
        }
        self.frame_target = Some(target);
        Ok(())
    }

    fn ReleaseFrameTarget(
        &mut self,
        input: &VizOutput,
    ) -> io::Result<(RenderUpdate, RenderTarget)> {
        let mut target = self
            .frame_target
            .take()
            .ok_or_else(|| io::Error::other("translated renderer has no bound frame target"))?;
        if let Some(id) = self.direct_root_pass.take() {
            let backing = self
                .render_pass_bitmaps
                .get_mut(&id)
                .ok_or_else(|| io::Error::other("translated root backing disappeared"))?;
            std::mem::swap(&mut backing.pixels, &mut target.pixels);
        } else {
            publish(
                input,
                &mut target,
                &self.render_pass_bitmaps,
                &self.drawn_damage,
            )?;
        }
        Ok((self.update, target))
    }

    fn EnsureScissorTestDisabled(&mut self) {
        self.scissor_rect = None;
    }

    fn SetScissorTestRect(&mut self, rect: Rect) {
        self.scissor_rect = Some(rect);
    }

    fn BeginDrawingRenderPass(
        &mut self,
        pass: &AggregatedRenderPass,
        needs_clear: bool,
        update: Rect,
        viewport: Size,
    ) -> io::Result<()> {
        if self.current_pass.is_some() || !self.pending_draws.is_empty() {
            return Err(io::Error::other(
                "translated renderer began a render pass before finishing the previous pass",
            ));
        }
        let backing = self
            .render_pass_bitmaps
            .get_mut(&pass.RenderPassId())
            .ok_or_else(|| io::Error::other("translated render-pass bitmap not allocated"))?;
        backing.rect.x = pass.OutputRect().x;
        backing.rect.y = pass.OutputRect().y;
        self.current_pass = Some(pass.RenderPassId());
        self.current_rect = pass.OutputRect();
        self.drawn_damage.insert(
            pass.RenderPassId(),
            Rect::new(
                update.x + pass.OutputRect().x,
                update.y + pass.OutputRect().y,
                update.width,
                update.height,
            ),
        );
        if update == Rect::new(0, 0, viewport.width(), viewport.height()) {
            self.EnsureScissorTestDisabled();
        } else {
            self.SetScissorTestRect(update);
        }
        self.pending_clear = if needs_clear {
            let bounds = self.scissor_rect.unwrap_or(Rect::new(
                0,
                0,
                self.current_rect.width,
                self.current_rect.height,
            ));
            Some((bounds, pass.has_transparent_background))
        } else {
            None
        };
        self.update.passes += 1;
        Ok(())
    }

    fn clip_draw_region(target: &mut RasterTarget<'_>, region: &[PointF; 4], visible: Rect) {
        let mut path = SkPathBuilder::with_fill_type(SkPathFillType::kWinding);
        for (index, point) in region.iter().enumerate() {
            let x = point.x() - visible.x as f32;
            let y = point.y() - visible.y as f32;
            if index == 0 {
                path.moveToXY(x, y);
            } else {
                path.lineToXY(x, y);
            }
        }
        path.close();
        target.clip_path(&path.detach(None), SkClipOp::kIntersect, false);
    }

    fn DrawQuadIntoTarget(
        &mut self,
        target: &mut RasterTarget<'_>,
        input: &VizOutput,
        pass: &AggregatedRenderPass,
        index: usize,
        draw_region: Option<&[PointF; 4]>,
    ) -> io::Result<()> {
        let quad_started = self.profile_input.then(std::time::Instant::now);
        let quad = pass
            .Quads()
            .get(index)
            .ok_or_else(|| io::Error::other("translated draw-quad index is invalid"))?;
        let shared = pass
            .TrySharedQuadState(
                quad.geometry
                    .shared_quad_state
                    .ok_or_else(|| io::Error::other("quad lacks SharedQuadState"))?,
            )
            .ok_or_else(|| io::Error::other("invalid SharedQuadState handle"))?;
        let current_rect = self.current_rect;
        let scissor = self.scissor_rect;
        let settings = self.settings.clone();
        target.save();
        let draw_result = (|| -> io::Result<()> {
            if let Some(scissor) = scissor {
                target.clip_rect(&rect(scissor), SkClipOp::kIntersect, false);
            }
            target.set_matrix(&SkMatrix::Translate(
                -current_rect.x as f32,
                -current_rect.y as f32,
            ));
            if renderer::ShouldApplyRoundedCorner(quad, shared) {
                target.clip_rrect(
                    &rrect(&shared.mask_filter_info.rounded_corner_bounds),
                    SkClipOp::kIntersect,
                    true,
                );
            }
            let transform = matrix(shared.quad_to_target_transform.native(), current_rect)?;
            target.set_matrix(&transform);
            let antialias =
                if settings.force_antialiasing || !is_scale_and_integer_translate(&transform) {
                    self.linear_sampling = true;
                    let exterior = quad.geometry.rect == shared.quad_layer_rect;
                    settings.allow_antialiasing && (settings.force_antialiasing || exterior)
                } else {
                    false
                };
            let should_blend = quad.geometry.needs_blending
                || shared.opacity < 1.
                || !shared.mask_filter_info.IsEmpty()
                || shared.blend_mode != BlendMode::kSrcOver;
            let blend = if should_blend {
                blend_mode(shared.blend_mode)
            } else {
                SkBlendMode::kSrc
            };
            if let Some(region) = draw_region {
                Self::clip_draw_region(target, region, quad.geometry.visible_rect);
            }
            match &quad.material {
                QuadMaterial::SolidColor { .. } => {
                    Self::DrawSolidColorQuad(target, quad, shared, blend, antialias)?;
                    self.update.solid_quads += 1;
                }
                QuadMaterial::Tile { .. } => {
                    self.DrawTileQuad(
                        target,
                        input,
                        quad,
                        shared,
                        blend,
                        antialias,
                        draw_region.is_none(),
                    )?;
                    self.update.tile_quads += 1;
                }
                QuadMaterial::DebugBorder { .. } => {
                    Self::DrawDebugBorderQuad(target, quad, shared, blend)?;
                }
                QuadMaterial::CompositorRenderPass { .. } => {
                    self.DrawRenderPassQuad(target, input, quad, shared, blend, antialias)?;
                    self.update.render_pass_quads += 1;
                }
                QuadMaterial::Picture { .. } => {
                    Self::DrawPictureQuad(target, quad, shared)?;
                    self.update.picture_quads += 1;
                }
                QuadMaterial::SharedElement { .. } => {
                    return Err(io::Error::other(
                        "unresolved SharedElement reached translated SoftwareRenderer",
                    ));
                }
            }
            Ok(())
        })();
        let restore_result = target
            .restore()
            .map_err(|error| raster_error("translated draw-quad restore failed", error));
        draw_result.and(restore_result)?;

        if let Some(started) = quad_started {
            let elapsed = started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
            let counter = match &quad.material {
                QuadMaterial::Tile { .. } => Some(&mut self.update.tile_ns),
                QuadMaterial::Picture { .. } => Some(&mut self.update.picture_ns),
                QuadMaterial::SolidColor { .. } => Some(&mut self.update.solid_ns),
                QuadMaterial::CompositorRenderPass { .. } => Some(&mut self.update.render_pass_ns),
                _ => None,
            };
            if let Some(counter) = counter {
                *counter = counter.saturating_add(elapsed);
            }
        }
        self.update.quads += 1;
        self.update.damage_pixels = self.update.damage_pixels.saturating_add(
            quad.geometry.visible_rect.width.max(0) as usize
                * quad.geometry.visible_rect.height.max(0) as usize,
        );
        Ok(())
    }

    fn DrawSolidColorQuad(
        target: &mut RasterTarget<'_>,
        quad: &NativeDrawQuad,
        shared: &NativeSharedQuadState,
        blend: SkBlendMode,
        antialias: bool,
    ) -> io::Result<()> {
        let QuadMaterial::SolidColor { color: value, .. } = &quad.material else {
            return Ok(());
        };
        let mut paint = SkPaint::default();
        paint.setColor4f(&color(*value, shared.opacity), None);
        paint.setBlendMode(blend);
        paint.setAntiAlias(antialias);
        unsafe { target.draw_rect(&rect(quad.geometry.visible_rect), &paint) }
            .map_err(|error| raster_error("translated solid-color draw failed", error))
    }

    fn DrawDebugBorderQuad(
        target: &mut RasterTarget<'_>,
        quad: &NativeDrawQuad,
        shared: &NativeSharedQuadState,
        blend: SkBlendMode,
    ) -> io::Result<()> {
        let QuadMaterial::DebugBorder {
            color: value,
            width,
        } = &quad.material
        else {
            return Ok(());
        };
        let mut paint = SkPaint::default();
        paint.setColor4f(&color(*value, shared.opacity), None);
        paint.setBlendMode(blend);
        paint.setStyle(Style::kStroke_Style);
        paint.setStrokeWidth(*width as f32);
        paint.setAntiAlias(false);
        unsafe { target.draw_rect(&rect(quad.geometry.rect), &paint) }
            .map_err(|error| raster_error("translated debug-border draw failed", error))
    }

    fn DrawTileQuad(
        &mut self,
        target: &mut RasterTarget<'_>,
        input: &VizOutput,
        quad: &NativeDrawQuad,
        shared: &NativeSharedQuadState,
        blend: SkBlendMode,
        antialias: bool,
        has_rectangular_clip: bool,
    ) -> io::Result<()> {
        let QuadMaterial::Tile {
            resource_id,
            tex_coord_rect,
            nearest_neighbor,
            ..
        } = &quad.material
        else {
            return Ok(());
        };
        let source = compositor_core::cc::base::math_util::MathUtil::ScaleRectProportional(
            *tex_coord_rect,
            RectF::from(quad.geometry.rect),
            RectF::from(quad.geometry.visible_rect),
        );
        // A one-to-one, pixel-aligned translation samples every source pixel at
        // its exact center. Linear and nearest filtering are therefore
        // identical, while the latter lets raster Skia use its copy/blend fast
        // path. Keep Chromium's requested filter for every scaled, rotated,
        // skewed or fractional mapping.
        let transform = shared.quad_to_target_transform.native();
        let nearly = |a: f64, b: f64| (a - b).abs() <= 1. / 4096.;
        let integral = |value: f64| (value - value.round()).abs() <= 1. / 4096.;
        // Sprite eligibility depends on the complete source-to-device matrix,
        // not on the source and CTM translations being integral separately.
        // Equal fractional source and device origins cancel: every destination
        // pixel center still lands exactly on a source pixel center. This is
        // the same condition SkTreatAsSprite evaluates after drawImageRect has
        // concatenated the subset matrix with the device CTM.
        let device_left = quad.geometry.visible_rect.x as f64 + transform.rc(0, 3);
        let device_top = quad.geometry.visible_rect.y as f64 + transform.rc(1, 3);
        let pixel_aligned_one_to_one = nearly(
            source.width() as f64,
            quad.geometry.visible_rect.width as f64,
        ) && nearly(
            source.height() as f64,
            quad.geometry.visible_rect.height as f64,
        ) && nearly(transform.rc(0, 0), 1.)
            && nearly(transform.rc(1, 1), 1.)
            && nearly(transform.rc(0, 1), 0.)
            && nearly(transform.rc(1, 0), 0.)
            && integral(device_left - source.x() as f64)
            && integral(device_top - source.y() as f64);
        let sampling = SkSamplingOptions::with_filter(if *nearest_neighbor {
            SkFilterMode::kNearest
        } else {
            SkFilterMode::kLinear
        });
        let fractional = |value: f32| (value - value.round()).abs() > 1. / 4096.;
        let fractional_source = fractional(source.x())
            || fractional(source.y())
            || fractional(source.right())
            || fractional(source.bottom());
        if self.profile_input {
            let source_width = source.width().max(0.);
            let source_height = source.height().max(0.);
            let target_scale_x = (transform.rc(0, 0).powi(2) + transform.rc(1, 0).powi(2)).sqrt();
            let target_scale_y = (transform.rc(0, 1).powi(2) + transform.rc(1, 1).powi(2)).sqrt();
            let target_width = quad.geometry.visible_rect.width.max(0) as f64 * target_scale_x;
            let target_height = quad.geometry.visible_rect.height.max(0) as f64 * target_scale_y;
            self.update.tile_source_pixels = self.update.tile_source_pixels.saturating_add(
                (f64::from(source_width) * f64::from(source_height))
                    .round()
                    .max(0.) as u64,
            );
            self.update.tile_target_pixels = self
                .update
                .tile_target_pixels
                .saturating_add((target_width * target_height).round().max(0.) as u64);
            self.update.tile_linear_quads += usize::from(!*nearest_neighbor);
            self.update.tile_fractional_source_quads += usize::from(fractional_source);
            self.update.tile_scaled_quads += usize::from(
                (f64::from(source_width) - target_width).abs() > 1. / 4096.
                    || (f64::from(source_height) - target_height).abs() > 1. / 4096.,
            );
            self.update.tile_blended_quads += usize::from(blend != SkBlendMode::kSrc);
            let integer_translation = |value: f64| (value - value.round()).abs() <= 1. / 4096.;
            self.update.tile_integer_translation_quads += usize::from(
                integer_translation(transform.rc(0, 3)) && integer_translation(transform.rc(1, 3)),
            );
        }
        // Keep the immutable read lease scoped to this draw. Chromium's
        // resource lifetime protocol releases it before cc attempts partial
        // raster into the same backing on a later frame.
        let lease = resource(input, *resource_id)?;
        let alpha = skia::src::core::SkFloatingPoint::sk_float_round2int(
            shared.opacity.clamp(0., 1.) * 255.,
        )
        .clamp(0, 255) as u8;
        let mask = &shared.mask_filter_info;
        let mut rounded_corner_regions = None;
        let blit_eligible = self.n32_tile_blit
            && pixel_aligned_one_to_one
            && has_rectangular_clip
            && !antialias
            && !mask.HasGradientMask()
            && matches!(blend, SkBlendMode::kSrc | SkBlendMode::kSrcOver);
        if blit_eligible {
            self.update.tile_blit_eligible_quads += 1;
            let mapping = raster::native::MapResourceForRead(lease.as_ref())?;
            let format = mapping.format();
            if format == compositor_core::adapter::resource_types::SinglePlaneFormat::BGRA_8888
                || format == compositor_core::adapter::resource_types::SinglePlaneFormat::BGRX_8888
            {
                let mode = if blend == SkBlendMode::kSrc {
                    N32BlitMode::Src
                } else {
                    N32BlitMode::SrcOver
                };

                // Skia's sprite path can still use its row blitter under a
                // rounded clip. Decompose the rrect into three disjoint,
                // guaranteed-interior rectangles (top stem, full-width
                // middle, bottom stem), then leave only the small corner
                // pieces to Canvas. This preserves the exact rrect clip while
                // avoiding an image shader over the large interior.
                let visible = quad.geometry.visible_rect;
                let mut direct_regions = Vec::with_capacity(3);
                let mut corner_regions = Vec::with_capacity(4);
                if mask.IsEmpty() {
                    direct_regions.push(visible);
                } else if mask.HasRoundedCorners() {
                    // MaskFilterInfo is expressed in render-target space and
                    // is installed before the quad-to-target matrix. Sprite
                    // eligibility above guarantees this matrix is a pure 1:1
                    // translation, so move the mask bounds back into the
                    // quad's local space before intersecting tile geometry.
                    let target_bounds = mask.rounded_corner_bounds.rect;
                    let bounds = RectF::new(
                        target_bounds.x() - transform.rc(0, 3) as f32,
                        target_bounds.y() - transform.rc(1, 3) as f32,
                        target_bounds.width(),
                        target_bounds.height(),
                    );
                    let radii = &mask.rounded_corner_bounds.corner_radii;
                    let left = (bounds.x() + radii[0].x().max(radii[3].x())).ceil() as i32;
                    let right = (bounds.right() - radii[1].x().max(radii[2].x())).floor() as i32;
                    let top = (bounds.y() + radii[0].y().max(radii[1].y())).ceil() as i32;
                    let bottom = (bounds.bottom() - radii[3].y().max(radii[2].y())).floor() as i32;
                    let bounds = Rect::FromBounds(
                        bounds.x().ceil() as i32,
                        bounds.y().ceil() as i32,
                        bounds.right().floor() as i32,
                        bounds.bottom().floor() as i32,
                    );
                    if left <= right && top <= bottom {
                        direct_regions.push(Rect::FromBounds(left, bounds.y, right, top));
                        direct_regions.push(Rect::FromBounds(
                            bounds.x,
                            top,
                            bounds.right(),
                            bottom,
                        ));
                        direct_regions.push(Rect::FromBounds(left, bottom, right, bounds.bottom()));
                        corner_regions.push(Rect::FromBounds(bounds.x, bounds.y, left, top));
                        corner_regions.push(Rect::FromBounds(right, bounds.y, bounds.right(), top));
                        corner_regions.push(Rect::FromBounds(
                            bounds.x,
                            bottom,
                            left,
                            bounds.bottom(),
                        ));
                        corner_regions.push(Rect::FromBounds(
                            right,
                            bottom,
                            bounds.right(),
                            bounds.bottom(),
                        ));
                    }
                } else {
                    let target_bounds = mask.rounded_corner_bounds.rect;
                    let bounds = RectF::new(
                        target_bounds.x() - transform.rc(0, 3) as f32,
                        target_bounds.y() - transform.rc(1, 3) as f32,
                        target_bounds.width(),
                        target_bounds.height(),
                    );
                    direct_regions.push(Rect::FromBounds(
                        bounds.x().ceil() as i32,
                        bounds.y().ceil() as i32,
                        bounds.right().floor() as i32,
                        bounds.bottom().floor() as i32,
                    ));
                }

                for region in &mut direct_regions {
                    region.Intersect(visible);
                }
                direct_regions.retain(|region| !region.IsEmpty());
                let direct_region_count = direct_regions.len();
                let mut successful_regions = Vec::with_capacity(direct_region_count);
                for mut region in direct_regions {
                    let mut destination = Rect::new(
                        region.x + transform.rc(0, 3).round() as i32 - self.current_rect.x,
                        region.y + transform.rc(1, 3).round() as i32 - self.current_rect.y,
                        region.width,
                        region.height,
                    );
                    let original_destination = destination;
                    destination.Intersect(Rect::new(
                        0,
                        0,
                        self.current_rect.width,
                        self.current_rect.height,
                    ));
                    if let Some(scissor) = self.scissor_rect {
                        destination.Intersect(scissor);
                    }
                    if destination.IsEmpty() {
                        successful_regions.push(region);
                        continue;
                    }
                    let source_x = source.x().round() as i32
                        + region.x.saturating_sub(visible.x)
                        + destination.x.saturating_sub(original_destination.x);
                    let source_y = source.y().round() as i32
                        + region.y.saturating_sub(visible.y)
                        + destination.y.saturating_sub(original_destination.y);
                    if target.blit_n32(
                        mapping.bytes(),
                        mapping.size().width(),
                        mapping.size().height(),
                        mapping.row_bytes(),
                        source_x,
                        source_y,
                        destination.x,
                        destination.y,
                        destination.width,
                        destination.height,
                        mode,
                        alpha,
                        lease.is_opaque(),
                    ) {
                        successful_regions.push(region);
                    }
                }
                if !successful_regions.is_empty() {
                    let direct_complete = successful_regions.len() == direct_region_count;
                    self.update.tile_blit_quads += 1;
                    if mask.IsEmpty() {
                        return Ok(());
                    }
                    for region in successful_regions {
                        target.clip_rect(&rect(region), SkClipOp::kDifference, false);
                    }
                    if direct_complete && direct_region_count > 0 && corner_regions.len() == 4 {
                        rounded_corner_regions = Some(corner_regions);
                    }
                }
            }
        }
        let image_started = self.profile_input.then(std::time::Instant::now);
        let image = lease_image(lease)?;
        if let Some(started) = image_started {
            self.update.tile_image_ns = self
                .update
                .tile_image_ns
                .saturating_add(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
        }
        let mut paint = SkPaint::default();
        paint.setAlphaf(alpha as f32 / 255.);
        paint.setBlendMode(blend);
        paint.setAntiAlias(antialias);
        let draw_started = self.profile_input.then(std::time::Instant::now);
        let result = if let Some(corner_regions) = rounded_corner_regions {
            let mut result = Ok(());
            for mut region in corner_regions {
                region.Intersect(quad.geometry.visible_rect);
                if region.IsEmpty() {
                    continue;
                }
                let source_region = RectF::new(
                    source.x() + (region.x - quad.geometry.visible_rect.x) as f32,
                    source.y() + (region.y - quad.geometry.visible_rect.y) as f32,
                    region.width as f32,
                    region.height as f32,
                );
                target.save();
                target.clip_rect(&rect(region), SkClipOp::kIntersect, false);
                let draw_result = unsafe {
                    target.draw_image_rect(
                        &image,
                        Some(&rectf(source_region)),
                        &rect(region),
                        &sampling,
                        Some(&paint),
                        SrcRectConstraint::kStrict_SrcRectConstraint,
                    )
                }
                .map_err(|error| raster_error("translated rounded tile draw failed", error));
                let restore_result = target
                    .restore()
                    .map_err(|error| raster_error("translated rounded tile restore failed", error));
                result = result.and(draw_result).and(restore_result);
            }
            result
        } else {
            unsafe {
                target.draw_image_rect(
                    &image,
                    Some(&rectf(source)),
                    &rect(quad.geometry.visible_rect),
                    &sampling,
                    Some(&paint),
                    SrcRectConstraint::kStrict_SrcRectConstraint,
                )
            }
            .map_err(|error| raster_error("translated tile draw failed", error))
        };
        if let Some(started) = draw_started {
            self.update.tile_draw_ns = self
                .update
                .tile_draw_ns
                .saturating_add(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
        }
        result
    }

    // Source: components/viz/service/display/software_renderer.cc:393-431.
    // Picture geometry belongs to SoftwareRenderer; PaintOp execution remains
    // in raster's translated replay adapter and uses this exact live pass
    // target. No tile allocation or RasterEngine resource is introduced here.
    fn DrawPictureQuad(
        target: &mut RasterTarget<'_>,
        quad: &NativeDrawQuad,
        shared: &NativeSharedQuadState,
    ) -> io::Result<()> {
        let QuadMaterial::Picture {
            content_rect,
            tex_coord_rect,
            contents_scale,
            nearest_neighbor,
            image_animation_map,
            display_item_list,
            raster_inducing_scroll_offsets,
        } = &quad.material
        else {
            return Ok(());
        };
        if !image_animation_map.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "animated Picture replay requires the native PaintImage frame resolver",
            ));
        }
        let content = if tex_coord_rect.IsEmpty() {
            SkMatrix::I().clone()
        } else {
            let sx = quad.geometry.rect.width as f32 / tex_coord_rect.width();
            let sy = quad.geometry.rect.height as f32 / tex_coord_rect.height();
            SkMatrix::MakeAll(
                sx,
                0.,
                quad.geometry.rect.x as f32 - tex_coord_rect.x() * sx,
                0.,
                sy,
                quad.geometry.rect.y as f32 - tex_coord_rect.y() * sy,
                0.,
                0.,
                1.,
            )
        };
        target.save();
        let draw_result = (|| -> io::Result<()> {
            target.concat(&content);
            target.translate(-content_rect.x as f32, -content_rect.y as f32);
            target.clip_rect(&rect(*content_rect), SkClipOp::kIntersect, false);
            target.scale(*contents_scale, *contents_scale);
            raster::native::ReplayDisplayItemListIntoTranslatedTarget(
                display_item_list
                    .as_deref()
                    .ok_or_else(|| io::Error::other("Picture quad lacks display list"))?,
                *nearest_neighbor,
                raster_inducing_scroll_offsets,
                shared.opacity,
                target,
            )
        })();
        let restore_result = target
            .restore()
            .map_err(|error| raster_error("translated Picture geometry restore failed", error));
        draw_result.and(restore_result)
    }

    fn DrawRenderPassQuad(
        &mut self,
        target: &mut RasterTarget<'_>,
        input: &VizOutput,
        quad: &NativeDrawQuad,
        shared: &NativeSharedQuadState,
        blend: SkBlendMode,
        antialias: bool,
    ) -> io::Result<()> {
        let QuadMaterial::CompositorRenderPass {
            render_pass_id,
            mask_resource_id,
            mask_uv_rect,
            mask_texture_size,
            filters_scale,
            filters_origin,
            ..
        } = &quad.material
        else {
            return Ok(());
        };
        let child = input
            .Frame()
            .RenderPasses()
            .iter()
            .find(|pass| pass.RenderPassId() == *render_pass_id)
            .ok_or_else(|| io::Error::other("referenced aggregate pass is absent"))?;
        let local = SkMatrix::MakeAll(
            filters_scale.x(),
            0.,
            filters_origin.x() * filters_scale.x(),
            0.,
            filters_scale.y(),
            filters_origin.y() * filters_scale.y(),
            0.,
            0.,
            1.,
        );
        let backing = self.render_pass_bitmaps.get(render_pass_id);
        let filter_image = if child.Filters().IsEmpty() {
            None
        } else {
            Self::ApplyImageFilter(child.Filters(), backing, &local, quad.geometry.rect)?
        };
        if !child.Filters().IsEmpty() && filter_image.is_none() {
            return Ok(());
        }
        let (dest, visible, content_size) = if let Some((_, dest)) = &filter_image {
            (*dest, *dest, dest.size())
        } else if let Some(backing) = backing {
            if backing.drawn_rect.IsEmpty() {
                return Ok(());
            }
            let size = if quad.geometry.rect.IsEmpty() {
                backing.drawn_rect.size()
            } else {
                quad.geometry.rect.size()
            };
            (quad.geometry.rect, quad.geometry.visible_rect, size)
        } else {
            return Ok(());
        };
        let source = SkRect::MakeLTRB(
            0.,
            0.,
            content_size.width() as f32,
            content_size.height() as f32,
        );
        let sampling = SkSamplingOptions::with_filter(if self.linear_sampling {
            SkFilterMode::kLinear
        } else {
            SkFilterMode::kNearest
        });
        let mask = if mask_resource_id.0 == 0 {
            None
        } else {
            Some(resource(input, *mask_resource_id)?)
        };
        let mask_image = mask.map(lease_image).transpose()?;
        let mask_source = SkRect::MakeLTRB(
            mask_uv_rect.x() * mask_texture_size.width() as f32,
            mask_uv_rect.y() * mask_texture_size.height() as f32,
            mask_uv_rect.right() * mask_texture_size.width() as f32,
            mask_uv_rect.bottom() * mask_texture_size.height() as f32,
        );
        let opacity = (shared.opacity * 255.) as u8 as f32 / 255.;
        if Self::ShouldApplyBackdropFilters(child.BackdropFilters(), shared.opacity) {
            debug_assert!(
                child.Filters().IsEmpty(),
                "Filters should always be in a separate Effect node"
            );
            target.save();
            target.clip_rect(&rect(visible), SkClipOp::kIntersect, false);
            let backdrop_mask = mask_image.as_ref().map(|image| {
                (
                    Arc::clone(image),
                    mask_source.clone(),
                    dest,
                    sampling.clone(),
                )
            });
            let draw_result = Self::DrawBackdropFilter(
                target,
                child.BackdropFilters(),
                child.BackdropFilterBounds(),
                &local,
                quad.geometry.visible_rect,
                shared.opacity,
                self.current_rect.size(),
                backdrop_mask,
            );
            let restore_result = target
                .restore()
                .map_err(|error| raster_error("translated backdrop outer restore failed", error));
            draw_result.and(restore_result)?;
        }

        // Source: components/viz/service/display/software_renderer.cc:505-541.
        // Chromium hands an unfiltered, pixel-aligned render-pass image to
        // SkCanvas::drawImageRect, whose raster sprite path selects the same
        // original N32 row blitter used by the tile fast path above. Preserve
        // that terminal operation without rebuilding an image shader.
        let transform = shared.quad_to_target_transform.native();
        let nearly = |a: f64, b: f64| (a - b).abs() <= 1. / 4096.;
        let integral = |value: f64| (value - value.round()).abs() <= 1. / 4096.;
        let direct_pass_blit = child.Filters().IsEmpty()
            && mask_image.is_none()
            && shared.mask_filter_info.IsEmpty()
            && !antialias
            && matches!(blend, SkBlendMode::kSrc | SkBlendMode::kSrcOver)
            && nearly(transform.rc(0, 0), 1.)
            && nearly(transform.rc(1, 1), 1.)
            && nearly(transform.rc(0, 1), 0.)
            && nearly(transform.rc(1, 0), 0.)
            && integral(transform.rc(0, 3))
            && integral(transform.rc(1, 3))
            && dest.width == content_size.width()
            && dest.height == content_size.height();
        if direct_pass_blit {
            if let Some(backing) = backing {
                let mut region = visible;
                let mut destination = Rect::new(
                    region.x + transform.rc(0, 3).round() as i32 - self.current_rect.x,
                    region.y + transform.rc(1, 3).round() as i32 - self.current_rect.y,
                    region.width,
                    region.height,
                );
                let original_destination = destination;
                destination.Intersect(Rect::new(
                    0,
                    0,
                    self.current_rect.width,
                    self.current_rect.height,
                ));
                if let Some(scissor) = self.scissor_rect {
                    destination.Intersect(scissor);
                }
                if destination.IsEmpty() {
                    return Ok(());
                }
                region.x += destination.x - original_destination.x;
                region.y += destination.y - original_destination.y;
                region.width = destination.width;
                region.height = destination.height;
                let source_x = region.x.saturating_sub(dest.x);
                let source_y = region.y.saturating_sub(dest.y);
                let mode = if blend == SkBlendMode::kSrc {
                    N32BlitMode::Src
                } else {
                    N32BlitMode::SrcOver
                };
                if target.blit_n32(
                    bytemuck::cast_slice(backing.pixels.as_slice()),
                    backing.rect.width,
                    backing.rect.height,
                    backing.rect.width as usize * 4,
                    source_x,
                    source_y,
                    destination.x,
                    destination.y,
                    destination.width,
                    destination.height,
                    mode,
                    (opacity * 255.).round().clamp(0., 255.) as u8,
                    !child.has_transparent_background,
                ) {
                    self.update.render_pass_blit_quads += 1;
                    return Ok(());
                }
            }
        }
        // RasterFromPixmapCopy owns an immutable copy of the pass. Construct
        // it only for the SkCanvas fallback: the direct N32 path above reads
        // the live pass backing synchronously and needs no snapshot.
        let image = if let Some((filtered, _)) = &filter_image {
            filtered.image()?
        } else if let Some(backing) = backing {
            backing.image()?
        } else {
            return Ok(());
        };
        target.save();
        target.clip_rect(&rect(visible), SkClipOp::kIntersect, false);
        let draw_result = (|| -> io::Result<()> {
            if mask_image.is_some() {
                let mut layer = SkPaint::default();
                layer.setAlphaf(opacity);
                layer.setBlendMode(blend);
                target
                    .save_layer(None, Some(&layer))
                    .map_err(|error| raster_error("translated child mask layer failed", error))?;
            }
            let mut content = SkPaint::default();
            content.setAlphaf(if mask_image.is_some() { 1. } else { opacity });
            content.setBlendMode(if mask_image.is_some() {
                SkBlendMode::kSrcOver
            } else {
                blend
            });
            content.setAntiAlias(antialias);
            unsafe {
                target.draw_image_rect(
                    &image,
                    Some(&source),
                    &rect(dest),
                    &sampling,
                    Some(&content),
                    SrcRectConstraint::kFast_SrcRectConstraint,
                )
            }
            .map_err(|error| raster_error("translated child-pass draw failed", error))?;
            if let Some(mask_image) = &mask_image {
                let mut mask_paint = SkPaint::default();
                mask_paint.setAlphaf(1.);
                mask_paint.setBlendMode(SkBlendMode::kDstIn);
                mask_paint.setAntiAlias(false);
                unsafe {
                    target.draw_image_rect(
                        mask_image,
                        Some(&mask_source),
                        &rect(dest),
                        &sampling,
                        Some(&mask_paint),
                        SrcRectConstraint::kFast_SrcRectConstraint,
                    )
                }
                .map_err(|error| raster_error("translated child-pass mask draw failed", error))?;
                target.restore().map_err(|error| {
                    raster_error("translated child-pass mask layer restore failed", error)
                })?;
            }
            Ok(())
        })();
        let restore_result = target
            .restore()
            .map_err(|error| raster_error("translated child-pass restore failed", error));
        draw_result.and(restore_result)
    }

    /// Source: components/viz/service/display/software_renderer.cc:799-840.
    /// Foreground filters render into their expanded forward bounds before
    /// opacity, masks and the render-pass blend are applied by the parent.
    fn ApplyImageFilter(
        filters: &paint::cc::paint::filter_operations::FilterOperations,
        source: Option<&PassBacking>,
        local: &SkMatrix,
        quad: Rect,
    ) -> io::Result<Option<(PassBacking, Rect)>> {
        let transform = TranslatedFilterMapTransform(local);
        let bound = filters.MapRect(
            foundation::gfx::Rect::new(
                foundation::gfx::Point::new(quad.x, quad.y),
                foundation::gfx::Size::new(quad.width, quad.height),
            ),
            Some(&transform),
        );
        let bounds = Rect::new(bound.x(), bound.y(), bound.width(), bound.height());
        if bounds.IsEmpty() {
            return Ok(None);
        }
        let mut bitmap = PassBacking::new(Rect::new(0, 0, bounds.width, bounds.height))?;
        let mut target = RasterTarget::new(
            bytemuck::cast_slice_mut(bitmap.pixels.as_mut_slice()),
            bounds.width,
            bounds.height,
            bounds.width as usize * 4,
            PixelFormat::Bgra8888,
            None,
        )
        .map_err(|error| raster_error("translated filter target creation failed", error))?;
        let offset = SkMatrix::Translate(-bounds.x as f32, -bounds.y as f32);
        let mut filter_matrix = offset.clone();
        filter_matrix.preConcat(local);
        let filter =
            raster::native::BuildTranslatedFilterOperations(filters, &SkM44::from(&filter_matrix))?;
        let mut layer = SkPaint::default();
        layer.setImageFilter(filter);
        layer.setBlendMode(SkBlendMode::kSrcOver);
        target.set_matrix(&offset);
        target
            .save_layer(None, Some(&layer))
            .map_err(|error| raster_error("translated image-filter layer failed", error))?;
        if let Some(source) = source {
            let image = source.image()?;
            let source_rect =
                SkRect::MakeLTRB(0., 0., source.rect.width as f32, source.rect.height as f32);
            let destination = Rect::new(quad.x, quad.y, source.rect.width, source.rect.height);
            let sampling = SkSamplingOptions::with_filter(SkFilterMode::kNearest);
            let mut paint = SkPaint::default();
            paint.setAlphaf(1.);
            paint.setBlendMode(SkBlendMode::kSrcOver);
            unsafe {
                target.draw_image_rect(
                    &image,
                    Some(&source_rect),
                    &rect(destination),
                    &sampling,
                    Some(&paint),
                    SrcRectConstraint::kFast_SrcRectConstraint,
                )
            }
            .map_err(|error| raster_error("translated image-filter source draw failed", error))?;
        } else {
            let mut paint = SkPaint::default();
            paint.setColor4f(
                &skia::src::core::SkColor::SkColor4f {
                    fR: 0.,
                    fG: 0.,
                    fB: 0.,
                    fA: 1.,
                },
                None,
            );
            paint.setBlendMode(SkBlendMode::kSrcOver);
            unsafe { target.draw_rect(&rect(bounds), &paint) }
                .map_err(|error| raster_error("translated image-filter fill failed", error))?;
        }
        target
            .restore()
            .map_err(|error| raster_error("translated image-filter layer restore failed", error))?;
        target
            .finish()
            .map_err(|error| raster_error("translated image-filter target finish failed", error))?;
        bitmap.drawn_rect = Rect::new(0, 0, bounds.width, bounds.height);
        Ok(Some((bitmap, bounds)))
    }

    fn ShouldApplyBackdropFilters(
        filters: &paint::cc::paint::filter_operations::FilterOperations,
        opacity: f32,
    ) -> bool {
        !filters.IsEmpty() && opacity != 0.
    }

    /// Source: components/viz/service/display/software_renderer.cc:842-872.
    fn ApplyBackdropFilterWithExactOutputSize(
        filters: &paint::cc::paint::filter_operations::FilterOperations,
        source: &PassBacking,
        local: &SkMatrix,
        output: Rect,
    ) -> io::Result<PassBacking> {
        let mut trace = browser_tracing::span("renderer", "Renderer.BackdropFilter");
        trace.set("width", output.width as f64);
        trace.set("height", output.height as f64);
        let transform = TranslatedFilterMapTransform(local);
        let input = filters.MapRectReverse(
            foundation::gfx::Rect::new(
                foundation::gfx::Point::new(output.x, output.y),
                foundation::gfx::Size::new(output.width, output.height),
            ),
            Some(&transform),
        );
        let reverse = Rect::new(input.x(), input.y(), input.width(), input.height());
        let needs_mirror = !output.Contains(reverse);
        let crop = if needs_mirror {
            let mut crop = PassBacking::new(Rect::new(0, 0, output.width, output.height))?;
            let mut target = RasterTarget::new(
                bytemuck::cast_slice_mut(crop.pixels.as_mut_slice()),
                output.width,
                output.height,
                output.width as usize * 4,
                PixelFormat::Bgra8888,
                None,
            )
            .map_err(|error| raster_error("translated backdrop crop allocation failed", error))?;
            let image = source.image()?;
            let source_rect =
                SkRect::MakeLTRB(0., 0., source.rect.width as f32, source.rect.height as f32);
            let mut paint = SkPaint::default();
            paint.setBlendMode(SkBlendMode::kSrc);
            unsafe {
                target.draw_image_rect(
                    &image,
                    Some(&source_rect),
                    &source_rect,
                    &SkSamplingOptions::with_filter(SkFilterMode::kNearest),
                    Some(&paint),
                    SrcRectConstraint::kFast_SrcRectConstraint,
                )
            }
            .map_err(|error| raster_error("translated backdrop crop draw failed", error))?;
            target
                .finish()
                .map_err(|error| raster_error("translated backdrop crop finish failed", error))?;
            crop.drawn_rect = crop.rect;
            Some(crop)
        } else {
            None
        };

        let mut result = PassBacking::new(Rect::new(0, 0, output.width, output.height))?;
        let mut target = RasterTarget::new(
            bytemuck::cast_slice_mut(result.pixels.as_mut_slice()),
            output.width,
            output.height,
            output.width as usize * 4,
            PixelFormat::Bgra8888,
            None,
        )
        .map_err(|error| raster_error("translated backdrop output allocation failed", error))?;
        let offset = SkMatrix::Translate(-output.x as f32, -output.y as f32);
        let mut filter_matrix = offset.clone();
        filter_matrix.preConcat(local);
        let filter =
            raster::native::BuildTranslatedFilterOperations(filters, &SkM44::from(&filter_matrix))?;
        let mut layer = SkPaint::default();
        layer.setImageFilter(filter);
        layer.setBlendMode(SkBlendMode::kSrcOver);
        target.set_matrix(&offset);
        target
            .save_layer(None, Some(&layer))
            .map_err(|error| raster_error("translated backdrop filter layer failed", error))?;
        let sampling = SkSamplingOptions::with_filter(SkFilterMode::kNearest);
        if let Some(crop) = &crop {
            let image = crop.image()?;
            let local_matrix = SkMatrix::Translate(output.x as f32, output.y as f32);
            let shader = image
                .makeShaderWithMatrix(
                    SkTileMode::kMirror,
                    SkTileMode::kMirror,
                    &sampling,
                    &local_matrix,
                )
                .ok_or_else(|| io::Error::other("translated backdrop mirror shader failed"))?;
            let mut paint = SkPaint::default();
            paint.setShader(Some(shader));
            paint.setBlendMode(SkBlendMode::kSrcOver);
            unsafe { target.draw_rect(&rect(reverse), &paint) }
                .map_err(|error| raster_error("translated backdrop mirror draw failed", error))?;
        } else {
            let image = source.image()?;
            let source_rect =
                SkRect::MakeLTRB(0., 0., source.rect.width as f32, source.rect.height as f32);
            let destination = Rect::new(output.x, output.y, source.rect.width, source.rect.height);
            let mut paint = SkPaint::default();
            paint.setBlendMode(SkBlendMode::kSrcOver);
            unsafe {
                target.draw_image_rect(
                    &image,
                    Some(&source_rect),
                    &rect(destination),
                    &sampling,
                    Some(&paint),
                    SrcRectConstraint::kFast_SrcRectConstraint,
                )
            }
            .map_err(|error| raster_error("translated backdrop source draw failed", error))?;
        }
        target.restore().map_err(|error| {
            raster_error("translated backdrop filter layer restore failed", error)
        })?;
        target
            .finish()
            .map_err(|error| raster_error("translated backdrop output finish failed", error))?;
        result.drawn_rect = result.rect;
        Ok(result)
    }

    /// Source: components/viz/service/display/software_renderer.cc:888-1028.
    fn DrawBackdropFilter(
        target: &mut RasterTarget<'_>,
        filters: &paint::cc::paint::filter_operations::FilterOperations,
        bounds: Option<&SkPath>,
        filter_matrix: &SkMatrix,
        visible: Rect,
        opacity: f32,
        viewport: Size,
        mask: Option<(Arc<dyn SkImage>, SkRect, Rect, SkSamplingOptions)>,
    ) -> io::Result<()> {
        if filters.IsEmpty() || opacity == 0. {
            return Ok(());
        }
        let content = if let Some(path) = bounds {
            filter_matrix.mapRect_value(path.getBounds())
        } else {
            rect(visible)
        };
        if !content.isFinite() || content.isEmpty() {
            return Ok(());
        }
        let content = SkRect::MakeLTRB(
            content.left().floor(),
            content.top().floor(),
            content.right().ceil(),
            content.bottom().ceil(),
        );
        let matrix = target.local_to_device().asM33();
        let Some(inverse) = matrix.invert() else {
            return Ok(());
        };
        let device = matrix.mapRect_value(&content);
        if !device.isFinite() || device.isEmpty() {
            return Ok(());
        }
        let left = (device.left().floor() as i32).max(0);
        let top = (device.top().floor() as i32).max(0);
        let right = (device.right().ceil() as i32).min(viewport.width());
        let bottom = (device.bottom().ceil() as i32).min(viewport.height());
        if right <= left || bottom <= top {
            return Ok(());
        }
        let device_rect = SkRect::MakeLTRB(left as f32, top as f32, right as f32, bottom as f32);
        let content = inverse.mapRect_value(&device_rect);
        if !content.isFinite() || content.isEmpty() {
            return Ok(());
        }
        let output_left = content.left().floor() as i32;
        let output_top = content.top().floor() as i32;
        let output_right = content.right().ceil() as i32;
        let output_bottom = content.bottom().ceil() as i32;
        let output = Rect::new(
            output_left,
            output_top,
            output_right - output_left,
            output_bottom - output_top,
        );
        if output.IsEmpty() {
            return Ok(());
        }

        let mut captured = PassBacking::new(Rect::new(0, 0, right - left, bottom - top))?;
        let info = image_info(
            captured.rect.width,
            captured.rect.height,
            compositor_core::adapter::resource_types::SinglePlaneFormat::BGRA_8888,
        )?;
        let row_bytes = captured.rect.width as usize * 4;
        if !target
            .read_pixels(
                &info,
                bytemuck::cast_slice_mut(captured.pixels.as_mut_slice()),
                row_bytes,
                left,
                top,
            )
            .map_err(|error| raster_error("translated backdrop readback failed", error))?
        {
            return Ok(());
        }
        captured.drawn_rect = captured.rect;
        let filtered = Self::ApplyBackdropFilterWithExactOutputSize(
            filters,
            &captured,
            filter_matrix,
            output,
        )?;

        let mut clipped = PassBacking::new(Rect::new(0, 0, output.width, output.height))?;
        let clipped_image = {
            let mut clip_target = RasterTarget::new(
                bytemuck::cast_slice_mut(clipped.pixels.as_mut_slice()),
                output.width,
                output.height,
                output.width as usize * 4,
                PixelFormat::Bgra8888,
                None,
            )
            .map_err(|error| raster_error("translated backdrop clip allocation failed", error))?;
            let mut image_tf = SkMatrix::Translate(-(left as f32), -(top as f32));
            image_tf.preConcat(&matrix);
            if let Some(path) = bounds {
                let mut clip_matrix = image_tf.clone();
                clip_matrix.preConcat(filter_matrix);
                clip_target.set_matrix(&clip_matrix);
                clip_target.clip_path(path, SkClipOp::kIntersect, true);
                clip_target.set_matrix(SkMatrix::I());
            }
            let image = filtered.image()?;
            let image_rect = SkRect::MakeLTRB(0., 0., output.width as f32, output.height as f32);
            let mut paint = SkPaint::default();
            paint.setAlphaf(opacity);
            paint.setBlendMode(SkBlendMode::kSrcOver);
            unsafe {
                clip_target.draw_image_rect(
                    &image,
                    Some(&image_rect),
                    &image_rect,
                    &SkSamplingOptions::with_filter(SkFilterMode::kNearest),
                    Some(&paint),
                    SrcRectConstraint::kStrict_SrcRectConstraint,
                )
            }
            .map_err(|error| raster_error("translated backdrop clip draw failed", error))?;
            clip_target
                .finish()
                .map_err(|error| raster_error("translated backdrop clip finish failed", error))?;
            clipped.drawn_rect = clipped.rect;
            clipped.image()?
        };

        let mut image_tf = SkMatrix::Translate(-(left as f32), -(top as f32));
        image_tf.preConcat(&matrix);
        let Some(image_tf_inv) = image_tf.invert() else {
            return Ok(());
        };
        target.save();
        let draw_result = (|| -> io::Result<()> {
            let mut clip_matrix = matrix.clone();
            clip_matrix.preConcat(&image_tf_inv);
            target.set_matrix(&clip_matrix);
            target.clip_rect(
                &SkRect::MakeLTRB(0., 0., output.width as f32, output.height as f32),
                SkClipOp::kIntersect,
                false,
            );
            target.set_matrix(&matrix);
            if mask.is_some() {
                target.save_layer(None, None).map_err(|error| {
                    raster_error("translated backdrop mask layer failed", error)
                })?;
            }
            let sampling = SkSamplingOptions::with_filter(SkFilterMode::kNearest);
            let shader = clipped_image
                .makeShaderWithMatrix(
                    SkTileMode::kDecal,
                    SkTileMode::kDecal,
                    &sampling,
                    &image_tf_inv,
                )
                .ok_or_else(|| io::Error::other("translated backdrop decal shader failed"))?;
            let mut paint = SkPaint::default();
            paint.setShader(Some(shader));
            paint.setBlendMode(SkBlendMode::kSrcOver);
            unsafe { target.draw_rect(&rect(visible), &paint) }
                .map_err(|error| raster_error("translated backdrop shader draw failed", error))?;
            if let Some((image, source, dest, sampling)) = &mask {
                let mut paint = SkPaint::default();
                paint.setBlendMode(SkBlendMode::kDstIn);
                unsafe {
                    target.draw_image_rect(
                        image,
                        Some(source),
                        &rect(*dest),
                        sampling,
                        Some(&paint),
                        SrcRectConstraint::kFast_SrcRectConstraint,
                    )
                }
                .map_err(|error| raster_error("translated backdrop mask draw failed", error))?;
                target.restore().map_err(|error| {
                    raster_error("translated backdrop mask layer restore failed", error)
                })?;
            }
            Ok(())
        })();
        let restore_result = target
            .restore()
            .map_err(|error| raster_error("translated backdrop shader restore failed", error));
        draw_result.and(restore_result)
    }

    fn DidChangeVisibility(&mut self, _visible: bool) {}

    fn UpdateRenderPassTextures(
        &mut self,
        _input: &VizOutput,
        requirements: &BTreeMap<RenderPassId, RenderPassRequirements>,
    ) -> io::Result<()> {
        let mut delete = Vec::new();
        for (id, backing) in &self.render_pass_bitmaps {
            let Some(required) = requirements.get(id) else {
                delete.push(*id);
                continue;
            };
            if backing.rect.width < required.size.width()
                || backing.rect.height < required.size.height()
            {
                delete.push(*id);
            }
        }
        for id in delete {
            self.render_pass_bitmaps.remove(&id);
        }
        Ok(())
    }

    fn AllocateRenderPassResourceIfNeeded(
        &mut self,
        id: RenderPassId,
        requirements: &RenderPassRequirements,
    ) -> io::Result<()> {
        if self.render_pass_bitmaps.contains_key(&id) {
            return Ok(());
        }
        self.render_pass_bitmaps.insert(
            id,
            PassBacking::new(Rect::new(
                0,
                0,
                requirements.size.width(),
                requirements.size.height(),
            ))?,
        );
        Ok(())
    }

    fn IsRenderPassResourceAllocated(&self, id: RenderPassId) -> bool {
        self.render_pass_bitmaps.contains_key(&id)
    }

    fn GetRenderPassBackingPixelSize(&self, id: RenderPassId) -> Size {
        self.render_pass_bitmaps
            .get(&id)
            .map_or(Size::default(), |backing| backing.rect.size())
    }

    fn SetRenderPassBackingDrawnRect(&mut self, id: RenderPassId, value: Rect) {
        if let Some(backing) = self.render_pass_bitmaps.get_mut(&id) {
            backing.drawn_rect = value;
        }
    }

    fn GetRenderPassBackingDrawnRect(&self, id: RenderPassId) -> Rect {
        self.render_pass_bitmaps
            .get(&id)
            .map_or(Rect::default(), |backing| backing.drawn_rect)
    }

    fn QueueDrawQuad(&mut self, index: usize, draw_region: Option<&[PointF; 4]>) {
        self.pending_draws.push(PendingDraw {
            index,
            draw_region: draw_region.copied(),
            scissor_rect: self.scissor_rect,
        });
    }

    fn FinishDrawingRenderPass(
        &mut self,
        input: &VizOutput,
        pass: &AggregatedRenderPass,
    ) -> io::Result<()> {
        let Some(id) = self.current_pass.take() else {
            return Ok(());
        };
        if id != pass.RenderPassId() {
            return Err(io::Error::other(
                "translated renderer finished a different render pass",
            ));
        }
        let mut backing = self
            .render_pass_bitmaps
            .remove(&id)
            .ok_or_else(|| io::Error::other("translated active pass backing disappeared"))?;
        let draws = std::mem::take(&mut self.pending_draws);
        let pending_clear = self.pending_clear.take();
        let result = match RasterTarget::new(
            bytemuck::cast_slice_mut(backing.pixels.as_mut_slice()),
            backing.rect.width,
            backing.rect.height,
            backing.rect.width as usize * 4,
            PixelFormat::Bgra8888,
            None,
        ) {
            Ok(mut target) => {
                let draw_result = (|| -> io::Result<()> {
                    if let Some((bounds, transparent)) = pending_clear {
                        let mut paint = SkPaint::default();
                        paint.setBlendMode(SkBlendMode::kSrc);
                        paint.setAntiAlias(false);
                        paint.setColor4f(
                            &skia::src::core::SkColor::SkColor4f {
                                fR: 0.,
                                fG: 0.,
                                fB: if transparent || !cfg!(debug_assertions) {
                                    0.
                                } else {
                                    1.
                                },
                                fA: if transparent { 0. } else { 1. },
                            },
                            None,
                        );
                        unsafe { target.draw_rect(&rect(bounds), &paint) }.map_err(|error| {
                            raster_error("translated framebuffer clear failed", error)
                        })?;
                    }
                    for draw in draws {
                        self.scissor_rect = draw.scissor_rect;
                        self.DrawQuadIntoTarget(
                            &mut target,
                            input,
                            pass,
                            draw.index,
                            draw.draw_region.as_ref(),
                        )?;
                    }
                    Ok(())
                })();
                let finish_result = target.finish().map_err(|error| {
                    raster_error("translated render-pass target finish failed", error)
                });
                match (draw_result, finish_result) {
                    (Ok(()), Ok(())) => Ok(()),
                    (Err(error), _) | (_, Err(error)) => Err(error),
                }
            }
            Err(error) => Err(raster_error(
                "translated render-pass target creation failed",
                error,
            )),
        };
        if result.is_ok() {
            backing.drawn_rect = self.current_rect;
        }
        let old = self.render_pass_bitmaps.insert(id, backing);
        debug_assert!(old.is_none());
        result
    }

    fn GetCurrentFramebufferDamage(&self, id: RenderPassId) -> Rect {
        self.framebuffer_damage
            .get(&id)
            .copied()
            .unwrap_or_default()
    }

    fn AbortDrawingFrame(&mut self) {
        self.current_pass = None;
        self.pending_draws.clear();
        self.pending_clear = None;
        self.render_pass_bitmaps.clear();
        self.framebuffer_damage.clear();
        self.drawn_damage.clear();
        self.frame_target = None;
        self.direct_root_pass = None;
    }
}

impl RendererBackend for TranslatedSoftwareRenderer {
    type Target = RenderTarget;

    fn Configure(&mut self, settings: &RendererSettings) {
        self.settings = settings.clone();
    }

    fn ValidateTarget(&self, input: &VizOutput, target: &RenderTarget) -> io::Result<Size> {
        validate_target(input, target)?;
        Ok(Size::new(target.width as i32, target.height as i32))
    }

    fn BindFrameTarget(&mut self, input: &VizOutput, target: RenderTarget) -> io::Result<()> {
        Self::BindFrameTarget(self, input, target)
    }

    fn ReleaseFrameTarget(
        &mut self,
        input: &VizOutput,
    ) -> io::Result<(RenderUpdate, RenderTarget)> {
        Self::ReleaseFrameTarget(self, input)
    }

    fn CanPartialSwap(&self) -> bool {
        Self::CanPartialSwap(self)
    }

    fn BeginDrawingFrame(&mut self, input: &VizOutput) -> io::Result<()> {
        Self::BeginDrawingFrame(self, input)
    }

    fn FinishDrawingFrame(&mut self) -> io::Result<()> {
        Self::FinishDrawingFrame(self)
    }

    fn EnsureScissorTestDisabled(&mut self) {
        Self::EnsureScissorTestDisabled(self)
    }

    fn SetScissorTestRect(&mut self, rect: Rect) {
        Self::SetScissorTestRect(self, rect)
    }

    fn BeginDrawingRenderPass(
        &mut self,
        pass: &AggregatedRenderPass,
        needs_clear: bool,
        update: Rect,
        viewport: Size,
    ) -> io::Result<()> {
        Self::BeginDrawingRenderPass(self, pass, needs_clear, update, viewport)
    }

    fn DoDrawQuad(
        &mut self,
        _input: &VizOutput,
        _pass: &AggregatedRenderPass,
        index: usize,
        draw_region: Option<&[PointF; 4]>,
    ) -> io::Result<()> {
        self.QueueDrawQuad(index, draw_region);
        Ok(())
    }

    fn FinishDrawingRenderPass(
        &mut self,
        input: &VizOutput,
        pass: &AggregatedRenderPass,
    ) -> io::Result<()> {
        Self::FinishDrawingRenderPass(self, input, pass)
    }

    fn GetCurrentFramebufferDamage(&self, id: RenderPassId) -> Rect {
        Self::GetCurrentFramebufferDamage(self, id)
    }

    fn DidChangeVisibility(&mut self, visible: bool) {
        Self::DidChangeVisibility(self, visible)
    }

    fn UpdateRenderPassTextures(
        &mut self,
        input: &VizOutput,
        requirements: &BTreeMap<RenderPassId, RenderPassRequirements>,
    ) -> io::Result<()> {
        Self::UpdateRenderPassTextures(self, input, requirements)
    }

    fn AllocateRenderPassResourceIfNeeded(
        &mut self,
        id: RenderPassId,
        requirements: &RenderPassRequirements,
    ) -> io::Result<()> {
        Self::AllocateRenderPassResourceIfNeeded(self, id, requirements)
    }

    fn IsRenderPassResourceAllocated(&self, id: RenderPassId) -> bool {
        Self::IsRenderPassResourceAllocated(self, id)
    }

    fn GetRenderPassBackingPixelSize(&self, id: RenderPassId) -> Size {
        Self::GetRenderPassBackingPixelSize(self, id)
    }

    fn SetRenderPassBackingDrawnRect(&mut self, id: RenderPassId, rect: Rect) {
        Self::SetRenderPassBackingDrawnRect(self, id, rect)
    }

    fn GetRenderPassBackingDrawnRect(&self, id: RenderPassId) -> Rect {
        Self::GetRenderPassBackingDrawnRect(self, id)
    }

    fn AbortDrawingFrame(&mut self) {
        Self::AbortDrawingFrame(self)
    }
}
