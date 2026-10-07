//! CPU execution of persistent layer/tile plans. The planner owns identities;
//! this renderer owns premultiplied tile pixels and draws into the borrowed output.
use crate::clip_mask::{ClipMaskCache, MaskCoverage};
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintFilterType};
use paint::paint_property_tree::{ClipPaintPropertyNode, EffectPaintPropertyNode};
use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use layer_tile::{
    FramePlan, RasterTask, TileId, TilePlacement, TileResourceOwner, UnsupportedReason,
};
use paint::paint_engine::PaintRect;
use skia::src::core::SkCanvas::RasterImageCache;

use crate::layer_replay::{LayerComposition, LayerReplay};
use crate::layer_tile_renderer::RasterUpdate;
use crate::tile_worker::{PendingTiles, TileJob, TileProduct, TileWorkerPool};

#[path = "layer_clip_quad.rs"]
mod layer_clip_quad;
#[cfg(feature = "compose_work_profile")]
#[path = "layer_compose_profile.rs"]
mod layer_compose_profile;
#[path = "layer_composition_preflight.rs"]
mod layer_composition_preflight;
#[path = "layer_direct_clip.rs"]
mod layer_direct_clip;
#[path = "layer_occlusion.rs"]
mod layer_occlusion;
#[path = "layer_solid.rs"]
pub(crate) mod layer_solid;
#[path = "layer_solid_analysis.rs"]
mod layer_solid_analysis;

// Bounded stack storage for software sampling spans; larger tiles use the
// existing general path. Tile geometry itself comes exclusively from FramePlan.
const TILE_SIZE: u32 = 256;

#[derive(Default)]
struct ComposeCosts {
    open: std::time::Duration,
    clip: std::time::Duration,
    mask_layers: usize,
    mask_raster_tasks: usize,
    mask_reused_tiles: usize,
    direct_mask_layers: usize,
    scratch: std::time::Duration,
    blit: std::time::Duration,
    close: std::time::Duration,
    scopes: usize,
    integral: usize,
    fractional: usize,
    fractional_phase_classes: [usize; 3],
    skipped_zero: usize,
    nominal_pixels: usize,
    occluded_tiles: usize,
    occluded_pixels: usize,
    tile_blits: [std::time::Duration; 5],
    tile_pixels: [usize; 5],
    solid_tiles: usize,
    direct_clip_layers: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayerTileRasterStats {
    pub layers: usize,
    /// Visible tiles in the current plan.
    pub tiles: usize,
    pub raster_tasks: usize,
    pub reused_tiles: usize,
    /// Includes retained offscreen tiles until the planner retires them.
    pub resident_bytes: usize,
    pub unsupported_reason: Option<&'static str>,
}

struct CachedTile {
    raster_frame: u64,
    generation: u64,
    pixel_size: (u32, u32),
    raster_scale: f64,
    white_backing: bool,
    rgba: Vec<u8>,
    // The macOS software output device and Chromium's native N32 resources
    // are BGRA. Keep that backend representation with the retained tile so an
    // unchanged integer tile is a row copy, rather than a fresh channel
    // permutation on every compositor frame. RGBA remains the canonical
    // sampler input for fractional edges, masks and isolated effects.
    bgra: Vec<u32>,
    // True byte-exact transparent margins, computed only for a fresh product.
    // Include RGB too: zero alpha alone does not prove an identity source.
    row_support: TileRowSupport,
}

#[derive(Clone, Copy)]
struct RowSupport {
    extent: (usize, usize),
    // Exact source alpha=255 throughout extent, even when other tile rows
    // or its transparent padding prevent the whole-tile opacity proof.
    opaque: bool,
    // A dense row uses its extent directly, without duplicating that span.
    runs: (usize, usize),
}

pub(crate) struct TileRowSupport {
    rows: Vec<RowSupport>,
    runs: Vec<(usize, usize)>,
    // Exact pixel proof, retained with this tile generation.
    opaque: bool,
    draw_mode: layer_solid::TileDrawMode,
}

impl TileRowSupport {
    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }

    fn spans(&self, y: usize) -> &[(usize, usize)] {
        let row = &self.rows[y];
        if row.runs.0 != row.runs.1 {
            &self.runs[row.runs.0..row.runs.1]
        } else if row.extent.0 != row.extent.1 {
            std::slice::from_ref(&row.extent)
        } else {
            &[]
        }
    }
}

#[derive(Clone, Copy)]
struct SurfaceBounds {
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
}
// Pixel storage can cover a subrectangle, while paint/mask coordinates remain
// global. Never substitute the destination stride/origin for the source view.
#[derive(Clone, Copy)]
struct TargetView {
    origin: (usize, usize),
    stride: usize,
}
impl TargetView {
    fn row_start(self, y: usize, x: usize) -> usize {
        (y - self.origin.1) * self.stride + (x - self.origin.0)
    }
    fn row_range(self, y: usize, left: usize, right: usize) -> std::ops::Range<usize> {
        let start = self.row_start(y, left);
        start..start + right - left
    }
}
fn union_bounds(current: &mut Option<SurfaceBounds>, next: Option<SurfaceBounds>) {
    if let Some(b) = next {
        *current = Some(current.map_or(b, |a| SurfaceBounds {
            left: a.left.min(b.left),
            top: a.top.min(b.top),
            right: a.right.max(b.right),
            bottom: a.bottom.max(b.bottom),
        }));
    }
}
#[derive(Clone)]
enum Scope {
    Clip(Arc<ClipPaintPropertyNode>),
    Effect(Arc<EffectPaintPropertyNode>),
    DstIn(Arc<EffectPaintPropertyNode>),
}
impl Scope {
    fn same_node(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Clip(a), Self::Clip(b)) => a.lifecycle.same_node(&b.lifecycle),
            (Self::Effect(a), Self::Effect(b)) => a.lifecycle.same_node(&b.lifecycle),
            (Self::DstIn(a), Self::DstIn(b)) => a.lifecycle.same_node(&b.lifecycle),
            _ => false,
        }
    }
    fn same_values(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Clip(a), Self::Clip(b)) => a.lifecycle.revision == b.lifecycle.revision,
            (Self::Effect(a), Self::Effect(b)) => {
                a.lifecycle.revision == b.lifecycle.revision && a.opacity == b.opacity
            }
            (Self::DstIn(a), Self::DstIn(b)) => {
                a.lifecycle.revision == b.lifecycle.revision && a.opacity == b.opacity
            }
            _ => false,
        }
    }
    fn opacity(&self) -> f32 {
        match self {
            Self::Clip(_) => 1.0,
            Self::Effect(effect) | Self::DstIn(effect) => effect.opacity,
        }
    }
}

fn effect_blur_sigmas(
    scope: &Scope,
    scale: f64,
) -> io::Result<Vec<(f32, f32)>> {
    let Scope::Effect(effect) = scope else {
        return Ok(Vec::new());
    };
    if effect.filters.is_empty() {
        return Ok(Vec::new());
    }
    let matrix = layer_tile::compositor_transform(&effect.local_transform_space, scale)
        .map_err(|_| io::Error::new(io::ErrorKind::Unsupported, "filter-transform"))?;
    let sx = matrix.values[0].hypot(matrix.values[1]);
    let sy = matrix.values[4].hypot(matrix.values[5]);
    effect
        .filters
        .iter()
        .map(|filter| {
            if filter.r#type != PaintFilterType::kBlur
                || !filter.amount.is_finite()
                || filter.amount < 0.0
            {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "filter-effect",
                ));
            }
            Ok(((filter.amount * sx) as f32, (filter.amount * sy) as f32))
        })
        .collect()
}

fn expanded_for_blur(
    bounds: Option<SurfaceBounds>,
    sigmas: &[(f32, f32)],
    width: u32,
    height: u32,
) -> Option<SurfaceBounds> {
    let mut bounds = bounds?;
    let x = sigmas
        .iter()
        .map(|(sigma, _)| (sigma * 3.0).ceil() as usize + 1)
        .sum::<usize>();
    let y = sigmas
        .iter()
        .map(|(_, sigma)| (sigma * 3.0).ceil() as usize + 1)
        .sum::<usize>();
    bounds.left = bounds.left.saturating_sub(x);
    bounds.top = bounds.top.saturating_sub(y);
    bounds.right = bounds.right.saturating_add(x).min(width as usize);
    bounds.bottom = bounds.bottom.saturating_add(y).min(height as usize);
    Some(bounds)
}
fn needs_clip_mask(clip: &ClipPaintPropertyNode, _scale: f64) -> bool {
    // PropertyTreeManager::SyntheticEffectType does not create an effect for
    // an axis-aligned plain rect, even at a fractional coordinate. Retained
    // compositor rects use LayerComposition's enclosing integer scissor
    // (draw_property_utils / SoftwareRenderer::SetClipRect). Clips removed
    // into a layer's PaintRecord remain AA, as in SwitchToClip/StartClip.
    // LayerReplay has already rejected non-translation transforms.
    clip.radii != PaintCornerRadii::default() || !clip.clip_path.is_empty()
}
fn append_clip_scopes(
    leaf: &Arc<ClipPaintPropertyNode>,
    scopes: &mut Vec<Scope>,
    seen: &mut Vec<Arc<ClipPaintPropertyNode>>,
    scale: f64,
) {
    let mut chain = Vec::new();
    let mut clip = Some(leaf.clone());
    while let Some(node) = clip {
        clip = node.parent.clone();
        chain.push(node);
    }
    for clip in chain.into_iter().rev() {
        if !seen
            .iter()
            .any(|old| old.lifecycle.same_node(&clip.lifecycle))
        {
            if needs_clip_mask(&clip, scale) {
                scopes.push(Scope::Clip(clip.clone()));
            }
            seen.push(clip);
        }
    }
}
// Actual Mask effects require their content parent to be a render surface,
// including opacity-one parents. Derive this from all committed mask layers,
// not just visible tiles: a fully transparent/offscreen source still clears DstIn.
fn masked_content_effects(
    plan: &layer_tile::FramePlan,
) -> io::Result<Vec<Arc<EffectPaintPropertyNode>>> {
    let mut parents: Vec<Arc<EffectPaintPropertyNode>> = Vec::new();
    for layer in &plan.layers {
        let mut node = Some(layer.properties.effect.clone());
        while let Some(effect) = node {
            node = effect.parent.clone();
            if effect.is_mask {
                let parent = effect
                    .parent
                    .as_ref()
                    .ok_or_else(|| io::Error::other("mask effect has no content parent"))?;
                if !parents
                    .iter()
                    .any(|old| old.lifecycle.same_node(&parent.lifecycle))
                {
                    parents.push(parent.clone());
                }
            }
        }
    }
    Ok(parents)
}
fn composition_scopes(
    effect: &Arc<EffectPaintPropertyNode>,
    retained: &[Arc<ClipPaintPropertyNode>],
    masked_parents: &[Arc<EffectPaintPropertyNode>],
    scale: f64,
) -> io::Result<Vec<Scope>> {
    let mut scopes = Vec::new();
    let mut seen = Vec::new();
    let mut chain = Vec::new();
    let mut node = Some(effect.clone());
    while let Some(effect) = node {
        node = effect.parent.clone();
        chain.push(effect);
    }
    for effect in chain.into_iter().rev() {
        if effect.is_mask {
            // Clip the mask source inside its DstIn surface. The content parent
            // must remain the immediate restore destination, so transparent
            // source outside its output clip also clears the parent contents.
            scopes.push(Scope::DstIn(effect.clone()));
            if let Some(clip) = &effect.output_clip {
                append_clip_scopes(clip, &mut scopes, &mut seen, scale);
            }
        } else if effect.opacity != 1.0
            || !effect.filters.is_empty()
            || masked_parents
                .iter()
                .any(|parent| parent.lifecycle.same_node(&effect.lifecycle))
        {
            // Ordinary opacity-one ancestors remain on the existing clip/tile
            // path. Only actual isolated effects add an output-clip surface.
            if let Some(clip) = &effect.output_clip {
                append_clip_scopes(clip, &mut scopes, &mut seen, scale);
            }
            scopes.push(Scope::Effect(effect));
        }
    }
    for clip in retained.iter().rev() {
        if !seen
            .iter()
            .any(|old| old.lifecycle.same_node(&clip.lifecycle))
        {
            if needs_clip_mask(clip, scale) {
                scopes.push(Scope::Clip(clip.clone()));
            }
            seen.push(clip.clone());
        }
    }
    Ok(scopes)
}

struct EffectSurface {
    scope: Scope,
    direct_dst_in: bool,
    direct_clip: bool,
    coverage: Option<Arc<MaskCoverage>>,
    // All intermediate surfaces retain premultiplied RGBA including alpha.
    pixels: Vec<u32>,
    view: TargetView,
    touched: Option<SurfaceBounds>,
    blur_sigmas: Vec<(f32, f32)>,
}
struct EffectScratch {
    pixels: Vec<u32>,
    touched: Option<SurfaceBounds>,
    view: TargetView,
}

struct LayerEffectPlan {
    chain: Vec<Scope>,
    opened_runs: Vec<usize>,
    direct_mask: Option<Vec<SurfaceBounds>>,
}

// Each opening gets its own run, even if the same native node reappears later.
// Enclose all actual visible tile writes in that run, including nested groups.
// Bounds affect allocation only; real mask coverage still runs at group exit.
fn effect_allocation_plan(
    plan: &layer_tile::FramePlan,
    replay: &LayerReplay<'_>,
    width: u32,
    height: u32,
    scale: f64,
) -> io::Result<(Vec<LayerEffectPlan>, Vec<Option<SurfaceBounds>>)> {
    let masked_parents = masked_content_effects(plan)?;
    let mut layers = Vec::with_capacity(plan.layers.len());
    let mut runs = Vec::new();
    let mut active: Vec<(Scope, usize)> = Vec::new();
    for layer in &plan.layers {
        if layer.tiles.is_empty() && !layer.properties.effect.is_mask {
            layers.push(LayerEffectPlan {
                chain: Vec::new(),
                opened_runs: Vec::new(),
                direct_mask: None,
            });
            continue;
        }
        let chain = composition_scopes(
            &layer.properties.effect,
            &replay.retained_clip_nodes(layer.id),
            &masked_parents,
            scale,
        )?;
        if chain
            .iter()
            .any(|scope| matches!(scope, Scope::Effect(effect) if effect.opacity == 0.0))
        {
            layers.push(LayerEffectPlan {
                chain,
                opened_runs: Vec::new(),
                direct_mask: None,
            });
            continue;
        }
        let common = active
            .iter()
            .zip(&chain)
            .take_while(|((old, _), new)| old.same_node(new))
            .count();
        active.truncate(common);
        let mut opened_runs = Vec::with_capacity(chain.len() - common);
        for scope in chain.iter().skip(common) {
            let index = runs.len();
            runs.push(None);
            active.push((scope.clone(), index));
            opened_runs.push(index);
        }
        if !active.is_empty() {
            let composition = replay
                .composition(layer.id)
                .ok_or_else(|| io::Error::other("paint layer has no composition"))?;
            let mut bounds = None;
            for tile in &layer.tiles {
                union_bounds(
                    &mut bounds,
                    tile_composition_geometry(tile, composition, width, height)?
                        .map(|(bounds, _, _)| bounds),
                );
            }
            for (scope, index) in &active {
                let sigmas = effect_blur_sigmas(scope, scale)?;
                union_bounds(
                    &mut runs[*index],
                    expanded_for_blur(bounds, &sigmas, width, height),
                );
            }
        }
        let direct_mask = direct_mask_partition(plan, layer, &chain, replay, width, height)?;
        layers.push(LayerEffectPlan {
            chain,
            opened_runs,
            direct_mask,
        });
    }
    Ok((layers, runs))
}

// Corresponds to PropertyTreeManager's conditional kBlendModeDstIn surface:
// one actual layer with no child surface can draw its tile quads directly.
// Reuse the existing tile sampler, retaining its ownership and AA convention.
fn direct_mask_partition(
    plan: &layer_tile::FramePlan,
    layer: &layer_tile::LayerPlan,
    chain: &[Scope],
    replay: &LayerReplay<'_>,
    width: u32,
    height: u32,
) -> io::Result<Option<Vec<SurfaceBounds>>> {
    let effect = &layer.properties.effect;
    if !effect.is_mask
        || effect.opacity != 1.0
        || !matches!(chain.last(), Some(Scope::DstIn(node)) if node.lifecycle.same_node(&effect.lifecycle))
        || plan
            .layers
            .iter()
            .filter(|candidate| {
                let mut node = Some(&candidate.properties.effect);
                while let Some(ancestor) = node {
                    if ancestor.lifecycle.same_node(&effect.lifecycle) {
                        return true;
                    }
                    node = ancestor.parent.as_ref();
                }
                false
            })
            .count()
            != 1
    {
        return Ok(None);
    }
    let composition = replay
        .composition(layer.id)
        .ok_or_else(|| io::Error::other("mask layer has no composition"))?;
    if composition.white_backing {
        return Ok(None);
    }
    layer_tile_partition(layer, composition, width, height)
}
fn layer_tile_partition(
    layer: &layer_tile::LayerPlan,
    composition: LayerComposition,
    width: u32,
    height: u32,
) -> io::Result<Option<Vec<SurfaceBounds>>> {
    let mut rectangles = Vec::with_capacity(layer.tiles.len());
    for tile in &layer.tiles {
        if let Some((bounds, _, _)) = tile_composition_geometry(tile, composition, width, height)? {
            rectangles.push(bounds);
        }
    }
    rectangles.sort_unstable_by_key(|b| (b.top, b.left, b.bottom, b.right));
    // Prove a disjoint row grid in one sorted scan. Conservatively reject any
    // staggered band, even if a more general rectangle intersection could pass.
    let mut band: Option<SurfaceBounds> = None;
    for &rect in &rectangles {
        if let Some(previous) = band {
            if rect.top == previous.top {
                if rect.bottom != previous.bottom || rect.left < previous.right {
                    return Ok(None);
                }
            } else if rect.top < previous.bottom {
                return Ok(None);
            }
        }
        band = Some(rect);
    }
    Ok(Some(rectangles))
}

fn partition_covers_viewport(rectangles: &[SurfaceBounds], width: u32, height: u32) -> bool {
    let Some(first) = rectangles.first() else {
        return false;
    };
    if first.top != 0 {
        return false;
    }
    let (mut top, mut bottom, mut right) = (first.top, first.bottom, 0);
    for rect in rectangles {
        if rect.top != top {
            if right != width as usize || rect.top != bottom {
                return false;
            }
            top = rect.top;
            bottom = rect.bottom;
            right = 0;
        }
        if rect.bottom != bottom || rect.left != right {
            return false;
        }
        right = rect.right;
    }
    right == width as usize && bottom == height as usize
}

impl CachedTile {
    fn matches(&self, tile: &TilePlacement, composition: LayerComposition) -> bool {
        self.generation == tile.generation
            && self.pixel_size == tile.pixel_size
            && self.raster_scale == tile.raster_scale
            && self.white_backing == composition.white_backing
    }

    fn matches_task(&self, task: &RasterTask, composition: LayerComposition) -> bool {
        self.generation == task.generation
            && self.pixel_size == task.pixel_size
            && self.raster_scale == task.raster_scale
            && self.white_backing == composition.white_backing
    }

    fn from_product(
        frame_id: u64,
        task: &RasterTask,
        composition: LayerComposition,
        rgba: Vec<u8>,
        bgra: Vec<u32>,
        row_support: TileRowSupport,
    ) -> Self {
        Self {
            raster_frame: frame_id,
            generation: task.generation,
            pixel_size: task.pixel_size,
            raster_scale: task.raster_scale,
            white_backing: composition.white_backing,
            rgba,
            bgra,
            row_support,
        }
    }
}

#[inline]
fn compact_solid_color(pixels: &CachedTile) -> Option<u32> {
    match pixels.row_support.draw_mode {
        layer_solid::TileDrawMode::SolidColor { premul_rgba }
            if pixels.rgba.is_empty() && pixels.bgra.is_empty() =>
        {
            Some(premul_rgba)
        }
        _ => None,
    }
}

fn valid_tile_storage(pixels: &CachedTile, pixel_size: (u32, u32)) -> io::Result<bool> {
    let bytes = tile_bytes(pixel_size)?;
    Ok(
        (pixels.rgba.len() == bytes && pixels.bgra.len() == bytes / 4)
            || compact_solid_color(pixels).is_some(),
    )
}

fn valid_product_storage(product: &TileProduct) -> io::Result<bool> {
    let bytes = tile_bytes(product.task.pixel_size)?;
    let compact_solid = matches!(
        product.row_support.draw_mode,
        layer_solid::TileDrawMode::SolidColor { .. }
    ) && product.rgba.is_empty()
        && product.bgra.is_empty();
    Ok((product.rgba.len() == bytes && product.bgra.len() == bytes / 4) || compact_solid)
}

pub(crate) struct LayerRaster {
    resource_owner: Option<TileResourceOwner>,
    raster_committed: bool,
    prepared_frame_id: Option<u64>,
    prepared_damage_pixels: usize,
    pixels: HashMap<TileId, CachedTile>,
    halo_scratch: Vec<u8>,
    worker_pool: Option<TileWorkerPool>,
    prepaint_worker_pool: Option<TileWorkerPool>,
    pending_prepaint: Option<PendingPrepaint>,
    effect_scratch: Vec<EffectScratch>,
    clip_masks: ClipMaskCache,
    stats: LayerTileRasterStats,
}

struct PendingPrepaint {
    owner: TileResourceOwner,
    tasks: Vec<(TileId, u64)>,
    pending: PendingTiles,
}

impl Default for LayerRaster {
    fn default() -> Self {
        Self {
            resource_owner: None,
            raster_committed: false,
            prepared_frame_id: None,
            prepared_damage_pixels: 0,
            pixels: HashMap::new(),
            halo_scratch: Vec::new(),
            worker_pool: None,
            prepaint_worker_pool: None,
            pending_prepaint: None,
            effect_scratch: Vec::new(),
            clip_masks: ClipMaskCache::default(),
            stats: LayerTileRasterStats::default(),
        }
    }
}

impl Drop for LayerRaster {
    fn drop(&mut self) {
        // The persistent engine can outlive this backend. Its ready tiles must
        // not refer to resources which disappear with the renderer.
        if let Some(owner) = &self.resource_owner {
            owner.invalidate();
        }
    }
}

impl LayerRaster {
    pub(crate) fn stats(&self) -> LayerTileRasterStats {
        self.stats
    }

    fn publish_prepaint(
        &mut self,
        owner: &TileResourceOwner,
        products: Vec<TileProduct>,
    ) -> io::Result<()> {
        let mut published = 0usize;
        for product in products {
            if !valid_product_storage(&product)?
                || product.row_support.len() != product.task.pixel_size.1 as usize
            {
                return Err(io::Error::other(
                    "background raster returned invalid tile storage",
                ));
            }
            if owner.raster_complete(product.task.tile_id, product.task.generation) {
                self.pixels.insert(
                    product.task.tile_id,
                    CachedTile::from_product(
                        0,
                        &product.task,
                        product.composition,
                        product.rgba,
                        product.bgra,
                        product.row_support,
                    ),
                );
                published += 1;
            }
        }
        if published != 0 {
            browser_tracing::instant(
                "raster",
                "RasterPrepaintPublished",
                &[("tiles", published as f64)],
            );
        }
        Ok(())
    }

