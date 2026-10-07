use crate::geometry::{finite, intersection, intersects, outset, pixel_aligned, shifted};
use crate::*;
use layoutng_assembly::fragment_tree::PaintResources;
use paint::display_item_id::{DisplayItemId, DisplayItemIdType};
use paint::paint_engine::{
    DisplayItem, DisplayItemType, PaintArtifact, PaintChunk, PaintRect, RasterEffectOutset,
};
use paint::paint_property_tree::{PropertyTreeState, TransformPaintPropertyNode};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::Arc;

// Matches cc::PictureLayerTilingSet's SOON border policy. Chromium expands
// the visible rect by 15% of its largest dimension, capped at 312 screen px,
// then TileManager schedules NOW before SOON. Our backend is synchronous, so
// admit a bounded SOON batch per otherwise idle frame. It runs on the
// renderer's separate prepaint lane and cannot consume the NOW workers.
const SOON_BORDER_VIEWPORT_PERCENTAGE: f64 = 0.15;
const MAX_SOON_BORDER_DEVICE_PIXELS: f64 = 312.0;
// The raster owner runs independently from display.  Keep enough SOON work in
// each publication to fill the retained border for a full desktop viewport;
// otherwise a page which becomes idle immediately after its first visible
// raster never gets another Update() in which to populate the skewport.
const MAX_SOON_RASTER_TASKS_PER_FRAME: usize = 128;

#[derive(Clone)]
struct RecordState {
    artifact_index: usize,
    id: DisplayItemId,
    rect: PaintRect,
    complete: bool,
    draws_content: bool,
    raster_effect_outset: RasterEffectOutset,
    items: Arc<[DisplayItem]>,
}

#[derive(Clone)]
struct TileState {
    id: TileId,
    previous_id: Option<TileId>,
    generation: u64,
    ready: bool,
    last_used: u64,
}

#[derive(Clone)]
struct TilingState {
    scale: f64,
    origin: (f64, f64),
    last_used: u64,
    tiles: BTreeMap<(i32, i32), TileState>,
}

#[derive(Clone)]
struct LayerState {
    id: LayerId,
    first_chunk: PaintChunk,
    raster_transform: Arc<TransformPaintPropertyNode>,
    singleton_chunk: bool,
    is_first_layer: bool,
    requires_transparent_backing: bool,
    records: Vec<RecordState>,
    tilings: Vec<TilingState>,
    dirty_diagnostics: u8,
}

struct PendingLayer {
    properties: PropertyTreeState,
    chunks: Vec<usize>,
    records: Vec<usize>,
    bounds: PaintRect,
    // Complete record ink plus native RasterEffectOutset, in the retained
    // transform's coordinates. Clips never enlarge this overlap proof.
    overlap_bounds: PaintRect,
    rect_known_to_be_opaque: PaintRect,
    complete: bool,
    singleton_chunk: bool,
}

/// Persistent CPU-agnostic metadata. Compatible chunks merge consecutively or
/// across proven disjoint layers in the same real transform/effect group.
/// General upcasts across compositing/effect boundaries remain unsupported.
#[derive(Clone)]
pub struct LayerTileManager {
    tile_size: u32,
    tile_budget: usize,
    frame_id: u64,
    next_layer_id: u64,
    next_tile_id: u64,
    layers: Vec<LayerState>,
    resources: Option<Arc<PaintResources>>,
    tile_lookup: HashMap<TileId, (LayerId, u64, (i32, i32))>,
}

impl Default for LayerTileManager {
    fn default() -> Self {
        Self::new(256)
    }
}

impl LayerTileManager {
    pub(crate) fn frame_id(&self) -> u64 {
        self.frame_id
    }

    /// Resource loss does not delete resident layer/tile objects. Re-request
    /// raster on their existing grid identities when they become visible.
    pub(crate) fn invalidate_resources(&mut self) {
        for layer in &mut self.layers {
            for tiling in &mut layer.tilings {
                for tile in tiling.tiles.values_mut() {
                    tile.ready = false;
                }
            }
        }
    }
    pub fn new(tile_size: u32) -> Self {
        assert!(tile_size > 0, "tile dimensions must be nonzero");
        Self {
            tile_size,
            tile_budget: 2048,
            frame_id: 0,
            next_layer_id: 1,
            next_tile_id: 1,
            layers: Vec::new(),
            resources: None,
            tile_lookup: HashMap::new(),
        }
    }

    /// Bounds all resident metadata and, through retired_tiles, backend pixels.
    /// A frame needing more visible tiles falls back rather than dropping tiles.
    pub fn set_tile_budget(&mut self, tiles: usize) {
        self.tile_budget = tiles.max(1);
    }

    /// Clear metadata after rejected replay or failed raster/composition. Keep
    /// allocation counters so asynchronous old tasks can never acknowledge a
    /// newly allocated tile with a recycled id.
    pub fn invalidate(&mut self) -> Vec<TileId> {
        let mut retired: Vec<_> = self.tile_lookup.keys().copied().collect();
        retired.sort_unstable();
        self.layers.clear();
        self.tile_lookup.clear();
        self.resources = None;
        retired
    }

    pub fn stats(&self) -> LayerTileStats {
        LayerTileStats {
            layer_count: self.layers.len(),
            tiling_count: self.layers.iter().map(|l| l.tilings.len()).sum(),
            tile_count: self.tile_lookup.len(),
            ready_tile_count: self
                .layers
                .iter()
                .flat_map(|l| &l.tilings)
                .flat_map(|t| t.tiles.values())
                .filter(|t| t.ready)
                .count(),
        }
    }

    /// TileManager-style task acknowledgement. Failed, superseded and retired
    /// task generations cannot make current tiles drawable.
    pub fn mark_rasterized(&mut self, id: TileId, generation: u64) -> bool {
        let Some((layer_id, scale, index)) = self.tile_lookup.get(&id).copied() else {
            return false;
        };
        let Some(layer) = self.layers.iter_mut().find(|l| l.id == layer_id) else {
            return false;
        };
        let Some(tiling) = layer
            .tilings
            .iter_mut()
            .find(|t| t.scale.to_bits() == scale)
        else {
            return false;
        };
        let Some(tile) = tiling.tiles.get_mut(&index) else {
            return false;
        };
        if tile.id != id || tile.generation != generation {
            return false;
        }
        tile.ready = true;
        true
    }

    pub fn is_rasterized(&self, id: TileId, generation: u64) -> bool {
        let Some((layer_id, scale, index)) = self.tile_lookup.get(&id).copied() else {
            return false;
        };
        self.layers
            .iter()
            .find(|layer| layer.id == layer_id)
            .and_then(|layer| {
                layer
                    .tilings
                    .iter()
                    .find(|tiling| tiling.scale.to_bits() == scale)
            })
            .and_then(|tiling| tiling.tiles.get(&index))
            .is_some_and(|tile| tile.id == id && tile.generation == generation && tile.ready)
    }

    /// For a backend pixel-cache eviction: request raster again without
    /// pretending the content changed or recycling an unrelated tile object.
    pub fn mark_tile_missing(&mut self, id: TileId) -> bool {
        let Some((layer_id, scale, index)) = self.tile_lookup.get(&id).copied() else {
            return false;
        };
        let Some(layer) = self.layers.iter_mut().find(|l| l.id == layer_id) else {
            return false;
        };
        let Some(tiling) = layer
            .tilings
            .iter_mut()
            .find(|t| t.scale.to_bits() == scale)
        else {
            return false;
        };
        let Some(tile) = tiling.tiles.get_mut(&index) else {
            return false;
        };
        tile.ready = false;
        true
    }

    /// Use backend-proven canonical local records for the legacy flat replay
    /// stream. Native chunk IDs and property identities still govern layer
    /// matching; canonical payload/bounds govern content invalidation/tiling.
    pub fn update_with_raster_content(
        &mut self,
        list: &Arc<PaintArtifact>,
        config: FrameConfig,
        content: &[RasterRecordContent],
    ) -> FramePlan {
        self.update_internal(list, config, Some(content))
    }

