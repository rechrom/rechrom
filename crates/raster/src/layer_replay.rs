//! CPU compilation and playback of the records carried by a FramePlan.
use crate::convert::{resources, ToSkia};
pub use layer_tile::recording::ReplayUnsupported;
use layer_tile::recording::{
    aligned_rect, concatenate_affine, intersect, intersection, is_draw, is_rect_draw, mapped_rect,
    record_pixel_bounds, State,
};
use layer_tile::{FramePlan, LayerId, RasterTask};
use paint::paint_engine::{DisplayItem, DisplayItemType as Kind, PaintRect};
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
    documents: BTreeMap<u64, Arc<dyn image_resource::DocumentPaintRecord>>,
    layers: BTreeMap<LayerId, Arc<PreparedLayer>>,
    scratch: Vec<u8>,
}
pub(crate) fn document_records(
    artifact: &paint::paint_engine::PaintArtifact,
) -> BTreeMap<u64, Arc<dyn image_resource::DocumentPaintRecord>> {
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

fn replay_document_artifact(
    canvas: &mut skia::cpu::canvas::Canvas,
    artifact: &paint::paint_engine::PaintArtifact,
    depth: usize,
) {
    let nested_resources = resources(artifact);
    let nested_documents = document_records(artifact);
    let empty_resources = ResourceContext::default();
    let replay_range = |canvas: &mut skia::cpu::canvas::Canvas,
                        range: std::ops::Range<usize>| {
        for nested in &artifact.items[range] {
            let nested_record = nested_documents
                .get(&nested.resource_id)
                .and_then(|record| {
                    record
                        .as_any()
                        .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
                });
            if nested.r#type != Kind::kDrawImageRect
                || !replay_document_image(canvas, nested, nested_record, depth)
            {
                canvas.replay_item(&nested.to_skia(), &nested_resources);
            }
        }
    };

    // PaintArtifact drawing operations are recorded in each PaintChunk's
    // property-tree coordinate space. Replaying only the flat PaintRecord
    // loses transforms such as an SVG <g transform=...>; Chromium applies the
    // PaintChunk properties before rasterizing the record as well.
    if artifact.display_items.is_empty() || artifact.chunks.is_empty() {
        replay_range(canvas, 0..artifact.items.len());
        return;
    }
    for chunk in &artifact.chunks {
        let begin = chunk.begin_index as usize;
        let end = chunk.end_index as usize;
        if begin > end || end > artifact.display_items.len() {
            continue;
        }
        canvas.replay_item(
            &DisplayItem {
                r#type: Kind::kSave,
                ..Default::default()
            }
            .to_skia(),
            &empty_resources,
        );
        let mut transforms = Vec::new();
        let mut current = Some(&chunk.properties.transform);
        while let Some(node) = current {
            transforms.push(node.matrix);
            current = node.parent.as_ref();
        }
        for matrix in transforms.into_iter().rev() {
            if matrix != Default::default() {
                canvas.replay_item(
                    &DisplayItem {
                        r#type: Kind::kConcat,
                        transform: matrix,
                        ..Default::default()
                    }
                    .to_skia(),
                    &empty_resources,
                );
            }
        }
        for record in &artifact.display_items[begin..end] {
            if record.record_begin <= record.record_end && record.record_end <= artifact.items.len()
            {
                replay_range(canvas, record.record_begin..record.record_end);
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
}
impl<'a> LayerReplay<'a> {
    fn document_record(
        &self,
        id: u64,
    ) -> Option<&paint::paint_engine::DocumentPaintArtifactRecord> {
        self.documents
            .get(&id)?
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
        documents: BTreeMap<u64, Arc<dyn image_resource::DocumentPaintRecord>>,
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
                let raster_guard = match list.display_items[index].raster_effect_outset {
                    paint::paint_engine::RasterEffectOutset::kWholePixel => 2,
                    _ => 1,
                };
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
        if width == 0
            || height == 0
            || !aligned_rect(task.tile_rect, task.raster_scale)
            || (task.tile_rect.width * task.raster_scale) != width as f64
            || (task.tile_rect.height * task.raster_scale) != height as f64
        {
            return Err(bad("invalid tile geometry"));
        }
        // Query only this task's semantic records. Skia's shadow receiver
        // supplies full blur-kernel padding itself; enlarge the destination
        // guard for a relevant shadow fringe/RasterEffectOutset, never by the
        // largest shadow anywhere on the page.
        let mut halo = 1u32;
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
        let target_width = width
            .checked_add(halo * 2)
            .ok_or_else(|| bad("tile width overflow"))?;
        let target_height = height
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
        // The gutter has a real layer-device origin. Gradient dithering and
        // raster pipeline vector groups use that origin independently of CTM.
        canvas.set_raster_origin((
            (task.tile_rect.x * task.raster_scale) as i32 - halo as i32,
            (task.tile_rect.y * task.raster_scale) as i32 - halo as i32,
        ));
        canvas.install_image_cache(std::mem::take(image_cache));
        canvas.install_clip_product_cache(std::mem::take(clip_cache));
        let mut transform = DisplayItem {
            r#type: Kind::kConcat,
            ..Default::default()
        };
        transform.transform.values[0] = task.raster_scale;
        transform.transform.values[5] = task.raster_scale;
        transform.transform.values[12] = halo as f64 - task.tile_rect.x * task.raster_scale;
        transform.transform.values[13] = halo as f64 - task.tile_rect.y * task.raster_scale;
        canvas.replay_item(&transform.to_skia(), &self.resources);
        let initialized = profile.map(|start| start.elapsed());
        let gutter = halo as f64 / task.raster_scale;
        let target_rect = PaintRect {
            x: task.tile_rect.x - gutter,
            y: task.tile_rect.y - gutter,
            width: task.tile_rect.width + 2.0 * gutter,
            height: task.tile_rect.height + 2.0 * gutter,
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
                    item.r#type == Kind::kDrawImageRect
                        && replay_document_image(
                            &mut canvas,
                            item,
                            self.document_record(item.resource_id),
                            0,
                        )
                });
                if !replayed_document {
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
            let start = ((row + halo as usize) * target_width as usize + halo as usize) * 4;
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
    use paint::paint_engine::{
        PaintArtifact, PaintChunk, RecordedDisplayItem, RecordedDisplayItemKind,
    };
    use paint::paint_property_tree::{
        PaintPropertyNodeLifecycle, PropertyTreeState, TransformPaintPropertyNode,
    };

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
            scroll: None,
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
}
