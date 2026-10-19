//! Source paint record -> existing Skia CPU command playback. The canonical cc
//! DisplayItemList retains RTree selection and passes actual op offsets here.
use super::*;
use crate::convert::{resources, ToSkia};
use command::{CommandKind, DrawCommand, PaintRect, ResourceContext};
use compositor_core::{
    adapter::{
        display_item_list::DisplayCanvas, picture_layer_impl::PictureRasterSource,
        picture_layer_tiling::TilingRasterSource,
    },
    runtime::paint::{
        ContentDisplayItemList, CpuPaintCanvas, CpuPaintOp, CpuPaintOpBuffer, CpuPaintOpRef,
        CpuPaintRecord, CpuPlaybackParams,
    },
};
use skia::{compat::commands as command, cpu::canvas::Canvas, raster::PixelStorage};

pub(super) struct ReplayCanvas {
    canvas: Canvas,
    error: Option<io::Error>,
    depth: usize,
    nearest: bool,
    // skia/ext/opacity_filter_canvas.cc:17-22. Per-primitive paint alpha,
    // independent of SaveLayer restore opacity and playback folding alpha.
    opacity: f32,
}
impl ReplayCanvas {
    fn command(&mut self, command: &DrawCommand, resources: &ResourceContext) {
        match command.r#type {
            CommandKind::kSave
            | CommandKind::kSaveLayer
            | CommandKind::kSaveLayerAlpha
            | CommandKind::kSaveLayerBlend
            | CommandKind::kSaveLayerFilter
            | CommandKind::kSaveLayerDstIn => self.depth += 1,
            CommandKind::kRestore => {
                if self.depth == 0 {
                    self.error
                        .get_or_insert_with(|| io::Error::other("native Paint restore underflow"));
                    return;
                }
                self.depth -= 1;
            }
            _ => {}
        }
        // Source: skia/ext/opacity_filter_canvas.cc:17-47. Matrix, clips and
        // layer saves forward unchanged; only individual draw paints filter.
        let is_draw = !matches!(
            command.r#type,
            CommandKind::kSave
                | CommandKind::kRestore
                | CommandKind::kConcat
                | CommandKind::kClipRect
                | CommandKind::kClipRoundedRect
                | CommandKind::kClipOutRoundedRect
                | CommandKind::kClipPath
                | CommandKind::kClipOutRect
                | CommandKind::kSaveLayer
                | CommandKind::kSaveLayerAlpha
                | CommandKind::kSaveLayerBlend
                | CommandKind::kSaveLayerFilter
                | CommandKind::kSaveLayerDstIn
                | CommandKind::kBeginMask
                | CommandKind::kEndMask
        );
        if is_draw && (self.opacity < 1. || self.nearest) {
            let mut filtered = command.clone();
            if self.opacity < 1. {
                // Each receiver selects ONE alpha representation:
                // SkCanvas::drawImageRect -> item.flags_alpha;
                // solid/glyph/path/shadow -> item.color.alpha;
                // compat::paint_for_shader gradients -> stop alpha;
                // compat::with_paint_for_shader patterns -> shader.flags_alpha.
                // Shader and solid-color fallback branches are exclusive.
                if matches!(
                    filtered.r#type,
                    CommandKind::kDrawImageRect | CommandKind::kDrawTiledImage
                ) {
                    filtered.flags_alpha *= self.opacity;
                } else {
                    filtered.color.alpha *= self.opacity;
                    if let Some(shader) = &mut filtered.paint_shader {
                        match shader.kind {
                            command::PaintShaderKind::kPattern => {
                                shader.flags_alpha *= self.opacity
                            }
                            _ => {
                                for stop in &mut shader.stops {
                                    stop.color.alpha *= self.opacity;
                                }
                            }
                        }
                    }
                }
            }
            // draw_tiled_image forwards each tile into drawImageRect, so it
            // needs the same image-sampling override as a direct image draw.
            if self.nearest
                && matches!(
                    filtered.r#type,
                    CommandKind::kDrawImageRect | CommandKind::kDrawTiledImage
                )
            {
                filtered.image_interpolation_quality = command::ImageInterpolationQuality::kNone;
            }
            self.canvas.replay_item(&filtered, resources);
        } else {
            self.canvas.replay_item(command, resources);
        }
    }
    fn simple(&mut self, kind: CommandKind) {
        self.command(
            &DrawCommand {
                r#type: kind,
                ..Default::default()
            },
            &ResourceContext { resources: None },
        );
    }
    fn translate(&mut self, x: f32, y: f32) {
        let mut transform = command::TransformMatrix::default();
        transform.values[12] = x as f64;
        transform.values[13] = y as f64;
        self.command(
            &DrawCommand {
                r#type: CommandKind::kConcat,
                transform,
                ..Default::default()
            },
            &ResourceContext { resources: None },
        );
    }
    fn scale(&mut self, x: f32, y: f32) {
        let mut transform = command::TransformMatrix::default();
        transform.values[0] = x as f64;
        transform.values[5] = y as f64;
        self.command(
            &DrawCommand {
                r#type: CommandKind::kConcat,
                transform,
                ..Default::default()
            },
            &ResourceContext { resources: None },
        );
    }
    fn clip(&mut self, rect: Rect, difference: bool) {
        self.command(
            &DrawCommand {
                r#type: if difference {
                    CommandKind::kClipOutRect
                } else {
                    CommandKind::kClipRect
                },
                rect: rect_command(rect),
                antialias: false,
                ..Default::default()
            },
            &ResourceContext { resources: None },
        );
    }
    fn fail(&mut self, message: &str) {
        self.error
            .get_or_insert_with(|| io::Error::new(io::ErrorKind::Unsupported, message.to_owned()));
    }
    fn record(
        &mut self,
        record: &CpuPaintRecord<SkRegion>,
        params: &mut CpuPlaybackParams<'_, SkRegion>,
    ) {
        if let Some(buffer) = record.buffer() {
            self.Playback(buffer, params, true, None);
        }
    }
    fn converted(
        &mut self,
        op: &CpuPaintOp<SkRegion>,
        params: &mut CpuPlaybackParams<'_, SkRegion>,
        folded_alpha: f32,
    ) {
        match op {
            CpuPaintOp::NativeAtom(source) => {
                let command = source.op().to_skia();
                self.command(&command, &resources(source.record().artifact()));
            }
            CpuPaintOp::DrawRect(rect, flags) => {
                self.shader_rect(*rect, flags, folded_alpha, params)
            }
            CpuPaintOp::DrawPath { path, flags } => {
                let c = flags.getColor4f();
                self.canvas.draw_native_path(
                    path,
                    command::Color {
                        red: c.fR,
                        green: c.fG,
                        blue: c.fB,
                        alpha: c.fA * folded_alpha * self.opacity,
                    },
                    flags.getBlendMode(),
                    flags.isAntiAlias(),
                    (flags.getStyle() == paint::cc::paint::paint_flags::Style::kStroke_Style)
                        .then(|| flags.getStrokeWidth()),
                );
            }
            CpuPaintOp::DrawDRRect {
                outer,
                inner,
                flags,
            } => self.looped_shape(outer, Some(inner), flags, folded_alpha),
            CpuPaintOp::DrawRRect(rect, flags) if flags.looper.is_some() => {
                self.looped_shape(rect, None, flags, folded_alpha)
            }
            CpuPaintOp::DrawRRect(rect, flags) => {
                let radius = |i: usize| command::PaintCornerRadius {
                    x: rect.corner_radii[i].x() as f64,
                    y: rect.corner_radii[i].y() as f64,
                };
                let radii = command::PaintCornerRadii {
                    top_left: radius(0),
                    top_right: radius(1),
                    bottom_right: radius(2),
                    bottom_left: radius(3),
                };
                let filtered = flags.image_filter.is_some();
                if let Some(filter) = &flags.image_filter {
                    match crate::native_filters::BuildGraph(
                        filter.as_ref(),
                        self.canvas.local_to_device(),
                    ) {
                        Ok(filters) => {
                            self.canvas.save_layer_with_native_filters(
                                &DrawCommand {
                                    opacity: flags.getAlphaf() * folded_alpha * self.opacity,
                                    ..Default::default()
                                },
                                filters,
                                flags.getBlendMode(),
                            );
                            self.depth += 1;
                        }
                        Err(error) => {
                            self.error.get_or_insert(error);
                            return;
                        }
                    }
                }
                let color = flags.getColor4f();
                self.canvas.draw_native_rrect(
                    float_rect(rect.rect),
                    radii,
                    command::Color {
                        red: color.fR,
                        green: color.fG,
                        blue: color.fB,
                        alpha: if filtered {
                            1.
                        } else {
                            flags.getAlphaf() * folded_alpha * self.opacity
                        },
                    },
                    if filtered {
                        skia::BlendMode::kSrcOver
                    } else {
                        flags.getBlendMode()
                    },
                    flags.isAntiAlias(),
                );
                if filtered {
                    self.simple(CommandKind::kRestore);
                }
            }
            CpuPaintOp::Save => self.simple(CommandKind::kSave),
            CpuPaintOp::Restore => self.simple(CommandKind::kRestore),
            CpuPaintOp::Translate(x, y) => self.translate(*x, *y),
            CpuPaintOp::Concat(transform) => self.command(
                &DrawCommand {
                    r#type: CommandKind::kConcat,
                    transform: command::TransformMatrix {
                        values: transform.native().GetColMajor(),
                    },
                    ..Default::default()
                },
                &ResourceContext { resources: None },
            ),
            CpuPaintOp::ClipRect {
                rect,
                difference,
                antialias,
            } => self.command(
                &DrawCommand {
                    r#type: if *difference {
                        CommandKind::kClipOutRect
                    } else {
                        CommandKind::kClipRect
                    },
                    rect: float_rect(*rect),
                    antialias: *antialias,
                    ..Default::default()
                },
                &ResourceContext { resources: None },
            ),
            CpuPaintOp::ClipRRect(rect, antialias) => {
                let radius = |index: usize| command::PaintCornerRadius {
                    x: rect.corner_radii[index].x() as f64,
                    y: rect.corner_radii[index].y() as f64,
                };
                self.command(
                    &DrawCommand {
                        r#type: CommandKind::kClipRoundedRect,
                        rect: float_rect(rect.rect),
                        corner_radii: command::PaintCornerRadii {
                            top_left: radius(0),
                            top_right: radius(1),
                            bottom_right: radius(2),
                            bottom_left: radius(3),
                        },
                        antialias: *antialias,
                        ..Default::default()
                    },
                    &ResourceContext { resources: None },
                );
            }
            CpuPaintOp::ClipPath {
                path,
                difference,
                antialias,
            } => {
                let Some(path) = path.single_path() else {
                    self.fail(
                        "native clip-path union requires a replayable Skia path-union capability",
                    );
                    return;
                };
                match paint::paint_clip_path::replay_path(path){
     Ok(commands)=>self.canvas.replay_clip_path(&DrawCommand{r#type:CommandKind::kClipPath,path:commands.to_skia(),even_odd:paint::paint_clip_path::is_even_odd(path),inverse_winding:path.isInverseFillType(),antialias:*antialias,..Default::default()},*difference),
     Err(_)=>self.fail("native clip path cannot be represented by the current Skia command receiver"),
    }
            }
            CpuPaintOp::SaveLayer { bounds, flags } => {
                let filters = match &flags.image_filter {
                    Some(filter) => crate::native_filters::BuildGraph(
                        filter.as_ref(),
                        self.canvas.local_to_device(),
                    ),
                    None => Ok(Vec::new()),
                };
                match filters {
                    Ok(mut filters) => {
                        if flags.color_filter.is_some() {
                            filters.push(skia::cpu::layer_filters::RenderSurfaceFilter::Luma);
                        }
                        self.canvas.save_layer_with_native_filters(
                            &DrawCommand {
                                rect: bounds.map(float_rect).unwrap_or_default(),
                                opacity: flags.getAlphaf(),
                                ..Default::default()
                            },
                            filters,
                            flags.getBlendMode(),
                        );
                        self.depth += 1;
                    }
                    Err(error) => {
                        self.error.get_or_insert(error);
                    }
                }
            }
            CpuPaintOp::SaveLayerAlpha { bounds, alpha } => self.command(
                &DrawCommand {
                    r#type: CommandKind::kSaveLayerAlpha,
                    rect: bounds.map(float_rect).unwrap_or_default(),
                    opacity: *alpha,
                    ..Default::default()
                },
                &ResourceContext { resources: None },
            ),
            CpuPaintOp::SaveLayerFilters {
                bounds,
                filters,
                backdrop_filter,
                flags,
            } => {
                let matrix = self.canvas.local_to_device();
                let backdrop = crate::native_filters::BuildGraph(backdrop_filter.as_ref(), matrix);
                let branches = if let Some(filter) = &flags.image_filter {
                    crate::native_filters::BuildGraph(filter.as_ref(), matrix)
                        .map(|filter| vec![filter])
                } else {
                    filters
                        .iter()
                        .map(|filter| crate::native_filters::BuildGraph(filter.as_ref(), matrix))
                        .collect::<io::Result<Vec<_>>>()
                };
                match (backdrop, branches) {
                    (Ok(backdrop), Ok(mut branches)) => {
                        // SkCanvas.cpp:904-909: Paint.imageFilter overrides the
                        // independent fFilters span. Color filters are restore
                        // paint, so apply them to every independent result.
                        if flags.color_filter.is_some() {
                            if branches.is_empty() {
                                branches.push(Vec::new());
                            }
                            for branch in &mut branches {
                                branch.push(skia::cpu::layer_filters::RenderSurfaceFilter::Luma);
                            }
                        }
                        self.canvas.save_layer_with_native_filter_branches(
                            &DrawCommand {
                                opacity: flags.getAlphaf(),
                                ..Default::default()
                            },
                            branches,
                            backdrop,
                            bounds.map(float_rect),
                            flags.getBlendMode(),
                        );
                        self.depth += 1;
                    }
                    (Err(error), _) | (_, Err(error)) => {
                        self.error.get_or_insert(error);
                    }
                }
            }
            CpuPaintOp::DrawColor { color, blend } => self.canvas.draw_native_paint(
                command::Color {
                    red: color.red,
                    green: color.green,
                    blue: color.blue,
                    alpha: color.alpha * folded_alpha * self.opacity,
                },
                *blend,
            ),
            CpuPaintOp::DrawTransparentColorSrc => self.canvas.clear_color_src(command::Color {
                red: 0.,
                green: 0.,
                blue: 0.,
                alpha: 0.,
            }),
            CpuPaintOp::DrawRecord(record) => self.record(record, params),
            // Source: cc/paint/paint_op.cc:1669-1680. Same native scoped scroll input.
            CpuPaintOp::DrawScrollingContents { element, list } => {
                self.simple(CommandKind::kSave);
                if let Some(offset) = params
                    .raster_inducing_scroll_offsets
                    .and_then(|scrolls| scrolls.get(element))
                {
                    self.translate(-offset.x(), -offset.y());
                    list.RasterWithParams(self, params);
                } else {
                    self.error.get_or_insert_with(|| {
                        io::Error::other(
                            "native DrawScrollingContents requires its actual scroll offset",
                        )
                    });
                }
                self.simple(CommandKind::kRestore);
            }
            CpuPaintOp::Noop => {}
        }
    }
}
impl DisplayCanvas for ReplayCanvas {
    fn LocalClipBounds(&self) -> Option<Rect> {
        let rect = self.canvas.getLocalClipBounds()?;
        let left = rect.x.floor() as i32;
        let top = rect.y.floor() as i32;
        let right = (rect.x + rect.width).ceil() as i32;
        let bottom = (rect.y + rect.height).ceil() as i32;
        Some(Rect::new(
            left,
            top,
            right.saturating_sub(left),
            bottom.saturating_sub(top),
        ))
    }
}
impl CpuPaintCanvas<SkRegion> for ReplayCanvas {
    fn Playback(
        &mut self,
        buffer: &CpuPaintOpBuffer<SkRegion>,
        params: &mut CpuPlaybackParams<'_, SkRegion>,
        local_ctm: bool,
        offsets: Option<&[usize]>,
    ) {
        if self.error.is_some() {
            return;
        }
        // Source: cc/paint/paint_op_buffer.cc:362-374. Restore to the original save
        // count and add a save only when this record must not change the caller CTM.
        let depth = self.depth;
        if local_ctm {
            self.simple(CommandKind::kSave);
        }
        // Source: cc/paint/paint_op_buffer.cc:399-441. Folding carries a
        // separate PaintFlags alpha and can expose an op in a nested record.
        // Resolve resources from that actual record, never the outer buffer.
        use paint::cc::paint::paint_op_buffer_iterator::{FoldedOp, PlaybackFoldingIterator};
        for item in PlaybackFoldingIterator::new(buffer, offsets) {
            if self.error.is_some() {
                break;
            }
            match item.op {
                FoldedOp::Borrowed(CpuPaintOpRef::Native { op, record }) => {
                    if op.r#type == paint::paint_engine::DisplayItemType::kDrawImageRect
                        && self.provider_image_rect(op, record, item.alpha, params)
                    {
                        continue;
                    }
                    let mut command = op.to_skia();
                    command.flags_alpha = item.alpha;
                    command.color.alpha *= item.alpha;
                    if let Some(shader) = &mut command.paint_shader {
                        shader.flags_alpha *= item.alpha;
                        for stop in &mut shader.stops {
                            stop.color.alpha *= item.alpha;
                        }
                    }
                    self.command(&command, &resources(record.artifact()));
                }
                FoldedOp::Borrowed(CpuPaintOpRef::Converted(op)) => {
                    self.converted(op, params, item.alpha)
                }
                FoldedOp::OwnedDrawColor { color } => {
                    // The folded source DrawColor always uses SrcOver.
                    self.canvas.draw_native_paint(
                        command::Color {
                            red: color[0],
                            green: color[1],
                            blue: color[2],
                            alpha: color[3] * self.opacity,
                        },
                        skia::BlendMode::kSrcOver,
                    );
                }
            }
        }
        while self.depth > depth {
            self.simple(CommandKind::kRestore);
        }
    }
}
fn rect_command(rect: Rect) -> PaintRect {
    PaintRect {
        x: rect.x as f64,
        y: rect.y as f64,
        width: rect.width as f64,
        height: rect.height as f64,
    }
}
fn float_rect(rect: compositor_core::adapter::geometry::RectF) -> PaintRect {
    PaintRect {
        x: rect.x() as f64,
        y: rect.y() as f64,
        width: rect.width() as f64,
        height: rect.height() as f64,
    }
}
fn foundation_rect(rect: Rect) -> foundation::gfx::Rect {
    foundation::gfx::Rect::new(
        foundation::gfx::Point::new(rect.x, rect.y),
        foundation::gfx::Size::new(rect.width, rect.height),
    )
}
fn cc_rect(rect: foundation::gfx::Rect) -> Rect {
    Rect::new(rect.x(), rect.y(), rect.width(), rect.height())
}
fn new_canvas(
    size: Size,
    format: PixelFormat,
    storage: Vec<u8>,
    cache: &mut RasterImageCache,
) -> io::Result<ReplayCanvas> {
    let mut canvas = Canvas::make_raster_direct_with_format_preserving(
        &ResourceContext { resources: None },
        size.width() as u32,
        size.height() as u32,
        size.width() as usize * 4,
        PixelStorage::owned(storage),
        format,
        false,
    )
    .ok_or_else(|| io::Error::other("Skia rejected native CPU resource metadata"))?;
    canvas.install_image_cache(std::mem::take(cache));
    Ok(ReplayCanvas {
        canvas,
        error: None,
        depth: 0,
        nearest: false,
        opacity: 1.,
    })
}
fn finish(mut canvas: ReplayCanvas, cache: &mut RasterImageCache) -> io::Result<Vec<u8>> {
    *cache = canvas.canvas.take_image_cache();
    if let Some(error) = canvas.error {
        return Err(error);
    }
    Ok(canvas.canvas.finish_direct().into_vec())
}
// Source: cc/raster/raster_source.cc:43-139. Only Canvas execution and native
// immutable source values cross this boundary, never tile/owner decisions.
pub(super) fn RasterSource(
    request: RasterPlayback<
        '_,
        CpuRasterSource<SkRegion>,
        RasterTransform,
        CpuImageProviderRef<'_, SkRegion>,
        ScrollOffsetMap,
    >,
    size: Size,
    format: PixelFormat,
    storage: Vec<u8>,
    cache: &mut RasterImageCache,
) -> io::Result<Vec<u8>> {
    let mut canvas = new_canvas(size, format, storage, cache)?;
    let mut raster_bounds = *request.content_rect;
    if !request.invalid_content_rect.IsEmpty() {
        raster_bounds.Intersect(*request.invalid_content_rect);
        if raster_bounds.IsEmpty() {
            return finish(canvas, cache);
        }
    }
    let partial = request.content_rect != request.invalid_content_rect;
    let source = request.source;
    if !source.requires_clear {
        let scale = request.raster_transform.scale();
        let translation = request.raster_transform.translation();
        // Source: RasterSource::GetContentSize cc152-155 ->
        // gfx::ScaleToCeiledSize size.cc77-81, preserving f32 SizeF arithmetic.
        let content = if scale.x() == 1. && scale.y() == 1. {
            source.size()
        } else {
            Size::new(
                (source.size().width() as f32 * scale.x()).ceil() as i32,
                (source.size().height() as f32 * scale.y()).ceil() as i32,
            )
        };
        let clear_scale = foundation::gfx::Vector2dF::new(
            scale.x() / source.recording_scale_factor,
            scale.y() / source.recording_scale_factor,
        );
        let mut outer = foundation::gfx::Rect::default();
        let mut inner = foundation::gfx::Rect::default();
        if paint::cc::paint::clear_for_opaque_raster::CalculateClearForOpaqueRasterRects(
            &foundation::gfx::Vector2dF::new(translation.x(), translation.y()),
            &clear_scale,
            &foundation::gfx::Size::new(content.width(), content.height()),
            &foundation_rect(*request.content_rect),
            &foundation_rect(*request.invalid_content_rect),
            &mut outer,
            &mut inner,
        ) {
            canvas.simple(CommandKind::kSave);
            canvas.clip(cc_rect(outer), false);
            if !inner.IsEmpty() {
                canvas.clip(cc_rect(inner), true);
            }
            let color = source.background_color;
            canvas.canvas.clear_color_src(command::Color {
                red: color.red,
                green: color.green,
                blue: color.blue,
                alpha: color.alpha,
            });
            canvas.simple(CommandKind::kRestore);
        }
    } else if !partial {
        canvas.canvas.clear_color_src(command::Color {
            red: 0.,
            green: 0.,
            blue: 0.,
            alpha: 0.,
        });
    }
    canvas.simple(CommandKind::kSave);
    canvas.translate(
        -request.content_rect.x as f32,
        -request.content_rect.y as f32,
    );
    canvas.clip(raster_bounds, false);
    let translation = request.raster_transform.translation();
    let scale = request.raster_transform.scale();
    canvas.translate(translation.x(), translation.y());
    canvas.scale(
        scale.x() / source.recording_scale_factor,
        scale.y() / source.recording_scale_factor,
    );
    if partial && source.requires_clear {
        canvas.canvas.clear_color_src(command::Color {
            red: 0.,
            green: 0.,
            blue: 0.,
            alpha: 0.,
        });
    }
    let list = source
        .GetDisplayItemList()
        .as_ref()
        .ok_or_else(|| io::Error::other("native RasterSource has no display list"))?;
    let mut params = CpuPlaybackParams {
        raster_inducing_scroll_offsets: Some(request.raster_inducing_scroll_offsets),
        image_provider: Some(request.image_provider),
    };
    for _ in 0..source.slow_down_raster_scale_factor_for_debug.max(1) {
        list.RasterWithParams(&mut canvas, &mut params);
    }
    canvas.simple(CommandKind::kRestore);
    finish(canvas, cache)
}
// Source: components/viz/service/display/software_renderer.cc:398-431;
// skia/ext/opacity_filter_canvas.cc:9-47. The caller has already concatenated
// RectToRect(texture, quad). This routine applies content origin, clip and scale
// to the very same pass canvas and filters each playback primitive's paint.
#[allow(clippy::too_many_arguments)]
pub(super) fn RasterPictureInto(
    list: &ContentDisplayItemList<SkRegion>,
    content: Rect,
    scale: f32,
    nearest: bool,
    animation: &std::collections::BTreeMap<
        compositor_core::adapter::image_animation::PaintImageId,
        usize,
    >,
    scrolls: &ScrollOffsetMap,
    opacity: f32,
    mut target: Canvas,
    cache: &mut RasterImageCache,
) -> io::Result<Canvas> {
    // AnimatedImagesProvider requires a PaintImage frame-selection resolver.
    // CpuPaintImage's existing resource ABI has no such resolver; rejecting the
    // missing capability preserves correctness instead of choosing frame zero.
    if !animation.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "animated Picture replay requires the native PaintImage frame resolver",
        ));
    }
    let original_cache = target.take_image_cache();
    target.install_image_cache(std::mem::take(cache));
    // Source uses SkScalarRoundToInt(opacity*255)<255 to choose the filter
    // canvas. A nearest-only wrapper still applies opacity_ when it is <1.
    let wrap = (opacity * 255.).round() < 255. || nearest;
    let mut replay = ReplayCanvas {
        canvas: target,
        error: None,
        depth: 0,
        nearest,
        opacity: if wrap && opacity < 1. { opacity } else { 1. },
    };
    replay.simple(CommandKind::kSave);
    replay.translate(-content.x as f32, -content.y as f32);
    replay.clip(content, false);
    replay.scale(scale, scale);
    list.RasterWithParams(
        &mut replay,
        &mut CpuPlaybackParams {
            raster_inducing_scroll_offsets: Some(scrolls),
            image_provider: None,
        },
    );
    replay.simple(CommandKind::kRestore);
    *cache = replay.canvas.take_image_cache();
    replay.canvas.install_image_cache(original_cache);
    if let Some(error) = replay.error {
        return Err(error);
    }
    Ok(replay.canvas)
}