    fn update_internal(
        &mut self,
        list: &Arc<PaintArtifact>,
        config: FrameConfig,
        content: Option<&[RasterRecordContent]>,
    ) -> FramePlan {
        self.frame_id += 1;
        let mut plan = FramePlan {
            frame_id: self.frame_id,
            config,
            layers: Vec::new(),
            tasks: Vec::new(),
            retired_tiles: Vec::new(),
            unsupported: None,
            source: list.clone(),
            raster_records: Arc::from([]),
            resource_owner: None,
        };
        if !finite(config.viewport)
            || !config.raster_scale.is_finite()
            || config.raster_scale <= 0.0
        {
            plan.unsupported = Some(UnsupportedReason::InvalidFrameGeometry);
            return plan;
        }
        if let Err(reason) = validate(list, content) {
            plan.unsupported = Some(reason);
            return plan;
        }
        let canonical: Option<HashMap<usize, &RasterRecordContent>> =
            content.map(|records| records.iter().map(|r| (r.record_index, r)).collect());
        let pending = match group_chunks(list, canonical.as_ref(), config.raster_scale) {
            Ok(pending) => pending,
            Err(reason) => {
                plan.unsupported = Some(reason);
                return plan;
            }
        };
        // Picture-layer backing is determined by the complete effect topology,
        // not just the visible mask tiles. A transparent mask child must clear
        // its parent content even when its own visible source is empty.
        let mut mask_parents = HashSet::new();
        for chunk in &list.chunks {
            let mut effect = Some(&chunk.properties.effect);
            while let Some(node) = effect {
                if node.is_mask {
                    if let Some(parent) = &node.parent {
                        mask_parents.insert(Arc::as_ptr(&parent.lifecycle.identity) as usize);
                    }
                }
                effect = node.parent.as_ref();
            }
        }
        // Preflight before mutating live metadata. Unknown bounds cover the
        // visible local viewport, never an invented infinite layer rectangle.
        let mut old_matches: HashMap<DisplayItemId, Vec<usize>> = HashMap::new();
        for (index, layer) in self.layers.iter().enumerate() {
            old_matches
                .entry(layer.first_chunk.id)
                .or_default()
                .push(index);
        }
        let mut old_used = vec![false; self.layers.len()];
        let mut geometry = Vec::with_capacity(pending.len());
        let mut resolved_states = HashMap::new();
        let mut layer_tile_counts = Vec::with_capacity(pending.len());
        let mut visible_count = 0usize;
        let mut interest_count = 0usize;
        let mut activation_interest_count = 0usize;
        for layer in &pending {
            let first = &list.chunks[layer.chunks[0]];
            if (DisplayItemIdType::kForeignLayerFirst.0..=DisplayItemIdType::kForeignLayerLast.0)
                .contains(&first.id.r#type.0)
            {
                plan.unsupported = Some(UnsupportedReason::ForeignLayer);
                return plan;
            }
            let key = state_snapshot_key(&layer.properties);
            let (translation, root_clip) = if let Some(resolved) = resolved_states.get(&key) {
                *resolved
            } else {
                match resolved_compositor_properties(&layer.properties, config.raster_scale) {
                    Ok(g) => {
                        // Scheduling uses CSS coordinates; renderer keeps the
                        // exact snapped device transform through composition.
                        let g = (
                            (g.0 .0 / config.raster_scale, g.0 .1 / config.raster_scale),
                            g.1.map(|r| PaintRect {
                                x: r.x / config.raster_scale,
                                y: r.y / config.raster_scale,
                                width: r.width / config.raster_scale,
                                height: r.height / config.raster_scale,
                            }),
                        );
                        resolved_states.insert(key, g);
                        g
                    }
                    Err(reason) => {
                        plan.unsupported = Some(reason);
                        return plan;
                    }
                }
            };
            let matched = old_matches.get(&first.id).and_then(|indices| {
                indices.iter().copied().find(|&index| {
                    !old_used[index]
                        && self.layers[index].singleton_chunk == layer.singleton_chunk
                        && first.matches(&self.layers[index].first_chunk)
                })
            });
            if let Some(index) = matched {
                old_used[index] = true;
            }
            // Select the retained grid before deriving tile indices. A changed
            // support minimum is not a new raster space. cc stabilizes partial
            // recorded origins for the same reason (ComputeTilingRect).
            let origin = matched
                .and_then(|index| {
                    let old = &self.layers[index];
                    old.raster_transform
                        .lifecycle
                        .same_node(&layer.properties.transform.lifecycle)
                        .then(|| {
                            old.tilings
                                .iter()
                                .find(|tiling| tiling.scale == config.raster_scale)
                                .map(|tiling| tiling.origin)
                        })
                        .flatten()
                })
                .unwrap_or_else(|| {
                    raster_origin(layer.bounds, config.raster_scale, layer.complete)
                });
            if !finite(layer.bounds)
                || !pixel_aligned(origin.0, config.raster_scale)
                || !pixel_aligned(origin.1, config.raster_scale)
            {
                plan.unsupported = Some(UnsupportedReason::FractionalPixelPlacement);
                return plan;
            }
            let visible_root =
                root_clip.map_or(config.viewport, |clip| intersection(config.viewport, clip));
            let mut visible = shifted(visible_root, (-translation.0, -translation.1));
            // Fractional placement samples across tile edges. Allocate the
            // neighboring source pixel as well; composition still clips to
            // the original viewport/clip and never clamps an internal seam.
            if !visible.is_empty()
                && (!pixel_aligned(translation.0, config.raster_scale)
                    || !pixel_aligned(translation.1, config.raster_scale))
            {
                let footprint = 1.0 / config.raster_scale;
                visible = PaintRect {
                    x: visible.x - footprint,
                    y: visible.y - footprint,
                    width: visible.width + 2.0 * footprint,
                    height: visible.height + 2.0 * footprint,
                };
            }
            // State/hit-test-only chunks retain layer metadata but have no
            // raster pixels, even when their ink bounds are marked unknown.
            let raster_content_bounds = if !layer
                .records
                .iter()
                .any(|&index| list.display_items[index].draws_content)
            {
                visible = PaintRect::default();
                Some(PaintRect::default())
            } else if layer.complete {
                let mut raster_bounds = PaintRect::default();
                for &index in &layer.records {
                    let record = &list.display_items[index];
                    if record.draws_content {
                        let rect = canonical
                            .as_ref()
                            .and_then(|c| c.get(&index))
                            .map_or(record.visual_rect, |r| r.visual_rect);
                        raster_bounds.union(outset(rect, record.raster_effect_outset));
                    }
                }
                visible = intersection(visible, raster_bounds);
                Some(raster_bounds)
            } else {
                None
            };
            // cc::EffectTree::UpdateIsDrawn / draw_property_utils skips an
            // active-tree opacity-zero subtree before computing drawable tile
            // coverage. Our supported effects have no backdrop filters, copy
            // requests or capture surfaces. Retain layer metadata and dirty
            // resident resources below, but request no invisible raster work.
            // A Mask's own zero opacity is DstIn, not an ordinary hidden group:
            // its transparent source must still clear its content parent.
            if !ordinary_effect_is_drawn(&layer.properties) {
                visible = PaintRect::default();
            }
            let visible_grid =
                match grid_range(visible, origin, self.tile_size, config.raster_scale) {
                    Some(g) => g,
                    None => {
                        plan.unsupported = Some(UnsupportedReason::TileBudgetExceeded);
                        return plan;
                    }
                };
            let count = grid_tile_count(visible_grid);
            layer_tile_counts.push(count);
            visible_count = match count.and_then(|n| visible_count.checked_add(n)) {
                Some(n) if n <= self.tile_budget => n,
                _ => {
                    if std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some() {
                        eprintln!("[layer_tile visible budget] pending_layers={} prepared_layers={} previous_tiles={} next_tiles={:?} budget={}", pending.len(), layer_tile_counts.len(), visible_count, count, self.tile_budget);
                        for (index, (layer, tiles)) in
                            pending.iter().zip(&layer_tile_counts).enumerate()
                        {
                            eprintln!("[layer_tile budget layer] index={} tiles={:?} chunks={} records={} complete={} bounds={:?} transform={} clip={} effect={} singleton={}", index, tiles, layer.chunks.len(), layer.records.len(), layer.complete, layer.bounds, layer.properties.transform.id, layer.properties.clip.id, layer.properties.effect.id, layer.singleton_chunk);
                        }
                    }
                    plan.unsupported = Some(UnsupportedReason::TileBudgetExceeded);
                    return plan;
                }
            };
            // PictureLayerTilingSet creates tiles in an interest area larger
            // than the draw rect. Keep this geometry separate: only
            // visible_grid produces quads, while interest_grid may produce
            // low-priority resident raster resources.
            let mut interest =
                soon_interest_rect(visible, raster_content_bounds, config.raster_scale);
            // The pending viewport is required for activation. The farther
            // velocity prediction is only skewport prepaint: it must not delay
            // activation of the current scroll position.
            let mut activation_interest = visible;
            let mut activation_target_grid = visible_grid;
            if let Some(target) = config.activation_scroll.and_then(|scroll| {
                scroll_target_rect(
                    &layer.properties,
                    scroll,
                    config.viewport,
                    raster_content_bounds,
                    config.raster_scale,
                )
            }) {
                activation_interest.union(target);
                interest.union(target);
                activation_target_grid =
                    grid_range(target, origin, self.tile_size, config.raster_scale)
                        .unwrap_or(visible_grid);
            }
            let mut predicted_grid = visible_grid;
            // Chromium's skewport changes tile priority without changing the
            // active tree's draw transform. Do the same: union the current
            // visible rect with the impl-side scroll target only for interest
            // selection. `translation` and visible quads remain committed.
            if let Some(predicted) = config.prepaint_scroll.and_then(|scroll| {
                scroll_target_rect(
                    &layer.properties,
                    scroll,
                    config.viewport,
                    raster_content_bounds,
                    config.raster_scale,
                )
            }) {
                predicted_grid = grid_range(predicted, origin, self.tile_size, config.raster_scale)
                    .unwrap_or(visible_grid);
                interest.union(predicted);
            }
            let activation_grid = grid_range(
                activation_interest,
                origin,
                self.tile_size,
                config.raster_scale,
            )
            .unwrap_or(visible_grid);
            let interest_grid =
                match grid_range(interest, origin, self.tile_size, config.raster_scale) {
                    Some(g) => g,
                    None => visible_grid,
                };
            let candidate_interest_count = grid_tile_count(interest_grid);
            interest_count = candidate_interest_count
                .and_then(|n| interest_count.checked_add(n))
                .unwrap_or(usize::MAX);
            activation_interest_count = grid_tile_count(activation_grid)
                .and_then(|n| activation_interest_count.checked_add(n))
                .unwrap_or(usize::MAX);
            geometry.push((
                translation,
                root_clip,
                origin,
                visible_grid,
                interest_grid,
                activation_grid,
                activation_target_grid,
                predicted_grid,
                matched,
                raster_content_bounds,
            ));
        }
        // Required visible coverage owns the budget. Under pressure discard
        // the ordinary SOON border first, while preserving the compositor's
        // current activation viewport. The farther velocity prediction remains
        // ordinary background prepaint. Chromium's tile queues make the same
        // distinction between generic SOON tiles and tiles required for
        // pending-tree activation.
        if interest_count > self.tile_budget {
            for item in &mut geometry {
                item.4 = if activation_interest_count <= self.tile_budget {
                    item.5
                } else {
                    // A long scroll delta can make the bounding corridor from
                    // the committed viewport to the activation viewport exceed
                    // the budget even though the two endpoint grids fit. The
                    // visible loop below always retains the committed endpoint;
                    // keep the activation endpoint here and discard only the
                    // intervening corridor. Falling back to item.3 dropped the
                    // target entirely and caused a no-work activation retry
                    // loop while images kept publishing new pending trees.
                    item.6
                };
            }
        }
        let resources = ResourceComparison::new(
            self.resources.as_deref(),
            list.resources.as_deref(),
            &list.image_dependencies,
        );
        let old_layers = std::mem::take(&mut self.layers);
        let mut old_layers: Vec<Option<LayerState>> = old_layers.into_iter().map(Some).collect();
        let mut soon_tasks = Vec::new();
        for (
            layer_index,
            (
                pending,
                (
                    translation,
                    root_clip,
                    origin,
                    grid,
                    interest_grid,
                    activation_grid,
                    activation_target_grid,
                    predicted_grid,
                    matched,
                    raster_content_bounds,
                ),
            ),
        ) in pending.into_iter().zip(geometry).enumerate()
        {
            let first = &list.chunks[pending.chunks[0]];
            if matched.is_none()
                && grid.0 < grid.2
                && grid.1 < grid.3
                && old_layers.iter().any(Option::is_some)
                && std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some()
            {
                eprintln!("[layer_tile layer match miss] first={:?} cacheable={} just_created={} singleton={} raster_transform={} raster_clip={} raster_effect={} bounds={:?} old_firsts={:?}", first.id, first.is_cacheable, first.client_is_just_created, pending.singleton_chunk, pending.properties.transform.id, pending.properties.clip.id, pending.properties.effect.id, pending.bounds,
                    old_layers.iter().flatten().take(8).map(|old| (old.id.0, old.first_chunk.id, old.first_chunk.is_cacheable, old.first_chunk.client_is_just_created)).collect::<Vec<_>>());
            }
            let previous = matched.and_then(|i| old_layers[i].take());
            let new_layer = previous.is_none();
            let was_first = previous.as_ref().is_some_and(|p| p.is_first_layer);
            let is_first_layer = layer_index == 0;
            let mut effect = Some(&pending.properties.effect);
            let mut in_masked_group = false;
            while let Some(node) = effect {
                in_masked_group |=
                    mask_parents.contains(&(Arc::as_ptr(&node.lifecycle.identity) as usize));
                effect = node.parent.as_ref();
            }
            let requires_transparent_backing =
                in_masked_group || needs_transparent_backing(&pending.properties);
            let mut state = previous.unwrap_or_else(|| {
                let id = LayerId(self.next_layer_id);
                self.next_layer_id += 1;
                LayerState {
                    id,
                    first_chunk: first.clone(),
                    raster_transform: pending.properties.transform.clone(),
                    singleton_chunk: pending.singleton_chunk,
                    is_first_layer,
                    requires_transparent_backing,
                    records: Vec::new(),
                    tilings: Vec::new(),
                    dirty_diagnostics: 0,
                }
            });
            // No current visibility and no retained pixels means there is no
            // raster snapshot to compare. Keep native layer identity/ordering,
            // then build fresh records only when pixels are first requested.
            if (grid.0 == grid.2 || grid.1 == grid.3)
                && (interest_grid.0 == interest_grid.2 || interest_grid.1 == interest_grid.3)
            {
                if state.tilings.iter().all(|tiling| tiling.tiles.is_empty()) {
                    state.records.clear();
                    state.tilings.clear();
                    state.first_chunk = first.clone();
                    state.raster_transform = pending.properties.transform.clone();
                    state.singleton_chunk = pending.singleton_chunk;
                    state.is_first_layer = is_first_layer;
                    state.requires_transparent_backing = requires_transparent_backing;
                    plan.layers.push(LayerPlan {
                        id: state.id,
                        is_first_layer,
                        requires_transparent_backing,
                        chunk_indices: pending.chunks,
                        properties: pending.properties,
                        bounds: pending.bounds,
                        bounds_are_complete: pending.complete,
                        raster_content_bounds: if is_first_layer && !requires_transparent_backing {
                            None
                        } else {
                            raster_content_bounds
                        },
                        rect_known_to_be_opaque: pending.rect_known_to_be_opaque,
                        raster_origin: origin,
                        root_translation: translation,
                        compositor_scroll: None,
                        root_clip,
                        record_indices: pending.records,
                        interest_tiles: Vec::new(),
                        tiles: Vec::new(),
                    });
                    self.layers.push(state);
                    continue;
                }
            }
            // The first root layer may use an opaque page backing; inside an
            // opacity/rounded-clip group its raster must stay transparent. Crossing
            // that boundary changes pixels, whereas another non-unit alpha
            // value only changes composition of the retained effect surface.
            let backing_changed = is_first_layer
                && state.requires_transparent_backing != requires_transparent_backing;
            let full_invalidation = was_first != is_first_layer || backing_changed;
            let records = build_records(list, &pending.records, canonical.as_ref(), &state.records);
            if std::env::var_os("BROWSER_PROFILE_TILES").is_some() {
                let unknown = records
                    .iter()
                    .filter(|record| record.draws_content && !record.complete)
                    .count();
                let widened: Vec<_> = records
                    .iter()
                    .zip(&pending.records)
                    .filter_map(|(record, &index)| {
                        let source = &list.display_items[index];
                        let glyph = record
                            .items
                            .iter()
                            .any(|item| item.r#type == DisplayItemType::kDrawGlyphRun);
                        let scaled = record.items.iter().any(|item| {
                            item.r#type == DisplayItemType::kConcat
                                && (item.transform.values[0] != 1.0
                                    || item.transform.values[5] != 1.0)
                        });
                        (glyph
                            && !scaled
                            && source.visual_rect_is_accurate
                            && record.complete
                            && record.rect.width * record.rect.height
                                > source.visual_rect.width * source.visual_rect.height * 2.0)
                            .then_some((index, source.visual_rect, record.rect))
                    })
                    .collect();
                let message=format!("tile-record-profile layer={} records={} unknown={} widened_glyphs={} examples={:?}",
                    state.id.0,records.len(),unknown,widened.len(),&widened[..widened.len().min(8)]);
                eprintln!("{message}");
            }
            let visible_tiles = grid.0 < grid.2 && grid.1 < grid.3;
            let diagnostics = visible_tiles
                && state.dirty_diagnostics < 32
                && std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some();
            let raster_space_changed = !state
                .raster_transform
                .lifecycle
                .same_node(&pending.properties.transform.lifecycle);
            let origin_changed = raster_space_changed && !state.tilings.is_empty();
            let (all_dirty, damage, first_change) = record_damage(
                &state.records,
                &records,
                full_invalidation,
                &resources,
                diagnostics,
            );
            let side = self.tile_size as f64 / config.raster_scale;
            let visible_tile_rect = PaintRect {
                x: origin.0 + grid.0 as f64 * side,
                y: origin.1 + grid.1 as f64 * side,
                width: (grid.2 - grid.0) as f64 * side,
                height: (grid.3 - grid.1) as f64 * side,
            };
            if diagnostics
                && (all_dirty
                    || origin_changed
                    || damage
                        .iter()
                        .any(|rect| intersects(*rect, visible_tile_rect)))
            {
                state.dirty_diagnostics += 1;
                eprintln!("[layer_tile dirty] layer={} full={} new_layer={} origin={} first_role={} backing={} old_records={} new_records={} damage_rects={} first_change={}", state.id.0, all_dirty, new_layer, origin_changed, was_first != is_first_layer, backing_changed, state.records.len(), records.len(), damage.len(), first_change.as_deref().unwrap_or("forced-backing-first-role-or-origin"));
            }
            // Each scale keeps its own resident grid anchor. Bounds/culling
            // changes only change live tile ranges. A real raster-space owner
            // change makes the old tiling coordinates incompatible.
            if raster_space_changed {
                for tiling in state.tilings.drain(..) {
                    plan.retired_tiles
                        .extend(tiling.tiles.values().map(|t| t.id));
                }
            }
            // Property-only scroll updates have no content invalidation. Keep
            // resident tile IDs/readiness, including offscreen tiles, without
            // visiting every grid slot merely to test an empty damage list.
            if all_dirty || !damage.is_empty() {
                for tiling in &mut state.tilings {
                    let side = self.tile_size as f64 / tiling.scale;
                    for (&index, tile) in &mut tiling.tiles {
                        let rect = tile_rect(tiling.origin, index, side);
                        if all_dirty || damage.iter().any(|r| intersects(rect, *r)) {
                            let previous_id = tile.id;
                            plan.retired_tiles.push(previous_id);
                            tile.id = TileId(self.next_tile_id);
                            self.next_tile_id += 1;
                            tile.previous_id = Some(previous_id);
                            tile.generation += 1;
                            tile.ready = false;
                        }
                    }
                }
            }
            let tiling_index = state
                .tilings
                .iter()
                .position(|t| t.scale == config.raster_scale)
                .unwrap_or_else(|| {
                    state.tilings.push(TilingState {
                        scale: config.raster_scale,
                        origin,
                        last_used: self.frame_id,
                        tiles: BTreeMap::new(),
                    });
                    state.tilings.len() - 1
                });
            let tiling = &mut state.tilings[tiling_index];
            tiling.last_used = self.frame_id;
            let side = self.tile_size as f64 / config.raster_scale;
            let mut placements = Vec::new();
            // NOW: visible resources are required for this frame and always
            // precede prepaint work in the task stream.
            for j in grid.1..grid.3 {
                for i in grid.0..grid.2 {
                    let index = (i, j);
                    let rect = tile_rect(origin, index, side);
                    let tile = tiling.tiles.entry(index).or_insert_with(|| {
                        let id = TileId(self.next_tile_id);
                        self.next_tile_id += 1;
                        TileState {
                            id,
                            previous_id: None,
                            generation: 1,
                            ready: false,
                            last_used: self.frame_id,
                        }
                    });
                    tile.last_used = self.frame_id;
                    let normalized = shifted(rect, (-origin.0, -origin.1));
                    placements.push(TilePlacement {
                        tile_id: tile.id,
                        generation: tile.generation,
                        tile_index: index,
                        tiling_rect: normalized,
                        tile_rect: rect,
                        raster_scale: config.raster_scale,
                        pixel_size: (self.tile_size, self.tile_size),
                    });
                    if !tile.ready {
                        let selected = records
                            .iter()
                            .zip(&pending.records)
                            .filter_map(|(r, &index)| {
                                (r.draws_content
                                    && (!r.complete
                                        || intersects(
                                            rect,
                                            outset(r.rect, r.raster_effect_outset),
                                        )))
                                .then_some(index)
                            })
                            .collect();
                        plan.tasks.push(RasterTask {
                            tile_id: tile.id,
                            previous_tile_id: tile.previous_id,
                            layer_id: state.id,
                            generation: tile.generation,
                            tile_index: index,
                            tiling_rect: normalized,
                            tile_rect: rect,
                            raster_scale: config.raster_scale,
                            pixel_size: (self.tile_size, self.tile_size),
                            required_for_activation: false,
                            record_indices: selected,
                        });
                    }
                }
            }
            let mut interest_placements = placements.clone();
            // SOON: create stable resident tile identities around the visible
            // rect, but do not expose them as drawable quads. Raster a bounded
            // number after all layers' NOW tasks; unready remainder is offered
            // again on the next frame. This is the synchronous equivalent of
            // Chromium's NOW/SOON priority queue and avoids a tile-row burst.
            for j in interest_grid.1..interest_grid.3 {
                for i in interest_grid.0..interest_grid.2 {
                    if i >= grid.0 && i < grid.2 && j >= grid.1 && j < grid.3 {
                        continue;
                    }
                    let index = (i, j);
                    let rect = tile_rect(origin, index, side);
                    let tile = tiling.tiles.entry(index).or_insert_with(|| {
                        let id = TileId(self.next_tile_id);
                        self.next_tile_id += 1;
                        TileState {
                            id,
                            previous_id: None,
                            generation: 1,
                            ready: false,
                            last_used: self.frame_id,
                        }
                    });
                    tile.last_used = self.frame_id;
                    let normalized = shifted(rect, (-origin.0, -origin.1));
                    interest_placements.push(TilePlacement {
                        tile_id: tile.id,
                        generation: tile.generation,
                        tile_index: index,
                        tiling_rect: normalized,
                        tile_rect: rect,
                        raster_scale: config.raster_scale,
                        pixel_size: (self.tile_size, self.tile_size),
                    });
                    if !tile.ready {
                        let selected = records
                            .iter()
                            .zip(&pending.records)
                            .filter_map(|(r, &record_index)| {
                                (r.draws_content
                                    && (!r.complete
                                        || intersects(
                                            rect,
                                            outset(r.rect, r.raster_effect_outset),
                                        )))
                                .then_some(record_index)
                            })
                            .collect();
                        let required_for_activation = i >= activation_target_grid.0
                            && i < activation_target_grid.2
                            && j >= activation_target_grid.1
                            && j < activation_target_grid.3;
                        let predicted = i >= predicted_grid.0
                            && i < predicted_grid.2
                            && j >= predicted_grid.1
                            && j < predicted_grid.3;
                        let activation = i >= activation_grid.0
                            && i < activation_grid.2
                            && j >= activation_grid.1
                            && j < activation_grid.3;
                        soon_tasks.push((
                            if required_for_activation {
                                0u8
                            } else if predicted {
                                1u8
                            } else if activation {
                                2u8
                            } else {
                                3u8
                            },
                            rect_manhattan_distance(rect, visible_tile_rect) * config.raster_scale,
                            RasterTask {
                                tile_id: tile.id,
                                previous_tile_id: tile.previous_id,
                                layer_id: state.id,
                                generation: tile.generation,
                                tile_index: index,
                                tiling_rect: normalized,
                                tile_rect: rect,
                                raster_scale: config.raster_scale,
                                pixel_size: (self.tile_size, self.tile_size),
                                required_for_activation,
                                record_indices: selected,
                            },
                        ));
                    }
                }
            }
            state.records = records;
            state.first_chunk = first.clone();
            state.raster_transform = pending.properties.transform.clone();
            state.singleton_chunk = pending.singleton_chunk;
            state.is_first_layer = is_first_layer;
            state.requires_transparent_backing = requires_transparent_backing;
            // TilingSet keeps bounded scale alternatives; old-scale pixels can
            // survive a zoom round-trip without retaining unbounded history.
            while state.tilings.len() > 2 {
                let index = state
                    .tilings
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.scale != config.raster_scale)
                    .min_by_key(|(_, t)| t.last_used)
                    .unwrap()
                    .0;
                let removed = state.tilings.remove(index);
                plan.retired_tiles
                    .extend(removed.tiles.values().map(|t| t.id));
            }
            plan.layers.push(LayerPlan {
                id: state.id,
                is_first_layer,
                requires_transparent_backing,
                chunk_indices: pending.chunks,
                properties: pending.properties,
                bounds: pending.bounds,
                bounds_are_complete: pending.complete,
                raster_content_bounds: if is_first_layer && !requires_transparent_backing {
                    None
                } else {
                    raster_content_bounds
                },
                rect_known_to_be_opaque: pending.rect_known_to_be_opaque,
                raster_origin: origin,
                root_translation: translation,
                compositor_scroll: None,
                root_clip,
                record_indices: pending.records,
                interest_tiles: interest_placements,
                tiles: placements,
            });
            self.layers.push(state);
        }
        // NOW work stays first.  The raster owner is off the display path, so
        // it can continue with a bounded SOON tail while the active tree keeps
        // drawing.  Deferring this tail to a hypothetical later Update() left
        // stable pages with only their current visible row rasterized; the
        // first compositor scroll then exposed white before a new raster job
        // could finish.  Chromium likewise keeps scheduling lower-priority
        // raster work after required-for-draw tiles instead of requiring a new
        // paint commit to make that work exist.
        soon_tasks.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.total_cmp(&b.1)));
        // The cap applies only to speculative background work. Truncating the
        // required-for-activation prefix made a fast scroll need several
        // pending-tree retries before all target tiles existed, and at the top
        // boundary could leave the current target with no tile at all. cc's
        // required-for-activation queue is foreground work and is never
        // discarded by the SOON/prepaint quota.
        let mut background_tasks = 0usize;
        plan.tasks
            .extend(soon_tasks.into_iter().filter_map(|(_, _, task)| {
                if task.required_for_activation {
                    Some(task)
                } else if background_tasks < MAX_SOON_RASTER_TASKS_PER_FRAME {
                    background_tasks += 1;
                    Some(task)
                } else {
                    None
                }
            }));
        for old in old_layers.into_iter().flatten() {
            plan.retired_tiles.extend(
                old.tilings
                    .iter()
                    .flat_map(|t| t.tiles.values())
                    .map(|t| t.id),
            );
        }
        self.resources = list.resources.clone();
        // A FramePlan is an immutable raster/composition transaction.  Never
        // retire a tile that the transaction still names: the backend would
        // otherwise delete its pixels and then either flash an uncovered quad
        // or reject the raster acknowledgement for that same tile.  Chromium
        // expresses this through required-for-draw / required-for-activation
        // eviction priorities.  All tiles in this plan's interest set belong
        // to one of its current priority bins; older tiles outside that set
        // are the only eviction candidates for this update.
        let protected_tiles: HashSet<_> = plan
            .layers
            .iter()
            .flat_map(|layer| layer.interest_tiles.iter().map(|tile| tile.tile_id))
            .collect();
        self.prune(&mut plan.retired_tiles, &protected_tiles);
        debug_assert!(plan.layers.iter().all(|layer| layer
            .interest_tiles
            .iter()
            .all(|tile| !plan.retired_tiles.contains(&tile.tile_id))));
        self.rebuild_lookup();
        plan
    }

    fn prune(&mut self, retired: &mut Vec<TileId>, protected: &HashSet<TileId>) {
        let resident_count: usize = self
            .layers
            .iter()
            .flat_map(|layer| &layer.tilings)
            .map(|tiling| tiling.tiles.len())
            .sum();
        let remove = resident_count.saturating_sub(self.tile_budget);
        if remove == 0 {
            return;
        }
        let mut candidates: Vec<_> = self
            .layers
            .iter()
            .flat_map(|l| {
                l.tilings.iter().flat_map(move |t| {
                    t.tiles
                        .iter()
                        .filter(|(_, tile)| !protected.contains(&tile.id))
                        .map(move |(&index, tile)| (tile.last_used, l.id, t.scale.to_bits(), index))
                })
            })
            .collect();
        debug_assert!(candidates.len() >= remove);
        candidates.sort_unstable_by_key(|c| c.0);
        for (_, layer_id, scale, index) in candidates.into_iter().take(remove) {
            let layer = self.layers.iter_mut().find(|l| l.id == layer_id).unwrap();
            let tiling = layer
                .tilings
                .iter_mut()
                .find(|t| t.scale.to_bits() == scale)
                .unwrap();
            let tile = tiling.tiles.remove(&index).unwrap();
            retired.push(tile.id);
        }
    }

    fn rebuild_lookup(&mut self) {
        self.tile_lookup.clear();
        for layer in &self.layers {
            for tiling in &layer.tilings {
                for (&index, tile) in &tiling.tiles {
                    self.tile_lookup
                        .insert(tile.id, (layer.id, tiling.scale.to_bits(), index));
                }
            }
        }
    }
}

