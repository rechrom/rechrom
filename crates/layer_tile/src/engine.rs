//! Explicit active/pending layer-tree lifecycle.
//!
//! Raster work never receives a callback into this engine. The compositor
//! publishes a [`PendingTreeUpdate`], routes its [`RasterBatch`] to a raster
//! executor, then feeds immutable [`RasterCompletion`] values back here before
//! activation.

use std::collections::{BTreeSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use paint::paint_engine::PaintArtifact;

use crate::manager::LayerTileManager;
use crate::recording::{CanonicalRasterContentCache, ReplayUnsupported};
use crate::{
    CompositorScrollOffset, FrameConfig, FramePlan, LayerTileStats, LayerTreeId, PendingTreeUpdate,
    RasterBatch, RasterCompletion, TileId, TileResourceRelease,
};

static NEXT_LAYER_TREE_ID: AtomicU64 = AtomicU64::new(1);

/// Owns persistent layer/tiling/tile metadata for one compositor client.
/// Pixels remain in `raster`; scheduling remains in `compositor`.
pub struct LayerTileEngine {
    id: LayerTreeId,
    manager: LayerTileManager,
    retired_tiles: BTreeSet<TileId>,
    recording: CanonicalRasterContentCache,
    active_plan: Option<FramePlan>,
    pending_plan: Option<FramePlan>,
    pending_batch: Option<RasterBatch>,
    viewport_history: VecDeque<(std::time::Instant, CompositorScrollOffset)>,
}

impl Default for LayerTileEngine {
    fn default() -> Self {
        let mut engine = Self::new(256);
        const DEFAULT_TILE_MEMORY_BUDGET_BYTES: usize = 256 * 1024 * 1024;
        const TILE_BYTES: usize = 256 * 256 * 4;
        engine.SetTileBudget(DEFAULT_TILE_MEMORY_BUDGET_BYTES / TILE_BYTES);
        engine
    }
}

#[allow(non_snake_case)]
impl LayerTileEngine {
    pub fn new(tile_size: u32) -> Self {
        Self {
            id: LayerTreeId(NEXT_LAYER_TREE_ID.fetch_add(1, Ordering::Relaxed)),
            manager: LayerTileManager::new(tile_size),
            retired_tiles: BTreeSet::new(),
            recording: CanonicalRasterContentCache::default(),
            active_plan: None,
            pending_plan: None,
            pending_batch: None,
            viewport_history: VecDeque::with_capacity(2),
        }
    }

    pub fn SetTileBudget(&mut self, tiles: usize) {
        self.manager.set_tile_budget(tiles);
    }

    /// Atomically records a new pending tree with the viewport state that
    /// selected its tiles. This replaces the old SetFrameConfig/Update pair.
    pub fn UpdatePending(
        &mut self,
        artifact: &Arc<PaintArtifact>,
        config: FrameConfig,
    ) -> Result<PendingTreeUpdate, ReplayUnsupported> {
        let mut update_trace = browser_tracing::span("layer_tile", "LayerTileUpdatePending");
        let timing = (std::env::var_os("LAYOUTNG_LAYER_PROFILE").is_some()
            || std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some())
        .then(std::time::Instant::now);

        self.pending_plan = None;
        self.pending_batch = None;
        let mut recording_trace = browser_tracing::span("layer_tile", "CanonicalRasterContent");
        recording_trace.set("recording_revision", artifact.recording_revision as f64);
        recording_trace.set(
            "cached_revision",
            self.recording.published_revision() as f64,
        );
        recording_trace.set(
            "same_items",
            self.recording.published_shares_items(artifact) as u8 as f64,
        );
        let content = match self.recording.prepare(artifact, config.raster_scale) {
            Ok(content) => content,
            Err(error) => {
                self.Invalidate();
                return Err(error);
            }
        };
        recording_trace.set("records", content.len() as f64);
        recording_trace.set(
            "recording_fast_hit",
            self.recording.last_fast_hit() as u8 as f64,
        );
        let recording_done = timing.map(|start| start.elapsed());
        drop(recording_trace);

        let mut plan_trace = browser_tracing::span("layer_tile", "PlanTiles");
        let prepaint_scroll = self.predict_prepaint_scroll(config);
        let mut plan =
            self.manager
                .update_with_raster_content(artifact, config, prepaint_scroll, &content);
        plan.layer_tree_id = self.id;
        plan.raster_records = content;
        let tasks: Arc<[crate::RasterTask]> = Arc::from(std::mem::take(&mut plan.tasks));
        plan.raster_task_count = tasks.len();
        let required_task_count = required_raster_task_count(&plan, &tasks);

        self.retired_tiles
            .extend(std::mem::take(&mut plan.retired_tiles));
        let batch = RasterBatch {
            layer_tree_id: self.id,
            frame_id: plan.frame_id,
            tasks,
            required_task_count,
        };
        self.recording.commit();

        if browser_tracing::enabled() {
            let tiles: usize = plan.layers.iter().map(|layer| layer.tiles.len()).sum();
            for trace in [&mut update_trace, &mut plan_trace] {
                trace.set("frame_id", plan.frame_id as f64);
                trace.set("layers", plan.layers.len() as f64);
                trace.set("tiles", tiles as f64);
                trace.set("raster_tasks", batch.tasks.len() as f64);
                trace.set("visible_raster_tasks", required_task_count as f64);
                trace.set(
                    "prepaint_raster_tasks",
                    batch.tasks.len().saturating_sub(required_task_count) as f64,
                );
                trace.set("retired_tiles", self.retired_tiles.len() as f64);
                trace.set("supported", plan.unsupported.is_none() as u8 as f64);
            }
        }

        self.pending_plan = Some(plan.clone());
        self.pending_batch = Some(batch.clone());
        drop(plan_trace);
        if let Some(start) = timing {
            let recording = recording_done.unwrap();
            eprintln!(
                "layer-tile-plan-stages canonical_ms={:.3} plan_ms={:.3}",
                recording.as_secs_f64() * 1000.0,
                (start.elapsed() - recording).as_secs_f64() * 1000.0
            );
        }
        Ok(PendingTreeUpdate {
            frame_plan: plan,
            raster_batch: batch,
        })
    }

    /// Apply explicit raster completion messages. Stale generations are
    /// ignored by the manager and can never make a replacement tile ready.
    pub fn ApplyRasterResults(&mut self, results: &[RasterCompletion]) -> usize {
        let mut accepted = 0;
        for result in results {
            if result.layer_tree_id == self.id
                && self
                    .manager
                    .mark_rasterized(result.tile_id, result.generation)
            {
                accepted += 1;
            }
        }
        if accepted != 0 {
            refresh_readiness(&self.manager, self.active_plan.as_mut());
            refresh_readiness(&self.manager, self.pending_plan.as_mut());
        }
        accepted
    }

    pub fn AcknowledgeResourceRelease(&mut self, release: &TileResourceRelease) {
        if release.layer_tree_id != self.id {
            return;
        }
        for id in release.tile_ids.iter() {
            self.retired_tiles.remove(id);
        }
    }

    pub fn PendingReadyForActivation(&self) -> bool {
        let (Some(plan), Some(batch)) = (&self.pending_plan, &self.pending_batch) else {
            return false;
        };
        plan.unsupported.is_none()
            && batch.tasks[..batch.required_task_count]
                .iter()
                .all(|task| self.manager.is_rasterized(task.tile_id, task.generation))
            && plan
                .layers
                .iter()
                .flat_map(|layer| layer.tiles.iter())
                .all(|tile| tile.ready)
    }

    /// Activation is compositor policy; LayerTile performs it only after the
    /// caller has observed readiness and explicitly requests the transition.
    /// The returned release is the first point at which resources belonging
    /// only to the superseded active tree may be reclaimed.
    pub fn ActivatePending(&mut self) -> Option<TileResourceRelease> {
        if !self.PendingReadyForActivation() {
            return None;
        }
        self.active_plan = self.pending_plan.take();
        self.pending_batch = None;
        let frame_id = self
            .active_plan
            .as_ref()
            .expect("pending plan checked above")
            .frame_id;
        Some(TileResourceRelease {
            layer_tree_id: self.id,
            frame_id,
            tile_ids: self.retired_tiles.iter().copied().collect(),
        })
    }

    pub fn GetActiveFramePlan(&self) -> Option<&FramePlan> {
        self.active_plan.as_ref()
    }

    pub fn GetPendingFramePlan(&self) -> Option<&FramePlan> {
        self.pending_plan.as_ref()
    }

    /// Release immutable publications before Paint mutates a uniquely-owned
    /// artifact. Resident metadata and backend pixels remain reusable.
    #[doc(hidden)]
    pub fn ReleaseFramePlans(&mut self) {
        self.active_plan = None;
        self.pending_plan = None;
        self.pending_batch = None;
    }

    pub fn GetStats(&self) -> LayerTileStats {
        self.manager.stats()
    }

    pub fn InvalidateResources(&mut self) {
        self.manager.invalidate_resources();
        refresh_readiness(&self.manager, self.active_plan.as_mut());
        refresh_readiness(&self.manager, self.pending_plan.as_mut());
    }

    pub fn Invalidate(&mut self) {
        self.retired_tiles.extend(self.manager.invalidate());
        self.active_plan = None;
        self.pending_plan = None;
        self.pending_batch = None;
        self.recording.clear();
        self.viewport_history.clear();
    }

    /// Matches PictureLayerTilingSet's software-raster skewport policy: use
    /// two retained visible-position samples, extrapolate one second, and cap
    /// the prediction at 2000 device pixels. The compositor supplies time;
    /// LayerTile owns the history and prediction policy.
    fn predict_prepaint_scroll(&mut self, config: FrameConfig) -> Option<CompositorScrollOffset> {
        let current = config.activation_scroll?;
        let Some(frame_time) = config.frame_time else {
            return Some(current);
        };
        if self
            .viewport_history
            .back()
            .is_some_and(|(_, sample)| sample.scroll_node_id != current.scroll_node_id)
        {
            self.viewport_history.clear();
        }
        let predicted = self.viewport_history.front().and_then(|(old_time, old)| {
            let elapsed = frame_time.checked_duration_since(*old_time)?.as_secs_f64();
            (elapsed > 0.0).then(|| {
                let limit = 2000.0 / config.raster_scale.max(f64::MIN_POSITIVE);
                let extrapolated = (current.translation_y - old.translation_y) * (1.0 / elapsed);
                CompositorScrollOffset {
                    scroll_node_id: current.scroll_node_id,
                    translation_y: current.translation_y + extrapolated.clamp(-limit, limit),
                }
            })
        });
        if self.viewport_history.len() == 2 {
            self.viewport_history.pop_front();
        }
        self.viewport_history.push_back((frame_time, current));
        predicted.or(Some(current))
    }
}

fn required_raster_task_count(plan: &FramePlan, tasks: &[crate::RasterTask]) -> usize {
    tasks
        .iter()
        .filter(|task| {
            task.required_for_activation
                || plan
                    .layers
                    .iter()
                    .any(|layer| layer.tiles.iter().any(|tile| tile.tile_id == task.tile_id))
        })
        .count()
}

fn refresh_readiness(manager: &LayerTileManager, plan: Option<&mut FramePlan>) {
    let Some(plan) = plan else { return };
    for tile in plan
        .layers
        .iter_mut()
        .flat_map(|layer| layer.tiles.iter_mut())
    {
        tile.ready = manager.is_rasterized(tile.tile_id, tile.generation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paint::paint_engine::PaintRect;
    use std::time::Duration;

    fn config(frame_time: std::time::Instant, node: u64, translation_y: f64) -> FrameConfig {
        FrameConfig {
            viewport: PaintRect {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
            },
            raster_scale: 2.0,
            activation_scroll: Some(CompositorScrollOffset {
                scroll_node_id: node,
                translation_y,
            }),
            frame_time: Some(frame_time),
        }
    }

    #[test]
    fn skewport_uses_retained_frame_history_and_resets_for_another_scroll_node() {
        let mut engine = LayerTileEngine::default();
        let start = std::time::Instant::now();
        assert_eq!(
            engine.predict_prepaint_scroll(config(start, 7, 0.0)),
            Some(CompositorScrollOffset {
                scroll_node_id: 7,
                translation_y: 0.0,
            })
        );
        let predicted = engine
            .predict_prepaint_scroll(config(start + Duration::from_millis(16), 7, -10.0))
            .unwrap();
        assert!(predicted.translation_y < -10.0);
        // The 2000-device-pixel cap is 1000 CSS pixels at DPR 2.
        assert!(predicted.translation_y >= -1010.0);

        assert_eq!(
            engine.predict_prepaint_scroll(config(start + Duration::from_millis(32), 9, -25.0,)),
            Some(CompositorScrollOffset {
                scroll_node_id: 9,
                translation_y: -25.0,
            })
        );
    }
}
