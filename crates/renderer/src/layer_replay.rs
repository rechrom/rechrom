//! CPU compilation and playback of the records carried by a FramePlan.
use crate::convert::{resources, ToSkia};
pub use layer_tile::recording::ReplayUnsupported;
use layer_tile::recording::{
    aligned_rect, concatenate_affine, finite_rect, intersect, intersection, is_draw, is_rect_draw,
    mapped_rect, record_pixel_bounds, State,
};
use layer_tile::{FramePlan, LayerId, RasterTask};
use layoutng_assembly::internal::layout_input::FontSmoothing;
use paint::paint_engine::{DisplayItem, DisplayItemType as Kind, PaintRect};
use paint::paint_property_tree::ClipPaintPropertyNode;
use skia::compat::commands::{DrawCommand, ResourceContext};
use skia::src::core::SkCanvas::RasterImageCache;
use skia::RasterClipProductCache;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerComposition {
    pub translation: (f64, f64),
    pub clip: Option<DeviceRect>,
    /// Output-quad limit from complete retained drawing support, in target
    /// device coordinates. This is separate from the external clip and leaves
    /// source tile coordinates/UVs intact. Unknown or white-backed support is
    /// unbounded here; enclosure follows the true scale and placement.
    pub content_bounds: Option<DeviceRect>,
    /// Destination cells whose complete bilinear sample footprint is opaque.
    pub opaque_bounds: Option<DeviceRect>,
    /// The first layer starts on the original full-canvas white backdrop.
    /// layer_tile invalidates tiles when this paint-order role changes.
    pub white_backing: bool,
    /// First allocated source column/row. A fractional leading filter fringe
    /// belongs only to this grid edge; interior seams keep one quad owner.
    pub grid_min: Option<(i32, i32)>,
}
struct PreparedRecord {
    commands: Vec<DrawCommand>,
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
    composition: LayerComposition,
    records: BTreeMap<usize, PreparedRecord>,
    retained_clips: Arc<[Arc<ClipPaintPropertyNode>]>,
}
pub struct LayerReplay<'a> {
    resources: ResourceContext<'a>,
    layers: BTreeMap<LayerId, Arc<PreparedLayer>>,
    scratch: Vec<u8>,
}
fn contains(a: PaintRect, b: PaintRect) -> bool {
    b.is_empty()
        || (a.x <= b.x
            && a.y <= b.y
            && a.x + a.width >= b.x + b.width
            && a.y + a.height >= b.y + b.height)
}
fn device_rect(rect: PaintRect, scale: f64) -> Result<DeviceRect, ReplayUnsupported> {
    if !finite_rect(rect) {
        return Err(ReplayUnsupported("invalid-composition-clip"));
    }
    // cc::ComputeLayerClipAndVisibleRect uses ToEnclosingClipRect for the
    // retained layer clip; SoftwareRenderer::SetClipRect applies that integer
    // scissor without AA. Rounded/path coverage remains a separate operation.
    // Descendant AA clips are already recorded in the layer's local raster
    // space by Prepare (PaintChunksToCcLayer::StartClip), not by this scissor.
    let left = (rect.x * scale).floor();
    let top = (rect.y * scale).floor();
    Ok(DeviceRect {
        x: left as i32,
        y: top as i32,
        width: ((rect.x + rect.width) * scale - left).ceil().max(0.0) as u32,
        height: ((rect.y + rect.height) * scale - top).ceil().max(0.0) as u32,
    })
}
fn content_device_rect(
    rect: PaintRect,
    scale: f64,
    translation: (f64, f64),
) -> Result<DeviceRect, ReplayUnsupported> {
    // CoverageIterator clips quad geometry to recorded content bounds while
    // retaining the tile's texture coordinates. Enclose only after applying
    // the true device placement. Complete source support already includes
    // the record's raster-effect outset; do not add destination padding.
    if rect.is_empty() {
        return Ok(DeviceRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        });
    }
    device_rect(
        PaintRect {
            x: rect.x * scale + translation.0,
            y: rect.y * scale + translation.1,
            width: rect.width * scale,
            height: rect.height * scale,
        },
        1.0,
    )
}
impl<'a> LayerReplay<'a> {
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
        layers: BTreeMap<LayerId, Arc<PreparedLayer>>,
        scratch: Vec<u8>,
    ) -> Self {
        Self {
            resources,
            layers,
            scratch,
        }
    }
    pub fn new(plan: &'a FramePlan) -> Result<Self, ReplayUnsupported> {
        let list = plan.GetPaintArtifact();
        let content = plan.GetRasterRecords();
        if plan.unsupported.is_some() {
            return Err(ReplayUnsupported("unsupported-layer-plan"));
        }
        let scale = plan.config.raster_scale;
        let mut layers = BTreeMap::new();
        let needed: BTreeSet<_> = plan
            .tasks
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
            let (root_translation, root_clip) =
                layer_tile::resolved_compositor_properties_with_scroll(
                    &layer.properties,
                    scale,
                    layer.compositor_scroll,
                )
                .map_err(|_| ReplayUnsupported("unsupported-compositor-properties"))?;
            if ![root_translation.0, root_translation.1]
                .iter()
                .all(|v| v.is_finite() && v.abs() <= (1 << 22) as f64)
            {
                return Err(ReplayUnsupported("large-property-placement"));
            }
            let translation = root_translation;
            let white_backing = layer.is_first_layer && !layer.requires_transparent_backing;
            let composition = LayerComposition {
                translation,
                clip: root_clip.map(|r| device_rect(r, 1.0)).transpose()?,
                content_bounds: if white_backing {
                    None
                } else {
                    layer
                        .raster_content_bounds
                        .map(|r| content_device_rect(r, scale, translation))
                        .transpose()?
                },
                opaque_bounds: sampled_opaque_device_rect(
                    layer.rect_known_to_be_opaque,
                    scale,
                    translation,
                )?,
                white_backing,
                grid_min: layer
                    .tiles
                    .iter()
                    .map(|tile| tile.tile_index)
                    .reduce(|a, b| (a.0.min(b.0), a.1.min(b.1))),
            };
            let mut retained_clips = Vec::new();
            let mut clip = Some(layer.properties.clip.clone());
            while let Some(node) = clip {
                clip = node.parent.clone();
                retained_clips.push(node);
            }
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
                if compile {
                    commands.push(
                        DisplayItem {
                            r#type: Kind::kSave,
                            ..Default::default()
                        }
                        .to_skia(),
                    );
                    draw_support.push(None);
                }
                for item in &*source.items {
                    match item.r#type {
                        Kind::kSave => saved.push((clip, complex_clip, record_transform)),
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
                                        .is_some_and(|image| {
                                            image.rgba8.chunks_exact(4).all(|pixel| pixel[3] == 255)
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
                        if item.r#type == Kind::kDrawGlyphRun
                            && !composition.white_backing
                            && item.font_smoothing != FontSmoothing::kNone
                        {
                            let support = clip.map_or(source.visual_rect, |c| {
                                intersection(c, source.visual_rect)
                            });
                            // SkCanvas.cpp::internalSaveLayer uses unknown
                            // pixel geometry unless preserve-LCD is requested.
                            // SkGlyphRunPainter then selects A8. Keep platform glyph
                            // masks; do not substitute outline rasterization.
                            if !source.bounds_are_complete
                                || !opaque_rects.iter().any(|&r| contains(r, support))
                            {
                                if std::env::var_os("LAYOUTNG_LAYER_REPLAY_VERBOSE").is_some() {
                                    eprintln!("layer-replay glyph-A8 layer={:?} record={index} complete={} visual={:?} support={support:?} opaque={opaque_rects:?} prefix={record_transform:?}",
                                        layer.id,source.bounds_are_complete,source.visual_rect);
                                }
                                command.font_smoothing =
                                    skia::compat::commands::FontSmoothing::kAntialiased;
                            }
                        }
                        commands.push(command);
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
                    records.insert(
                        index,
                        PreparedRecord {
                            commands,
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
                    retained_clips: Arc::from(retained_clips),
                }),
            );
        }
        let resources = if plan.tasks.is_empty() {
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
            layers,
            scratch: Vec::new(),
        })
    }
    pub fn composition(&self, id: LayerId) -> Option<LayerComposition> {
        self.layers.get(&id).map(|l| l.composition)
    }
    /// Unbaked actual property nodes, leaf to root. Retained axis-aligned rects
    /// use the compositor scissor. Rounded/path nodes retain real coverage,
    /// excluding effect-output ancestors handled by the corresponding group.
    pub fn retained_clip_nodes(&self, id: LayerId) -> Arc<[Arc<ClipPaintPropertyNode>]> {
        self.layers
            .get(&id)
            .map_or_else(|| Arc::from([]), |layer| layer.retained_clips.clone())
    }
    /// SoftwareRenderer::SetClipRect uses a non-AA enclosing scissor, whereas
    /// SetClipRRect uses AA. This is external compositor ownership only;
    /// StartClip's recorded descendant rects still use AA=true in Prepare.
    pub fn retained_clip_antialias(&self, node: &Arc<ClipPaintPropertyNode>) -> bool {
        node.radii.HasRadius() || !node.clip_path.is_empty()
    }
    pub fn set_scratch(&mut self, scratch: Vec<u8>) {
        self.scratch = scratch;
    }
    pub fn take_scratch(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.scratch)
    }
    pub fn raster_tile(
        &mut self,
        task: &RasterTask,
        image_cache: &mut RasterImageCache,
        clip_cache: &mut RasterClipProductCache,
    ) -> std::io::Result<Vec<u8>> {
        self.raster_tile_with_pixels(task, image_cache, clip_cache, Vec::new())
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
            for (command, support) in record.commands.iter().zip(&record.draw_support) {
                if support.is_some_and(|rect| intersection(rect, target_rect).is_empty()) {
                    culled_draws += 1;
                    continue;
                }
                let started = profile.map(|_| std::time::Instant::now());
                canvas.replay_item(command, &self.resources);
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
mod affine_upcast_regression {
    use super::*;
    use layer_tile::recording::PrepareRasterContent;
    use layoutng_assembly::internal::layout_input::TransformMatrix;
    use layoutng_assembly::internal::layout_input_types::Color;
    use paint::paint_engine::PaintArtifact;
    use paint::paint_engine::{RasterEffectOutset, RecordedDisplayItemKind};
    use paint::paint_property_tree::PropertyTreeState;
    use paint::paint_property_tree::{ScrollPaintPropertyNode, TransformPaintPropertyNode};

    #[test]
    fn baidu_half_turn_omitted_transform_tiles_match_full_canvas() {
        half_turn_tiles_match_full_canvas(false);
    }

    #[test]
    fn half_turn_shared_affine_rounded_clip_tiles_match_full_canvas() {
        half_turn_tiles_match_full_canvas(true);
    }

    fn half_turn_tiles_match_full_canvas(with_clip: bool) {
        let root = PropertyTreeState::default();
        let scroll = Arc::new(ScrollPaintPropertyNode {
            lifecycle: Default::default(),
            id: 6,
            parent: root.transform.scroll.clone(),
            overflow_clip: None,
            container_rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 512.0,
                height: 512.0,
            },
            contents_rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 512.0,
                height: 1024.0,
            },
            user_scrollable_horizontal: false,
            user_scrollable_vertical: true,
        });
        let anchor = Arc::new(TransformPaintPropertyNode {
            lifecycle: Default::default(),
            id: 7,
            parent: Some(root.transform.clone()),
            matrix: TransformMatrix::default(),
            origin: [0.0; 3],
            scroll: Some(scroll),
            direct_compositing_reasons: Vec::new(),
        });
        // Exact matrix from the first real Baidu rejection, including sin(pi).
        let matrix = TransformMatrix {
            values: [
                -1.0,
                1.2246467991473532e-16,
                0.0,
                0.0,
                -1.2246467991473532e-16,
                -1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                328.0,
                411.0,
                0.0,
                1.0,
            ],
        };
        let transform = Arc::new(TransformPaintPropertyNode {
            lifecycle: Default::default(),
            id: 10,
            parent: Some(anchor.clone()),
            matrix,
            origin: [0.0; 3],
            scroll: None,
            direct_compositing_reasons: Vec::new(),
        });
        // Both clip dimensions exceed the 256px tile size, including when
        // the local tiling grid starts at this clip's own enclosed bounds.
        // Put tile seams through straight clip edges, not the middle of a
        // corner curve. Original Skia clips/subdivides curves before building
        // analytic edges, so a different raster clip may change corner AA.
        let clip_rect = PaintRect {
            x: 20.0,
            y: 25.0,
            width: 280.0,
            height: 280.0,
        };
        let clip = if with_clip {
            Arc::new(paint::paint_property_tree::ClipPaintPropertyNode {
                lifecycle: Default::default(),
                id: 11,
                parent: Some(root.clip.clone()),
                local_transform_space: transform.clone(),
                rect: Some(clip_rect),
                radii: layoutng_assembly::internal::paint_input::PaintCornerRadii {
                    top_left: layoutng_assembly::internal::paint_input::PaintCornerRadius {
                        x: 13.0,
                        y: 13.0,
                    },
                    top_right: layoutng_assembly::internal::paint_input::PaintCornerRadius {
                        x: 13.0,
                        y: 13.0,
                    },
                    bottom_right: layoutng_assembly::internal::paint_input::PaintCornerRadius {
                        x: 13.0,
                        y: 13.0,
                    },
                    bottom_left: layoutng_assembly::internal::paint_input::PaintCornerRadius {
                        x: 13.0,
                        y: 13.0,
                    },
                },
                clip_path: Vec::new(),
                clip_path_even_odd: false,
                pixel_moving_filter: None,
            })
        } else {
            root.clip.clone()
        };
        let properties = PropertyTreeState {
            transform,
            clip,
            ..root
        };
        let rect = PaintRect {
            x: 8.0,
            y: 18.0,
            width: 300.0,
            height: if with_clip { 300.0 } else { 280.0 },
        };
        let mut list = PaintArtifact {
            items: vec![
                DisplayItem {
                    r#type: Kind::kSave,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: Kind::kConcat,
                    transform: matrix,
                    ..Default::default()
                },
                DisplayItem {
                    r#type: Kind::kDrawRect,
                    rect,
                    antialias: true,
                    color: Color {
                        red: 0.2,
                        green: 0.4,
                        blue: 0.7,
                        alpha: 1.0,
                    },
                    ..Default::default()
                },
                DisplayItem {
                    r#type: Kind::kRestore,
                    ..Default::default()
                },
            ]
            .into(),
            display_items: vec![paint::paint_engine::RecordedDisplayItem {
                kind: RecordedDisplayItemKind::Drawing,
                id: Default::default(),
                visual_rect: rect,
                visual_rect_is_accurate: true,
                draws_content: true,
                raster_effect_outset: RasterEffectOutset::kNone,
                record_begin: 2,
                record_end: 3,
                scroll_translation: Some(anchor.clone()),
            }],
            chunks: vec![paint::paint_engine::PaintChunk {
                end_index: 1,
                bounds: rect,
                drawable_bounds: rect,
                properties,
                ..Default::default()
            }],
            ..Default::default()
        };
        if with_clip {
            std::sync::Arc::make_mut(&mut list.items).insert(
                2,
                DisplayItem {
                    r#type: Kind::kClipRoundedRect,
                    rect: clip_rect,
                    corner_radii: list.chunks[0].properties.clip.radii,
                    antialias: true,
                    ..Default::default()
                },
            );
            list.display_items[0].record_begin += 1;
            list.display_items[0].record_end += 1;
        }
        let lowered = layer_tile::raster_properties(&list.chunks[0].properties).unwrap();
        assert!(lowered.transform.lifecycle.same_node(&anchor.lifecycle));
        let canonical = PrepareRasterContent(&list, 1.0).unwrap();
        let prefix = canonical[0]
            .items
            .iter()
            .find(|item| item.r#type == Kind::kConcat)
            .unwrap();
        assert_eq!(prefix.transform.values[1], matrix.values[1] as f32 as f64);
        assert_ne!(prefix.transform.values[1], 0.0);
        if with_clip {
            let prefix_index = canonical[0]
                .items
                .iter()
                .position(|item| item.r#type == Kind::kConcat)
                .unwrap();
            let clip = &canonical[0].items[prefix_index + 1];
            assert_eq!(clip.r#type, Kind::kClipRoundedRect);
            assert_eq!(clip.rect, clip_rect);
            assert_eq!(clip.corner_radii, list.chunks[0].properties.clip.radii);
            assert!(clip.antialias);
        }
        let mut actual = vec![0u32; 512 * 512];
        let mut renderer = crate::layer_tile_renderer::LayerTileRenderer::default();
        let list = Arc::new(list);
        let mut engine = layer_tile::LayerTileEngine::default();
        engine.SetFrameConfig(layer_tile::FrameConfig {
            viewport: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 512.0,
                height: 512.0,
            },
            raster_scale: 1.0,
            activation_scroll: None,
            prepaint_scroll: None,
        });
        engine.Update(&list).unwrap();
        renderer
            .paint(
                engine.GetFramePlan().unwrap(),
                512,
                512,
                &mut actual,
                skia::PixelFormat::Bgra8888,
            )
            .unwrap();
        assert!(renderer.layer_tile_stats().raster_tasks > 1);
        let expected = crate::surface::RenderDisplayItemListIntoTarget(
            &list,
            512,
            512,
            1.0,
            skia::PixelStorage::owned(vec![0; 512 * 512 * 4]),
            skia::PixelFormat::Bgra8888,
        )
        .unwrap()
        .into_vec();
        if with_clip {
            let canonical_list = PaintArtifact {
                items: canonical[0].items.to_vec().into(),
                ..(*list).clone()
            };
            let canonical_pixels = crate::surface::RenderDisplayItemListIntoTarget(
                &canonical_list,
                512,
                512,
                1.0,
                skia::PixelStorage::owned(vec![0; 512 * 512 * 4]),
                skia::PixelFormat::Bgra8888,
            )
            .unwrap()
            .into_vec();
            for (index, (canonical, expected)) in canonical_pixels
                .chunks_exact(4)
                .zip(expected.chunks_exact(4))
                .enumerate()
            {
                assert_eq!(
                    canonical,
                    expected,
                    "canonical ClipOp full-canvas pixel {},{}",
                    index % 512,
                    index / 512
                );
            }
        }
        for (index, (actual, expected)) in actual.iter().zip(expected.chunks_exact(4)).enumerate() {
            assert_eq!(
                actual.to_ne_bytes().as_slice(),
                expected,
                "pixel {},{}",
                index % 512,
                index / 512
            );
        }
    }
}

// GeometryMapper maps an enclosed opacity proof, never its outer enclosure.
// This software sampler reads base + one pixel on each fractional axis; keep
// that full source footprint inside the proof before selecting an Src kernel.
pub(crate) fn sampled_opaque_device_rect(
    rect: PaintRect,
    scale: f64,
    translation: (f64, f64),
) -> Result<Option<DeviceRect>, ReplayUnsupported> {
    if rect.is_empty() {
        return Ok(None);
    }
    let tx = translation.0.ceil();
    let ty = translation.1.ceil();
    let x = (rect.x * scale).ceil() + tx;
    let y = (rect.y * scale).ceil() + ty;
    let right = ((rect.x + rect.width) * scale).floor() + tx - f64::from(tx != translation.0);
    let bottom = ((rect.y + rect.height) * scale).floor() + ty - f64::from(ty != translation.1);
    if ![x, y, right, bottom]
        .iter()
        .all(|v| v.is_finite() && v.abs() <= i32::MAX as f64)
    {
        return Err(ReplayUnsupported("invalid-opaque-device-bounds"));
    }
    Ok((right > x && bottom > y).then_some(DeviceRect {
        x: x as i32,
        y: y as i32,
        width: (right - x).max(0.0) as u32,
        height: (bottom - y).max(0.0) as u32,
    }))
}
