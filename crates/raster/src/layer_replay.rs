//! CPU compilation and playback of the records carried by a FramePlan.
use crate::convert::{resources, ToSkia};
pub use layer_tile::recording::ReplayUnsupported;
use layer_tile::recording::{
    aligned_rect, concatenate_affine, intersect, intersection, is_draw, is_rect_draw, mapped_rect,
    record_pixel_bounds, State,
};
use layer_tile::{FramePlan, LayerId, RasterTask};
use paint::paint_engine::{DisplayItem, DisplayItemType as Kind, PaintRect};
use paint::paint_property_tree::{
    ClipPaintPropertyNode, EffectPaintPropertyNode, PropertyTreeState,
};
use skia::compat::commands::{DrawCommand, ResourceContext};
use skia::src::core::SkCanvas::RasterImageCache;
use skia::RasterClipProductCache;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

struct PreparedRecord {
    commands: Vec<DrawCommand>,
    source_items: Vec<Option<DisplayItem>>,
    decoration_glyph_runs: Vec<Vec<DrawCommand>>,
    draw_support: Vec<Option<PaintRect>>,
    raster_guard: u32,
    shadow_regions: Vec<PaintRect>,
}
// The actual receiver allocates ceil(3*sigma)+1 source padding for both outer
// shadows crossing its device and every inset shadow (canvas.rs:1640,2139).
// Tiles must keep that implementation, not clip the blur at their boundary.
// Only its extra AA/support fringe needs a destination gutter; the full blur
// kernel is already sampled from the receiver's separately padded A8 mask.
fn shadow_support(item: &DisplayItem, state: State, scale: f64) -> PaintRect {
    let mut source = item.rect;
    if !item.inset {
        source = PaintRect {
            x: source.x - item.spread + item.shadow_offset.x,
            y: source.y - item.spread + item.shadow_offset.y,
            width: (source.width + 2.0 * item.spread).max(0.0),
            height: (source.height + 2.0 * item.spread).max(0.0),
        };
    }
    let mut bounds = mapped_rect(source, state);
    let sigma = (item.blur_radius * 0.5) as f32;
    let (dx, dy) = if state.cross_axis == (0.0, 0.0) {
        (
            ((scale as f32 * state.axis_scale.0 as f32) * sigma).abs(),
            ((scale as f32 * state.axis_scale.1 as f32) * sigma).abs(),
        )
    } else {
        (
            skia::Point::from_xy(
                (scale as f32 * state.axis_scale.0 as f32) * sigma,
                (scale as f32 * state.cross_axis.1 as f32) * sigma,
            )
            .length(),
            skia::Point::from_xy(
                (scale as f32 * state.cross_axis.0 as f32) * sigma,
                (scale as f32 * state.axis_scale.1 as f32) * sigma,
            )
            .length(),
        )
    };
    let sigma = (dx * dy).sqrt().min(128.0);
    let extent = if item.inset {
        1.0 / scale
    } else {
        ((3.0 * sigma).ceil() as f64 + 2.0) / scale
    };
    bounds.x -= extent;
    bounds.y -= extent;
    bounds.width += 2.0 * extent;
    bounds.height += 2.0 * extent;
    bounds
}
pub(crate) struct PreparedLayer {
    composition: RasterLayerState,
    records: BTreeMap<usize, PreparedRecord>,
}

#[derive(Clone, Copy)]
pub(crate) struct RasterLayerState {
    pub white_backing: bool,
}
pub struct LayerReplay<'a> {
    resources: ResourceContext<'a>,
    documents: BTreeMap<u64, Arc<image_resource::DocumentPaintContent>>,
    layers: BTreeMap<LayerId, Arc<PreparedLayer>>,
    scratch: Vec<u8>,
}
pub(crate) fn document_records(
    artifact: &paint::paint_engine::PaintArtifact,
) -> BTreeMap<u64, Arc<image_resource::DocumentPaintContent>> {
    artifact
        .resources
        .iter()
        .flat_map(|resources| &resources.images)
        .filter_map(|image| match &image.content {
            image_resource::PaintImageContent::Document(record) => Some((image.id, record.clone())),
            image_resource::PaintImageContent::Bitmap(_) => None,
        })
        .collect()
}

fn validate_document_clip_chain(
    leaf: &Arc<ClipPaintPropertyNode>,
) -> Result<(), ReplayUnsupported> {
    let mut current = Some(leaf);
    while let Some(clip) = current {
        if clip.clip_path_error.is_some() {
            return Err(ReplayUnsupported("unsupported-document-clip-path"));
        }
        // LayoutSVGResourceClipper unions independently filled child paths.
        // A single SkCanvas clipPath of their concatenation cannot represent
        // that union (even-odd overlaps and opposite winding cancel). This
        // document replay route has no PathOps/mask-union owner yet. Reject
        // before replay rather than silently replacing union with winding.
        if clip.clip_path_children.len() > 1 {
            return Err(ReplayUnsupported("union-document-clip-requires-mask"));
        }
        current = clip.parent.as_ref();
    }
    Ok(())
}

fn validate_document_artifact(
    artifact: &paint::paint_engine::PaintArtifact,
    depth: usize,
) -> Result<(), ReplayUnsupported> {
    for chunk in &artifact.chunks {
        let records = artifact.display_items.get(chunk.begin_index as usize..chunk.end_index as usize)
            .ok_or(ReplayUnsupported("invalid-document-record-range"))?;
        for record in records {
            let raw = artifact.items.get(record.record_begin..record.record_end)
                .ok_or(ReplayUnsupported("invalid-document-record-range"))?;
            layer_tile::recording::mask_record_body(raw, chunk.properties.effect.is_mask)?;
        }
        validate_document_clip_chain(&chunk.properties.clip)?;
        let mut current = Some(&chunk.properties.effect);
        while let Some(effect) = current {
            if let Some(clip) = &effect.output_clip {
                validate_document_clip_chain(clip)?;
            }
            current = effect.parent.as_ref();
        }
    }
    validate_document_images(artifact, depth)
}

fn validate_document_images(
    artifact: &paint::paint_engine::PaintArtifact,
    depth: usize,
) -> Result<(), ReplayUnsupported> {
    let documents = document_records(artifact);
    for (id, key) in artifact.DocumentImageContainerRequirements() {
        let record = documents
            .get(&id)
            .and_then(|content| content.exact_record_for(&key))
            .ok_or(ReplayUnsupported("missing-document-image-container"))?;
        let record = record
            .as_any()
            .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
            .ok_or(ReplayUnsupported("unsupported-document-image-record"))?;
        // Keep admission bounded by the same nesting limit as document
        // replay; a deeper resource must not become an apparently empty image.
        if depth >= 8 {
            return Err(ReplayUnsupported("document-image-depth-exceeded"));
        }
        validate_document_artifact(&record.artifact, depth + 1)?;
    }
    Ok(())
}

fn hydrate_bitmap_pattern_shader(
    shader: &mut skia::compat::commands::PaintShader,
    resources: &layoutng_assembly::fragment_tree::PaintResources,
) {
    let Some(pattern) = shader.pattern.as_mut() else {
        return;
    };
    for command in &mut Arc::make_mut(pattern).commands {
        match command {
            skia::compat::commands::SvgPatternCommand::DrawImage(image) => {
                let Some(resource) = resources
                    .images
                    .iter()
                    .find(|resource| resource.id == image.resource_id)
                else {
                    continue;
                };
                let Some(pixels) = resource.BitmapPixels() else {
                    continue;
                };
                image.raster_width = resource.width;
                image.raster_height = resource.height;
                image.rgba8 = Arc::clone(pixels);
            }
            skia::compat::commands::SvgPatternCommand::DrawShape(shape)
            | skia::compat::commands::SvgPatternCommand::DrawGlyph(shape) => {
                if let Some(shader) = shape.fill_server.as_mut() {
                    hydrate_bitmap_pattern_shader(shader, resources);
                }
                if let Some(shader) = shape.stroke_server.as_mut() {
                    hydrate_bitmap_pattern_shader(shader, resources);
                }
            }
            skia::compat::commands::SvgPatternCommand::SaveLayerMask(mask) => {
                let nested = skia::compat::commands::PaintShader {
                    pattern: Some(Arc::new(skia::compat::commands::SvgPatternData {
                        commands: Arc::make_mut(mask).commands.clone(),
                        content_transform: Arc::make_mut(mask).content_transform,
                        opacity: 1.0,
                    })),
                    ..Default::default()
                };
                let mut nested = nested;
                hydrate_bitmap_pattern_shader(&mut nested, resources);
                Arc::make_mut(mask).commands = nested.pattern.unwrap().commands.clone();
            }
            _ => {}
        }
    }
}

fn hydrate_bitmap_patterns(
    command: &mut DrawCommand,
    resources: Option<&layoutng_assembly::fragment_tree::PaintResources>,
) {
    let Some(resources) = resources else { return };
    if let Some(shader) = command.paint_shader.as_mut() {
        hydrate_bitmap_pattern_shader(shader, resources);
    }
    for mask in &mut command.mask_layers {
        if let Some(shader) = mask.paint_shader.as_mut() {
            hydrate_bitmap_pattern_shader(shader, resources);
        }
    }
}