    fn collect_prepaint(&mut self, required: &[RasterTask]) -> io::Result<()> {
        let Some(mut batch) = self.pending_prepaint.take() else {
            return Ok(());
        };
        let needed_now = required.iter().any(|task| {
            batch
                .tasks
                .iter()
                .any(|&(id, generation)| id == task.tile_id && generation == task.generation)
        });
        if needed_now {
            let products = batch.pending.finish()?;
            return self.publish_prepaint(&batch.owner, products);
        }
        match batch.pending.try_finish() {
            None => {
                self.pending_prepaint = Some(batch);
                Ok(())
            }
            Some(Ok(products)) => self.publish_prepaint(&batch.owner, products),
            Some(Err(_)) => {
                // SOON work is speculative. Drop a failed lane and let the
                // planner offer the still-unready tile again; visible NOW
                // resources and this frame remain valid.
                self.prepaint_worker_pool = None;
                Ok(())
            }
        }
    }

    fn schedule_prepaint(
        &mut self,
        plan: &FramePlan,
        replay: &mut LayerReplay<'_>,
        draw_modes: &[layer_solid::TileDrawMode],
        first_task: usize,
        clip_limit: usize,
    ) -> io::Result<usize> {
        if first_task >= plan.tasks.len() || self.pending_prepaint.is_some() {
            return Ok(0);
        }
        let mut unresolved = Vec::new();
        for (plan_index, (task, mode)) in plan
            .tasks
            .iter()
            .zip(draw_modes)
            .enumerate()
            .skip(first_task)
        {
            let composition = replay
                .composition(task.layer_id)
                .ok_or_else(|| io::Error::other("prepaint task has no layer composition"))?;
            if self
                .pixels
                .get(&task.tile_id)
                .is_some_and(|pixels| pixels.matches_task(task, composition))
            {
                continue;
            }
            unresolved.push((plan_index, task, *mode));
        }
        if unresolved.is_empty() {
            if !plan.DidRasterizeTasks(&plan.tasks[first_task..]) {
                return Err(io::Error::other(
                    "completed prepaint no longer matches current tile generations",
                ));
            }
            return Ok(0);
        }
        if self.prepaint_worker_pool.is_none() {
            self.prepaint_worker_pool = Some(TileWorkerPool::new_prepaint(clip_limit)?);
        }
        let mut jobs = Vec::with_capacity(unresolved.len());
        let mut task_keys = Vec::with_capacity(unresolved.len());
        for (plan_index, task, mode) in unresolved {
            let layer = replay
                .prepaint_layer(task)
                .ok_or_else(|| io::Error::other("prepaint task has incomplete prepared records"))?;
            let reuse = task
                .previous_tile_id
                .and_then(|id| self.pixels.remove(&id))
                .map(|pixels| pixels.rgba)
                .unwrap_or_default();
            let solid_color = match mode {
                layer_solid::TileDrawMode::SolidColor { premul_rgba } => Some(premul_rgba),
                layer_solid::TileDrawMode::Resource => None,
            };
            task_keys.push((task.tile_id, task.generation));
            jobs.push(TileJob {
                plan_index,
                task: task.clone(),
                layer,
                reuse,
                solid_color,
            });
        }
        let scheduled = jobs.len();
        let owner = plan
            .ResourceOwner()
            .ok_or_else(|| io::Error::other("prepaint plan has no tile resource owner"))?;
        let pending = self
            .prepaint_worker_pool
            .as_mut()
            .expect("initialized prepaint pool")
            .submit(plan.GetPaintArtifact().resources.clone(), jobs);
        self.pending_prepaint = Some(PendingPrepaint {
            owner,
            tasks: task_keys,
            pending,
        });
        Ok(scheduled)
    }

    fn reset_tiles(&mut self) {
        if let Some(owner) = &self.resource_owner {
            owner.invalidate();
        }
        self.pending_prepaint = None;
        self.raster_committed = false;
        self.prepared_frame_id = None;
        self.prepared_damage_pixels = 0;
        self.pixels.clear();
        self.clip_masks = ClipMaskCache::default();
    }

    pub(crate) fn invalidate(&mut self) {
        self.reset_tiles();
        self.stats = LayerTileRasterStats::default();
    }

    fn unsupported(&mut self, reason: &'static str) -> io::Result<RasterUpdate> {
        self.reset_tiles();
        self.stats.resident_bytes = 0;
        self.stats.raster_tasks = 0;
        self.stats.reused_tiles = 0;
        self.stats.unsupported_reason = Some(reason);
        Err(io::Error::new(io::ErrorKind::Unsupported, reason))
    }

    fn finish_effect_group(
        &mut self,
        groups: &mut Vec<EffectSurface>,
        target: &mut [u32],
        frame_view: TargetView,
        target_format: skia::PixelFormat,
    ) {
        #[cfg(feature = "compose_work_profile")]
        layer_compose_profile::class(5);
        let mut group = groups.pop().expect("active effect group");
        // Direct quads already performed this Mask effect, including exterior
        // clearing. Keep scope ordering/identity without allocating a source.
        if group.direct_dst_in || group.direct_clip {
            return;
        }
        if !group.blur_sigmas.is_empty() && !group.pixels.is_empty() {
            let width = group.view.stride;
            let height = group.pixels.len() / width;
            // Effect surfaces store native little-endian RGBA words, the same
            // premultiplied byte layout consumed by SkBlurEngine.
            let bytes = unsafe {
                std::slice::from_raw_parts_mut(
                    group.pixels.as_mut_ptr().cast::<u8>(),
                    group.pixels.len() * std::mem::size_of::<u32>(),
                )
            };
            for &(sigma_x, sigma_y) in &group.blur_sigmas {
                let ok = skia::cpu::layer_filters::blur_rgba(
                    bytes,
                    width as u32,
                    height as u32,
                    sigma_x,
                    sigma_y,
                );
                debug_assert!(ok);
                if let Some(mut touched) = group.touched {
                    let x = (sigma_x * 3.0).ceil() as usize + 1;
                    let y = (sigma_y * 3.0).ceil() as usize + 1;
                    touched.left = touched.left.saturating_sub(x).max(group.view.origin.0);
                    touched.top = touched.top.saturating_sub(y).max(group.view.origin.1);
                    touched.right = touched
                        .right
                        .saturating_add(x)
                        .min(group.view.origin.0 + width);
                    touched.bottom = touched
                        .bottom
                        .saturating_add(y)
                        .min(group.view.origin.1 + height);
                    group.touched = Some(touched);
                }
            }
        }
        if let Scope::DstIn(effect) = &group.scope {
            let parent = groups
                .last_mut()
                .expect("preflight validated isolated mask destination");
            let expected = effect
                .parent
                .as_ref()
                .expect("preflight validated mask content parent");
            debug_assert!(matches!(&parent.scope, Scope::Effect(actual)
                if actual.lifecycle.same_node(&expected.lifecycle)));
            parent.touched = dst_in_effect_surface(
                &mut parent.pixels,
                &group.pixels,
                parent.touched,
                group.touched,
                parent.view,
                group.view,
                effect.opacity,
            );
        } else if let Some(parent) = groups.last_mut() {
            let written = composite_effect_surface(
                &mut parent.pixels,
                &group.pixels,
                group.touched,
                parent.view,
                group.view,
                skia::PixelFormat::Rgba8888,
                group.scope.opacity(),
                group.coverage.as_deref(),
            );
            union_bounds(&mut parent.touched, written);
        } else {
            composite_effect_surface(
                target,
                &group.pixels,
                group.touched,
                frame_view,
                group.view,
                target_format,
                group.scope.opacity(),
                group.coverage.as_deref(),
            );
        }
        // Scratch contains no reusable rendered result: every acquisition is
        // transparent after clearing old writes. Bound retained capacity.
        let bytes = group.pixels.capacity().saturating_mul(4);
        let retained = self
            .effect_scratch
            .iter()
            .map(|p| p.pixels.capacity().saturating_mul(4))
            .fold(0usize, usize::saturating_add);
        if self.effect_scratch.len() < 4
            && bytes <= 32 * 1024 * 1024
            && retained <= 32 * 1024 * 1024 - bytes
        {
            self.effect_scratch.push(EffectScratch {
                pixels: group.pixels,
                touched: group.touched,
                view: group.view,
            });
        }
    }

    fn cleared_effect_surface(&mut self, expected: usize) -> Vec<u32> {
        if let Some(mut scratch) = self.effect_scratch.pop() {
            if scratch.pixels.len() == expected {
                // All writes were enclosed by touched. Every other pixel has
                // remained transparent since allocation; clear only old dirt.
                if let Some(bounds) = scratch.touched {
                    for y in bounds.top..bounds.bottom {
                        scratch.pixels[scratch.view.row_range(y, bounds.left, bounds.right)]
                            .fill(0);
                    }
                }
            } else {
                scratch.pixels.resize(expected, 0);
                scratch.pixels.fill(0);
            }
            scratch.pixels
        } else {
            vec![0; expected]
        }
    }

    /// Unsupported input is an error with its reason recorded in public stats.
    /// The plan already contains all metadata. Raster products become reusable
    /// after task validation; composition is preflighted before output writes.
    pub(crate) fn paint(
        &mut self,
        plan: &FramePlan,
        width: u32,
        height: u32,
        buffer: &mut [u32],
        format: skia::PixelFormat,
        row_stride: usize,
        image_cache: &mut RasterImageCache,
        clip_cache: &mut skia::RasterClipProductCache,
    ) -> io::Result<RasterUpdate> {
        self.prepare(plan, width, height, image_cache, clip_cache)?;
        self.compose(
            plan,
            width,
            height,
            buffer,
            format,
            row_stride,
            image_cache,
            clip_cache,
        )
    }

    pub(crate) fn prepare(
        &mut self,
        plan: &FramePlan,
        width: u32,
        height: u32,
        image_cache: &mut RasterImageCache,
        clip_cache: &mut skia::RasterClipProductCache,
    ) -> io::Result<()> {
        let mut render_trace = browser_tracing::span("render", "LayerTilePrepare");
        render_trace.set("frame_id", plan.frame_id as f64);
        if plan.unsupported.is_none() && !plan.IsCurrentFrame() {
            return Err(io::Error::other(
                "FramePlan is no longer current or its engine was released",
            ));
        }
        if self.prepared_frame_id == Some(plan.frame_id) {
            render_trace.set("no_work", 1.0);
            return Ok(());
        }
        let owner = plan
            .ResourceOwner()
            .ok_or_else(|| io::Error::other("FramePlan has no tile resource owner"))?;
        if !self
            .resource_owner
            .as_ref()
            .is_some_and(|previous| previous.same_engine(&owner))
        {
            self.reset_tiles();
            self.resource_owner = Some(owner);
        }
        self.raster_committed = false;
        let result = self
            .paint_frame(
                plan,
                width,
                height,
                &mut [],
                skia::PixelFormat::Bgra8888,
                width as usize,
                image_cache,
                clip_cache,
                false,
            )
            .map(|_| ());
        if let Err(error) = &result {
            if !self.raster_committed
                && (error.kind() != io::ErrorKind::Unsupported
                    || self.stats.unsupported_reason.is_none())
            {
                self.invalidate();
            }
        }
        render_trace.set("succeeded", if result.is_ok() { 1.0 } else { 0.0 });
        if result.is_ok() {
            render_trace.set("layers", self.stats.layers as f64);
            render_trace.set("tiles", self.stats.tiles as f64);
            render_trace.set("raster_tasks", self.stats.raster_tasks as f64);
            render_trace.set("resident_bytes", self.stats.resident_bytes as f64);
            render_trace.set("damage_pixels", self.prepared_damage_pixels as f64);
        }
        result
    }

    pub(crate) fn compose(
        &mut self,
        plan: &FramePlan,
        width: u32,
        height: u32,
        buffer: &mut [u32],
        format: skia::PixelFormat,
        row_stride: usize,
        image_cache: &mut RasterImageCache,
        clip_cache: &mut skia::RasterClipProductCache,
    ) -> io::Result<RasterUpdate> {
        let mut render_trace = browser_tracing::span("render", "LayerTilePaint");
        render_trace.set("frame_id", plan.frame_id as f64);
        if self.prepared_frame_id != Some(plan.frame_id)
            || (plan.unsupported.is_none() && !plan.IsCurrentFrame())
        {
            return Err(io::Error::other(
                "FramePlan raster resources are not prepared",
            ));
        }
        let result = self.paint_frame(
            plan,
            width,
            height,
            buffer,
            format,
            row_stride,
            image_cache,
            clip_cache,
            true,
        );
        render_trace.set("succeeded", if result.is_ok() { 1.0 } else { 0.0 });
        if let Ok(update) = &result {
            render_trace.set("layers", self.stats.layers as f64);
            render_trace.set("tiles", self.stats.tiles as f64);
            render_trace.set("raster_tasks", self.stats.raster_tasks as f64);
            render_trace.set("reused_tiles", self.stats.reused_tiles as f64);
            render_trace.set("resident_bytes", self.stats.resident_bytes as f64);
            render_trace.set("damage_pixels", update.damage_pixels as f64);
        }
        result
    }