fn ordinary_effect_is_drawn(properties: &PropertyTreeState) -> bool {
    let mut effect = Some(properties.effect.as_ref());
    while let Some(node) = effect {
        if !node.is_mask && node.opacity == 0.0 {
            return false;
        }
        effect = node.parent.as_deref();
    }
    true
}

#[cfg(test)]
#[test]
fn opacity_zero_skips_raster_but_preserves_hidden_damage_and_mask_semantics() {
    use paint::paint_engine::RecordedDisplayItem;
    use paint::paint_engine::RecordedDisplayItemKind;
    use paint::paint_property_tree::PaintPropertyNodeLifecycle;
    let mut properties = PropertyTreeState::default();
    let mut effect = (*properties.effect).clone();
    effect.lifecycle = PaintPropertyNodeLifecycle::default();
    effect.id = 1;
    effect.parent = Some(properties.effect.clone());
    effect.opacity = 0.5;
    properties.effect = Arc::new(effect);
    let rect = PaintRect {
        x: 0.0,
        y: 0.0,
        width: 64.0,
        height: 64.0,
    };
    let id = DisplayItemId {
        client_id: 1,
        r#type: DisplayItemIdType::kBoxDecorationBackground,
        fragment: 0,
    };
    let list = Arc::new(PaintArtifact {
        items: vec![DisplayItem {
            r#type: DisplayItemType::kDrawRect,
            rect,
            ..Default::default()
        }]
        .into(),
        display_items: vec![RecordedDisplayItem {
            kind: RecordedDisplayItemKind::Drawing,
            id,
            visual_rect: rect,
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: RasterEffectOutset::kNone,
            record_begin: 0,
            record_end: 1,
            scroll_translation: None,
        }],
        chunks: vec![PaintChunk {
            id,
            is_cacheable: true,
            end_index: 1,
            bounds: rect,
            drawable_bounds: rect,
            properties,
            ..Default::default()
        }],
        ..Default::default()
    });
    let config = FrameConfig {
        viewport: rect,
        raster_scale: 1.0,
        activation_scroll: None,
        prepaint_scroll: None,
    };
    let mut manager = LayerTileManager::new(256);
    let first = manager.update_internal(&list, config, None);
    assert!(first.unsupported.is_none());
    assert_eq!(first.tasks.len(), 1);
    let old_tile = first.tasks[0].tile_id;
    assert!(manager.mark_rasterized(old_tile, first.tasks[0].generation));

    let mut hidden = (*list).clone();
    Arc::make_mut(&mut hidden.chunks[0].properties.effect).opacity = 0.0;
    Arc::make_mut(&mut hidden.items)[0].rect.width = 32.0;
    let hidden = Arc::new(hidden);
    let hidden_plan = manager.update_internal(&hidden, config, None);
    assert_eq!(
        hidden_plan.layers.len(),
        1,
        "retain the real effect/layer topology"
    );
    assert!(hidden_plan.layers[0].tiles.is_empty());
    assert!(
        hidden_plan.tasks.is_empty(),
        "never raster an undrawn opacity-zero layer"
    );
    assert!(
        hidden_plan.retired_tiles.contains(&old_tile),
        "hidden content still invalidates resident pixels"
    );

    let mut shown = (*hidden).clone();
    Arc::make_mut(&mut shown.chunks[0].properties.effect).opacity = 0.5;
    let shown_plan = manager.update_internal(&Arc::new(shown), config, None);
    assert_eq!(shown_plan.layers[0].id, first.layers[0].id);
    assert_eq!(
        shown_plan.tasks.len(),
        1,
        "newly exposed damaged pixels require raster"
    );
    assert_ne!(shown_plan.tasks[0].tile_id, old_tile);

    // A transparent Mask source still has DstIn semantics and must clear its
    // destination. ObjectPaintProperties::Mask itself has opacity one; a zero
    // opacity Mask node is rejected by the existing geometry admission guard.
    let mut mask = (*list).clone();
    let node = Arc::make_mut(&mut mask.chunks[0].properties.effect);
    node.is_mask = true;
    node.opacity = 1.0;
    let mut mask_manager = LayerTileManager::new(256);
    let mask_plan = mask_manager.update_internal(&Arc::new(mask), config, None);
    assert!(mask_plan.unsupported.is_none());
    assert_eq!(mask_plan.tasks.len(), 1);
    let mut descendant = (*hidden).clone();
    let mut child = (*descendant.chunks[0].properties.effect).clone();
    child.id = 2;
    child.lifecycle = PaintPropertyNodeLifecycle::default();
    child.opacity = 1.0;
    child.is_mask = true;
    child.parent = Some(descendant.chunks[0].properties.effect.clone());
    descendant.chunks[0].properties.effect = Arc::new(child);
    let descendant = manager.update_internal(&Arc::new(descendant), config, None);
    assert!(descendant.unsupported.is_none());
    assert_eq!(descendant.layers.len(), 1);
    assert!(descendant.layers[0].properties.effect.is_mask);
    assert!(
        descendant.tasks.is_empty(),
        "ordinary opacity-zero ancestors hide their mask subtree too"
    );
}