pub(crate) fn rasterize_pattern_document_record(
    record: &paint::paint_engine::DocumentPaintArtifactRecord,
    depth: usize,
) -> Option<(u32, u32, Vec<u8>)> {
    if depth >= 8 || record.record_size.width == 0 || record.record_size.height == 0 {
        return None;
    }
    // record_size is the unzoomed CSS viewport selected by
    // SVGImageForContainer. The outer Paint artifact already maps that
    // viewport through effective zoom, and pattern-tile replay subsequently
    // maps its logical tile to device pixels. Rasterizing this intermediate
    // document image at zoom * DPR would therefore apply zoom twice. Blink's
    // container scale leaves only the device-pixel density here.
    let scale = record.container_key.device_pixel_ratio();
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let width = (f64::from(record.record_size.width) * scale)
        .ceil()
        .max(1.0) as u32;
    let height = (f64::from(record.record_size.height) * scale)
        .ceil()
        .max(1.0) as u32;
    let byte_count = usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?
        .checked_mul(4)?;
    if width > 8192 || height > 8192 || byte_count > 256 * 1024 * 1024 {
        return None;
    }
    validate_document_artifact(&record.artifact, depth + 1).ok()?;
    let empty = ResourceContext::default();
    let mut canvas = skia::cpu::canvas::Canvas::new(&empty, width, height);
    canvas.clear_transparent();
    let mut transform = DisplayItem {
        r#type: Kind::kConcat,
        ..Default::default()
    };
    transform.transform.values[0] = f64::from(width) / f64::from(record.record_size.width);
    transform.transform.values[5] = f64::from(height) / f64::from(record.record_size.height);
    canvas.replay_item(&transform.to_skia(), &empty);
    replay_document_artifact(&mut canvas, &record.artifact, depth + 1);
    Some((width, height, canvas.finish()))
}

fn hydrate_document_pattern_shader(
    target: &mut skia::compat::commands::PaintShader,
    source: &layoutng_assembly::internal::paint_input::PaintShader,
    documents: &BTreeMap<u64, Arc<image_resource::DocumentPaintContent>>,
    depth: usize,
) {
    let (Some(target_pattern), Some(source_pattern)) =
        (target.pattern.as_mut(), source.pattern.as_ref())
    else {
        return;
    };
    for (target, source) in Arc::make_mut(target_pattern)
        .commands
        .iter_mut()
        .zip(&source_pattern.commands)
    {
        match (target, source) {
            (
                skia::compat::commands::SvgPatternCommand::DrawImage(target),
                layoutng_assembly::internal::paint_input::SvgPatternCommand::DrawImage(source),
            ) => {
                let Some(key) = source.document_container_key.as_ref() else {
                    continue;
                };
                let Some(record) = documents
                    .get(&source.resource_id)
                    .and_then(|content| content.exact_record_for(key))
                    .and_then(|record| {
                        record
                            .as_any()
                            .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
                    })
                else {
                    continue;
                };
                let Some((width, height, pixels)) =
                    rasterize_pattern_document_record(record, depth)
                else {
                    continue;
                };
                if record.intrinsic_size.width == 0 || record.intrinsic_size.height == 0 {
                    continue;
                }
                let sx = f64::from(width) / f64::from(record.intrinsic_size.width);
                let sy = f64::from(height) / f64::from(record.intrinsic_size.height);
                target.source.x *= sx;
                target.source.y *= sy;
                target.source.width *= sx;
                target.source.height *= sy;
                target.raster_width = width;
                target.raster_height = height;
                target.rgba8 = Arc::new(pixels);
            }
            (
                skia::compat::commands::SvgPatternCommand::DrawShape(target)
                | skia::compat::commands::SvgPatternCommand::DrawGlyph(target),
                layoutng_assembly::internal::paint_input::SvgPatternCommand::DrawShape(source)
                | layoutng_assembly::internal::paint_input::SvgPatternCommand::DrawGlyph(source),
            ) => {
                if let (Some(target), Some(source)) =
                    (target.fill_server.as_mut(), source.fill_server.as_ref())
                {
                    hydrate_document_pattern_shader(target, source, documents, depth);
                }
                if let (Some(target), Some(source)) =
                    (target.stroke_server.as_mut(), source.stroke_server.as_ref())
                {
                    hydrate_document_pattern_shader(target, source, documents, depth);
                }
            }
            (
                skia::compat::commands::SvgPatternCommand::SaveLayerMask(target),
                layoutng_assembly::internal::paint_input::SvgPatternCommand::SaveLayerMask(source),
            ) => {
                let target = Arc::make_mut(target);
                let mut target_shader = skia::compat::commands::PaintShader {
                    pattern: Some(Arc::new(skia::compat::commands::SvgPatternData {
                        commands: target.commands.clone(),
                        content_transform: target.content_transform,
                        opacity: 1.0,
                    })),
                    ..Default::default()
                };
                let source_shader = layoutng_assembly::internal::paint_input::PaintShader {
                    pattern: Some(Arc::new(
                        layoutng_assembly::internal::paint_input::SvgPatternData {
                            commands: source.commands.clone(),
                            ..Default::default()
                        },
                    )),
                    ..Default::default()
                };
                hydrate_document_pattern_shader(
                    &mut target_shader,
                    &source_shader,
                    documents,
                    depth,
                );
                target.commands = target_shader.pattern.unwrap().commands.clone();
            }
            _ => {}
        }
    }
}

fn hydrate_document_patterns(
    target: &mut DrawCommand,
    source: &DisplayItem,
    documents: &BTreeMap<u64, Arc<image_resource::DocumentPaintContent>>,
    depth: usize,
) {
    if let (Some(target), Some(source)) =
        (target.paint_shader.as_mut(), source.paint_shader.as_ref())
    {
        hydrate_document_pattern_shader(target, source, documents, depth);
    }
    for (target, source) in target.mask_layers.iter_mut().zip(&source.mask_layers) {
        if let (Some(target), Some(source)) =
            (target.paint_shader.as_mut(), source.paint_shader.as_ref())
        {
            hydrate_document_pattern_shader(target, source, documents, depth);
        }
    }
}

fn replay_document_image(
    canvas: &mut skia::cpu::canvas::Canvas,
    image: &DisplayItem,
    record: Option<&paint::paint_engine::DocumentPaintArtifactRecord>,
    depth: usize,
) -> bool {
    let Some(record) = record else { return false };
    if depth >= 8 || image.rect.width <= 0.0 || image.rect.height <= 0.0 {
        return false;
    }
    let mut source = image.source_rect;
    if source.width <= 0.0 || source.height <= 0.0 {
        source = PaintRect {
            x: 0.0,
            y: 0.0,
            width: record.record_size.width as f64,
            height: record.record_size.height as f64,
        };
    } else if record.intrinsic_size.width != 0 && record.intrinsic_size.height != 0 {
        // Outer DrawImage source coordinates are expressed in the resource's
        // intrinsic space. The nested PaintRecord is container-sized, just as
        // SVGImageForContainer records into its concrete container viewport.
        let scale_x = record.record_size.width as f64 / record.intrinsic_size.width as f64;
        let scale_y = record.record_size.height as f64 / record.intrinsic_size.height as f64;
        source.x *= scale_x;
        source.y *= scale_y;
        source.width *= scale_x;
        source.height *= scale_y;
    }
    if source.width <= 0.0 || source.height <= 0.0 {
        return false;
    }
    if std::env::var_os("BROWSER_TRACE_STYLE_IMAGES").is_some() {
        eprintln!(
            "document-image-replay resource_id={} destination={:?} source={:?} intrinsic={}x{} record={}x{} items={} display_items={} chunks={}",
            image.resource_id,
            image.rect,
            source,
            record.intrinsic_size.width,
            record.intrinsic_size.height,
            record.record_size.width,
            record.record_size.height,
            record.artifact.items.len(),
            record.artifact.display_items.len(),
            record.artifact.chunks.len(),
        );
    }
    let empty_resources = ResourceContext::default();
    let save_kind = if image.opacity < 1.0 {
        Kind::kSaveLayerAlpha
    } else {
        Kind::kSave
    };
    canvas.replay_item(
        &DisplayItem {
            r#type: save_kind,
            rect: image.rect,
            opacity: image.opacity,
            ..Default::default()
        }
        .to_skia(),
        &empty_resources,
    );
    canvas.replay_item(
        &DisplayItem {
            r#type: Kind::kClipRect,
            rect: image.rect,
            ..Default::default()
        }
        .to_skia(),
        &empty_resources,
    );
    let sx = image.rect.width / source.width;
    let sy = image.rect.height / source.height;
    let mut transform = DisplayItem {
        r#type: Kind::kConcat,
        ..Default::default()
    };
    transform.transform.values[0] = sx;
    transform.transform.values[5] = sy;
    transform.transform.values[12] = image.rect.x - source.x * sx;
    transform.transform.values[13] = image.rect.y - source.y * sy;
    canvas.replay_item(&transform.to_skia(), &empty_resources);

    replay_document_artifact(canvas, &record.artifact, depth + 1);
    canvas.replay_item(
        &DisplayItem {
            r#type: Kind::kRestore,
            ..Default::default()
        }
        .to_skia(),
        &empty_resources,
    );
    true
}

// Mirrors source_replay::DrawTiledImage, but each tile replays the selected
// SVG document record instead of asking the bitmap ResourceContext for pixels.
// Background and table-background producers provide tile_rect in destination
// coordinates, so this path has the same placement and repeat bounds as the
// ordinary bitmap implementation.
fn replay_tiled_document_image(
    canvas: &mut skia::cpu::canvas::Canvas,
    image: &DisplayItem,
    record: Option<&paint::paint_engine::DocumentPaintArtifactRecord>,
    depth: usize,
) -> bool {
    let Some(record) = record else { return false };
    let tile_rect = if image.tile_rect.width > 0.0 && image.tile_rect.height > 0.0 {
        image.tile_rect
    } else {
        PaintRect {
            x: image.source_rect.x * image.tile_scale.x + image.tile_phase.x,
            y: image.source_rect.y * image.tile_scale.y + image.tile_phase.y,
            width: image.source_rect.width * image.tile_scale.x,
            height: image.source_rect.height * image.tile_scale.y,
        }
    };
    let step_x = tile_rect.width + image.tile_spacing.width;
    let step_y = tile_rect.height + image.tile_spacing.height;
    if step_x <= 0.0 || step_y <= 0.0 {
        return false;
    }
    let first_x = if image.repeat_x {
        tile_rect.x - ((tile_rect.x - image.rect.x) / step_x).ceil() * step_x
    } else {
        tile_rect.x
    };
    let first_y = if image.repeat_y {
        tile_rect.y - ((tile_rect.y - image.rect.y) / step_y).ceil() * step_y
    } else {
        tile_rect.y
    };
    let empty_resources = ResourceContext::default();
    canvas.replay_item(
        &DisplayItem {
            r#type: Kind::kSave,
            ..Default::default()
        }
        .to_skia(),
        &empty_resources,
    );
    canvas.replay_item(
        &DisplayItem {
            r#type: Kind::kClipRect,
            rect: image.rect,
            ..Default::default()
        }
        .to_skia(),
        &empty_resources,
    );
    let mut tile = image.clone();
    tile.r#type = Kind::kDrawImageRect;
    let mut y = first_y;
    let mut rows = 0;
    while y < image.rect.y + image.rect.height && rows < 10_000 {
        rows += 1;
        let mut x = first_x;
        let mut columns = 0;
        while x < image.rect.x + image.rect.width && columns < 10_000 {
            columns += 1;
            tile.rect = PaintRect {
                x,
                y,
                width: tile_rect.width,
                height: tile_rect.height,
            };
            replay_document_image(canvas, &tile, Some(record), depth);
            x += if image.repeat_x {
                step_x
            } else {
                image.rect.width + step_x
            };
        }
        y += if image.repeat_y {
            step_y
        } else {
            image.rect.height + step_y
        };
    }
    canvas.replay_item(
        &DisplayItem {
            r#type: Kind::kRestore,
            ..Default::default()
        }
        .to_skia(),
        &empty_resources,
    );
    true
}