    fn paint_frame(
        &mut self,
        plan: &FramePlan,
        width: u32,
        height: u32,
        buffer: &mut [u32],
        format: skia::PixelFormat,
        row_stride: usize,
        image_cache: &mut RasterImageCache,
        clip_cache: &mut skia::RasterClipProductCache,
        compose: bool,
    ) -> io::Result<RasterUpdate> {
        let list = plan.GetPaintArtifact();
        let scale = plan.config.raster_scale;
        let timing = (std::env::var_os("LAYOUTNG_LAYER_PROFILE").is_some()
            || std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some())
        .then(std::time::Instant::now);
        let replay_trace = browser_tracing::span("render", "PrepareLayerReplay");
        if width == 0 || !scale.is_finite() || scale <= 0.0 || !cfg!(target_endian = "little") {
            return Err(io::Error::other(
                "unsupported layer target dimensions or byte order",
            ));
        }
        if compose {
            let expected = row_stride
                .checked_mul(height as usize)
                .filter(|&n| n > 0 && n <= isize::MAX as usize / 4)
                .ok_or_else(|| io::Error::other("invalid layer target storage extent"))?;
            if row_stride < width as usize
                || row_stride > u32::MAX as usize
                || buffer.len() != expected
            {
                return Err(io::Error::other(
                    "unsupported layer target dimensions or byte order",
                ));
            }
        }
        if plan.config.viewport
            != (PaintRect {
                x: 0.0,
                y: 0.0,
                width: width as f64 / scale,
                height: height as f64 / scale,
            })
        {
            return Err(io::Error::other(
                "FramePlan viewport does not match the raster target",
            ));
        }
        if !compose {
            self.stats = LayerTileRasterStats {
                layers: plan.layers.len(),
                tiles: plan.layers.iter().map(|layer| layer.tiles.len()).sum(),
                ..LayerTileRasterStats::default()
            };
        }
        if let Some(reason) = plan.unsupported {
            return self.unsupported(unsupported_reason(reason));
        }
        let required_tasks = plan.RequiredRasterTaskCount();
        self.collect_prepaint(&plan.tasks[..required_tasks.min(plan.tasks.len())])?;
        let mut replay = match LayerReplay::new(plan) {
            Ok(replay) => replay,
            Err(reason) => return self.unsupported(reason.reason()),
        };
        // Fractional phase is a backend capability decision, never a failed
        // raster transaction. Reject it before any task executes.
        if plan
            .layers
            .iter()
            .flat_map(|layer| &layer.tiles)
            .any(|tile| {
                device_position(tile.tile_rect.x * tile.raster_scale).is_err()
                    || device_position(tile.tile_rect.y * tile.raster_scale).is_err()
            })
        {
            return self.unsupported("fractional-pixel-placement");
        }
        // Keep unchanged cached buffers in place, allocating only dirty products.
        let replay_done = timing.map(|start| start.elapsed());
        drop(replay_trace);
        if !compose {
            let mut raster_trace = browser_tracing::span("raster", "RasterTiles");
            // RasterTiles measures work which blocks this pending tree. SOON
            // tasks are published by RasterWorkerBatch after this span ends.
            raster_trace.set("raster_tasks", required_tasks as f64);
            raster_trace.set("planned_raster_tasks", plan.tasks.len() as f64);
            raster_trace.set("visible_raster_tasks", required_tasks as f64);
            raster_trace.set(
                "prepaint_raster_tasks",
                plan.tasks.len().saturating_sub(required_tasks) as f64,
            );
            let mut staged = HashMap::with_capacity(plan.tasks.len());
            let mut damage_pixels = 0usize;
            let profile_tiles = std::env::var_os("BROWSER_PROFILE_TILES").is_some();
            let mut tile_calls = std::time::Duration::ZERO;
            let mut tile_support = std::time::Duration::ZERO;
            // TileManager performs bounded PaintOp solid-color analysis before
            // raster. Keep its result with the corresponding actual generation;
            // never derive this mode by scanning the resulting pixel buffer.
            let draw_modes: Vec<_> = plan
                .tasks
                .iter()
                .map(|task| match replay.solid_analysis_commands(task) {
                    Ok(records) => layer_solid_analysis::analyze(task, records),
                    Err(_) => layer_solid::TileDrawMode::Resource,
                })
                .collect();
            if required_tasks == 0 && !plan.tasks.is_empty() {
                let clip_limit =
                    (clip_cache.accounting().configured_byte_limit / 2).min(1024 * 1024);
                let scheduled =
                    self.schedule_prepaint(plan, &mut replay, &draw_modes, 0, clip_limit)?;
                raster_trace.set("async_prepaint_tasks", scheduled as f64);
                for tile in &plan.retired_tiles {
                    self.pixels.remove(tile);
                }
                plan.DidReleaseTileResources();
                self.raster_committed = true;
                self.prepared_frame_id = Some(plan.frame_id);
                self.prepared_damage_pixels = 0;
                self.stats.raster_tasks = 0;
                self.stats.resident_bytes = self
                    .pixels
                    .values()
                    .map(|tile| tile.rgba.len() + tile.bgra.len() * std::mem::size_of::<u32>())
                    .sum();
                raster_trace.set("worker_tasks", 0.0);
                raster_trace.set("damage_pixels", 0.0);
                return Ok(RasterUpdate {
                    mode: "layer-tiles-prepared",
                    reason: "scheduled-prepaint-tiles",
                    damage_pixels: 0,
                    copied_bytes: 0,
                });
            }
            // Every prepared record and resource payload is immutable. Mutable
            // Canvas/image/clip caches stay worker-local.
            let foreground_tasks = &plan.tasks[..required_tasks.min(plan.tasks.len())];
            let worker_layers: Vec<_> = foreground_tasks
                .iter()
                .map(|task| replay.worker_layer(task))
                .collect();
            let eligible = worker_layers.iter().filter(|layer| layer.is_some()).count();
            if self.worker_pool.is_none() && eligible >= 4 {
                let clip_limit =
                    (clip_cache.accounting().configured_byte_limit / 2).min(1024 * 1024);
                self.worker_pool = Some(TileWorkerPool::new(clip_limit)?);
            }
            let parallel = eligible >= 2
                && self
                    .worker_pool
                    .as_ref()
                    .is_some_and(TileWorkerPool::available);
            let mut worker_jobs = Vec::new();
            let mut caller_jobs = Vec::new();
            let mut products = Vec::with_capacity(foreground_tasks.len());
            for (plan_index, (task, layer)) in
                foreground_tasks.iter().zip(worker_layers).enumerate()
            {
                // The old identity is already unpublished; transfer its allocation
                // exactly once to the task producing this plan's new generation.
                let reuse = task
                    .previous_tile_id
                    .and_then(|id| self.pixels.remove(&id))
                    .map(|pixels| pixels.rgba)
                    .unwrap_or_default();
                if let Some(layer) = layer.filter(|_| parallel) {
                    worker_jobs.push(TileJob {
                        plan_index,
                        task: task.clone(),
                        layer,
                        reuse,
                        solid_color: match draw_modes[plan_index] {
                            layer_solid::TileDrawMode::SolidColor { premul_rgba } => {
                                Some(premul_rgba)
                            }
                            layer_solid::TileDrawMode::Resource => None,
                        },
                    });
                } else if let layer_solid::TileDrawMode::SolidColor { premul_rgba } =
                    draw_modes[plan_index]
                {
                    let composition = replay
                        .composition(task.layer_id)
                        .ok_or_else(|| io::Error::other("solid task has no layer composition"))?;
                    products.push(layer_solid::raster_product(
                        plan_index,
                        task,
                        composition,
                        premul_rgba,
                        reuse,
                        profile_tiles,
                    )?);
                } else {
                    caller_jobs.push((plan_index, task, reuse));
                }
            }
            let worker_tasks = worker_jobs.len();
            let pending = if worker_jobs.is_empty() {
                None
            } else {
                Some(
                    self.worker_pool
                        .as_mut()
                        .expect("initialized raster pool")
                        .submit(list.resources.clone(), worker_jobs),
                )
            };
            let has_caller_jobs = !caller_jobs.is_empty();
            if has_caller_jobs {
                replay.set_scratch(std::mem::take(&mut self.halo_scratch));
            }
            let mut caller_error = None;
            for (plan_index, task, reuse) in caller_jobs {
                let mut trace = browser_tracing::span("raster", "RasterTile");
                trace.set("tile_id", task.tile_id.0 as f64);
                trace.set("layer_id", task.layer_id.0 as f64);
                let result = (|| -> io::Result<TileProduct> {
                    let composition = replay
                        .composition(task.layer_id)
                        .ok_or_else(|| io::Error::other("raster task has no layer composition"))?;
                    let started = profile_tiles.then(std::time::Instant::now);
                    let rgba =
                        replay.raster_tile_with_pixels(task, image_cache, clip_cache, reuse)?;
                    let call_time = started.map_or(std::time::Duration::ZERO, |s| s.elapsed());
                    let started = profile_tiles.then(std::time::Instant::now);
                    let row_support = tile_row_support(
                        &rgba,
                        task.pixel_size.0 as usize,
                        composition.white_backing,
                    );
                    let bgra = cached_bgra(&rgba);
                    Ok(TileProduct {
                        plan_index,
                        task: task.clone(),
                        composition,
                        rgba,
                        bgra,
                        row_support,
                        call_time,
                        support_time: started.map_or(std::time::Duration::ZERO, |s| s.elapsed()),
                    })
                })();
                match result {
                    Ok(product) => products.push(product),
                    Err(error) => {
                        caller_error = Some(error);
                        break;
                    }
                }
            }
            if has_caller_jobs {
                self.halo_scratch = replay.take_scratch();
            }
            // Always drain background work before returning an error. Tiles become
            // publishable only after all lanes' products match the original plan.
            if let Some(pending) = pending {
                match pending.finish() {
                    Ok(mut ready) => products.append(&mut ready),
                    Err(error) => {
                        caller_error.get_or_insert(error);
                    }
                }
            }
            if let Some(error) = caller_error {
                self.worker_pool = None;
                return Err(error);
            }
            products.sort_unstable_by_key(|product| product.plan_index);
            if products.len() != foreground_tasks.len() {
                return Err(io::Error::other(
                    "raster lanes returned an incomplete task plan",
                ));
            }
            for (index, (task, mut product)) in foreground_tasks.iter().zip(products).enumerate() {
                if product.plan_index != index
                    || product.task.tile_id != task.tile_id
                    || product.task.layer_id != task.layer_id
                    || product.task.generation != task.generation
                    || product.task.tile_rect != task.tile_rect
                    || product.task.raster_scale != task.raster_scale
                    || product.task.pixel_size != task.pixel_size
                    || product.task.record_indices != task.record_indices
                    || replay.composition(task.layer_id) != Some(product.composition)
                {
                    return Err(io::Error::other(
                        "raster lane product differs from its submitted tile generation",
                    ));
                }
                let expected_bytes = tile_bytes(task.pixel_size)?;
                if !valid_product_storage(&product)?
                    || product.row_support.len() != task.pixel_size.1 as usize
                {
                    return Err(io::Error::other(
                        "raster task returned invalid pixel storage",
                    ));
                }
                damage_pixels = damage_pixels.saturating_add(expected_bytes / 4);
                tile_calls += product.call_time;
                tile_support += product.support_time;
                product.row_support.draw_mode = draw_modes[index];
                if staged
                    .insert(
                        task.tile_id,
                        CachedTile::from_product(
                            plan.frame_id,
                            task,
                            product.composition,
                            product.rgba,
                            product.bgra,
                            product.row_support,
                        ),
                    )
                    .is_some()
                {
                    return Err(io::Error::other("duplicate raster task tile identity"));
                }
            }
            if profile_tiles {
                eprintln!("tile-raster-overhead tasks={} worker_tasks={} calls_ms={:.3} row_support_ms={:.3}",
                foreground_tasks.len(), worker_tasks, tile_calls.as_secs_f64()*1000.0, tile_support.as_secs_f64()*1000.0);
            }
            if !plan.DidRasterizeTasks(foreground_tasks) {
                return Err(io::Error::other(
                    "raster completion no longer matches planned generation",
                ));
            }
            // Publish successful tile resources before composition, as cc does.
            // A later composition error cannot discard valid raster products.
            self.pixels.extend(std::mem::take(&mut staged));
            let clip_limit = (clip_cache.accounting().configured_byte_limit / 2).min(1024 * 1024);
            let scheduled = self.schedule_prepaint(
                plan,
                &mut replay,
                &draw_modes,
                foreground_tasks.len(),
                clip_limit,
            )?;
            for tile in &plan.retired_tiles {
                self.pixels.remove(tile);
            }
            plan.DidReleaseTileResources();
            self.raster_committed = true;
            let raster_done = timing.map(|start| start.elapsed());
            raster_trace.set("worker_tasks", worker_tasks as f64);
            raster_trace.set("async_prepaint_tasks", scheduled as f64);
            raster_trace.set("damage_pixels", damage_pixels as f64);
            drop(raster_trace);

            self.prepared_frame_id = Some(plan.frame_id);
            self.prepared_damage_pixels = damage_pixels;
            self.stats.raster_tasks = foreground_tasks.len();
            self.stats.resident_bytes = self
                .pixels
                .values()
                .map(|tile| tile.rgba.len() + tile.bgra.len() * std::mem::size_of::<u32>())
                .sum();
            return Ok(RasterUpdate {
                mode: "layer-tiles-prepared",
                reason: if plan.tasks.is_empty() {
                    "reused-visible-tiles"
                } else {
                    "rasterized-dirty-tiles"
                },
                damage_pixels,
                copied_bytes: 0,
            });
        }

        let staged: HashMap<TileId, CachedTile> = HashMap::new();
        let damage_pixels = self.prepared_damage_pixels;
        let raster_done = replay_done;

        let observe_costs = timing.is_some() || browser_tracing::enabled();
        let mut compose_trace = browser_tracing::span("compose", "ComposeTiles");
        let mut preflight_trace = browser_tracing::span("compose", "ComposePreflight");
        let white = if format == skia::PixelFormat::Bgrx8888 {
            0x00ff_ffff
        } else {
            u32::MAX
        };
        let (effect_plans, effect_bounds) =
            effect_allocation_plan(plan, &replay, width, height, scale)?;
        preflight_trace.set("effect_runs", effect_bounds.len() as f64);
        let direct_clips =
            layer_direct_clip::eligible_runs(plan, &effect_plans, &replay, width, height)?;
        let visible_rects = layer_occlusion::visible_rects(
            plan,
            &effect_plans,
            &direct_clips,
            &replay,
            &staged,
            &self.pixels,
            width,
            height,
        )?;
        let clip_observations = self.clip_masks.observations();
        let prepared = match layer_composition_preflight::prepare(
            plan,
            &effect_plans,
            &effect_bounds,
            &direct_clips,
            &visible_rects,
            &replay,
            &self.pixels,
            &mut self.clip_masks,
            width,
            height,
            scale,
            observe_costs,
        ) {
            Ok(prepared) => prepared,
            Err(error)
                if error.kind() == io::ErrorKind::Unsupported
                    && error.to_string() == "non-contiguous-opacity-effect-group" =>
            {
                return self.unsupported("non-contiguous-opacity-effect-group");
            }
            Err(error) => return Err(error),
        };
        let clip_after = self.clip_masks.observations();
        preflight_trace.set("clip_prepare_ms", prepared.clip_time.as_secs_f64() * 1000.0);
        for (index, name) in [
            "clip_cache_hits",
            "clip_relocations",
            "clip_rasters",
            "clip_raster_pixels",
        ]
        .into_iter()
        .enumerate()
        {
            preflight_trace.set(
                name,
                clip_after[index].saturating_sub(clip_observations[index]) as f64,
            );
        }
        let frame_view = TargetView {
            origin: (0, 0),
            stride: row_stride,
        };
        // SoftwareRenderer leaves an opaque root framebuffer uncleared. Here
        // skip only when the actual first white-backed layer proves every
        // visible pixel will be overwritten, including fractional neighbors.
        let root_clear_skipped = if let (Some(layer), Some(effects)) =
            (plan.layers.first(), effect_plans.first())
        {
            let composition = replay
                .composition(layer.id)
                .ok_or_else(|| io::Error::other("first paint layer has no composition"))?;
            composition.white_backing
                && effects.chain.is_empty()
                && layer.tiles.iter().all(|tile| {
                    staged
                        .get(&tile.tile_id)
                        .or_else(|| self.pixels.get(&tile.tile_id))
                        .is_some_and(|pixels| {
                            pixels.matches(tile, composition) && pixels.row_support.opaque
                        })
                })
                && layer_tile_partition(layer, composition, width, height)?
                    .is_some_and(|rectangles| partition_covers_viewport(&rectangles, width, height))
        } else {
            false
        };
        drop(preflight_trace);
        let mut draw_trace = browser_tracing::span("compose", "ComposeDraw");
        let (mut integral_tiles, mut fractional_tiles) = (0usize, 0usize);
        // viz::SoftwareRenderer::BeginDrawingRenderPass draws the root pass
        // into OutputDevice::BeginPaint's canvas. All fallible admission above
        // precedes this point; keep only tile and transient effect resources.
        if root_clear_skipped {
            // Padding is observable to callers but no tile writes it.
            if row_stride > width as usize {
                for row in buffer.chunks_exact_mut(row_stride) {
                    row[width as usize..].fill(white);
                }
            }
        } else {
            buffer.fill(white);
        }
        let mut reused = 0usize;
        let mut groups: Vec<EffectSurface> = Vec::new();
        let mut compose_costs = observe_costs.then(|| ComposeCosts {
            clip: prepared.clip_time,
            ..Default::default()
        });
        #[cfg(feature = "compose_work_profile")]
        layer_compose_profile::begin();
        for (((layer, effect_plan), layer_visible), direct_clip) in plan
            .layers
            .iter()
            .zip(effect_plans)
            .zip(visible_rects)
            .zip(direct_clips)
        {
            if layer.properties.effect.is_mask {
                if let Some(cost) = &mut compose_costs {
                    cost.mask_layers += 1;
                    cost.mask_raster_tasks += plan
                        .tasks
                        .iter()
                        .filter(|task| task.layer_id == layer.id)
                        .count();
                }
            }
            // Keep the resident layer's identity, but a layer with no visible
            // tiles contributes no pixels or group coverage. Do not open a
            // full-size opacity/clip surface for offscreen animation layers.
            if layer.tiles.is_empty() && !layer.properties.effect.is_mask {
                continue;
            }
            let direct_mask = effect_plan.direct_mask;
            let chain = effect_plan.chain;
            let common = groups
                .iter()
                .zip(&chain)
                .take_while(|(active, wanted)| active.scope.same_node(wanted))
                .count();
            debug_assert!(groups
                .iter()
                .zip(&chain)
                .take(common)
                .all(|(active, wanted)| active.scope.same_values(wanted)));
            if chain
                .iter()
                .any(|scope| matches!(scope, Scope::Effect(effect) if effect.opacity == 0.0))
            {
                if let Some(cost) = &mut compose_costs {
                    cost.skipped_zero += 1;
                }
                continue;
            }
            debug_assert_eq!(chain.len() - common, effect_plan.opened_runs.len());
            let close_start = compose_costs.as_ref().map(|_| std::time::Instant::now());
            while groups.len() > common {
                self.finish_effect_group(&mut groups, buffer, frame_view, format);
            }
            if let Some(cost) = &mut compose_costs {
                cost.close += close_start.unwrap().elapsed();
            }
            let open_start = compose_costs.as_ref().map(|_| std::time::Instant::now());
            for (scope, run) in chain.into_iter().skip(common).zip(effect_plan.opened_runs) {
                if let Some(cost) = &mut compose_costs {
                    cost.scopes += 1;
                }
                let bounds = effect_bounds[run].unwrap_or(SurfaceBounds {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                });
                let coverage = prepared.coverage[run].clone();
                let scratch_start = compose_costs.as_ref().map(|_| std::time::Instant::now());
                let view = TargetView {
                    origin: (bounds.left, bounds.top),
                    stride: bounds.right - bounds.left,
                };
                let len = view
                    .stride
                    .checked_mul(bounds.bottom - bounds.top)
                    .expect("preflight validated effect dimensions");
                let direct_dst_in = direct_mask.is_some() && matches!(&scope, Scope::DstIn(_));
                let direct_clip = direct_clip.is_some_and(|candidate| candidate.opening_run == run);
                let blur_sigmas = effect_blur_sigmas(&scope, scale)?;
                let pixels = if direct_dst_in || direct_clip {
                    Vec::new()
                } else {
                    self.cleared_effect_surface(len)
                };
                if let Some(cost) = &mut compose_costs {
                    cost.scratch += scratch_start.unwrap().elapsed();
                }
                groups.push(EffectSurface {
                    scope,
                    coverage,
                    direct_dst_in,
                    direct_clip,
                    pixels,
                    view,
                    touched: None,
                    blur_sigmas,
                });
            }
            if let Some(cost) = &mut compose_costs {
                cost.open += open_start.unwrap().elapsed();
            }
            if let Some(candidate) = direct_clip {
                debug_assert!(
                    candidate.scope_index + 1 == groups.len()
                        && groups.last().is_some_and(|g| g.direct_clip)
                );
                if let Some(cost) = &mut compose_costs {
                    cost.direct_clip_layers += 1;
                }
            }
            let composition = replay
                .composition(layer.id)
                .expect("preflight validated layer composition");
            let blit_start = compose_costs.as_ref().map(|_| std::time::Instant::now());
            if let Some(rectangles) = &direct_mask {
                let index = groups
                    .len()
                    .checked_sub(2)
                    .expect("preflight validated direct mask destination");
                let expected = layer
                    .properties
                    .effect
                    .parent
                    .as_ref()
                    .expect("preflight validated direct mask parent");
                debug_assert!(
                    groups.last().is_some_and(|group| group.direct_dst_in)
                        && matches!(&groups[index].scope, Scope::Effect(parent)
                        if parent.lifecycle.same_node(&expected.lifecycle))
                );
                let parent = &mut groups[index];
                parent.touched = clear_mask_exterior(
                    &mut parent.pixels,
                    parent.view,
                    parent.touched,
                    rectangles,
                );
                if let Some(cost) = &mut compose_costs {
                    cost.direct_mask_layers += 1;
                }
            }
            let needs_neighbors = composition.translation.0.fract() != 0.0
                || composition.translation.1.fract() != 0.0;
            let mut neighbors = HashMap::with_capacity(if needs_neighbors {
                layer.tiles.len()
            } else {
                0
            });
            for tile in layer.tiles.iter().filter(|_| needs_neighbors) {
                let pixels = staged
                    .get(&tile.tile_id)
                    .or_else(|| self.pixels.get(&tile.tile_id))
                    .expect("preflight validated resident tile");
                neighbors.insert(
                    (
                        device_position(tile.tile_rect.x * tile.raster_scale)
                            .expect("preflight validated tile x"),
                        device_position(tile.tile_rect.y * tile.raster_scale)
                            .expect("preflight validated tile y"),
                    ),
                    pixels,
                );
            }
            for (tile, draw_visible) in layer.tiles.iter().zip(layer_visible) {
                let origin = (
                    device_position(tile.tile_rect.x * tile.raster_scale)
                        .expect("preflight validated tile x"),
                    device_position(tile.tile_rect.y * tile.raster_scale)
                        .expect("preflight validated tile y"),
                );
                if let Some(cost) = &mut compose_costs {
                    if composition.translation.0.fract() == 0.0
                        && composition.translation.1.fract() == 0.0
                    {
                        cost.integral += 1;
                    } else {
                        cost.fractional += 1;
                        let x = origin.0 as f64 + composition.translation.0;
                        let y = origin.1 as f64 + composition.translation.1;
                        let phases = [x.ceil() - x, y.ceil() - y];
                        let exact = |scale: f64| {
                            phases.iter().all(|phase| {
                                let weight = phase * scale;
                                weight.is_finite() && weight == weight.round()
                            })
                        };
                        cost.fractional_phase_classes[if exact(256.0) {
                            0
                        } else if exact(65536.0) {
                            1
                        } else {
                            2
                        }] += 1;
                    }
                    cost.nominal_pixels += tile.pixel_size.0 as usize * tile.pixel_size.1 as usize;
                }
                let adjacent = [
                    neighbors
                        .get(&(origin.0 + i64::from(tile.pixel_size.0), origin.1))
                        .copied(),
                    neighbors
                        .get(&(origin.0, origin.1 + i64::from(tile.pixel_size.1)))
                        .copied(),
                    neighbors
                        .get(&(
                            origin.0 + i64::from(tile.pixel_size.0),
                            origin.1 + i64::from(tile.pixel_size.1),
                        ))
                        .copied(),
                ];
                let pixels = if let Some(pixels) = staged.get(&tile.tile_id) {
                    pixels
                } else {
                    let cached = self
                        .pixels
                        .get(&tile.tile_id)
                        .expect("preflight validated resident tile");
                    if cached.raster_frame != plan.frame_id {
                        reused += 1;
                        if layer.properties.effect.is_mask {
                            if let Some(cost) = &mut compose_costs {
                                cost.mask_reused_tiles += 1;
                            }
                        }
                    }
                    self.pixels
                        .get(&tile.tile_id)
                        .expect("preflight validated resident tile")
                };
                debug_assert!(pixels.matches(tile, composition));
                if let Some(cost) = &mut compose_costs {
                    if let Some((original, _, _)) =
                        tile_composition_geometry(tile, composition, width, height)
                            .expect("preflight validated tile geometry")
                    {
                        let original_area =
                            (original.right - original.left) * (original.bottom - original.top);
                        let visible_area =
                            draw_visible.map_or(0, |b| (b.right - b.left) * (b.bottom - b.top));
                        cost.occluded_pixels += original_area - visible_area;
                        if draw_visible.is_none() {
                            cost.occluded_tiles += 1;
                        }
                    }
                }
                // Keep resource/generation accounting even for hidden quads.
                // Mask quads retain their original partition and exterior clear.
                if draw_visible.is_none() {
                    continue;
                }
                // Count quads that reach a visible target, after occlusion and
                // empty direct-clip rejection. No per-pixel instrumentation.
                if direct_clip.is_none() || groups.last().is_some_and(|g| g.coverage.is_some()) {
                    if needs_neighbors {
                        fractional_tiles += 1;
                    } else {
                        integral_tiles += 1;
                    }
                }
                let tile_start = compose_costs.as_ref().map(|_| std::time::Instant::now());
                let origin_device = (
                    origin.0 as f64 + composition.translation.0,
                    origin.1 as f64 + composition.translation.1,
                );
                let phase = (
                    origin_device.0.ceil() - origin_device.0,
                    origin_device.1.ceil() - origin_device.1,
                );
                let tile_class = if direct_mask.is_some() {
                    3
                } else if layer_solid::color_for_sampling(pixels, adjacent, phase).is_some() {
                    4
                } else if composition.translation.0.fract() == 0.0
                    && composition.translation.1.fract() == 0.0
                {
                    0
                } else {
                    let horizontal = composition.translation.0.fract() != 0.0;
                    let vertical = composition.translation.1.fract() != 0.0;
                    let opaque = draw_visible
                        .is_some_and(|bounds| opaque_covers(composition, bounds))
                        || ((composition.white_backing || pixels.row_support.opaque)
                            && adjacent
                                .iter()
                                .zip([horizontal, vertical, horizontal && vertical])
                                .all(|(next, used)| {
                                    !used
                                        || next.is_none_or(|next| {
                                            next.white_backing || next.row_support.opaque
                                        })
                                }));
                    if opaque {
                        1
                    } else {
                        2
                    }
                };
                #[cfg(feature = "compose_work_profile")]
                {
                    layer_compose_profile::class(tile_class);
                    if let Some(b) = draw_visible {
                        layer_compose_profile::quad((b.right - b.left) * (b.bottom - b.top));
                    }
                }
                if direct_mask.is_some() {
                    let index = groups.len() - 2;
                    let parent = &mut groups[index];
                    dst_in_tile(
                        &mut parent.pixels,
                        parent.view,
                        parent.touched,
                        width,
                        height,
                        tile,
                        composition,
                        pixels,
                        adjacent,
                    )
                    .expect("preflight validated direct mask tile");
                } else if direct_clip.is_some() {
                    let clip_index = groups.len() - 1;
                    let coverage = groups[clip_index].coverage.clone();
                    // An empty actual clip draws nothing. Keep its scope
                    // identity but write directly to the original parent.
                    if let Some(coverage) = coverage {
                        if clip_index > 0 {
                            let parent = &mut groups[clip_index - 1];
                            let touched = layer_clip_quad::compose_clipped_tile(
                                &mut parent.pixels,
                                width,
                                height,
                                parent.view,
                                skia::PixelFormat::Rgba8888,
                                tile,
                                composition,
                                pixels,
                                adjacent,
                                draw_visible,
                                &coverage,
                            )
                            .expect("preflight validated clipped tile");
                            union_bounds(&mut parent.touched, touched);
                        } else {
                            layer_clip_quad::compose_clipped_tile(
                                buffer,
                                width,
                                height,
                                frame_view,
                                format,
                                tile,
                                composition,
                                pixels,
                                adjacent,
                                draw_visible,
                                &coverage,
                            )
                            .expect("preflight validated clipped tile");
                        }
                    }
                } else if let Some(group) = groups.last_mut() {
                    debug_assert!(!composition.white_backing);
                    let touched = compose_tile(
                        &mut group.pixels,
                        width,
                        height,
                        group.view,
                        skia::PixelFormat::Rgba8888,
                        tile,
                        composition,
                        pixels,
                        adjacent,
                        draw_visible,
                    )
                    .expect("preflight validated tile draw");
                    union_bounds(&mut group.touched, touched);
                } else {
                    compose_tile(
                        buffer,
                        width,
                        height,
                        frame_view,
                        format,
                        tile,
                        composition,
                        pixels,
                        adjacent,
                        draw_visible,
                    )
                    .expect("preflight validated tile draw");
                }
                if let Some(cost) = &mut compose_costs {
                    cost.tile_blits[tile_class] += tile_start.unwrap().elapsed();
                    if let Some(bounds) = draw_visible {
                        cost.tile_pixels[tile_class] +=
                            (bounds.right - bounds.left) * (bounds.bottom - bounds.top);
                    }
                    if tile_class == 4 {
                        cost.solid_tiles += 1;
                    }
                }
            }
            if let Some(cost) = &mut compose_costs {
                cost.blit += blit_start.unwrap().elapsed();
            }
        }
        let close_start = compose_costs.as_ref().map(|_| std::time::Instant::now());
        while !groups.is_empty() {
            self.finish_effect_group(&mut groups, buffer, frame_view, format);
        }
        if let Some(cost) = &mut compose_costs {
            cost.close += close_start.unwrap().elapsed();
        }
        let compose_done = timing.map(|start| start.elapsed());
        draw_trace.set("reused_tiles", reused as f64);
        draw_trace.set("integral_tiles", integral_tiles as f64);
        draw_trace.set("fractional_tiles", fractional_tiles as f64);
        draw_trace.set(
            "root_clear_skipped",
            if root_clear_skipped { 1.0 } else { 0.0 },
        );
        if let Some(cost) = &compose_costs {
            let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0;
            // Scratch is a subset of open. Clip preparation is preflight work.
            draw_trace.set("open_effects_ms", ms(cost.open));
            draw_trace.set("scratch_ms", ms(cost.scratch));
            draw_trace.set("blit_ms", ms(cost.blit));
            draw_trace.set("finish_effects_ms", ms(cost.close));
            browser_tracing::instant(
                "compose",
                "ComposeBlitCosts",
                &[
                    ("integral_ms", ms(cost.tile_blits[0])),
                    ("fractional_opaque_ms", ms(cost.tile_blits[1])),
                    ("fractional_blend_ms", ms(cost.tile_blits[2])),
                    ("mask_ms", ms(cost.tile_blits[3])),
                    ("solid_ms", ms(cost.tile_blits[4])),
                    ("integral_pixels", cost.tile_pixels[0] as f64),
                ],
            );
            // Trace events retain at most six numeric fields. Keep pixel work
            // separate from timings so the final classes are not truncated.
            browser_tracing::instant(
                "compose",
                "ComposeBlitPixels",
                &[
                    ("integral", cost.tile_pixels[0] as f64),
                    ("fractional_opaque", cost.tile_pixels[1] as f64),
                    ("fractional_blend", cost.tile_pixels[2] as f64),
                    ("mask", cost.tile_pixels[3] as f64),
                    ("solid", cost.tile_pixels[4] as f64),
                    ("occluded", cost.occluded_pixels as f64),
                ],
            );
            browser_tracing::instant(
                "compose",
                "ComposeEffects",
                &[
                    ("direct_clip_layers", cost.direct_clip_layers as f64),
                    ("mask_layers", cost.mask_layers as f64),
                    ("direct_mask_layers", cost.direct_mask_layers as f64),
                    ("effect_scopes", cost.scopes as f64),
                    ("finish_ms", ms(cost.close)),
                    ("scratch_ms", ms(cost.scratch)),
                ],
            );
        }
        drop(draw_trace);
        compose_trace.set("layers", self.stats.layers as f64);
        compose_trace.set("tiles", self.stats.tiles as f64);
        compose_trace.set("raster_tasks", plan.tasks.len() as f64);
        compose_trace.set("reused_tiles", reused as f64);
        compose_trace.set(
            "root_clear_skipped",
            if root_clear_skipped { 1.0 } else { 0.0 },
        );
        drop(compose_trace);
        self.stats.raster_tasks = plan.tasks.len();
        self.stats.reused_tiles = reused;
        self.stats.resident_bytes = self
            .pixels
            .values()
            .map(|tile| tile.rgba.len() + tile.bgra.len() * std::mem::size_of::<u32>())
            .sum();
        if let Some(start) = timing {
            let ms = |time: std::time::Duration| time.as_secs_f64() * 1000.0;
            let replay_time = replay_done.unwrap();
            let raster = raster_done.unwrap();
            let compose = compose_done.unwrap();
            eprintln!("layer-tile-stages replay_ms={:.3} raster_ms={:.3} compose_ms={:.3} commit_ms={:.3} tasks={} reused={}",
                ms(replay_time),
                ms(raster-replay_time), ms(compose-raster), ms(start.elapsed()-compose),
                self.stats.raster_tasks, self.stats.reused_tiles);
            if let Some(cost) = compose_costs {
                eprintln!("layer-tile-compose-cost open_ms={:.3} clip_ms={:.3} scratch_ms={:.3} blit_ms={:.3} close_ms={:.3} scopes={} integral_tiles={} fractional_tiles={} skip_zero_layers={} nominal_tile_pixels={} phase_8bit_tiles={} phase_16bit_tiles={} phase_f64_tiles={} mask_layers={} mask_raster_tasks={} mask_reused_tiles={} direct_mask_layers={} root_clear_skipped={} occluded_tiles={} occluded_pixels={} integral_blit_ms={:.3} fractional_opaque_blit_ms={:.3} fractional_blend_blit_ms={:.3} mask_blit_ms={:.3} solid_tiles={} solid_blit_ms={:.3} direct_clip_layers={}",
                    ms(cost.open), ms(cost.clip), ms(cost.scratch), ms(cost.blit), ms(cost.close), cost.scopes,
                    cost.integral, cost.fractional, cost.skipped_zero, cost.nominal_pixels,
                    cost.fractional_phase_classes[0], cost.fractional_phase_classes[1], cost.fractional_phase_classes[2],
                    cost.mask_layers, cost.mask_raster_tasks, cost.mask_reused_tiles, cost.direct_mask_layers, root_clear_skipped,
                    cost.occluded_tiles, cost.occluded_pixels, ms(cost.tile_blits[0]), ms(cost.tile_blits[1]),
                    ms(cost.tile_blits[2]), ms(cost.tile_blits[3]), cost.solid_tiles, ms(cost.tile_blits[4]), cost.direct_clip_layers);
            }
        }
        #[cfg(feature = "compose_work_profile")]
        layer_compose_profile::end();
        Ok(RasterUpdate {
            mode: "layer-tiles",
            reason: if plan.tasks.is_empty() {
                "reused-visible-tiles"
            } else {
                "rasterized-dirty-tiles"
            },
            damage_pixels,
            copied_bytes: 0,
        })
    }
}

