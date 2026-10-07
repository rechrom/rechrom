use foundation::{
    DowncastFrom, EBreakBetween, HeapVector, LayoutUnit, Member, Traceable, Vector, Visitor,
};
use layoutng_assembly::break_token_algorithm_data::{
    BreakTokenAlgorithmData, BreakTokenAlgorithmDataType, BreakTokenAlgorithmDataVirtual,
};
use layoutng_assembly::internal::gap::gap_geometry::GapGeometry;
use layoutng_assembly::internal::grid_item::GridItems;
use layoutng_assembly::internal::grid_layout_data::GridLayoutSubtree;
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;

// cpp: layoutng_grid/grid_break_token_data.h:25-44
#[derive(Clone)]
pub struct GridItemPlacementData {
    pub offset: LogicalOffset,
    pub relative_offset: Option<LogicalOffset>,
    pub has_descendant_that_depends_on_percentage_block_size: bool,
}
impl GridItemPlacementData {
    pub fn new(
        offset: LogicalOffset,
        has_descendant_that_depends_on_percentage_block_size: bool,
        relative_offset: Option<LogicalOffset>,
    ) -> Self {
        Self {
            offset,
            relative_offset,
            has_descendant_that_depends_on_percentage_block_size,
        }
    }
}

// cpp: layoutng_grid/grid_break_token_data.h:46-135
#[repr(C)]
pub struct GridBreakTokenData {
    pub base: BreakTokenAlgorithmData,
    pub grid_items: Member<GridItems>,
    pub grid_layout_subtree: Member<GridLayoutSubtree>,
    pub intrinsic_block_size: LayoutUnit,
    pub offset_in_stitched_container: LayoutUnit,
    pub grid_items_placement_data: Vector<GridItemPlacementData>,
    pub row_offset_adjustments: Vector<LayoutUnit>,
    pub row_break_between: Vector<EBreakBetween>,
    pub oof_children: HeapVector<Member<LayoutBox>>,
    pub full_gap_geometry: Member<GapGeometry>,
    pub track_idx_to_set_idx: Vector<u32>,
    pub column_gaps_segment_ranges_start_indices: Vector<u32>,
    pub cumulative_gap_offset_adjustment: LayoutUnit,
    pub first_unprocessed_row_gap_idx: u32,
}
impl GridBreakTokenData {
    // cpp: layoutng_grid/grid_break_token_data.h:47-78
    pub fn new(
        grid_items: *mut GridItems,
        grid_layout_subtree: *const GridLayoutSubtree,
        intrinsic_block_size: LayoutUnit,
        offset_in_stitched_container: LayoutUnit,
        grid_items_placement_data: &Vector<GridItemPlacementData>,
        row_offset_adjustments: &Vector<LayoutUnit>,
        row_break_between: &Vector<EBreakBetween>,
        oof_children: &HeapVector<Member<LayoutBox>>,
        full_gap_geometry: *const GapGeometry,
        track_idx_to_set_idx: &mut Vector<u32>,
        column_gaps_segment_ranges_start_indices: &mut Vector<u32>,
        cumulative_gap_offset_adjustment: LayoutUnit,
        first_unprocessed_row_gap_idx: u32,
    ) -> Self {
        Self {
            base: BreakTokenAlgorithmData::new(BreakTokenAlgorithmDataType::kGridData),
            grid_items: Member::from_ptr(grid_items),
            grid_layout_subtree: Member::from_ptr(grid_layout_subtree.cast_mut()),
            intrinsic_block_size,
            offset_in_stitched_container,
            grid_items_placement_data: grid_items_placement_data.clone(),
            row_offset_adjustments: row_offset_adjustments.clone(),
            row_break_between: row_break_between.clone(),
            oof_children: oof_children.clone(),
            full_gap_geometry: Member::from_ptr(full_gap_geometry.cast_mut()),
            track_idx_to_set_idx: track_idx_to_set_idx.clone(),
            column_gaps_segment_ranges_start_indices: column_gaps_segment_ranges_start_indices
                .clone(),
            cumulative_gap_offset_adjustment,
            first_unprocessed_row_gap_idx,
        }
    }
    // cpp: layoutng_grid/grid_break_token_data.h:80-86
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.grid_items);
        visitor.Trace(&self.grid_layout_subtree);
        visitor.Trace(&self.oof_children);
        visitor.Trace(&self.full_gap_geometry);
        BreakTokenAlgorithmDataVirtual::Trace(&self.base, visitor);
    }
    // cpp: layoutng_grid/grid_break_token_data.h:88-91
    pub fn GetTotalRowGapCount(&self) -> u32 {
        let geometry = self.full_gap_geometry.Get();
        assert!(!geometry.is_null());
        unsafe { &*geometry }.GetMainGaps().len() as u32
    }
    // cpp: layoutng_grid/grid_break_token_data.h:93-96
    pub fn GetFirstUnprocessedRowGapIndex(&self, _line_index: Option<u32>) -> u32 {
        self.first_unprocessed_row_gap_idx
    }
}
const _: () = assert!(std::mem::offset_of!(GridBreakTokenData, base) == 0);
impl BreakTokenAlgorithmDataVirtual for GridBreakTokenData {
    fn base(&self) -> &BreakTokenAlgorithmData {
        &self.base
    }
    fn GetTotalRowGapCount(&self) -> u32 {
        GridBreakTokenData::GetTotalRowGapCount(self)
    }
    fn GetFirstUnprocessedRowGapIndex(&self, line_index: Option<u32>) -> u32 {
        GridBreakTokenData::GetFirstUnprocessedRowGapIndex(self, line_index)
    }
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        GridBreakTokenData::Trace(self, visitor);
    }
}
impl DowncastFrom<BreakTokenAlgorithmData> for GridBreakTokenData {
    // cpp: layoutng_grid/grid_break_token_data.h:137-142
    fn AllowFrom(data: &BreakTokenAlgorithmData) -> bool {
        data.IsGridType()
    }
}
impl Traceable for GridBreakTokenData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        GridBreakTokenData::Trace(self, visitor);
    }
}
