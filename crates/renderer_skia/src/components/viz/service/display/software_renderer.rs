// Source: components/viz/service/display/software_renderer.h:1-140
// Source: components/viz/service/display/software_renderer.cc:1-1119
// CPU execution translation. Host device and immutable resource adapters replace
// OutputSurface and DisplayResourceProvider; traversal remains DirectRenderer's.
#![allow(non_snake_case)]
use super::renderer_utils::IsScaleAndIntegerTranslate;
use crate::{
    native::{PictureRasterRequest, PictureRasterizer, RenderTarget},
    software_adapter::*,
};
use compositor_core::{
    adapter::{
        geometry::{PointF, Rect, RectF, Size},
        property_types::BlendMode,
    },
    runtime::frame::{QuadMaterial, RenderPassId},
};
use renderer::{RenderPassRequirements, RenderUpdate, RendererBackend, RendererSettings};
use skia::{
    compat::commands::{self as command, CommandKind, DrawCommand, PaintRect, ResourceContext},
    cpu::canvas::Canvas,
    raster::{FilterQuality, PixelStorage, Transform as SkTransform},
};
use std::{collections::BTreeMap, io};
use viz::common::quads::aggregated_render_pass::{NativeDrawQuad, NativeSharedQuadState};
use viz::native::{AggregatedRenderPass, VizOutput};

