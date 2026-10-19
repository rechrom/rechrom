//! Native Viz pass execution through the existing Skia CPU Canvas.
//! Frame resources and completed child passes remain actual borrowed owners;
//! no layer graph, scene snapshot or private blend/clip/sampling engine lives here.
use compositor_core::{
    adapter::{
        geometry::{Rect, RectF},
        layer_tree_impl::{Color4f, ResourceId},
        property_types::BlendMode,
    },
    runtime::{cpu_raster::CpuRasterResourceLease, frame::QuadMaterial},
};
use skia::{
    compat::commands::{self as command, CommandKind, DrawCommand, PaintRect, ResourceContext},
    cpu::canvas::Canvas,
    raster::{
        FilterQuality, Paint, Pattern, PixelStorage, PixmapMut, PixmapRef, SpreadMode,
        Transform as SkTransform,
    },
};
use std::{collections::BTreeMap, io};
use viz::native::{AggregatedFrame, AggregatedRenderPass, SurfaceId};
type PassId = compositor_core::runtime::frame::RenderPassId;
/// The actual immutable display-list owner carried by the producer's frame.
pub type PictureDisplayItemList=<compositor_core::NativeCompositorTypes as compositor_core::runtime::frame::FrameValueTypes>::PictureDisplayItemList;
pub struct PictureRasterRequest<'a> {
    pub display_list: &'a PictureDisplayItemList,
    pub content_rect: Rect,
    pub contents_scale: f32,
    pub nearest_neighbor: bool,
    pub image_animation_map: &'a std::collections::BTreeMap<
        compositor_core::adapter::image_animation::PaintImageId,
        usize,
    >,
    pub raster_inducing_scroll_offsets:
        &'a compositor_core::adapter::prioritized_tile::ScrollOffsetMap,
}
/// External CPU Paint playback only. Implementations execute the source
/// SoftwareRenderer::DrawPictureQuad (cc list Raster) translate/clip/scale and
/// image/scroll inputs into an actual content-sized immutable CPU allocation.
/// This capability cannot access a compositor tree, tile or scheduler owner.
pub trait PictureRasterizer: Send {
    fn Rasterize(
        &mut self,
        request: PictureRasterRequest<'_>,
    ) -> io::Result<
        std::sync::Arc<
            dyn compositor_core::runtime::cpu_raster::CpuRasterResourceLease<
                PixelFormat = skia::PixelFormat,
            >,
        >,
    >;
}

#[cfg(feature = "pure_replay")]
impl PictureRasterizer for raster::native::CpuRasterProvider {
    fn Rasterize(
        &mut self,
        request: PictureRasterRequest<'_>,
    ) -> io::Result<
        std::sync::Arc<
            dyn compositor_core::runtime::cpu_raster::CpuRasterResourceLease<
                PixelFormat = skia::PixelFormat,
            >,
        >,
    > {
        self.RasterizePicture(
            request.display_list,
            request.content_rect,
            request.contents_scale,
            request.nearest_neighbor,
            request.image_animation_map,
            request.raster_inducing_scroll_offsets,
        )
    }
}

