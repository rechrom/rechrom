//! Resident layer/tile state and the complete input to a raster backend.
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, Weak};

use paint::paint_engine::{PaintArtifact, PaintRect};

use crate::manager::LayerTileManager;
use crate::recording::{CanonicalRasterContentCache, ReplayUnsupported};
use crate::{FrameConfig, FramePlan, LayerTileStats, TileId};

pub(crate) struct EngineState {
    pub manager: LayerTileManager,
    // Keep notifications until the backend consumes them. Several updates can
    // happen before presentation; replacing a plan must not leak old pixels.
    pub retired_tiles: BTreeSet<TileId>,
    pub valid_plan: bool,
}

/// Internal backend connection, like cc raster task completion to TileManager.
/// Weak ownership lets frames/resources outlive their engine without a cycle.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct TileResourceOwner(pub(crate) Weak<Mutex<EngineState>>);

impl TileResourceOwner {
    pub fn same_engine(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }

    /// Accept one independently completed raster task. Tile identity and
    /// generation are the synchronization token, so a result from an older
    /// frame can warm the current tiling only while that exact tile survives.
    pub fn raster_complete(&self, id: TileId, generation: u64) -> bool {
        self.0.upgrade().is_some_and(|state| {
            state
                .lock()
                .expect("layer/tile state poisoned")
                .manager
                .mark_rasterized(id, generation)
        })
    }

    pub fn raster_is_ready(&self, id: TileId, generation: u64) -> bool {
        self.0.upgrade().is_some_and(|state| {
            state
                .lock()
                .expect("layer/tile state poisoned")
                .manager
                .is_rasterized(id, generation)
        })
    }

    /// Losing backend pixels invalidates readiness, retaining layer/tile
    /// identities. Superseded plans cannot acknowledge the replacement work.
    pub fn invalidate(&self) {
        if let Some(state) = self.0.upgrade() {
            let mut state = state.lock().expect("layer/tile state poisoned");
            state.manager.invalidate_resources();
            state.valid_plan = false;
        }
    }
}

/// Owns persistent layer/tiling/tile metadata. The renderer owns raster
/// resources, execution and composition; no pixel storage lives here.
pub struct LayerTileEngine {
    state: Arc<Mutex<EngineState>>,
    config: FrameConfig,
    recording: CanonicalRasterContentCache,
    frame_plan: Option<FramePlan>,
}

impl Default for LayerTileEngine {
    fn default() -> Self {
        let mut engine = Self::new(256);
        // Match the existing retained software path: 512 RGBA tiles = 128 MiB.
        engine.SetTileBudget(512);
        engine
    }
}

#[allow(non_snake_case)]
impl LayerTileEngine {
    pub fn new(tile_size: u32) -> Self {
        Self {
            state: Arc::new(Mutex::new(EngineState {
                manager: LayerTileManager::new(tile_size),
                retired_tiles: BTreeSet::new(),
                valid_plan: false,
            })),
            config: FrameConfig {
                viewport: PaintRect::default(),
                raster_scale: 1.0,
                activation_scroll: None,
                prepaint_scroll: None,
            },
            recording: CanonicalRasterContentCache::default(),
            frame_plan: None,
        }
    }

    pub fn SetFrameConfig(&mut self, config: FrameConfig) {
        self.config = config;
    }

    pub fn SetTileBudget(&mut self, tiles: usize) {
        self.state
            .lock()
            .expect("layer/tile state poisoned")
            .manager
            .set_tile_budget(tiles);
    }

    /// Retain the immutable paint result by sharing its Arc, not copying its
    /// command/glyph/image payloads. Record lowering precedes tile selection.
    pub fn Update(&mut self, artifact: &Arc<PaintArtifact>) -> Result<(), ReplayUnsupported> {
        let mut update_trace = browser_tracing::span("layer_tile", "LayerTileUpdate");
        let timing = (std::env::var_os("LAYOUTNG_LAYER_PROFILE").is_some()
            || std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some())
        .then(std::time::Instant::now);
        self.frame_plan = None;
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
        let content = match self.recording.prepare(artifact, self.config.raster_scale) {
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
        let mut state = self.state.lock().expect("layer/tile state poisoned");
        let mut plan = state
            .manager
            .update_with_raster_content(artifact, self.config, &content);
        state
            .retired_tiles
            .extend(plan.retired_tiles.iter().copied());
        plan.retired_tiles = state.retired_tiles.iter().copied().collect();
        state.valid_plan = plan.unsupported.is_none();
        plan.raster_records = content;
        plan.resource_owner = Some(TileResourceOwner(Arc::downgrade(&self.state)));
        // This cache proves only input/geometry equivalence. Its validity does
        // not depend on whether a backend later rasterizes/composes successfully.
        self.recording.commit();
        if browser_tracing::enabled() {
            let tiles: usize = plan.layers.iter().map(|layer| layer.tiles.len()).sum();
            let required_tasks = plan.RequiredRasterTaskCount();
            for trace in [&mut update_trace, &mut plan_trace] {
                trace.set("frame_id", plan.frame_id as f64);
                trace.set("layers", plan.layers.len() as f64);
                trace.set("tiles", tiles as f64);
                trace.set("raster_tasks", plan.tasks.len() as f64);
                trace.set("visible_raster_tasks", required_tasks as f64);
                trace.set(
                    "prepaint_raster_tasks",
                    plan.tasks.len().saturating_sub(required_tasks) as f64,
                );
                trace.set("retired_tiles", plan.retired_tiles.len() as f64);
                trace.set(
                    "supported",
                    if plan.unsupported.is_none() { 1.0 } else { 0.0 },
                );
            }
        }
        self.frame_plan = Some(plan);
        drop(plan_trace);
        if let Some(start) = timing {
            let recording = recording_done.unwrap();
            eprintln!(
                "layer-tile-plan-stages canonical_ms={:.3} plan_ms={:.3}",
                recording.as_secs_f64() * 1000.0,
                (start.elapsed() - recording).as_secs_f64() * 1000.0
            );
        }
        Ok(())
    }

    pub fn GetFramePlan(&self) -> Option<&FramePlan> {
        self.frame_plan.as_ref()
    }

    /// Drop the immutable publication for the preceding frame before Page
    /// mutates its uniquely-owned PaintArtifact. Resident layer/tile metadata,
    /// canonical-record cache and backend pixels remain reusable; Update will
    /// publish the replacement plan before the next composition.
    #[doc(hidden)]
    pub fn ReleaseFramePlan(&mut self) {
        self.frame_plan = None;
        self.state
            .lock()
            .expect("layer/tile state poisoned")
            .valid_plan = false;
    }

    pub fn GetStats(&self) -> LayerTileStats {
        self.state
            .lock()
            .expect("layer/tile state poisoned")
            .manager
            .stats()
    }

    pub fn Invalidate(&mut self) {
        let mut state = self.state.lock().expect("layer/tile state poisoned");
        let retired = state.manager.invalidate();
        state.retired_tiles.extend(retired);
        state.valid_plan = false;
        self.frame_plan = None;
        self.recording.clear();
    }
}
