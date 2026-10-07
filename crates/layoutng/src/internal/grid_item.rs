#![allow(non_snake_case)]

use font_engine::FontBaseline;
use foundation::{
    HeapVector, LayoutUnit, MakeGarbageCollected, Member, TextDirection, Visitor,
    WritingDirectionMode, WritingMode,
};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::grid_area::{GridArea, GridSpan};
use layoutng_style::style::grid_enums::GridTrackSizingDirection::{self, kForColumns, kForRows};

use super::baseline_utils::BaselineGroup;
use super::block_node::BlockNode;
use super::constraint_space::AutoSizeBehavior;
use super::grid_track_collection::{
    GridLayoutTrackCollection, GridSizingTrackCollection, PropertyId, SetIterator,
    TrackSpanProperties,
};
use super::min_max_sizes::MinMaxSizes;

// GridPlacementData and the non-inline GridItemData bodies are owned by
// //src/layoutng_grid. Keep their signatures while translating this header.
#[repr(C)]
pub struct GridPlacementData {
    _opaque: [u8; 0],
}

unsafe extern "Rust" {
    fn GridItemDataNewProvider(
        item_node: BlockNode,
        parent_grid_style: &ComputedStyle,
        root_grid_style: &ComputedStyle,
        parent_must_consider_columns: bool,
        parent_must_consider_rows: bool,
    ) -> GridItemData;
    fn GridItemDataSetAlignmentFallbackProvider(
        item: &mut GridItemData,
        track_direction: GridTrackSizingDirection,
        has_synthesized_baseline: bool,
    );
    fn GridItemDataUpdateSpanProvider(
        item: &mut GridItemData,
        span: &GridSpan,
        track_direction: GridTrackSizingDirection,
        start_offset: u32,
        track_collection: &GridLayoutTrackCollection,
    );
    fn GridItemDataComputeSetIndicesProvider(
        item: &mut GridItemData,
        track_collection: &GridLayoutTrackCollection,
    );
    fn GridItemDataComputeOutOfFlowItemPlacementProvider(
        item: &mut GridItemData,
        track_collection: &GridLayoutTrackCollection,
        placement_data: &GridPlacementData,
        grid_style: &ComputedStyle,
    );
    fn GridItemDataCalculateAvailableSizeProvider(
        item: &GridItemData,
        track_collection: &GridLayoutTrackCollection,
        start_offset: *mut LayoutUnit,
    ) -> LayoutUnit;
    fn GridItemsAppendProvider(items: &mut GridItems, other: *mut GridItems);
    fn GridItemsSortByOrderPropertyProvider(items: &mut GridItems);
}

// cpp: layoutng/internal/grid_item.h:20-20
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxisEdge {
    kStart,
    kCenter,
    kEnd,
    kFirstBaseline,
    kLastBaseline,
}

// Blink's wtf_size_t is an unsigned 32-bit index in this standalone input.
// cpp: layoutng/internal/grid_item.h:22-25
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridItemIndices {
    pub begin: u32,
    pub end: u32,
}

impl Default for GridItemIndices {
    fn default() -> Self {
        Self {
            begin: u32::MAX,
            end: u32::MAX,
        }
    }
}

// cpp: layoutng/internal/grid_item.h:27-30
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OutOfFlowItemPlacement {
    pub range_index: GridItemIndices,
    pub offset_in_range: GridItemIndices,
}

// cpp: layoutng/internal/grid_item.h:325-417
#[derive(Clone, Debug, Default)]
pub struct GridItemDataVirtualItemContributions {
    pub min_max_contribution: MinMaxSizes,
    pub intrinsic_min_assuming_track_placement: LayoutUnit,
    pub intrinsic_min_ignoring_track_placement: LayoutUnit,
    pub intrinsic_min_ignoring_track_placement_unclamped: LayoutUnit,
    pub min_clamp_size: LayoutUnit,
    pub group_shared_baseline: LayoutUnit,
}

impl GridItemDataVirtualItemContributions {
    // cpp: layoutng/internal/grid_item.h:329-362
    pub fn Trace(&self, _visitor: &mut Visitor) {}

    pub fn EncompassContributionSize(&mut self, sizes: MinMaxSizes) {
        self.min_max_contribution.Encompass(&sizes);
    }