/// Borrowed host output; no reference escapes Render.
pub struct RenderTarget<'a> {
    pub width: u32,
    pub height: u32,
    pub row_stride: usize,
    pub format: skia::PixelFormat,
    pub pixels: &'a mut [u32],
    /// The target still contains the pixels from this Renderer instance's
    /// previous completed aggregate at the same size and format.
    pub retained: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderUpdate {
    pub passes: usize,
    pub quads: usize,
    pub damage_pixels: usize,
    pub tile_quads: usize,
    pub picture_quads: usize,
    pub solid_quads: usize,
    pub tile_ns: u64,
    pub picture_ns: u64,
    pub solid_ns: u64,
}
struct PassBacking {
    rect: Rect,
    pixels: Vec<u8>,
}
impl PassBacking {
    fn new(rect: Rect) -> io::Result<Self> {
        let len = (rect.width as usize)
            .checked_mul(rect.height as usize)
            .and_then(|v| v.checked_mul(4))
            .filter(|v| *v <= isize::MAX as usize)
            .ok_or_else(|| io::Error::other("native pass allocation overflow"))?;
        Ok(Self {
            rect,
            pixels: vec![0; len],
        })
    }
    fn pixmap(&self) -> PixmapRef<'_> {
        PixmapRef::from_bytes_with_format(
            &self.pixels,
            self.rect.width as u32,
            self.rect.height as u32,
            skia::PixelFormat::Bgra8888,
        )
        .expect("actual N32 pass backing")
    }
}
#[derive(Default)]
pub struct Renderer {
    passes: BTreeMap<(SurfaceId, PassId), PassBacking>,
    picture_rasterizer: Option<Box<dyn PictureRasterizer>>,
}
impl Renderer {
    pub fn WithPictureRasterizer(rasterizer: Box<dyn PictureRasterizer>) -> Self {
        Self {
            passes: BTreeMap::new(),
            picture_rasterizer: Some(rasterizer),
        }
    }
    pub fn InvalidateSurface(&mut self, surface: SurfaceId) {
        self.passes.retain(|(id, _), _| *id != surface);
    }
    pub fn Render(
        &mut self,
        frame: &AggregatedFrame<'_>,
        target: RenderTarget<'_>,
    ) -> io::Result<RenderUpdate> {
        let profile = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        validate_target(frame, &target)?;
        let mut update = RenderUpdate::default();
        self.passes.retain(|(surface, id), _| {
            frame
                .RenderPasses()
                .iter()
                .any(|pass| pass.Surface() == *surface && pass.RenderPassId() == *id)
        });
        for pass in frame.RenderPasses() {
            let pass_rect = pass.OutputRect();
            if pass_rect.IsEmpty() {
                continue;
            }
            let key = (pass.Surface(), pass.RenderPassId());
            let previous = self.passes.remove(&key);
            let retained = previous
                .as_ref()
                .is_some_and(|backing| backing.rect == pass_rect)
                && pass.DamageIsContinuous();
            let mut backing = match previous {
                Some(backing) if backing.rect == pass_rect => backing,
                _ => PassBacking::new(pass_rect)?,
            };
            let mut damage = if retained {
                pass.DamageRect()
            } else {
                pass_rect
            };
            damage.Intersect(pass_rect);
            if damage.IsEmpty() {
                self.passes.insert(key, backing);
                continue;
            }
            let storage = std::mem::take(&mut backing.pixels);
            let mut canvas = Canvas::make_raster_direct_with_format_preserving(
                &ResourceContext::default(),
                pass_rect.width as u32,
                pass_rect.height as u32,
                pass_rect.width as usize * 4,
                PixelStorage::owned(storage),
                skia::PixelFormat::Bgra8888,
                false,
            )
            .ok_or_else(|| io::Error::other("Skia rejected native pass storage"))?;
            canvas.set_matrix(SkTransform::from_translate(
                -pass_rect.x as f32,
                -pass_rect.y as f32,
            ));
            clip_rect(&mut canvas, damage);
            canvas.draw_native_rect(
                rect(damage),
                command::Color {
                    red: 0.,
                    green: 0.,
                    blue: 0.,
                    alpha: 0.,
                },
                skia::BlendMode::kSrc,
                false,
            );
            draw_pass(
                &mut canvas,
                pass,
                frame.RenderPasses(),
                &self.passes,
                &mut self.picture_rasterizer,
                &mut update,
            )?;
            backing.pixels = canvas.finish_direct().into_vec();
            self.passes.insert(key, backing);
            update.passes += 1;
        }
        let passes_done = profile.map(|_| std::time::Instant::now());
        // Draw the already-completed root pixels into the actual borrowed host
        // storage. bytemuck validates the N32 byte view; there is no readback copy.
        let pixels = bytemuck::cast_slice_mut(target.pixels);
        let mut output = PixmapMut::from_bytes_with_format(
            pixels,
            target.row_stride as u32,
            target.height,
            target.format,
        )
        .ok_or_else(|| io::Error::other("invalid borrowed host N32 target"))?;
        let full_bounds = Rect::new(0, 0, target.width as i32, target.height as i32);
        if !target.retained {
            let mut background = Paint::default();
            background.set_color_rgba8(255, 255, 255, 255);
            background.blend_mode = skia::BlendMode::kSrc;
            background.anti_alias = false;
            let bounds =
                skia::raster::Rect::from_xywh(0., 0., target.width as f32, target.height as f32)
                    .expect("validated output bounds");
            output.fill_rect(bounds, &background, SkTransform::identity(), None);
        }
        for pass in frame.RenderPasses().iter().filter(|pass| pass.IsRoot()) {
            let backing = self
                .passes
                .get(&(pass.Surface(), pass.RenderPassId()))
                .ok_or_else(|| io::Error::other("completed root pass is unavailable"))?;
            let d = pass.Destination();
            let mut damage = if target.retained && pass.DamageIsContinuous() {
                let source = pass.OutputRect();
                let damage = pass.DamageRect();
                if source.width == d.width && source.height == d.height {
                    Rect::new(
                        d.x.saturating_add(damage.x.saturating_sub(source.x)),
                        d.y.saturating_add(damage.y.saturating_sub(source.y)),
                        damage.width,
                        damage.height,
                    )
                } else {
                    d
                }
            } else {
                d
            };
            damage.Intersect(full_bounds);
            if damage.IsEmpty() {
                continue;
            }
            if target.format == skia::PixelFormat::Bgra8888
                && d.width == backing.rect.width
                && d.height == backing.rect.height
                && d.x >= 0
                && d.y >= 0
            {
                // SoftwareRenderer's completed root device is already the
                // final surface. Skia's matching-info path is rect_memcpy;
                // replaying it through an image shader needlessly resamples
                // every output pixel and can blend retained transparency twice.
                let source_x = damage.x - d.x;
                let source_y = damage.y - d.y;
                let bytes = output.data_mut();
                let row_bytes = damage.width as usize * 4;
                for row in 0..damage.height as usize {
                    let src = ((source_y as usize + row) * backing.rect.width as usize
                        + source_x as usize)
                        * 4;
                    let dst =
                        ((damage.y as usize + row) * target.row_stride + damage.x as usize) * 4;
                    bytes[dst..dst + row_bytes]
                        .copy_from_slice(&backing.pixels[src..src + row_bytes]);
                }
                continue;
            }
            let sx = d.width as f32 / backing.rect.width as f32;
            let sy = d.height as f32 / backing.rect.height as f32;
            let paint = Paint {
                shader: Pattern::new(
                    backing.pixmap(),
                    SpreadMode::Pad,
                    FilterQuality::Bilinear,
                    1.,
                    SkTransform::from_row(sx, 0., 0., sy, d.x as f32, d.y as f32),
                ),
                blend_mode: skia::BlendMode::kSrcOver,
                anti_alias: false,
                ..Paint::default()
            };
            if let Some(bounds) = skia::raster::Rect::from_xywh(
                damage.x as f32,
                damage.y as f32,
                damage.width as f32,
                damage.height as f32,
            ) {
                output.fill_rect(bounds, &paint, SkTransform::identity(), None);
            }
        }
        if let (Some(started), Some(passes)) = (profile, passes_done) {
            eprintln!(
                "renderer-profile passes_ms={:.3} root_ms={:.3} total_ms={:.3} passes={} quads={} damage_pixels={} tiles={} tile_ms={:.3} pictures={} picture_ms={:.3} solids={} solid_ms={:.3}",
                (passes - started).as_secs_f64() * 1000.0,
                passes.elapsed().as_secs_f64() * 1000.0,
                started.elapsed().as_secs_f64() * 1000.0,
                update.passes,
                update.quads,
                update.damage_pixels,
                update.tile_quads,
                update.tile_ns as f64 / 1_000_000.0,
                update.picture_quads,
                update.picture_ns as f64 / 1_000_000.0,
                update.solid_quads,
                update.solid_ns as f64 / 1_000_000.0,
            );
        }
        Ok(update)
    }
}
fn simple(canvas: &mut Canvas, kind: CommandKind) {
    canvas.replay_item(
        &DrawCommand {
            r#type: kind,
            ..Default::default()
        },
        &ResourceContext::default(),
    );
}
fn rect(r: Rect) -> PaintRect {
    PaintRect {
        x: r.x as f64,
        y: r.y as f64,
        width: r.width as f64,
        height: r.height as f64,
    }
}
fn rectf(r: RectF) -> PaintRect {
    PaintRect {
        x: r.x() as f64,
        y: r.y() as f64,
        width: r.width() as f64,
        height: r.height() as f64,
    }
}
fn color(c: Color4f, opacity: f32) -> command::Color {
    command::Color {
        red: c.red,
        green: c.green,
        blue: c.blue,
        alpha: c.alpha * opacity,
    }
}
fn clip_rect(canvas: &mut Canvas, r: Rect) {
    canvas.replay_item(
        &DrawCommand {
            r#type: CommandKind::kClipRect,
            rect: rect(r),
            antialias: false,
            ..Default::default()
        },
        &ResourceContext::default(),
    );
}
fn native_transform(
    transform: &foundation::gfx::Transform,
    origin: Rect,
) -> io::Result<SkTransform> {
    // The current CPU Skia matrix backend is affine. Do not silently discard a
    // legal perspective matrix while that native primitive is unavailable.
    if transform.rc(3, 0) != 0. || transform.rc(3, 1) != 0. || transform.rc(3, 3) != 1. {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Skia CPU perspective matrix primitive is not exposed",
        ));
    }
    Ok(SkTransform::from_row(
        transform.rc(0, 0) as f32,
        transform.rc(1, 0) as f32,
        transform.rc(0, 1) as f32,
        transform.rc(1, 1) as f32,
        transform.rc(0, 3) as f32 - origin.x as f32,
        transform.rc(1, 3) as f32 - origin.y as f32,
    ))
}
fn resource<'a>(
    pass: &'a AggregatedRenderPass<'_>,
    id: ResourceId,
) -> io::Result<std::sync::Arc<dyn CpuRasterResourceLease<PixelFormat = skia::PixelFormat>>> {
    pass.Resources()
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| io::Error::other("native frame resource is absent"))?
        .CpuReadLease()
        .ok_or_else(|| io::Error::other("native frame resource has no CPU lease"))
}
fn image_view(
    lease: &dyn CpuRasterResourceLease<PixelFormat = skia::PixelFormat>,
) -> io::Result<PixmapRef<'_>> {
    let size = lease.size();
    if lease.row_bytes() != size.width() as usize * 4 {
        return Err(io::Error::other(
            "native CPU image row bytes are not tightly packed",
        ));
    }
    PixmapRef::from_bytes_with_format(
        lease.pixels(),
        size.width() as u32,
        size.height() as u32,
        lease.format(),
    )
    .ok_or_else(|| io::Error::other("invalid actual native image pixels"))
}
fn draw_pass(
    canvas: &mut Canvas,
    pass: &AggregatedRenderPass<'_>,
    passes: &[AggregatedRenderPass<'_>],
    completed: &BTreeMap<(SurfaceId, PassId), PassBacking>,
    rasterizer: &mut Option<Box<dyn PictureRasterizer>>,
    update: &mut RenderUpdate,
) -> io::Result<()> {
    let output = pass.OutputRect();
    // Source: SoftwareRenderer visits the source front-to-back quad list in reverse.
    for quad in pass.Quads().iter().rev() {
        let quad_started = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        let shared = pass
            .TrySharedQuadState(
                quad.geometry
                    .shared_quad_state
                    .ok_or_else(|| io::Error::other("native quad lacks SQS"))?,
            )
            .ok_or_else(|| io::Error::other("native SQS handle outside resident pass"))?;
        if quad.geometry.visible_rect.IsEmpty() || shared.opacity <= 0. {
            continue;
        }
        simple(canvas, CommandKind::kSave);
        canvas.set_matrix(SkTransform::from_translate(
            -output.x as f32,
            -output.y as f32,
        ));
        if let Some(clip) = shared.clip_rect {
            clip_rect(canvas, clip);
        }
        // SoftwareRenderer.cc:278-295 installs the target-space rounded clip before
        // the actual source content matrix. Its analytic receiver lives in Skia.
        if shared.mask_filter_info.HasRoundedCorners() {
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
        canvas.set_matrix(native_transform(
            shared.quad_to_target_transform.native(),
            output,
        )?);
        if !matches!(&quad.material, QuadMaterial::CompositorRenderPass { .. }) {
            clip_rect(canvas, quad.geometry.visible_rect);
        }
        let blend = skia_blend_mode(shared.blend_mode);
        let matrix = canvas.local_to_device();
        let antialias = matrix.kx != 0.
            || matrix.ky != 0.
            || matrix.tx.fract() != 0.
            || matrix.ty.fract() != 0.;
        match &quad.material {
            QuadMaterial::SolidColor {
                color: c,
                force_anti_aliasing_off,
            } => {
                canvas.draw_native_rect(
                    rect(quad.geometry.visible_rect),
                    color(*c, shared.opacity),
                    blend,
                    antialias && !*force_anti_aliasing_off,
                );
                update.solid_quads += 1;
                update.solid_ns += quad_started.map_or(0, |v| v.elapsed().as_nanos() as u64);
            }
            QuadMaterial::DebugBorder { color: c, width } => canvas.draw_native_debug_border(
                rect(quad.geometry.rect),
                color(*c, shared.opacity),
                *width as f32,
                blend,
                false,
            ),
            QuadMaterial::Tile {
                resource_id,
                tex_coord_rect,
                nearest_neighbor,
                ..
            } => {
                let lease = resource(pass, *resource_id)?;
                // Source: components/viz/service/display/software_renderer.cc:494-512.
                // TileDrawQuad's texture rectangle describes the whole quad.
                // When occlusion or the viewport exposes only `visible_rect`,
                // crop the source proportionally and draw that crop directly
                // into the visible destination.  Relying on a destination clip
                // leaves the image shader based on the offscreen whole-quad
                // origin, which is observably wrong for partially visible CPU
                // tiles after compositor scrolling.
                let visible_tex_coord_rect =
                    compositor_core::cc::base::math_util::MathUtil::ScaleRectProportional(
                        *tex_coord_rect,
                        RectF::from(quad.geometry.rect),
                        RectF::from(quad.geometry.visible_rect),
                    );
                let tile_blend = if shared.are_contents_opaque
                    && shared.opacity == 1.
                    && blend == skia::BlendMode::kSrcOver
                {
                    skia::BlendMode::kSrc
                } else {
                    blend
                };
                canvas.draw_premultiplied_image(
                    image_view(lease.as_ref())?,
                    rectf(visible_tex_coord_rect),
                    rect(quad.geometry.visible_rect),
                    if *nearest_neighbor {
                        FilterQuality::Nearest
                    } else {
                        FilterQuality::Bilinear
                    },
                    shared.opacity,
                    tile_blend,
                    false,
                );
                update.tile_quads += 1;
                update.tile_ns += quad_started.map_or(0, |v| v.elapsed().as_nanos() as u64);
            }
            QuadMaterial::Picture {
                content_rect,
                tex_coord_rect,
                contents_scale,
                nearest_neighbor,
                image_animation_map,
                display_item_list,
                raster_inducing_scroll_offsets,
            } => {
                let actual = rasterizer
                    .as_deref_mut()
                    .ok_or_else(|| io::Error::other("Picture quad requires injected CPU playback"))?
                    .Rasterize(PictureRasterRequest {
                        display_list: display_item_list.as_deref().ok_or_else(|| {
                            io::Error::other("native Picture quad lacks actual display list")
                        })?,
                        content_rect: *content_rect,
                        contents_scale: *contents_scale,
                        nearest_neighbor: *nearest_neighbor,
                        image_animation_map,
                        raster_inducing_scroll_offsets,
                    })?;
                if actual.size() != content_rect.size() {
                    return Err(io::Error::other(
                        "Picture playback size differs from native content rect",
                    ));
                }
                canvas.draw_premultiplied_image(
                    image_view(actual.as_ref())?,
                    rectf(*tex_coord_rect),
                    rect(quad.geometry.rect),
                    if *nearest_neighbor {
                        FilterQuality::Nearest
                    } else {
                        FilterQuality::Bilinear
                    },
                    shared.opacity,
                    blend,
                    false,
                );
                update.picture_quads += 1;
                update.picture_ns += quad_started.map_or(0, |v| v.elapsed().as_nanos() as u64);
            }
            QuadMaterial::CompositorRenderPass {
                render_pass_id,
                mask_resource_id,
                mask_uv_rect,
                mask_texture_size,
                filters_scale,
                filters_origin,
                ..
            } => {
                let child = completed
                    .get(&(pass.Surface(), *render_pass_id))
                    .ok_or_else(|| {
                        io::Error::other("native child render-pass backing is unavailable")
                    })?;
                let child_pass = passes
                    .iter()
                    .find(|p| p.Surface() == pass.Surface() && p.RenderPassId() == *render_pass_id)
                    .ok_or_else(|| {
                        io::Error::other("native child render-pass value is unavailable")
                    })?;
                let filter_matrix = SkTransform::from_row(
                    filters_scale.x(),
                    0.,
                    0.,
                    filters_scale.y(),
                    filters_origin.x() * filters_scale.x(),
                    filters_origin.y() * filters_scale.y(),
                );
                let filters = raster::native_filters::BuildFilters(
                    child_pass.Filters(),
                    canvas.local_to_device().pre_concat(filter_matrix),
                )?;
                let has_filters = !filters.is_empty();
                let filtered_rect = if has_filters {
                    use foundation::gfx::{Point, Size};
                    let r = quad.geometry.rect;
                    let m = skia::include::core::SkRRect::SkMatrix3x3 {
                        values: [
                            filter_matrix.sx,
                            filter_matrix.kx,
                            filter_matrix.tx,
                            filter_matrix.ky,
                            filter_matrix.sy,
                            filter_matrix.ty,
                            0.,
                            0.,
                            1.,
                        ],
                    };
                    let r = child_pass.Filters().MapRect(
                        foundation::gfx::Rect::new(
                            Point::new(r.x, r.y),
                            Size::new(r.width, r.height),
                        ),
                        Some(&m),
                    );
                    Rect::new(r.x(), r.y(), r.width(), r.height())
                } else {
                    quad.geometry.rect
                };
                clip_rect(
                    canvas,
                    if has_filters {
                        filtered_rect
                    } else {
                        quad.geometry.visible_rect
                    },
                );
                let mask = if mask_resource_id.0 != 0 {
                    Some(resource(pass, *mask_resource_id)?)
                } else {
                    None
                };
                let mask_uv = PaintRect {
                    x: mask_uv_rect.x() as f64 * mask_texture_size.width() as f64,
                    y: mask_uv_rect.y() as f64 * mask_texture_size.height() as f64,
                    width: mask_uv_rect.width() as f64 * mask_texture_size.width() as f64,
                    height: mask_uv_rect.height() as f64 * mask_texture_size.height() as f64,
                };
                if !child_pass.BackdropFilters().IsEmpty() {
                    if mask.is_some() {
                        canvas.save_layer_with_native_filters(
                            &DrawCommand::default(),
                            Vec::new(),
                            skia::BlendMode::kSrcOver,
                        );
                    }
                    draw_backdrop(
                        canvas,
                        child_pass.BackdropFilters(),
                        child_pass.BackdropFilterBounds(),
                        filter_matrix,
                        quad.geometry.visible_rect,
                        shared.opacity,
                    )?;
                    if let Some(mask) = mask.as_deref() {
                        canvas.draw_premultiplied_image(
                            image_view(mask)?,
                            mask_uv,
                            rect(filtered_rect),
                            FilterQuality::Bilinear,
                            1.,
                            skia::BlendMode::kDstIn,
                            false,
                        );
                        simple(canvas, CommandKind::kRestore);
                    }
                }
                // The source applies filtering before mask and opacity/blend. Separate
                // native devices preserve that order without copying the child bitmap.
                let group = has_filters || mask_resource_id.0 != 0;
                if group {
                    canvas.save_layer_with_native_filters(
                        &DrawCommand {
                            opacity: shared.opacity,
                            ..Default::default()
                        },
                        Vec::new(),
                        blend,
                    );
                }
                if has_filters {
                    canvas.save_layer_with_native_filters(
                        &DrawCommand::default(),
                        filters,
                        skia::BlendMode::kSrcOver,
                    );
                }
                canvas.draw_premultiplied_image(
                    child.pixmap(),
                    PaintRect {
                        x: 0.,
                        y: 0.,
                        width: child.rect.width as f64,
                        height: child.rect.height as f64,
                    },
                    rect(quad.geometry.rect),
                    FilterQuality::Bilinear,
                    if group { 1. } else { shared.opacity },
                    if group {
                        skia::BlendMode::kSrcOver
                    } else {
                        blend
                    },
                    false,
                );
                if has_filters {
                    simple(canvas, CommandKind::kRestore);
                }
                if let Some(mask) = mask.as_deref() {
                    canvas.draw_premultiplied_image(
                        image_view(mask)?,
                        mask_uv,
                        rect(filtered_rect),
                        FilterQuality::Bilinear,
                        1.,
                        skia::BlendMode::kDstIn,
                        false,
                    );
                }
                if group {
                    simple(canvas, CommandKind::kRestore);
                }
            }
            QuadMaterial::SharedElement { .. } => {
                return Err(io::Error::other(
                    "unresolved shared-element quad reached native Renderer",
                ))
            }
        }
        simple(canvas, CommandKind::kRestore);
        update.quads += 1;
        update.damage_pixels = update.damage_pixels.saturating_add(
            quad.geometry.visible_rect.width as usize * quad.geometry.visible_rect.height as usize,
        );
    }
    Ok(())
}
/// Source: SoftwareRenderer::GetBackdropFilterShader cc891-1017. The copied
/// value is the explicitly required backdrop readPixels region, not a child
/// pass clone or retained scene snapshot. Sampling/filtering remain in Skia.
fn draw_backdrop(
    canvas: &mut Canvas,
    filters: &paint::cc::paint::filter_operations::FilterOperations,
    bounds: Option<&skia::include::core::SkPath::SkPath>,
    filter_matrix: SkTransform,
    visible: Rect,
    opacity: f32,
) -> io::Result<()> {
    use raster::convert::ToSkia;
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
    let Some(readback) = canvas.read_pixels_intersection(device) else {
        return Ok(());
    };
    let (device, bitmap) = readback;
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
    let mut effect = Canvas::make_raster_direct_with_format_preserving(
        &ResourceContext::default(),
        width,
        height,
        width as usize * 4,
        PixelStorage::owned(vec![0; len]),
        skia::PixelFormat::Rgba8888,
        false,
    )
    .ok_or_else(|| io::Error::other("Skia rejected actual backdrop bitmap"))?;
    let effects = raster::native_filters::BuildFilters(
        filters,
        SkTransform::from_translate(-left, -top).pre_concat(filter_matrix),
    )?;
    effect.save_layer_with_native_filters(
        &DrawCommand::default(),
        effects,
        skia::BlendMode::kSrcOver,
    );
    effect.set_matrix(SkTransform::from_translate(-left, -top).pre_concat(inverse));
    // Source ApplyBackdropFilterWithExactOutputSize wraps insufficient input
    // in Crop(availableBackdrop,kMirror). Draw that real source bitmap through
    // the existing native mirror shader across the exact reverse input bound.
    let local_matrix = skia::include::core::SkRRect::SkMatrix3x3 {
        values: [
            filter_matrix.sx,
            filter_matrix.kx,
            filter_matrix.tx,
            filter_matrix.ky,
            filter_matrix.sy,
            filter_matrix.ty,
            0.,
            0.,
            1.,
        ],
    };
    let input = filters.MapRectReverse(
        foundation::gfx::Rect::new(
            foundation::gfx::Point::new(left as i32, top as i32),
            foundation::gfx::Size::new(width as i32, height as i32),
        ),
        &local_matrix,
    );
    let input = skia::raster::Rect::from_xywh(
        input.x() as f32,
        input.y() as f32,
        input.width() as f32,
        input.height() as f32,
    )
    .and_then(|r| r.transform(matrix));
    if let Some(input) = input {
        effect.draw_premultiplied_image_shader(
            bitmap.as_ref(),
            SkTransform::from_translate(device.left() as f32, device.top() as f32),
            PaintRect {
                x: input.left() as f64,
                y: input.top() as f64,
                width: input.width() as f64,
                height: input.height() as f64,
            },
            SpreadMode::Reflect,
            FilterQuality::Bilinear,
            1.,
            skia::BlendMode::kSrcOver,
            false,
        );
    }
    simple(&mut effect, CommandKind::kRestore);
    let pixels = effect.finish_direct().into_vec();
    let image =
        PixmapRef::from_bytes(&pixels, width, height).expect("native backdrop filter output");
    simple(canvas, CommandKind::kSave);
    if let Some(path) = bounds {
        canvas.set_matrix(matrix.pre_concat(filter_matrix));
        let commands = paint::paint_clip_path::replay_path(path).map_err(|_| {
            io::Error::other("actual backdrop SkPath cannot be represented by native receiver")
        })?;
        canvas.replay_clip_path(
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
        canvas.set_matrix(matrix);
    }
    canvas.draw_premultiplied_image(
        image,
        PaintRect {
            x: 0.,
            y: 0.,
            width: width as f64,
            height: height as f64,
        },
        PaintRect {
            x: left as f64,
            y: top as f64,
            width: width as f64,
            height: height as f64,
        },
        FilterQuality::Bilinear,
        opacity,
        skia::BlendMode::kSrcOver,
        false,
    );
    simple(canvas, CommandKind::kRestore);
    Ok(())
}

fn skia_blend_mode(mode: BlendMode) -> skia::BlendMode {
    raster::native_filters::NativeBlendMode(mode)
}
fn validate_target(frame: &AggregatedFrame<'_>, target: &RenderTarget<'_>) -> io::Result<()> {
    let output = frame.OutputRect();
    let length = target
        .row_stride
        .checked_mul(target.height as usize)
        .filter(|&length| {
            output == Rect::new(0, 0, target.width as i32, target.height as i32)
                && target.row_stride >= target.width as usize
                && target.pixels.len() == length
        })
        .ok_or_else(|| io::Error::other("aggregated frame does not match render target"))?;
    debug_assert_eq!(length, target.pixels.len());
    Ok(())
}