fn tile_bytes(size: (u32, u32)) -> io::Result<usize> {
    (size.0 as usize)
        .checked_mul(size.1 as usize)
        .and_then(|n| n.checked_mul(4))
        .filter(|&n| size.0 > 0 && size.1 > 0 && n <= isize::MAX as usize)
        .ok_or_else(|| io::Error::other("invalid tile pixel dimensions"))
}

fn device_position(value: f64) -> io::Result<i64> {
    if !value.is_finite()
        || value != value.round()
        || value < i32::MIN as f64
        || value > i32::MAX as f64
    {
        return Err(io::Error::other(
            "tile has unsupported fractional or overflowing pixel placement",
        ));
    }
    Ok(value as i64)
}

pub(crate) fn tile_row_support(rgba: &[u8], width: usize, opaque: bool) -> TileRowSupport {
    let mut support = TileRowSupport {
        rows: Vec::with_capacity(rgba.len() / (width * 4)),
        runs: Vec::new(),
        opaque: true,
        draw_mode: layer_solid::TileDrawMode::Resource,
    };
    for row in rgba.chunks_exact(width * 4) {
        if opaque {
            support.rows.push(RowSupport {
                extent: (0, width),
                opaque: true,
                runs: (0, 0),
            });
            continue;
        }
        // Most image tiles are opaque although their layer started clear.
        // Prove this once from actual pixels, instead of reading/blending the
        // destination on every cached composition. Transparent tiles stop the
        // proof at their first non-opaque row.
        if support.opaque {
            if row.chunks_exact(4).all(|pixel| pixel[3] == 255) {
                support.rows.push(RowSupport {
                    extent: (0, width),
                    opaque: true,
                    runs: (0, 0),
                });
                continue;
            }
            support.opaque = false;
        }
        let nonzero = |pixel: &[u8]| pixel != [0, 0, 0, 0];
        let Some(left) = row.chunks_exact(4).position(nonzero) else {
            support.rows.push(RowSupport {
                extent: (0, 0),
                opaque: false,
                runs: (0, 0),
            });
            continue;
        };
        let right = row.chunks_exact(4).rposition(nonzero).unwrap() + 1;
        let run_start = support.runs.len();
        let mut row_opaque = row[left * 4 + 3] == 255;
        let mut span_start = left;
        let mut previous = left;
        // Cache only long holes. Short glyph/AA holes stay in the SIMD span,
        // avoiding an allocation or kernel call per small coverage fragment.
        if right - left > 17 {
            for (x, pixel) in row.chunks_exact(4).enumerate().take(right).skip(left + 1) {
                // Reuse the existing product classification scan. A zero hole
                // inside this extent also rejects Src: copying zero must not
                // erase the underlying destination.
                row_opaque &= pixel[3] == 255;
                if nonzero(pixel) {
                    if x - previous - 1 >= 16 {
                        support.runs.push((span_start, previous + 1));
                        span_start = x;
                    }
                    previous = x;
                }
            }
        } else {
            row_opaque &= row[left * 4..right * 4]
                .chunks_exact(4)
                .all(|pixel| pixel[3] == 255);
        }
        if support.runs.len() != run_start {
            support.runs.push((span_start, right));
        }
        support.rows.push(RowSupport {
            extent: (left, right),
            opaque: row_opaque,
            runs: (run_start, support.runs.len()),
        });
    }
    support
}

// Merge real source support without allocating per composed row. Horizontal
// sampling reaches x+1; a narrow lower tile clamps its last real pixel through
// the rest of the upper tile. The true right edge still uses tile_edge_pixel.
fn for_fractional_spans<const H: bool>(
    upper: &[(usize, usize)],
    lower: &[(usize, usize)],
    width: usize,
    lower_width: usize,
    mut visit: impl FnMut(usize, usize),
) {
    let normalize = |&(left, right): &(usize, usize), clamp: bool| {
        let left = if H { left.saturating_sub(1) } else { left };
        let right = if clamp && right == lower_width {
            width
        } else {
            right.min(width)
        };
        (left < right).then_some((left, right))
    };
    let mut upper = upper
        .iter()
        .filter_map(|span| normalize(span, false))
        .peekable();
    let mut lower = lower
        .iter()
        .filter_map(|span| normalize(span, true))
        .peekable();
    let mut merged: Option<(usize, usize)> = None;
    loop {
        let next = match (upper.peek(), lower.peek()) {
            (Some(a), Some(b)) if a.0 <= b.0 => upper.next(),
            (Some(_), Some(_)) => lower.next(),
            (Some(_), None) => upper.next(),
            (None, Some(_)) => lower.next(),
            (None, None) => break,
        }
        .unwrap();
        merged = Some(match merged {
            Some(old) if next.0 <= old.1 => (old.0, old.1.max(next.1)),
            Some(old) => {
                visit(old.0, old.1);
                next
            }
            None => next,
        });
    }
    if let Some((left, right)) = merged {
        visit(left, right);
    }
}

#[cfg(test)]
mod sparse_support_test {
    use super::*;

    #[test]
    fn opaque_row_src_preserves_transparent_margins_holes_and_aa() {
        let (width, height) = (19u32, 6u32);
        let mut rgba = vec![0; width as usize * height as usize * 4];
        for y in 1..6usize {
            for x in 3..16usize {
                let a = if y == 3 && x == 3 { 127 } else { 255 };
                let at = (y * width as usize + x) * 4;
                rgba[at..at + 4].copy_from_slice(&[a / 3, a / 2, a, a]);
            }
        }
        // A short zero hole stays within the dense support extent. Its row
        // must keep SrcOver so copying zero cannot erase the parent surface.
        rgba[(2 * width as usize + 9) * 4..(2 * width as usize + 9) * 4 + 4].fill(0);
        let support = tile_row_support(&rgba, width as usize, false);
        assert!(!support.opaque);
        assert!(support.rows[1].opaque && support.rows[4].opaque);
        assert!(!support.rows[2].opaque && !support.rows[3].opaque);
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: width as f64,
            height: height as f64,
        };
        let tile = TilePlacement {
            tile_id: TileId(1),
            generation: 1,
            tile_index: (0, 0),
            tiling_rect: rect,
            tile_rect: rect,
            raster_scale: 1.0,
            pixel_size: (width, height),
        };
        let pixels = CachedTile {
            raster_frame: 0,
            generation: 1,
            pixel_size: (width, height),
            raster_scale: 1.0,
            white_backing: false,
            rgba,
            row_support: support,
        };
        let composition = LayerComposition {
            translation: (0.0, 0.0),
            clip: None,
            content_bounds: None,
            opaque_bounds: None,
            white_backing: false,
            grid_min: None,
        };
        let view = TargetView {
            origin: (0, 0),
            stride: width as usize,
        };
        for format in [
            skia::PixelFormat::Rgba8888,
            skia::PixelFormat::Bgra8888,
            skia::PixelFormat::Bgrx8888,
        ] {
            let initial = if format == skia::PixelFormat::Bgrx8888 {
                0x00714a25
            } else {
                0xb8714a25
            };
            let mut actual = vec![initial; width as usize * height as usize];
            let mut expected = actual.clone();
            let expected_bytes = unsafe {
                std::slice::from_raw_parts_mut(
                    expected.as_mut_ptr().cast::<u8>(),
                    expected.len() * 4,
                )
            };
            skia::blend_premultiplied_rgba_row(expected_bytes, &pixels.rgba, format);
            compose_tile(
                &mut actual,
                width,
                height,
                view,
                format,
                &tile,
                composition,
                &pixels,
                [None; 3],
                None,
            )
            .unwrap();
            assert_eq!(actual, expected, "opaque row Src format={format:?}");
        }
    }

    #[test]
    fn fractional_sparse_row_matches_dense_sampler_with_clamped_neighbor() {
        let tile = |row: Vec<u8>| {
            let width = row.len() / 4;
            CachedTile {
                raster_frame: 0,
                generation: 1,
                pixel_size: (width as u32, 1),
                raster_scale: 1.0,
                white_backing: false,
                row_support: tile_row_support(&row, width, false),
                rgba: row,
                bgra: Vec::new(),
            }
        };
        let mut upper = vec![0; 96 * 4];
        for x in [0, 1, 35, 36, 82, 83, 95] {
            upper[x * 4..x * 4 + 4].copy_from_slice(&[40, 20, 10, 90]);
        }
        // RGB also participates in the exact-zero proof.
        upper[35 * 4..35 * 4 + 4].copy_from_slice(&[1, 0, 0, 0]);
        let mut lower = vec![0; 64 * 4];
        for x in [2, 3, 37, 38, 63] {
            lower[x * 4..x * 4 + 4].copy_from_slice(&[10, 30, 50, 120]);
        }
        let upper = tile(upper);
        let lower = tile(lower);
        let right = tile(vec![30, 10, 20, 80]);
        let phase = (0.375, 0.3125);
        let mut expected = vec![0xb8734a25; 96];
        let mut actual = expected.clone();
        for (x, dst) in expected.iter_mut().enumerate() {
            let a = rgba_word(&upper.rgba, x);
            let b = if x + 1 == 96 {
                rgba_word(&right.rgba, 0)
            } else {
                rgba_word(&upper.rgba, x + 1)
            };
            let c = rgba_word(&lower.rgba, x.min(63));
            // Missing diagonal neighbor clamps to the original tile, matching
            // the original complete row sampler's actual edge selection.
            let d = if x + 1 == 96 {
                rgba_word(&upper.rgba, 95)
            } else {
                rgba_word(&lower.rgba, (x + 1).min(63))
            };
            store_linear::<false>(
                dst,
                linear_word::<true, true>(a, b, c, d, phase, (1.0 - phase.0, 1.0 - phase.1)),
                skia::PixelFormat::Bgrx8888,
            );
        }
        let mut sampled = [false; 96];
        for_fractional_spans::<true>(
            upper.row_support.spans(0),
            lower.row_support.spans(0),
            96,
            64,
            |left, right| sampled[left..right].fill(true),
        );
        assert!(!sampled[10]);
        assert!(sampled[35] && sampled[80]);
        assert!(fractional_row::<true, true, false, false>(
            &mut actual,
            &upper,
            [Some(&right), Some(&lower), None],
            0,
            0,
            phase,
            (1.0 - phase.0, 1.0 - phase.1),
            skia::PixelFormat::Bgrx8888
        )
        .is_some());
        assert_eq!(actual, expected);
    }

    #[test]
    fn fractional_opaque_rows_match_scalar_srcover_with_aa_holes_and_neighbors() {
        let tile = |width: usize, height: usize| {
            let rgba: Vec<u8> = (0..height)
                .flat_map(|y| {
                    (0..width).flat_map(move |x| {
                        let a = if x < 3 || x + 3 >= width {
                            0
                        } else if y == 2 && x == 9 {
                            0
                        } else if y == 3 && x == 3 {
                            127
                        } else {
                            255
                        };
                        [a / 3, a / 2, a, a]
                    })
                })
                .collect();
            CachedTile {
                raster_frame: 0,
                generation: 1,
                pixel_size: (width as u32, height as u32),
                raster_scale: 1.0,
                white_backing: false,
                row_support: tile_row_support(&rgba, width, false),
                rgba,
                bgra: Vec::new(),
            }
        };
        let pixels = tile(29, 6);
        let mut lower = tile(13, 1);
        // A narrow neighbor whose last real pixel is opaque clamps through
        // the upper tile's remainder. The diagonal still follows tile_edge_pixel.
        lower.rgba[10 * 4..]
            .copy_from_slice(&[85, 127, 255, 255, 85, 127, 255, 255, 85, 127, 255, 255]);
        lower.row_support = tile_row_support(&lower.rgba, 13, false);
        assert!(pixels.row_support.rows[1].opaque);
        assert!(!pixels.row_support.rows[2].opaque && !pixels.row_support.rows[3].opaque);
        let adjacent = [None, Some(&lower), None];
        fn check<const H: bool, const V: bool>(
            pixels: &CachedTile,
            adjacent: [Option<&CachedTile>; 3],
            phase: (f64, f64),
            format: skia::PixelFormat,
        ) {
            let sampler = PreparedFractionalSampler::for_tile::<H, V, false>(phase, format);
            for sy in -1..pixels.pixel_size.1 as i64 {
                let mut expected = vec![
                    if format == skia::PixelFormat::Bgrx8888 {
                        0x00714a25
                    } else {
                        0xb8714a25
                    };
                    pixels.pixel_size.0 as usize + 1
                ];
                let mut actual = expected.clone();
                for (i, dst) in expected.iter_mut().enumerate() {
                    let x = i as i64 - 1;
                    let a = signed_tile_pixel(pixels, adjacent, x, sy);
                    let b = if H {
                        signed_tile_pixel(pixels, adjacent, x + 1, sy)
                    } else {
                        a
                    };
                    let c = if V {
                        signed_tile_pixel(pixels, adjacent, x, sy + 1)
                    } else {
                        a
                    };
                    let d = if H && V {
                        signed_tile_pixel(pixels, adjacent, x + 1, sy + 1)
                    } else if V {
                        c
                    } else {
                        b
                    };
                    *dst = source_over_rgba_word(
                        *dst,
                        linear_word::<H, V>(a, b, c, d, phase, sampler.inverse),
                        format,
                    );
                }
                fractional_row_signed_execute::<H, V, false, true, true>(
                    &mut actual,
                    pixels,
                    adjacent,
                    sy,
                    -1,
                    &sampler,
                    format,
                );
                assert_eq!(
                    actual, expected,
                    "phase={phase:?} sy={sy} format={format:?}"
                );
            }
        }
        for format in [
            skia::PixelFormat::Rgba8888,
            skia::PixelFormat::Bgra8888,
            skia::PixelFormat::Bgrx8888,
        ] {
            for phase in [(0.375, 0.3125), (0.1, 0.2)] {
                check::<false, true>(&pixels, adjacent, (0.0, phase.1), format);
                check::<true, false>(&pixels, adjacent, (phase.0, 0.0), format);
                check::<true, true>(&pixels, adjacent, phase, format);
            }
        }
        if std::env::var_os("RENDERER_TIMING_OPAQUE_FRACTIONAL").is_some() {
            let pixels = tile(256, 256);
            let mut baseline = tile(256, 256);
            // Same source/support geometry, with only the new row opacity
            // proof disabled, exercises the previous production blend kernel.
            for row in &mut baseline.row_support.rows {
                row.opaque = false;
            }
            let mut output = vec![0x00714a25; 256 * 256];
            let view = TargetView {
                origin: (0, 0),
                stride: 256,
            };
            let bounds = SurfaceBounds {
                left: 0,
                top: 0,
                right: 256,
                bottom: 256,
            };
            for phase in [(0.0, 0.3125), (0.375, 0.3125)] {
                output.fill(0x00714a25);
                fractional_tile::<false>(
                    &mut output,
                    view,
                    bounds,
                    (0, 0),
                    &baseline,
                    [None; 3],
                    phase,
                    skia::PixelFormat::Bgrx8888,
                );
                let expected = output.clone();
                output.fill(0x00714a25);
                fractional_tile::<false>(
                    &mut output,
                    view,
                    bounds,
                    (0, 0),
                    &pixels,
                    [None; 3],
                    phase,
                    skia::PixelFormat::Bgrx8888,
                );
                assert!(
                    output == expected,
                    "timed tile phase={phase:?} first mismatch={:?}",
                    output.iter().zip(&expected).position(|(a, b)| a != b)
                );
                let measure = |output: &mut [u32], source: &CachedTile| {
                    output.fill(0x00714a25);
                    let started = std::time::Instant::now();
                    for _ in 0..128 {
                        std::hint::black_box(fractional_tile::<false>(
                            std::hint::black_box(output),
                            view,
                            bounds,
                            (0, 0),
                            std::hint::black_box(source),
                            [None; 3],
                            phase,
                            skia::PixelFormat::Bgrx8888,
                        ));
                    }
                    started.elapsed().as_secs_f64() * 1000.0
                };
                let mut old = [0.0f64; 3];
                let mut new = [0.0f64; 3];
                for round in 0..3 {
                    old[round] = measure(&mut output, &baseline);
                    new[round] = measure(&mut output, &pixels);
                }
                old.sort_by(f64::total_cmp);
                new.sort_by(f64::total_cmp);
                eprintln!("opaque-fractional-row-timing phase={phase:?} iterations=128 pixels=65536 baseline_ms={:.3} src_ms={:.3}",old[1],new[1]);
            }
        }
        if std::env::var_os("RENDERER_TIMING_SCROLL_SNAPPING").is_some() {
            let pixels = tile(256, 256);
            let rect = PaintRect {
                x: 0.0,
                y: 0.0,
                width: 256.0,
                height: 256.0,
            };
            let placement = TilePlacement {
                tile_id: TileId(1),
                generation: 1,
                tile_index: (0, 0),
                tiling_rect: rect,
                tile_rect: rect,
                raster_scale: 1.0,
                pixel_size: (256, 256),
            };
            let base = LayerComposition {
                translation: (0.0, -0.25),
                clip: None,
                content_bounds: None,
                opaque_bounds: None,
                white_backing: false,
                grid_min: Some((0, 0)),
            };
            let snapped = LayerComposition {
                translation: (0.0, 0.0),
                ..base
            };
            let view = TargetView {
                origin: (0, 0),
                stride: 256,
            };
            let mut output = vec![0x00714a25; 256 * 256];
            let measure = |output: &mut [u32], composition| {
                output.fill(0x00714a25);
                let started = std::time::Instant::now();
                for _ in 0..128 {
                    std::hint::black_box(
                        compose_tile(
                            std::hint::black_box(output),
                            256,
                            256,
                            view,
                            skia::PixelFormat::Bgrx8888,
                            &placement,
                            composition,
                            std::hint::black_box(&pixels),
                            [None; 3],
                            None,
                        )
                        .unwrap(),
                    );
                }
                started.elapsed().as_secs_f64() * 1000.0
            };
            let (mut fractional, mut integral) = ([0.0f64; 3], [0.0f64; 3]);
            for round in 0..3 {
                fractional[round] = measure(&mut output, base);
                integral[round] = measure(&mut output, snapped);
            }
            fractional.sort_by(f64::total_cmp);
            integral.sort_by(f64::total_cmp);
            eprintln!("scroll-snapping-compose-timing iterations=128 tile_pixels=65536 fractional_ms={:.3} integral_ms={:.3}",
                fractional[1],integral[1]);
        }
    }
}

#[inline]
fn rgba_word(row: &[u8], x: usize) -> u32 {
    let i = x * 4;
    u32::from_le_bytes([row[i], row[i + 1], row[i + 2], row[i + 3]])
}

// Only the last column/row can enter a different tile. Keep the original
// right/down/diagonal selection and missing-neighbor clamp for those edges.
fn tile_edge_pixel(
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    x: usize,
    y: usize,
) -> u32 {
    let width = pixels.pixel_size.0 as usize;
    let height = pixels.pixel_size.1 as usize;
    let right = x >= width;
    let down = y >= height;
    let neighbor = match (right, down) {
        (true, false) => adjacent[0],
        (false, true) => adjacent[1],
        (true, true) => adjacent[2],
        _ => None,
    };
    let (source, x, y) = match neighbor {
        Some(next) => (
            next,
            if right { x - width } else { x },
            if down { y - height } else { y },
        ),
        None => (pixels, x, y),
    };
    let width = source.pixel_size.0 as usize;
    let height = source.pixel_size.1 as usize;
    if let Some(color) = compact_solid_color(source) {
        return color;
    }
    rgba_word(
        &source.rgba[y.min(height - 1) * width * 4..][..width * 4],
        x.min(width - 1),
    )
}

