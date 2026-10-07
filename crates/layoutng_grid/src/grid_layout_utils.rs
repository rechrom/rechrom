use crate::{
    grid_node::GridNode,
    grid_sizing_tree::{GridSizingSubtree, SubgriddedItemData},
    grid_track_sizing_algorithm::{GridTrackSizingAlgorithm, SizingConstraint},
    layout_grid::LayoutGrid,
};
use font_engine::FontBaseline;
use foundation::LayoutUnit;
use foundation::{kIndefiniteSize, HashMap, LengthType, MinimumValueForLength, WritingMode};
use layoutng_assembly::internal::grid_item::GridItemData;
use layoutng_assembly::internal::grid_track_collection::GridLayoutTrackCollection;
use layoutng_assembly::logical_box_fragment::LogicalBoxFragment;
use layoutng_assembly::{
    box_fragment_builder::BoxFragmentBuilder,
    internal::{
        baseline_utils::BaselineGroup,
        block_node::BlockNode,
        constraint_space::ConstraintSpace,
        disable_layout_side_effects_scope::DisableLayoutSideEffectsScope,
        gap::gap_geometry::GapGeometry,
        grid_item::{AxisEdge, GridItems},
        grid_layout_data::GridLayoutData,
        grid_track_collection::{GridSizingTrackCollection, GridTrackBaselines, PropertyId},
        layout_input_node::MinMaxSizesFloatInput,
        length_utils::*,
        min_max_sizes::{MinMaxSizes, MinMaxSizesResult},
    },
    layout_result::LayoutResult,
    physical_box_fragment::PhysicalBoxFragment,
};
use layoutng_geometry::geometry::{
    box_strut::BoxStrut,
    fragment_geometry::FragmentGeometry,
    logical_offset::LogicalOffset,
    logical_size::LogicalSize,
    static_position::{BlockEdge, InlineEdge, LogicalStaticPosition},
};
use layoutng_style::style::{
    computed_style::ComputedStyle,
    grid_area::{GridArea, GridSpan},
    grid_enums::GridTrackSizingDirection::{self, *},
    grid_track_list::{GridTrackList, GridTrackRepeatType},
    grid_track_size::{GridTrackSize, GridTrackSizeHashTraits},
};

// cpp: layoutng_grid/grid_layout_utils.h:72-91
// C++ virtual calls map to an object-safe trait with the same required methods.
pub trait BaselineAccumulator {
    fn Accumulate(
        &mut self,
        item: &GridItemData,
        fragment: &LogicalBoxFragment,
        block_offset: LayoutUnit,
        item_stacking_position: LayoutUnit,
        item_moved_to_earlier_opening: bool,
    );
    fn FirstBaseline(&self) -> Option<LayoutUnit>;
    fn LastBaseline(&self) -> Option<LayoutUnit>;
}