    pub fn EncompassIntrinsicMinIgnoringTrackPlacement(&mut self, size: LayoutUnit) {
        self.intrinsic_min_ignoring_track_placement =
            self.intrinsic_min_ignoring_track_placement.max(size);
    }

    pub fn EncompassIntrinsicMinIgnoringTrackPlacementUnclamped(&mut self, size: LayoutUnit) {
        self.intrinsic_min_ignoring_track_placement_unclamped = self
            .intrinsic_min_ignoring_track_placement_unclamped
            .max(size);
    }

    pub fn EncompassIntrinsicMinAssumingTrackPlacement(&mut self, size: LayoutUnit) {
        self.intrinsic_min_assuming_track_placement =
            self.intrinsic_min_assuming_track_placement.max(size);
    }

    pub fn EncompassMinClampSize(&mut self, min_clamp: LayoutUnit) {
        self.min_clamp_size = self.min_clamp_size.max(min_clamp);
    }

    pub fn SetSharedBaseline(&mut self, baseline: LayoutUnit) {
        self.group_shared_baseline = baseline;
    }
}

// cpp: layoutng/internal/grid_item.h:32-42,271-319,417-418
pub struct GridItemData {
    pub node: BlockNode,
    pub resolved_position: GridArea,
    pub has_subgridded_columns: bool,
    pub has_subgridded_rows: bool,
    pub is_auto_placed: bool,
    pub is_considered_for_column_sizing: bool,
    pub is_considered_for_row_sizing: bool,
    pub is_opposite_direction_in_root_grid_columns: bool,
    pub is_opposite_direction_in_root_grid_rows: bool,
    pub is_overflow_safe_for_columns: bool,
    pub is_overflow_safe_for_rows: bool,
    pub is_parallel_with_root_grid: bool,
    pub is_sizing_dependent_on_block_size: bool,
    pub is_subgridded_to_parent_grid: bool,
    pub must_consider_grid_items_for_column_sizing: bool,
    pub must_consider_grid_items_for_row_sizing: bool,
    pub parent_grid_font_baseline: FontBaseline,
    pub column_alignment: AxisEdge,
    pub row_alignment: AxisEdge,
    pub column_fallback_alignment: Option<AxisEdge>,
    pub row_fallback_alignment: Option<AxisEdge>,
    pub column_auto_behavior: AutoSizeBehavior,
    pub row_auto_behavior: AutoSizeBehavior,
    pub column_baseline_group: BaselineGroup,
    pub row_baseline_group: BaselineGroup,
    pub column_baseline_writing_mode: WritingMode,
    pub row_baseline_writing_mode: WritingMode,
    pub column_span_properties: TrackSpanProperties,
    pub row_span_properties: TrackSpanProperties,
    pub column_set_indices: GridItemIndices,
    pub row_set_indices: GridItemIndices,
    pub column_range_indices: GridItemIndices,
    pub row_range_indices: GridItemIndices,
    pub column_placement: OutOfFlowItemPlacement,
    pub row_placement: OutOfFlowItemPlacement,
    pub contribution_sizes: Member<GridItemDataVirtualItemContributions>,
}

impl Default for GridItemData {
    // C++ leaves several enum members uninitialized in the default constructor;
    // Rust gives them safe values until the grid owner populates them.
    // cpp: layoutng/internal/grid_item.h:33-33,274-287
    fn default() -> Self {
        Self {
            node: BlockNode::null(),
            resolved_position: GridArea::default(),
            has_subgridded_columns: false,
            has_subgridded_rows: false,
            is_auto_placed: false,
            is_considered_for_column_sizing: true,
            is_considered_for_row_sizing: true,
            is_opposite_direction_in_root_grid_columns: false,
            is_opposite_direction_in_root_grid_rows: false,
            is_overflow_safe_for_columns: false,
            is_overflow_safe_for_rows: false,
            is_parallel_with_root_grid: false,
            is_sizing_dependent_on_block_size: false,
            is_subgridded_to_parent_grid: false,
            must_consider_grid_items_for_column_sizing: false,
            must_consider_grid_items_for_row_sizing: false,
            parent_grid_font_baseline: FontBaseline::kAlphabeticBaseline,
            column_alignment: AxisEdge::kStart,
            row_alignment: AxisEdge::kStart,
            column_fallback_alignment: None,
            row_fallback_alignment: None,
            column_auto_behavior: AutoSizeBehavior::kFitContent,
            row_auto_behavior: AutoSizeBehavior::kFitContent,
            column_baseline_group: BaselineGroup::kMajor,
            row_baseline_group: BaselineGroup::kMajor,
            column_baseline_writing_mode: WritingMode::kHorizontalTb,
            row_baseline_writing_mode: WritingMode::kHorizontalTb,
            column_span_properties: TrackSpanProperties::default(),
            row_span_properties: TrackSpanProperties::default(),
            column_set_indices: GridItemIndices::default(),
            row_set_indices: GridItemIndices::default(),
            column_range_indices: GridItemIndices::default(),
            row_range_indices: GridItemIndices::default(),
            column_placement: OutOfFlowItemPlacement::default(),
            row_placement: OutOfFlowItemPlacement::default(),
            contribution_sizes: Member::default(),
        }
    }
}