#[inline]
fn linear_word<const H: bool, const V: bool>(
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    phase: (f64, f64),
    inverse: (f64, f64),
) -> u32 {
    let mut result = 0;
    for shift in [0, 8, 16, 24] {
        let channel = |word: u32| ((word >> shift) & 255) as f64;
        // Preserve the original f64 operation order, including round().
        let upper = if H {
            channel(a) * inverse.0 + channel(b) * phase.0
        } else {
            channel(a)
        };
        let value = if V {
            let lower = if H {
                channel(c) * inverse.0 + channel(d) * phase.0
            } else {
                channel(c)
            };
            upper * inverse.1 + lower * phase.1
        } else {
            upper
        };
        result |= (value.round().clamp(0.0, 255.0) as u32) << shift;
    }
    result
}

#[inline]
fn store_linear<const OPAQUE: bool>(
    destination: &mut u32,
    mut source: u32,
    format: skia::PixelFormat,
) {
    if OPAQUE {
        if format != skia::PixelFormat::Rgba8888 {
            source = (source & 0xff00_ff00) | ((source & 0xff) << 16) | ((source >> 16) & 0xff);
        }
        if format == skia::PixelFormat::Bgrx8888 {
            source &= 0x00ff_ffff;
        }
        *destination = source;
    } else {
        *destination = source_over_rgba_word(*destination, source, format);
    }
}

fn exact_fraction_weight(phase: f64) -> Option<u16> {
    let weight = phase * 256.0;
    (weight.is_finite() && (0.0..=256.0).contains(&weight) && weight == weight.round())
        .then_some(weight as u16)
}

// Like SkRasterPipelineBlitter's compiled shader context, these parameters
// belong to the actual tile geometry and remain unchanged across scanlines.
// Do not derive them from nominal layer translation: adding the tile origin
// can change the exact f64 phase used by the original sampler.
struct PreparedFractionalSampler {
    phase: (f64, f64),
    inverse: (f64, f64),
    fixed: Option<(u16, u16)>,
    valid: bool,
    #[cfg(target_arch = "aarch64")]
    compiled: Option<CompiledFractionalKernel>,
    #[cfg(target_arch = "aarch64")]
    compiled_src: Option<CompiledFractionalKernel>,
}
#[cfg(target_arch = "aarch64")]
type CompiledFractionalKernel =
    unsafe fn(&mut [u8], &[u8], &[u8], &PreparedFractionalSampler) -> usize;

// SkRasterPipeline::compile and SkRasterPipelineBlitter::blitRect select
// shader/blend/storage stages once, before running all spans. Keep that
// selection outside this backend's scanline/span traversal as well.
#[cfg(target_arch = "aarch64")]
unsafe fn compiled_fractional_kernel<
    const H: bool,
    const V: bool,
    const BLEND: bool,
    const SWAP: bool,
    const ZERO_X: bool,
    const FIXED: bool,
>(
    output: &mut [u8],
    upper: &[u8],
    lower: &[u8],
    sampler: &PreparedFractionalSampler,
) -> usize {
    unsafe {
        if FIXED {
            fixed_linear_neon_storage::<H, V, BLEND, SWAP, ZERO_X>(
                output,
                upper,
                lower,
                sampler.fixed.expect("compiled fixed sampler"),
            )
        } else {
            general_linear_neon_storage::<H, V, BLEND, SWAP, ZERO_X>(
                output,
                upper,
                lower,
                sampler.phase,
                sampler.inverse,
            )
        }
    }
}

#[cfg(target_arch = "aarch64")]
fn select_fractional_kernel<const H: bool, const V: bool, const BLEND: bool, const FIXED: bool>(
    format: skia::PixelFormat,
) -> CompiledFractionalKernel {
    match format {
        skia::PixelFormat::Rgba8888 => {
            compiled_fractional_kernel::<H, V, BLEND, false, false, FIXED>
        }
        skia::PixelFormat::Bgra8888 => {
            compiled_fractional_kernel::<H, V, BLEND, true, false, FIXED>
        }
        skia::PixelFormat::Bgrx8888 => compiled_fractional_kernel::<H, V, BLEND, true, true, FIXED>,
    }
}
impl PreparedFractionalSampler {
    fn new(phase: (f64, f64)) -> Self {
        Self {
            phase,
            inverse: (1.0 - phase.0, 1.0 - phase.1),
            fixed: exact_fraction_weight(phase.0).zip(exact_fraction_weight(phase.1)),
            valid: [phase.0, phase.1]
                .iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
            #[cfg(target_arch = "aarch64")]
            compiled: None,
            #[cfg(target_arch = "aarch64")]
            compiled_src: None,
        }
    }

    fn for_tile<const H: bool, const V: bool, const OPAQUE: bool>(
        phase: (f64, f64),
        format: skia::PixelFormat,
    ) -> Self {
        let mut sampler = Self::new(phase);
        #[cfg(target_arch = "aarch64")]
        {
            sampler.compiled = Some(if sampler.fixed.is_some() {
                if OPAQUE {
                    select_fractional_kernel::<H, V, false, true>(format)
                } else {
                    select_fractional_kernel::<H, V, true, true>(format)
                }
            } else {
                if OPAQUE {
                    select_fractional_kernel::<H, V, false, false>(format)
                } else {
                    select_fractional_kernel::<H, V, true, false>(format)
                }
            });
            // A translucent tile can contain opaque scanline interiors. Compile
            // their Src storage with the same sampling arithmetic, once per tile.
            sampler.compiled_src = Some(if OPAQUE {
                sampler.compiled.unwrap()
            } else if sampler.fixed.is_some() {
                select_fractional_kernel::<H, V, false, true>(format)
            } else {
                select_fractional_kernel::<H, V, false, false>(format)
            });
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            let _ = format;
        }
        sampler
    }
}

#[inline]
fn fractional_compiled_span<const H: bool, const V: bool, const OPAQUE: bool>(
    output: &mut [u32],
    upper: &[u8],
    lower: &[u8],
    first: usize,
    sampler: &PreparedFractionalSampler,
    format: skia::PixelFormat,
) -> bool {
    #[cfg(not(target_arch = "aarch64"))]
    {
        return fractional_fixed_span_prepared::<H, V, OPAQUE, true>(
            output, upper, lower, first, sampler, format,
        );
    }
    #[cfg(target_arch = "aarch64")]
    {
        if output.len() > TILE_SIZE as usize || !sampler.valid {
            return false;
        }
        let bytes = output.len() * 4;
        let start = first * 4;
        let required = bytes + if H { 4 } else { 0 };
        if start + required > upper.len() || (V && start + required > lower.len()) {
            return false;
        }
        let kernel = if OPAQUE {
            sampler.compiled_src
        } else {
            sampler.compiled
        }
        .expect("prepared tile program");
        // Bounds above include the actual H neighbor. Output is a complete
        // exclusive N32 row; RGBA effects and final BGRX retain their formats.
        let done = unsafe {
            let target = std::slice::from_raw_parts_mut(output.as_mut_ptr().cast::<u8>(), bytes);
            kernel(
                target,
                &upper[start..],
                if V { &lower[start..] } else { &upper[start..] },
                sampler,
            )
        };
        #[cfg(feature = "compose_work_profile")]
        layer_compose_profile::span(output.len(), done);
        for i in done..output.len() {
            let x = first + i;
            let a = rgba_word(upper, x);
            let b = if H { rgba_word(upper, x + 1) } else { a };
            let c = if V { rgba_word(lower, x) } else { a };
            let d = if H && V {
                rgba_word(lower, x + 1)
            } else if V {
                c
            } else {
                b
            };
            store_linear::<OPAQUE>(
                &mut output[i],
                linear_word::<H, V>(a, b, c, d, sampler.phase, sampler.inverse),
                format,
            );
        }
        true
    }
}

#[inline]
fn fixed_linear_word<const H: bool, const V: bool>(
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    weight: (u16, u16),
) -> u32 {
    let wx = u32::from(weight.0);
    let wy = u32::from(weight.1);
    let mut result = 0;
    for shift in [0, 8, 16, 24] {
        let channel = |word: u32| (word >> shift) & 255;
        let value = if H && V {
            let upper = channel(a) * (256 - wx) + channel(b) * wx;
            let lower = channel(c) * (256 - wx) + channel(d) * wx;
            (upper * (256 - wy) + lower * wy + 32768) >> 16
        } else if H {
            (channel(a) * (256 - wx) + channel(b) * wx + 128) >> 8
        } else {
            (channel(a) * (256 - wy) + channel(c) * wy + 128) >> 8
        };
        result |= value << shift;
    }
    result
}

// SkPMSrcOver_neon8 / SkMulDiv255Round_neon8: retain both rounded
// narrowing shifts and saturating add. Sampling hands off its actual N32
// registers, matching RasterPipeline's color -> srcover_rgba_8888 -> store.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn src_over_neon8(
    source: core::arch::aarch64::uint8x8_t,
    destination: core::arch::aarch64::uint8x8_t,
    inverse_alpha: core::arch::aarch64::uint8x8_t,
) -> core::arch::aarch64::uint8x8_t {
    use core::arch::aarch64::*;
    unsafe {
        let product = vmull_u8(destination, inverse_alpha);
        // SkBlitRow_opts.h::SkMulDiv255Round_neon8, including both
        // rounded shifts; keep the instructions corresponding to the source.
        let scaled = vraddhn_u16(product, vrshrq_n_u16::<8>(product));
        vqadd_u8(source, scaled)
    }
}

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn fixed_linear_neon<const H: bool, const V: bool, const BLEND: bool>(
    output: &mut [u8],
    upper: &[u8],
    lower: &[u8],
    weight: (u16, u16),
    format: skia::PixelFormat,
) -> usize {
    // SkRasterPipelineBlitter selects swap_rb and the storage stage when
    // compiling its pipeline. Select this backend's storage before the loop.
    unsafe {
        match format {
            skia::PixelFormat::Rgba8888 => {
                fixed_linear_neon_storage::<H, V, BLEND, false, false>(output, upper, lower, weight)
            }
            skia::PixelFormat::Bgra8888 => {
                fixed_linear_neon_storage::<H, V, BLEND, true, false>(output, upper, lower, weight)
            }
            skia::PixelFormat::Bgrx8888 => {
                fixed_linear_neon_storage::<H, V, BLEND, true, true>(output, upper, lower, weight)
            }
        }
    }
}

// Keep storage-specialized loops separate so dispatch cannot reconverge
// into runtime format selects inside each four/eight-pixel batch.
#[cfg(target_arch = "aarch64")]
#[inline(never)]
unsafe fn fixed_linear_neon_storage<
    const H: bool,
    const V: bool,
    const BLEND: bool,
    const SWAP: bool,
    const ZERO_X: bool,
