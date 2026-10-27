//! Priority-ordered tile work selection under GPU-memory budgets.
//!
//! Concrete raster, image and resource operations are injected by the host;
//! this module owns the scheduling walk and every stop/continue decision.

use crate::{
    tile_manager_policy::{
        ClassifyTileMemoryFit, MemoryUsage, RasterTaskLimitReached, ScheduledTileMemoryLimit,
        ShouldRasterOccludedTiles, TileMemoryFit, TilePriorityViolatesMemoryPolicy,
        TileSchedulingStats, TileSchedulingStopReason,
    },
    tile_priority::{PriorityBin, TilePriority},
    TileMemoryLimitPolicy,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileAssignmentConfig {
    pub memory_limit_policy: TileMemoryLimitPolicy,
    pub hard_memory_limit: MemoryUsage,
    pub soft_memory_limit: MemoryUsage,
    pub initial_memory_usage: MemoryUsage,
    pub raster_task_limit: usize,
    pub has_checker_imaged_tiles: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileAssignmentOutcome {
    pub stats: TileSchedulingStats,
    pub final_memory_usage: MemoryUsage,
    pub all_tiles_scheduled: bool,
    pub had_enough_memory_for_needed_tiles: bool,
}

pub trait TileAssignmentContext {
    type PreparedRasterCandidate;

    fn queue_is_empty(&self) -> bool;
    fn current_priority(&self) -> TilePriority;
    fn current_is_occluded(&self) -> bool;
    fn pop_current(&mut self);

    /// Marks the analysis as attempted and performs it against the real raster
    /// source. A `true` result means the adapter has installed solid-color draw
    /// state and sent the native tile-state notification.
    fn analyze_current_solid_color(&mut self) -> bool;
    fn current_is_prepaint_images_only(&self) -> bool;
    fn queue_current_for_image_processing(&mut self);

    /// Resolves the target color parameters once, at the same point as the
    /// native loop, and retains them in a call-scoped candidate token.
    fn prepare_current_raster_candidate(&mut self) -> Self::PreparedRasterCandidate;
    fn current_needs_raster(&self) -> bool;
    fn queue_checker_decode_only(&mut self, candidate: Self::PreparedRasterCandidate);
    fn current_memory_requirement(&self) -> MemoryUsage;

    /// Resource eviction remains an injected `compositor_resource` operation;
    /// Tile owns when and to what budget it must run.
    fn free_lower_priority_resources(
        &mut self,
        limit: &MemoryUsage,
        protected_priority: &TilePriority,
        usage: &mut MemoryUsage,
    );
    fn free_resources_to_hard_limit(&mut self, hard_limit: &MemoryUsage, usage: &mut MemoryUsage);

    /// Creates or reuses the concrete raster task and appends it to host work.
    /// `false` preserves the native "task could not be created" skip path.
    fn schedule_current_raster(
        &mut self,
        candidate: Self::PreparedRasterCandidate,
        schedule_priority: u32,
    ) -> bool;

    fn current_should_decode_checkered_images(&self) -> bool;
    fn queue_current_oom_checker_decode(&mut self);
    fn trace_memory_policy_violation(&mut self);
    fn report_hard_memory_limit_exceeded(&mut self);
}

/// Selects all work for one prepare cycle. Queue order is authoritative: the
/// first policy, task-count or memory barrier terminates the main walk.
pub fn AssignTileWork<C: TileAssignmentContext>(
    context: &mut C,
    config: TileAssignmentConfig,
) -> TileAssignmentOutcome {
    let mut trace = browser_tracing::span(
        "compositor.tile",
        "Compositor.Tile.AssignMemoryAndSelectRasterWork",
    );
    let mut stats = TileSchedulingStats::default();
    let mut memory_usage = config.initial_memory_usage;
    let mut schedule_priority = 1u32;
    let mut all_tiles_scheduled = true;
    let mut had_enough_memory_for_needed_tiles = true;
    while !context.queue_is_empty() {
        stats.candidates_considered += 1;
        let priority = context.current_priority();
        if TilePriorityViolatesMemoryPolicy(config.memory_limit_policy, &priority) {
            stats.stop_reason = TileSchedulingStopReason::MemoryPolicy;
            context.trace_memory_policy_violation();
            break;
        }
        debug_assert!(
            !context.current_is_occluded() || ShouldRasterOccludedTiles(config.memory_limit_policy)
        );

        if context.analyze_current_solid_color() {
            stats.solid_color_tiles += 1;
            context.pop_current();
            continue;
        }
        if context.current_is_prepaint_images_only() {
            stats.image_only_tiles += 1;
            context.queue_current_for_image_processing();
            context.pop_current();
            continue;
        }

        let prepared = context.prepare_current_raster_candidate();
        if !context.current_needs_raster() {
            stats.checker_decode_only_tiles += 1;
            context.queue_checker_decode_only(prepared);
            context.pop_current();
            continue;
        }
        if RasterTaskLimitReached(stats.scheduled_raster_tiles, config.raster_task_limit) {
            stats.stop_reason = TileSchedulingStopReason::RasterTaskLimit;
            all_tiles_scheduled = false;
            break;
        }

        let memory_required = context.current_memory_requirement();
        let tile_is_needed_now = priority.priority_bin == PriorityBin::NOW;
        let scheduled_tile_limit = ScheduledTileMemoryLimit(
            config.hard_memory_limit,
            config.soft_memory_limit,
            &memory_required,
            tile_is_needed_now,
        );
        context.free_lower_priority_resources(&scheduled_tile_limit, &priority, &mut memory_usage);
        match ClassifyTileMemoryFit(&memory_usage, &scheduled_tile_limit, tile_is_needed_now) {
            TileMemoryFit::Fits => {}
            TileMemoryFit::ExceedsHardLimitForNeededTile => {
                stats.stop_reason = TileSchedulingStopReason::HardMemoryLimit;
                context.report_hard_memory_limit_exceeded();
                had_enough_memory_for_needed_tiles = false;
                all_tiles_scheduled = false;
                break;
            }
            TileMemoryFit::ExceedsSoftLimit => {
                stats.stop_reason = TileSchedulingStopReason::SoftMemoryLimit;
                all_tiles_scheduled = false;
                break;
            }
        }

        if !context.schedule_current_raster(prepared, schedule_priority) {
            context.pop_current();
            continue;
        }
        schedule_priority += 1;
        stats.scheduled_raster_tiles += 1;
        memory_usage += &memory_required;
        context.pop_current();
    }
    context.free_resources_to_hard_limit(&config.hard_memory_limit, &mut memory_usage);

    if !had_enough_memory_for_needed_tiles && config.has_checker_imaged_tiles {
        while !context.queue_is_empty() {
            if context.current_priority().priority_bin > PriorityBin::NOW {
                break;
            }
            if context.current_should_decode_checkered_images() {
                context.queue_current_oom_checker_decode();
            }
            context.pop_current();
        }
    }

    stats.final_memory_bytes = memory_usage.memory_bytes();
    trace.set("candidates_considered", stats.candidates_considered as f64);
    trace.set("solid_color_tiles", stats.solid_color_tiles as f64);
    trace.set("image_only_tiles", stats.image_only_tiles as f64);
    trace.set(
        "checker_decode_only_tiles",
        stats.checker_decode_only_tiles as f64,
    );
    trace.set(
        "scheduled_raster_tiles",
        stats.scheduled_raster_tiles as f64,
    );
    trace.set("stop_reason", stats.stop_reason.TraceValue() as f64);
    trace.set("final_memory_bytes", stats.final_memory_bytes as f64);

    TileAssignmentOutcome {
        stats,
        final_memory_usage: memory_usage,
        all_tiles_scheduled,
        had_enough_memory_for_needed_tiles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile_priority::TileResolution;

    #[derive(Clone, Copy)]
    enum Kind {
        Solid,
        ImagesOnly,
        CheckerOnly,
        Raster { bytes: usize, schedules: bool },
    }

    #[derive(Clone, Copy)]
    struct Candidate {
        priority: TilePriority,
        kind: Kind,
        decode: bool,
    }

    struct Context {
        candidates: Vec<Candidate>,
        index: usize,
        scheduled: Vec<u32>,
        oom_decodes: usize,
    }

    impl Context {
        fn current(&self) -> Candidate {
            self.candidates[self.index]
        }
    }

    impl TileAssignmentContext for Context {
        type PreparedRasterCandidate = ();
        fn queue_is_empty(&self) -> bool {
            self.index == self.candidates.len()
        }
        fn current_priority(&self) -> TilePriority {
            self.current().priority
        }
        fn current_is_occluded(&self) -> bool {
            false
        }
        fn pop_current(&mut self) {
            self.index += 1;
        }
        fn analyze_current_solid_color(&mut self) -> bool {
            matches!(self.current().kind, Kind::Solid)
        }
        fn current_is_prepaint_images_only(&self) -> bool {
            matches!(self.current().kind, Kind::ImagesOnly)
        }
        fn queue_current_for_image_processing(&mut self) {}
        fn prepare_current_raster_candidate(&mut self) {}
        fn current_needs_raster(&self) -> bool {
            !matches!(self.current().kind, Kind::CheckerOnly)
        }
        fn queue_checker_decode_only(&mut self, _: ()) {}
        fn current_memory_requirement(&self) -> MemoryUsage {
            match self.current().kind {
                Kind::Raster { bytes, .. } => MemoryUsage::FromEstimatedBytes(bytes),
                _ => MemoryUsage::default(),
            }
        }
        fn free_lower_priority_resources(
            &mut self,
            _: &MemoryUsage,
            _: &TilePriority,
            _: &mut MemoryUsage,
        ) {
        }
        fn free_resources_to_hard_limit(&mut self, _: &MemoryUsage, _: &mut MemoryUsage) {}
        fn schedule_current_raster(&mut self, _: (), priority: u32) -> bool {
            let Kind::Raster { schedules, .. } = self.current().kind else {
                unreachable!()
            };
            if schedules {
                self.scheduled.push(priority);
            }
            schedules
        }
        fn current_should_decode_checkered_images(&self) -> bool {
            self.current().decode
        }
        fn queue_current_oom_checker_decode(&mut self) {
            self.oom_decodes += 1;
        }
        fn trace_memory_policy_violation(&mut self) {}
        fn report_hard_memory_limit_exceeded(&mut self) {}
    }

    fn priority(bin: PriorityBin) -> TilePriority {
        TilePriority::new(TileResolution::HIGH_RESOLUTION, bin, 0.0)
    }

    fn config(hard: usize, soft: usize) -> TileAssignmentConfig {
        TileAssignmentConfig {
            memory_limit_policy: TileMemoryLimitPolicy::ALLOW_ANYTHING,
            hard_memory_limit: MemoryUsage::new(hard, 10),
            soft_memory_limit: MemoryUsage::new(soft, 10),
            initial_memory_usage: MemoryUsage::default(),
            raster_task_limit: usize::MAX,
            has_checker_imaged_tiles: true,
        }
    }

    #[test]
    fn owns_candidate_categories_and_schedule_priority() {
        let mut context = Context {
            candidates: vec![
                Candidate {
                    priority: priority(PriorityBin::NOW),
                    kind: Kind::Solid,
                    decode: false,
                },
                Candidate {
                    priority: priority(PriorityBin::NOW),
                    kind: Kind::ImagesOnly,
                    decode: false,
                },
                Candidate {
                    priority: priority(PriorityBin::NOW),
                    kind: Kind::CheckerOnly,
                    decode: false,
                },
                Candidate {
                    priority: priority(PriorityBin::NOW),
                    kind: Kind::Raster {
                        bytes: 10,
                        schedules: true,
                    },
                    decode: false,
                },
            ],
            index: 0,
            scheduled: Vec::new(),
            oom_decodes: 0,
        };
        let outcome = AssignTileWork(&mut context, config(100, 50));
        assert_eq!(outcome.stats.solid_color_tiles, 1);
        assert_eq!(outcome.stats.image_only_tiles, 1);
        assert_eq!(outcome.stats.checker_decode_only_tiles, 1);
        assert_eq!(outcome.stats.scheduled_raster_tiles, 1);
        assert_eq!(context.scheduled, vec![1]);
    }

    #[test]
    fn hard_limit_stops_needed_tiles_and_scans_remaining_now_checker_work() {
        let now = priority(PriorityBin::NOW);
        let mut context = Context {
            candidates: vec![
                Candidate {
                    priority: now,
                    kind: Kind::Raster {
                        bytes: 101,
                        schedules: true,
                    },
                    decode: true,
                },
                Candidate {
                    priority: now,
                    kind: Kind::Raster {
                        bytes: 1,
                        schedules: true,
                    },
                    decode: true,
                },
            ],
            index: 0,
            scheduled: Vec::new(),
            oom_decodes: 0,
        };
        let outcome = AssignTileWork(&mut context, config(100, 50));
        assert_eq!(
            outcome.stats.stop_reason,
            TileSchedulingStopReason::HardMemoryLimit
        );
        assert!(!outcome.had_enough_memory_for_needed_tiles);
        assert!(!outcome.all_tiles_scheduled);
        assert_eq!(context.oom_decodes, 2);
    }

    #[test]
    fn raster_task_limit_stops_without_consuming_the_candidate() {
        let mut context = Context {
            candidates: vec![Candidate {
                priority: priority(PriorityBin::NOW),
                kind: Kind::Raster {
                    bytes: 1,
                    schedules: true,
                },
                decode: false,
            }],
            index: 0,
            scheduled: Vec::new(),
            oom_decodes: 0,
        };
        let mut limits = config(100, 50);
        limits.raster_task_limit = 0;
        let outcome = AssignTileWork(&mut context, limits);
        assert_eq!(
            outcome.stats.stop_reason,
            TileSchedulingStopReason::RasterTaskLimit
        );
        assert_eq!(context.index, 0);
    }
}