impl Clone for GridItemData {
    // cpp: layoutng/internal/grid_item.h:34-35
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            resolved_position: self.resolved_position,
            has_subgridded_columns: self.has_subgridded_columns,
            has_subgridded_rows: self.has_subgridded_rows,
            is_auto_placed: self.is_auto_placed,
            is_considered_for_column_sizing: self.is_considered_for_column_sizing,
            is_considered_for_row_sizing: self.is_considered_for_row_sizing,
            is_opposite_direction_in_root_grid_columns: self
                .is_opposite_direction_in_root_grid_columns,
            is_opposite_direction_in_root_grid_rows: self.is_opposite_direction_in_root_grid_rows,
            is_overflow_safe_for_columns: self.is_overflow_safe_for_columns,
            is_overflow_safe_for_rows: self.is_overflow_safe_for_rows,
            is_parallel_with_root_grid: self.is_parallel_with_root_grid,
            is_sizing_dependent_on_block_size: self.is_sizing_dependent_on_block_size,
            is_subgridded_to_parent_grid: self.is_subgridded_to_parent_grid,
            must_consider_grid_items_for_column_sizing: self
                .must_consider_grid_items_for_column_sizing,
            must_consider_grid_items_for_row_sizing: self.must_consider_grid_items_for_row_sizing,
            parent_grid_font_baseline: self.parent_grid_font_baseline,
            column_alignment: self.column_alignment,
            row_alignment: self.row_alignment,
            column_fallback_alignment: self.column_fallback_alignment,
            row_fallback_alignment: self.row_fallback_alignment,
            column_auto_behavior: self.column_auto_behavior,
            row_auto_behavior: self.row_auto_behavior,
            column_baseline_group: self.column_baseline_group,
            row_baseline_group: self.row_baseline_group,
            column_baseline_writing_mode: self.column_baseline_writing_mode,
            row_baseline_writing_mode: self.row_baseline_writing_mode,
            column_span_properties: self.column_span_properties,
            row_span_properties: self.row_span_properties,
            column_set_indices: self.column_set_indices,
            row_set_indices: self.row_set_indices,
            column_range_indices: self.column_range_indices,
            row_range_indices: self.row_range_indices,
            column_placement: self.column_placement,
            row_placement: self.row_placement,
            contribution_sizes: Member::from_ptr(self.contribution_sizes.Get()),
        }
    }
}

impl GridItemData {
    // cpp: layoutng/internal/grid_item.h:37-44
    pub fn new(
        item_node: BlockNode,
        parent_grid_style: &ComputedStyle,
        root_grid_style: &ComputedStyle,
        parent_must_consider_columns: bool,
        parent_must_consider_rows: bool,
    ) -> Self {
        unsafe {
            GridItemDataNewProvider(
                item_node,
                parent_grid_style,
                root_grid_style,
                parent_must_consider_columns,
                parent_must_consider_rows,
            )
        }
    }

    pub fn new_with_parent_style(item_node: BlockNode, parent_style: &ComputedStyle) -> Self {
        Self::new(item_node, parent_style, parent_style, false, false)
    }

