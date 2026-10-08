use crate::{
    layer_replay::{LayerReplay, RasterLayerState},
    solid::{self, RasterDrawMode},
    solid_analysis,
    tile_worker::{PendingTiles, RasterJob, RasterProduct, RasterWorkerPool},
};
use layer_tile::{
    FramePlan, LayerTreeId, RasterBatch, RasterCompletion, RasterTask, TileId, TilePlacement,
    TileResourceRelease,
};
use skia::src::core::SkCanvas::RasterImageCache;
use std::{collections::HashMap, io};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RasterStats {
    pub layers: usize,
    pub tiles: usize,
    pub raster_tasks: usize,
    pub reused_tiles: usize,
    pub resident_bytes: usize,
    pub unsupported_reason: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct RasterUpdate {
    pub mode: &'static str,
    pub reason: &'static str,
    pub damage_pixels: usize,
    pub completions: Vec<RasterCompletion>,
}

#[derive(Clone, Copy)]
#[doc(hidden)]
pub struct RowSupport {
    pub extent: (usize, usize),
    pub opaque: bool,
    pub runs: (usize, usize),
}

pub struct RasterRowSupport {
    #[doc(hidden)]
    pub rows: Vec<RowSupport>,
    #[doc(hidden)]
    pub runs: Vec<(usize, usize)>,
    #[doc(hidden)]
    pub opaque: bool,
    #[doc(hidden)]
    pub draw_mode: RasterDrawMode,
}

impl RasterRowSupport {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_opaque(&self) -> bool {
        self.opaque
    }

    pub fn draw_mode(&self) -> RasterDrawMode {
        self.draw_mode
    }

    pub fn row_is_opaque(&self, y: usize) -> bool {
        self.rows[y].opaque
    }

    pub fn spans(&self, y: usize) -> &[(usize, usize)] {
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

/// Immutable, generation-scoped tile resource. Its storage is private to the
/// raster service; consumers obtain only a shared read lease.
pub struct RasterResource {
    #[doc(hidden)]
    pub raster_frame: u64,
    #[doc(hidden)]
    pub generation: u64,
    #[doc(hidden)]
    pub pixel_size: (u32, u32),
    #[doc(hidden)]
    pub raster_scale: f64,
    #[doc(hidden)]
    pub white_backing: bool,
    #[doc(hidden)]
    pub rgba: Vec<u8>,
    #[doc(hidden)]
    pub bgra: Vec<u32>,
    #[doc(hidden)]
    pub row_support: RasterRowSupport,
}

impl RasterResource {
    pub fn raster_frame(&self) -> u64 {
        self.raster_frame
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn pixel_size(&self) -> (u32, u32) {
        self.pixel_size
    }
    pub fn raster_scale(&self) -> f64 {
        self.raster_scale
    }
    pub fn white_backing(&self) -> bool {
        self.white_backing
    }
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }
    pub fn bgra(&self) -> &[u32] {
        &self.bgra
    }
    pub fn row_support(&self) -> &RasterRowSupport {
        &self.row_support
    }

    pub fn matches(&self, tile: &TilePlacement, white_backing: bool) -> bool {
        self.generation == tile.generation
            && self.pixel_size == tile.pixel_size
            && self.raster_scale == tile.raster_scale
            && self.white_backing == white_backing
    }

    fn matches_task(&self, task: &RasterTask, state: RasterLayerState) -> bool {
        self.generation == task.generation
            && self.pixel_size == task.pixel_size
            && self.raster_scale == task.raster_scale
            && self.white_backing == state.white_backing
    }

    fn from_product(frame_id: u64, product: RasterProduct) -> Self {
        Self {
            raster_frame: frame_id,
            generation: product.task.generation,
            pixel_size: product.task.pixel_size,
            raster_scale: product.task.raster_scale,
            white_backing: product.composition.white_backing,
            rgba: product.rgba,
            bgra: product.bgra,
            row_support: product.row_support,
        }
    }
}

pub trait RasterResourceProvider {
    fn stats(&self) -> RasterStats;
    fn prepared_frame_id(&self) -> Option<u64>;
    fn prepared_damage_pixels(&self) -> usize;
    fn resource(&self, id: TileId) -> Option<&RasterResource>;
    fn resources(&self) -> &HashMap<TileId, RasterResource>;
    fn resident_bytes(&self) -> usize;
}

struct PendingPrepaint {
    layer_tree_id: LayerTreeId,
    frame_id: u64,
    tasks: Vec<(TileId, u64)>,
    pending: PendingTiles,
}

pub struct RasterEngine {
    layer_tree_id: Option<LayerTreeId>,
    prepared_frame_id: Option<u64>,
    prepared_damage_pixels: usize,
    resources: HashMap<TileId, RasterResource>,
    halo_scratch: Vec<u8>,
    worker_pool: Option<RasterWorkerPool>,
    prepaint_worker_pool: Option<RasterWorkerPool>,
    pending_prepaint: Option<PendingPrepaint>,
    image_cache: RasterImageCache,
    clip_cache: skia::RasterClipProductCache,
    stats: RasterStats,
}

impl Default for RasterEngine {
    fn default() -> Self {
        Self {
            layer_tree_id: None,
            prepared_frame_id: None,
            prepared_damage_pixels: 0,
            resources: HashMap::new(),
            halo_scratch: Vec::new(),
            worker_pool: None,
            prepaint_worker_pool: None,
            pending_prepaint: None,
            image_cache: RasterImageCache::default(),
            clip_cache: skia::RasterClipProductCache::default(),
            stats: RasterStats::default(),
        }
    }
}

impl RasterResourceProvider for RasterEngine {
    fn stats(&self) -> RasterStats {
        self.stats
    }
    fn prepared_frame_id(&self) -> Option<u64> {
        self.prepared_frame_id
    }
    fn prepared_damage_pixels(&self) -> usize {
        self.prepared_damage_pixels
    }
    fn resource(&self, id: TileId) -> Option<&RasterResource> {
        self.resources.get(&id)
    }
    fn resources(&self) -> &HashMap<TileId, RasterResource> {
        &self.resources
    }
    fn resident_bytes(&self) -> usize {
        self.stats.resident_bytes
    }
}

impl RasterEngine {
    pub fn stats(&self) -> RasterStats {
        self.stats
    }

    pub fn image_cache_stats(&self) -> skia::src::core::SkCanvas::RasterImageCacheStats {
        self.image_cache.stats()
    }

    pub fn image_cache_resident_pixel_bytes(&self) -> usize {
        self.image_cache.resident_pixel_bytes()
    }

    pub fn clip_product_cache_stats(&self) -> skia::RasterClipProductCacheStats {
        self.clip_cache.stats()
    }

    pub fn clip_product_cache_accounting(&self) -> skia::RasterClipProductCacheAccounting {
        self.clip_cache.accounting()
    }

    pub fn with_clip_product_cache_byte_limit(limit: usize) -> Self {
        let mut engine = Self::default();
        engine.clip_cache = skia::RasterClipProductCache::with_byte_limit(limit);
        engine
    }

    pub fn invalidate(&mut self) {
        self.layer_tree_id = None;
        self.pending_prepaint = None;
        self.prepared_frame_id = None;
        self.prepared_damage_pixels = 0;
        self.resources.clear();
        self.stats = RasterStats::default();
    }

    fn fail(&mut self, reason: &'static str) -> io::Error {
        self.invalidate();
        self.stats.unsupported_reason = Some(reason);
        io::Error::new(io::ErrorKind::Unsupported, reason)
    }

    fn publish_prepaint(
        &mut self,
        layer_tree_id: LayerTreeId,
        frame_id: u64,
        products: Vec<RasterProduct>,
    ) -> io::Result<Vec<RasterCompletion>> {
        let mut completions = Vec::with_capacity(products.len());
        for product in products {
            if !valid_product_storage(&product)?
                || product.row_support.len() != product.task.pixel_size.1 as usize
            {
                return Err(io::Error::other(
                    "background raster returned invalid tile storage",
                ));
            }
            completions.push(RasterCompletion {
                layer_tree_id,
                frame_id,
                tile_id: product.task.tile_id,
                generation: product.task.generation,
            });
            self.resources.insert(
                product.task.tile_id,
                RasterResource::from_product(frame_id, product),
            );
        }
        Ok(completions)
    }

    fn collect_prepaint(&mut self, required: &[RasterTask]) -> io::Result<Vec<RasterCompletion>> {
        let Some(mut batch) = self.pending_prepaint.take() else {
            return Ok(Vec::new());
        };
        let needed_now = required.iter().any(|task| {
            batch
                .tasks
                .iter()
                .any(|&(id, generation)| id == task.tile_id && generation == task.generation)
        });
        if needed_now {
            let products = batch.pending.finish()?;
            return self.publish_prepaint(batch.layer_tree_id, batch.frame_id, products);
        }
        match batch.pending.try_finish() {
            None => {
                self.pending_prepaint = Some(batch);
                Ok(Vec::new())
            }
            Some(Ok(products)) => {
                self.publish_prepaint(batch.layer_tree_id, batch.frame_id, products)
            }
            Some(Err(_)) => {
                self.prepaint_worker_pool = None;
                Ok(Vec::new())
            }
        }
    }

    fn schedule_prepaint(
        &mut self,
        plan: &FramePlan,
        batch: &RasterBatch,
        replay: &mut LayerReplay<'_>,
        draw_modes: &[RasterDrawMode],
        first: usize,
    ) -> io::Result<usize> {
        if first >= batch.tasks.len() || self.pending_prepaint.is_some() {
            return Ok(0);
        }
        let mut jobs = Vec::new();
        let mut keys = Vec::new();
        for (index, (task, mode)) in batch.tasks.iter().zip(draw_modes).enumerate().skip(first) {
            let composition = replay
                .composition(task.layer_id)
                .ok_or_else(|| io::Error::other("prepaint task has no layer composition"))?;
            if self
                .resources
                .get(&task.tile_id)
                .is_some_and(|resource| resource.matches_task(task, composition))
            {
                continue;
            }
            let layer = replay
                .prepaint_layer(task)
                .ok_or_else(|| io::Error::other("prepaint task has incomplete prepared records"))?;
            let reuse = task
                .previous_tile_id
                .and_then(|id| self.resources.remove(&id))
                .map(|resource| resource.rgba)
                .unwrap_or_default();
            jobs.push(RasterJob {
                plan_index: index,
                task: task.clone(),
                layer,
                reuse,
                solid_color: match mode {
                    RasterDrawMode::SolidColor { premul_rgba } => Some(*premul_rgba),
                    RasterDrawMode::Resource => None,
                },
            });
            keys.push((task.tile_id, task.generation));
        }
        if jobs.is_empty() {
            return Ok(0);
        }
        if self.prepaint_worker_pool.is_none() {
            let limit = (self.clip_cache.accounting().configured_byte_limit / 2).min(1024 * 1024);
            self.prepaint_worker_pool = Some(RasterWorkerPool::new_prepaint(limit)?);
        }
        let count = jobs.len();
        let pending = self
            .prepaint_worker_pool
            .as_mut()
            .unwrap()
            .submit(plan.GetPaintArtifact().resources.clone(), jobs);
        self.pending_prepaint = Some(PendingPrepaint {
            layer_tree_id: batch.layer_tree_id,
            frame_id: batch.frame_id,
            tasks: keys,
            pending,
        });
        Ok(count)
    }

    pub fn prepare(
        &mut self,
        plan: &FramePlan,
        batch: &RasterBatch,
        width: u32,
        height: u32,
    ) -> io::Result<RasterUpdate> {
        let mut trace = browser_tracing::span("raster", "PrepareTiles");
        trace.set("frame_id", plan.frame_id as f64);
        if batch.layer_tree_id != plan.layer_tree_id || batch.frame_id != plan.frame_id {
            return Err(io::Error::other(
                "raster protocols refer to different layer trees",
            ));
        }
        if self.layer_tree_id != Some(plan.layer_tree_id) {
            self.invalidate();
            self.layer_tree_id = Some(plan.layer_tree_id);
        }
        let required = batch.required_task_count.min(batch.tasks.len());
        let mut completions = self.collect_prepaint(&batch.tasks[..required])?;
        if self.prepared_frame_id == Some(plan.frame_id) {
            return Ok(RasterUpdate {
                mode: "tile-raster",
                reason: "already-prepared",
                damage_pixels: 0,
                completions,
            });
        }
        let scale = plan.config.raster_scale;
        if width == 0 || !scale.is_finite() || scale <= 0.0 || !cfg!(target_endian = "little") {
            return Err(self.fail("unsupported-raster-target"));
        }
        let expected = paint::PaintRect {
            x: 0.0,
            y: 0.0,
            width: width as f64 / scale,
            height: height as f64 / scale,
        };
        if plan.config.viewport != expected {
            return Err(self.fail("raster-viewport-mismatch"));
        }
        if let Some(reason) = plan.unsupported {
            return Err(self.fail(unsupported_reason(reason)));
        }
        self.stats = RasterStats {
            layers: plan.layers.len(),
            tiles: plan.layers.iter().map(|layer| layer.tiles.len()).sum(),
            ..Default::default()
        };
        let mut replay =
            LayerReplay::new(plan, &batch.tasks).map_err(|error| self.fail(error.reason()))?;
        let draw_modes: Vec<_> = batch
            .tasks
            .iter()
            .map(|task| {
                replay
                    .solid_analysis_commands(task)
                    .map_or(RasterDrawMode::Resource, |records| {
                        solid_analysis::analyze(task, records)
                    })
            })
            .collect();
        let tasks = &batch.tasks[..required];
        let layers: Vec<_> = tasks.iter().map(|task| replay.worker_layer(task)).collect();
        let eligible = layers.iter().filter(|layer| layer.is_some()).count();
        if self.worker_pool.is_none() && eligible >= 4 {
            let limit = (self.clip_cache.accounting().configured_byte_limit / 2).min(1024 * 1024);
            self.worker_pool = Some(RasterWorkerPool::new(limit)?);
        }
        let parallel = eligible >= 2
            && self
                .worker_pool
                .as_ref()
                .is_some_and(RasterWorkerPool::available);
        let mut worker_jobs = Vec::new();
        let mut local_jobs = Vec::new();
        let mut products = Vec::with_capacity(tasks.len());
        for (index, (task, layer)) in tasks.iter().zip(layers).enumerate() {
            let reuse = task
                .previous_tile_id
                .and_then(|id| self.resources.remove(&id))
                .map(|resource| resource.rgba)
                .unwrap_or_default();
            if let Some(layer) = layer.filter(|_| parallel) {
                worker_jobs.push(RasterJob {
                    plan_index: index,
                    task: task.clone(),
                    layer,
                    reuse,
                    solid_color: match draw_modes[index] {
                        RasterDrawMode::SolidColor { premul_rgba } => Some(premul_rgba),
                        RasterDrawMode::Resource => None,
                    },
                });
            } else if let RasterDrawMode::SolidColor { premul_rgba } = draw_modes[index] {
                let composition = replay
                    .composition(task.layer_id)
                    .ok_or_else(|| io::Error::other("solid task has no layer composition"))?;
                products.push(solid::raster_product(
                    index,
                    task,
                    composition,
                    premul_rgba,
                    reuse,
                )?);
            } else {
                local_jobs.push((index, task, reuse));
            }
        }
        let pending = (!worker_jobs.is_empty()).then(|| {
            self.worker_pool
                .as_mut()
                .unwrap()
                .submit(plan.GetPaintArtifact().resources.clone(), worker_jobs)
        });
        let used_local_replay = !local_jobs.is_empty();
        if used_local_replay {
            replay.set_scratch(std::mem::take(&mut self.halo_scratch));
        }
        for (index, task, reuse) in local_jobs {
            let composition = replay
                .composition(task.layer_id)
                .ok_or_else(|| io::Error::other("raster task has no layer composition"))?;
            let rgba = replay.raster_tile_with_pixels(
                task,
                &mut self.image_cache,
                &mut self.clip_cache,
                reuse,
            )?;
            let row_support =
                tile_row_support(&rgba, task.pixel_size.0 as usize, composition.white_backing);
            products.push(RasterProduct {
                plan_index: index,
                task: task.clone(),
                composition,
                bgra: cached_bgra(&rgba),
                rgba,
                row_support,
            });
        }
        if used_local_replay {
            self.halo_scratch = replay.take_scratch();
        }
        if let Some(pending) = pending {
            products.extend(pending.finish()?);
        }
        products.sort_unstable_by_key(|product| product.plan_index);
        if products.len() != tasks.len() {
            return Err(io::Error::other(
                "raster lanes returned an incomplete task plan",
            ));
        }
        let mut damage_pixels = 0usize;
        // A generation becomes visible as one transaction.  Do not publish a
        // prefix of the batch if validation or the LayerTile completion ack
        // rejects a later result.
        let mut staged = HashMap::with_capacity(products.len());
        for (index, (task, mut product)) in tasks.iter().zip(products).enumerate() {
            if product.plan_index != index
                || product.task.tile_id != task.tile_id
                || product.task.generation != task.generation
                || product.task.layer_id != task.layer_id
            {
                return Err(io::Error::other(
                    "raster product differs from submitted tile generation",
                ));
            }
            product.row_support.draw_mode = draw_modes[index];
            damage_pixels = damage_pixels.saturating_add(tile_bytes(task.pixel_size)? / 4);
            staged.insert(
                task.tile_id,
                RasterResource::from_product(plan.frame_id, product),
            );
        }
        self.resources.extend(staged);
        completions.extend(tasks.iter().map(|task| RasterCompletion {
            layer_tree_id: batch.layer_tree_id,
            frame_id: batch.frame_id,
            tile_id: task.tile_id,
            generation: task.generation,
        }));
        let _ = self.schedule_prepaint(plan, batch, &mut replay, &draw_modes, required)?;
        self.prepared_frame_id = Some(plan.frame_id);
        self.prepared_damage_pixels = damage_pixels;
        self.stats.raster_tasks = tasks.len();
        self.stats.reused_tiles = self.stats.tiles.saturating_sub(tasks.len());
        self.stats.resident_bytes = self.resources.values().map(resource_bytes).sum();
        Ok(RasterUpdate {
            mode: "tile-raster",
            reason: if tasks.is_empty() {
                "reused-visible-tiles"
            } else {
                "rasterized-dirty-tiles"
            },
            damage_pixels,
            completions,
        })
    }

    /// Retire old pixels only after the compositor has activated the pending
    /// tree. Until this call the preceding active frame remains drawable.
    pub fn release_resources(
        &mut self,
        release: &TileResourceRelease,
    ) -> io::Result<TileResourceRelease> {
        if self.layer_tree_id != Some(release.layer_tree_id) {
            return Err(io::Error::other(
                "resource release belongs to another layer tree",
            ));
        }
        for tile in release.tile_ids.iter() {
            self.resources.remove(tile);
        }
        self.stats.resident_bytes = self.resources.values().map(resource_bytes).sum();
        Ok(release.clone())
    }
}

fn resource_bytes(resource: &RasterResource) -> usize {
    resource.rgba.len() + resource.bgra.len() * std::mem::size_of::<u32>()
}

pub(crate) fn tile_bytes(size: (u32, u32)) -> io::Result<usize> {
    (size.0 as usize)
        .checked_mul(size.1 as usize)
        .and_then(|n| n.checked_mul(4))
        .filter(|&n| size.0 > 0 && size.1 > 0 && n <= isize::MAX as usize)
        .ok_or_else(|| io::Error::other("invalid tile pixel dimensions"))
}

fn valid_product_storage(product: &RasterProduct) -> io::Result<bool> {
    let bytes = tile_bytes(product.task.pixel_size)?;
    let solid = matches!(
        product.row_support.draw_mode,
        RasterDrawMode::SolidColor { .. }
    ) && product.rgba.is_empty()
        && product.bgra.is_empty();
    Ok((product.rgba.len() == bytes && product.bgra.len() == bytes / 4) || solid)
}

pub(crate) fn tile_row_support(rgba: &[u8], width: usize, opaque: bool) -> RasterRowSupport {
    let mut support = RasterRowSupport {
        rows: Vec::with_capacity(rgba.len() / (width * 4)),
        runs: Vec::new(),
        opaque: true,
        draw_mode: RasterDrawMode::Resource,
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
        if right - left > 17 {
            for (x, pixel) in row.chunks_exact(4).enumerate().take(right).skip(left + 1) {
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

pub(crate) fn cached_bgra(rgba: &[u8]) -> Vec<u32> {
    let mut bgra = vec![0; rgba.len() / 4];
    #[allow(unused_mut)]
    let mut at = 0;
    #[cfg(target_arch = "aarch64")]
    unsafe {
        use core::arch::aarch64::*;
        let swap = vld1q_u8([2u8, 1, 0, 3, 6, 5, 4, 7, 10, 9, 8, 11, 14, 13, 12, 15].as_ptr());
        while at + 16 <= bgra.len() {
            for block in 0..4usize {
                let offset = (at + block * 4) * 4;
                let pixels = vqtbl1q_u8(vld1q_u8(rgba.as_ptr().add(offset)), swap);
                vst1q_u8(bgra.as_mut_ptr().add(at + block * 4).cast::<u8>(), pixels);
            }
            at += 16;
        }
        if at + 8 <= bgra.len() {
            for block in 0..2usize {
                let offset = (at + block * 4) * 4;
                let pixels = vqtbl1q_u8(vld1q_u8(rgba.as_ptr().add(offset)), swap);
                vst1q_u8(bgra.as_mut_ptr().add(at + block * 4).cast::<u8>(), pixels);
            }
            at += 8;
        }
    }
    for (pixel, bytes) in bgra[at..].iter_mut().zip(rgba[at * 4..].chunks_exact(4)) {
        let word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        *pixel = (word & 0xff00_ff00) | ((word & 0xff) << 16) | ((word >> 16) & 0xff);
    }
    bgra
}

fn unsupported_reason(reason: layer_tile::UnsupportedReason) -> &'static str {
    match reason {
        layer_tile::UnsupportedReason::InvalidFrameGeometry => "invalid-frame-geometry",
        layer_tile::UnsupportedReason::MissingSemanticRecords => "missing-semantic-records",
        layer_tile::UnsupportedReason::InvalidChunkRange => "invalid-chunk-range",
        layer_tile::UnsupportedReason::InvalidRecordRange => "invalid-record-range",
        layer_tile::UnsupportedReason::NonTranslationTransform => "non-translation-transform",
        layer_tile::UnsupportedReason::FractionalPixelPlacement => "fractional-pixel-placement",
        layer_tile::UnsupportedReason::UnsupportedClip => "unsupported-clip",
        layer_tile::UnsupportedReason::OpacityEffect => "opacity-effect",
        layer_tile::UnsupportedReason::BlendEffect => "blend-effect",
        layer_tile::UnsupportedReason::FilterEffect => "filter-effect",
        layer_tile::UnsupportedReason::MaskEffect => "mask-effect",
        layer_tile::UnsupportedReason::ForeignLayer => "foreign-layer",
        layer_tile::UnsupportedReason::TileBudgetExceeded => "tile-budget-exceeded",
    }
}
