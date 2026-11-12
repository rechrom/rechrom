#![allow(non_snake_case)]

use foundation::{RuleBreak, RuleVisibilityItems, Vector};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::grid_enums::GridTrackSizingDirection;

use super::super::gap_geometry::{BlockedStatus, ContainerType, GapGeometry};
use super::super::gap_intersection::GapIntersection;
use super::super::gap_utils::GapSegmentState;

pub struct CSSGapDecorationUtils;

impl CSSGapDecorationUtils {
    pub fn ResolveRuleBreakValue(
        style: &ComputedStyle,
        direction: GridTrackSizingDirection,
        container_type: ContainerType,
    ) -> RuleBreak {
        let rule_break = if direction == GridTrackSizingDirection::kForColumns {
            style.ColumnRuleBreak()
        } else {
            style.RowRuleBreak()
        };
        if container_type == ContainerType::kMultiColumn && rule_break == RuleBreak::kNormal {
            if direction == GridTrackSizingDirection::kForColumns {
                RuleBreak::kIntersection
            } else {
                RuleBreak::kNone
            }
        } else {
            rule_break
        }
    }

    pub fn ResolveRuleVisibilityItemsValue(
        style: &ComputedStyle,
        container_type: ContainerType,
        direction: GridTrackSizingDirection,
    ) -> RuleVisibilityItems {
        let visibility = if direction == GridTrackSizingDirection::kForColumns {
            style.ColumnRuleVisibilityItems()
        } else {
            style.RowRuleVisibilityItems()
        };
        if visibility != RuleVisibilityItems::kNormal {
            return visibility;
        }
        match container_type {
            ContainerType::kGrid | ContainerType::kGridLanes => RuleVisibilityItems::kAll,
            ContainerType::kFlex | ContainerType::kMultiColumn => RuleVisibilityItems::kBetween,
        }
    }

    pub fn HasOverlapJoin(style: &ComputedStyle, is_column_gap: bool) -> bool {
        let (cap_start, cap_end, junction_start, junction_end) = if is_column_gap {
            (
                style.ColumnRuleInsetCapStart(),
                style.ColumnRuleInsetCapEnd(),
                style.ColumnRuleInsetJunctionStart(),
                style.ColumnRuleInsetJunctionEnd(),
            )
        } else {
            (
                style.RowRuleInsetCapStart(),
                style.RowRuleInsetCapEnd(),
                style.RowRuleInsetJunctionStart(),
                style.RowRuleInsetJunctionEnd(),
            )
        };
        cap_start.IsOverlapJoin()
            || cap_end.IsOverlapJoin()
            || junction_start.IsOverlapJoin()
            || junction_end.IsOverlapJoin()
    }

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