    // cpp: layoutng/internal/grid_item.h:46-56
    pub fn SetAlignmentFallback(&mut self, direction: GridTrackSizingDirection, synthesized: bool) {
        unsafe { GridItemDataSetAlignmentFallbackProvider(self, direction, synthesized) }
    }

    pub fn UpdateSpan(
        &mut self,
        span: &GridSpan,
        direction: GridTrackSizingDirection,
        start_offset: u32,
        collection: &GridLayoutTrackCollection,
    ) {
        unsafe { GridItemDataUpdateSpanProvider(self, span, direction, start_offset, collection) }
    }

    // cpp: layoutng/internal/grid_item.h:89-107
    pub fn ComputeSetIndices(&mut self, collection: &GridLayoutTrackCollection) {
        unsafe { GridItemDataComputeSetIndicesProvider(self, collection) }
    }

    pub fn ComputeOutOfFlowItemPlacement(
        &mut self,
        collection: &GridLayoutTrackCollection,
        placement: &GridPlacementData,
        style: &ComputedStyle,
    ) {
        unsafe {
            GridItemDataComputeOutOfFlowItemPlacementProvider(self, collection, placement, style)
        }
    }

    pub fn CalculateAvailableSize(
        &self,
        collection: &GridLayoutTrackCollection,
        start_offset: *mut LayoutUnit,
    ) -> LayoutUnit {
        unsafe { GridItemDataCalculateAvailableSizeProvider(self, collection, start_offset) }
    }

    // cpp: layoutng/internal/grid_item.h:58-87
    pub fn Alignment(&self, direction: GridTrackSizingDirection) -> AxisEdge {
        if direction == kForColumns {
            self.column_fallback_alignment
                .unwrap_or(self.column_alignment)
        } else {
            self.row_fallback_alignment.unwrap_or(self.row_alignment)
        }
    }

    pub fn IsOverflowSafe(&self, direction: GridTrackSizingDirection) -> bool {
        if direction == kForColumns {
            self.column_fallback_alignment.is_some() || self.is_overflow_safe_for_columns
        } else {
            self.row_fallback_alignment.is_some() || self.is_overflow_safe_for_rows
        }
    }

    pub fn IsBaselineAligned(&self, direction: GridTrackSizingDirection) -> bool {
        matches!(
            self.Alignment(direction),
            AxisEdge::kFirstBaseline | AxisEdge::kLastBaseline
        )
    }

    pub fn IsBaselineSpecified(&self, direction: GridTrackSizingDirection) -> bool {
        matches!(
            if direction == kForColumns {
                self.column_alignment
            } else {
                self.row_alignment
            },
            AxisEdge::kFirstBaseline | AxisEdge::kLastBaseline
        )
    }

    pub fn IsLastBaselineSpecified(&self, direction: GridTrackSizingDirection) -> bool {
        (if direction == kForColumns {
            self.column_alignment
        } else {
            self.row_alignment
        }) == AxisEdge::kLastBaseline
    }

    // cpp: layoutng/internal/grid_item.h:109-128
    pub fn BaselineGroup(&self, direction: GridTrackSizingDirection) -> BaselineGroup {
        if direction == kForColumns {
            self.column_baseline_group
        } else {
            self.row_baseline_group
        }
    }

    pub fn BaselineWritingDirection(
        &self,
        direction: GridTrackSizingDirection,
    ) -> WritingDirectionMode {
        let mode = if direction == kForColumns {
            self.column_baseline_writing_mode
        } else {
            self.row_baseline_writing_mode
        };
        WritingDirectionMode::new(mode, TextDirection::kLtr)
    }

    pub fn SetIndices(&self, direction: GridTrackSizingDirection) -> &GridItemIndices {
        if direction == kForColumns {
            &self.column_set_indices
        } else {
            &self.row_set_indices
        }
    }

    // cpp: layoutng/internal/grid_item.h:130-145
    pub fn SetIterator(&self, collection: &mut GridSizingTrackCollection) -> SetIterator {
        let indices = self.SetIndices(collection.base.Direction());
        collection.GetSetIteratorRange(indices.begin, indices.end)
    }

    pub fn RangeIndices(&mut self, direction: GridTrackSizingDirection) -> &mut GridItemIndices {
        if direction == kForColumns {
            &mut self.column_range_indices
        } else {
            &mut self.row_range_indices
        }
    }

