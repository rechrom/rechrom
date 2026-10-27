use crate::{
    grid_data::GridPlacementData,
    grid_item::ComputeOutOfFlowItemPlacement,
    grid_layout_utils::{TrackEndOffset, TrackStartOffset},
};
use layoutng_assembly::internal::{grid_item::GridItemData, grid_layout_data::GridLayoutData};
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_style::style::{
    computed_style::ComputedStyle, grid_enums::GridTrackSizingDirection::kForColumns,
};

// cpp: layoutng_grid/grid_oof_placement.h:16-21
// cpp: layoutng_grid/grid_oof_placement.cc:12-61
pub fn ComputeGridOutOfFlowItemContainingRect(
    placement: &GridPlacementData,
    layout: &GridLayoutData,
    style: &ComputedStyle,
    padding: &LogicalRect,
    item: &mut GridItemData,
) -> LogicalRect {
    debug_assert!(item.IsOutOfFlow());
    let columns = unsafe { &*layout.Columns() };
    let rows = unsafe { &*layout.Rows() };
    ComputeOutOfFlowItemPlacement(item, columns, placement, style);
    ComputeOutOfFlowItemPlacement(item, rows, placement, style);
    let mut rect = *padding;
    let pos = &item.column_placement;
    if pos.range_index.begin != u32::MAX {
        debug_assert_ne!(pos.offset_in_range.begin, u32::MAX);
        rect.ShiftInlineStartEdgeTo(TrackStartOffset(
            columns,
            pos.range_index.begin,
            pos.offset_in_range.begin,
        ));
    }
    if pos.range_index.end != u32::MAX {
        debug_assert_ne!(pos.offset_in_range.end, u32::MAX);
        rect.ShiftInlineEndEdgeTo(TrackEndOffset(
            columns,
            pos.range_index.end,
            pos.offset_in_range.end,
        ));
    }
    let pos = &item.row_placement;
    if pos.range_index.begin != u32::MAX {
        debug_assert_ne!(pos.offset_in_range.begin, u32::MAX);
        rect.ShiftBlockStartEdgeTo(TrackStartOffset(
            rows,
            pos.range_index.begin,
            pos.offset_in_range.begin,
        ));
    }
    if pos.range_index.end != u32::MAX {
        debug_assert_ne!(pos.offset_in_range.end, u32::MAX);
        rect.ShiftBlockEndEdgeTo(TrackEndOffset(
            rows,
            pos.range_index.end,
            pos.offset_in_range.end,
        ));
    }
    rect
}

// cpp: layoutng_grid/grid_oof_placement.h:23-28
// cpp: layoutng_grid/grid_oof_placement.cc:63-100
pub fn ComputeGridLanesOutOfFlowItemContainingRect(
    placement: &GridPlacementData,
    layout: &GridLayoutData,
    style: &ComputedStyle,
    padding: &LogicalRect,
    item: &mut GridItemData,
) -> LogicalRect {
    debug_assert!(item.IsOutOfFlow());
    let columns = style.GridLanesTrackSizingDirection() == kForColumns;
    let tracks = unsafe {
        &*if columns {
            layout.Columns()
        } else {
            layout.Rows()
        }
    };
    ComputeOutOfFlowItemPlacement(item, tracks, placement, style);
    let mut rect = *padding;
    let pos = if columns {
        &item.column_placement
    } else {
        &item.row_placement
    };
    if pos.range_index.begin != u32::MAX {
        debug_assert_ne!(pos.offset_in_range.begin, u32::MAX);
        let offset = TrackStartOffset(tracks, pos.range_index.begin, pos.offset_in_range.begin);
        if columns {
            rect.ShiftInlineStartEdgeTo(offset);
        } else {
            rect.ShiftBlockStartEdgeTo(offset);
        }
    }
    if pos.range_index.end != u32::MAX {
        debug_assert_ne!(pos.offset_in_range.end, u32::MAX);
        let offset = TrackEndOffset(tracks, pos.range_index.end, pos.offset_in_range.end);
        if columns {
            rect.ShiftInlineEndEdgeTo(offset);
        } else {
            rect.ShiftBlockEndEdgeTo(offset);
        }
    }
    rect
}