fn requires_own_chunk(chunk: &PaintChunk) -> bool {
    let item_type = chunk.id.r#type;
    (DisplayItemIdType::kForeignLayerFirst.0..=DisplayItemIdType::kForeignLayerLast.0)
        .contains(&item_type.0)
        || item_type == DisplayItemIdType::kScrollbarHorizontal
        || item_type == DisplayItemIdType::kScrollbarVertical
}

fn group_chunks(
    list: &PaintArtifact,
    canonical: Option<&HashMap<usize, &RasterRecordContent>>,
    raster_scale: f64,
) -> Result<Vec<PendingLayer>, UnsupportedReason> {
    let mut groups: Vec<PendingLayer> = Vec::new();
    let mut lowered_states = HashMap::new();
    for (index, chunk) in list.chunks.iter().enumerate() {
        let key = state_snapshot_key(&chunk.properties);
        let properties = if let Some(lowered) = lowered_states.get(&key) {
            PropertyTreeState::clone(lowered)
        } else {
            let lowered = raster_properties(&chunk.properties)?;
            lowered_states.insert(key, lowered.clone());
            lowered
        };
        // Native records in the omitted transform's coordinates require a
        // renderer-proven record conversion before bounds/pixels can be tiled.
        if canonical.is_none() && !properties.same_nodes(&chunk.properties) {
            return Err(UnsupportedReason::NonTranslationTransform);
        }
        // A direct-compositing anchor separates property groups, not every
        // chunk within that group. PendingLayer::CanUpcastWith accepts chunks
        // at the same real transform/effect; only foreign/scrollbar entities
        // remain singleton layers in this subset.
        let own = requires_own_chunk(chunk);
        let mut group = PendingLayer {
            properties,
            chunks: Vec::new(),
            records: Vec::new(),
            bounds: PaintRect::default(),
            overlap_bounds: PaintRect::default(),
            rect_known_to_be_opaque: PaintRect::default(),
            complete: true,
            singleton_chunk: own,
        };
        if let Some(canonical) = canonical {
            for record in chunk.begin_index as usize..chunk.end_index as usize {
                let content = canonical[&record];
                group.bounds.union(content.visual_rect);
                group.rect_known_to_be_opaque = PaintRect::maximum_covered_rect(
                    group.rect_known_to_be_opaque,
                    content.rect_known_to_be_opaque,
                );
                group.overlap_bounds.union(outset(
                    content.visual_rect,
                    list.display_items[record].raster_effect_outset,
                ));
                group.complete &= content.bounds_are_complete;
            }
        } else {
            group.bounds.union(chunk.bounds);
            group.rect_known_to_be_opaque = chunk.rect_known_to_be_opaque;
            group
                .overlap_bounds
                .union(outset(chunk.bounds, chunk.raster_effect_outset));
            group.complete &= chunk.bounds_are_complete;
        }
        // Keep the established adjacent merge path unchanged. Non-adjacent
        // reordering additionally requires backend-certified complete ink.
        if let Some(last) = groups.last_mut().filter(|last| {
            !own && !last.singleton_chunk && last.properties.same_nodes(&group.properties)
        }) {
            // Do not allocate temporary record/chunk vectors for the common
            // adjacent path: extend the existing backing directly.
            last.chunks.push(index);
            last.records
                .extend(chunk.begin_index as usize..chunk.end_index as usize);
            last.bounds.union(group.bounds);
            last.rect_known_to_be_opaque = PaintRect::maximum_covered_rect(
                last.rect_known_to_be_opaque,
                group.rect_known_to_be_opaque,
            );
            last.overlap_bounds.union(group.overlap_bounds);
            last.complete &= group.complete;
        } else {
            group.chunks.push(index);
            group
                .records
                .extend(chunk.begin_index as usize..chunk.end_index as usize);
            if canonical.is_some() {
                append_with_overlap_merge(&mut groups, group, raster_scale);
            } else {
                groups.push(group);
            }
        }
    }
    Ok(groups)
}