    pub fn ResetPlacementIndices(&mut self) {
        self.column_range_indices = GridItemIndices::default();
        self.row_range_indices = GridItemIndices::default();
        self.column_set_indices = GridItemIndices::default();
        self.row_set_indices = GridItemIndices::default();
    }

    // cpp: layoutng/internal/grid_item.h:147-163
    pub fn MaybeTranslateSpan(
        &mut self,
        start_offset: u32,
        direction: GridTrackSizingDirection,
    ) -> &GridSpan {
        self.resolved_position
            .MaybeTranslateSpan(start_offset, direction)
    }

    pub fn Span(&self, direction: GridTrackSizingDirection) -> &GridSpan {
        self.resolved_position.Span(direction)
    }
    pub fn StartLine(&self, direction: GridTrackSizingDirection) -> u32 {
        self.resolved_position.StartLine(direction)
    }
    pub fn EndLine(&self, direction: GridTrackSizingDirection) -> u32 {
        self.resolved_position.EndLine(direction)
    }
    pub fn SpanSize(&self, direction: GridTrackSizingDirection) -> u32 {
        self.resolved_position.SpanSize(direction)
    }

    // cpp: layoutng/internal/grid_item.h:165-195
    pub fn IsSubgrid(&self) -> bool {
        self.has_subgridded_columns || self.has_subgridded_rows
    }
    pub fn IsConsideredForSizing(&self, direction: GridTrackSizingDirection) -> bool {
        if direction == kForColumns {
            self.is_considered_for_column_sizing
        } else {
            self.is_considered_for_row_sizing
        }
    }
    pub fn IsOppositeDirectionInRootGrid(&self, direction: GridTrackSizingDirection) -> bool {
        if direction == kForColumns {
            self.is_opposite_direction_in_root_grid_columns
        } else {
            self.is_opposite_direction_in_root_grid_rows
        }
    }
    pub fn MustCachePlacementIndices(&self, direction: GridTrackSizingDirection) -> bool {
        !self.is_subgridded_to_parent_grid
            || self.IsConsideredForSizing(direction)
            || self.MustConsiderGridItemsForSizing(direction)
    }
    pub fn MustConsiderGridItemsForSizing(&self, direction: GridTrackSizingDirection) -> bool {
        if direction == kForColumns {
            self.must_consider_grid_items_for_column_sizing
        } else {
            self.must_consider_grid_items_for_row_sizing
        }
    }
    pub fn IsOutOfFlow(&self) -> bool {
        !self.node.GetLayoutBox().is_null() && self.node.IsOutOfFlowPositioned()
    }

    // cpp: layoutng/internal/grid_item.h:197-234
    pub fn GetTrackSpanProperties(
        &self,
        direction: GridTrackSizingDirection,
    ) -> &TrackSpanProperties {
        if direction == kForColumns {
            &self.column_span_properties
        } else {
            &self.row_span_properties
        }
    }
    pub fn SetTrackSpanProperty(
        &mut self,
        property: PropertyId,
        direction: GridTrackSizingDirection,
    ) {
        if direction == kForColumns {
            self.column_span_properties.SetProperty(property)
        } else {
            self.row_span_properties.SetProperty(property)
        }
    }
    pub fn IsSpanningFlexibleTrack(&self, direction: GridTrackSizingDirection) -> bool {
        self.GetTrackSpanProperties(direction)
            .HasProperty(PropertyId::kHasFlexibleTrack)
    }
    pub fn IsSpanningIntrinsicTrack(&self, direction: GridTrackSizingDirection) -> bool {
        self.GetTrackSpanProperties(direction)
            .HasProperty(PropertyId::kHasIntrinsicTrack)
    }
    pub fn IsSpanningAutoMinimumTrack(&self, direction: GridTrackSizingDirection) -> bool {
        self.GetTrackSpanProperties(direction)
            .HasProperty(PropertyId::kHasAutoMinimumTrack)
    }
    pub fn IsSpanningFixedMinimumTrack(&self, direction: GridTrackSizingDirection) -> bool {
        self.GetTrackSpanProperties(direction)
            .HasProperty(PropertyId::kHasFixedMinimumTrack)
    }
    pub fn IsSpanningFixedMaximumTrack(&self, direction: GridTrackSizingDirection) -> bool {
        self.GetTrackSpanProperties(direction)
            .HasProperty(PropertyId::kHasFixedMaximumTrack)
    }