fn replay_mask_layers(
    canvas: &mut skia::cpu::canvas::Canvas,
    item: &DisplayItem,
    resources: &ResourceContext,
    documents: &BTreeMap<u64, Arc<image_resource::DocumentPaintContent>>,
    depth: usize,
) -> bool {
    if item.r#type != Kind::kDrawMask {
        return false;
    }
    let command = item.to_skia();
    debug_assert_eq!(command.mask_layers.len(), item.mask_layers.len());
    canvas.draw_mask_layers_with(&command.mask_layers, resources, |canvas, index, _| {
        let Some(layer) = item.mask_layers.get(index) else {
            return false;
        };
        let Some(key) = layer.document_image_container_key.as_ref() else {
            return false;
        };
        let Some(record) = documents
            .get(&layer.resource_id)
            .and_then(|content| content.record_for(key))
            .and_then(|record| {
                record
                    .as_any()
                    .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
            })
        else {
            return false;
        };
        replay_tiled_document_image(
            canvas,
            &DisplayItem {
                r#type: Kind::kDrawTiledImage,
                rect: layer.clip_rect,
                source_rect: layer.source_rect,
                tile_rect: layer.tile_rect,
                resource_id: layer.resource_id,
                image_interpolation_quality: layer.image_interpolation_quality,
                document_image_container_key: Some(key.clone()),
                repeat_x: layer.repeat_x,
                repeat_y: layer.repeat_y,
                tile_scale: layer.tile_scale,
                tile_spacing: layer.tile_spacing,
                ..Default::default()
            },
            Some(record),
            depth,
        )
    });
    true
}

#[derive(Clone)]
enum DocumentPropertyScope {
    Clip(Arc<ClipPaintPropertyNode>),
    Effect(Arc<EffectPaintPropertyNode>, bool),
}

impl DocumentPropertyScope {
    fn same_node(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Clip(left), Self::Clip(right)) => left.lifecycle.same_node(&right.lifecycle),
            (Self::Effect(left, _), Self::Effect(right, _)) => {
                left.lifecycle.same_node(&right.lifecycle)
            }
            _ => false,
        }
    }
}

fn document_clip_chain(
    leaf: &Arc<ClipPaintPropertyNode>,
    scopes: &mut Vec<DocumentPropertyScope>,
    seen: &mut Vec<Arc<ClipPaintPropertyNode>>,
) {
    let mut chain = Vec::new();
    let mut current = Some(leaf.clone());
    while let Some(clip) = current {
        current = clip.parent.clone();
        chain.push(clip);
    }
    for clip in chain.into_iter().rev() {
        if (clip.rect.is_some() || (clip.clip_path_is_empty || !clip.clip_path.is_empty()))
            && !seen
                .iter()
                .any(|old| old.lifecycle.same_node(&clip.lifecycle))
        {
            scopes.push(DocumentPropertyScope::Clip(clip.clone()));
            seen.push(clip);
        }
    }
}

fn document_effect_is_real(
    effect: &EffectPaintPropertyNode,
    masked_parents: &[Arc<EffectPaintPropertyNode>],
) -> bool {
    effect.is_mask
        || effect.opacity != 1.0
        || !effect.filters.is_empty()
        || effect.blend_mode != layoutng_assembly::internal::paint_input::PaintBlendMode::kNormal
        || effect.isolates_blending
        || masked_parents
            .iter()
            .any(|parent| parent.lifecycle.same_node(&effect.lifecycle))
}

// PaintChunksToCcLayer switches Effect, Clip and Transform in that order. The
// effect chain is shared across consecutive chunks and is closed only to the
// lifecycle-identity LCA. In particular, two chunks under opacity:.5 must be
// drawn into one surface and have alpha applied once after both are complete.
fn document_property_scopes(
    state: &PropertyTreeState,
    masked_parents: &[Arc<EffectPaintPropertyNode>],
) -> Vec<DocumentPropertyScope> {
    let mut scopes = Vec::new();
    let mut seen_clips = Vec::new();
    let mut effects = Vec::new();
    let mut current = Some(state.effect.clone());
    while let Some(effect) = current {
        current = effect.parent.clone();
        effects.push(effect);
    }
    for effect in effects.into_iter().rev() {
        if document_effect_is_real(&effect, masked_parents) {
            let force_isolation = masked_parents
                .iter()
                .any(|parent| parent.lifecycle.same_node(&effect.lifecycle));
            if effect.is_mask {
                scopes.push(DocumentPropertyScope::Effect(
                    effect.clone(),
                    force_isolation,
                ));
                if let Some(output_clip) = &effect.output_clip {
                    document_clip_chain(output_clip, &mut scopes, &mut seen_clips);
                }
            } else {
                if let Some(output_clip) = &effect.output_clip {
                    document_clip_chain(output_clip, &mut scopes, &mut seen_clips);
                }
                scopes.push(DocumentPropertyScope::Effect(effect, force_isolation));
            }
        }
    }
    document_clip_chain(&state.clip, &mut scopes, &mut seen_clips);
    scopes
}

fn inverse_affine(
    matrix: &layoutng_assembly::internal::layout_input::TransformMatrix,
) -> Option<layoutng_assembly::internal::layout_input::TransformMatrix> {
    let values = matrix.values;
    let determinant = values[0] * values[5] - values[4] * values[1];
    if !determinant.is_finite() || determinant.abs() <= f64::EPSILON {
        return None;
    }
    let mut inverse = layoutng_assembly::internal::layout_input::TransformMatrix::default();
    inverse.values[0] = values[5] / determinant;
    inverse.values[1] = -values[1] / determinant;
    inverse.values[4] = -values[4] / determinant;
    inverse.values[5] = values[0] / determinant;
    inverse.values[12] = -(inverse.values[0] * values[12] + inverse.values[4] * values[13]);
    inverse.values[13] = -(inverse.values[1] * values[12] + inverse.values[5] * values[13]);
    inverse
        .values
        .iter()
        .all(|value| value.is_finite())
        .then_some(inverse)
}

fn replay_concat(
    canvas: &mut skia::cpu::canvas::Canvas,
    matrix: layoutng_assembly::internal::layout_input::TransformMatrix,
    resources: &ResourceContext,
) {
    if matrix != Default::default() {
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kConcat,
                transform: matrix,
                ..Default::default()
            }
            .to_skia(),
            resources,
        );
    }
}

fn replay_document_clip_open(
    canvas: &mut skia::cpu::canvas::Canvas,
    clip: &ClipPaintPropertyNode,
    resources: &ResourceContext,
) {
    canvas.replay_item(
        &DisplayItem {
            r#type: Kind::kSave,
            ..Default::default()
        }
        .to_skia(),
        resources,
    );
    let Ok(matrix) = layer_tile::compositor_transform(&clip.local_transform_space, 1.0) else {
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kClipRect,
                rect: PaintRect::default(),
                ..Default::default()
            }
            .to_skia(),
            resources,
        );
        return;
    };
    let Some(_) = inverse_affine(&matrix) else {
        // A singular local space has no drawable projection. Keep the canvas
        // in the document root coordinate space for following sibling scopes.
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kClipRect,
                rect: PaintRect::default(),
                ..Default::default()
            }
            .to_skia(),
            resources,
        );
        return;
    };
    let saved_matrix = canvas.local_to_device();
    replay_concat(canvas, matrix, resources);
    // A path node's rect is its conservative scheduling bound, not an
    // additional clip. Applying both would multiply antialiased edge coverage.
    if let Some(rect) = clip
        .rect
        .filter(|_| !clip.clip_path_is_empty && clip.clip_path.is_empty())
    {
        canvas.replay_item(
            &DisplayItem {
                r#type: if clip.radii == Default::default() {
                    Kind::kClipRect
                } else {
                    Kind::kClipRoundedRect
                },
                rect,
                corner_radii: clip.radii,
                antialias: true,
                ..Default::default()
            }
            .to_skia(),
            resources,
        );
    }
    if (clip.clip_path_is_empty || !clip.clip_path.is_empty()) {
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kClipPath,
                path: clip.clip_path.clone(),
                even_odd: clip.clip_path_even_odd,
                antialias: true,
                ..Default::default()
            }
            .to_skia(),
            resources,
        );
    }
    // SkCanvas materializes the clip in device space. Restore the exact
    // receiver CTM without changing that clip. Concatenating an f64 inverse
    // through SkMatrix's f32 receiver introduces M*inverse(M) roundoff before
    // the next chunk, unlike PaintChunksToCcLayer's retained transform state.
    // No device/saveLayer boundary is crossed inside this clip scope.
    canvas.set_matrix(saved_matrix);
}