fn merge_pending_layer(home: &mut PendingLayer, guest: PendingLayer) {
    // Appending retains source record order even when intervening disjoint
    // layers are reordered. The home first chunk remains the native layer key.
    home.chunks.extend(guest.chunks);
    home.records.extend(guest.records);
    home.bounds.union(guest.bounds);
    home.rect_known_to_be_opaque = PaintRect::maximum_covered_rect(
        home.rect_known_to_be_opaque,
        guest.rect_known_to_be_opaque,
    );
    home.overlap_bounds.union(guest.overlap_bounds);
    home.complete &= guest.complete;
}

#[cfg(test)]
mod overlap_layerization_tests {
    use super::*;
    use paint::paint_property_tree::PaintPropertyNodeLifecycle;

    fn layer(index: usize, properties: PropertyTreeState, rect: PaintRect) -> PendingLayer {
        PendingLayer {
            properties,
            chunks: vec![index],
            records: vec![index],
            bounds: rect,
            overlap_bounds: rect,
            rect_known_to_be_opaque: PaintRect::default(),
            complete: true,
            singleton_chunk: false,
        }
    }

    #[test]
    fn nonconsecutive_merge_preserves_order_and_overlap_sparsity_barriers() {
        let home = PropertyTreeState::default();
        let mut intervening = home.clone();
        let mut clip = (*home.clip).clone();
        clip.lifecycle = PaintPropertyNodeLifecycle::default();
        intervening.clip = Arc::new(clip);
        let rect = |x, y| PaintRect {
            x,
            y,
            width: 10.0,
            height: 10.0,
        };

        let mut layers = vec![
            layer(0, home.clone(), rect(0.0, 0.0)),
            layer(1, intervening.clone(), rect(100.0, 0.0)),
        ];
        append_with_overlap_merge(&mut layers, layer(2, home.clone(), rect(20.0, 0.0)), 1.0);
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].chunks, [0, 2]);
        assert_eq!(layers[0].records, [0, 2]);
        assert_eq!(layers[1].records, [1]);

        let mut layers = vec![
            layer(0, home.clone(), rect(0.0, 0.0)),
            layer(1, intervening.clone(), rect(25.0, 0.0)),
        ];
        append_with_overlap_merge(&mut layers, layer(2, home.clone(), rect(20.0, 0.0)), 1.0);
        assert_eq!(
            layers.len(),
            3,
            "intervening overlap must preserve paint order"
        );

        // RasterEffectOutset is part of the overlap proof even if ordinary
        // allocation rectangles do not intersect.
        let mut middle = layer(1, intervening.clone(), rect(34.0, 0.0));
        middle.overlap_bounds = PaintRect {
            x: 29.0,
            y: 0.0,
            width: 20.0,
            height: 10.0,
        };
        let mut layers = vec![layer(0, home.clone(), rect(0.0, 0.0)), middle];
        append_with_overlap_merge(&mut layers, layer(2, home.clone(), rect(20.0, 0.0)), 1.0);
        assert_eq!(layers.len(), 3, "outset/fringe overlap cannot be ignored");

        let mut layers = vec![
            layer(0, home.clone(), rect(0.0, 0.0)),
            layer(1, intervening, rect(2000.0, 2000.0)),
        ];
        append_with_overlap_merge(&mut layers, layer(2, home, rect(1000.0, 1000.0)), 1.0);
        assert_eq!(
            layers.len(),
            3,
            "official sparse-area limit must reject a large empty layer"
        );
    }
}