    // cpp: layoutng/internal/grid_item.h:236-256
    pub fn RelativeDirectionInSubgrid(
        &self,
        direction: GridTrackSizingDirection,
    ) -> GridTrackSizingDirection {
        debug_assert!(self.IsSubgrid());
        if self.is_parallel_with_root_grid == (direction == kForColumns) {
            kForColumns
        } else {
            kForRows
        }
    }
    pub fn RelativeDirectionFilterInSubgrid(
        &self,
        direction: Option<GridTrackSizingDirection>,
    ) -> Option<GridTrackSizingDirection> {
        debug_assert!(self.IsSubgrid());
        direction.map(|direction| self.RelativeDirectionInSubgrid(direction))
    }

    // cpp: layoutng/internal/grid_item.h:258-269
    pub fn ResetContributionSizes(&mut self) {
        self.contribution_sizes = Member::from_ptr(MakeGarbageCollected(
            GridItemDataVirtualItemContributions::default(),
        ));
    }
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.node);
        visitor.Trace(&self.contribution_sizes);
    }
}

// The C++ HeapVector's inline capacity is an allocation detail.
// cpp: layoutng/internal/grid_item.h:420-422
pub type GridItemDataVector = HeapVector<Member<GridItemData>>;

// cpp: layoutng/internal/grid_item.h:424-469
#[derive(Clone, Copy)]
pub struct GridItemIterator<const IS_CONST: bool> {
    current_index_: u32,
    item_data_: *mut GridItemDataVector,
}

impl<const IS_CONST: bool> GridItemIterator<IS_CONST> {
    pub fn new(item_data: *mut GridItemDataVector, current_index: u32) -> Self {
        debug_assert!(!item_data.is_null());
        debug_assert!(current_index as usize <= unsafe { &*item_data }.len());
        Self {
            current_index_: current_index,
            item_data_: item_data,
        }
    }

    pub fn NotEqual(&self, other: &Self) -> bool {
        self.current_index_ != other.current_index_ || self.item_data_ != other.item_data_
    }

    pub fn Advance(&mut self) -> &mut Self {
        self.current_index_ += 1;
        self
    }

    pub fn AdvancePostfix(&mut self) -> Self {
        let previous = *self;
        self.Advance();
        previous
    }

    pub fn Get(&self) -> *mut GridItemData {
        let data = unsafe { &*self.item_data_ };
        debug_assert!((self.current_index_ as usize) < data.len());
        data[self.current_index_ as usize].Get()
    }

    // cpp: layoutng/internal/grid_item.h:464-464
    pub fn Current(&self) -> &GridItemData {
        unsafe { &*self.Get() }
    }
}

impl GridItemIterator<false> {
    pub fn CurrentMut(&mut self) -> &mut GridItemData {
        unsafe { &mut *self.Get() }
    }
}

// cpp: layoutng/internal/grid_item.h:471-485
pub struct GridItemRange<const IS_CONST: bool> {
    begin_: GridItemIterator<IS_CONST>,
    end_: GridItemIterator<IS_CONST>,
}

impl<const IS_CONST: bool> GridItemRange<IS_CONST> {
    pub fn new(begin: GridItemIterator<IS_CONST>, end: GridItemIterator<IS_CONST>) -> Self {
        Self {
            begin_: begin,
            end_: end,
        }
    }
    pub fn begin(&self) -> GridItemIterator<IS_CONST> {
        self.begin_
    }
    pub fn end(&self) -> GridItemIterator<IS_CONST> {
        self.end_
    }
}

// cpp: layoutng/internal/grid_item.h:420-422,487-492,563-575
#[derive(Default)]
pub struct GridItems {
    first_subgridded_item_index_: u32,
    has_stacking_axis_alignment_: bool,
    item_data_: GridItemDataVector,
}