// cpp: layoutng_grid/grid_layout_utils.h:41-48
// cpp: layoutng_grid/grid_layout_utils.cc:30-56
pub fn AlignmentOffset(
    container: LayoutUnit,
    size: LayoutUnit,
    start: LayoutUnit,
    end: LayoutUnit,
    baseline: LayoutUnit,
    edge: AxisEdge,
    safe: bool,
) -> LayoutUnit {
    let mut free = container - size - start - end;
    if safe {
        free = free.ClampNegativeToZero();
    }
    match edge {
        AxisEdge::kStart => start,
        AxisEdge::kCenter => start + free / 2,
        AxisEdge::kEnd => start + free,
        AxisEdge::kFirstBaseline | AxisEdge::kLastBaseline => baseline,
    }
}
// cpp: layoutng_grid/grid_layout_utils.h:50-68
#[derive(Clone, Copy)]
pub struct GridTrackGap {
    pub line_index: u32,
    pub center_offset: LayoutUnit,
}
#[derive(Default)]
pub struct GridTrackGapData {
    pub gaps: Vec<GridTrackGap>,
    pub content_start: LayoutUnit,
    pub content_end: LayoutUnit,
    pub track_count: u32,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GridTrackGapType {
    kMain,
    kCross,
}
// cpp: layoutng_grid/grid_layout_utils.cc:58-127
pub fn BuildGridTrackGapData(
    tracks: &GridLayoutTrackCollection,
    kind: GridTrackGapType,
    geometry: &mut GapGeometry,
) -> GridTrackGapData {
    let positions = LayoutGrid::ComputeExpandedPositions(tracks);
    let mut data = GridTrackGapData {
        track_count: positions.len() as u32 - 1,
        ..Default::default()
    };
    if !tracks.HasNonCollapsedLine() {
        return data;
    }
    data.content_start = positions[0];
    data.content_end = *positions.last().unwrap();
    let hint = data.track_count.saturating_sub(1);
    data.gaps.reserve(hint as usize);
    if kind == GridTrackGapType::kMain {
        geometry.ReserveMainGaps(hint);
    } else {
        geometry.ReserveCrossGaps(hint);
    }
    let first = tracks.FirstNonCollapsedLineIndex();
    let gutter = tracks.GutterSize();
    for range in 0..tracks.RangeCount() {
        if tracks
            .RangeProperties(range)
            .HasProperty(PropertyId::kIsCollapsed)
        {
            continue;
        }
        let start = tracks.RangeStartLine(range);
        let end = start + tracks.RangeTrackCount(range);
        let mut line = start;
        if start == first {
            line += 1;
        }
        while line < end {
            let center = LayoutUnit::from_f32(positions[line as usize] - gutter / 2.0f32);
            data.gaps.push(GridTrackGap {
                line_index: line,
                center_offset: center,
            });
            if kind == GridTrackGapType::kMain {
                geometry.AddMainGap(center);
            } else {
                geometry.AddCrossGap(LogicalOffset::new(center, LayoutUnit::default()));
            }
            line += 1;
        }
    }
    assert_eq!(
        data.gaps.len() as u32,
        if kind == GridTrackGapType::kMain {
            geometry.MainGapCount()
        } else {
            geometry.CrossGapCount()
        }
    );
    data
}
// cpp: layoutng_grid/grid_layout_utils.cc:129-142
pub fn GetTrackBaseline(
    item: &GridItemData,
    layout: &GridLayoutData,
    direction: GridTrackSizingDirection,
) -> LayoutUnit {
    let indices = item.SetIndices(direction);
    if item.BaselineGroup(direction) == BaselineGroup::kMajor {
        layout.MajorBaseline(direction, indices.begin)
    } else {
        layout.MinorBaseline(direction, indices.end.wrapping_sub(1))
    }
}
// cpp: layoutng_grid/grid_layout_utils.cc:144-151
pub fn GetLogicalBaseline(
    fragment: &LogicalBoxFragment,
    font: FontBaseline,
    last: bool,
) -> LayoutUnit {
    if last {
        fragment.BlockSize() - fragment.LastBaselineOrSynthesize(font)
    } else {
        fragment.FirstBaselineOrSynthesize(font)
    }
}
// cpp: layoutng_grid/grid_layout_utils.cc:153-169
pub fn SetTrackBaseline(
    item: &GridItemData,
    direction: GridTrackSizingDirection,
    baseline: LayoutUnit,
    layout: &mut GridLayoutData,
) {
    let indices = item.SetIndices(direction);
    if item.BaselineGroup(direction) == BaselineGroup::kMajor {
        layout.SetMajorBaseline(direction, indices.begin, baseline);
    } else {
        layout.SetMinorBaseline(direction, indices.end.wrapping_sub(1), baseline);
    }
}
// cpp: layoutng_grid/grid_layout_utils.cc:171-190
pub fn GetExtraMarginForBaseline(
    margins: &BoxStrut,
    item: &SubgriddedItemData,
    direction: GridTrackSizingDirection,
    mode: WritingMode,
) -> LayoutUnit {
    let tracks = if direction == kForColumns {
        item.Columns(Some(mode))
    } else {
        item.Rows(Some(mode))
    };
    let indices = item.SetIndices(tracks.Direction());
    let extra = if item.BaselineGroup(direction) == BaselineGroup::kMajor {
        tracks.StartExtraMargin(indices.begin)
    } else {
        tracks.EndExtraMargin(indices.end)
    };
    extra
        + if item.IsLastBaselineSpecified(direction) {
            margins.block_end
        } else {
            margins.block_start
        }
}
// cpp: layoutng_grid/grid_layout_utils.cc:192-208
pub fn StoreItemBaseline(
    fragment: &LogicalBoxFragment,
    direction: GridTrackSizingDirection,
    font: FontBaseline,
    extra: LayoutUnit,
    layout: &mut GridLayoutData,
    item: &mut GridItemData,
) {
    item.SetAlignmentFallback(direction, fragment.FirstBaseline().is_none());
    let baseline = GetLogicalBaseline(fragment, font, item.IsLastBaselineSpecified(direction));
    SetTrackBaseline(item, direction, extra + baseline, layout);
}
// cpp: layoutng_grid/grid_layout_utils.cc:210-235
pub fn MeasureAndStoreItemBaseline(
    result: &LayoutResult,
    item: &mut GridItemData,
    subgridded: &SubgriddedItemData,
    space: &ConstraintSpace,
    direction: GridTrackSizingDirection,
    font: FontBaseline,
    mode: WritingMode,
    layout: &mut GridLayoutData,
) {
    let physical = foundation::To::<PhysicalBoxFragment>(result.GetPhysicalFragment());
    let fragment = LogicalBoxFragment::new(item.BaselineWritingDirection(direction), unsafe {
        &*physical
    });
    item.SetAlignmentFallback(direction, fragment.FirstBaseline().is_none());
    if !item.IsBaselineAligned(direction) {
        return;
    }
    let margins = ComputeMarginsForDirection(
        space,
        item.node.Style(),
        item.BaselineWritingDirection(direction),
    );
    let extra = GetExtraMarginForBaseline(&margins, subgridded, direction, mode);
    StoreItemBaseline(&fragment, direction, font, extra, layout, item);
}
// cpp: layoutng_grid/grid_layout_utils.cc:237-264
pub fn ComputeBaselineOffset(
    item: &GridItemData,
    layout: &GridLayoutData,
    baseline_fragment: &LogicalBoxFragment,
    fragment: &LogicalBoxFragment,
    font: FontBaseline,
    direction: GridTrackSizingDirection,
    available: LayoutUnit,
) -> LayoutUnit {
    if !item.IsBaselineAligned(direction) {
        return LayoutUnit::default();
    }
    let delta = GetTrackBaseline(item, layout, direction)
        - GetLogicalBaseline(
            baseline_fragment,
            font,
            item.IsLastBaselineSpecified(direction),
        );
    if item.BaselineGroup(direction) == BaselineGroup::kMajor {
        return delta;
    }
    available
        - delta
        - if direction == kForColumns {
            fragment.InlineSize()
        } else {
            fragment.BlockSize()
        }
}
// cpp: layoutng_grid/grid_layout_utils.cc:266-293
pub fn LayoutGridItemForMeasure(
    item: &GridItemData,
    space: &ConstraintSpace,
    constraint: SizingConstraint,
    after_layout: bool,
) -> *const LayoutResult {
    let _disable = if !unsafe { &*item.node.GetLayoutBox() }.NeedsLayout()
        && (after_layout
            || constraint != SizingConstraint::kLayout
            || item.is_subgridded_to_parent_grid)
    {
        Some(DisableLayoutSideEffectsScope::new())
    } else {
        None
    };
    item.node
        .Layout(space, std::ptr::null(), std::ptr::null(), std::ptr::null())
}
// cpp: layoutng_grid/grid_layout_utils.cc:295-341
pub fn ComputeAvailableSizes(
    bsp: &BoxStrut,
    node: &BlockNode,
    space: &ConstraintSpace,
    builder: &BoxFragmentBuilder,
    available: &mut LogicalSize,
    min: &mut LogicalSize,
    max: &mut LogicalSize,
) {
    if available.inline_size == kIndefiniteSize {
        let sum = bsp.InlineSum();
        let sizes = ComputeMinMaxInlineSizes(
            space,
            node,
            builder.BorderPadding(),
            None,
            &|_| {
                MinMaxSizesResult::new(
                    MinMaxSizes {
                        min_size: kIndefiniteSize,
                        max_size: kIndefiniteSize,
                    },
                    false,
                )
            },
            TransferredSizesMode::kNormal,
            FitContentMode::kNormal,
            kIndefiniteSize,
        );
        min.inline_size = (sizes.min_size - sum).ClampNegativeToZero();
        max.inline_size = if sizes.max_size == LayoutUnit::Max() {
            sizes.max_size
        } else {
            (sizes.max_size - sum).ClampNegativeToZero()
        };
    }
    if available.block_size == kIndefiniteSize {
        let sum = bsp.BlockSum();
        let sizes =
            ComputeInitialMinMaxBlockSizes(space, node, builder.BorderPadding(), kIndefiniteSize);
        min.block_size = (sizes.min_size - sum).ClampNegativeToZero();
        max.block_size = if sizes.max_size == LayoutUnit::Max() {
            sizes.max_size
        } else {
            (sizes.max_size - sum).ClampNegativeToZero()
        };
    }
}
// cpp: layoutng_grid/grid_layout_utils.cc:343-460
pub fn CalculateAutomaticRepetitions(
    tracks: &GridTrackList,
    gutter: LayoutUnit,
    mut available: LayoutUnit,
    min: LayoutUnit,
    mut max: LayoutUnit,
    intrinsic: Option<&HashMap<GridTrackSize, LayoutUnit, GridTrackSizeHashTraits>>,
) -> u32 {
    debug_assert!(tracks.HasAutoRepeater());
    if available == kIndefiniteSize {
        available = min;
    } else {
        max = available;
    }
    let mut auto_size = LayoutUnit::default();
    let mut non_auto = LayoutUnit::default();
    for repeat in 0..tracks.RepeaterCount() {
        let auto = matches!(
            tracks.RepeatType(repeat),
            GridTrackRepeatType::kAutoFill | GridTrackRepeatType::kAutoFit
        );
        let mut repeat_size = LayoutUnit::default();
        for i in 0..tracks.RepeatSize(repeat) {
            let size = tracks.RepeatTrackSize(repeat, i);
            let is_intrinsic = size.IsTrackDefinitionIntrinsic();
            if is_intrinsic && intrinsic.is_none() {
                return 0;
            }
            let min = size
                .HasFixedMinTrackBreadth()
                .then(|| MinimumValueForLength(size.MinTrackBreadth(), available));
            let max = size
                .HasFixedMaxTrackBreadth()
                .then(|| MinimumValueForLength(size.MaxTrackBreadth(), available));
            let mut contribution = if is_intrinsic {
                *intrinsic
                    .unwrap()
                    .get(size)
                    .expect("intrinsic repeat track size is absent")
            } else {
                match (min, max) {
                    (Some(a), Some(b)) => a.max(b),
                    (Some(a), None) | (None, Some(a)) => a,
                    _ => LayoutUnit::default(),
                }
            };
            if auto {
                contribution = contribution.max(LayoutUnit::from_signed(1));
            }
            repeat_size += contribution + gutter;
        }
        if !auto {
            non_auto += repeat_size * tracks.RepeatCount(repeat, 0);
        } else {
            debug_assert_eq!(auto_size, LayoutUnit::default());
            auto_size = repeat_size;
        }
    }
    debug_assert!(auto_size > LayoutUnit::default());
    non_auto -= gutter;
    let count = if max != LayoutUnit::Max() {
        ((max - non_auto) / auto_size).Floor()
    } else {
        ((available - non_auto) / auto_size).Ceil()
    };
    if count <= 0 {
        1
    } else {
        count as u32
    }
}

// cpp: layoutng_grid/grid_layout_utils.cc:467-494
fn ComputeTrackSizesInRange(
    tracks: &GridLayoutTrackCollection,
    begin: u32,
    count: u32,
) -> Vec<(i32, i32)> {
    let mut sizes = Vec::with_capacity(count as usize);
    for i in begin..begin + count {
        let size = tracks.GetSetOffset(i + 1) - tracks.GetSetOffset(i);
        let track_count = tracks.GetSetTrackCount(i);
        debug_assert!(size >= LayoutUnit::default());
        let size = (size - tracks.GutterSize() * track_count).ClampNegativeToZero();
        debug_assert!(track_count > 0);
        sizes.push((
            size.RawValue() / track_count as i32,
            size.RawValue() % track_count as i32,
        ));
    }
    sizes
}
// cpp: layoutng_grid/grid_layout_utils.cc:498-531
fn ComputeTrackOffsetInRange(
    tracks: &GridLayoutTrackCollection,
    begin: u32,
    count: u32,
    offset: u32,
) -> LayoutUnit {
    if count == 0 || offset == 0 {
        return LayoutUnit::default();
    }
    let sizes = ComputeTrackSizesInRange(tracks, begin, count);
    let floor = offset / count;
    let remaining = offset % count;
    let mut result = tracks.GutterSize() * offset;
    for (i, (quot, rem)) in sizes.into_iter().enumerate() {
        let set_count = floor + u32::from((remaining as usize) > i);
        result += LayoutUnit::FromRawValue(
            (set_count as i32).min(rem) + (set_count.wrapping_mul(quot as u32)) as i32,
        );
    }
    result
}
// cpp: layoutng_grid/grid_layout_utils.cc:534-569
fn TrackOffset<const SNAP: bool>(
    tracks: &GridLayoutTrackCollection,
    range: u32,
    offset: u32,
) -> LayoutUnit {
    let begin = tracks.RangeBeginSetIndex(range);
    let track_count = tracks.RangeTrackCount(range);
    let set_count = tracks.RangeSetCount(range);
    let mut result;
    if offset == track_count {
        debug_assert!(SNAP);
        result = tracks.GetSetOffset(begin + set_count);
    } else {
        debug_assert!(offset != 0 || !SNAP);
        debug_assert!(offset < track_count);
        result = tracks.GetSetOffset(begin)
            + ComputeTrackOffsetInRange(tracks, begin, set_count, offset);
    }
    if SNAP && (set_count != 0 || range != 0) {
        result -= tracks.GutterSize();
    }
    result
}
// cpp: layoutng_grid/grid_layout_utils.h:164-171
// cpp: layoutng_grid/grid_layout_utils.cc:573-599
pub fn TrackStartOffset(tracks: &GridLayoutTrackCollection, range: u32, offset: u32) -> LayoutUnit {
    if tracks.RangeCount() == 0 {
        debug_assert_eq!(range, 0);
        debug_assert_eq!(offset, 0);
        return tracks.GetSetOffset(0);
    }
    let count = tracks.RangeTrackCount(range);
    if offset == count && range == tracks.RangeCount() - 1 {
        return TrackOffset::<true>(tracks, range, offset);
    }
    debug_assert!(offset < count);
    TrackOffset::<false>(tracks, range, offset)
}
// cpp: layoutng_grid/grid_layout_utils.h:173-176
// cpp: layoutng_grid/grid_layout_utils.cc:601-622
pub fn TrackEndOffset(tracks: &GridLayoutTrackCollection, range: u32, offset: u32) -> LayoutUnit {
    if tracks.RangeCount() == 0 {
        debug_assert_eq!(range, 0);
        debug_assert_eq!(offset, 0);
        return tracks.GetSetOffset(0);
    }
    if offset == 0 && range == 0 {
        return TrackOffset::<false>(tracks, range, offset);
    }
    debug_assert!(offset > 0);
    TrackOffset::<true>(tracks, range, offset)
}

// cpp: layoutng_grid/grid_layout_utils.cc:624-662
pub fn AlignmentOffsetForOutOfFlow(
    inline: AxisEdge,
    block: AxisEdge,
    size: LogicalSize,
    pos: &mut LogicalStaticPosition,
) {
    match inline {
        AxisEdge::kStart | AxisEdge::kFirstBaseline => pos.inline_edge = InlineEdge::kInlineStart,
        AxisEdge::kCenter => {
            pos.inline_edge = InlineEdge::kInlineCenter;
            pos.offset.inline_offset += size.inline_size / 2;
        }
        AxisEdge::kEnd | AxisEdge::kLastBaseline => {
            pos.inline_edge = InlineEdge::kInlineEnd;
            pos.offset.inline_offset += size.inline_size;
        }
    }
    match block {
        AxisEdge::kStart | AxisEdge::kFirstBaseline => pos.block_edge = BlockEdge::kBlockStart,
        AxisEdge::kCenter => {
            pos.block_edge = BlockEdge::kBlockCenter;
            pos.offset.block_offset += size.block_size / 2;
        }
        AxisEdge::kEnd | AxisEdge::kLastBaseline => {
            pos.block_edge = BlockEdge::kBlockEnd;
            pos.offset.block_offset += size.block_size;
        }
    }
}
// cpp: layoutng_grid/grid_layout_utils.cc:664-759
pub fn CalculateIntrinsicMinimumContribution(
    parallel: bool,
    special: bool,
    min_content: &mut dyn FnMut() -> LayoutUnit,
    max_content: &mut dyn FnMut() -> LayoutUnit,
    subgrid_minmax: &mut dyn FnMut() -> MinMaxSizesResult,
    space: &ConstraintSpace,
    item: *const GridItemData,
    maybe_clamp: &mut bool,
) -> LayoutUnit {
    let (node, is_subgrid) = {
        let item = unsafe { &*item };
        (item.node.clone(), item.IsSubgrid())
    };
    let style = node.Style();
    *maybe_clamp = false;
    let main = if parallel {
        style.LogicalWidth()
    } else {
        style.LogicalHeight()
    };
    let min = if parallel {
        style.LogicalMinWidth()
    } else {
        style.LogicalMinHeight()
    };
    match main.GetType() {
        LengthType::kAuto
        | LengthType::kFitContent
        | LengthType::kStretch
        | LengthType::kPercent
        | LengthType::kCalculated => {
            let bp = ComputeBorders(space, &node) + ComputePadding(space, style);
            let scroll = if parallel {
                style.IsOverflowValueScrollableInline()
            } else {
                style.IsOverflowValueScrollableBlock()
            };
            if !min.HasAuto() || scroll || special {
                if parallel {
                    let callback = std::cell::RefCell::new(subgrid_minmax);
                    let sizes = |kind| {
                        if is_subgrid {
                            (callback.borrow_mut())()
                        } else {
                            node.ComputeMinMaxSizes(
                                style.GetWritingMode(),
                                kind,
                                space,
                                MinMaxSizesFloatInput::default(),
                            )
                        }
                    };
                    return ResolveMinInlineLength(
                        space,
                        style,
                        &bp,
                        &sizes,
                        min,
                        None,
                        kIndefiniteSize,
                        FitContentMode::kNormal,
                    );
                }
                return ResolveInitialMinBlockLength(space, style, &bp, min, kIndefiniteSize);
            }
            *maybe_clamp = true;
            min_content()
        }
        LengthType::kMinContent | LengthType::kMaxContent | LengthType::kFixed => {
            if main.IsMaxContent() {
                max_content()
            } else {
                min_content()
            }
        }
        LengthType::kMinIntrinsic
        | LengthType::kFlex
        | LengthType::kNone
        | LengthType::kContent
        | LengthType::kOverlapJoin => unreachable!("invalid grid contribution length"),
    }
}
// cpp: layoutng_grid/grid_layout_utils.cc:761-774
pub fn ClampIntrinsicMinSize(content: LayoutUnit, min: LayoutUnit, max: LayoutUnit) -> LayoutUnit {
    assert_ne!(max, kIndefiniteSize);
    debug_assert!(content >= min);
    content.min(max.max(min))
}
// cpp: layoutng_grid/grid_layout_utils.cc:776-795
pub fn SubgriddedAreaInParent(item: &SubgriddedItemData) -> GridArea {
    if !item.IsSubgrid() {
        return GridArea::default();
    }
    let mut area = item.resolved_position.clone();
    if !item.has_subgridded_columns {
        area.columns = GridSpan::IndefiniteGridSpan(1);
    }
    if !item.has_subgridded_rows {
        area.rows = GridSpan::IndefiniteGridSpan(1);
    }
    if !item.is_parallel_with_root_grid {
        std::mem::swap(&mut area.columns, &mut area.rows);
    }
    area
}
// cpp: layoutng_grid/grid_layout_utils.cc:803-854
pub fn CalculateInitialFragmentGeometryForSubgrid(
    item: &GridItemData,
    space: &ConstraintSpace,
    subtree: &GridSizingSubtree,
) -> FragmentGeometry {
    debug_assert!(item.IsSubgrid());
    let node = &item.node;
    let standalone = if item.is_parallel_with_root_grid {
        !item.has_subgridded_columns
    } else {
        !item.has_subgridded_rows
    };
    if node.IsGrid() && standalone && subtree.IsPresent() {
        let grid = GridNode::new(node.GetLayoutBox());
        return CalculateInitialFragmentGeometryWithMinMaxSizes(
            space,
            &grid,
            std::ptr::null(),
            &|_| grid.ComputeSubgridMinMaxSizes(subtree, space),
            false,
        );
    }
    let needs = std::cell::Cell::new(false);
    let geometry = CalculateInitialFragmentGeometryWithMinMaxSizes(
        space,
        node,
        std::ptr::null(),
        &|_| {
            needs.set(true);
            MinMaxSizesResult::default()
        },
        false,
    );
    if needs.get() {
        return CalculateInitialFragmentGeometry(space, node, std::ptr::null(), true);
    }
    geometry
}
// cpp: layoutng_grid/grid_layout_utils.cc:856-886
pub fn CreateSubgridTrackCollection(
    item: &SubgriddedItemData,
    style: &ComputedStyle,
    space: &ConstraintSpace,
    bsp: &BoxStrut,
    available: LogicalSize,
    direction: GridTrackSizingDirection,
) -> *mut GridLayoutTrackCollection {
    debug_assert!(item.IsSubgrid());
    let columns = if item.is_parallel_with_root_grid {
        direction == kForColumns
    } else {
        direction == kForRows
    };
    let parent = if columns {
        item.Columns(None)
    } else {
        item.Rows(None)
    };
    let ranges = if columns {
        &item.column_range_indices
    } else {
        &item.row_range_indices
    };
    parent.CreateSubgridTrackCollection(
        ranges.begin,
        ranges.end,
        GridTrackSizingAlgorithm::CalculateGutterSizeWithParent(
            style,
            &available,
            direction,
            parent.GutterSize(),
        ),
        &ComputeMarginsForSelf(space, style),
        bsp,
        direction,
        if columns {
            item.is_opposite_direction_in_root_grid_columns
        } else {
            item.is_opposite_direction_in_root_grid_rows
        },
        item.is_auto_placed,
    )
}
// cpp: layoutng_grid/grid_layout_utils.cc:888-919
pub fn CreateSubgridBaselines(
    item: &SubgriddedItemData,
    style: &ComputedStyle,
    space: &ConstraintSpace,
    bsp: &BoxStrut,
    available: LogicalSize,
    direction: GridTrackSizingDirection,
    baselines: &GridTrackBaselines,
) -> *mut GridTrackBaselines {
    debug_assert!(item.IsSubgrid());
    let columns = if item.is_parallel_with_root_grid {
        direction == kForColumns
    } else {
        direction == kForRows
    };
    let parent = if columns {
        item.Columns(None)
    } else {
        item.Rows(None)
    };
    let ranges = if columns {
        &item.column_range_indices
    } else {
        &item.row_range_indices
    };
    parent.CreateSubgridBaselines(
        ranges.begin,
        ranges.end,
        GridTrackSizingAlgorithm::CalculateGutterSizeWithParent(
            style,
            &available,
            direction,
            parent.GutterSize(),
        ),
        &ComputeMarginsForSelf(space, style),
        bsp,
        direction,
        if columns {
            item.is_opposite_direction_in_root_grid_columns
        } else {
            item.is_opposite_direction_in_root_grid_rows
        },
        baselines,
    )
}
// cpp: layoutng_grid/grid_layout_utils.cc:921-941
pub fn InitializeTrackCollection(
    item: &SubgriddedItemData,
    style: &ComputedStyle,
    space: &ConstraintSpace,
    bsp: &BoxStrut,
    available: LogicalSize,
    direction: GridTrackSizingDirection,
    layout: &mut GridLayoutData,
) {
    if layout.HasSubgriddedAxis(direction) {
        debug_assert!(item.IsSubgrid());
        layout.SetTrackCollection(CreateSubgridTrackCollection(
            item, style, space, bsp, available, direction,
        ));
        return;
    }
    unsafe { &mut *layout.SizingCollection(direction) }.BuildSets(style, &available);
}
// cpp: layoutng_grid/grid_layout_utils.cc:943-950
pub fn HasBlockSizeDependentGridItem(items: &GridItems) -> bool {
    let range = items.IncludeSubgriddedItemsConst();
    let mut it = range.begin();
    let end = range.end();
    while it.NotEqual(&end) {
        if unsafe { &*it.Get() }.is_sizing_dependent_on_block_size {
            return true;
        }
        it.Advance();
    }
    false
}
// cpp: layoutng_grid/grid_layout_utils.cc:952-992
pub fn ValidateMinMaxSizesCache(
    node: &BlockNode,
    subtree: &GridSizingSubtree,
    direction: GridTrackSizingDirection,
) -> bool {
    debug_assert!(subtree.HasValidRootFor(node));
    let mut invalidate = false;
    let mut child = subtree.FirstChild();
    if child.IsPresent() {
        let items = unsafe { &*subtree.GetGridItems() };
        let mut it = items.begin_const();
        let end = items.end_const();
        while it.NotEqual(&end) {
            let item = unsafe { &*it.Get() };
            it.Advance();
            if !item.IsSubgrid() {
                continue;
            }
            debug_assert!(child.IsPresent());
            invalidate |= ValidateMinMaxSizesCache(
                &item.node,
                &child,
                item.RelativeDirectionInSubgrid(direction),
            );
            child = child.NextSibling();
        }
    }
    let layout = unsafe { &*subtree.LayoutData() };
    if layout.IsSubgridWithStandaloneAxis(direction) {
        assert!(node.IsGrid());
        let grid = GridNode::new(node.GetLayoutBox());
        if !invalidate {
            invalidate = grid.ShouldInvalidateSubgridMinMaxSizesCacheFor(layout);
        }
        if invalidate {
            grid.InvalidateSubgridMinMaxSizesCache();
        }
    }
    invalidate
}
// cpp: layoutng_grid/grid_layout_utils.cc:994-1023
pub fn NeedsAdditionalLayoutPass(
    style: &ComputedStyle,
    space: &ConstraintSpace,
    node: &BlockNode,
    bp: &BoxStrut,
    tracks: &GridSizingTrackCollection,
    inline: LayoutUnit,
) -> bool {
    let mut needed = style.RowGap().as_ref().is_some_and(|gap| gap.HasPercent())
        || tracks.base.IsDependentOnAvailableSize();
    if space.IsInitialBlockSizeIndefinite() {
        needed |=
            ComputeBlockSizeForFragment(space, node, bp, kIndefiniteSize, inline, kIndefiniteSize)
                != kIndefiniteSize;
    }
    needed
}
// cpp: layoutng_grid/grid_layout_utils.cc:1025-1037
pub fn GetSynthesizedLogicalBaseline(
    item: &GridItemData,
    size: LayoutUnit,
    direction: GridTrackSizingDirection,
) -> LayoutUnit {
    let baseline = LogicalBoxFragment::SynthesizedBaseline(
        item.parent_grid_font_baseline,
        item.BaselineWritingDirection(direction).IsFlippedLines(),
        size,
    );
    if item.IsLastBaselineSpecified(direction) {
        size - baseline
    } else {
        baseline
    }
}
// cpp: layoutng_grid/grid_layout_utils.cc:1039-1061
pub fn LargestAutoPlacedSubgridContribution(
    start: LayoutUnit,
    end: LayoutUnit,
    delta: LayoutUnit,
    span: u32,
) -> LayoutUnit {
    if span == 1 {
        return start + end;
    }
    let half = delta / 2i32;
    if span == 2 {
        return (start + half).max(end + half);
    }
    (start + half).max(delta).max(end + half)
}
// cpp: layoutng_grid/grid_layout_utils.cc:1063-1160
pub fn AccommodateSubgridExtraMargins(
    subtree: &GridSizingSubtree,
    tracks: &mut GridSizingTrackCollection,
    direction: GridTrackSizingDirection,
) {
    let items = unsafe { &*subtree.GetGridItems() };
    let range = items.IncludeSubgriddedItemsConst();
    let mut it = range.begin();
    let end = range.end();
    while it.NotEqual(&end) {
        let item = unsafe { &*it.Get() };
        it.Advance();
        if !item.MustConsiderGridItemsForSizing(direction) {
            continue;
        }
        if !item.is_auto_placed && !item.IsSpanningIntrinsicTrack(direction) {
            continue;
        }
        debug_assert!(item.IsSubgrid());
        let columns = item.RelativeDirectionInSubgrid(direction) == kForColumns;
        let layout = unsafe { &*subtree.SubgridSizingSubtree(item).LayoutData() };
        let sub = unsafe {
            &*if columns {
                layout.Columns()
            } else {
                layout.Rows()
            }
        };
        let mut start = sub.StartExtraMarginDefault();
        let mut finish = sub.EndExtraMarginDefault();
        if item.IsOppositeDirectionInRootGrid(direction) {
            std::mem::swap(&mut start, &mut finish);
        }
        if item.is_auto_placed {
            let span = item.Span(direction);
            let count = if span.IsTranslatedDefinite() {
                span.IntegerSpan()
            } else {
                span.IndefiniteSpanSize()
            };
            let largest = LargestAutoPlacedSubgridContribution(
                start,
                finish,
                sub.AccumulatedGutterSizeDelta(),
                count,
            );
            if largest == LayoutUnit::default() {
                continue;
            }
            for i in 0..tracks.base.GetSetCount() {
                let contribution = largest * tracks.base.GetSetTrackCount(i);
                AccommodateExtraMargin(tracks, contribution, i);
            }
            continue;
        }
        let indices = item.SetIndices(direction);
        let span = indices.end - indices.begin;
        let last = indices.begin + span - 1;
        if indices.begin < last {
            AccommodateExtraMargin(tracks, start, indices.begin);
            AccommodateExtraMargin(tracks, finish, last);
        } else {
            AccommodateExtraMargin(tracks, start + finish, indices.begin);
        }
    }
}
// cpp: layoutng_grid/grid_layout_utils.cc:1071-1078
fn AccommodateExtraMargin(tracks: &mut GridSizingTrackCollection, extra: LayoutUnit, index: u32) {
    let set = unsafe { &mut *tracks.GetSetAt(index) };
    if set.track_size.HasIntrinsicMinTrackBreadth() && set.BaseSize() < extra {
        set.IncreaseBaseSize(extra);
    }
}

// cpp: layoutng_grid/grid_layout_utils.cc:1159-1184
// Source explicit template instantiations are monomorphized from the shared
// builder functions when GridLayoutAlgorithm calls them; no duplicate body.