fn replay_document_effect_open(
    canvas: &mut skia::cpu::canvas::Canvas,
    effect: &EffectPaintPropertyNode,
    force_isolation: bool,
    resources: &ResourceContext,
) {
    if effect.is_mask {
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kSaveLayerDstIn,
                ..Default::default()
            }
            .to_skia(),
            resources,
        );
        return;
    }
    let blend_surface = force_isolation
        || effect.isolates_blending
        || effect.blend_mode != layoutng_assembly::internal::paint_input::PaintBlendMode::kNormal;
    if blend_surface {
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kSaveLayerBlend,
                blend_mode: effect.blend_mode,
                opacity: effect.opacity,
                ..Default::default()
            }
            .to_skia(),
            resources,
        );
    } else if effect.opacity != 1.0 {
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kSaveLayerAlpha,
                opacity: effect.opacity,
                ..Default::default()
            }
            .to_skia(),
            resources,
        );
    }
    if !effect.filters.is_empty() {
        let filter_transform = layer_tile::compositor_transform(&effect.local_transform_space, 1.0)
            .ok()
            .and_then(|matrix| inverse_affine(&matrix).map(|inverse| (matrix, inverse)));
        // SaveLayerFilter samples its CTM when the surface is opened. Restore
        // document-root coordinates inside the surface, then undo the sampled
        // CTM after the matching restore in replay_document_effect_close.
        if let Some((matrix, _)) = filter_transform {
            replay_concat(canvas, matrix, resources);
        }
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kSaveLayerFilter,
                filters: effect.filters.clone(),
                ..Default::default()
            }
            .to_skia(),
            resources,
        );
        if let Some((_, inverse)) = filter_transform {
            replay_concat(canvas, inverse, resources);
        }
    }
}

fn replay_document_effect_close(
    canvas: &mut skia::cpu::canvas::Canvas,
    effect: &EffectPaintPropertyNode,
    force_isolation: bool,
    resources: &ResourceContext,
) {
    if effect.is_mask {
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kRestore,
                ..Default::default()
            }
            .to_skia(),
            resources,
        );
        return;
    }
    if !effect.filters.is_empty() {
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kRestore,
                ..Default::default()
            }
            .to_skia(),
            resources,
        );
        if let Some(inverse) = layer_tile::compositor_transform(&effect.local_transform_space, 1.0)
            .ok()
            .and_then(|matrix| inverse_affine(&matrix))
        {
            replay_concat(canvas, inverse, resources);
        }
    }
    let blend_surface = force_isolation
        || effect.isolates_blending
        || effect.blend_mode != layoutng_assembly::internal::paint_input::PaintBlendMode::kNormal;
    if blend_surface || effect.opacity != 1.0 {
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kRestore,
                ..Default::default()
            }
            .to_skia(),
            resources,
        );
    }
}

fn replay_document_scope_close(
    canvas: &mut skia::cpu::canvas::Canvas,
    scope: &DocumentPropertyScope,
    resources: &ResourceContext,
) {
    match scope {
        DocumentPropertyScope::Clip(_) => canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kRestore,
                ..Default::default()
            }
            .to_skia(),
            resources,
        ),
        DocumentPropertyScope::Effect(effect, force_isolation) => {
            replay_document_effect_close(canvas, effect, *force_isolation, resources)
        }
    }
}

