// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tile_manager.h:254-286
// Source: cc/tiles/tile_manager.cc:896-910,2323-2403
//
//! Tile-budget accounting and readiness policy. Resource allocation remains in
//! `compositor_resource`; this module owns the tile decisions made from sizes.

use crate::{
    tile::Tile,
    tile_draw_info::TileDrawResource,
    tile_priority::{PriorityBin, TilePriority},
    tile_types::TileTypes,
    TileMemoryLimitPolicy, TreePriority,
};
use std::ops::{AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemoryUsage {
    memory_bytes_: i64,
    resource_count_: i32,
}

impl MemoryUsage {
    pub fn new(memory_bytes: usize, resource_count: usize) -> Self {
        let result = Self {
            memory_bytes_: memory_bytes as i64,
            resource_count_: resource_count as i32,
        };
        debug_assert!(memory_bytes <= i64::MAX as usize);
        debug_assert!(resource_count <= i32::MAX as usize);
        result
    }

    pub fn FromEstimatedBytes(memory_bytes: usize) -> Self {
        Self::new(memory_bytes, 1)
    }

    pub fn FromTile<B: TileTypes>(tile: &Tile<B>) -> Self {
        let draw_info = tile.draw_info();
        if draw_info.has_resource() {
            return Self::new(draw_info.GetResource().memory_usage(), 1);
        }
        Self::default()
    }

    pub fn memory_bytes(&self) -> i64 {
        self.memory_bytes_
    }

    pub fn Exceeds(&self, limit: &Self) -> bool {
        self.memory_bytes_ > limit.memory_bytes_ || self.resource_count_ > limit.resource_count_
    }
}

impl AddAssign<&Self> for MemoryUsage {
    fn add_assign(&mut self, other: &Self) {
        self.memory_bytes_ += other.memory_bytes_;
        self.resource_count_ += other.resource_count_;
    }
}

impl SubAssign<&Self> for MemoryUsage {
    fn sub_assign(&mut self, other: &Self) {
        self.memory_bytes_ -= other.memory_bytes_;
        self.resource_count_ -= other.resource_count_;
    }
}

impl Sub<&Self> for MemoryUsage {
    type Output = Self;

    fn sub(self, other: &Self) -> Self {
        let mut result = self;
        result -= other;
        result
    }
}

#[derive(Default)]
pub struct Signals {
    pub activate_tile_tasks_completed: bool,
    pub draw_tile_tasks_completed: bool,
    pub all_tile_tasks_completed: bool,
    pub activate_gpu_work_completed: bool,
    pub draw_gpu_work_completed: bool,
    pub did_notify_ready_to_activate: bool,
    pub did_notify_ready_to_draw: bool,
    pub did_notify_all_tile_tasks_completed: bool,
}

pub fn TilePriorityViolatesMemoryPolicy(
    policy: TileMemoryLimitPolicy,
    priority: &TilePriority,
) -> bool {
    use TileMemoryLimitPolicy::*;
    match policy {
        ALLOW_NOTHING => true,
        ALLOW_ABSOLUTE_MINIMUM => priority.priority_bin > PriorityBin::NOW,
        ALLOW_PREPAINT_ONLY => priority.priority_bin > PriorityBin::SOON,
        ALLOW_ANYTHING => priority.distance_to_visible == f32::INFINITY,
    }
}

pub fn ShouldRasterOccludedTiles(policy: TileMemoryLimitPolicy) -> bool {
    use TileMemoryLimitPolicy::*;
    policy != ALLOW_NOTHING && policy != ALLOW_ABSOLUTE_MINIMUM
}

pub fn ScheduledTileMemoryLimit(
    hard_limit: MemoryUsage,
    soft_limit: MemoryUsage,
    memory_required: &MemoryUsage,
    tile_is_needed_now: bool,
) -> MemoryUsage {
    let limit = if tile_is_needed_now {
        hard_limit
    } else {
        soft_limit
    };
    limit - memory_required
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileMemoryFit {
    Fits,
    ExceedsSoftLimit,
    ExceedsHardLimitForNeededTile,
}

pub fn ClassifyTileMemoryFit(
    usage: &MemoryUsage,
    scheduled_tile_limit: &MemoryUsage,
    tile_is_needed_now: bool,
) -> TileMemoryFit {
    if !usage.Exceeds(scheduled_tile_limit) {
        TileMemoryFit::Fits
    } else if tile_is_needed_now {
        TileMemoryFit::ExceedsHardLimitForNeededTile
    } else {
        TileMemoryFit::ExceedsSoftLimit
    }
}

pub const fn RasterTaskLimitReached(scheduled_tasks: usize, task_limit: usize) -> bool {
    scheduled_tasks >= task_limit
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TileSchedulingStopReason {
    #[default]
    QueueExhausted,
    MemoryPolicy,
    RasterTaskLimit,
    SoftMemoryLimit,
    HardMemoryLimit,
}

impl TileSchedulingStopReason {
    pub const fn TraceValue(self) -> u8 {
        match self {
            Self::QueueExhausted => 0,
            Self::MemoryPolicy => 1,
            Self::RasterTaskLimit => 2,
            Self::SoftMemoryLimit => 3,
            Self::HardMemoryLimit => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TileSchedulingStats {
    pub candidates_considered: usize,
    pub solid_color_tiles: usize,
    pub image_only_tiles: usize,
    pub checker_decode_only_tiles: usize,
    pub scheduled_raster_tiles: usize,
    pub stop_reason: TileSchedulingStopReason,
    pub final_memory_bytes: i64,
}

pub fn ShouldWaitForAllRequiredTiles(
    tree_priority: TreePriority,
    memory_limit_policy: TileMemoryLimitPolicy,
    oom_freeze_mitigation_enabled: bool,
    did_oom_on_last_assign: bool,
) -> bool {
    let wait_for_all_required_tiles = tree_priority == TreePriority::SMOOTHNESS_TAKES_PRIORITY
        || memory_limit_policy == TileMemoryLimitPolicy::ALLOW_NOTHING;
    let should_skip_wait = oom_freeze_mitigation_enabled && did_oom_on_last_assign;
    wait_for_all_required_tiles && !should_skip_wait
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrimPrepaintAction {
    Skip,
    FreeResource,
    ClearUsedAndRetryLater,
}

pub fn DecideTrimPrepaintAction(
    priority: &TilePriority,
    violates_memory_policy: bool,
    tile_was_used: bool,
) -> TrimPrepaintAction {
    if priority.priority_bin <= PriorityBin::SOON || !violates_memory_policy {
        return TrimPrepaintAction::Skip;
    }
    if tile_was_used {
        TrimPrepaintAction::ClearUsedAndRetryLater
    } else {
        TrimPrepaintAction::FreeResource
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvictionStep {
    StopWithinLimit,
    StopAtPriorityBarrier,
    Evict,
}

pub fn DecideEvictionStep(
    usage: &MemoryUsage,
    limit: &MemoryUsage,
    candidate_priority: Option<&TilePriority>,
    protected_priority: Option<&TilePriority>,
) -> EvictionStep {
    if !usage.Exceeds(limit) {
        return EvictionStep::StopWithinLimit;
    }
    if let (Some(candidate), Some(protected)) = (candidate_priority, protected_priority) {
        if !protected.IsHigherPriorityThan(candidate) {
            return EvictionStep::StopAtPriorityBarrier;
        }
    }
    EvictionStep::Evict
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoRasterWorkAction {
    WaitForRequiredTiles { schedule_another_prepare: bool },
    MarkRequiredTilesOutOfMemory,
}

pub fn DecideNoRasterWorkAction(
    tree_priority: TreePriority,
    memory_limit_policy: TileMemoryLimitPolicy,
    oom_freeze_mitigation_enabled: bool,
    did_oom_on_last_assign: bool,
    all_tiles_scheduled: bool,
) -> NoRasterWorkAction {
    if ShouldWaitForAllRequiredTiles(
        tree_priority,
        memory_limit_policy,
        oom_freeze_mitigation_enabled,
        did_oom_on_last_assign,
    ) {
        return NoRasterWorkAction::WaitForRequiredTiles {
            schedule_another_prepare: !all_tiles_scheduled,
        };
    }
    NoRasterWorkAction::MarkRequiredTilesOutOfMemory
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needed_tiles_use_the_hard_budget() {
        let hard = MemoryUsage::new(100, 10);
        let soft = MemoryUsage::new(50, 5);
        let required = MemoryUsage::FromEstimatedBytes(20);
        assert_eq!(
            ScheduledTileMemoryLimit(hard, soft, &required, true),
            MemoryUsage::new(80, 9)
        );
        assert_eq!(
            ScheduledTileMemoryLimit(hard, soft, &required, false),
            MemoryUsage::new(30, 4)
        );
    }

    #[test]
    fn reports_which_budget_was_exceeded() {
        let usage = MemoryUsage::new(81, 1);
        let limit = MemoryUsage::new(80, 9);
        assert_eq!(
            ClassifyTileMemoryFit(&usage, &limit, true),
            TileMemoryFit::ExceedsHardLimitForNeededTile
        );
        assert_eq!(
            ClassifyTileMemoryFit(&usage, &limit, false),
            TileMemoryFit::ExceedsSoftLimit
        );
    }

    #[test]
    fn oom_mitigation_skips_wait_that_would_freeze_the_pipeline() {
        assert!(ShouldWaitForAllRequiredTiles(
            TreePriority::SMOOTHNESS_TAKES_PRIORITY,
            TileMemoryLimitPolicy::ALLOW_ANYTHING,
            false,
            true,
        ));
        assert!(!ShouldWaitForAllRequiredTiles(
            TreePriority::SMOOTHNESS_TAKES_PRIORITY,
            TileMemoryLimitPolicy::ALLOW_ANYTHING,
            true,
            true,
        ));
    }

    #[test]
    fn trim_gives_recently_used_prepaint_tiles_one_more_cycle() {
        let priority = TilePriority::new(
            crate::tile_priority::TileResolution::HIGH_RESOLUTION,
            PriorityBin::EVENTUALLY,
            10.0,
        );
        assert_eq!(
            DecideTrimPrepaintAction(&priority, true, true),
            TrimPrepaintAction::ClearUsedAndRetryLater
        );
        assert_eq!(
            DecideTrimPrepaintAction(&priority, true, false),
            TrimPrepaintAction::FreeResource
        );
    }

    #[test]
    fn eviction_respects_priority_barrier() {
        let usage = MemoryUsage::new(101, 2);
        let limit = MemoryUsage::new(100, 2);
        let protected = TilePriority::new(
            crate::tile_priority::TileResolution::HIGH_RESOLUTION,
            PriorityBin::SOON,
            1.0,
        );
        let higher_candidate = TilePriority::new(
            crate::tile_priority::TileResolution::HIGH_RESOLUTION,
            PriorityBin::NOW,
            0.0,
        );
        assert_eq!(
            DecideEvictionStep(&usage, &limit, Some(&higher_candidate), Some(&protected)),
            EvictionStep::StopAtPriorityBarrier
        );
    }

    #[test]
    fn no_work_policy_requests_retry_only_when_waiting_on_unscheduled_tiles() {
        assert_eq!(
            DecideNoRasterWorkAction(
                TreePriority::SMOOTHNESS_TAKES_PRIORITY,
                TileMemoryLimitPolicy::ALLOW_ANYTHING,
                false,
                false,
                false,
            ),
            NoRasterWorkAction::WaitForRequiredTiles {
                schedule_another_prepare: true
            }
        );
        assert_eq!(
            DecideNoRasterWorkAction(
                TreePriority::SAME_PRIORITY_FOR_BOTH_TREES,
                TileMemoryLimitPolicy::ALLOW_ANYTHING,
                false,
                false,
                true,
            ),
            NoRasterWorkAction::MarkRequiredTilesOutOfMemory
        );
    }
}