>(
    output: &mut [u8],
    upper: &[u8],
    lower: &[u8],
    weight: (u16, u16),
) -> usize {
    use core::arch::aarch64::*;
    unsafe fn horizontal(a: uint8x8_t, b: uint8x8_t, w: u16) -> uint16x8_t {
        unsafe { vmlaq_n_u16(vmulq_n_u16(vmovl_u8(a), 256 - w), vmovl_u8(b), w) }
    }
    unsafe fn vertical(a: uint16x8_t, b: uint16x8_t, w: u16) -> uint8x8_t {
        unsafe {
            let lo = vmlal_n_u16(vmull_n_u16(vget_low_u16(a), 256 - w), vget_low_u16(b), w);
            let hi = vmlal_n_u16(vmull_n_u16(vget_high_u16(a), 256 - w), vget_high_u16(b), w);
            vmovn_u16(vcombine_u16(vrshrn_n_u32::<16>(lo), vrshrn_n_u32::<16>(hi)))
        }
    }
    let mut offset = 0;
    // Packed unaligned loads cover four complete RGBA pixels. Caller excludes
    // the right tile edge and proves both source spans cover every load. The
    // shifted horizontal load reads exactly one extra real neighbor pixel.
    unsafe {
        // Corresponding swap_rb + store_8888 storage stages. Canonical source
        // channels and all sampling arithmetic remain unchanged above the store.
        let swap = vld1q_u8([2u8, 1, 0, 3, 6, 5, 4, 7, 10, 9, 8, 11, 14, 13, 12, 15].as_ptr());
        let keep_rgb = vdupq_n_u32(0x00ff_ffff);
        let repeat_alpha =
            vld1q_u8([3u8, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15].as_ptr());
        let block = |output: *mut u8, upper: *const u8, lower: *const u8| {
            let a = vld1q_u8(upper);
            let b = if H { vld1q_u8(upper.add(4)) } else { a };
            let c = if V { vld1q_u8(lower) } else { a };
            let d = if H && V { vld1q_u8(lower.add(4)) } else { c };
            let (lo, hi) = if H && V {
                (
                    vertical(
                        horizontal(vget_low_u8(a), vget_low_u8(b), weight.0),
                        horizontal(vget_low_u8(c), vget_low_u8(d), weight.0),
                        weight.1,
                    ),
                    vertical(
                        horizontal(vget_high_u8(a), vget_high_u8(b), weight.0),
                        horizontal(vget_high_u8(c), vget_high_u8(d), weight.0),
                        weight.1,
                    ),
                )
            } else {
                let (other, w) = if H { (b, weight.0) } else { (c, weight.1) };
                (
                    vrshrn_n_u16::<8>(horizontal(vget_low_u8(a), vget_low_u8(other), w)),
                    vrshrn_n_u16::<8>(horizontal(vget_high_u8(a), vget_high_u8(other), w)),
                )
            };
            let mut result = vcombine_u8(lo, hi);
            if SWAP {
                result = vqtbl1q_u8(result, swap);
            }
            if BLEND {
                let inverse = vmvnq_u8(vqtbl1q_u8(result, repeat_alpha));
                let destination = vld1q_u8(output);
                result = vcombine_u8(
                    src_over_neon8(
                        vget_low_u8(result),
                        vget_low_u8(destination),
                        vget_low_u8(inverse),
                    ),
                    src_over_neon8(
                        vget_high_u8(result),
                        vget_high_u8(destination),
                        vget_high_u8(inverse),
                    ),
                );
            }
            if ZERO_X {
                result = vreinterpretq_u8_u32(vandq_u32(vreinterpretq_u32_u8(result), keep_rgb));
            }
            vst1q_u8(output, result);
        };
        while offset + 16 <= output.len() {
            let upper = upper.as_ptr().add(offset);
            let lower = if V { lower.as_ptr().add(offset) } else { upper };
            block(output.as_mut_ptr().add(offset), upper, lower);
            offset += 16;
        }
        // SkBlitRow_opts.h::SkPMSrcOver_neon2 and its two/one-pixel tail:
        // narrow loads cover only real pixels, and a lane store writes the
        // final pixel. Sampling keeps the same arithmetic as the full block.
        let block_two =
            |a: uint8x8_t, b: uint8x8_t, c: uint8x8_t, d: uint8x8_t, destination: uint8x8_t| {
                let mut result = if H && V {
                    vertical(
                        horizontal(a, b, weight.0),
                        horizontal(c, d, weight.0),
                        weight.1,
                    )
                } else {
                    let (other, w) = if H { (b, weight.0) } else { (c, weight.1) };
                    vrshrn_n_u16::<8>(horizontal(a, other, w))
                };
                if SWAP {
                    result = vtbl1_u8(result, vget_low_u8(swap));
                }
                if BLEND {
                    let inverse = vmvn_u8(vtbl1_u8(result, vget_low_u8(repeat_alpha)));
                    result = src_over_neon8(result, destination, inverse);
                }
                if ZERO_X {
                    result = vreinterpret_u8_u32(vand_u32(
                        vreinterpret_u32_u8(result),
                        vget_low_u32(keep_rgb),
                    ));
                }
                result
            };
        if offset + 8 <= output.len() {
            let a = vld1_u8(upper.as_ptr().add(offset));
            let b = if H {
                vld1_u8(upper.as_ptr().add(offset + 4))
            } else {
                a
            };
            let c = if V {
                vld1_u8(lower.as_ptr().add(offset))
            } else {
                a
            };
            let d = if H && V {
                vld1_u8(lower.as_ptr().add(offset + 4))
            } else {
                c
            };
            let destination = if BLEND {
                vld1_u8(output.as_ptr().add(offset))
            } else {
                vdup_n_u8(0)
            };
            vst1_u8(
                output.as_mut_ptr().add(offset),
                block_two(a, b, c, d, destination),
            );
            offset += 8;
        }
        if offset < output.len() {
            let pixel =
                |ptr: *const u8| vcreate_u8(u64::from(std::ptr::read_unaligned(ptr.cast::<u32>())));
            let a = pixel(upper.as_ptr().add(offset));
            let b = if H {
                pixel(upper.as_ptr().add(offset + 4))
            } else {
                a
            };
            let c = if V {
                pixel(lower.as_ptr().add(offset))
            } else {
                a
            };
            let d = if H && V {
                pixel(lower.as_ptr().add(offset + 4))
            } else {
                c
            };
            let destination = if BLEND {
                pixel(output.as_ptr().add(offset))
            } else {
                vdup_n_u8(0)
            };
            let result = block_two(a, b, c, d, destination);
            vst1_lane_u32::<0>(
                output.as_mut_ptr().add(offset).cast::<u32>(),
                vreinterpret_u32_u8(result),
            );
            offset += 4;
        }
    }
    offset / 4
}

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn general_linear_neon<const H: bool, const V: bool, const BLEND: bool>(
    output: &mut [u8],
    upper: &[u8],
    lower: &[u8],
    phase: (f64, f64),
    inverse: (f64, f64),
    format: skia::PixelFormat,
) -> usize {
    unsafe {
        match format {
            skia::PixelFormat::Rgba8888 => {
                general_linear_neon_storage::<H, V, BLEND, false, false>(
                    output, upper, lower, phase, inverse,
                )
            }
            skia::PixelFormat::Bgra8888 => general_linear_neon_storage::<H, V, BLEND, true, false>(
                output, upper, lower, phase, inverse,
            ),
            skia::PixelFormat::Bgrx8888 => general_linear_neon_storage::<H, V, BLEND, true, true>(
                output, upper, lower, phase, inverse,
            ),
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(never)]
unsafe fn general_linear_neon_storage<
    const H: bool,
    const V: bool,
    const BLEND: bool,
    const SWAP: bool,
    const ZERO_X: bool,
>(
    output: &mut [u8],
    upper: &[u8],
    lower: &[u8],
    phase: (f64, f64),
    inverse: (f64, f64),
) -> usize {
    use core::arch::aarch64::*;
    #[inline(always)]
    unsafe fn pair<const H: bool, const V: bool>(
        a: uint32x2_t,
        b: uint32x2_t,
        c: uint32x2_t,
        d: uint32x2_t,
        phase: (f64, f64),
        inverse: (f64, f64),
    ) -> uint32x2_t {
        unsafe {
            let convert = |v| vcvtq_f64_u64(vmovl_u32(v));
            // Separate multiply/add intrinsics preserve linear_word's exact
            // f64 order. No phase quantization, f32 narrowing or fused multiply.
            let top = if H {
                vaddq_f64(
                    vmulq_n_f64(convert(a), inverse.0),
                    vmulq_n_f64(convert(b), phase.0),
                )
            } else {
                convert(a)
            };
            let value = if V {
                let bottom = if H {
                    vaddq_f64(
                        vmulq_n_f64(convert(c), inverse.0),
                        vmulq_n_f64(convert(d), phase.0),
                    )
                } else {
                    convert(c)
                };
                vaddq_f64(vmulq_n_f64(top, inverse.1), vmulq_n_f64(bottom, phase.1))
            } else {
                top
            };
            // Inputs and positive weights enclose [0,255]. FRINTA implements
            // the scalar positive round() tie rule before integer conversion.
            vmovn_u64(vcvtq_u64_f64(vrndaq_f64(value)))
        }
    }
    #[inline(always)]
    unsafe fn four<const H: bool, const V: bool>(
        a: uint16x4_t,
        b: uint16x4_t,
        c: uint16x4_t,
        d: uint16x4_t,
        phase: (f64, f64),
        inverse: (f64, f64),
    ) -> uint16x4_t {
        unsafe {
            let (a, b, c, d) = (vmovl_u16(a), vmovl_u16(b), vmovl_u16(c), vmovl_u16(d));
            vmovn_u32(vcombine_u32(
                pair::<H, V>(
                    vget_low_u32(a),
                    vget_low_u32(b),
                    vget_low_u32(c),
                    vget_low_u32(d),
                    phase,
                    inverse,
                ),
                pair::<H, V>(
                    vget_high_u32(a),
                    vget_high_u32(b),
                    vget_high_u32(c),
                    vget_high_u32(d),
                    phase,
                    inverse,
                ),
            ))
        }
    }
    #[inline(always)]
    unsafe fn eight<const H: bool, const V: bool>(
        a: uint8x8_t,
        b: uint8x8_t,
        c: uint8x8_t,
        d: uint8x8_t,
        phase: (f64, f64),
        inverse: (f64, f64),
    ) -> uint8x8_t {
        unsafe {
            let (a, b, c, d) = (vmovl_u8(a), vmovl_u8(b), vmovl_u8(c), vmovl_u8(d));
            vmovn_u16(vcombine_u16(
                four::<H, V>(
                    vget_low_u16(a),
                    vget_low_u16(b),
                    vget_low_u16(c),
                    vget_low_u16(d),
                    phase,
                    inverse,
                ),
                four::<H, V>(
                    vget_high_u16(a),
                    vget_high_u16(b),
                    vget_high_u16(c),
                    vget_high_u16(d),
                    phase,
                    inverse,
                ),
            ))
        }
    }
    let mut at = 0;
    // Interleaved RGBA rows are complete and caller excludes the actual tile
    // edge. Each 8-pixel load and optional right neighbor is inside its span.
    unsafe {
        while at + 32 <= output.len() {
            let a = vld4_u8(upper.as_ptr().add(at));
            let b = if H {
                vld4_u8(upper.as_ptr().add(at + 4))
            } else {
                a
            };
            let c = if V {
                vld4_u8(lower.as_ptr().add(at))
            } else {
                a
            };
            let d = if H && V {
                vld4_u8(lower.as_ptr().add(at + 4))
            } else if V {
                c
            } else {
                b
            };
            let (r, g, b, alpha) = (
                eight::<H, V>(a.0, b.0, c.0, d.0, phase, inverse),
                eight::<H, V>(a.1, b.1, c.1, d.1, phase, inverse),
                eight::<H, V>(a.2, b.2, c.2, d.2, phase, inverse),
                eight::<H, V>(a.3, b.3, c.3, d.3, phase, inverse),
            );
            let (mut r, mut b) = if SWAP { (b, r) } else { (r, b) };
            let (mut g, mut alpha) = (g, alpha);
            if BLEND {
                let inverse = vmvn_u8(alpha);
                let destination = vld4_u8(output.as_ptr().add(at));
                r = src_over_neon8(r, destination.0, inverse);
                g = src_over_neon8(g, destination.1, inverse);
                b = src_over_neon8(b, destination.2, inverse);
                alpha = src_over_neon8(alpha, destination.3, inverse);
            }
            let alpha = if ZERO_X { vdup_n_u8(0) } else { alpha };
            // Skia's NEON store_8888_ keeps channels in registers through the
            // final format permutation and interleaved pixel store.
            vst4_u8(output.as_mut_ptr().add(at), uint8x8x4_t(r, g, b, alpha));
            at += 32;
        }
    }
    at / 4
}

fn fractional_general_span_prepared<
    const H: bool,
    const V: bool,
    const OPAQUE: bool,
    const COMPOSITOR: bool,
>(
    output: &mut [u32],
    upper: &[u8],
    lower: &[u8],
    first: usize,
    sampler: &PreparedFractionalSampler,
    format: skia::PixelFormat,
) -> bool {
    if output.len() > TILE_SIZE as usize || !sampler.valid {
        return false;
    }
    let bytes = output.len() * 4;
    let start = first * 4;
    let required = bytes + if H { 4 } else { 0 };
    if start + required > upper.len() || (V && start + required > lower.len()) {
        return false;
    }
    let phase = sampler.phase;
    let inverse = sampler.inverse;
    let mut scratch = [0u32; TILE_SIZE as usize];
    let rgba = &mut scratch[..output.len()];
    #[cfg(target_arch = "aarch64")]
    // SAFETY: the checks above cover the byte output and all source neighbors;
    // valid phases keep every rounded channel within an initialized N32 pixel.
    let done = unsafe {
        let target = if OPAQUE || COMPOSITOR {
            &mut *output
        } else {
            &mut *rgba
        };
        let target = std::slice::from_raw_parts_mut(target.as_mut_ptr().cast::<u8>(), bytes);
        let lower = if V { &lower[start..] } else { &upper[start..] };
        if !OPAQUE && COMPOSITOR {
            general_linear_neon::<H, V, true>(
                target,
                &upper[start..],
                lower,
                phase,
                inverse,
                format,
            )
        } else {
            general_linear_neon::<H, V, false>(
                target,
                &upper[start..],
                lower,
                phase,
                inverse,
                if OPAQUE {
                    format
                } else {
                    skia::PixelFormat::Rgba8888
                },
            )
        }
    };
    #[cfg(not(target_arch = "aarch64"))]
    let done = 0;
    #[cfg(feature = "compose_work_profile")]
    layer_compose_profile::span(output.len(), done);
    for i in done..output.len() {
        let x = first + i;
        let a = rgba_word(upper, x);
        let b = if H { rgba_word(upper, x + 1) } else { a };
        let c = if V { rgba_word(lower, x) } else { a };
        let d = if H && V {
            rgba_word(lower, x + 1)
        } else if V {
            c
        } else {
            b
        };
        let sampled = linear_word::<H, V>(a, b, c, d, phase, inverse);
        if OPAQUE {
            store_linear::<true>(&mut output[i], sampled, format);
        } else if COMPOSITOR {
            store_linear::<false>(&mut output[i], sampled, format);
        } else {
            rgba[i] = sampled;
        }
    }
    if !OPAQUE && !COMPOSITOR {
        blend_group_row(output, rgba, format);
    }
    true
}

fn fractional_fixed_span_prepared<
    const H: bool,
    const V: bool,
    const OPAQUE: bool,
    const COMPOSITOR: bool,
>(
    output: &mut [u32],
    upper: &[u8],
    lower: &[u8],
    first: usize,
    sampler: &PreparedFractionalSampler,
    format: skia::PixelFormat,
) -> bool {
    let Some((wx, wy)) = sampler.fixed else {
        return fractional_general_span_prepared::<H, V, OPAQUE, COMPOSITOR>(
            output, upper, lower, first, sampler, format,
        );
    };
    if output.len() > TILE_SIZE as usize {
        return false;
    }
    let bytes = output.len() * 4;
    let start = first * 4;
    let required = bytes + if H { 4 } else { 0 };
    if start + required > upper.len() || (V && start + required > lower.len()) {
        return false;
    }
    // Dyadic weights make every original f64 multiply/add exact (at most a
    // 24-bit numerator). Add half the denominator only once, preserving
    // round()'s positive ties-up rule, including two-axis interpolation.
    let mut scratch = [0u32; TILE_SIZE as usize];
    let rgba = &mut scratch[..output.len()];
    #[cfg(target_arch = "aarch64")]
    let done = unsafe {
        let target = if OPAQUE || COMPOSITOR {
            &mut *output
        } else {
            &mut *rgba
        };
        let target = std::slice::from_raw_parts_mut(target.as_mut_ptr().cast::<u8>(), bytes);
        let lower = if V { &lower[start..] } else { &upper[start..] };
        if !OPAQUE && COMPOSITOR {
            fixed_linear_neon::<H, V, true>(target, &upper[start..], lower, (wx, wy), format)
        } else {
            fixed_linear_neon::<H, V, false>(
                target,
                &upper[start..],
                lower,
                (wx, wy),
                if OPAQUE {
                    format
                } else {
                    skia::PixelFormat::Rgba8888
                },
            )
        }
    };
    #[cfg(not(target_arch = "aarch64"))]
    let done = 0;
    #[cfg(feature = "compose_work_profile")]
    layer_compose_profile::span(output.len(), done);
    for i in done..output.len() {
        let x = first + i;
        let a = rgba_word(upper, x);
        let b = if H { rgba_word(upper, x + 1) } else { a };
        let c = if V { rgba_word(lower, x) } else { a };
        let d = if H && V { rgba_word(lower, x + 1) } else { c };
        let sampled = fixed_linear_word::<H, V>(a, b, c, d, (wx, wy));
        if OPAQUE {
            store_linear::<true>(&mut output[i], sampled, format);
        } else if COMPOSITOR {
            store_linear::<false>(&mut output[i], sampled, format);
        } else {
            rgba[i] = sampled;
        }
    }
    if !OPAQUE && !COMPOSITOR {
        blend_group_row(output, rgba, format);
    }
    true
}

#[cfg(test)]
fn fractional_fixed_span<
    const H: bool,
    const V: bool,
    const OPAQUE: bool,
    const COMPOSITOR: bool,
>(
    output: &mut [u32],
    upper: &[u8],
    lower: &[u8],
    first: usize,
    phase: (f64, f64),
    format: skia::PixelFormat,
) -> bool {
    fractional_fixed_span_prepared::<H, V, OPAQUE, COMPOSITOR>(
        output,
        upper,
        lower,
        first,
        &PreparedFractionalSampler::new(phase),
        format,
    )
}

#[cfg(test)]
mod fractional_kernel_test {
    use super::*;

    #[test]
    fn fractional_simd_matches_original_sampling_and_blending() {
        let row = |seed: u32| -> Vec<u8> {
            (0..19u32)
                .flat_map(|x| {
                    let alpha = if x % 5 == 0 {
                        0
                    } else {
                        ((x * 37 + seed) % 256) as u8
                    };
                    [
                        ((x * 19 + seed) % (u32::from(alpha) + 1)) as u8,
                        ((x * 29 + seed) % (u32::from(alpha) + 1)) as u8,
                        ((x * 43 + seed) % (u32::from(alpha) + 1)) as u8,
                        alpha,
                    ]
                })
                .collect()
        };
        let upper = row(17);
        let lower = row(61);
        // Effect close uses the final frame's zero-X invariant. Compare its
        // fused BGRX store with the original scalar SrcOver, including zero
        // source pixels and a vector tail.
        let source: Vec<u32> = upper
            .chunks_exact(4)
            .map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]]))
            .collect();
        let mut actual = vec![0x00734a25; source.len()];
        let expected: Vec<u32> = actual
            .iter()
            .zip(&source)
            .map(|(&dst, &src)| source_over_rgba_word(dst, src, skia::PixelFormat::Bgrx8888))
            .collect();
        blend_compositor_row(&mut actual, &source, skia::PixelFormat::Bgrx8888);
        assert_eq!(actual, expected);
        // Exercise fractional production BGRX with the actual zero-X proof,
        // exact transparent source pixels, SIMD blocks and a scalar tail.
        let mut upper_final = upper.clone();
        let mut lower_final = lower.clone();
        // Nine source pixels make the first eight H-filtered pixels exactly
        // zero, exercising complete fixed/general source blocks, then a tail.
        upper_final[..36].fill(0);
        lower_final[..36].fill(0);
        fn check_compositor(
            upper: &[u8],
            lower: &[u8],
            phase: (f64, f64),
            format: skia::PixelFormat,
            destination: u32,
        ) {
            let mut expected = vec![destination; 17];
            let mut actual = expected.clone();
            for (x, dst) in expected.iter_mut().enumerate() {
                let sampled = linear_word::<true, true>(
                    rgba_word(upper, x),
                    rgba_word(upper, x + 1),
                    rgba_word(lower, x),
                    rgba_word(lower, x + 1),
                    phase,
                    (1.0 - phase.0, 1.0 - phase.1),
                );
                *dst = source_over_rgba_word(*dst, sampled, format);
            }
            assert!(fractional_fixed_span::<true, true, false, true>(
                &mut actual,
                upper,
                lower,
                0,
                phase,
                format
            ));
            assert_eq!(actual, expected);
        }
        check_compositor(
            &upper_final,
            &lower_final,
            (0.375, 0.3125),
            skia::PixelFormat::Bgrx8888,
            0x00734a25,
        );
        // The general f64 sampler uses eight deinterleaved source registers;
        // cover its fused RGBA blend while retaining source zeroes and a tail.
        check_compositor(
            &upper_final,
            &lower_final,
            (0.1, 0.2),
            skia::PixelFormat::Rgba8888,
            0x80734a25,
        );
        fn check<const H: bool, const V: bool, const OPAQUE: bool>(
            upper: &[u8],
            lower: &[u8],
            phase: (f64, f64),
            format: skia::PixelFormat,
        ) {
            // Retain the original floating-point sampler and scalar blend as
            // the independent reference, including a vector tail and raw X.
            let mut expected = vec![0xb8734a25; 17];
            let mut actual = expected.clone();
            for (x, dst) in expected.iter_mut().enumerate() {
                let a = rgba_word(upper, x);
                let b = if H { rgba_word(upper, x + 1) } else { a };
                let c = if V { rgba_word(lower, x) } else { a };
                let d = if H && V {
                    rgba_word(lower, x + 1)
                } else if V {
                    c
                } else {
                    b
                };
                store_linear::<OPAQUE>(
                    dst,
                    linear_word::<H, V>(a, b, c, d, phase, (1.0 - phase.0, 1.0 - phase.1)),
                    format,
                );
            }
            assert!(fractional_fixed_span::<H, V, OPAQUE, false>(
                &mut actual,
                upper,
                lower,
                0,
                phase,
                format
            ));
            assert_eq!(
                actual, expected,
                "phase={phase:?} opaque={OPAQUE} format={format:?}"
            );
            // Production's compiled program uses the actual final-target X=0
            // invariant. Cover exact-sized source/output tails independently
            // against scalar sampling + SrcOver, rather than the vector body.
            let sampler = PreparedFractionalSampler::for_tile::<H, V, OPAQUE>(phase, format);
            for count in [1usize, 2, 3, 7, 17] {
                let destination = if format == skia::PixelFormat::Bgrx8888 {
                    0x00734a25
                } else {
                    0xb8734a25
                };
                let mut expected = vec![destination; count];
                let mut actual = expected.clone();
                for (x, dst) in expected.iter_mut().enumerate() {
                    let a = rgba_word(upper, x);
                    let b = if H { rgba_word(upper, x + 1) } else { a };
                    let c = if V { rgba_word(lower, x) } else { a };
                    let d = if H && V {
                        rgba_word(lower, x + 1)
                    } else if V {
                        c
                    } else {
                        b
                    };
                    store_linear::<OPAQUE>(
                        dst,
                        linear_word::<H, V>(a, b, c, d, phase, (1.0 - phase.0, 1.0 - phase.1)),
                        format,
                    );
                }
                let extent = (count + usize::from(H)) * 4;
                assert!(fractional_compiled_span::<H, V, OPAQUE>(
                    &mut actual,
                    &upper[..extent],
                    &lower[..extent],
                    0,
                    &sampler,
                    format
                ));
                assert_eq!(
                    actual, expected,
                    "compiled count={count} phase={phase:?} opaque={OPAQUE} format={format:?}"
                );
            }
        }
        for format in [
            skia::PixelFormat::Rgba8888,
            skia::PixelFormat::Bgra8888,
            skia::PixelFormat::Bgrx8888,
        ] {
            check::<true, false, false>(&upper, &lower, (0.375, 0.0), format);
            check::<false, true, false>(&upper, &lower, (0.0, 0.5), format);
            check::<true, true, false>(&upper, &lower, (0.375, 0.3125), format);
            check::<true, true, true>(&upper, &lower, (0.375, 0.3125), format);
            for phase in [
                (0.1, 0.2),
                (1.0 / 3.0, 7.0 / 9.0),
                (
                    f64::from_bits(0.5f64.to_bits() - 1),
                    f64::from_bits(0.5f64.to_bits() + 1),
                ),
            ] {
                check::<true, false, false>(&upper, &lower, (phase.0, 0.0), format);
                check::<false, true, false>(&upper, &lower, (0.0, phase.1), format);
                check::<true, true, false>(&upper, &lower, phase, format);
                check::<true, true, true>(&upper, &lower, phase, format);
            }
        }
    }
}

fn fractional_row_prepared<
    const H: bool,
    const V: bool,
    const OPAQUE: bool,
    const COMPOSITOR: bool,
>(
    output: &mut [u32],
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    sy: usize,
    sx: usize,
    sampler: &PreparedFractionalSampler,
    inverse: (f64, f64),
    format: skia::PixelFormat,
) -> Option<(usize, usize)> {
    fractional_row_execute::<H, V, OPAQUE, COMPOSITOR, false>(
        output, pixels, adjacent, sy, sx, sampler, inverse, format,
    )
}

fn fractional_row_execute<
    const H: bool,
    const V: bool,
    const OPAQUE: bool,
    const COMPOSITOR: bool,
    const COMPILED: bool,
>(
    output: &mut [u32],
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    sy: usize,
    sx: usize,
    sampler: &PreparedFractionalSampler,
    inverse: (f64, f64),
    format: skia::PixelFormat,
) -> Option<(usize, usize)> {
    let phase = sampler.phase;
    let width = pixels.pixel_size.0 as usize;
    let height = pixels.pixel_size.1 as usize;
    let (below, below_y) = if V && sy + 1 >= height {
        match adjacent[1] {
            Some(next) => (next, 0),
            None => (pixels, height - 1),
        }
    } else {
        (pixels, if V { sy + 1 } else { sy })
    };
    let lower_width = below.pixel_size.0 as usize;
    let mut support = pixels.row_support.rows[sy].extent;
    if V {
        let mut next = below.row_support.rows[below_y].extent;
        // Clamping a narrow neighbor's last nonzero pixel can extend its
        // support through the current tile's remaining columns.
        if next.0 < next.1 && next.1 == lower_width {
            next.1 = width.max(next.1);
        }
        if next.0 < next.1 {
            support = if support.0 == support.1 {
                next
            } else {
                (support.0.min(next.0), support.1.max(next.1))
            };
        }
    }
    if H {
        if support.0 < support.1 {
            support.0 = support.0.saturating_sub(1);
        }
        if tile_edge_pixel(pixels, adjacent, width, sy) != 0
            || (V && tile_edge_pixel(pixels, adjacent, width, sy + 1) != 0)
        {
            support = if support.0 == support.1 {
                (width - 1, width)
            } else {
                (support.0.min(width - 1), support.1.max(width))
            };
        }
    }
    let first = sx.max(support.0);
    let last = (sx + output.len()).min(support.1);
    if first >= last {
        return None;
    }
    // SOLID_COLOR_MODE has no backing resource. A scalar edge path preserves
    // the same bilinear footprint when a solid meets a differently colored
    // resource; the common all-solid case was already handled as one fill.
    if compact_solid_color(pixels).is_some() || compact_solid_color(below).is_some() {
        for x in first..last {
            let a = tile_edge_pixel(pixels, adjacent, x, sy);
            let b = if H {
                tile_edge_pixel(pixels, adjacent, x + 1, sy)
            } else {
                a
            };
            let c = if V {
                tile_edge_pixel(pixels, adjacent, x, sy + 1)
            } else {
                a
            };
            let d = if H && V {
                tile_edge_pixel(pixels, adjacent, x + 1, sy + 1)
            } else if V {
                c
            } else {
                b
            };
            let sampled = linear_word::<H, V>(a, b, c, d, phase, inverse);
            store_linear::<OPAQUE>(&mut output[x - sx], sampled, format);
        }
        return Some((first - sx, last - sx));
    }
    let upper = &pixels.rgba[sy * width * 4..][..width * 4];
    let lower = &below.rgba[below_y * lower_width * 4..][..lower_width * 4];
    let interior_end = if H { last.min(width - 1) } else { last };
    // SkRasterPipelineBlitter.cpp strength-reduces SrcOver to Src for an
    // opaque source. Reuse the
    // immutable product's row proof for the complete bilinear footprint;
    // transparent margins, holes and the actual neighbor edge keep SrcOver.
    // A narrow lower tile clamps its last opaque pixel beyond its width.
    let upper_support = pixels.row_support.rows[sy];
    let below_support = below.row_support.rows[below_y];
    let opaque_span = if !OPAQUE
        && sampler.valid
        && (COMPILED || inverse == sampler.inverse)
        && upper_support.opaque
        && (!V || below_support.opaque)
    {
        let left = upper_support
            .extent
            .0
            .max(if V { below_support.extent.0 } else { 0 });
        let right = upper_support.extent.1.saturating_sub(usize::from(H)).min(
            if !V || below_support.extent.1 == lower_width {
                width
            } else {
                below_support.extent.1.saturating_sub(usize::from(H))
            },
        );
        Some((left, right))
    } else {
        None
    };
    if first < interior_end {
        let lower_support = if V {
            below.row_support.spans(below_y)
        } else {
            &[]
        };
        let mut draw_span = |first: usize, end: usize, opaque: bool| {
            if first >= end {
                return;
            }
            let input = &upper[first * 4..end * 4];
            let destination = &mut output[first - sx..end - sx];
            let processed = if opaque {
                if COMPILED {
                    fractional_compiled_span::<H, V, true>(
                        destination,
                        upper,
                        lower,
                        first,
                        sampler,
                        format,
                    )
                } else {
                    fractional_fixed_span_prepared::<H, V, true, COMPOSITOR>(
                        destination,
                        upper,
                        lower,
                        first,
                        sampler,
                        format,
                    )
                }
            } else if COMPILED {
                fractional_compiled_span::<H, V, OPAQUE>(
                    destination,
                    upper,
                    lower,
                    first,
                    sampler,
                    format,
                )
            } else {
                fractional_fixed_span_prepared::<H, V, OPAQUE, COMPOSITOR>(
                    destination,
                    upper,
                    lower,
                    first,
                    sampler,
                    format,
                )
            };
            if !processed {
                for (i, (dst, bytes)) in destination
                    .iter_mut()
                    .zip(input.chunks_exact(4))
                    .enumerate()
                {
                    let x = first + i;
                    let a = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    let b = if H { rgba_word(upper, x + 1) } else { a };
                    let c = if V {
                        rgba_word(lower, x.min(lower_width - 1))
                    } else {
                        a
                    };
                    let d = if H && V {
                        rgba_word(lower, (x + 1).min(lower_width - 1))
                    } else if V {
                        c
                    } else {
                        b
                    };
                    let sampled = linear_word::<H, V>(a, b, c, d, phase, inverse);
                    if opaque {
                        store_linear::<true>(dst, sampled, format);
                    } else {
                        store_linear::<OPAQUE>(dst, sampled, format);
                    }
                }
            }
        };
        for_fractional_spans::<H>(
            pixels.row_support.spans(sy),
            lower_support,
            width,
            lower_width,
            |span_left, span_right| {
                let first = first.max(span_left);
                let end = interior_end.min(span_right);
                if first >= end {
                    return;
                }
                if let Some((opaque_left, opaque_right)) = opaque_span {
                    let left = first.max(opaque_left);
                    let right = end.min(opaque_right);
                    if left < right {
                        draw_span(first, left, false);
                        draw_span(left, right, true);
                        draw_span(right, end, false);
                        return;
                    }
                }
                draw_span(first, end, OPAQUE);
            },
        );
    }
    if H && last == width && first < width {
        let x = width - 1;
        let a = rgba_word(upper, x);
        let b = tile_edge_pixel(pixels, adjacent, width, sy);
        let c = if V {
            rgba_word(lower, x.min(lower_width - 1))
        } else {
            a
        };
        let d = if V {
            tile_edge_pixel(pixels, adjacent, width, sy + 1)
        } else {
            b
        };
        store_linear::<OPAQUE>(
            &mut output[x - sx],
            linear_word::<H, V>(a, b, c, d, phase, inverse),
            format,
        );
    }
    // Both the SIMD interior and scalar neighbor edge only write within this
    // existing support span, including horizontal/vertical neighbor expansion.
    Some((first - sx, last - sx))
}

// Tile texture filtering includes a transparent pixel before the first
// source column/row. Keep all nonnegative spans on the existing SIMD kernel;
// only this one-pixel outer fringe needs signed scalar sampling. Missing
// right/down resources retain the original clamping contract.
fn signed_tile_pixel(
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    x: i64,
    y: i64,
) -> u32 {
    if x < 0 || y < 0 {
        return 0;
    }
    tile_edge_pixel(pixels, adjacent, x as usize, y as usize)
}

fn fractional_row_signed_execute<
    const H: bool,
    const V: bool,
    const OPAQUE: bool,
    const COMPOSITOR: bool,
    const COMPILED: bool,
>(
    output: &mut [u32],
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    sy: i64,
    sx: i64,
    sampler: &PreparedFractionalSampler,
    format: skia::PixelFormat,
) -> Option<(usize, usize)> {
    debug_assert!(sx >= -1 && sy >= -1);
    if sx >= 0 && sy >= 0 {
        return fractional_row_execute::<H, V, OPAQUE, COMPOSITOR, COMPILED>(
            output,
            pixels,
            adjacent,
            sy as usize,
            sx as usize,
            sampler,
            sampler.inverse,
            format,
        );
    }
    // A negative row uses transparent upper samples and real source row zero;
    // a negative column needs just one scalar cell before the regular span.
    let count = if sy < 0 {
        output.len()
    } else {
        output.len().min(1)
    };
    let mut span = None;
    for (i, dst) in output[..count].iter_mut().enumerate() {
        let x = sx + i as i64;
        let a = signed_tile_pixel(pixels, adjacent, x, sy);
        let b = if H {
            signed_tile_pixel(pixels, adjacent, x + 1, sy)
        } else {
            a
        };
        let c = if V {
            signed_tile_pixel(pixels, adjacent, x, sy + 1)
        } else {
            a
        };
        let d = if H && V {
            signed_tile_pixel(pixels, adjacent, x + 1, sy + 1)
        } else if V {
            c
        } else {
            b
        };
        let sampled = linear_word::<H, V>(a, b, c, d, sampler.phase, sampler.inverse);
        if sampled != 0 {
            // The negative footprint contains transparency even when the
            // actual tile's complete backing is opaque.
            store_linear::<false>(dst, sampled, format);
            span = Some(span.map_or((i, i + 1), |(first, _)| (first, i + 1)));
        }
    }
    if count < output.len() {
        if let Some((first, last)) = fractional_row_execute::<H, V, OPAQUE, COMPOSITOR, COMPILED>(
            &mut output[count..],
            pixels,
            adjacent,
            sy as usize,
            (sx + count as i64) as usize,
            sampler,
            sampler.inverse,
            format,
        ) {
            span = Some(span.map_or((count + first, count + last), |(start, _)| {
                (start, count + last)
            }));
        }
    }
    span
}