fn replay_document_artifact(
    canvas: &mut skia::cpu::canvas::Canvas,
    artifact: &paint::paint_engine::PaintArtifact,
    depth: usize,
) {
    let nested_resources = resources(artifact);
    let nested_documents = document_records(artifact);
    let empty_resources = ResourceContext::default();
    let replay_range = |canvas: &mut skia::cpu::canvas::Canvas, items: &[DisplayItem]| {
        for nested in items {
            let nested_record = nested_documents
                .get(&nested.resource_id)
                .and_then(|content| {
                    content.record_for(nested.document_image_container_key.as_ref()?)
                })
                .and_then(|record| {
                    record
                        .as_any()
                        .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
                });
            let replayed_document = match nested.r#type {
                Kind::kDrawImageRect => replay_document_image(canvas, nested, nested_record, depth),
                Kind::kDrawTiledImage => {
                    replay_tiled_document_image(canvas, nested, nested_record, depth)
                }
                Kind::kDrawMask => {
                    replay_mask_layers(canvas, nested, &nested_resources, &nested_documents, depth)
                }
                _ => false,
            };
            if !replayed_document {
                let mut command = nested.to_skia();
                hydrate_bitmap_patterns(&mut command, artifact.resources.as_deref());
                hydrate_document_patterns(&mut command, nested, &nested_documents, depth);
                canvas.replay_item(&command, &nested_resources);
            }
        }
    };

    // PaintArtifact drawing operations are recorded in each PaintChunk's
    // property-tree coordinate space. Replaying only the flat PaintRecord
    // loses transforms, clips and effects such as an SVG <g opacity=...>.
    // Match PaintChunksToCcLayer: switch shared Effect/Clip scopes to their
    // lifecycle LCA, then apply the individual chunk Transform to its record.
    if artifact.display_items.is_empty() || artifact.chunks.is_empty() {
        replay_range(canvas, &artifact.items);
        return;
    }
    let mut masked_parents = Vec::new();
    for chunk in &artifact.chunks {
        let mut current = Some(chunk.properties.effect.clone());
        while let Some(effect) = current {
            current = effect.parent.clone();
            if effect.is_mask {
                if let Some(parent) = &effect.parent {
                    if !masked_parents
                        .iter()
                        .any(|old: &Arc<EffectPaintPropertyNode>| {
                            old.lifecycle.same_node(&parent.lifecycle)
                        })
                    {
                        masked_parents.push(parent.clone());
                    }
                }
            }
        }
    }
    let mut active_scopes: Vec<DocumentPropertyScope> = Vec::new();
    for chunk in &artifact.chunks {
        let begin = chunk.begin_index as usize;
        let end = chunk.end_index as usize;
        if begin > end || end > artifact.display_items.len() {
            continue;
        }
        let scopes = document_property_scopes(&chunk.properties, &masked_parents);
        let common = active_scopes
            .iter()
            .zip(&scopes)
            .take_while(|(active, wanted)| active.same_node(wanted))
            .count();
        while active_scopes.len() > common {
            let scope = active_scopes.pop().expect("active document scope");
            replay_document_scope_close(canvas, &scope, &empty_resources);
        }
        for scope in scopes.into_iter().skip(common) {
            match &scope {
                DocumentPropertyScope::Clip(clip) => {
                    replay_document_clip_open(canvas, clip, &empty_resources)
                }
                DocumentPropertyScope::Effect(effect, force_isolation) => {
                    replay_document_effect_open(canvas, effect, *force_isolation, &empty_resources)
                }
            }
            active_scopes.push(scope);
        }
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kSave,
                ..Default::default()
            }
            .to_skia(),
            &empty_resources,
        );
        if let Ok(matrix) = layer_tile::compositor_transform(&chunk.properties.transform, 1.0) {
            replay_concat(canvas, matrix, &empty_resources);
        }
        for record in &artifact.display_items[begin..end] {
            if record.record_begin <= record.record_end && record.record_end <= artifact.items.len()
            {
                // The effect scope already owns the complete parent DstIn.
                // Share LayerTile's formal mask boundary validation; ordinary
                // nested DstIn paint operations remain in non-mask records.
                let body = layer_tile::recording::mask_record_body(
                    &artifact.items[record.record_begin..record.record_end],
                    chunk.properties.effect.is_mask,
                ).expect("validated document mask record");
                replay_range(canvas, body);
            }
        }
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kRestore,
                ..Default::default()
            }
            .to_skia(),
            &empty_resources,
        );
    }
    while let Some(scope) = active_scopes.pop() {
        replay_document_scope_close(canvas, &scope, &empty_resources);
    }
}
impl<'a> LayerReplay<'a> {
    fn document_record(
        &self,
        item: &DisplayItem,
    ) -> Option<&paint::paint_engine::DocumentPaintArtifactRecord> {
        self.documents
            .get(&item.resource_id)?
            .record_for(item.document_image_container_key.as_ref()?)?
            .as_any()
            .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
    }
    /// Borrow the exact compiled PaintRecord commands in this raster task's
    /// playback order. Missing layer/record ownership must reject analysis;
    /// callers must not filter errors into an apparently empty recording.
    pub(crate) fn solid_analysis_commands<'b>(
        &'b self,
        task: &'b RasterTask,
    ) -> Result<
        impl Iterator<Item = Result<&'b [DrawCommand], ReplayUnsupported>> + 'b,
        ReplayUnsupported,
    > {
        let layer = self
            .layers
            .get(&task.layer_id)
            .ok_or(ReplayUnsupported("unknown-solid-analysis-layer"))?;
        Ok(task.record_indices.iter().map(move |index| {
            layer
                .records
                .get(index)
                .map(|record| record.commands.as_slice())
                .ok_or(ReplayUnsupported("missing-solid-analysis-record"))
        }))
    }
    pub(crate) fn worker_layer(&self, task: &RasterTask) -> Option<Arc<PreparedLayer>> {
        let layer = self.layers.get(&task.layer_id)?;
        // Commands and resource payloads are immutable retained owners. Each
        // worker has its own mutable Canvas/image/clip caches.
        task.record_indices
            .iter()
            .all(|index| layer.records.contains_key(index))
            .then(|| layer.clone())
    }
    /// SOON raster owns a separate low-priority cache lane, so it may replay
    /// resource-bearing records without contending with the current frame's
    /// authoritative image cache. All command/resource payloads are retained
    /// by the prepared layer and PaintResources Arcs.
    pub(crate) fn prepaint_layer(&self, task: &RasterTask) -> Option<Arc<PreparedLayer>> {
        let layer = self.layers.get(&task.layer_id)?;
        task.record_indices
            .iter()
            .all(|index| layer.records.contains_key(index))
            .then(|| layer.clone())
    }
    pub(crate) fn for_worker(
        resources: ResourceContext<'a>,
        documents: BTreeMap<u64, Arc<image_resource::DocumentPaintContent>>,
        layers: BTreeMap<LayerId, Arc<PreparedLayer>>,
        scratch: Vec<u8>,
    ) -> Self {
        Self {
            resources,
            documents,
            layers,
            scratch,
        }
    }
    pub fn new(plan: &'a FramePlan, tasks: &[RasterTask]) -> Result<Self, ReplayUnsupported> {
        let list = plan.GetPaintArtifact();
        let documents = document_records(list);
        // Top-level clip masks belong to Renderer. Only nested document
        // artifacts use this raster-local property replay, so validate those
        // at admission, before any tile or worker can render partial content.
        validate_document_images(list, 0)?;
        let content = plan.GetRasterRecords();
        if plan.unsupported.is_some() {
            return Err(ReplayUnsupported("unsupported-layer-plan"));
        }
        let scale = plan.config.raster_scale;
        let mut layers = BTreeMap::new();
        let needed: BTreeSet<_> = tasks
            .iter()
            .flat_map(|task| {
                task.record_indices
                    .iter()
                    .map(move |&index| (task.layer_id, index))
            })
            .collect();
        let needed_layers: BTreeSet<_> = needed.iter().map(|&(layer, _)| layer).collect();
        let mut image_is_opaque = BTreeMap::new();
        for layer in &plan.layers {
            let composition = RasterLayerState {
                white_backing: layer.is_first_layer && !layer.requires_transparent_backing,
            };
            let mut records = BTreeMap::new();
            let mut opaque_rects = Vec::new();
            for &index in layer
                .record_indices
                .iter()
                .filter(|_| needed_layers.contains(&layer.id))
            {
                let source = content
                    .get(index)
                    .filter(|r| r.record_index == index)
                    .ok_or(ReplayUnsupported("missing-canonical-record"))?;
                let mut clip = None;
                let mut complex_clip = false;
                let mut record_transform = State::default();
                let mut saved = Vec::new();
                let compile = needed.contains(&(layer.id, index));
                let mut shadow_regions = Vec::new();
                // PictureLayerTiling's overlapping border texel is the normal
                // AA/effect guard. Do not wrap every tile in another private
                // surface gutter; that changes Skia's device-coordinate seeds.
                let raster_guard = 0;
                let mut commands = if compile {
                    Vec::with_capacity(source.items.len() + 2)
                } else {
                    Vec::new()
                };
                let mut draw_support = if compile {
                    Vec::with_capacity(source.items.len() + 2)
                } else {
                    Vec::new()
                };
                let mut source_items = if compile {
                    Vec::with_capacity(source.items.len() + 2)
                } else {
                    Vec::new()
                };
                let mut decoration_glyph_runs = if compile {
                    Vec::with_capacity(source.items.len() + 2)
                } else {
                    Vec::new()
                };
                if compile {
                    commands.push(
                        DisplayItem {
                            r#type: Kind::kSave,
                            ..Default::default()
                        }
                        .to_skia(),
                    );
                    draw_support.push(None);
                    source_items.push(None);
                    decoration_glyph_runs.push(Vec::new());
                }
                for item in &*source.items {
                    match item.r#type {
                        Kind::kSave | Kind::kSaveLayerBlend | Kind::kSaveLayerDstIn => {
                            saved.push((clip, complex_clip, record_transform))
                        }
                        Kind::kRestore => {
                            (clip, complex_clip, record_transform) = saved
                                .pop()
                                .ok_or(ReplayUnsupported("unbalanced-canonical-record"))?;
                        }
                        Kind::kConcat => {
                            concatenate_affine(&mut record_transform, &item.transform)?
                        }
                        Kind::kClipRect => {
                            let rect = mapped_rect(item.rect, record_transform);
                            if record_transform.cross_axis == (0.0, 0.0)
                                && aligned_rect(rect, scale)
                            {
                                clip = intersect(clip, rect);
                            } else {
                                complex_clip = true;
                            }
                        }
                        Kind::kClipRoundedRect => {
                            clip = intersect(clip, mapped_rect(item.rect, record_transform));
                            complex_clip = true;
                        }
                        Kind::kClipOutRoundedRect | Kind::kClipOutRect | Kind::kClipPath => {
                            complex_clip = true
                        }
                        _ => {
                            let rect = mapped_rect(item.rect, record_transform);
                            // Identity/translation prefixes do not discard a
                            // real opaque backdrop. Prove coverage in the same
                            // ancestor-layer coordinates as glyph ink bounds.
                            let unit_scale = record_transform.axis_scale == (1.0, 1.0)
                                && record_transform.cross_axis == (0.0, 0.0);
                            let direct_opaque_rect = unit_scale
                                && !complex_clip
                                && is_rect_draw(item.r#type)
                                && item.color.alpha == 1.0
                                && item.opacity == 1.0
                                && aligned_rect(rect, scale);
                            let direct_opaque_image = unit_scale
                                && !complex_clip
                                && item.r#type == Kind::kDrawImageRect
                                && item.opacity == 1.0
                                && aligned_rect(rect, scale)
                                && *image_is_opaque.entry(item.resource_id).or_insert_with(|| {
                                    list.resources
                                        .as_ref()
                                        .and_then(|r| {
                                            r.images.iter().find(|r| r.id == item.resource_id)
                                        })
                                        .and_then(|image| image.BitmapPixels())
                                        .is_some_and(|pixels| {
                                            pixels.chunks_exact(4).all(|pixel| pixel[3] == 255)
                                        })
                                });
                            if direct_opaque_rect || direct_opaque_image {
                                let rect = clip.map_or(rect, |c| intersection(c, rect));
                                opaque_rects.push(rect);
                            }
                        }
                    }
                    if compile {
                        // The planner selects whole semantic PaintRecords. A
                        // record may contain distant draws; use the existing
                        // complete support proof to avoid rasterizing a draw
                        // outside this tile. Unknown ink and all state ops stay.
                        let support = if is_draw(item.r#type) {
                            record_pixel_bounds(
                                std::slice::from_ref(item),
                                record_transform,
                                clip,
                                scale,
                                false,
                            )
                            .map(|mut rect| {
                                // Cover native point narrowing and AA at
                                // draw/clip boundaries without weakening
                                // the record's independent bounds contract.
                                let fringe = 1.0 / scale;
                                rect.x -= fringe;
                                rect.y -= fringe;
                                rect.width += 2.0 * fringe;
                                rect.height += 2.0 * fringe;
                                rect
                            })
                        } else {
                            None
                        };
                        draw_support.push(support);
                        if item.r#type == Kind::kDrawBoxShadow && item.blur_radius > 0.0 {
                            shadow_regions.push(shadow_support(item, record_transform, scale));
                        }
                        let mut command = item.to_skia();
                        hydrate_bitmap_patterns(&mut command, list.resources.as_deref());
                        // Keep the CSS/platform FontSmoothingMode attached to
                        // the run. Chromium lets Skia downgrade LCD coverage
                        // when the raster target has unknown pixel geometry;
                        // it does not rewrite the font description to
                        // `-webkit-font-smoothing: antialiased`, which would
                        // also disable CoreText hinting on macOS.
                        commands.push(command);
                        source_items.push(Some(item.clone()));
                        let decoration_runs = if item.r#type == Kind::kStrokeLine
                            && item.is_text_decoration
                            && item.skip_ink
                        {
                            source
                                .items
                                .iter()
                                .filter(|glyph| {
                                    glyph.r#type == Kind::kDrawGlyphRun
                                        && glyph.fragment_instance_id == item.fragment_instance_id
                                        && !glyph.is_shadow
                                        && !glyph.stroke_glyphs
                                })
                                .map(ToSkia::to_skia)
                                .collect()
                        } else {
                            Vec::new()
                        };
                        decoration_glyph_runs.push(decoration_runs);
                    }
                }
                if !saved.is_empty() {
                    return Err(ReplayUnsupported("unbalanced-canonical-record"));
                }
                if compile {
                    commands.push(
                        DisplayItem {
                            r#type: Kind::kRestore,
                            ..Default::default()
                        }
                        .to_skia(),
                    );
                    draw_support.push(None);
                    source_items.push(None);
                    decoration_glyph_runs.push(Vec::new());
                    records.insert(
                        index,
                        PreparedRecord {
                            commands,
                            source_items,
                            decoration_glyph_runs,
                            draw_support,
                            raster_guard,
                            shadow_regions,
                        },
                    );
                }
            }
            layers.insert(
                layer.id,
                Arc::new(PreparedLayer {
                    composition,
                    records,
                }),
            );
        }
        let resources = if tasks.is_empty() {
            ResourceContext::default()
        } else {
            resources(list)
        };
        // SkCanvas construction otherwise repeats this immutable catalog
        // validation (and HashSet allocation) for every tile. Canvas reads
        // image resources at replay_item, not at construction; validate once
        // here and keep passing the complete catalog to every actual draw.
        if let Some(catalog) = &resources.resources {
            let mut ids = std::collections::HashSet::with_capacity(catalog.images.len());
            for image in &catalog.images {
                let length = (image.width as usize)
                    .checked_mul(image.height as usize)
                    .and_then(|pixels| pixels.checked_mul(4));
                if image.id == 0
                    || image.width == 0
                    || image.height == 0
                    || length != Some(image.rgba8.len())
                    || !ids.insert(image.id)
                {
                    return Err(ReplayUnsupported("invalid-replay-image-catalog"));
                }
            }
        }
        Ok(Self {
            resources,
            documents,
            layers,
            scratch: Vec::new(),
        })
    }
    pub fn composition(&self, id: LayerId) -> Option<RasterLayerState> {
        self.layers.get(&id).map(|l| l.composition)
    }
    pub fn set_scratch(&mut self, scratch: Vec<u8>) {
        self.scratch = scratch;
    }
    pub fn take_scratch(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.scratch)
    }
    pub fn raster_tile_with_pixels(
        &mut self,
        task: &RasterTask,
        image_cache: &mut RasterImageCache,
        clip_cache: &mut RasterClipProductCache,
        mut reuse: Vec<u8>,
    ) -> std::io::Result<Vec<u8>> {
        let profile = std::env::var_os("BROWSER_PROFILE_TILES")
            .is_some()
            .then(std::time::Instant::now);
        let mut costs = [(None, 0usize, std::time::Duration::ZERO); 256];
        let image_before = profile.map(|_| image_cache.stats());
        let bad = |s| std::io::Error::other(s);
        let layer = self
            .layers
            .get(&task.layer_id)
            .ok_or_else(|| bad("unknown raster layer"))?;
        let (width, height) = task.pixel_size;
        let (raster_width, raster_height) = task.raster_pixel_size;
        if width == 0
            || height == 0
            || raster_width == 0
            || raster_height == 0
            || !aligned_rect(task.tile_rect, task.raster_scale)
            || !aligned_rect(task.raster_rect, task.raster_scale)
            || (task.tile_rect.width * task.raster_scale) != width as f64
            || (task.tile_rect.height * task.raster_scale) != height as f64
            || (task.raster_rect.width * task.raster_scale) != raster_width as f64
            || (task.raster_rect.height * task.raster_scale) != raster_height as f64
            || task.content_offset.0 + width > raster_width
            || task.content_offset.1 + height > raster_height
        {
            return Err(bad("invalid tile geometry"));
        }
        // Query only this task's semantic records. Skia's shadow receiver
        // supplies full blur-kernel padding itself; enlarge the destination
        // guard for a relevant shadow fringe/RasterEffectOutset, never by the
        // largest shadow anywhere on the page.
        // PictureLayer border texels are part of `raster_rect`; ordinary
        // antialiasing therefore needs no second, private tile gutter.
        let mut halo = 0u32;
        for index in &task.record_indices {
            let record = layer
                .records
                .get(index)
                .ok_or_else(|| bad("task record does not belong to layer"))?;
            halo = halo.max(record.raster_guard);
            if record
                .shadow_regions
                .iter()
                .any(|region| !intersection(*region, task.tile_rect).is_empty())
            {
                halo = halo.max(2);
            }
        }
        let target_width = raster_width
            .checked_add(halo * 2)
            .ok_or_else(|| bad("tile width overflow"))?;
        let target_height = raster_height
            .checked_add(halo * 2)
            .ok_or_else(|| bad("tile height overflow"))?;
        let length = (target_width as usize)
            .checked_mul(target_height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(|| bad("tile bytes overflow"))?;
        self.scratch.resize(length, 0);
        let storage = skia::PixelStorage::owned(std::mem::take(&mut self.scratch));
        let validated_catalog = ResourceContext::default();
        let mut canvas = if layer.composition.white_backing {
            skia::cpu::canvas::Canvas::make_raster_direct_with_format(
                &validated_catalog,
                target_width,
                target_height,
                target_width as usize * 4,
                storage,
                skia::PixelFormat::Rgba8888,
            )
        } else {
            skia::cpu::canvas::Canvas::make_raster_direct_with_format_preserving(
                &validated_catalog,
                target_width,
                target_height,
                target_width as usize * 4,
                storage,
                skia::PixelFormat::Rgba8888,
                false,
            )
        }
        .ok_or_else(|| bad("tile raster allocation"))?;
        if !layer.composition.white_backing {
            canvas.clear_transparent();
        }
        // This is a real per-tile Skia device. Playback's CTM below maps page
        // coordinates into it; shader seeds and ordered dithering remain in
        // tile-local device coordinates, matching RasterSource::PlaybackToCanvas.
        canvas.install_image_cache(std::mem::take(image_cache));
        canvas.install_clip_product_cache(std::mem::take(clip_cache));
        let mut transform = DisplayItem {
            r#type: Kind::kConcat,
            ..Default::default()
        };
        transform.transform.values[0] = task.raster_scale;
        transform.transform.values[5] = task.raster_scale;
        transform.transform.values[12] = halo as f64 - task.raster_rect.x * task.raster_scale;
        transform.transform.values[13] = halo as f64 - task.raster_rect.y * task.raster_scale;
        canvas.replay_item(&transform.to_skia(), &self.resources);
        let initialized = profile.map(|start| start.elapsed());
        let gutter = halo as f64 / task.raster_scale;
        let target_rect = PaintRect {
            x: task.raster_rect.x - gutter,
            y: task.raster_rect.y - gutter,
            width: task.raster_rect.width + 2.0 * gutter,
            height: task.raster_rect.height + 2.0 * gutter,
        };
        let mut culled_draws = 0usize;
        for &record in &task.record_indices {
            let record = layer
                .records
                .get(&record)
                .ok_or_else(|| bad("task record does not belong to layer"))?;
            for (((command, support), source_item), decoration_glyph_runs) in record
                .commands
                .iter()
                .zip(&record.draw_support)
                .zip(&record.source_items)
                .zip(&record.decoration_glyph_runs)
            {
                if support.is_some_and(|rect| intersection(rect, target_rect).is_empty()) {
                    culled_draws += 1;
                    continue;
                }
                let started = profile.map(|_| std::time::Instant::now());
                let replayed_document = source_item.as_ref().is_some_and(|item| {
                    let record = self.document_record(item);
                    match item.r#type {
                        Kind::kDrawImageRect => replay_document_image(&mut canvas, item, record, 0),
                        Kind::kDrawTiledImage => {
                            replay_tiled_document_image(&mut canvas, item, record, 0)
                        }
                        Kind::kDrawMask => replay_mask_layers(
                            &mut canvas,
                            item,
                            &self.resources,
                            &self.documents,
                            0,
                        ),
                        _ => false,
                    }
                });
                if !replayed_document {
                    let mut hydrated;
                    let command = if let Some(source_item) = source_item.as_ref() {
                        hydrated = command.clone();
                        hydrate_document_patterns(&mut hydrated, source_item, &self.documents, 0);
                        &hydrated
                    } else {
                        command
                    };
                    if decoration_glyph_runs.is_empty() {
                        canvas.replay_item(command, &self.resources);
                    } else {
                        canvas.replay_text_decoration_with_skip_ink(
                            command,
                            decoration_glyph_runs,
                            &self.resources,
                        );
                    }
                }
                if let Some(started) = started {
                    let cost = &mut costs[command.r#type as usize];
                    cost.0 = Some(command.r#type);
                    cost.1 += 1;
                    cost.2 += started.elapsed();
                }
            }
        }
        *image_cache = canvas.take_image_cache();
        *clip_cache = canvas.take_clip_product_cache();
        self.scratch = canvas.finish_direct().into_vec();
        reuse.resize(width as usize * height as usize * 4, 0);
        for row in 0..height as usize {
            let start = ((row + halo as usize + task.content_offset.1 as usize)
                * target_width as usize
                + halo as usize
                + task.content_offset.0 as usize)
                * 4;
            let dest = row * width as usize * 4;
            reuse[dest..dest + width as usize * 4]
                .copy_from_slice(&self.scratch[start..start + width as usize * 4]);
        }
        if let Some(start) = profile {
            let mut kinds: Vec<_> = costs
                .into_iter()
                .filter_map(|(kind, count, time)| {
                    kind.map(|kind| (kind, count, time.as_secs_f64() * 1000.0))
                })
                .collect();
            kinds.sort_by(|a, b| b.2.total_cmp(&a.2));
            let before = image_before.unwrap();
            let after = image_cache.stats();
            let message=format!("tile-raster-profile layer={} grid={:?} previous={} rect={:?} records={} culled_draws={} halo={} init_ms={:.3} total_ms={:.3} image_convert={} image_reuse={} mip_build={} kinds={:?}",
                task.layer_id.0,task.tile_index,task.previous_tile_id.is_some(),task.tile_rect,
                task.record_indices.len(),culled_draws,halo,initialized.unwrap().as_secs_f64()*1000.0,
                start.elapsed().as_secs_f64()*1000.0,after.source_conversions-before.source_conversions,
                after.source_reuses-before.source_reuses,after.mip_builds-before.mip_builds,kinds);
            eprintln!("{message}");
        }
        Ok(reuse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_assembly::internal::layout_input::TransformMatrix;
    use layoutng_assembly::internal::layout_input_types::Color;
    use layoutng_assembly::internal::paint_input::{
        PaintBlendMode, PaintFilterOperation, PaintFilterType,
    };
    use paint::paint_engine::{
        PaintArtifact, PaintChunk, RecordedDisplayItem, RecordedDisplayItemKind,
    };
    use paint::paint_property_tree::{
        ClipPaintPropertyNode, EffectPaintPropertyNode, PaintPropertyNodeLifecycle,
        PropertyTreeState, TransformPaintPropertyNode,
    };

    fn document_clip_fixture(child_count: usize) -> PaintArtifact {
        use layoutng_assembly::internal::layout_input::{
            Offset, PaintClipPathChild, PaintPathCommand, PaintPathVerb,
        };
        let root = PropertyTreeState::default();
        let children = (0..child_count)
            .map(|index| {
                let x = index as f64 * 2.0;
                PaintClipPathChild {
                    commands: [
                        (PaintPathVerb::kMoveTo, x, 0.0),
                        (PaintPathVerb::kLineTo, x + 4.0, 0.0),
                        (PaintPathVerb::kLineTo, x + 4.0, 4.0),
                        (PaintPathVerb::kLineTo, x, 4.0),
                        (PaintPathVerb::kClose, 0.0, 0.0),
                    ]
                    .into_iter()
                    .map(|(verb, x, y)| PaintPathCommand {
                        verb,
                        point: Offset { x, y },
                        ..Default::default()
                    })
                    .collect(),
                    even_odd: true,
                }
            })
            .collect::<Vec<_>>();
        let clip = Arc::new(ClipPaintPropertyNode {
            lifecycle: PaintPropertyNodeLifecycle::default(),
            id: 1,
            parent: Some(root.clip.clone()),
            rect: Some(PaintRect {
                width: 8.0,
                height: 8.0,
                ..Default::default()
            }),
            clip_path: children
                .iter()
                .flat_map(|child| child.commands.clone())
                .collect(),
            clip_path_is_empty: children.is_empty(),
            clip_path_children: children,
            clip_path_even_odd: true,
            ..(*root.clip).clone()
        });
        let rect = PaintRect {
            width: 8.0,
            height: 8.0,
            ..Default::default()
        };
        PaintArtifact {
            items: vec![DisplayItem {
                r#type: Kind::kDrawRect,
                rect,
                color: Color {
                    red: 1.0,
                    alpha: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            }]
            .into(),
            display_items: vec![RecordedDisplayItem {
                kind: RecordedDisplayItemKind::Drawing,
                id: Default::default(),
                visual_rect: rect,
                visual_rect_is_accurate: true,
                draws_content: true,
                raster_effect_outset: paint::paint_engine::RasterEffectOutset::kNone,
                record_begin: 0,
                record_end: 1,
                scroll_translation: None,
            }],
            chunks: vec![PaintChunk {
                begin_index: 0,
                end_index: 1,
                bounds: rect,
                drawable_bounds: rect,
                properties: PropertyTreeState { clip, ..root },
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn embed_document_fixture(artifact: PaintArtifact) -> PaintArtifact {
        let key = image_resource::ContainerKey::new(8, 8, 1.0, 1.0);
        let record = paint::paint_engine::DocumentPaintArtifactRecord {
            artifact: Arc::new(artifact),
            intrinsic_size: image_resource::IntrinsicSize {
                width: 8,
                height: 8,
            },
            record_size: image_resource::IntrinsicSize {
                width: 8,
                height: 8,
            },
            container_key: key.clone(),
        };
        PaintArtifact {
            items: vec![DisplayItem {
                r#type: Kind::kDrawImageRect,
                resource_id: 1,
                document_image_container_key: Some(key.clone()),
                ..Default::default()
            }]
            .into(),
            resources: Some(Arc::new(layoutng_assembly::fragment_tree::PaintResources {
                images: vec![layoutng_assembly::internal::layout_input::PaintImage {
                    id: 1,
                    width: 8,
                    height: 8,
                    document_container_key: Some(key.clone()),
                    content: image_resource::PaintImageContent::Document(Arc::new(
                        image_resource::DocumentPaintContent::new(key, Arc::new(record)),
                    )),
                    ..Default::default()
                }],
                ..Default::default()
            })),
            ..Default::default()
        }
    }

    #[test]
    fn document_clip_admission_rejects_partial_graph_and_effect_clip_errors() {
        let mut artifact = document_clip_fixture(1);
        Arc::make_mut(&mut artifact.chunks[0].properties.clip).clip_path_error =
            Some("nested clip-path on clip content is unsupported".into());
        assert_eq!(
            validate_document_artifact(&artifact, 0),
            Err(ReplayUnsupported("unsupported-document-clip-path"))
        );
        let failed_clip = artifact.chunks[0].properties.clip.clone();
        artifact.chunks[0].properties.clip = PropertyTreeState::default().clip;
        Arc::make_mut(&mut artifact.chunks[0].properties.effect).output_clip = Some(failed_clip);
        assert_eq!(
            validate_document_artifact(&artifact, 0),
            Err(ReplayUnsupported("unsupported-document-clip-path"))
        );
        let nested = embed_document_fixture(embed_document_fixture(artifact));
        assert_eq!(
            validate_document_images(&nested, 0),
            Err(ReplayUnsupported("unsupported-document-clip-path"))
        );
    }

    #[test]
    fn document_clip_admission_rejects_evenodd_union_without_flattening() {
        let artifact = document_clip_fixture(2);
        assert_eq!(
            validate_document_artifact(&artifact, 0),
            Err(ReplayUnsupported("union-document-clip-requires-mask"))
        );
        // Admission must leave the separately filled children immutable.
        assert_eq!(
            artifact.chunks[0].properties.clip.clip_path_children.len(),
            2
        );
        assert_eq!(
            validate_document_images(&embed_document_fixture(artifact), 0),
            Err(ReplayUnsupported("union-document-clip-requires-mask"))
        );
    }

    #[test]
    fn document_clip_admission_preserves_empty_and_single_child_replay() {
        for child_count in [0, 1] {
            let artifact = document_clip_fixture(child_count);
            assert_eq!(validate_document_artifact(&artifact, 0), Ok(()));
            assert_eq!(
                validate_document_images(&embed_document_fixture(artifact.clone()), 0),
                Ok(())
            );
            let resources = ResourceContext::default();
            let mut canvas = skia::cpu::canvas::Canvas::new(&resources, 8, 8);
            canvas.clear_transparent();
            replay_document_artifact(&mut canvas, &artifact, 0);
            let pixels = canvas.finish_direct().into_vec();
            let pixel = |x: usize, y: usize| &pixels[(y * 8 + x) * 4..(y * 8 + x + 1) * 4];
            assert_eq!(
                pixel(2, 2),
                if child_count == 0 {
                    &[0, 0, 0, 0]
                } else {
                    &[255, 0, 0, 255]
                }
            );
            assert_eq!(pixel(6, 2), &[0, 0, 0, 0]);
        }
    }

    #[test]
    fn document_artifact_replays_chunk_transform() {
        let root = PropertyTreeState::default();
        let mut matrix = TransformMatrix::default();
        matrix.values[12] = 20.0;
        let transform = Arc::new(TransformPaintPropertyNode {
            lifecycle: PaintPropertyNodeLifecycle::default(),
            id: 1,
            parent: Some(root.transform.clone()),
            matrix,
            origin: [0.0; 3],
            flattens_inherited_transform: false,
            rendering_context_id: 0,
            backface_visibility: Default::default(),
            scroll: None,
            sticky_constraint: None,
            direct_compositing_reasons: Vec::new(),
        });
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 8.0,
            height: 8.0,
        };
        let artifact = PaintArtifact {
            items: vec![DisplayItem {
                r#type: Kind::kDrawRect,
                rect,
                color: Color {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
                antialias: false,
                ..Default::default()
            }]
            .into(),
            display_items: vec![RecordedDisplayItem {
                kind: RecordedDisplayItemKind::Drawing,
                id: Default::default(),
                visual_rect: rect,
                visual_rect_is_accurate: true,
                draws_content: true,
                raster_effect_outset: paint::paint_engine::RasterEffectOutset::kNone,
                record_begin: 0,
                record_end: 1,
                scroll_translation: None,
            }],
            chunks: vec![PaintChunk {
                begin_index: 0,
                end_index: 1,
                bounds: rect,
                drawable_bounds: rect,
                properties: PropertyTreeState {
                    transform,
                    clip: root.clip,
                    effect: root.effect,
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        let resources = ResourceContext::default();
        let mut canvas = skia::cpu::canvas::Canvas::new(&resources, 40, 16);
        replay_document_artifact(&mut canvas, &artifact, 0);
        let pixels = canvas.finish_direct().into_vec();
        let pixel = |x: usize, y: usize| &pixels[(y * 40 + x) * 4..(y * 40 + x + 1) * 4];
        assert_eq!(pixel(2, 2), &[255, 255, 255, 255]);
        assert_eq!(pixel(22, 2), &[255, 0, 0, 255]);
    }

    #[test]
    fn document_artifact_keeps_shared_opacity_around_sibling_clips() {
        let root = PropertyTreeState::default();
        let effect = Arc::new(EffectPaintPropertyNode {
            lifecycle: PaintPropertyNodeLifecycle::default(),
            id: 1,
            parent: Some(root.effect.clone()),
            opacity: 0.5,
            ..(*root.effect).clone()
        });
        let clip = |id, x| {
            Arc::new(ClipPaintPropertyNode {
                lifecycle: PaintPropertyNodeLifecycle::default(),
                id,
                parent: Some(root.clip.clone()),
                local_transform_space: root.transform.clone(),
                rect: Some(PaintRect {
                    x,
                    y: 0.0,
                    width: 6.0,
                    height: 8.0,
                }),
                ..(*root.clip).clone()
            })
        };
        let left_clip = clip(2, 0.0);
        let right_clip = clip(3, 4.0);
        let red = |rect| DisplayItem {
            r#type: Kind::kDrawRect,
            rect,
            color: Color {
                red: 1.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },
            antialias: false,
            ..Default::default()
        };
        let left_rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 8.0,
            height: 8.0,
        };
        let right_rect = PaintRect {
            x: 2.0,
            y: 0.0,
            width: 10.0,
            height: 8.0,
        };
        let record = |index, rect| RecordedDisplayItem {
            kind: RecordedDisplayItemKind::Drawing,
            id: Default::default(),
            visual_rect: rect,
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: paint::paint_engine::RasterEffectOutset::kNone,
            record_begin: index,
            record_end: index + 1,
            scroll_translation: None,
        };
        let artifact = PaintArtifact {
            items: vec![red(left_rect), red(right_rect)].into(),
            display_items: vec![record(0, left_rect), record(1, right_rect)],
            chunks: vec![
                PaintChunk {
                    begin_index: 0,
                    end_index: 1,
                    bounds: left_rect,
                    drawable_bounds: left_rect,
                    properties: PropertyTreeState {
                        transform: root.transform.clone(),
                        clip: left_clip,
                        effect: effect.clone(),
                    },
                    ..Default::default()
                },
                PaintChunk {
                    begin_index: 1,
                    end_index: 2,
                    bounds: right_rect,
                    drawable_bounds: right_rect,
                    properties: PropertyTreeState {
                        transform: root.transform,
                        clip: right_clip,
                        effect,
                    },
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let resources = ResourceContext::default();
        let mut canvas = skia::cpu::canvas::Canvas::new(&resources, 12, 8);
        replay_document_artifact(&mut canvas, &artifact, 0);
        let pixels = canvas.finish_direct().into_vec();
        let pixel = |x: usize, y: usize| &pixels[(y * 12 + x) * 4..(y * 12 + x + 1) * 4];
        // Both chunks overlap at x=4..6. Shared group opacity is restored once,
        // so overlap and non-overlap pixels are byte-identical.
        assert_eq!(pixel(2, 2), pixel(5, 2));
        assert_eq!(pixel(5, 2), pixel(8, 2));
        assert_eq!(pixel(11, 2), &[255, 255, 255, 255]);
    }

    #[test]
    fn document_artifact_nests_effect_surfaces() {
        let root = PropertyTreeState::default();
        let parent = Arc::new(EffectPaintPropertyNode {
            lifecycle: PaintPropertyNodeLifecycle::default(),
            id: 1,
            parent: Some(root.effect.clone()),
            opacity: 0.5,
            ..(*root.effect).clone()
        });
        let child = Arc::new(EffectPaintPropertyNode {
            lifecycle: PaintPropertyNodeLifecycle::default(),
            id: 2,
            parent: Some(parent),
            opacity: 0.5,
            ..(*root.effect).clone()
        });
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 4.0,
            height: 4.0,
        };
        let artifact = PaintArtifact {
            items: vec![DisplayItem {
                r#type: Kind::kDrawRect,
                rect,
                color: Color {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
                antialias: false,
                ..Default::default()
            }]
            .into(),
            display_items: vec![RecordedDisplayItem {
                kind: RecordedDisplayItemKind::Drawing,
                id: Default::default(),
                visual_rect: rect,
                visual_rect_is_accurate: true,
                draws_content: true,
                raster_effect_outset: paint::paint_engine::RasterEffectOutset::kNone,
                record_begin: 0,
                record_end: 1,
                scroll_translation: None,
            }],
            chunks: vec![PaintChunk {
                begin_index: 0,
                end_index: 1,
                bounds: rect,
                drawable_bounds: rect,
                properties: PropertyTreeState {
                    transform: root.transform,
                    clip: root.clip,
                    effect: child,
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        let resources = ResourceContext::default();
        let mut canvas = skia::cpu::canvas::Canvas::new(&resources, 4, 4);
        replay_document_artifact(&mut canvas, &artifact, 0);
        let pixels = canvas.finish_direct().into_vec();
        // Two nested 0.5 surfaces produce 0.25 red coverage over white.
        assert_eq!(&pixels[..4], &[255, 191, 191, 255]);
    }

    #[test]
    fn document_artifact_restores_non_normal_blend_equation() {
        let root = PropertyTreeState::default();
        let multiply = Arc::new(EffectPaintPropertyNode {
            lifecycle: PaintPropertyNodeLifecycle::default(),
            id: 1,
            parent: Some(root.effect.clone()),
            blend_mode: PaintBlendMode::kMultiply,
            isolates_blending: true,
            ..(*root.effect).clone()
        });
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 4.0,
            height: 4.0,
        };
        let draw = |color| DisplayItem {
            r#type: Kind::kDrawRect,
            rect,
            color,
            antialias: false,
            ..Default::default()
        };
        let record = |index| RecordedDisplayItem {
            kind: RecordedDisplayItemKind::Drawing,
            id: Default::default(),
            visual_rect: rect,
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: paint::paint_engine::RasterEffectOutset::kNone,
            record_begin: index,
            record_end: index + 1,
            scroll_translation: None,
        };
        let artifact = PaintArtifact {
            items: vec![
                draw(Color {
                    red: 0.0,
                    green: 0.0,
                    blue: 1.0,
                    alpha: 1.0,
                }),
                draw(Color {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                }),
            ]
            .into(),
            display_items: vec![record(0), record(1)],
            chunks: vec![
                PaintChunk {
                    begin_index: 0,
                    end_index: 1,
                    bounds: rect,
                    drawable_bounds: rect,
                    properties: root.clone(),
                    ..Default::default()
                },
                PaintChunk {
                    begin_index: 1,
                    end_index: 2,
                    bounds: rect,
                    drawable_bounds: rect,
                    properties: PropertyTreeState {
                        transform: root.transform,
                        clip: root.clip,
                        effect: multiply,
                    },
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let resources = ResourceContext::default();
        let mut canvas = skia::cpu::canvas::Canvas::new(&resources, 4, 4);
        replay_document_artifact(&mut canvas, &artifact, 0);
        let pixels = canvas.finish_direct().into_vec();
        // Red multiplied by the blue backdrop is black. Isolation-only would
        // incorrectly leave this pixel red.
        assert_eq!(&pixels[..4], &[0, 0, 0, 255]);
    }

    #[test]
    fn document_artifact_applies_dst_in_mask_to_complete_parent_group() {
        let root = PropertyTreeState::default();
        let content = Arc::new(EffectPaintPropertyNode {
            lifecycle: PaintPropertyNodeLifecycle::default(),
            id: 1,
            parent: Some(root.effect.clone()),
            ..(*root.effect).clone()
        });
        let mask = Arc::new(EffectPaintPropertyNode {
            lifecycle: PaintPropertyNodeLifecycle::default(),
            id: 2,
            parent: Some(content.clone()),
            is_mask: true,
            ..(*root.effect).clone()
        });
        let full = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 8.0,
            height: 4.0,
        };
        let left = PaintRect { width: 4.0, ..full };
        let draw = |rect, color| DisplayItem {
            r#type: Kind::kDrawRect,
            rect,
            color,
            antialias: false,
            ..Default::default()
        };
        let record = |index, rect| RecordedDisplayItem {
            kind: RecordedDisplayItemKind::Drawing,
            id: Default::default(),
            visual_rect: rect,
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: paint::paint_engine::RasterEffectOutset::kNone,
            record_begin: index,
            record_end: index + 1,
            scroll_translation: None,
        };
        let artifact = PaintArtifact {
            items: vec![
                draw(
                    full,
                    Color {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 1.0,
                    },
                ),
                DisplayItem {r#type: Kind::kSaveLayerDstIn, ..Default::default()},
                draw(
                    left,
                    Color {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                ),
                DisplayItem {r#type: Kind::kRestore, ..Default::default()},
            ]
            .into(),
            display_items: vec![record(0, full), RecordedDisplayItem {record_end:4, ..record(1,left)}],
            chunks: vec![
                PaintChunk {
                    begin_index: 0,
                    end_index: 1,
                    bounds: full,
                    drawable_bounds: full,
                    properties: PropertyTreeState {
                        transform: root.transform.clone(),
                        clip: root.clip.clone(),
                        effect: content,
                    },
                    ..Default::default()
                },
                PaintChunk {
                    begin_index: 1,
                    end_index: 2,
                    bounds: left,
                    drawable_bounds: left,
                    properties: PropertyTreeState {
                        transform: root.transform,
                        clip: root.clip,
                        effect: mask,
                    },
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert_eq!(validate_document_artifact(&artifact, 0), Ok(()));
        let resources = ResourceContext::default();
        let mut canvas = skia::cpu::canvas::Canvas::new(&resources, 8, 4);
        replay_document_artifact(&mut canvas, &artifact, 0);
        let pixels = canvas.finish_direct().into_vec();
        let pixel = |x: usize| &pixels[x * 4..(x + 1) * 4];
        assert_eq!(pixel(2), &[255, 0, 0, 255]);
        assert_eq!(pixel(6), &[255, 255, 255, 255]);
    }

    #[test]
    fn document_artifact_filters_shared_chunk_content_once() {
        let root = PropertyTreeState::default();
        let filter = Arc::new(EffectPaintPropertyNode {
            lifecycle: PaintPropertyNodeLifecycle::default(),
            id: 1,
            parent: Some(root.effect.clone()),
            filters: vec![PaintFilterOperation {
                r#type: PaintFilterType::kBlur,
                amount: 1.0,
                ..Default::default()
            }],
            ..(*root.effect).clone()
        });
        let left = PaintRect {
            x: 3.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        };
        let right = PaintRect {
            x: 6.0,
            width: 3.0,
            ..left
        };
        let union = PaintRect { width: 6.0, ..left };
        let draw = |rect| DisplayItem {
            r#type: Kind::kDrawRect,
            rect,
            color: Color {
                red: 1.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },
            antialias: false,
            ..Default::default()
        };
        let record = |index, rect| RecordedDisplayItem {
            kind: RecordedDisplayItemKind::Drawing,
            id: Default::default(),
            visual_rect: rect,
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: paint::paint_engine::RasterEffectOutset::kNone,
            record_begin: index,
            record_end: index + 1,
            scroll_translation: None,
        };
        let state = |effect| PropertyTreeState {
            transform: root.transform.clone(),
            clip: root.clip.clone(),
            effect,
        };
        let split = PaintArtifact {
            items: vec![draw(left), draw(right)].into(),
            display_items: vec![record(0, left), record(1, right)],
            chunks: vec![
                PaintChunk {
                    begin_index: 0,
                    end_index: 1,
                    bounds: left,
                    drawable_bounds: left,
                    properties: state(filter.clone()),
                    ..Default::default()
                },
                PaintChunk {
                    begin_index: 1,
                    end_index: 2,
                    bounds: right,
                    drawable_bounds: right,
                    properties: state(filter.clone()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let combined = PaintArtifact {
            items: vec![draw(union)].into(),
            display_items: vec![record(0, union)],
            chunks: vec![PaintChunk {
                begin_index: 0,
                end_index: 1,
                bounds: union,
                drawable_bounds: union,
                properties: state(filter),
                ..Default::default()
            }],
            ..Default::default()
        };
        let render = |artifact: &PaintArtifact| {
            let resources = ResourceContext::default();
            let mut canvas = skia::cpu::canvas::Canvas::new(&resources, 12, 8);
            replay_document_artifact(&mut canvas, artifact, 0);
            canvas.finish_direct().into_vec()
        };
        let split = render(&split);
        let combined = render(&combined);
        assert_eq!(split, combined);
        assert_ne!(
            &split[(4 * 12 + 2) * 4..(4 * 12 + 3) * 4],
            &[255, 255, 255, 255]
        );
    }
}