#[derive(Clone)]
struct NativeLooperPaint {
    color: skia::include::core::SkColor::SkColor4f,
    sigma: Option<(f32, bool)>,
    blend: skia::BlendMode,
    antialias: bool,
}
impl paint::cc::paint::draw_looper::LooperPaint for NativeLooperPaint {
    fn setAlpha(&mut self, alpha: u8) {
        self.color.fA = f32::from(alpha) * (1. / 255.);
    }
    fn setMaskFilterNormalBlur(&mut self, sigma: f32, respect_ctm: bool) {
        self.sigma = Some((sigma, respect_ctm));
    }
    fn setColorFilterSRGBBlend(
        &mut self,
        color: skia::include::core::SkColor::SkColor4f,
        mode: skia::BlendMode,
    ) {
        assert_eq!(
            mode,
            skia::BlendMode::kSrcIn,
            "canonical DrawLooper native color-filter invariant"
        );
        let alpha = self.color.fA;
        self.color = color;
        self.color.fA *= alpha;
    }
}
impl paint::cc::paint::draw_looper::LooperCanvas for ReplayCanvas {
    type Matrix = skia::raster::Transform;
    fn getSaveCount(&self) -> i32 {
        self.depth as i32 + 1
    }
    fn save(&mut self) {
        self.simple(CommandKind::kSave);
    }
    fn restoreToCount(&mut self, count: i32) {
        while self.depth as i32 + 1 > count {
            self.simple(CommandKind::kRestore);
        }
    }
    fn getLocalToDevice(&self) -> Self::Matrix {
        self.canvas.local_to_device()
    }
    fn setMatrixPostTranslate(&mut self, mut matrix: Self::Matrix, x: f32, y: f32) {
        matrix.tx += x;
        matrix.ty += y;
        self.canvas.set_matrix(matrix);
    }
    fn translate(&mut self, x: f32, y: f32) {
        ReplayCanvas::translate(self, x, y);
    }
}
fn native_radii(rect: &compositor_core::adapter::geometry::RRectF) -> command::PaintCornerRadii {
    let radius = |i: usize| command::PaintCornerRadius {
        x: rect.corner_radii[i].x() as f64,
        y: rect.corner_radii[i].y() as f64,
    };
    command::PaintCornerRadii {
        top_left: radius(0),
        top_right: radius(1),
        bottom_right: radius(2),
        bottom_left: radius(3),
    }
}
impl ReplayCanvas {
    fn looped_shape(
        &mut self,
        outer: &compositor_core::adapter::geometry::RRectF,
        inner: Option<&compositor_core::adapter::geometry::RRectF>,
        flags: &compositor_core::runtime::paint::CpuPaintFlags,
        alpha: f32,
    ) {
        let mut color = *flags.getColor4f();
        color.fA *= alpha;
        let paint = NativeLooperPaint {
            color,
            sigma: None,
            blend: flags.getBlendMode(),
            antialias: flags.isAntiAlias(),
        };
        let draw = |canvas: &mut Self, p: NativeLooperPaint| {
            let color = command::Color {
                red: p.color.fR,
                green: p.color.fG,
                blue: p.color.fB,
                alpha: p.color.fA * canvas.opacity,
            };
            if let Some((sigma, respect_ctm)) = p.sigma {
                canvas.canvas.draw_native_shadow_rrect(
                    float_rect(outer.rect),
                    native_radii(outer),
                    inner.map(|r| (float_rect(r.rect), native_radii(r))),
                    command::Offset::default(),
                    sigma,
                    respect_ctm,
                    color,
                    p.antialias,
                );
            } else if let Some(inner) = inner {
                canvas.canvas.draw_native_drrect(
                    float_rect(outer.rect),
                    native_radii(outer),
                    float_rect(inner.rect),
                    native_radii(inner),
                    color,
                    p.blend,
                    p.antialias,
                );
            } else {
                canvas.canvas.draw_native_rrect(
                    float_rect(outer.rect),
                    native_radii(outer),
                    color,
                    p.blend,
                    p.antialias,
                );
            }
        };
        if let Some(looper) = &flags.looper {
            looper.Apply(self, &paint, draw);
        } else {
            draw(self, paint);
        }
    }
    fn shader_rect(
        &mut self,
        rect: compositor_core::adapter::geometry::RectF,
        flags: &compositor_core::runtime::paint::CpuPaintFlags,
        folded_alpha: f32,
        params: &mut CpuPlaybackParams<'_, SkRegion>,
    ) {
        use compositor_core::runtime::paint::CpuPaintShader;
        let Some(shader) = &flags.shader else {
            let c = flags.getColor4f();
            self.canvas.draw_native_rect(
                float_rect(rect),
                command::Color {
                    red: c.fR,
                    green: c.fG,
                    blue: c.fB,
                    alpha: c.fA * folded_alpha * self.opacity,
                },
                flags.getBlendMode(),
                flags.isAntiAlias(),
            );
            return;
        };
        if let CpuPaintShader::PaintRecord {
            record,
            tile_rect,
            tile_mode_x,
            tile_mode_y,
            local_matrix,
        } = shader.as_ref()
        {
            let mut local = skia::raster::Transform::identity();
            if let Some(m) = local_matrix {
                let m = m.native();
                if m.rc(3, 0) != 0. || m.rc(3, 1) != 0. || m.rc(3, 3) != 1. {
                    self.fail("CPU record shader requires the native perspective shader pipeline");
                    return;
                }
                local = skia::raster::Transform::from_row(
                    m.rc(0, 0) as f32,
                    m.rc(1, 0) as f32,
                    m.rc(0, 1) as f32,
                    m.rc(1, 1) as f32,
                    m.rc(0, 3) as f32,
                    m.rc(1, 3) as f32,
                );
            }
            let total = self.canvas.local_to_device().pre_concat(local);
            let Some(tile) = skia::src::shaders::SkPictureShader::raster_tile(
                [
                    tile_rect.x(),
                    tile_rect.y(),
                    tile_rect.right(),
                    tile_rect.bottom(),
                ],
                total,
            ) else {
                return;
            };
            // SkPictureShader::CachedImageInfo::makeImage allocates one transparent
            // N32 tile. Allocation failure is an explicit backend failure, not
            // an opaque/empty substitute or an unchecked Canvas::new panic.
            let Some(row_bytes) = (tile.width as usize).checked_mul(4) else {
                self.fail("CPU record shader row size overflow");
                return;
            };
            let Some(length) = row_bytes.checked_mul(tile.height as usize) else {
                self.fail("CPU record shader allocation size overflow");
                return;
            };
            let mut pixels = Vec::new();
            if let Err(error) = pixels.try_reserve_exact(length) {
                self.error.get_or_insert_with(|| io::Error::other(error));
                return;
            }
            pixels.resize(length, 0);
            let context = ResourceContext { resources: None };
            let Some(mut canvas) = Canvas::make_raster_direct_with_format_preserving(
                &context,
                tile.width,
                tile.height,
                row_bytes,
                PixelStorage::owned(pixels),
                skia::PixelFormat::Rgba8888,
                false,
            ) else {
                self.fail("Skia rejected CPU record shader N32 tile");
                return;
            };
            canvas.set_matrix(tile.record_to_tile);
            let mut replay = Self {
                canvas,
                error: None,
                depth: 0,
                nearest: false,
                opacity: 1.,
            };
            replay.record(record, params);
            if let Some(error) = replay.error {
                self.error.get_or_insert(error);
                return;
            }
            let pixels = replay.canvas.finish_direct().into_vec();
            let Some(image) = skia::raster::PixmapRef::from_bytes(&pixels, tile.width, tile.height)
            else {
                self.fail("CPU record shader returned invalid N32 tile metadata");
                return;
            };
            self.canvas.draw_record_shader_rect(
                image,
                local.pre_concat(tile.image_to_record),
                float_rect(rect),
                *tile_mode_x,
                *tile_mode_y,
                skia::raster::FilterQuality::Bilinear,
                flags.getAlphaf() * folded_alpha * self.opacity,
                flags.getBlendMode(),
                flags.isAntiAlias(),
            );
            return;
        }
        let (source, mask_layer) = shader
            .source()
            .expect("native shader variant; record shader dispatched above");
        let mut command = if let Some(index) = mask_layer {
            let layer = &source.op().mask_layers[index];
            DrawCommand {
                rect: layer.clip_rect.to_skia(),
                source_rect: layer.source_rect.to_skia(),
                tile_rect: layer.tile_rect.to_skia(),
                resource_id: layer.resource_id,
                image_interpolation_quality: layer.image_interpolation_quality.to_skia(),
                repeat_x: layer.repeat_x,
                repeat_y: layer.repeat_y,
                tile_scale: layer.tile_scale.to_skia(),
                tile_spacing: layer.tile_spacing.to_skia(),
                paint_shader: layer.paint_shader.to_skia(),
                ..Default::default()
            }
        } else {
            source.op().to_skia()
        };
        command.r#type = match shader.as_ref() {
            CpuPaintShader::PaintRecord { .. } => unreachable!("record shader dispatched above"),
            CpuPaintShader::ImagePattern { .. } => CommandKind::kDrawTiledImage,
            CpuPaintShader::GeneratedPattern { .. } => {
                if source.op().r#type == paint::paint_engine::DisplayItemType::kDrawGradientRect
                    && mask_layer.is_none()
                {
                    CommandKind::kDrawGradientRect
                } else {
                    CommandKind::kDrawTiledGradient
                }
            }
        };
        command.rect = float_rect(rect);
        command.flags_alpha = flags.getAlphaf() * folded_alpha;
        if let Some(shader) = command.paint_shader.as_mut() {
            shader.flags_alpha *= command.flags_alpha;
            for stop in &mut shader.stops {
                stop.color.alpha *= command.flags_alpha;
            }
        }
        // OpacityFilterCanvas overrides drawImage sampling, not sampling
        // embedded in a DrawRect's SkShader. This local tiled-image command
        // executes that shader, so preserve its authored filtering choice.
        let nearest = self.nearest;
        self.nearest = false;
        self.command(&command, &resources(source.record().artifact()));
        self.nearest = nearest;
    }
}