fn append_with_overlap_merge(groups: &mut Vec<PendingLayer>, guest: PendingLayer, scale: f64) {
    // PaintArtifactCompositor::LayerizeGroup's reverse Merge/MightOverlap
    // search, restricted to identical retained transforms and effects. This
    // never reorders across a scroller, animated anchor, mask/effect boundary,
    // foreign layer, scrollbar, or unknown ink.
    if !guest.singleton_chunk
        && guest.complete
        && !guest.properties.effect.is_mask
        && finite(guest.bounds)
        && finite(guest.overlap_bounds)
    {
        // Fractional tile placement samples one adjacent native pixel.
        // Include that support before declaring disjointness; use actual
        // record ink, never clip-only or loose chunk bounds.
        let sampling_support = |rect: PaintRect| {
            if rect.is_empty() {
                return rect;
            }
            let fringe = 1.0 / scale;
            PaintRect {
                x: rect.x - fringe,
                y: rect.y - fringe,
                width: rect.width + 2.0 * fringe,
                height: rect.height + 2.0 * fringe,
            }
        };
        let guest_support = sampling_support(guest.overlap_bounds);
        for index in (0..groups.len()).rev() {
            let candidate = &groups[index];
            if candidate.singleton_chunk
                || !candidate.complete
                || candidate.properties.transform.lifecycle != guest.properties.transform.lifecycle
                || candidate.properties.effect.lifecycle != guest.properties.effect.lifecycle
                || !finite(candidate.bounds)
                || !finite(candidate.overlap_bounds)
            {
                break;
            }
            if candidate.properties.same_nodes(&guest.properties) {
                let mut merged = candidate.bounds;
                merged.union(guest.bounds);
                // PendingLayer::CanMerge: kMergeSparsityAreaTolerance = 10000.
                let added_area = merged.width * merged.height
                    - candidate.bounds.width * candidate.bounds.height
                    - guest.bounds.width * guest.bounds.height;
                if added_area.is_finite() && added_area <= 10000.0 * scale * scale {
                    merge_pending_layer(&mut groups[index], guest);
                    return;
                }
            }
            if intersects(guest_support, sampling_support(candidate.overlap_bounds)) {
                break;
            }
        }
    }
    groups.push(guest);
}

fn state_snapshot_key(state: &PropertyTreeState) -> (usize, usize, usize) {
    // Frame-local immutable Arc snapshots only. This memoization is not a
    // display-item/layer identity and never survives the input artifact.
    (
        Arc::as_ptr(&state.transform) as usize,
        Arc::as_ptr(&state.clip) as usize,
        Arc::as_ptr(&state.effect) as usize,
    )
}

fn validate(
    list: &PaintArtifact,
    content: Option<&[RasterRecordContent]>,
) -> Result<(), UnsupportedReason> {
    if std::env::var_os("LAYOUTNG_LAYER_REPLAY_VERBOSE").is_some() {
        for item in list
            .items
            .iter()
            .filter(|item| item.r#type == DisplayItemType::kDrawMask)
        {
            eprintln!(
                "[layer_tile source Mask DrawingDisplayItem] node={} fragment={} rect={:?} layers={}",
                item.node_id,
                item.fragment_instance_id,
                item.rect,
                item.mask_layers.len()
            );
            for (index, layer) in item.mask_layers.iter().enumerate() {
                eprintln!("[layer_tile source mask layer] index={} resource={} shader={} clip={:?} radii={:?} source={:?} tile={:?} repeat=({},{}) rules=({:?},{:?}) scale={:?} spacing={:?} mode={:?} composite={:?}", index, layer.resource_id, layer.paint_shader.is_some(), layer.clip_rect, layer.clip_radii, layer.source_rect, layer.tile_rect, layer.repeat_x, layer.repeat_y, layer.repeat_rule_x, layer.repeat_rule_y, layer.tile_scale, layer.tile_spacing, layer.mode, layer.composite);
            }
        }
    }
    // An empty artifact may still carry balanced legacy state wrappers (for
    // example Save/viewport ClipRect/Restore before the document first paints).
    // Inspect the actual commands: an empty canonical record list alone is
    // never evidence that an unrecorded draw is safe to ignore.
    let empty_state_artifact =
        list.chunks.is_empty() && list.display_items.is_empty() && balanced_state_only(&list.items);
    if !list.items.is_empty()
        && (list.chunks.is_empty() || list.display_items.is_empty())
        && !empty_state_artifact
    {
        return Err(UnsupportedReason::MissingSemanticRecords);
    }
    let mut end = 0usize;
    for chunk in &list.chunks {
        if chunk.begin_index as usize != end
            || chunk.end_index < chunk.begin_index
            || chunk.end_index as usize > list.display_items.len()
        {
            return Err(UnsupportedReason::InvalidChunkRange);
        }
        end = chunk.end_index as usize;
    }
    if end != list.display_items.len() {
        return Err(UnsupportedReason::InvalidChunkRange);
    }
    let mut previous_end = 0;
    for record in &list.display_items {
        if record.record_begin < previous_end
            || record.record_end < record.record_begin
            || record.record_end > list.items.len()
            || !finite(record.visual_rect)
        {
            return Err(UnsupportedReason::InvalidRecordRange);
        }
        previous_end = record.record_end;
    }
    if let Some(content) = content {
        let mut seen = HashSet::new();
        if content.len() != list.display_items.len() {
            return Err(UnsupportedReason::InvalidRecordRange);
        }
        for record in content {
            if record.record_index >= list.display_items.len()
                || !seen.insert(record.record_index)
                || !finite(record.visual_rect)
            {
                return Err(UnsupportedReason::InvalidRecordRange);
            }
        }
    }
    Ok(())
}

fn balanced_state_only(items: &[DisplayItem]) -> bool {
    let mut depth = 0usize;
    for item in items {
        match item.r#type {
            DisplayItemType::kSave => depth += 1,
            DisplayItemType::kRestore => {
                let Some(previous) = depth.checked_sub(1) else {
                    return false;
                };
                depth = previous;
            }
            DisplayItemType::kConcat
            | DisplayItemType::kClipRect
            | DisplayItemType::kClipRoundedRect
            | DisplayItemType::kClipOutRoundedRect
            | DisplayItemType::kClipPath
            | DisplayItemType::kClipOutRect => {}
            // Layer/effect/mask operations are not plain state wrappers.
            // All draw variants must have semantic records.
            _ => return false,
        }
    }
    depth == 0
}

