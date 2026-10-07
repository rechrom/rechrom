use crate::{grid_data::GridPlacementData, grid_placement::GridPlacement};
use foundation::{
    IsParallelWritingMode, LayoutUnit, LogicalToLogical, LogicalToPhysical, PhysicalToLogical,
};
use layoutng_assembly::internal::{
    baseline_utils::{BaselineGroup, DetermineBaselineGroupDefault, DetermineBaselineWritingMode},
    block_node::BlockNode,
    constraint_space::AutoSizeBehavior,
    grid_item::{AxisEdge, GridItemData, GridItems, GridPlacementData as OpaquePlacementData},
    grid_track_collection::GridLayoutTrackCollection,
};
use layoutng_style::style::{
    computed_style::ComputedStyle,
    computed_style_constants::{ItemPosition, OverflowAlignment},
    grid_area::GridSpan,
    grid_enums::GridTrackSizingDirection::{self, *},
    style_self_alignment_data::StyleSelfAlignmentData,
};

// cpp: layoutng_grid/grid_item.cc:19-154
fn AxisEdgeFromItemPosition(
    direction: GridTrackSizingDirection,
    subgridded: bool,
    replaced: bool,
    out_of_flow: bool,
    style: &ComputedStyle,
    parent: &ComputedStyle,
    root: &ComputedStyle,
    auto_behavior: &mut AutoSizeBehavior,
    safe: &mut bool,
) -> AxisEdge {
    if subgridded {
        *auto_behavior = AutoSizeBehavior::kStretchImplicit;
        *safe = true;
        return AxisEdge::kStart;
    }
    let columns = direction == kForColumns;
    let root_direction = root.GetWritingDirection();
    let normal =
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kNormal, OverflowAlignment::kDefault);
    let alignment = if columns
        == IsParallelWritingMode(root_direction.GetWritingMode(), parent.GetWritingMode())
    {
        style.ResolvedJustifySelf(&normal, parent)
    } else {
        style.ResolvedAlignSelf(&normal, parent)
    };
    *auto_behavior = AutoSizeBehavior::kFitContent;
    *safe = alignment.Overflow() == OverflowAlignment::kSafe;
    let applies = if !parent.IsDisplayGridLanes() {
        true
    } else {
        parent.GridLanesTrackSizingDirection() == direction
    };
    if style.MayHaveMargin() && !out_of_flow {
        let start_auto = if columns {
            style.MarginInlineStartUsing(root).IsAuto()
        } else {
            style.MarginBlockStartUsing(root).IsAuto()
        };
        let end_auto = if columns {
            style.MarginInlineEndUsing(root).IsAuto()
        } else {
            style.MarginBlockEndUsing(root).IsAuto()
        };
        if start_auto || end_auto {
            *safe = true;
        }
        if start_auto && end_auto {
            return AxisEdge::kCenter;
        } else if start_auto {
            return AxisEdge::kEnd;
        } else if end_auto {
            return AxisEdge::kStart;
        }
    }
    match alignment.GetPosition() {
        ItemPosition::kSelfStart | ItemPosition::kSelfEnd => {
            let physical = LogicalToPhysical::new(
                style.GetWritingDirection(),
                AxisEdge::kStart,
                AxisEdge::kEnd,
                AxisEdge::kStart,
                AxisEdge::kEnd,
            );
            let logical = PhysicalToLogical::new(
                root_direction,
                physical.Top(),
                physical.Right(),
                physical.Bottom(),
                physical.Left(),
            );
            if alignment.GetPosition() == ItemPosition::kSelfStart {
                if columns {
                    logical.InlineStart()
                } else {
                    logical.BlockStart()
                }
            } else if columns {
                logical.InlineEnd()
            } else {
                logical.BlockEnd()
            }
        }
        ItemPosition::kAnchorCenter | ItemPosition::kCenter => AxisEdge::kCenter,
        ItemPosition::kFlexStart | ItemPosition::kStart => AxisEdge::kStart,
        ItemPosition::kFlexEnd | ItemPosition::kEnd => AxisEdge::kEnd,
        ItemPosition::kStretch => {
            if applies {
                *auto_behavior = AutoSizeBehavior::kStretchExplicit;
            }
            AxisEdge::kStart
        }
        ItemPosition::kBaseline => AxisEdge::kFirstBaseline,
        ItemPosition::kLastBaseline => AxisEdge::kLastBaseline,
        ItemPosition::kLeft => {
            debug_assert!(columns);
            if root_direction.IsLtr() {
                AxisEdge::kStart
            } else {
                AxisEdge::kEnd
            }
        }
        ItemPosition::kRight => {
            debug_assert!(columns);
            if root_direction.IsRtl() {
                AxisEdge::kStart
            } else {
                AxisEdge::kEnd
            }
        }
        ItemPosition::kNormal => {
            if applies {
                *auto_behavior = if replaced {
                    AutoSizeBehavior::kFitContent
                } else {
                    AutoSizeBehavior::kStretchImplicit
                };
            }
            AxisEdge::kStart
        }
        ItemPosition::kLegacy | ItemPosition::kAuto => unreachable!("unresolved alignment"),
    }
}
// cpp: layoutng_grid/grid_item.cc:159-271
#[unsafe(no_mangle)]
pub extern "Rust" fn GridItemDataNewProvider(
    node: BlockNode,
    parent: &ComputedStyle,
    root: &ComputedStyle,
    consider_columns: bool,
    consider_rows: bool,
) -> GridItemData {
    let mut item = GridItemData {
        node,
        is_considered_for_column_sizing: false,
        is_considered_for_row_sizing: false,
        parent_grid_font_baseline: parent.GetFontBaseline(),
        ..GridItemData::default()
    };
    let style = item.node.Style();
    let root_direction = root.GetWritingDirection();
    let mode = style.GetWritingMode();
    item.is_parallel_with_root_grid = IsParallelWritingMode(root_direction.GetWritingMode(), mode);
    item.column_baseline_writing_mode = DetermineBaselineWritingMode(root_direction, mode, false);
    item.row_baseline_writing_mode = DetermineBaselineWritingMode(root_direction, mode, true);
    if item.node.IsGrid()
        && !item.node.ShouldApplyLayoutContainment()
        && !item.node.ShouldApplyPaintContainment()
        && !style.IsContainerForSizeContainerQueries()
    {
        item.has_subgridded_columns = if item.is_parallel_with_root_grid {
            style.GridTemplateColumns().IsSubgriddedAxis()
        } else {
            style.GridTemplateRows().IsSubgriddedAxis()
        };
        item.has_subgridded_rows = if item.is_parallel_with_root_grid {
            style.GridTemplateRows().IsSubgriddedAxis()
        } else {
            style.GridTemplateColumns().IsSubgriddedAxis()
        };
        if parent.IsDisplayGridLanes() {
            if parent.GridLanesTrackSizingDirection() == kForColumns {
                item.has_subgridded_rows = false;
            } else {
                item.has_subgridded_columns = false;
            }
        }
    }
    let out_of_flow = item.node.IsOutOfFlowPositioned();
    let replaced = item.node.IsReplaced();
    let mut safe = false;
    item.column_alignment = AxisEdgeFromItemPosition(
        kForColumns,
        item.has_subgridded_columns,
        replaced,
        out_of_flow,
        style,
        parent,
        root,
        &mut item.column_auto_behavior,
        &mut safe,
    );
    item.is_overflow_safe_for_columns = safe;
    item.column_baseline_group = DetermineBaselineGroupDefault(
        root_direction,
        item.column_baseline_writing_mode,
        false,
        item.column_alignment == AxisEdge::kLastBaseline,
    );
    item.row_alignment = AxisEdgeFromItemPosition(
        kForRows,
        item.has_subgridded_rows,
        replaced,
        out_of_flow,
        style,
        parent,
        root,
        &mut item.row_auto_behavior,
        &mut safe,
    );
    item.is_overflow_safe_for_rows = safe;
    item.row_baseline_group = DetermineBaselineGroupDefault(
        root_direction,
        item.row_baseline_writing_mode,
        true,
        item.row_alignment == AxisEdge::kLastBaseline,
    );
    let converter = LogicalToLogical::new(
        style.GetWritingDirection(),
        root_direction,
        false,
        true,
        false,
        true,
    );
    item.is_opposite_direction_in_root_grid_columns = converter.InlineStart();
    item.is_opposite_direction_in_root_grid_rows = converter.BlockStart();
    if consider_columns {
        item.must_consider_grid_items_for_column_sizing = item.has_subgridded_columns;
        item.is_considered_for_column_sizing = !item.has_subgridded_columns;
    }
    if consider_rows {
        item.must_consider_grid_items_for_row_sizing = item.has_subgridded_rows;
        item.is_considered_for_row_sizing = !item.has_subgridded_rows;
    }
    item
}
// cpp: layoutng_grid/grid_item.cc:273-328
#[unsafe(no_mangle)]
pub extern "Rust" fn GridItemDataSetAlignmentFallbackProvider(
    item: &mut GridItemData,
    direction: GridTrackSizingDirection,
    synthesized: bool,
) {
    if !item.IsBaselineSpecified(direction) {
        return;
    }
    let can_participate = if synthesized
        && (item.IsSpanningIntrinsicTrack(direction) || item.IsSpanningFlexibleTrack(direction))
    {
        let style = item.node.Style();
        let parallel = item.is_parallel_with_root_grid == (direction == kForRows);
        if parallel {
            !style.LogicalHeight().HasPercentOrStretch()
                && !style.LogicalMinHeight().HasPercentOrStretch()
                && !style.LogicalMaxHeight().HasPercentOrStretch()
        } else {
            !style.LogicalWidth().HasPercentOrStretch()
                && !style.LogicalMinWidth().HasPercentOrStretch()
                && !style.LogicalMaxWidth().HasPercentOrStretch()
        }
    } else {
        true
    };
    let fallback = if can_participate {
        None
    } else {
        Some(if item.BaselineGroup(direction) == BaselineGroup::kMajor {
            AxisEdge::kStart
        } else {
            AxisEdge::kEnd
        })
    };
    if direction == kForColumns {
        item.column_fallback_alignment = fallback;
    } else {
        item.row_fallback_alignment = fallback;
    }
}
// cpp: layoutng_grid/grid_item.cc:330-339
#[unsafe(no_mangle)]
pub extern "Rust" fn GridItemDataUpdateSpanProvider(
    item: &mut GridItemData,
    span: &GridSpan,
    direction: GridTrackSizingDirection,
    start_offset: u32,
    tracks: &GridLayoutTrackCollection,
) {
    item.resolved_position.SetSpan(span, direction);
    item.MaybeTranslateSpan(start_offset, direction);
    item.ResetPlacementIndices();
    GridItemDataComputeSetIndicesProvider(item, tracks);
}
// cpp: layoutng_grid/grid_item.cc:341-381
#[unsafe(no_mangle)]
pub extern "Rust" fn GridItemDataComputeSetIndicesProvider(
    item: &mut GridItemData,
    tracks: &GridLayoutTrackCollection,
) {
    debug_assert!(!item.IsOutOfFlow());
    let direction = tracks.Direction();
    debug_assert!(item.MustCachePlacementIndices(direction));
    let start = item.StartLine(direction);
    let end = item.EndLine(direction);
    let range = item.RangeIndices(direction);
    if cfg!(debug_assertions) && range.begin != u32::MAX {
        debug_assert_eq!(tracks.RangeIndexFromGridLine(start), range.begin);
        debug_assert_eq!(tracks.RangeIndexFromGridLine(end - 1), range.end);
    }
    if range.begin == u32::MAX {
        debug_assert_eq!(range.end, u32::MAX);
        range.begin = tracks.RangeIndexFromGridLine(start);
        range.end = tracks.RangeIndexFromGridLine(end - 1);
    }
    debug_assert!(range.end < tracks.RangeCount());
    debug_assert!(range.begin <= range.end);
    let begin = tracks.RangeBeginSetIndex(range.begin);
    let end = tracks.RangeBeginSetIndex(range.end) + tracks.RangeSetCount(range.end);
    let set = if direction == kForColumns {
        &mut item.column_set_indices
    } else {
        &mut item.row_set_indices
    };
    set.begin = begin;
    set.end = end;
}
// cpp: layoutng_grid/grid_item.cc:383-452
pub fn ComputeOutOfFlowItemPlacement(
    item: &mut GridItemData,
    tracks: &GridLayoutTrackCollection,
    placement_data: &GridPlacementData,
    grid_style: &ComputedStyle,
) {
    debug_assert!(item.IsOutOfFlow());
    let direction = tracks.Direction();
    let columns = direction == kForColumns;
    let placement = if columns {
        &mut item.column_placement
    } else {
        &mut item.row_placement
    };
    let start = &mut placement.offset_in_range.begin;
    let end = &mut placement.offset_in_range.end;
    GridPlacement::ResolveOutOfFlowItemGridLines(
        tracks,
        &placement_data.line_resolver,
        grid_style,
        item.node.Style(),
        placement_data.StartOffset(direction),
        start,
        end,
    );
    if cfg!(debug_assertions) {
        if *start != u32::MAX && *end != u32::MAX {
            debug_assert!(*end <= tracks.EndLineOfImplicitGrid());
            debug_assert!(*start < *end);
        } else if *start != u32::MAX {
            debug_assert!(*start <= tracks.EndLineOfImplicitGrid());
        } else if *end != u32::MAX {
            debug_assert!(*end <= tracks.EndLineOfImplicitGrid());
        }
    }
    let count = tracks.RangeCount();
    let start_range = &mut placement.range_index.begin;
    if *start != u32::MAX {
        if count == 0 {
            debug_assert_eq!(*start, 0);
            *start_range = 0;
        } else {
            *start_range = if *start < tracks.EndLineOfImplicitGrid() {
                tracks.RangeIndexFromGridLine(*start)
            } else {
                count - 1
            };
            *start -= tracks.RangeStartLine(*start_range);
        }
    }
    let end_range = &mut placement.range_index.end;
    if *end != u32::MAX {
        if count == 0 {
            debug_assert_eq!(*end, 0);
            *end_range = 0;
        } else {
            *end_range = if *end != 0 {
                tracks.RangeIndexFromGridLine(*end - 1)
            } else {
                0
            };
            *end -= tracks.RangeStartLine(*end_range);
        }
    }
}
// The layoutng header intentionally forward-declares GridPlacementData. This
// typed Rust provider connects that pointer to its actual Grid package owner.
#[unsafe(no_mangle)]
pub extern "Rust" fn GridItemDataComputeOutOfFlowItemPlacementProvider(
    item: &mut GridItemData,
    tracks: &GridLayoutTrackCollection,
    placement: &OpaquePlacementData,
    style: &ComputedStyle,
) {
    let data = unsafe { &*(placement as *const OpaquePlacementData).cast::<GridPlacementData>() };
    ComputeOutOfFlowItemPlacement(item, tracks, data, style);
}
// cpp: layoutng_grid/grid_item.cc:454-469
#[unsafe(no_mangle)]
pub extern "Rust" fn GridItemDataCalculateAvailableSizeProvider(
    item: &GridItemData,
    tracks: &GridLayoutTrackCollection,
    start_offset: *mut LayoutUnit,
) -> LayoutUnit {
    debug_assert!(!item.is_subgridded_to_parent_grid);
    debug_assert!(!item.IsOutOfFlow());
    let indices = item.SetIndices(tracks.Direction());
    if !start_offset.is_null() {
        unsafe {
            *start_offset = tracks.GetSetOffset(indices.begin);
        }
    }
    let available = tracks.CalculateSetSpanSizeRange(indices.begin, indices.end);
    if available.MightBeSaturated() {
        LayoutUnit::default()
    } else {
        available
    }
}
// cpp: layoutng_grid/grid_item.cc:471-476
#[unsafe(no_mangle)]
pub extern "Rust" fn GridItemsAppendProvider(items: &mut GridItems, other: *mut GridItems) {
    let other = unsafe { &mut *other }.GridPackageItemsMut();
    let data = items.GridPackageItemsMut();
    data.reserve(other.len());
    for item in other.iter() {
        data.push(*item);
    }
}
// cpp: layoutng_grid/grid_item.cc:478-480
#[unsafe(no_mangle)]
pub extern "Rust" fn GridItemsSortByOrderPropertyProvider(items: &mut GridItems) {
    items
        .GridPackageItemsMut()
        .sort_by_key(|item| unsafe { &*item.Get() }.node.Style().Order());
}