impl ReplayCanvas {
    // Source: cc/paint/paint_op.cc:1448-1490. The owned scoped result outlives
    // sampling; source crop offset and scale come from that actual result.
    fn provider_image_rect(
        &mut self,
        op: &paint::paint_engine::DisplayItem,
        record: &compositor_core::runtime::paint::PaintRecord,
        alpha: f32,
        params: &mut CpuPlaybackParams<'_, SkRegion>,
    ) -> bool {
        use compositor_core::runtime::paint::{
            CpuDrawImage, CpuFilterQuality, CpuPaintImage, CpuScopedRasterResult,
        };
        let Some(provider) = params.image_provider.as_deref_mut() else {
            return false;
        };
        let image = match CpuPaintImage::FromPaintResource(record, op.resource_id) {
            Ok(image) => image,
            Err(error) => {
                self.error.get_or_insert_with(|| io::Error::other(error));
                return true;
            }
        };
        if op.image_orientation.to_skia() != command::ImageOrientation::kDefault {
            self.fail("decoded image use requires the source oriented PaintImage owner");
            return true;
        }
        let src = op.source_rect;
        let dst = op.rect;
        if src.width <= 0. || src.height <= 0. || dst.width <= 0. || dst.height <= 0. {
            return true;
        }
        let transform = self.canvas.local_to_device();
        let sx = (dst.width / src.width) as f32;
        let sy = (dst.height / src.height) as f32;
        let quality = match if self.nearest {
            command::ImageInterpolationQuality::kNone
        } else {
            op.image_interpolation_quality.to_skia()
        } {
            command::ImageInterpolationQuality::kNone => CpuFilterQuality::kNone,
            command::ImageInterpolationQuality::kLow => CpuFilterQuality::kLow,
            command::ImageInterpolationQuality::kMedium => CpuFilterQuality::kMedium,
        };
        let draw = CpuDrawImage {
            image,
            source_rect: Rect::new(
                src.x.floor() as i32,
                src.y.floor() as i32,
                ((src.x + src.width).ceil() - src.x.floor()) as i32,
                ((src.y + src.height).ceil() - src.y.floor()) as i32,
            ),
            filter_quality: quality,
            use_dark_mode: false,
            scale: compositor_core::adapter::geometry::Vector2dF::new(
                (transform.sx * sx).hypot(transform.ky * sx),
                (transform.kx * sy).hypot(transform.sy * sy),
            ),
            matrix_is_decomposable: true,
            frame_index: compositor_core::adapter::image_animation::kDefaultFrameIndex,
            target: None,
        };
        match provider.GetRasterContent(&draw) {
            CpuScopedRasterResult::Unavailable => {}
            CpuScopedRasterResult::Record(record) => {
                // Source: paint_op.cc:1404-1445. This is actual deferred record
                // content; all nested image lookups reborrow the same provider.
                self.simple(CommandKind::kSave);
                self.translate(dst.x as f32, dst.y as f32);
                self.scale(sx, sy);
                self.translate(-src.x as f32, -src.y as f32);
                self.command(
                    &DrawCommand {
                        r#type: CommandKind::kClipRect,
                        rect: src.to_skia(),
                        ..Default::default()
                    },
                    &ResourceContext::default(),
                );
                self.record(&record, params);
                self.simple(CommandKind::kRestore);
            }
            CpuScopedRasterResult::Decoded(lease) => {
                let size = lease.size();
                let format = match pixel_format(lease.format()) {
                    Ok(format) => format,
                    Err(error) => {
                        self.error.get_or_insert(error);
                        return true;
                    }
                };
                if lease.row_bytes() != size.width() as usize * 4 {
                    self.fail("decoded image native pixmap row stride is not supported by the current Skia value ABI");
                    return true;
                }
                let Some(pixels) = skia::raster::PixmapRef::from_bytes_with_format(
                    lease.pixels(),
                    size.width() as u32,
                    size.height() as u32,
                    format,
                ) else {
                    self.error.get_or_insert_with(|| {
                        io::Error::other("invalid native decoded image allocation")
                    });
                    return true;
                };
                let offset = lease.src_rect_offset();
                let scale = lease.scale_adjustment();
                let source = PaintRect {
                    x: (src.x + f64::from(offset.width())) * f64::from(scale.width()),
                    y: (src.y + f64::from(offset.height())) * f64::from(scale.height()),
                    width: src.width * f64::from(scale.width()),
                    height: src.height * f64::from(scale.height()),
                };
                let quality = match if self.nearest {
                    CpuFilterQuality::kNone
                } else {
                    lease.filter_quality()
                } {
                    CpuFilterQuality::kNone => skia::FilterQuality::Nearest,
                    CpuFilterQuality::kLow | CpuFilterQuality::kMedium => {
                        skia::FilterQuality::Bilinear
                    }
                    CpuFilterQuality::kHigh => {
                        self.fail("native decoded image cubic sampling needs the source BC sampling value path");
                        return true;
                    }
                };
                self.canvas.draw_premultiplied_image(
                    pixels,
                    source,
                    dst.to_skia(),
                    quality,
                    alpha * self.opacity,
                    skia::BlendMode::kSrcOver,
                    op.antialias,
                );
            }
        }
        true
    }
}