fn build_records(
    list: &PaintArtifact,
    indices: &[usize],
    canonical: Option<&HashMap<usize, &RasterRecordContent>>,
    old: &[RecordState],
) -> Vec<RecordState> {
    let by_id: Option<HashMap<_, _>> = canonical
        .is_none()
        .then(|| old.iter().map(|r| (r.id, r)).collect());
    indices
        .iter()
        .map(|&index| {
            let record = &list.display_items[index];
            let content = canonical.and_then(|c| c.get(&index).copied());
            let items = if let Some(content) = content {
                content.items.clone()
            } else {
                let ops = &list.items[record.record_begin..record.record_end];
                by_id
                    .as_ref()
                    .and_then(|by_id| by_id.get(&record.id))
                    .filter(|r| r.items.as_ref() == ops)
                    .map_or_else(|| Arc::from(ops), |r| r.items.clone())
            };
            RecordState {
                artifact_index: index,
                id: record.id,
                rect: content.map_or(record.visual_rect, |c| c.visual_rect),
                complete: content.map_or(record.visual_rect_is_accurate, |c| c.bounds_are_complete),
                draws_content: record.draws_content,
                raster_effect_outset: record.raster_effect_outset,
                items,
            }
        })
        .collect()
}

fn record_damage(
    old: &[RecordState],
    new: &[RecordState],
    force: bool,
    resources: &ResourceComparison<'_>,
    diagnostics: bool,
) -> (bool, Vec<PaintRect>, Option<String>) {
    let mut all = force;
    let mut damage = Vec::new();
    let mut first_change = None;
    let mut first_unknown = None;
    // Match real source identities, not vector positions. Inserting/removing a
    // preceding record must not invalidate every unchanged subsequent record.
    // A queue also prevents incorrectly reusing one old record twice if a
    // malformed/translated artifact still contains duplicate IDs.
    let mut old_by_id: HashMap<DisplayItemId, VecDeque<usize>> = HashMap::new();
    let same_id_order = old.len() == new.len() && old.iter().zip(new).all(|(a, b)| a.id == b.id);
    if !same_id_order {
        for (index, record) in old.iter().enumerate() {
            old_by_id.entry(record.id).or_default().push_back(index);
        }
    }
    let mut matched = if same_id_order {
        Vec::new()
    } else {
        vec![false; old.len()]
    };
    let mut highest_old_index = None;
    for (new_index, b) in new.iter().enumerate() {
        let index = if same_id_order {
            Some(new_index)
        } else {
            old_by_id.get_mut(&b.id).and_then(VecDeque::pop_front)
        };
        let a = index.map(|index| &old[index]);
        let reordered = (a.is_some_and(|a| a.draws_content) || b.draws_content)
            && index.is_some_and(|index| highest_old_index.is_some_and(|max| index < max));
        if let Some(index) = index.filter(|_| !same_id_order) {
            matched[index] = true;
            if a.is_some_and(|a| a.draws_content) || b.draws_content {
                highest_old_index =
                    Some(highest_old_index.map_or(index, |max: usize| max.max(index)));
            }
        }
        let resource_changed = a.is_some_and(|a| resources.record_changed(a, b));
        let unchanged = a.is_some_and(|a| {
            // Ink/accuracy/outset are allocation metadata. If the actual
            // replay and referenced resources are identical, changing only
            // that metadata cannot change already-rasterized pixels.
            a.draws_content == b.draws_content
                && !resource_changed
                && !reordered
                && (Arc::ptr_eq(&a.items, &b.items)
                    || pixel_records_equal(&a.items, &b.items, resources))
        });
        if unchanged {
            continue;
        }
        if diagnostics && first_change.is_none() {
            first_change = Some(record_change_diagnostic(
                a,
                Some(b),
                reordered,
                resource_changed,
            ));
        }
        if diagnostics
            && first_unknown.is_none()
            && a.into_iter()
                .chain(Some(b))
                .any(|record| record.draws_content && !record.complete)
        {
            first_unknown = Some(record_change_diagnostic(
                a,
                Some(b),
                reordered,
                resource_changed,
            ));
        }
        for record in a.into_iter().chain(Some(b)) {
            add_record_damage(record, &mut all, &mut damage);
        }
    }
    for (_, record) in old
        .iter()
        .enumerate()
        .filter(|(index, _)| !same_id_order && !matched[*index])
    {
        if diagnostics && first_change.is_none() {
            first_change = Some(record_change_diagnostic(Some(record), None, false, false));
        }
        if diagnostics && first_unknown.is_none() && record.draws_content && !record.complete {
            first_unknown = Some(record_change_diagnostic(Some(record), None, false, false));
        }
        add_record_damage(record, &mut all, &mut damage);
    }
    if let Some(unknown) = first_unknown {
        first_change
            .get_or_insert_with(String::new)
            .push_str(&format!(" first_unknown_draw=[{}]", unknown));
    }
    (all, damage, first_change)
}

fn add_record_damage(record: &RecordState, all: &mut bool, damage: &mut Vec<PaintRect>) {
    if record.draws_content {
        // True unknown ink remains conservative. State-only records cannot
        // invalidate raster pixels merely because their visual rect is loose.
        *all |= !record.complete;
        damage.push(outset(record.rect, record.raster_effect_outset));
    }
}

struct ResourceComparison<'a> {
    old: Option<&'a PaintResources>,
    new: Option<&'a PaintResources>,
    changed_images: HashSet<u64>,
    changed_records: HashSet<usize>,
    identical_catalog: bool,
    font_changes: RefCell<HashMap<(u32, u32), bool>>,
}

impl<'a> ResourceComparison<'a> {
    fn new(
        old: Option<&'a PaintResources>,
        new: Option<&'a PaintResources>,
        dependencies: &paint::paint_engine::ImageDependencyIndex,
    ) -> Self {
        let identical_catalog = match (old, new) {
            (None, None) => true,
            (Some(a), Some(b)) => std::ptr::eq(a, b),
            _ => false,
        };
        if identical_catalog {
            return Self {
                old,
                new,
                changed_images: HashSet::new(),
                changed_records: HashSet::new(),
                identical_catalog,
                font_changes: RefCell::new(HashMap::new()),
            };
        }
        let old_images: HashMap<_, _> = old
            .into_iter()
            .flat_map(|r| &r.images)
            .map(|image| (image.id, image))
            .collect();
        let new_images: HashMap<_, _> = new
            .into_iter()
            .flat_map(|r| &r.images)
            .map(|image| (image.id, image))
            .collect();
        let changed_images: HashSet<u64> = old_images
            .keys()
            .chain(new_images.keys())
            .copied()
            .filter(|id| *id != 0)
            .filter(|id| match (old_images.get(id), new_images.get(id)) {
                (Some(a), Some(b)) => {
                    a.width != b.width
                        || a.height != b.height
                        || a.resolution_scale != b.resolution_scale
                        || a.revision != b.revision
                        || a.content != b.content
                }
                _ => true,
            })
            .collect();
        let changed_records = changed_images
            .iter()
            .flat_map(|id| dependencies.dependencies(*id))
            .map(|dependency| dependency.record_index)
            .collect();
        Self {
            old,
            new,
            changed_images,
            changed_records,
            identical_catalog,
            font_changes: RefCell::new(HashMap::new()),
        }
    }

    fn font_changed(&self, old_index: u32, new_index: u32) -> bool {
        if self.identical_catalog && old_index == new_index {
            return false;
        }
        let key = (old_index, new_index);
        if let Some(changed) = self.font_changes.borrow().get(&key).copied() {
            return changed;
        }
        let changed = self.old.and_then(|r| r.fonts.get(old_index as usize))
            != self.new.and_then(|r| r.fonts.get(new_index as usize));
        self.font_changes.borrow_mut().insert(key, changed);
        changed
    }

    fn item_images_changed(&self, item: &DisplayItem) -> bool {
        self.changed_images.contains(&item.resource_id)
            || item
                .paint_shader
                .as_ref()
                .is_some_and(|shader| self.changed_images.contains(&shader.resource_id))
            || item.mask_layers.iter().any(|mask| {
                self.changed_images.contains(&mask.resource_id)
                    || mask
                        .paint_shader
                        .as_ref()
                        .is_some_and(|shader| self.changed_images.contains(&shader.resource_id))
            })
    }

