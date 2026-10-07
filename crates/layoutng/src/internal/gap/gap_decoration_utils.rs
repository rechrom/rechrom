#![allow(non_snake_case)]

use foundation::{RuleVisibilityItems, Vector};
use layoutng_style::style::grid_enums::GridTrackSizingDirection;

use super::gap_geometry::{BlockedStatus, ContainerType, GapGeometry};
use super::gap_intersection::GapIntersection;
use super::gap_utils::GapSegmentState;

// cpp: layoutng/internal/gap/gap_decoration_utils.h:8-20
pub struct CSSGapDecorationUtils;

impl CSSGapDecorationUtils {
    // cpp: layoutng/internal/gap/gap_decoration_utils.h:12-12
    // cpp: layoutng/internal/gap/gap_decoration_utils.cc:8-33
    pub fn IsRuleSegmentVisible(
        gap_state: GapSegmentState,
        rule_visibility: RuleVisibilityItems,
    ) -> bool {
        if rule_visibility == RuleVisibilityItems::kAll {
            return true;
        }
        match rule_visibility {
            RuleVisibilityItems::kAround => !gap_state.IsEmpty(),
            RuleVisibilityItems::kBetween => !gap_state.HasEmptyStatus(),
            RuleVisibilityItems::kAll | RuleVisibilityItems::kNormal => {
                panic!("rule visibility must be resolved before gap decoration")
            }
        }
    }

    // cpp: layoutng/internal/gap/gap_decoration_utils.h:13-19
    // cpp: layoutng/internal/gap/gap_decoration_utils.cc:35-74
    pub fn HasCrossGapSegment(
        cross_direction: GridTrackSizingDirection,
        gap_index: u32,
        intersection_index: u32,
        rule_visibility: RuleVisibilityItems,
        cross_rule_visibility: RuleVisibilityItems,
        gap_geometry: &GapGeometry,
        intersections: &Vector<GapIntersection>,
    ) -> bool {
        if (gap_geometry.GetContainerType() != ContainerType::kGrid
            && gap_geometry.GetContainerType() != ContainerType::kMultiColumn)
            || rule_visibility != RuleVisibilityItems::kBetween
        {
            return true;
        }

        let cross_gap_index = intersection_index.wrapping_sub(1);
        let cross_intersection_index = gap_index.wrapping_add(1);

        let is_cross_before_visible = Self::IsRuleSegmentVisible(
            gap_geometry.GetIntersectionGapSegmentState(
                cross_direction,
                cross_gap_index,
                gap_index,
            ),
            cross_rule_visibility,
        );
        let is_cross_after_visible = Self::IsRuleSegmentVisible(
            gap_geometry.GetIntersectionGapSegmentState(
                cross_direction,
                cross_gap_index,
                cross_intersection_index,
            ),
            cross_rule_visibility,
        );

        let cross_blocked = gap_geometry.GetIntersectionBlockedStatus(
            cross_direction,
            cross_gap_index,
            cross_intersection_index,
            intersections,
        );

        let is_cross_before_present = is_cross_before_visible
            && !cross_blocked.HasBlockedStatus(BlockedStatus::kBlockedBefore);
        let is_cross_after_present =
            is_cross_after_visible && !cross_blocked.HasBlockedStatus(BlockedStatus::kBlockedAfter);

        is_cross_before_present || is_cross_after_present
    }
}