fn fractional_row_signed<const OPAQUE: bool, const COMPOSITOR: bool>(
    output: &mut [u32],
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    sy: i64,
    sx: i64,
    sampler: &PreparedFractionalSampler,
    format: skia::PixelFormat,
) -> Option<(usize, usize)> {
    if sampler.phase.0 == 0.0 {
        fractional_row_signed_execute::<false, true, OPAQUE, COMPOSITOR, false>(
            output, pixels, adjacent, sy, sx, sampler, format,
        )
    } else if sampler.phase.1 == 0.0 {
        fractional_row_signed_execute::<true, false, OPAQUE, COMPOSITOR, false>(
            output, pixels, adjacent, sy, sx, sampler, format,
        )
    } else {
        fractional_row_signed_execute::<true, true, OPAQUE, COMPOSITOR, false>(
            output, pixels, adjacent, sy, sx, sampler, format,
        )
    }
}

#[cfg(test)]
fn fractional_row<const H: bool, const V: bool, const OPAQUE: bool, const COMPOSITOR: bool>(
    output: &mut [u32],
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    sy: usize,
    sx: usize,
    phase: (f64, f64),
    inverse: (f64, f64),
    format: skia::PixelFormat,
) -> Option<(usize, usize)> {
    // Keep the generic raw-X and caller-supplied scalar-edge inverse contract.
    fractional_row_prepared::<H, V, OPAQUE, COMPOSITOR>(
        output,
        pixels,
        adjacent,
        sy,
        sx,
        &PreparedFractionalSampler::new(phase),
        inverse,
        format,
    )
}

// Only compose_tile enters this production path. Its final BGRX frame starts
// with X=0 and all writes preserve X=0; isolated effect targets are RGBA. Keep
// generic fractional_row/span callers on COMPOSITOR=false for arbitrary raw X.
fn fractional_tile<const OPAQUE: bool>(
    target: &mut [u32],
    view: TargetView,
    bounds: SurfaceBounds,
    origin: (i64, i64),
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    phase: (f64, f64),
    format: skia::PixelFormat,
) -> Option<SurfaceBounds> {
    if phase.0 == 0.0 {
        fractional_tile_axes::<false, true, OPAQUE>(
            target, view, bounds, origin, pixels, adjacent, phase, format,
        )
    } else if phase.1 == 0.0 {
        fractional_tile_axes::<true, false, OPAQUE>(
            target, view, bounds, origin, pixels, adjacent, phase, format,
        )
    } else {
        fractional_tile_axes::<true, true, OPAQUE>(
            target, view, bounds, origin, pixels, adjacent, phase, format,
        )
    }
}

fn fractional_tile_axes<const H: bool, const V: bool, const OPAQUE: bool>(
    target: &mut [u32],
    view: TargetView,
    bounds: SurfaceBounds,
    origin: (i64, i64),
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    phase: (f64, f64),
    format: skia::PixelFormat,
) -> Option<SurfaceBounds> {
    let sampler = PreparedFractionalSampler::for_tile::<H, V, OPAQUE>(phase, format);
    let mut touched = None;
    for y in bounds.top..bounds.bottom {
        let output = &mut target[view.row_range(y, bounds.left, bounds.right)];
        let sy = y as i64 - origin.1;
        let sx = bounds.left as i64 - origin.0;
        let span = fractional_row_signed_execute::<H, V, OPAQUE, true, true>(
            output, pixels, adjacent, sy, sx, &sampler, format,
        );
        if !OPAQUE {
            if let Some((first, last)) = span {
                union_bounds(
                    &mut touched,
                    Some(SurfaceBounds {
                        left: bounds.left + first,
                        right: bounds.left + last,
                        top: y,
                        bottom: y + 1,
                    }),
                );
            }
        }
    }
    if OPAQUE {
        Some(bounds)
    } else {
        touched
    }
}

fn tile_composition_geometry(
    tile: &TilePlacement,
    composition: LayerComposition,
    width: u32,
    height: u32,
) -> io::Result<Option<(SurfaceBounds, (i64, i64), (f64, f64))>> {
    let origin_x =
        device_position(tile.tile_rect.x * tile.raster_scale)? as f64 + composition.translation.0;
    let origin_y =
        device_position(tile.tile_rect.y * tile.raster_scale)? as f64 + composition.translation.1;
    if !origin_x.is_finite()
        || !origin_y.is_finite()
        || origin_x.abs() > i32::MAX as f64
        || origin_y.abs() > i32::MAX as f64
    {
        return Err(io::Error::other("invalid composition placement"));
    }
    let x = origin_x.ceil() as i64;
    let y = origin_y.ceil() as i64;
    let phase = (x as f64 - origin_x, y as f64 - origin_y);
    // The source grid may start exactly at recorded ink after a new layer is
    // allocated. Linear filtering must still sample its transparent negative
    // neighbor. Retained grids with padding already own this same output cell
    // in their ordinary tile interior. Extend only the first row/column, so no
    // internal seam is blended twice. This shared geometry also sizes effect
    // targets and the direct Mask partition.
    let leading = (!composition.white_backing)
        .then_some(composition.grid_min)
        .flatten();
    let mut left =
        (x - i64::from(phase.0 != 0.0 && leading.is_some_and(|min| tile.tile_index.0 == min.0)))
            .max(0);
    let mut top =
        (y - i64::from(phase.1 != 0.0 && leading.is_some_and(|min| tile.tile_index.1 == min.1)))
            .max(0);
    let mut right = (x + tile.pixel_size.0 as i64).min(width as i64);
    let mut bottom = (y + tile.pixel_size.1 as i64).min(height as i64);
    // cc CoverageIterator's geometryRect is limited by the recorded content
    // bounds as well as the external compositor scissor. Keep texture origin,
    // phase and retained neighbor resources unchanged for the visible quad.
    for clip in [composition.clip, composition.content_bounds]
        .into_iter()
        .flatten()
    {
        left = left.max(clip.x as i64);
        top = top.max(clip.y as i64);
        right = right.min(clip.x as i64 + clip.width as i64);
        bottom = bottom.min(clip.y as i64 + clip.height as i64);
    }
    if left >= right || top >= bottom {
        return Ok(None);
    }
    let bounds = SurfaceBounds {
        left: left as usize,
        top: top as usize,
        right: right as usize,
        bottom: bottom as usize,
    };
    Ok(Some((bounds, (x, y), phase)))
}

// SkSwizzler_opts.inc::RGBA_to_BGRA (370-400): an opaque Src span needs
// only source loads and destination stores, including an unaligned source.
fn copy_opaque_rgba_row(output: &mut [u32], input: &[u8], format: skia::PixelFormat) {
    debug_assert_eq!(input.len(), output.len() * 4);
    if format == skia::PixelFormat::Rgba8888 {
        // SAFETY: complete initialized N32 output, exclusively borrowed.
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                output.as_mut_ptr().cast::<u8>(),
                std::mem::size_of_val(output),
            )
        };
        bytes.copy_from_slice(input);
        return;
    }
    #[allow(unused_mut)]
    let mut at = 0;
    #[cfg(target_arch = "aarch64")]
    {
        use core::arch::aarch64::*;
        // SkSwizzler's 8888 channel permutation stays in packed vectors. Src
        // needs no deinterleave/reinterleave or destination load: use the same
        // packed representation as SkOpts' opaque source block.
        unsafe {
            let swap = vld1q_u8([2u8, 1, 0, 3, 6, 5, 4, 7, 10, 9, 8, 11, 14, 13, 12, 15].as_ptr());
            let keep = vdupq_n_u32(if format == skia::PixelFormat::Bgrx8888 {
                0x00ff_ffff
            } else {
                u32::MAX
            });
            while at + 16 <= output.len() {
                for block in 0..4usize {
                    let offset = (at + block * 4) * 4;
                    let rgba = vld1q_u8(input.as_ptr().add(offset));
                    let bgra = vreinterpretq_u8_u32(vandq_u32(
                        vreinterpretq_u32_u8(vqtbl1q_u8(rgba, swap)),
                        keep,
                    ));
                    vst1q_u8(output.as_mut_ptr().add(at + block * 4).cast::<u8>(), bgra);
                }
                at += 16;
            }
            if at + 8 <= output.len() {
                for block in 0..2usize {
                    let offset = (at + block * 4) * 4;
                    let rgba = vld1q_u8(input.as_ptr().add(offset));
                    let bgra = vreinterpretq_u8_u32(vandq_u32(
                        vreinterpretq_u32_u8(vqtbl1q_u8(rgba, swap)),
                        keep,
                    ));
                    vst1q_u8(output.as_mut_ptr().add(at + block * 4).cast::<u8>(), bgra);
                }
                at += 8;
            }
        }
    }
    let keep = if format == skia::PixelFormat::Bgrx8888 {
        0x0000_ff00
    } else {
        0xff00_ff00
    };
    for (pixel, bytes) in output[at..].iter_mut().zip(input[at * 4..].chunks_exact(4)) {
        let word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        *pixel = (word & keep) | ((word & 0xff) << 16) | ((word >> 16) & 0xff);
    }
}

pub(crate) fn cached_bgra(rgba: &[u8]) -> Vec<u32> {
    let mut bgra = vec![0; rgba.len() / 4];
    copy_opaque_rgba_row(&mut bgra, rgba, skia::PixelFormat::Bgra8888);
    bgra
}

#[cfg(test)]
#[test]
fn opaque_copy_writes_complete_pixels_in_all_output_formats() {
    for columns in [0, 1, 7, 8, 9, 15, 16, 17, 24, 31, 32, 257] {
        // Exercise an unaligned source and vector/tail boundaries. The guard
        // words must survive the write into the middle of the target row.
        let mut source = vec![0x7b; 1 + columns * 4];
        for (i, pixel) in source[1..].chunks_exact_mut(4).enumerate() {
            pixel.copy_from_slice(&[i as u8, (i * 3) as u8, (i * 7) as u8, 255]);
        }
        for format in [
            skia::PixelFormat::Rgba8888,
            skia::PixelFormat::Bgra8888,
            skia::PixelFormat::Bgrx8888,
        ] {
            let mut target = vec![0x1234_5678; columns + 2];
            copy_opaque_rgba_row(&mut target[1..1 + columns], &source[1..], format);
            assert_eq!(target[0], 0x1234_5678);
            assert_eq!(target[columns + 1], 0x1234_5678);
            for (i, actual) in target[1..1 + columns].iter().enumerate() {
                let [r, g, b] = [i as u8, (i * 3) as u8, (i * 7) as u8];
                let expected = match format {
                    skia::PixelFormat::Rgba8888 => [r, g, b, 255],
                    skia::PixelFormat::Bgra8888 => [b, g, r, 255],
                    skia::PixelFormat::Bgrx8888 => [b, g, r, 0],
                };
                assert_eq!(actual.to_le_bytes(), expected);
            }
        }
    }
}

fn opaque_covers(composition: LayerComposition, bounds: SurfaceBounds) -> bool {
    composition.opaque_bounds.is_some_and(|rect| {
        rect.x as i64 <= bounds.left as i64
            && rect.y as i64 <= bounds.top as i64
            && i64::from(rect.x) + i64::from(rect.width) >= bounds.right as i64
            && i64::from(rect.y) + i64::from(rect.height) >= bounds.bottom as i64
    })
}

fn compose_tile(
    target: &mut [u32],
    width: u32,
    height: u32,
    view: TargetView,
    format: skia::PixelFormat,
    tile: &TilePlacement,
    composition: LayerComposition,
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    draw_visible: Option<SurfaceBounds>,
) -> io::Result<Option<SurfaceBounds>> {
    let Some((mut bounds, (x, y), phase)) =
        tile_composition_geometry(tile, composition, width, height)?
    else {
        return Ok(None);
    };
    // DrawTileQuad maps visible_rect back to the unchanged source UVs. Keep
    // the original tile origin/phase and shorten only the destination extent.
    if let Some(visible) = draw_visible {
        bounds.left = bounds.left.max(visible.left);
        bounds.top = bounds.top.max(visible.top);
        bounds.right = bounds.right.min(visible.right);
        bounds.bottom = bounds.bottom.min(visible.bottom);
        if bounds.left >= bounds.right || bounds.top >= bounds.bottom {
            return Ok(None);
        }
    }
    let (left, top, right, bottom) = (
        bounds.left as i64,
        bounds.top as i64,
        bounds.right as i64,
        bounds.bottom as i64,
    );
    let touched = Some(bounds);
    let has_leading_fringe = left < x || top < y;
    if !has_leading_fringe
        && layer_solid::try_compose(target, view, bounds, format, pixels, adjacent, phase)
    {
        return Ok(touched);
    }
    if !valid_tile_storage(pixels, tile.pixel_size)? {
        return Err(io::Error::other("invalid cached tile extent"));
    }
    let rgba = &pixels.rgba;
    let source_stride = tile.pixel_size.0 as usize * 4;
    let columns = (right - left) as usize;
    if phase != (0.0, 0.0) {
        // Each phase has one row kernel; only the true tile edges consult
        // adjacent pixels. Interior loads use contiguous preselected rows.
        // Missing neighbors use opaque edge clamping. Any real neighbor
        // participating in interpolation must carry the same exact proof.
        // Only actual shader samples affect opacity: H reads the right tile,
        // V reads the lower tile, and their conjunction reads the diagonal.
        let required = [
            phase.0 != 0.0,
            phase.1 != 0.0,
            phase.0 != 0.0 && phase.1 != 0.0,
        ];
        let opaque = !has_leading_fringe
            && (opaque_covers(composition, bounds)
                || ((composition.white_backing || pixels.row_support.opaque)
                    && adjacent.iter().zip(required).all(|(next, required)| {
                        !required
                            || next.is_none_or(|next| next.white_backing || next.row_support.opaque)
                    })));
        let bounds = touched.expect("nonempty tile composition");
        return Ok(if opaque {
            fractional_tile::<true>(
                target,
                view,
                bounds,
                (x, y),
                pixels,
                adjacent,
                phase,
                format,
            )
        } else {
            fractional_tile::<false>(
                target,
                view,
                bounds,
                (x, y),
                pixels,
                adjacent,
                phase,
                format,
            )
        });
    }
    if composition.white_backing || pixels.row_support.opaque || opaque_covers(composition, bounds)
    {
        // Either the backing guarantees opacity or the tile's actual pixels
        // prove it. SrcOver then equals copying/swizzling this clipped row.
        let native_bgra =
            matches!(format, skia::PixelFormat::Bgra8888) && pixels.bgra.len() == rgba.len() / 4;
        for row in top..bottom {
            let source = (row - y) as usize * source_stride + (left - x) as usize * 4;
            let destination = view.row_start(row as usize, left as usize);
            let output = &mut target[destination..destination + columns];
            if native_bgra {
                let source = source / 4;
                output.copy_from_slice(&pixels.bgra[source..source + columns]);
            } else {
                let input = &rgba[source..source + columns * 4];
                copy_opaque_rgba_row(output, input, format);
            }
        }
        return Ok(touched);
    }
    let mut written = None;
    for row in top..bottom {
        let sy = (row - y) as usize;
        let (support_left, support_right) = pixels.row_support.rows[sy].extent;
        let first = ((left - x) as usize).max(support_left);
        let last = ((right - x) as usize).min(support_right);
        if first >= last {
            continue;
        }
        // The cached runs exclude only complete RGBA-zero holes. Integer
        // placement can skip the same proven no-op pixels as interpolation.
        for &(span_left, span_right) in pixels.row_support.spans(sy) {
            let span_left = first.max(span_left);
            let span_right = last.min(span_right);
            if span_left >= span_right {
                continue;
            }
            let source = sy * source_stride + span_left * 4;
            let destination = view.row_start(row as usize, (x + span_left as i64) as usize);
            let input = &rgba[source..source + (span_right - span_left) * 4];
            let output = &mut target[destination..destination + span_right - span_left];
            // SoftwareRenderer/SkBlitRow: opaque SrcOver equals Src. The
            // immutable product proves every byte alpha in this row extent;
            // no destination read or per-frame alpha scan is needed. Filtered
            // fractional samples keep their separate complete-footprint proof.
            if pixels.row_support.rows[sy].opaque {
                if matches!(format, skia::PixelFormat::Bgra8888)
                    && pixels.bgra.len() == rgba.len() / 4
                {
                    let source = source / 4;
                    output.copy_from_slice(&pixels.bgra[source..source + output.len()]);
                } else {
                    copy_opaque_rgba_row(output, input, format);
                }
            } else {
                // SAFETY: exclusive initialized complete N32 pixel row; no
                // alias survives this little-endian byte view.
                let output_bytes = unsafe {
                    std::slice::from_raw_parts_mut(
                        output.as_mut_ptr().cast::<u8>(),
                        std::mem::size_of_val(output),
                    )
                };
                skia::blend_premultiplied_rgba_row(output_bytes, input, format);
            }
        }
        union_bounds(
            &mut written,
            Some(SurfaceBounds {
                left: (x + first as i64) as usize,
                right: (x + last as i64) as usize,
                top: row as usize,
                bottom: row as usize + 1,
            }),
        );
    }
    Ok(written)
}

#[inline]
fn source_over_word(destination: u32, source: [u8; 4], format: skia::PixelFormat) -> u32 {
    source_over_rgba_word(destination, u32::from_le_bytes(source), format)
}

#[inline]
fn source_over_rgba_word(destination: u32, mut source: u32, format: skia::PixelFormat) -> u32 {
    if source == 0 {
        return destination;
    }
    let inverse = 255 - (source >> 24);
    if format != skia::PixelFormat::Rgba8888 {
        source = (source & 0xff00_ff00) | ((source & 0xff) << 16) | ((source >> 16) & 0xff);
    }
    if inverse != 0 {
        // Same rounded div255 as multiply_255, two isolated 16-bit lanes
        // at a time. Premultiplied source-over cannot overflow a byte.
        let mut rb = (destination & 0x00ff_00ff) * inverse + 0x0080_0080;
        rb = ((rb + ((rb >> 8) & 0x00ff_00ff)) >> 8) & 0x00ff_00ff;
        let mut ga = ((destination >> 8) & 0x00ff_00ff) * inverse + 0x0080_0080;
        ga = ((ga + ((ga >> 8) & 0x00ff_00ff)) >> 8) & 0x00ff_00ff;
        source += rb | (ga << 8);
    }
    if format == skia::PixelFormat::Bgrx8888 {
        source &= 0x00ff_ffff;
    }
    source
}

#[inline]
fn scale_mask_word(source: u32, coverage: u8) -> u32 {
    let coverage = u32::from(coverage);
    // Two independent 16-bit lanes preserve multiply_255's rounded div255.
    // Each product + rounding adjustment stays below 65536, so no lane carries.
    let mut rb = (source & 0x00ff_00ff) * coverage + 0x0080_0080;
    rb = ((rb + ((rb >> 8) & 0x00ff_00ff)) >> 8) & 0x00ff_00ff;
    let mut ga = ((source >> 8) & 0x00ff_00ff) * coverage + 0x0080_0080;
    ga = ((ga + ((ga >> 8) & 0x00ff_00ff)) >> 8) & 0x00ff_00ff;
    rb | (ga << 8)
}

fn blend_rgba_words(target: &mut [u32], source: &[u32], format: skia::PixelFormat) {
    // SAFETY: both rows contain initialized complete N32 words. Their byte
    // views have identical extents; target is exclusively borrowed and the
    // independently allocated source surface/scratch cannot alias it.
    let destination = unsafe {
        std::slice::from_raw_parts_mut(
            target.as_mut_ptr().cast::<u8>(),
            std::mem::size_of_val(target),
        )
    };
    let rgba = unsafe {
        std::slice::from_raw_parts(source.as_ptr().cast::<u8>(), std::mem::size_of_val(source))
    };
    skia::blend_premultiplied_rgba_row(destination, rgba, format);
}

fn blend_group_row(target: &mut [u32], source: &[u32], format: skia::PixelFormat) {
    if format == skia::PixelFormat::Bgrx8888 {
        // General callers can retain arbitrary raw X at an exact-zero source.
        blend_rgba_words(target, source, skia::PixelFormat::Bgra8888);
        for (dst, &src) in target.iter_mut().zip(source) {
            if src != 0 {
                *dst &= 0x00ff_ffff;
            }
        }
    } else {
        blend_rgba_words(target, source, format);
    }
}

fn blend_compositor_row(target: &mut [u32], source: &[u32], format: skia::PixelFormat) {
    #[cfg(feature = "compose_work_profile")]
    layer_compose_profile::blend(target.len());
    // Called only for final-frame/effect surfaces. Final BGRX starts with
    // X=0 and all admitted composition writes preserve X=0; effect parents
    // are RGBA. The existing BGRX SIMD store clears X in its blend pass,
    // avoiding the general helper's second complete source/destination scan.
    blend_rgba_words(target, source, format);
}

#[cfg(target_arch = "aarch64")]
unsafe fn scale_effect_neon(
    output: &mut [u32],
    input: &[u32],
    opacity: f32,
    coverage: Option<&[u8]>,
) -> (usize, u32) {
    use core::arch::aarch64::*;
    unsafe fn alpha_half(bytes: uint8x8_t, opacity: f32) -> uint8x8_t {
        unsafe {
            let wide = vmovl_u8(bytes);
            // FCVTN rounds the same f32 product to nearest, ties-even. Do not
            // replace opacity with an 8-bit factor or fuse it with mask alpha.
            let low = vcvtnq_u32_f32(vmulq_n_f32(
                vcvtq_f32_u32(vmovl_u16(vget_low_u16(wide))),
                opacity,
            ));
            let high = vcvtnq_u32_f32(vmulq_n_f32(
                vcvtq_f32_u32(vmovl_u16(vget_high_u16(wide))),
                opacity,
            ));
            vmovn_u16(vcombine_u16(vmovn_u32(low), vmovn_u32(high)))
        }
    }
    unsafe fn mask_half(bytes: uint8x8_t, alpha: uint8x8_t) -> uint8x8_t {
        unsafe {
            let product = vmull_u8(bytes, alpha);
            vraddhn_u16(product, vrshrq_n_u16::<8>(product))
        }
    }
    let mut at = 0;
    let mut any = 0;
    // Caller provides complete, equally sized pixel rows and coverage spans.
    // Unaligned loads/stores stay within four complete pixels; masks read only
    // their four real coverage bytes. Source and scratch are separate buffers.
    unsafe {
        let repeat = vld1q_u8([0u8, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3].as_ptr());
        let mut seen = vdupq_n_u32(0);
        while at + 4 <= input.len() {
            let mut bytes = vreinterpretq_u8_u32(vld1q_u32(input.as_ptr().add(at)));
            if opacity != 1.0 {
                bytes = vcombine_u8(
                    alpha_half(vget_low_u8(bytes), opacity),
                    alpha_half(vget_high_u8(bytes), opacity),
                );
            }
            if let Some(mask) = coverage {
                let four = std::ptr::read_unaligned(mask.as_ptr().add(at).cast::<u32>());
                let alpha = vqtbl1q_u8(vreinterpretq_u8_u32(vdupq_n_u32(four)), repeat);
                bytes = vcombine_u8(
                    mask_half(vget_low_u8(bytes), vget_low_u8(alpha)),
                    mask_half(vget_high_u8(bytes), vget_high_u8(alpha)),
                );
            }
            let words = vreinterpretq_u32_u8(bytes);
            vst1q_u32(output.as_mut_ptr().add(at), words);
            seen = vorrq_u32(seen, words);
            at += 4;
        }
        any |= vmaxvq_u32(seen);
    }
    (at, any)
}