    fn record_changed(&self, old: &RecordState, new: &RecordState) -> bool {
        // The catalogue is an immutable snapshot. Referenced images/fonts
        // cannot change while the exact catalogue Arc is retained.
        if self.identical_catalog || (!old.draws_content && !new.draws_content) {
            return false;
        }
        if old.id == new.id && self.changed_records.contains(&new.artifact_index) {
            return true;
        }
        old.items
            .iter()
            .chain(new.items.iter())
            .any(|item| self.item_images_changed(item))
            || old.items.iter().zip(new.items.iter()).any(|(a, b)| {
                a.r#type == DisplayItemType::kDrawGlyphRun
                    && b.r#type == DisplayItemType::kDrawGlyphRun
                    && self.font_changed(a.font_face_index, b.font_face_index)
            })
    }
}

fn pixel_records_equal(
    old: &[DisplayItem],
    new: &[DisplayItem],
    resources: &ResourceComparison<'_>,
) -> bool {
    old.len() == new.len()
        && old.iter().zip(new).all(|(a, b)| {
            if a == b {
                return true;
            }
            // Record identity is already matched above. These source/export
            // annotations do not participate in native paint-op replay pixels.
            let mut normalized = a.clone();
            normalized.node_id = b.node_id;
            normalized.fragment_instance_id = b.fragment_instance_id;
            normalized.phase = b.phase;
            if a.r#type == DisplayItemType::kDrawGlyphRun
                && !resources.font_changed(a.font_face_index, b.font_face_index)
            {
                // A reordered catalogue may rebase the same real face's handle.
                normalized.font_face_index = b.font_face_index;
            }
            normalized == *b
        })
}

fn record_change_diagnostic(
    old: Option<&RecordState>,
    new: Option<&RecordState>,
    reordered: bool,
    resource: bool,
) -> String {
    let record = new.or(old).unwrap();
    let item = record.items.iter().find(|item| {
        !matches!(
            item.r#type,
            DisplayItemType::kSave | DisplayItemType::kRestore | DisplayItemType::kConcat
        )
    });
    let pair = old
        .zip(new)
        .and_then(|(a, b)| a.items.iter().zip(b.items.iter()).find(|(a, b)| a != b));
    format!("id={:?} type={:?} added={} removed={} reordered={} resource={} old_rect={:?} new_rect={:?} accurate={:?} ops={:?} old_ops={:?} new_ops={:?} payload_diff={:?}", record.id, item.map(|item| item.r#type), old.is_none(), new.is_none(), reordered, resource,
        old.map(|r|r.rect), new.map(|r|r.rect), old.zip(new).map(|(a,b)|(a.complete,b.complete)),
        old.zip(new).map(|(a,b)|(a.items.len(),b.items.len())),
        old.map(|r|r.items.iter().take(8).map(|item|(item.r#type,item.rect,item.glyphs.len(),item.path.len())).collect::<Vec<_>>()),
        new.map(|r|r.items.iter().take(8).map(|item|(item.r#type,item.rect,item.glyphs.len(),item.path.len())).collect::<Vec<_>>()),
        pair.map(|(a,b)| format!("kind:{} rect:{} transform:{} glyphs:{} font:{} image:{} shader:{} color:{} radii:{} shadow:{} source_node:{} source_fragment:{} phase:{}",
            a.r#type!=b.r#type,a.rect!=b.rect,a.transform!=b.transform,a.glyphs!=b.glyphs,
            a.font_face_index!=b.font_face_index,a.resource_id!=b.resource_id,a.paint_shader!=b.paint_shader,
            a.color!=b.color,a.corner_radii!=b.corner_radii,a.shadow_offset!=b.shadow_offset,
            a.node_id!=b.node_id,a.fragment_instance_id!=b.fragment_instance_id,a.phase!=b.phase)))
}

fn scroll_target_rect(
    properties: &PropertyTreeState,
    scroll: CompositorScrollOffset,
    viewport: PaintRect,
    raster_content_bounds: Option<PaintRect>,
    raster_scale: f64,
) -> Option<PaintRect> {
    if !layer_has_scroll_node(&properties.transform, scroll.scroll_node_id) {
        return None;
    }
    // Resolve transform and clip together at the target scroll state. A clip
    // whose local transform descends from the scroll node moves with its
    // content. Reusing the committed clip here could produce an empty target
    // while activation, correctly using the updated clip, required pixels.
    let (device_translation, device_clip) =
        resolved_compositor_properties_with_scroll(properties, raster_scale, Some(scroll)).ok()?;
    let target_translation = (
        device_translation.0 / raster_scale,
        device_translation.1 / raster_scale,
    );
    let target_clip = device_clip.map(|clip| PaintRect {
        x: clip.x / raster_scale,
        y: clip.y / raster_scale,
        width: clip.width / raster_scale,
        height: clip.height / raster_scale,
    });
    let visible_root = target_clip.map_or(viewport, |clip| intersection(viewport, clip));
    let mut target = shifted(visible_root, (-target_translation.0, -target_translation.1));
    // Bilinear sampling may read the neighboring source pixel. Use the same
    // footprint for draw, activation and skewport targets.
    if !target.is_empty()
        && (!pixel_aligned(target_translation.0, raster_scale)
            || !pixel_aligned(target_translation.1, raster_scale))
    {
        let footprint = 1.0 / raster_scale;
        target = PaintRect {
            x: target.x - footprint,
            y: target.y - footprint,
            width: target.width + 2.0 * footprint,
            height: target.height + 2.0 * footprint,
        };
    }
    if let Some(bounds) = raster_content_bounds {
        target = intersection(target, bounds);
    }
    Some(target)
}

fn layer_has_scroll_node(transform: &Arc<TransformPaintPropertyNode>, scroll_node_id: u64) -> bool {
    let mut node = Some(transform);
    while let Some(transform) = node {
        if transform
            .scroll
            .as_ref()
            .is_some_and(|candidate| candidate.id == scroll_node_id)
        {
            return true;
        }
        node = transform.parent.as_ref();
    }
    false
}

fn tile_rect(origin: (f64, f64), index: (i32, i32), side: f64) -> PaintRect {
    PaintRect {
        x: origin.0 + index.0 as f64 * side,
        y: origin.1 + index.1 as f64 * side,
        width: side,
        height: side,
    }
}

fn raster_origin(bounds: PaintRect, scale: f64, complete: bool) -> (f64, f64) {
    // cc PictureLayerTiling::ComputeTilingRect snaps recorded-subset bounds to
    // limit origin changes (picture_layer_tiling.cc:891); an origin change
    // resets the tile map (line 179). Our incomplete ink is not a known complete
    // picture boundary. Anchor that subset's grid at the retained raster
    // space's (0,0), so culling/added records cannot move existing tile slots.
    // Negative indices remain valid. This does not move record coordinates or
    // claim that unknown ink is complete; allocation still covers the viewport.
    if !complete {
        return (0.0, 0.0);
    }
    (
        (bounds.x * scale).floor() / scale,
        (bounds.y * scale).floor() / scale,
    )
}

fn grid_range(
    rect: PaintRect,
    origin: (f64, f64),
    tile_size: u32,
    scale: f64,
) -> Option<(i32, i32, i32, i32)> {
    if rect.is_empty() {
        return Some((0, 0, 0, 0));
    }
    let side = tile_size as f64 / scale;
    if !side.is_finite() || side <= 0.0 {
        return None;
    }
    let values = [
        ((rect.x - origin.0) / side).floor(),
        ((rect.y - origin.1) / side).floor(),
        ((rect.x + rect.width - origin.0) / side).ceil(),
        ((rect.y + rect.height - origin.1) / side).ceil(),
    ];
    if values
        .iter()
        .any(|v| !v.is_finite() || *v < i32::MIN as f64 || *v > i32::MAX as f64)
    {
        return None;
    }
    Some((
        values[0] as i32,
        values[1] as i32,
        values[2] as i32,
        values[3] as i32,
    ))
}

fn grid_tile_count(grid: (i32, i32, i32, i32)) -> Option<usize> {
    ((grid.2 as i64 - grid.0 as i64).max(0) as usize)
        .checked_mul((grid.3 as i64 - grid.1 as i64).max(0) as usize)
}

fn rect_manhattan_distance(a: PaintRect, b: PaintRect) -> f64 {
    let a_right = a.x + a.width;
    let a_bottom = a.y + a.height;
    let b_right = b.x + b.width;
    let b_bottom = b.y + b.height;
    let dx = if a_right < b.x {
        b.x - a_right
    } else if b_right < a.x {
        a.x - b_right
    } else {
        0.0
    };
    let dy = if a_bottom < b.y {
        b.y - a_bottom
    } else if b_bottom < a.y {
        a.y - b_bottom
    } else {
        0.0
    };
    dx + dy
}

fn soon_interest_rect(
    visible: PaintRect,
    raster_content_bounds: Option<PaintRect>,
    scale: f64,
) -> PaintRect {
    if visible.is_empty() || !scale.is_finite() || scale <= 0.0 {
        return visible;
    }
    let distance = (visible.width.max(visible.height) * SOON_BORDER_VIEWPORT_PERCENTAGE)
        .min(MAX_SOON_BORDER_DEVICE_PIXELS / scale);
    let expanded = PaintRect {
        x: visible.x - distance,
        y: visible.y - distance,
        width: visible.width + distance * 2.0,
        height: visible.height + distance * 2.0,
    };
    raster_content_bounds.map_or(expanded, |bounds| intersection(expanded, bounds))
}