impl GridItems {
    // Non-inline implementation access for the owning layoutng_grid package.
    pub fn GridPackageItemsMut(&mut self) -> &mut GridItemDataVector {
        &mut self.item_data_
    }
    // cpp: layoutng/internal/grid_item.h:494-518
    pub fn begin(&mut self) -> GridItemIterator<false> {
        GridItemIterator::new(&mut self.item_data_, 0)
    }
    pub fn end(&mut self) -> GridItemIterator<false> {
        GridItemIterator::new(&mut self.item_data_, self.first_subgridded_item_index_)
    }
    pub fn IncludeSubgriddedItems(&mut self) -> GridItemRange<false> {
        let end = GridItemIterator::new(&mut self.item_data_, self.item_data_.len() as u32);
        GridItemRange::new(self.begin(), end)
    }
    pub fn ItemsSubgriddedToParent(&mut self) -> GridItemRange<false> {
        GridItemRange::new(
            GridItemIterator::new(&mut self.item_data_, self.first_subgridded_item_index_),
            GridItemIterator::new(&mut self.item_data_, self.item_data_.len() as u32),
        )
    }
    pub fn begin_const(&self) -> GridItemIterator<true> {
        GridItemIterator::new(&self.item_data_ as *const _ as *mut _, 0)
    }
    pub fn end_const(&self) -> GridItemIterator<true> {
        GridItemIterator::new(
            &self.item_data_ as *const _ as *mut _,
            self.first_subgridded_item_index_,
        )
    }
    pub fn IncludeSubgriddedItemsConst(&self) -> GridItemRange<true> {
        GridItemRange::new(
            self.begin_const(),
            GridItemIterator::new(
                &self.item_data_ as *const _ as *mut _,
                self.item_data_.len() as u32,
            ),
        )
    }
    pub fn ItemsSubgriddedToParentConst(&self) -> GridItemRange<true> {
        GridItemRange::new(
            self.end_const(),
            GridItemIterator::new(
                &self.item_data_ as *const _ as *mut _,
                self.item_data_.len() as u32,
            ),
        )
    }

    // cpp: layoutng/internal/grid_item.h:520-524
    pub fn IsEmpty(&self) -> bool {
        self.item_data_.is_empty()
    }
    pub fn Size(&self) -> u32 {
        self.item_data_.len() as u32
    }
    pub fn Append(&mut self, other: *mut GridItems) {
        unsafe { GridItemsAppendProvider(self, other) }
    }
    pub fn SortByOrderProperty(&mut self) {
        unsafe { GridItemsSortByOrderPropertyProvider(self) }
    }

    // cpp: layoutng/internal/grid_item.h:526-535
    pub fn AppendItem(&mut self, new_item_data: *mut GridItemData) {
        assert!(!new_item_data.is_null());
        if !unsafe { &*new_item_data }.is_subgridded_to_parent_grid {
            debug_assert_eq!(
                self.first_subgridded_item_index_ as usize,
                self.item_data_.len()
            );
            self.first_subgridded_item_index_ += 1;
        }
        self.item_data_.push(Member::from_ptr(new_item_data));
    }

    // cpp: layoutng/internal/grid_item.h:537-552
    pub fn At(&mut self, index: u32) -> &mut GridItemData {
        debug_assert!((index as usize) < self.item_data_.len());
        let item = self.item_data_[index as usize].Get();
        debug_assert!(!item.is_null());
        unsafe { &mut *item }
    }
    pub fn AtConst(&self, index: u32) -> &GridItemData {
        debug_assert!((index as usize) < self.item_data_.len());
        let item = self.item_data_[index as usize].Get();
        debug_assert!(!item.is_null());
        unsafe { &*item }
    }
    pub fn GetMember(&self, index: u32) -> &Member<GridItemData> {
        debug_assert!((index as usize) < self.item_data_.len());
        &self.item_data_[index as usize]
    }

    // cpp: layoutng/internal/grid_item.h:554-561
    pub fn ReserveInitialCapacity(&mut self, capacity: u32) {
        let additional = (capacity as usize).saturating_sub(self.item_data_.len());
        self.item_data_.reserve(additional);
    }
    pub fn SetHasStackingAxisAlignment(&mut self) {
        self.has_stacking_axis_alignment_ = true;
    }
    pub fn HasStackingAxisAlignment(&self) -> bool {
        self.has_stacking_axis_alignment_
    }
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.item_data_);
    }
}