fn scale_effect_row(
    output: &mut [u32],
    input: &[u32],
    opacity: f32,
    alpha: Option<&[u8; 256]>,
    mask: Option<&[u8]>,
) -> u32 {
    let mut at = 0;
    let mut any = 0;
    #[cfg(target_arch = "aarch64")]
    if opacity.is_finite() && (0.0..=1.0).contains(&opacity) {
        // SAFETY: all spans cover input.len() complete pixels, established by
        // the group row/chunk slices; the helper never crosses their ends.
        (at, any) = unsafe { scale_effect_neon(output, input, opacity, mask) };
    }
    for i in at..input.len() {
        let coverage = mask.map_or(255, |row| row[i]);
        let mut word = if input[i] == 0 || coverage == 0 {
            0
        } else {
            input[i]
        };
        if word != 0 {
            if let Some(alpha) = alpha {
                word = u32::from(alpha[(word & 255) as usize])
                    | (u32::from(alpha[((word >> 8) & 255) as usize]) << 8)
                    | (u32::from(alpha[((word >> 16) & 255) as usize]) << 16)
                    | (u32::from(alpha[(word >> 24) as usize]) << 24);
            }
            if coverage != 255 {
                word = scale_mask_word(word, coverage);
            }
        }
        output[i] = word;
        any |= word;
    }
    any
}

// Tile geometry is the actual clipped draw-quad partition. Clear everything
// else once, including a completely offscreen mask, before applying any quad.
fn clear_mask_exterior(
    target: &mut [u32],
    view: TargetView,
    touched: Option<SurfaceBounds>,
    rectangles: &[SurfaceBounds],
) -> Option<SurfaceBounds> {
    let bounds = touched?;
    let mut retained = None;
    for rect in rectangles {
        let overlap = SurfaceBounds {
            left: bounds.left.max(rect.left),
            top: bounds.top.max(rect.top),
            right: bounds.right.min(rect.right),
            bottom: bounds.bottom.min(rect.bottom),
        };
        if overlap.left < overlap.right && overlap.top < overlap.bottom {
            union_bounds(&mut retained, Some(overlap));
        }
    }
    let mut first = 0;
    for y in bounds.top..bounds.bottom {
        while first < rectangles.len() && rectangles[first].bottom <= y {
            first += 1;
        }
        let mut at = first;
        let mut cursor = bounds.left;
        while at < rectangles.len() && rectangles[at].top <= y {
            let rect = rectangles[at];
            let left = rect.left.clamp(bounds.left, bounds.right);
            let right = rect.right.clamp(bounds.left, bounds.right);
            if cursor < left {
                target[view.row_range(y, cursor, left)].fill(0);
            }
            cursor = cursor.max(right);
            at += 1;
        }
        if cursor < bounds.right {
            target[view.row_range(y, cursor, bounds.right)].fill(0);
        }
    }
    retained
}
fn dst_in_tile(
    target: &mut [u32],
    view: TargetView,
    touched: Option<SurfaceBounds>,
    width: u32,
    height: u32,
    tile: &TilePlacement,
    composition: LayerComposition,
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
) -> io::Result<()> {
    if !valid_tile_storage(pixels, tile.pixel_size)? {
        return Err(io::Error::other("invalid direct mask tile extent"));
    }
    let Some(parent) = touched else {
        return Ok(());
    };
    let Some((mut bounds, (x, y), phase)) =
        tile_composition_geometry(tile, composition, width, height)?
    else {
        return Ok(());
    };
    bounds.left = bounds.left.max(parent.left);
    bounds.right = bounds.right.min(parent.right);
    bounds.top = bounds.top.max(parent.top);
    bounds.bottom = bounds.bottom.min(parent.bottom);
    if bounds.left >= bounds.right || bounds.top >= bounds.bottom {
        return Ok(());
    }
    if phase == (0.0, 0.0) && compact_solid_color(pixels).is_some() {
        // The analyzer admits only opaque solids. DstIn(alpha=1) is identity
        // inside this tile; clear_mask_exterior already handled the outside.
        return Ok(());
    }
    let stride = tile.pixel_size.0 as usize * 4;
    let sampler = PreparedFractionalSampler::new(phase);
    for row in bounds.top..bounds.bottom {
        if phase != (0.0, 0.0) {
            let output = &mut target[view.row_range(row, bounds.left, bounds.right)];
            // Transparent sampler skips must remain A=0 for DstIn. Bounded
            // chunks preserve the same global source coordinate and neighbors.
            let sy = row as i64 - y;
            let first = bounds.left as i64 - x;
            for (chunk, output) in output.chunks_mut(256).enumerate() {
                let mut sampled = [0u32; 256];
                let sampled = &mut sampled[..output.len()];
                let sx = first + (chunk * 256) as i64;
                fractional_row_signed::<false, false>(
                    sampled,
                    pixels,
                    adjacent,
                    sy,
                    sx,
                    &sampler,
                    skia::PixelFormat::Rgba8888,
                );
                dst_in_row(output, sampled, 1.0);
            }
            continue;
        }
        let start = (row as i64 - y) as usize * stride + (bounds.left as i64 - x) as usize * 4;
        let input = &pixels.rgba[start..start + (bounds.right - bounds.left) * 4];
        let output = &mut target[view.row_range(row, bounds.left, bounds.right)];
        // Source RGBA bytes are initialized and independent of the destination.
        // Preserve the existing N32 DstIn arithmetic for either alignment.
        let (prefix, words, suffix) = unsafe { input.align_to::<u32>() };
        if prefix.is_empty() && suffix.is_empty() {
            dst_in_row(output, words, 1.0);
        } else {
            let mut scratch = [0u32; 256];
            for (dst, bytes) in output.chunks_mut(256).zip(input.chunks(256 * 4)) {
                for (word, pixel) in scratch.iter_mut().zip(bytes.chunks_exact(4)) {
                    *word = u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);
                }
                dst_in_row(dst, &scratch[..dst.len()], 1.0);
            }
        }
    }
    Ok(())
}

// SoftwareRenderer's Mask/kDstIn restore scales the entire isolated content
// by the source alpha, including transparent source outside the mask quads.
// Tiles first form one source surface; never apply DstIn independently per tile.
fn dst_in_effect_surface(
    target: &mut [u32],
    source: &[u32],
    target_touched: Option<SurfaceBounds>,
    source_touched: Option<SurfaceBounds>,
    target_view: TargetView,
    source_view: TargetView,
    opacity: f32,
) -> Option<SurfaceBounds> {
    let bounds = target_touched?;
    let overlap = source_touched
        .and_then(|source| {
            let clipped = SurfaceBounds {
                left: bounds.left.max(source.left),
                top: bounds.top.max(source.top),
                right: bounds.right.min(source.right),
                bottom: bounds.bottom.min(source.bottom),
            };
            (clipped.left < clipped.right && clipped.top < clipped.bottom).then_some(clipped)
        })
        .filter(|_| opacity != 0.0);
    for y in bounds.top..bounds.bottom {
        let row = &mut target[target_view.row_range(y, bounds.left, bounds.right)];
        if let Some(inside) = overlap.filter(|inside| y >= inside.top && y < inside.bottom) {
            let left = inside.left - bounds.left;
            let right = inside.right - bounds.left;
            row[..left].fill(0);
            row[right..].fill(0);
            let input = &source[source_view.row_range(y, inside.left, inside.right)];
            dst_in_row(&mut row[left..right], input, opacity);
        } else {
            row.fill(0);
        }
    }
    overlap
}
fn dst_in_row(target: &mut [u32], source: &[u32], opacity: f32) {
    #[allow(unused_mut)]
    let mut done = 0;
    #[cfg(target_arch = "aarch64")]
    if opacity == 1.0 {
        use core::arch::aarch64::*;
        // Corresponding N32 DstIn: round each destination channel * source A
        // with Skia's div255. The source is ordinary resident RGBA mask pixels.
        // SAFETY: equal complete rows, unaligned four-pixel loads stay in bounds.
        unsafe {
            let repeat =
                vld1q_u8([3u8, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15].as_ptr());
            while done + 4 <= target.len() {
                let dst = vreinterpretq_u8_u32(vld1q_u32(target.as_ptr().add(done)));
                let src = vreinterpretq_u8_u32(vld1q_u32(source.as_ptr().add(done)));
                let alpha = vqtbl1q_u8(src, repeat);
                let low = vmull_u8(vget_low_u8(dst), vget_low_u8(alpha));
                let high = vmull_u8(vget_high_u8(dst), vget_high_u8(alpha));
                let result = vcombine_u8(
                    vraddhn_u16(low, vrshrq_n_u16::<8>(low)),
                    vraddhn_u16(high, vrshrq_n_u16::<8>(high)),
                );
                vst1q_u32(target.as_mut_ptr().add(done), vreinterpretq_u32_u8(result));
                done += 4;
            }
        }
    }
    for (dst, &src) in target[done..].iter_mut().zip(&source[done..]) {
        let alpha = ((src >> 24) as f32 * opacity).round_ties_even() as u8;
        *dst = scale_mask_word(*dst, alpha);
    }
}

fn composite_effect_surface(
    target: &mut [u32],
    source: &[u32],
    touched: Option<SurfaceBounds>,
    target_view: TargetView,
    source_view: TargetView,
    format: skia::PixelFormat,
    opacity: f32,
    mask: Option<&MaskCoverage>,
) -> Option<SurfaceBounds> {
    let mut bounds = touched?;
    if opacity == 0.0 {
        return None;
    }
    if let Some(mask) = mask {
        let mask_bounds = mask.bounds?;
        bounds.left = bounds.left.max(mask_bounds.0);
        bounds.top = bounds.top.max(mask_bounds.1);
        bounds.right = bounds.right.min(mask_bounds.2);
        bounds.bottom = bounds.bottom.min(mask_bounds.3);
        if bounds.left >= bounds.right || bounds.top >= bounds.bottom {
            return None;
        }
    }
    // Same N32 restore mapping as SkCanvas: scale each premultiplied channel
    // in f32 and round ties-even, rather than quantizing opacity to an 8-bit alpha.
    let alpha: Option<[u8; 256]> = (opacity != 1.0)
        .then(|| std::array::from_fn(|v| (v as f32 * opacity).round_ties_even() as u8));
    // Fixed stack scratch, independent of viewport width or allocation pools.
    let mut scratch = [0u32; 256];
    for y in bounds.top..bounds.bottom {
        let (left, right) = mask.map_or((bounds.left, bounds.right), |mask| {
            let (left, right) = mask.rows[y - mask.origin.1];
            (bounds.left.max(left), bounds.right.min(right))
        });
        if left >= right {
            continue;
        }
        let target_range = target_view.row_range(y, left, right);
        let source_range = source_view.row_range(y, left, right);
        let coverage = mask.map(|mask| {
            let offset = (y - mask.origin.1) * mask.stride + left - mask.origin.0;
            &mask.pixels[offset..offset + right - left]
        });
        if alpha.is_none() && coverage.is_none() {
            blend_compositor_row(&mut target[target_range], &source[source_range], format);
            continue;
        }
        for (chunk, (output, input)) in target[target_range]
            .chunks_mut(256)
            .zip(source[source_range].chunks(256))
            .enumerate()
        {
            let mask = coverage.map(|row| &row[chunk * 256..chunk * 256 + input.len()]);
            if mask.is_some_and(|row| row.iter().all(|&v| v == 0)) {
                continue;
            }
            if alpha.is_none() && mask.is_none_or(|row| row.iter().all(|&v| v == 255)) {
                blend_compositor_row(output, input, format);
                continue;
            }
            let scaled = &mut scratch[..input.len()];
            // Opacity then true DstIn, preserving the original rounding at
            // both stages. SIMD only replaces the independent channel loop.
            let any = scale_effect_row(scaled, input, opacity, alpha.as_ref(), mask);
            if any != 0 {
                blend_compositor_row(output, scaled, format);
            }
        }
    }
    Some(bounds)
}

#[inline]
fn multiply_255(channel: u8, alpha: u8) -> u16 {
    let product = channel as u16 * alpha as u16 + 128;
    (product + (product >> 8)) >> 8
}

fn unsupported_reason(reason: UnsupportedReason) -> &'static str {
    match reason {
        UnsupportedReason::InvalidFrameGeometry => "invalid-frame-geometry",
        UnsupportedReason::MissingSemanticRecords => "missing-semantic-records",
        UnsupportedReason::InvalidChunkRange => "invalid-chunk-range",
        UnsupportedReason::InvalidRecordRange => "invalid-record-range",
        UnsupportedReason::NonTranslationTransform => "non-translation-transform",
        UnsupportedReason::FractionalPixelPlacement => "fractional-pixel-placement",
        UnsupportedReason::UnsupportedClip => "unsupported-clip",
        UnsupportedReason::OpacityEffect => "opacity-effect",
        UnsupportedReason::BlendEffect => "blend-effect",
        UnsupportedReason::FilterEffect => "filter-effect",
        UnsupportedReason::MaskEffect => "mask-effect",
        UnsupportedReason::ForeignLayer => "foreign-layer",
        UnsupportedReason::TileBudgetExceeded => "tile-budget-exceeded",
    }
}

#[cfg(all(test, target_arch = "aarch64"))]
mod effect_row_test {
    use super::*;

    #[test]
    fn effect_simd_preserves_opacity_then_mask_rounding() {
        let input: Vec<u32> = (0u32..257)
            .map(|v| {
                if v == 256 {
                    0
                } else {
                    v | ((255 - v) << 8) | (((v + 127) & 255) << 16) | (((v + 63) & 255) << 24)
                }
            })
            .collect();
        let mut output = vec![0; input.len()];
        for opacity in [
            0.0f32,
            f32::MIN_POSITIVE,
            0.001,
            0.1,
            0.25,
            0.5,
            0.73,
            0.9,
            f32::from_bits(1.0f32.to_bits() - 1),
            1.0,
        ] {
            let alpha = (opacity != 1.0)
                .then(|| std::array::from_fn(|v| (v as f32 * opacity).round_ties_even() as u8));
            for coverage in 0..=256u16 {
                let mask: Vec<u8> = (0..input.len())
                    .map(|i| (coverage as usize + i * 17) as u8)
                    .collect();
                let mask = (coverage != 256).then_some(mask.as_slice());
                let any = scale_effect_row(&mut output, &input, opacity, alpha.as_ref(), mask);
                let mut reference_any = 0u32;
                for (index, (&src, &actual)) in input.iter().zip(&output).enumerate() {
                    let mut reference = 0u32;
                    for shift in [0, 8, 16, 24] {
                        let value =
                            (((src >> shift) & 255) as f32 * opacity).round_ties_even() as u8;
                        let value = multiply_255(value, mask.map_or(255, |row| row[index]));
                        reference |= u32::from(value) << shift;
                    }
                    assert_eq!(
                        actual, reference,
                        "opacity={opacity} coverage={coverage} pixel={index}"
                    );
                    reference_any |= reference;
                }
                assert_eq!(any != 0, reference_any != 0);
            }
        }
    }
}

#[cfg(test)]
mod direct_mask_quad_test {
    use super::*;

    #[test]
    fn fractional_direct_mask_matches_isolated_source_with_holes_neighbors_and_clip() {
        let (width, height) = (112u32, 14u32);
        let composition = LayerComposition {
            translation: (3.625, 2.6875),
            clip: Some(crate::layer_replay::DeviceRect {
                x: 5,
                y: 3,
                width: 85,
                height: 8,
            }),
            content_bounds: None,
            opaque_bounds: None,
            white_backing: false,
            grid_min: None,
        };
        let mut tiles = Vec::new();
        for index in 0..4usize {
            let (column, row) = (index % 2, index / 2);
            let rect = PaintRect {
                x: (column * 48) as f64,
                y: (row * 4) as f64,
                width: 48.0,
                height: 4.0,
            };
            let placement = TilePlacement {
                tile_id: TileId(index as u64 + 1),
                generation: 1,
                tile_index: (column as i32, row as i32),
                tiling_rect: rect,
                tile_rect: rect,
                raster_scale: 1.0,
                pixel_size: (48, 4),
            };
            let rgba: Vec<u8> = (0..4usize)
                .flat_map(|y| {
                    (0..48usize).flat_map(move |x| {
                        let (gx, gy) = (column * 48 + x, row * 4 + y);
                        // Long exact-zero holes exercise support skips; fractional AA
                        // alpha and true neighbor edges remain present around them.
                        let a = if (6..26).contains(&(gx % 48)) || gy == 5 {
                            0
                        } else {
                            ((gx * 37 + gy * 53 + 91) % 256) as u8
                        };
                        [a / 3, a / 2, a, a]
                    })
                })
                .collect();
            let cached = CachedTile {
                raster_frame: 0,
                generation: 1,
                pixel_size: (48, 4),
                raster_scale: 1.0,
                white_backing: false,
                row_support: tile_row_support(&rgba, 48, false),
                rgba,
                bgra: Vec::new(),
            };
            tiles.push((placement, cached));
        }
        let adjacent = |index: usize| {
            [
                (index % 2 == 0).then(|| &tiles[index + 1].1),
                (index < 2).then(|| &tiles[index + 2].1),
                (index == 0).then(|| &tiles[3].1),
            ]
        };
        let source_view = TargetView {
            origin: (0, 0),
            stride: width as usize,
        };
        let mut source = vec![0; width as usize * height as usize];
        let mut source_touched = None;
        let mut rectangles = Vec::new();
        for (index, (tile, pixels)) in tiles.iter().enumerate() {
            union_bounds(
                &mut source_touched,
                compose_tile(
                    &mut source,
                    width,
                    height,
                    source_view,
                    skia::PixelFormat::Rgba8888,
                    tile,
                    composition,
                    pixels,
                    adjacent(index),
                    None,
                )
                .unwrap(),
            );
            rectangles.push(
                tile_composition_geometry(tile, composition, width, height)
                    .unwrap()
                    .unwrap()
                    .0,
            );
        }
        rectangles.sort_unstable_by_key(|b| (b.top, b.left, b.bottom, b.right));
        // Content ROI and stride differ from the source surface, with padding
        // left observable so accidental whole-row clearing is caught too.
        let view = TargetView {
            origin: (1, 1),
            stride: 109,
        };
        let touched = Some(SurfaceBounds {
            left: 1,
            top: 1,
            right: 108,
            bottom: 13,
        });
        let original: Vec<u32> = (0..109 * 12)
            .map(|i| {
                let a = ((i * 19 + 67) % 256) as u32;
                (a / 4) | ((a / 2) << 8) | ((a / 3) << 16) | (a << 24)
            })
            .collect();
        let mut expected = original.clone();
        dst_in_effect_surface(
            &mut expected,
            &source,
            touched,
            source_touched,
            view,
            source_view,
            1.0,
        );
        let mut actual = original;
        let retained = clear_mask_exterior(&mut actual, view, touched, &rectangles);
        for (index, (tile, pixels)) in tiles.iter().enumerate() {
            dst_in_tile(
                &mut actual,
                view,
                retained,
                width,
                height,
                tile,
                composition,
                pixels,
                adjacent(index),
            )
            .unwrap();
        }
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod opaque_quad_test {
    use super::*;

    #[test]
    fn bounded_opaque_quad_matches_src_over_with_transparent_padding_and_fractional_sampling() {
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 20.0,
        };
        let proof = PaintRect {
            x: 3.0,
            y: 2.0,
            width: 26.0,
            height: 16.0,
        };
        let tile = TilePlacement {
            tile_id: TileId(1),
            generation: 1,
            tile_index: (0, 0),
            tiling_rect: rect,
            tile_rect: rect,
            raster_scale: 1.0,
            pixel_size: (32, 20),
        };
        let rgba: Vec<u8> = (0..20usize)
            .flat_map(|y| {
                (0..32usize).flat_map(move |x| {
                    if (3..29).contains(&x) && (2..18).contains(&y) {
                        [(x * 7) as u8, (y * 11) as u8, 91, 255]
                    } else {
                        [0; 4]
                    }
                })
            })
            .collect();
        let pixels = CachedTile {
            raster_frame: 0,
            generation: 1,
            pixel_size: (32, 20),
            raster_scale: 1.0,
            white_backing: false,
            row_support: tile_row_support(&rgba, 32, false),
            rgba,
        };
        assert!(
            !pixels.row_support.opaque,
            "allocation padding is transparent"
        );
        let (width, height) = (48, 32);
        let view = TargetView {
            origin: (0, 0),
            stride: width as usize,
        };
        for translation in [(4.0, 3.0), (4.375, 3.625), (-1.625, -0.375)] {
            for format in [
                skia::PixelFormat::Rgba8888,
                skia::PixelFormat::Bgra8888,
                skia::PixelFormat::Bgrx8888,
            ] {
                let opaque =
                    crate::layer_replay::sampled_opaque_device_rect(proof, 1.0, translation)
                        .unwrap()
                        .unwrap();
                let safe = SurfaceBounds {
                    left: opaque.x.max(0) as usize,
                    top: opaque.y.max(0) as usize,
                    right: (i64::from(opaque.x) + i64::from(opaque.width)) as usize,
                    bottom: (i64::from(opaque.y) + i64::from(opaque.height)) as usize,
                };
                let base = LayerComposition {
                    translation,
                    clip: None,
                    content_bounds: None,
                    opaque_bounds: None,
                    white_backing: false,
                    grid_min: None,
                };
                let optimized = LayerComposition {
                    opaque_bounds: Some(opaque),
                    ..base
                };
                assert!(opaque_covers(optimized, safe));
                assert!(!opaque_covers(
                    optimized,
                    SurfaceBounds {
                        right: safe.right + 1,
                        ..safe
                    }
                ));
                for visible in [
                    Some(safe),
                    Some(SurfaceBounds {
                        right: safe.right + 1,
                        ..safe
                    }),
                    None,
                ] {
                    let background = if format == skia::PixelFormat::Bgrx8888 {
                        0x005f3921
                    } else {
                        0x9f5f3921
                    };
                    let mut expected = vec![background; (width * height) as usize];
                    let mut actual = expected.clone();
                    compose_tile(
                        &mut expected,
                        width,
                        height,
                        view,
                        format,
                        &tile,
                        base,
                        &pixels,
                        [None; 3],
                        visible,
                    )
                    .unwrap();
                    compose_tile(
                        &mut actual,
                        width,
                        height,
                        view,
                        format,
                        &tile,
                        optimized,
                        &pixels,
                        [None; 3],
                        visible,
                    )
                    .unwrap();
                    assert_eq!(
                        actual, expected,
                        "translation={translation:?} format={format:?}"
                    );
                }
            }
        }
    }
}