// Source: components/viz/service/display/software_renderer.h:81-138. SkBitmap ownership is represented by the
// CPU backing adapter; a Canvas temporarily owns its allocation during a pass.
#[derive(Default)]
pub struct SoftwareRenderer {
    render_pass_bitmaps: BTreeMap<RenderPassId, PassBacking>,
    picture_rasterizer: Option<Box<dyn PictureRasterizer>>,
    settings: RendererSettings,
    update: RenderUpdate,
    current_canvas: Option<Canvas>,
    current_pass: Option<RenderPassId>,
    current_rect: Rect,
    scissor_rect: Option<Rect>,
    linear_sampling: bool,
    profile_input: bool,
    framebuffer_damage: BTreeMap<RenderPassId, Rect>,
    drawn_damage: BTreeMap<RenderPassId, Rect>,
}
impl SoftwareRenderer {
    // Source: components/viz/service/display/software_renderer.cc:112-125. Device/provider are typed adapters.
    pub fn new(rasterizer: Box<dyn PictureRasterizer>) -> Self {
        Self {
            picture_rasterizer: Some(rasterizer),
            ..Self::default()
        }
    }
    // Source: components/viz/service/display/software_renderer.cc:127-129.
    fn CanPartialSwap(&self) -> bool {
        true
    }
    // Source: components/viz/service/display/software_renderer.cc:131-133. Trace instrumentation excluded.
    fn BeginDrawingFrame(&mut self, input: &VizOutput) -> io::Result<()> {
        self.update = RenderUpdate::default();
        // Local profiler adapter: cache the environment decision once per frame.
        self.profile_input = std::env::var_os("BROWSER_PROFILE_INPUT").is_some();
        self.framebuffer_damage.clear();
        self.drawn_damage.clear();
        for pass in input.Frame().RenderPasses() {
            if input.RootDestination(pass.RenderPassId()).is_some()
                && self
                    .render_pass_bitmaps
                    .get(&pass.RenderPassId())
                    .is_none_or(|b| b.drawn_rect != pass.OutputRect())
            {
                self.framebuffer_damage
                    .insert(pass.RenderPassId(), pass.OutputRect());
            }
        }
        Ok(())
    }
    // Source: components/viz/service/display/software_renderer.cc:135-145. Host EndPaint is publish's boundary.
    fn FinishDrawingFrame(&mut self) -> io::Result<()> {
        self.FinishDrawingRenderPass()?;
        self.current_canvas = None;
        Ok(())
    }
    // SOURCE_EXCLUDED: components/viz/service/display/software_renderer.cc:147-154. Actual SwapBuffers belongs
    // to host presentation; Renderer returns ready pixels without driving it.
    // Source: components/viz/service/display/software_renderer.cc:156-163.
    fn EnsureScissorTestDisabled(&mut self) {
        self.scissor_rect = None;
    }
    fn SetScissorTestRect(&mut self, rect: Rect) {
        self.scissor_rect = Some(rect);
    }
    // Source: components/viz/service/display/software_renderer.cc:165-185. Clip is specified in device space.
    fn SetClipRect(canvas: &mut Canvas, rect: Rect) {
        let matrix = canvas.local_to_device();
        canvas.set_matrix(SkTransform::identity());
        clip_rect(canvas, rect);
        canvas.set_matrix(matrix);
    }
    // Source: components/viz/service/display/software_renderer.cc:187-201. The target-to-device translation
    // is installed before clipping; the content matrix is installed afterward.
    fn SetClipRRect(canvas: &mut Canvas, shared: &NativeSharedQuadState) {
        let r = &shared.mask_filter_info.rounded_corner_bounds;
        let corner = |i: usize| command::PaintCornerRadius {
            x: r.corner_radii[i].x() as f64,
            y: r.corner_radii[i].y() as f64,
        };
        canvas.replay_item(
            &DrawCommand {
                r#type: CommandKind::kClipRoundedRect,
                rect: rectf(r.rect),
                corner_radii: command::PaintCornerRadii {
                    top_left: corner(0),
                    top_right: corner(1),
                    bottom_right: corner(2),
                    bottom_left: corner(3),
                },
                antialias: true,
                ..Default::default()
            },
            &ResourceContext::default(),
        );
    }
    // Source: components/viz/service/display/software_renderer.cc:203-216.
    fn ClearCanvas(&mut self, color: command::Color) {
        if let Some(canvas) = &mut self.current_canvas {
            let bounds = self.scissor_rect.unwrap_or(Rect::new(
                0,
                0,
                self.current_rect.width,
                self.current_rect.height,
            ));
            canvas.draw_native_rect(rect(bounds), color, skia::BlendMode::kSrc, false);
        }
    }
    // Source: components/viz/service/display/software_renderer.cc:218-228.
    fn ClearFramebuffer(&mut self, transparent: bool) {
        if transparent {
            self.ClearCanvas(command::Color {
                red: 0.,
                green: 0.,
                blue: 0.,
                alpha: 0.,
            });
        } else if cfg!(debug_assertions) {
            self.ClearCanvas(command::Color {
                red: 0.,
                green: 0.,
                blue: 1.,
                alpha: 1.,
            });
        }
    }
    // Source: components/viz/service/display/software_renderer.cc:230-266. Root/output device BeginPaint is
    // adapted to the same CPU allocation as offscreen passes, then published
    // into the borrowed host target. No mutable target reference is retained.
    fn BeginDrawingRenderPass(
        &mut self,
        pass: &AggregatedRenderPass,
        needs_clear: bool,
        update: Rect,
        viewport: Size,
    ) -> io::Result<()> {
        self.FinishDrawingRenderPass()?;
        let backing = self
            .render_pass_bitmaps
            .get_mut(&pass.RenderPassId())
            .ok_or_else(|| io::Error::other("render pass bitmap not allocated"))?;
        backing.rect.x = pass.OutputRect().x;
        backing.rect.y = pass.OutputRect().y;
        let pixels = std::mem::take(&mut backing.pixels);
        self.current_canvas = Some(
            Canvas::make_raster_direct_with_format_preserving(
                &ResourceContext::default(),
                backing.rect.width as u32,
                backing.rect.height as u32,
                backing.rect.width as usize * 4,
                PixelStorage::owned(pixels),
                skia::PixelFormat::Bgra8888,
                false,
            )
            .ok_or_else(|| io::Error::other("Skia rejected render pass bitmap"))?,
        );
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
        if needs_clear {
            self.ClearFramebuffer(pass.has_transparent_background);
        }
        self.update.passes += 1;
        Ok(())
    }
    // Source: components/viz/service/display/software_renderer.cc:268-270. Resource leases are CPU-only by type.
    // Source: components/viz/service/display/software_renderer.cc:272-373.
    fn DoDrawQuad(
        &mut self,
        input: &VizOutput,
        pass: &AggregatedRenderPass,
        index: usize,
        draw_region: Option<&[PointF; 4]>,
    ) -> io::Result<()> {
        let Some(mut canvas) = self.current_canvas.take() else {
            return Ok(());
        };
        let quad_started = self.profile_input.then(std::time::Instant::now);
        let quad = &pass.Quads()[index];
        let shared = pass
            .TrySharedQuadState(
                quad.geometry
                    .shared_quad_state
                    .ok_or_else(|| io::Error::other("quad lacks SharedQuadState"))?,
            )
            .ok_or_else(|| io::Error::other("invalid SharedQuadState handle"))?;
        let rounded = renderer::ShouldApplyRoundedCorner(quad, shared);
        // A save also confines the CPU receiver's per-material visible clip.
        simple(&mut canvas, CommandKind::kSave);
        if let Some(scissor) = self.scissor_rect {
            Self::SetClipRect(&mut canvas, scissor);
        }
        canvas.set_matrix(SkTransform::from_translate(
            -self.current_rect.x as f32,
            -self.current_rect.y as f32,
        ));
        if rounded {
            Self::SetClipRRect(&mut canvas, shared);
        }
        canvas.set_matrix(native_transform(
            shared.quad_to_target_transform.native(),
            self.current_rect,
        )?);
        let matrix = canvas.local_to_device();
        let antialias = if self.settings.force_antialiasing || !IsScaleAndIntegerTranslate(matrix) {
            self.linear_sampling = true;
            let exterior = quad.geometry.rect == shared.quad_layer_rect;
            self.settings.allow_antialiasing && (self.settings.force_antialiasing || exterior)
        } else {
            false
        };
        let should_blend = quad.geometry.needs_blending
            || shared.opacity < 1.
            || !shared.mask_filter_info.IsEmpty()
            || shared.blend_mode != BlendMode::kSrcOver;
        let blend = if should_blend {
            skia_blend_mode(shared.blend_mode)
        } else {
            skia::BlendMode::kSrc
        };
        if let Some(region) = draw_region {
            // The source draw_region is relative to visible_rect's origin.
            // DirectRenderer supplies the four local BSP intersection points.
            use skia::compat::commands::{Offset, PaintPathCommand, PaintPathVerb};
            let path = region
                .iter()
                .enumerate()
                .map(|(i, p)| PaintPathCommand {
                    verb: if i == 0 {
                        PaintPathVerb::kMoveTo
                    } else {
                        PaintPathVerb::kLineTo
                    },
                    point: Offset {
                        x: p.x() as f64 - quad.geometry.visible_rect.x as f64,
                        y: p.y() as f64 - quad.geometry.visible_rect.y as f64,
                    },
                    ..Default::default()
                })
                .collect();
            canvas.replay_clip_path(
                &DrawCommand {
                    r#type: CommandKind::kClipPath,
                    path,
                    ..Default::default()
                },
                false,
            );
        }
        match &quad.material {
            QuadMaterial::CompositorRenderPass { .. } => {
                self.DrawRenderPassQuad(&mut canvas, input, quad, shared, blend, antialias)?
            }
            QuadMaterial::DebugBorder { .. } => {
                Self::DrawDebugBorderQuad(&mut canvas, quad, shared, blend)
            }
            QuadMaterial::Picture { .. } => {
                canvas = self.DrawPictureQuad(canvas, quad, shared)?;
            }
            QuadMaterial::SolidColor { .. } => {
                Self::DrawSolidColorQuad(&mut canvas, quad, shared, blend, antialias);
                self.update.solid_quads += 1;
            }
            QuadMaterial::Tile { .. } => {
                Self::DrawTileQuad(&mut canvas, input, quad, shared, blend, antialias)?;
                self.update.tile_quads += 1;
            }
            QuadMaterial::SharedElement { .. } => {
                return Err(io::Error::other(
                    "unresolved SharedElement reached SoftwareRenderer",
                ))
            }
        }
        // Local profiler adapter: measure setup plus the material execution,
        // matching RenderUpdate's existing per-material replay timing contract.
        if let Some(started) = quad_started {
            let elapsed = started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
            let counter = match &quad.material {
                QuadMaterial::Tile { .. } => Some(&mut self.update.tile_ns),
                QuadMaterial::Picture { .. } => Some(&mut self.update.picture_ns),
                QuadMaterial::SolidColor { .. } => Some(&mut self.update.solid_ns),
                _ => None,
            };
            if let Some(counter) = counter {
                *counter = counter.saturating_add(elapsed);
            }
        }
        canvas.set_matrix(SkTransform::identity());
        simple(&mut canvas, CommandKind::kRestore);
        self.update.quads += 1;
        // Existing RenderUpdate semantics: replayed visible quad pixels,
        // including overlap. Actual pass damage is recorded in drawn_damage.
        self.update.damage_pixels = self.update.damage_pixels.saturating_add(
            quad.geometry.visible_rect.width.max(0) as usize
                * quad.geometry.visible_rect.height.max(0) as usize,
        );
        self.current_canvas = Some(canvas);
        Ok(())
    }
    // Source: components/viz/service/display/software_renderer.cc:375-391.
    fn DrawDebugBorderQuad(
        canvas: &mut Canvas,
        quad: &NativeDrawQuad,
        shared: &NativeSharedQuadState,
        blend: skia::BlendMode,
    ) {
        if let QuadMaterial::DebugBorder { color: c, width } = &quad.material {
            canvas.draw_native_debug_border(
                rect(quad.geometry.rect),
                color(*c, shared.opacity),
                *width as f32,
                blend,
                false,
            );
        }
    }
    // Source: components/viz/service/display/software_renderer.cc:393-431. The immutable display list's Raster
    // capability performs translate(-content origin), clip, scale and animated
    // image selection inside its owning raster engine.
    fn DrawPictureQuad(
        &mut self,
        mut canvas: Canvas,
        quad: &NativeDrawQuad,
        shared: &NativeSharedQuadState,
    ) -> io::Result<Canvas> {
        if let QuadMaterial::Picture {
            content_rect,
            tex_coord_rect,
            contents_scale,
            nearest_neighbor,
            image_animation_map,
            display_item_list,
            raster_inducing_scroll_offsets,
        } = &quad.material
        {
            // Source: components/viz/service/display/software_renderer.cc:394-395. Source rectangle maps content into quad space.
            let content = if tex_coord_rect.IsEmpty() {
                SkTransform::identity()
            } else {
                let sx = quad.geometry.rect.width as f32 / tex_coord_rect.width();
                let sy = quad.geometry.rect.height as f32 / tex_coord_rect.height();
                SkTransform::from_row(
                    sx,
                    0.,
                    0.,
                    sy,
                    quad.geometry.rect.x as f32 - tex_coord_rect.x() * sx,
                    quad.geometry.rect.y as f32 - tex_coord_rect.y() * sy,
                )
            };
            canvas.set_matrix(canvas.local_to_device().pre_concat(content));
            // Source: components/viz/service/display/software_renderer.cc:397-430. The raster receiver uses this exact live canvas,
            // applies per-primitive opacity/filtering, and replays content origin,
            // clipping, scale, selected image frames and inducing scroll offsets.
            canvas = self
                .picture_rasterizer
                .as_deref_mut()
                .ok_or_else(|| io::Error::other("Picture quad requires CPU Raster capability"))?
                .RasterizeInto(
                    PictureRasterRequest {
                        display_list: display_item_list
                            .as_deref()
                            .ok_or_else(|| io::Error::other("Picture quad lacks display list"))?,
                        content_rect: *content_rect,
                        contents_scale: *contents_scale,
                        nearest_neighbor: *nearest_neighbor,
                        image_animation_map,
                        raster_inducing_scroll_offsets,
                    },
                    canvas,
                    shared.opacity,
                )?;
            self.update.picture_quads += 1;
        }
        Ok(canvas)
    }
    // Source: components/viz/service/display/software_renderer.cc:433-438.
    fn DrawSolidColorQuad(
        canvas: &mut Canvas,
        quad: &NativeDrawQuad,
        shared: &NativeSharedQuadState,
        blend: skia::BlendMode,
        aa: bool,
    ) {
        if let QuadMaterial::SolidColor { color: c, .. } = &quad.material {
            canvas.draw_native_rect(
                rect(quad.geometry.visible_rect),
                color(*c, shared.opacity),
                blend,
                aa,
            );
        }
    }
    // SOURCE_EXCLUDED: components/viz/service/display/software_renderer.cc:440-490. TextureDrawQuad is absent
    // from the immutable aggregate QuadMaterial contract (CPU tiles carry leases).
    // Source: components/viz/service/display/software_renderer.cc:494-513.
    fn DrawTileQuad(
        canvas: &mut Canvas,
        input: &VizOutput,
        quad: &NativeDrawQuad,
        shared: &NativeSharedQuadState,
        blend: skia::BlendMode,
        antialias: bool,
    ) -> io::Result<()> {
        if let QuadMaterial::Tile {
            resource_id,
            tex_coord_rect,
            nearest_neighbor,
            ..
        } = &quad.material
        {
            let lock = resource(input, *resource_id)?;
            let visible = compositor_core::cc::base::math_util::MathUtil::ScaleRectProportional(
                *tex_coord_rect,
                RectF::from(quad.geometry.rect),
                RectF::from(quad.geometry.visible_rect),
            );
            canvas.draw_premultiplied_image_strict(
                image_view(lock.as_ref())?,
                rectf(visible),
                rect(quad.geometry.visible_rect),
                if *nearest_neighbor {
                    FilterQuality::Nearest
                } else {
                    FilterQuality::Bilinear
                },
                (shared.opacity * 255.) as u8 as f32 / 255.,
                blend,
                antialias,
            );
        }
        Ok(())
    }
    // SOURCE_EXCLUDED: components/viz/service/display/software_renderer.cc:515-535. VizDebugger capture only.
    // Source: components/viz/service/display/software_renderer.cc:537-623. The shader/mask objects are adapted
    // as CPU layer devices, retaining filter -> mask -> backdrop -> content order.
    fn DrawRenderPassQuad(
        &mut self,
        canvas: &mut Canvas,
        input: &VizOutput,
        quad: &NativeDrawQuad,
        shared: &NativeSharedQuadState,
        blend: skia::BlendMode,
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
        let source = self.render_pass_bitmaps.get(render_pass_id);
        let child_pass = input
            .Frame()
            .RenderPasses()
            .iter()
            .find(|p| p.RenderPassId() == *render_pass_id)
            .ok_or_else(|| io::Error::other("referenced aggregate pass absent"))?;
        let local = SkTransform::from_row(
            filters_scale.x(),
            0.,
            0.,
            filters_scale.y(),
            filters_origin.x() * filters_scale.x(),
            filters_origin.y() * filters_scale.y(),
        );
        let filter_image = if child_pass.Filters().IsEmpty() {
            None
        } else {
            Self::ApplyImageFilter(child_pass.Filters(), source, local, quad.geometry.rect)?
        };
        if !child_pass.Filters().IsEmpty() && filter_image.is_none() {
            return Ok(());
        }
        let (image, dest) = if let Some((image, dest)) = &filter_image {
            (image.pixmap(), *dest)
        } else if let Some(source) = source {
            (source.pixmap(), quad.geometry.rect)
        } else {
            return Ok(());
        };
        let visible = if filter_image.is_some() {
            dest
        } else {
            quad.geometry.visible_rect
        };
        let content_size = if filter_image.is_some() {
            dest.size()
        } else if quad.geometry.rect.IsEmpty() {
            Size::new(image.width() as i32, image.height() as i32)
        } else {
            quad.geometry.rect.size()
        };
        let paint_opacity = (shared.opacity * 255.) as u8 as f32 / 255.;
        let sampling = if self.linear_sampling {
            FilterQuality::Bilinear
        } else {
            FilterQuality::Nearest
        };
        let mask = if mask_resource_id.0 != 0 {
            Some(resource(input, *mask_resource_id)?)
        } else {
            None
        };
        let mask_rect = PaintRect {
            x: mask_uv_rect.x() as f64 * mask_texture_size.width() as f64,
            y: mask_uv_rect.y() as f64 * mask_texture_size.height() as f64,
            width: mask_uv_rect.width() as f64 * mask_texture_size.width() as f64,
            height: mask_uv_rect.height() as f64 * mask_texture_size.height() as f64,
        };
        // Source: components/viz/service/display/software_renderer.cc:611-619: backdrop uses the same mask as content, but its
        // shader already includes opacity. Each paint receives its own mask.
        if Self::ShouldApplyBackdropFilters(child_pass.BackdropFilters(), shared.opacity) {
            debug_assert!(
                child_pass.Filters().IsEmpty(),
                "Filters should always be in a separate Effect node"
            );
            simple(canvas, CommandKind::kSave);
            clip_rect(canvas, visible);
            let backdrop_mask = mask
                .as_deref()
                .map(|mask| image_view(mask))
                .transpose()?
                .map(|image| (image, mask_rect, dest, sampling));
            Self::GetBackdropFilterShader(
                canvas,
                child_pass.BackdropFilters(),
                child_pass.BackdropFilterBounds(),
                local,
                quad.geometry.visible_rect,
                shared.opacity,
                self.current_rect.size(),
                backdrop_mask,
            )?;
            simple(canvas, CommandKind::kRestore);
        }
        simple(canvas, CommandKind::kSave);
        clip_rect(canvas, visible);
        if mask.is_some() {
            canvas.save_layer_with_native_filters(
                &DrawCommand {
                    opacity: paint_opacity,
                    ..Default::default()
                },
                Vec::new(),
                blend,
            );
        }
        canvas.draw_premultiplied_image(
            image,
            PaintRect {
                x: 0.,
                y: 0.,
                width: content_size.width() as f64,
                height: content_size.height() as f64,
            },
            rect(dest),
            sampling,
            if mask.is_some() { 1. } else { paint_opacity },
            if mask.is_some() {
                skia::BlendMode::kSrcOver
            } else {
                blend
            },
            antialias,
        );
        if let Some(mask) = mask.as_deref() {
            canvas.draw_premultiplied_image(
                image_view(mask)?,
                mask_rect,
                rect(dest),
                sampling,
                1.,
                skia::BlendMode::kDstIn,
                false,
            );
            simple(canvas, CommandKind::kRestore);
        }
        simple(canvas, CommandKind::kRestore);
        Ok(())
    }
    // Source: components/viz/service/display/software_renderer.cc:772-777. Host owns backbuffer visibility.
    fn DidChangeVisibility(&mut self, _visible: bool) {}
    // SOURCE_EXCLUDED: components/viz/service/display/software_renderer.cc:625-633. VideoHole material cannot
    // enter this CPU aggregate contract; unresolved SharedElement is an error.
    // SOURCE_EXCLUDED: components/viz/service/display/software_renderer.cc:635-770. CopyOutputRequest and shared
    // image destinations are not exposed by immutable VizOutput.
    // Source: components/viz/service/display/software_renderer.cc:779-788.
    fn ShouldApplyBackdropFilters(
        filters: &paint::cc::paint::filter_operations::FilterOperations,
        opacity: f32,
    ) -> bool {
        !filters.IsEmpty() && opacity != 0.
    }
    // Source: components/viz/service/display/software_renderer.cc:799-840. Image filters render into their
    // expanded forward bounds before opacity, masks and the final blend.
    fn ApplyImageFilter(
        filters: &paint::cc::paint::filter_operations::FilterOperations,
        source: Option<&PassBacking>,
        local: SkTransform,
        quad: Rect,
    ) -> io::Result<Option<(PassBacking, Rect)>> {
        let matrix = skia::include::core::SkRRect::SkMatrix3x3 {
            values: [
                local.sx, local.kx, local.tx, local.ky, local.sy, local.ty, 0., 0., 1.,
            ],
        };
        let bound = filters.MapRect(
            foundation::gfx::Rect::new(
                foundation::gfx::Point::new(quad.x, quad.y),
                foundation::gfx::Size::new(quad.width, quad.height),
            ),
            Some(&matrix),
        );
        let bounds = Rect::new(bound.x(), bound.y(), bound.width(), bound.height());
        if bounds.IsEmpty() {
            return Ok(None);
        }
        let mut bitmap = PassBacking::new(Rect::new(0, 0, bounds.width, bounds.height))?;
        let mut canvas = Canvas::make_raster_direct_with_format_preserving(
            &ResourceContext::default(),
            bounds.width as u32,
            bounds.height as u32,
            bounds.width as usize * 4,
            PixelStorage::owned(std::mem::take(&mut bitmap.pixels)),
            skia::PixelFormat::Bgra8888,
            false,
        )
        .ok_or_else(|| io::Error::other("filter raster surface allocation failed"))?;
        let offset = SkTransform::from_translate(-bounds.x as f32, -bounds.y as f32);
        let effects = raster::native_filters::BuildFilters(filters, offset.pre_concat(local))?;
        // The native layer rebases local-to-device for its expanded input
        // allocation. Install the source canvas matrix before opening it.
        canvas.set_matrix(offset);
        canvas.save_layer_with_native_filters(
            &DrawCommand::default(),
            effects,
            skia::BlendMode::kSrcOver,
        );
        if let Some(source) = source {
            canvas.draw_premultiplied_image(
                source.pixmap(),
                PaintRect {
                    x: 0.,
                    y: 0.,
                    width: source.rect.width as f64,
                    height: source.rect.height as f64,
                },
                rect(Rect::new(
                    quad.x,
                    quad.y,
                    source.rect.width,
                    source.rect.height,
                )),
                FilterQuality::Nearest,
                1.,
                skia::BlendMode::kSrcOver,
                false,
            );
        } else {
            canvas.draw_native_rect(
                rect(bounds),
                command::Color {
                    red: 0.,
                    green: 0.,
                    blue: 0.,
                    alpha: 1.,
                },
                skia::BlendMode::kSrcOver,
                false,
            );
        }
        simple(&mut canvas, CommandKind::kRestore);
        bitmap.pixels = canvas.finish_direct().into_vec();
        Ok(Some((bitmap, bounds)))
    }
    // Source: components/viz/service/display/software_renderer.cc:842-872.
    fn ApplyBackdropFilterWithExactOutputSize(
        filters: &paint::cc::paint::filter_operations::FilterOperations,
        bitmap: skia::raster::PixmapRef<'_>,
        local: SkTransform,
        output: Rect,
    ) -> io::Result<PassBacking> {
        use skia::raster::SpreadMode;
        let matrix = skia::include::core::SkRRect::SkMatrix3x3 {
            values: [
                local.sx, local.kx, local.tx, local.ky, local.sy, local.ty, 0., 0., 1.,
            ],
        };
        let input = filters.MapRectReverse(
            foundation::gfx::Rect::new(
                foundation::gfx::Point::new(output.x, output.y),
                foundation::gfx::Size::new(output.width, output.height),
            ),
            &matrix,
        );
        let reverse = Rect::new(input.x(), input.y(), input.width(), input.height());
        let needs_mirror = !output.Contains(reverse);
        // Source: components/viz/service/display/software_renderer.cc:848-855. Crop's available content rectangle is the mirror
        // period, even when the captured device bitmap has different dimensions.
        let crop = if needs_mirror {
            let mut crop = PassBacking::new(Rect::new(0, 0, output.width, output.height))?;
            let mut receiver = Canvas::make_raster_direct_with_format_preserving(
                &ResourceContext::default(),
                output.width as u32,
                output.height as u32,
                output.width as usize * 4,
                PixelStorage::owned(std::mem::take(&mut crop.pixels)),
                skia::PixelFormat::Bgra8888,
                false,
            )
            .ok_or_else(|| io::Error::other("backdrop crop allocation failed"))?;
            receiver.draw_premultiplied_image(
                bitmap,
                PaintRect {
                    x: 0.,
                    y: 0.,
                    width: bitmap.width() as f64,
                    height: bitmap.height() as f64,
                },
                PaintRect {
                    x: 0.,
                    y: 0.,
                    width: bitmap.width() as f64,
                    height: bitmap.height() as f64,
                },
                FilterQuality::Nearest,
                1.,
                skia::BlendMode::kSrc,
                false,
            );
            crop.pixels = receiver.finish_direct().into_vec();
            Some(crop)
        } else {
            None
        };
        // Source: components/viz/service/display/software_renderer.cc:857-862. Exact output surface and failure path.
        let mut result = PassBacking::new(Rect::new(0, 0, output.width, output.height))?;
        let mut receiver = Canvas::make_raster_direct_with_format_preserving(
            &ResourceContext::default(),
            output.width as u32,
            output.height as u32,
            output.width as usize * 4,
            PixelStorage::owned(std::mem::take(&mut result.pixels)),
            skia::PixelFormat::Bgra8888,
            false,
        )
        .ok_or_else(|| io::Error::other("backdrop output allocation failed"))?;
        // Source: components/viz/service/display/software_renderer.cc:864-870. Local filter, translate, native-size bitmap paint.
        let offset = SkTransform::from_translate(-output.x as f32, -output.y as f32);
        let effects = raster::native_filters::BuildFilters(filters, offset.pre_concat(local))?;
        // Preserve the filter layer's device-origin adjustment after it opens.
        receiver.set_matrix(offset);
        receiver.save_layer_with_native_filters(
            &DrawCommand::default(),
            effects,
            skia::BlendMode::kSrcOver,
        );
        if let Some(crop) = &crop {
            receiver.draw_premultiplied_image_shader(
                crop.pixmap(),
                SkTransform::from_translate(output.x as f32, output.y as f32),
                rect(reverse),
                SpreadMode::Reflect,
                FilterQuality::Nearest,
                1.,
                skia::BlendMode::kSrcOver,
                false,
            );
        } else {
            receiver.draw_premultiplied_image(
                bitmap,
                PaintRect {
                    x: 0.,
                    y: 0.,
                    width: bitmap.width() as f64,
                    height: bitmap.height() as f64,
                },
                PaintRect {
                    x: output.x as f64,
                    y: output.y as f64,
                    width: bitmap.width() as f64,
                    height: bitmap.height() as f64,
                },
                FilterQuality::Nearest,
                1.,
                skia::BlendMode::kSrcOver,
                false,
            );
        }
        simple(&mut receiver, CommandKind::kRestore);
        result.pixels = receiver.finish_direct().into_vec();
        Ok(result)
    }
    // Source: components/viz/service/display/software_renderer.cc:874-886.
    fn GetBackdropBitmap(
        canvas: &Canvas,
        bounds: skia::raster::IntRect,
    ) -> Option<skia::raster::Pixmap> {
        canvas.read_pixels(bounds)
    }
    // Source: components/viz/service/display/software_renderer.cc:888-1028.
    fn GetBackdropFilterShader(
        canvas: &mut Canvas,
        filters: &paint::cc::paint::filter_operations::FilterOperations,
        bounds: Option<&skia::include::core::SkPath::SkPath>,
        filter_matrix: SkTransform,
        visible: Rect,
        opacity: f32,
        viewport: Size,
        mask: Option<(skia::raster::PixmapRef<'_>, PaintRect, Rect, FilterQuality)>,
    ) -> io::Result<()> {
        use raster::convert::ToSkia;
        use skia::raster::{PixmapRef, SpreadMode};
        // Source: components/viz/service/display/software_renderer.cc:891-928. Filter guard and local bounds.
        if filters.IsEmpty() || opacity == 0. {
            return Ok(());
        }
        let content = if let Some(path) = bounds {
            let r = path.getBounds();
            skia::raster::Rect::from_ltrb(r.left, r.top, r.right, r.bottom)
                .and_then(|r| r.transform(filter_matrix))
        } else {
            skia::raster::Rect::from_xywh(
                visible.x as f32,
                visible.y as f32,
                visible.width as f32,
                visible.height as f32,
            )
        };
        let Some(content) = content else {
            return Ok(());
        };
        let Some(content) = skia::raster::Rect::from_ltrb(
            content.left().floor(),
            content.top().floor(),
            content.right().ceil(),
            content.bottom().ceil(),
        ) else {
            return Ok(());
        };
        // Source: components/viz/service/display/software_renderer.cc:930-949. Device intersection and inverse.
        let matrix = canvas.local_to_device();
        let Some(inverse) = matrix.invert() else {
            return Ok(());
        };
        let Some(device) = content.transform(matrix).and_then(|r| {
            skia::raster::IntRect::from_ltrb(
                r.left().floor() as i32,
                r.top().floor() as i32,
                r.right().ceil() as i32,
                r.bottom().ceil() as i32,
            )
        }) else {
            return Ok(());
        };
        let Some(device) = skia::raster::IntRect::from_ltrb(
            device.left().max(0),
            device.top().max(0),
            device.right().min(viewport.width()),
            device.bottom().min(viewport.height()),
        ) else {
            return Ok(());
        };
        let Some(content) = skia::raster::Rect::from_ltrb(
            device.left() as f32,
            device.top() as f32,
            device.right() as f32,
            device.bottom() as f32,
        )
        .and_then(|r| r.transform(inverse)) else {
            return Ok(());
        };
        let left = content.left().floor();
        let top = content.top().floor();
        let width = (content.right().ceil() - left) as u32;
        let height = (content.bottom().ceil() - top) as u32;
        if width == 0 || height == 0 {
            return Ok(());
        }
        let len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or_else(|| io::Error::other("native backdrop bitmap allocation overflow"))?;
        // Source: components/viz/service/display/software_renderer.cc:949.
        let Some(bitmap) = Self::GetBackdropBitmap(canvas, device) else {
            return Ok(());
        };
        // Source: components/viz/service/display/software_renderer.cc:951-980.
        let filtered = Self::ApplyBackdropFilterWithExactOutputSize(
            filters,
            bitmap.as_ref(),
            filter_matrix,
            Rect::new(left as i32, top as i32, width as i32, height as i32),
        )?;
        let image = filtered.pixmap();
        // Source: components/viz/service/display/software_renderer.cc:982-995. The filtered image is painted
        // into a separate bitmap; its path clip uses the translated device matrix.
        let mut clipped = Canvas::make_raster_direct_with_format_preserving(
            &ResourceContext::default(),
            width,
            height,
            width as usize * 4,
            PixelStorage::owned(vec![0; len]),
            skia::PixelFormat::Rgba8888,
            false,
        )
        .ok_or_else(|| io::Error::other("backdrop clip surface allocation failed"))?;
        let image_tf = SkTransform::from_translate(-(device.left() as f32), -(device.top() as f32))
            .pre_concat(matrix);
        // Source: components/viz/service/display/software_renderer.cc:997-1003.
        if let Some(path) = bounds {
            clipped.set_matrix(image_tf.pre_concat(filter_matrix));
            let commands = paint::paint_clip_path::replay_path(path).map_err(|_| {
                io::Error::other("backdrop path cannot be represented by native receiver")
            })?;
            clipped.replay_clip_path(
                &DrawCommand {
                    r#type: CommandKind::kClipPath,
                    path: commands.to_skia(),
                    even_odd: paint::paint_clip_path::is_even_odd(path),
                    inverse_winding: path.isInverseFillType(),
                    antialias: true,
                    ..Default::default()
                },
                false,
            );
            clipped.set_matrix(SkTransform::identity());
        }
        // Source: components/viz/service/display/software_renderer.cc:1005-1017. Opacity is applied to the
        // prefiltered image before its final shader samples the clipped bitmap.
        let image_rect = PaintRect {
            x: 0.,
            y: 0.,
            width: width as f64,
            height: height as f64,
        };
        clipped.draw_premultiplied_image_strict(
            image,
            image_rect,
            image_rect,
            FilterQuality::Nearest,
            opacity,
            skia::BlendMode::kSrcOver,
            false,
        );
        let pixels = clipped.finish_direct().into_vec();
        let clipped_image =
            PixmapRef::from_bytes(&pixels, width, height).expect("backdrop clipped N32 bitmap");
        // Source: components/viz/service/display/software_renderer.cc:1019-1027. A decal image shader inverts the
        // translated content/device matrix to sample the result in content space.
        let Some(image_tf_inv) = image_tf.invert() else {
            return Ok(());
        };
        simple(canvas, CommandKind::kSave);
        canvas.set_matrix(matrix.pre_concat(image_tf_inv));
        clip_rect(canvas, Rect::new(0, 0, width as i32, height as i32));
        canvas.set_matrix(matrix);
        // Source: components/viz/service/display/software_renderer.cc:611-619 applies the paint mask after constructing the
        // backdrop shader. Capture the original device before opening this
        // local mask receiver, which starts with transparent pixels.
        if mask.is_some() {
            canvas.save_layer_with_native_filters(
                &DrawCommand::default(),
                Vec::new(),
                skia::BlendMode::kSrcOver,
            );
        }
        canvas.draw_premultiplied_image_shader(
            clipped_image,
            image_tf_inv,
            rect(visible),
            SpreadMode::Pad,
            FilterQuality::Nearest,
            1.,
            skia::BlendMode::kSrcOver,
            false,
        );
        if let Some((image, source, dest, sampling)) = mask {
            canvas.draw_premultiplied_image(
                image,
                source,
                rect(dest),
                sampling,
                1.,
                skia::BlendMode::kDstIn,
                false,
            );
            simple(canvas, CommandKind::kRestore);
        }
        simple(canvas, CommandKind::kRestore);

        Ok(())
    }
    // Source: components/viz/service/display/software_renderer.cc:1030-1057.
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
    // Source: components/viz/service/display/software_renderer.cc:1059-1085.
    fn AllocateRenderPassResourceIfNeeded(
        &mut self,
        id: RenderPassId,
        requirements: &RenderPassRequirements,
    ) -> io::Result<()> {
        if let Some(backing) = self.render_pass_bitmaps.get(&id) {
            debug_assert!(
                backing.rect.width >= requirements.size.width()
                    && backing.rect.height >= requirements.size.height()
            );
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
    // Source: components/viz/service/display/software_renderer.cc:1087-1097.
    fn IsRenderPassResourceAllocated(&self, id: RenderPassId) -> bool {
        self.render_pass_bitmaps.contains_key(&id)
    }
    fn GetRenderPassBackingPixelSize(&self, id: RenderPassId) -> Size {
        self.render_pass_bitmaps
            .get(&id)
            .map_or(Size::default(), |b| b.rect.size())
    }
    // Source: components/viz/service/display/software_renderer.cc:1099-1117.
    fn SetRenderPassBackingDrawnRect(&mut self, id: RenderPassId, rect: Rect) {
        if let Some(backing) = self.render_pass_bitmaps.get_mut(&id) {
            backing.drawn_rect = rect;
        }
    }
    fn GetRenderPassBackingDrawnRect(&self, id: RenderPassId) -> Rect {
        self.render_pass_bitmaps
            .get(&id)
            .map_or(Rect::default(), |b| b.drawn_rect)
    }
    // Local adapter: Chromium's SkCanvas borrows the bitmap; the Rust receiver
    // owns the allocation until finish_direct, then returns it without a copy.
    fn FinishDrawingRenderPass(&mut self) -> io::Result<()> {
        if let Some(canvas) = self.current_canvas.take() {
            let id = self.current_pass.take().expect("canvas belongs to a pass");
            let backing = self
                .render_pass_bitmaps
                .get_mut(&id)
                .expect("resident bitmap");
            backing.pixels = canvas.finish_direct().into_vec();
            backing.drawn_rect = self.current_rect;
        }
        Ok(())
    }
    // Local output-device adapter: newly allocated root pixels require repaint.
    fn GetCurrentFramebufferDamage(&self, id: RenderPassId) -> Rect {
        self.framebuffer_damage
            .get(&id)
            .copied()
            .unwrap_or_default()
    }
    // Local frame failure adapter: discard partial and moved allocations so a
    // failed frame cannot become retained content in a later successful draw.
    fn AbortDrawingFrame(&mut self) {
        self.current_canvas = None;
        self.current_pass = None;
        self.render_pass_bitmaps.clear();
        self.framebuffer_damage.clear();
        self.drawn_damage.clear();
    }
}

// Local trait forwarding: source implementation above remains in .cc order.
impl RendererBackend for SoftwareRenderer {
    type Target<'a> = RenderTarget<'a>;
    fn Configure(&mut self, settings: &RendererSettings) {
        self.settings = settings.clone();
    }
    fn ValidateTarget(&self, input: &VizOutput, target: &RenderTarget<'_>) -> io::Result<Size> {
        validate_target(input, target)?;
        Ok(Size::new(target.width as i32, target.height as i32))
    }
    fn PublishFrame(
        &mut self,
        input: &VizOutput,
        target: RenderTarget<'_>,
    ) -> io::Result<RenderUpdate> {
        publish(input, target, &self.render_pass_bitmaps, &self.drawn_damage)?;
        Ok(self.update)
    }
    fn CanPartialSwap(&self) -> bool {
        SoftwareRenderer::CanPartialSwap(self)
    }
    fn BeginDrawingFrame(&mut self, input: &VizOutput) -> io::Result<()> {
        SoftwareRenderer::BeginDrawingFrame(self, input)
    }
    fn FinishDrawingFrame(&mut self) -> io::Result<()> {
        SoftwareRenderer::FinishDrawingFrame(self)
    }
    fn EnsureScissorTestDisabled(&mut self) {
        SoftwareRenderer::EnsureScissorTestDisabled(self)
    }
    fn SetScissorTestRect(&mut self, rect: Rect) {
        SoftwareRenderer::SetScissorTestRect(self, rect)
    }
    fn BeginDrawingRenderPass(
        &mut self,
        pass: &AggregatedRenderPass,
        needs_clear: bool,
        update: Rect,
        viewport: Size,
    ) -> io::Result<()> {
        SoftwareRenderer::BeginDrawingRenderPass(self, pass, needs_clear, update, viewport)
    }
    fn DoDrawQuad(
        &mut self,
        input: &VizOutput,
        pass: &AggregatedRenderPass,
        index: usize,
        draw_region: Option<&[PointF; 4]>,
    ) -> io::Result<()> {
        SoftwareRenderer::DoDrawQuad(self, input, pass, index, draw_region)
    }
    fn FinishDrawingRenderPass(&mut self) -> io::Result<()> {
        SoftwareRenderer::FinishDrawingRenderPass(self)
    }
    fn GetCurrentFramebufferDamage(&self, id: RenderPassId) -> Rect {
        SoftwareRenderer::GetCurrentFramebufferDamage(self, id)
    }
    fn DidChangeVisibility(&mut self, _visible: bool) {
        SoftwareRenderer::DidChangeVisibility(self, _visible)
    }
    fn UpdateRenderPassTextures(
        &mut self,
        _input: &VizOutput,
        requirements: &BTreeMap<RenderPassId, RenderPassRequirements>,
    ) -> io::Result<()> {
        SoftwareRenderer::UpdateRenderPassTextures(self, _input, requirements)
    }
    fn AllocateRenderPassResourceIfNeeded(
        &mut self,
        id: RenderPassId,
        requirements: &RenderPassRequirements,
    ) -> io::Result<()> {
        SoftwareRenderer::AllocateRenderPassResourceIfNeeded(self, id, requirements)
    }
    fn IsRenderPassResourceAllocated(&self, id: RenderPassId) -> bool {
        SoftwareRenderer::IsRenderPassResourceAllocated(self, id)
    }
    fn GetRenderPassBackingPixelSize(&self, id: RenderPassId) -> Size {
        SoftwareRenderer::GetRenderPassBackingPixelSize(self, id)
    }
    fn SetRenderPassBackingDrawnRect(&mut self, id: RenderPassId, rect: Rect) {
        SoftwareRenderer::SetRenderPassBackingDrawnRect(self, id, rect)
    }
    fn GetRenderPassBackingDrawnRect(&self, id: RenderPassId) -> Rect {
        SoftwareRenderer::GetRenderPassBackingDrawnRect(self, id)
    }
    fn AbortDrawingFrame(&mut self) {
        SoftwareRenderer::AbortDrawingFrame(self)
    }
}
